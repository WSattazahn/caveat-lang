// The runtime workflow does what the round-1 carry promises: only the jobs
// it names can carry, each runs nothing but its carry step when it does, a
// failed decision runs everything, and nothing that reads the ledger's events
// is carried. Read line by line, with no YAML dependency.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { CARRIED, CARRIED_NAMES, CARRY_JOB, CARRY_NOTICE, CARRY_STEP, SCOPE_JOB } from './reuse.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const read = file => readFileSync(path.join(root, file), 'utf8');
const workflow = read('.github/workflows/runtime.yml');
// Only a successful regression scope, and only in the attempt that decided:
// a re-run of failed jobs keeps the scope's outputs and must run in full.
const DECIDED_HERE = 'needs.changes.outputs.decided_attempt == github.run_attempt';
const REUSED = `      REUSED: \${{ needs.changes.result == 'success' && needs.changes.outputs.reuse == 'true' && ${DECIDED_HERE} }}`;
const HIDE = '.github/ci/hide-ledger-events.sh';

// The jobs, each with its own lines; comment and blank lines left out.
function jobsOf(text) {
  const lines = text.split('\n');
  const jobs = new Map();
  let job = null;
  for (const line of lines.slice(lines.indexOf('jobs:') + 1)) {
    if (!line.trim() || /^\s*#/.test(line)) continue;
    const key = /^ {2}([A-Za-z_][\w-]*):$/.exec(line);
    if (key) jobs.set(key[1], job = { id: key[1], lines: [] });
    else if (job) job.lines.push(line);
  }
  return jobs;
}
const jobs = jobsOf(workflow);
const field = (job, key) => job.lines.find(line => line.startsWith(`    ${key}: `))?.slice(key.length + 6);
const block = (job, key) => {
  const start = job.lines.indexOf(`    ${key}:`);
  if (start < 0) return [];
  const end = job.lines.findIndex((line, index) => index > start && !line.startsWith('      '));
  return job.lines.slice(start + 1, end < 0 ? undefined : end).map(line => line.trim());
};
// The steps, each with its lines and its name, id, if, uses and run.
function stepsOf(job) {
  const steps = [];
  for (const line of job.lines.slice(job.lines.indexOf('    steps:') + 1)) {
    if (!line.startsWith('      ')) break;
    if (line.startsWith('      - ')) steps.push({ lines: [`        ${line.slice(8)}`] });
    else steps.at(-1).lines.push(line);
  }
  for (const step of steps) {
    for (const key of ['name', 'id', 'if', 'uses', 'run', 'continue-on-error', 'timeout-minutes']) {
      step[key] = step.lines.find(line => line.startsWith(`        ${key}: `))?.slice(key.length + 10);
    }
    step.text = step.lines.join('\n');
  }
  return steps;
}
const job = id => {
  assert.ok(jobs.has(id), `runtime.yml has no job ${id}`);
  return jobs.get(id);
};
const needsOf = j => (field(j, 'needs') ?? '').replace(/^\[|\]$/g, '').split(',').map(name => name.trim()).filter(Boolean);

test('exactly the carried jobs can carry, under the check names the decision checks', () => {
  const gating = [...jobs.values()].filter(j => j.lines.some(line => line.includes('REUSED'))).map(j => j.id).sort();
  assert.deepEqual(gating, CARRIED.map(entry => entry.job).sort());
  for (const { job: id, names } of CARRIED) {
    const j = job(id);
    const name = field(j, 'name');
    const browsers = /^ +browser: \[(.+)\]$/.exec(j.lines.find(line => /^ +browser: \[/.test(line)) ?? '')?.[1].split(',').map(b => b.trim());
    assert.deepEqual(browsers ? browsers.map(b => name.replace('${{ matrix.browser }}', b)) : [name], names, id);
  }
  assert.equal(CARRIED_NAMES.length, new Set(CARRIED_NAMES).size);
  // The security audit reads today's advisories and audits the kit the
  // Evidence games job packs; core semantics replays the ledger.
  for (const id of ['core', 'kit-security', 'glowcap-page', 'changes']) assert.ok(!CARRIED.some(entry => entry.job === id), id);
});

test('a carried job waits for the regression scope; when the scope fails, the jobs that did not need it before still run', () => {
  // native, native-reactive and compatibility did not wait for the
  // regression scope before, and lean-conformance and reproducible waited
  // only for core: they must still run when it fails. last-beacon and
  // light-the-way need its games output, and were skipped then before too.
  const expected = {
    native: '${{ !cancelled() }}',
    'native-reactive': '${{ !cancelled() }}',
    compatibility: '${{ !cancelled() }}',
    'lean-conformance': "${{ !cancelled() && needs.core.result == 'success' }}",
    reproducible: "${{ !cancelled() && needs.core.result == 'success' }}",
    'last-beacon': "needs.changes.outputs.games == 'true'",
    'light-the-way': undefined,
  };
  assert.deepEqual(Object.keys(expected).sort(), CARRIED.map(entry => entry.job).sort());
  for (const [id, condition] of Object.entries(expected)) {
    const j = job(id);
    assert.ok(needsOf(j).includes('changes'), `${id} must need changes`);
    assert.equal(field(j, 'if'), condition, id);
    assert.ok(j.lines.includes(REUSED), `${id} must set REUSED only from a successful regression scope, in the attempt that decided`);
  }
});

test('a carrying job runs only its carry step, and a full run every other step', () => {
  const carrySteps = new Set();
  for (const { job: id } of CARRIED) {
    const [carry, ...rest] = stepsOf(job(id));
    assert.equal(carry.name, CARRY_STEP, id);
    assert.equal(carry.if, "env.REUSED == 'true'", id);
    assert.match(carry.text, /"\$GITHUB_EVENT_NAME" != pull_request \|\|/, `${id}: a carry outside a pull request run must fail`);
    assert.match(carry.text, /exit 1/);
    // The weekly game QA retry count tells a carried check run by this notice.
    assert.ok(carry.text.includes(`echo "::notice title=${CARRY_NOTICE}::`), `${id}: the carry step must leave its notice`);
    carrySteps.add(carry.text);
    assert.ok(rest.length > 1, id);
    for (const step of rest) {
      const label = `${id}: ${step.name ?? step.uses ?? step.run}`;
      assert.ok(step.if, `${label} has no if`);
      assert.ok(step.if === "env.REUSED != 'true'" || step.if.endsWith(" && env.REUSED != 'true'"), `${label}: if ${step.if}`);
      assert.ok(!/\|\||\bREUSED\b.*\bREUSED\b/.test(step.if), `${label}: if ${step.if}`);
    }
    // Each full run takes the ledger's events out of its checkout first.
    const checkout = rest.findIndex(step => step.uses?.startsWith('actions/checkout@'));
    assert.ok(checkout >= 0, `${id} has no checkout`);
    assert.equal(rest[checkout + 1]?.run, HIDE, `${id}: the step after the checkout must take the ledger's events out`);
    assert.equal(rest.filter(step => step.uses?.startsWith('actions/checkout@')).length, 1, `${id}: one checkout`);
    assert.ok(!job(id).lines.some(line => /ledger\/|bench\/carmack|scripts\/done-check|test:ledger|test:done-check/.test(line)),
      `${id} must not name the ledger or its readers`);
  }
  assert.equal(carrySteps.size, 1, 'every carry step must be the same step');
});

test('the browser build that carried jobs use must match a rebuild made with the ledger events out of reach', () => {
  // Core semantics builds it with the events in reach and does not hide them.
  const users = CARRIED.map(entry => entry.job)
    .filter(id => stepsOf(job(id)).some(step => step.uses?.startsWith('actions/download-artifact@') && step.lines.includes('          name: browser-dist')));
  assert.deepEqual(users.sort(), ['last-beacon', 'lean-conformance', 'light-the-way', 'reproducible']);
  const steps = stepsOf(job('reproducible'));
  const build = steps.findIndex(step => step.run === 'npm run build');
  assert.ok(build > steps.findIndex(step => step.run === HIDE), 'the rebuild must come after the hide');
  const compare = steps.find(step => step.name === 'Require identical bytes');
  assert.ok(compare?.lines.includes(`          diff -r --exclude='.*' "$first" dist`), 'every file of the build must be compared');
});

test('the regression scope decides the carry with read-only access, and any failure runs everything', () => {
  const scope = job('changes');
  assert.equal(field(scope, 'name'), SCOPE_JOB);
  assert.deepEqual(block(scope, 'permissions'), ['actions: read', 'checks: read', 'contents: read']);
  const outputs = block(scope, 'outputs');
  for (const [output, from] of [['reuse', 'reuse'], ['round1_url', 'url'], ['round1_tree', 'tree'], ['round1_finished', 'finished'],
    ['carried_names', 'names'], ['decided_attempt', 'attempt']]) {
    assert.ok(outputs.includes(`${output}: \${{ steps.reuse.outputs.${from} }}`), output);
  }
  const steps = stepsOf(scope);
  const decision = steps.find(step => step.id === 'reuse');
  assert.ok(decision, 'no reuse step');
  assert.equal(decision.if, "github.event_name == 'pull_request'");
  assert.equal(decision['continue-on-error'], 'true');
  assert.equal(decision['timeout-minutes'], '2');
  assert.equal(decision.run, 'node scripts/ci-reuse/reuse.mjs');
  assert.ok(steps.indexOf(decision) > steps.findIndex(step => step.id === 'scope'));
  // File names are printed between stop-commands, so none reads as a record.
  const listing = steps.find(step => step.id === 'scope').text;
  assert.match(listing, /echo "::stop-commands::\$stop"\n\s+printf '%s\\n' "\$files"\n\s+echo "::\$stop::"/);
  assert.equal((listing.match(/printf '%s\\n' "\$files"\n/g) ?? []).length, 1);
});

test('the round-1 carry job appears only in a carrying run, and refuses outside a pull request', () => {
  const marker = [...jobs.values()].filter(j => field(j, 'name') === CARRY_JOB);
  assert.equal(marker.length, 1);
  const [j] = marker;
  assert.deepEqual(needsOf(j), ['changes']);
  assert.equal(field(j, 'if'), `needs.changes.outputs.reuse == 'true' && ${DECIDED_HERE}`);
  const [step] = stepsOf(j);
  assert.match(step.text, /"\$GITHUB_EVENT_NAME" != pull_request \|\|/);
  assert.match(step.text, /exit 1/);
});

test('everything that reads the ledger events still runs on the push that adds them', () => {
  const core = job('core');
  assert.equal(field(core, 'needs'), undefined, 'core semantics starts at once');
  assert.equal(field(core, 'if'), undefined);
  const runs = stepsOf(core).map(step => step.run);
  assert.ok(runs.includes('npm run test:ledger') && runs.includes('npm run test:ci-reuse'));
  assert.match(JSON.parse(read('package.json')).scripts['test:ledger'], /node ledger\/replay-check\.mjs/);
  assert.match(read('.github/workflows/carmack.yml'), /\n {4}paths:\n(?: {6}- '.+'\n)*? {6}- 'ledger\/events\.jsonl'\n/);
});

test('only known code names the ledger events file', () => {
  const grep = (pattern, ...paths) => {
    const result = spawnSync('git', ['grep', '-l', '-E', pattern, '--', ...paths], { cwd: root, encoding: 'utf8' });
    assert.ok(result.status === 0 || result.status === 1, result.stderr);
    return result.stdout.split('\n').filter(Boolean).sort();
  };
  // A new name here may be a new reader: if a carried job can reach it, take
  // that job out of CARRIED (README.md).
  assert.deepEqual(grep("ledger/events\\.jsonl|ledger['\"]?[[:space:]]*,[[:space:]]*['\"]events\\.jsonl", '.', ':!*.md', ':!docs/**', ':!scripts/ci-reuse/**'), [
    HIDE, // takes the file out of the carried jobs' checkouts
    '.github/workflows/carmack.yml', // carmack runs again on a ledger push
    'bench/carmack/workloads.mjs', // carmack's ledger workload
    'experiments/token-cost/measure.mjs', // run by hand only
    'scripts/done-check/done.cav', // the done check: run by hand, tested in core
    'scripts/done-check/done.mjs',
    'scripts/done-check/done.test.mjs',
  ]);
  assert.deepEqual(grep('events\\.jsonl', 'ledger', ':!*.md'), ['ledger/replay-check.mjs']);
});

test('the hiding script puts the ledger events out of reach and leaves the revision and git status as they were', () => {
  const dir = mkdtempSync(path.join(tmpdir(), 'ci-reuse-hide-'));
  try {
    const run = (...args) => spawnSync('git', args, { cwd: dir, encoding: 'utf8' });
    const git = (...args) => {
      const result = run(...args);
      assert.equal(result.status, 0, result.stderr);
      return result.stdout;
    };
    git('init', '-q');
    mkdirSync(path.join(dir, 'ledger'));
    const events = '{"event":"merge"}\n';
    writeFileSync(path.join(dir, 'ledger', 'events.jsonl'), events);
    writeFileSync(path.join(dir, 'ledger', 'README.md'), 'kept\n');
    git('add', '-A');
    git('-c', 'user.email=ci@example.test', '-c', 'user.name=ci', '-c', 'commit.gpgsign=false', 'commit', '-q', '-m', 'x');
    const ids = () => git('rev-parse', 'HEAD', 'HEAD^{tree}', ':ledger/events.jsonl');
    const before = ids();
    const hidden = spawnSync('bash', [path.join(root, HIDE)], { cwd: dir, encoding: 'utf8' });
    assert.equal(hidden.status, 0, hidden.stderr);
    // Out of the checkout: reading the file fails, and git status stays clean.
    assert.equal(existsSync(path.join(dir, 'ledger', 'events.jsonl')), false);
    assert.equal(readFileSync(path.join(dir, 'ledger', 'README.md'), 'utf8'), 'kept\n');
    assert.equal(git('status', '--porcelain', '--untracked-files=no'), '');
    assert.equal(git('status', '--short'), '');
    assert.equal(run('diff-index', '--quiet', 'HEAD', '--').status, 0);
    // Out of git's reach: each read through git gets the placeholder line.
    const placeholder = `ledger/events.jsonl is hidden from this job: ${HIDE}\n`;
    assert.equal(git('show', 'HEAD:ledger/events.jsonl'), placeholder);
    assert.equal(git('cat-file', 'blob', ':ledger/events.jsonl'), placeholder);
    const archive = run('archive', '--format=tar', 'HEAD');
    assert.equal(archive.status, 0, archive.stderr);
    assert.ok(archive.stdout.includes(placeholder) && !archive.stdout.includes(events), 'git archive must give the placeholder');
    for (const args of [['--cached', 'merge'], ['merge', 'HEAD']]) assert.equal(run('grep', '-q', ...args).status, 1, `git grep ${args.join(' ')}`);
    // The commit, its tree and the index name what they named before.
    assert.equal(ids(), before);
    assert.equal(git('--no-replace-objects', 'show', 'HEAD:ledger/events.jsonl'), events);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
