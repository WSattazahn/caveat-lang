// The round-1 carry's decision: what it accepts, every refusal, and that a
// failed decision runs everything.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {
  appendOnly, CARRIED_NAMES, CARRY_STEP, DECISION_TITLE, decide, escapeData, formatRecord, localGit, main, MAX_AGE_MINUTES,
  parseRecord, RECORD_TITLE, SCOPE_JOB, WORKFLOW,
} from './reuse.mjs';

const EVENTS = 'ledger/events.jsonl';
const temporary = [];
test.after(() => { for (const dir of temporary) rmSync(dir, { recursive: true, force: true }); });

function repository() {
  const dir = mkdtempSync(path.join(tmpdir(), 'ci-reuse-'));
  temporary.push(dir);
  const git = (...args) => {
    const result = spawnSync('git', args, { cwd: dir, encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.trim();
  };
  git('init', '-q');
  git('config', 'user.email', 'ci@example.test');
  git('config', 'user.name', 'ci');
  git('config', 'commit.gpgsign', 'false');
  const write = (file, text) => {
    mkdirSync(path.dirname(path.join(dir, file)), { recursive: true });
    writeFileSync(path.join(dir, file), text);
  };
  const commit = message => {
    git('add', '-A');
    git('commit', '-q', '--allow-empty', '-m', message);
    return git('rev-parse', 'HEAD');
  };
  return { dir, git, write, commit, local: localGit(dir) };
}

// main, a pull request's code change, then its approval commit, which only
// adds two lines to the ledger's events.
function pullRequest({ approval = '{"event":"ci_run"}\n{"event":"approve"}\n' } = {}) {
  const repo = repository();
  repo.write(EVENTS, '{"event":"merge"}\n');
  repo.write('runtime/src/lib.rs', 'fn a() {}\n');
  repo.write('kit/package.json', '{"version":"1.4.1"}\n');
  repo.git('checkout', '-q', '-b', 'main');
  const base = repo.commit('main');
  repo.git('checkout', '-q', '-b', 'feature');
  repo.write('runtime/src/lib.rs', 'fn b() {}\n');
  const before = repo.commit('the change');
  repo.write(EVENTS, `{"event":"merge"}\n${approval}`);
  const head = repo.commit('ledger: record green CI and approval');
  return { ...repo, base, before, head };
}

const NOW = Date.parse('2026-11-13T02:00:00Z');
const ENV = { GITHUB_EVENT_NAME: 'pull_request', GITHUB_REPOSITORY: 'o/r', GITHUB_RUN_ID: '200', GAMES: 'true', LEGACY_DOOR: 'true' };
const eventOf = pr => ({
  action: 'synchronize', before: pr.before,
  pull_request: { head: { sha: pr.head, ref: 'feature', repo: { full_name: 'o/r' } }, base: { sha: pr.base, ref: 'main' } },
});
const fullJob = (name, id) => ({ id, name, conclusion: 'success', steps: [{ name: CARRY_STEP, conclusion: 'skipped' }, { name: 'the work', conclusion: 'success' }] });

// GitHub as round 1 left it: a green run of the previous head that recorded
// its tree, and ran every carried job in full.
function github(pr, { round1 = {}, runs, jobs, annotations } = {}) {
  const run = {
    id: 100, path: WORKFLOW, head_sha: pr.before, head_branch: 'feature', head_repository: { full_name: 'o/r' },
    status: 'completed', conclusion: 'success', updated_at: '2026-11-13T01:55:00Z',
    html_url: 'https://github.com/o/r/actions/runs/100', run_attempt: 1, ...round1,
  };
  const record = formatRecord({ tree: pr.local.tree(pr.before), head: pr.before, games: 'true', legacy_door: 'true', reuse: false });
  const calls = [];
  const api = async url => {
    calls.push(url);
    if (url === `/repos/o/r/actions/workflows/runtime.yml/runs?event=pull_request&head_sha=${pr.before}&per_page=100`) return { workflow_runs: runs ?? [run] };
    if (url === '/repos/o/r/actions/runs/100/jobs?filter=latest&per_page=100') {
      return { jobs: jobs ?? [{ id: 7, name: SCOPE_JOB, conclusion: 'success', steps: [] }, ...CARRIED_NAMES.map((name, index) => fullJob(name, 10 + index))] };
    }
    if (url === '/repos/o/r/check-runs/7/annotations?per_page=100') {
      return annotations ?? [{ annotation_level: 'notice', title: 'Round-1 carry', message: 'Running everything: new code.' },
        { annotation_level: 'notice', title: RECORD_TITLE, message: record }];
    }
    throw new Error(`unexpected request ${url}`);
  };
  return { api, calls, record };
}

const decideFor = (pr, { env = {}, event = {}, ...state } = {}) => {
  const { api, calls } = github(pr, state);
  return decide({ env: { ...ENV, ...env }, event: { ...eventOf(pr), ...event }, git: pr.local, api, now: NOW }).then(decision => ({ decision, calls }));
};

test('the tested-tree record reads back exactly and nothing else reads as one', () => {
  const tree = 'a'.repeat(40);
  const head = 'b'.repeat(40);
  const text = formatRecord({ tree, head, games: 'true', legacy_door: 'false', reuse: false });
  assert.equal(text, `tree=${tree} head=${head} games=true legacy_door=false reuse=false`);
  assert.deepEqual(parseRecord(text), { tree, head, games: 'true', legacy_door: 'false', reuse: false });
  assert.equal(parseRecord(text.replace('reuse=false', 'reuse=true')).reuse, true);
  for (const bad of [`${text} `, ` ${text}`, text.replace(tree, tree.toUpperCase()), text.replace(tree, tree.slice(1)),
    text.replace('games=true', 'games=yes'), text.replace(' head=', '  head='), `${text}\n${text}`, '', null, undefined, 42]) {
    assert.equal(parseRecord(bad), null, `${JSON.stringify(bad)} read as a record`);
  }
  assert.equal(formatRecord({ tree, head: undefined, games: 'true', legacy_door: 'false', reuse: false }), null);
  assert.equal(formatRecord({ tree, head, games: '', legacy_door: 'false', reuse: false }), null);
  assert.equal(escapeData('50% done\r\nnext'), '50%25 done%0D%0Anext');
});

test('only lines added at the end of the ledger events, each with its newline, make an append', () => {
  const pr = pullRequest();
  assert.equal(appendOnly(pr.local, pr.before, pr.head), null);
  const after = change => {
    pr.git('checkout', '-q', '--detach', pr.before);
    change();
    return pr.commit('another push');
  };
  const cases = [
    ['nothing', () => {}, /the tree did not change/],
    ['another file too', () => { pr.write(EVENTS, '{"event":"merge"}\n{"event":"x"}\n'); pr.write('runtime/src/lib.rs', 'fn c() {}\n'); },
      /files other than ledger\/events\.jsonl changed since the previous head \(runtime\/src\/lib\.rs\)/],
    ['a rewritten line', () => pr.write(EVENTS, '{"event":"Merge"}\n{"event":"x"}\n'), /changed other than by lines added at its end/],
    ['a removed line', () => pr.write(EVENTS, ''), /changed other than by lines added at its end/],
    ['a line without its newline', () => pr.write(EVENTS, '{"event":"merge"}\n{"event":"x"}'), /has a line without its newline/],
    ['the file made executable', () => { pr.write(EVENTS, '{"event":"merge"}\n{"event":"x"}\n'); chmodSync(path.join(pr.dir, EVENTS), 0o755); },
      /was not edited in place \(M, mode 100644 to 100755\)/],
    ['the file deleted', () => rmSync(path.join(pr.dir, EVENTS)), /was not edited in place \(D/],
    ['the file renamed', () => pr.git('mv', EVENTS, 'ledger/events2.jsonl'), /files other than ledger\/events\.jsonl changed .*ledger\/events2\.jsonl/],
  ];
  for (const [what, change, reason] of cases) assert.match(appendOnly(pr.local, pr.before, after(change)) ?? '', reason, what);
  assert.match(appendOnly(pr.local, 'f'.repeat(40), pr.head), /the previous head ffffffffffff is not in this checkout/);

  const unterminated = pullRequest();
  unterminated.git('checkout', '-q', unterminated.before);
  unterminated.write(EVENTS, '{"event":"merge"}');
  const before = unterminated.commit('no final newline');
  unterminated.write(EVENTS, '{"event":"merge"}\n{"event":"x"}\n');
  assert.match(appendOnly(unterminated.local, before, unterminated.commit('then a line')), /has a line without its newline/);
});

test('carries round 1 when the push only adds ledger lines and round 1 ran green on the same tree', async () => {
  const pr = pullRequest();
  const { decision, calls } = await decideFor(pr);
  assert.equal(decision.reuse, true, decision.reason);
  assert.equal(decision.url, 'https://github.com/o/r/actions/runs/100/attempts/1');
  assert.equal(decision.tree, pr.local.tree(pr.before));
  assert.equal(decision.finished, '2026-11-13T01:55:00Z');
  assert.match(decision.reason, /^round 1 \(https:\/\/github\.com\/o\/r\/actions\/runs\/100\/attempts\/1\) ran these jobs to success on tree [0-9a-f]{40}; this push only adds lines at the end of ledger\/events\.jsonl$/);
  assert.equal(calls.length, 3);
  // Only the newest earlier run of this pull request's previous head counts.
  const runs = [
    { id: 90, conclusion: 'success' }, { id: 100, conclusion: 'success' }, { id: 250, conclusion: 'failure' },
    { id: 120, conclusion: 'failure', head_branch: 'other' }, { id: 130, conclusion: 'failure', head_repository: { full_name: 'fork/r' } },
    { id: 140, conclusion: 'failure', path: '.github/workflows/carmack.yml' },
  ].map(run => ({ path: WORKFLOW, head_sha: pr.before, head_branch: 'feature', head_repository: { full_name: 'o/r' }, status: 'completed',
    updated_at: '2026-11-13T01:55:00Z', html_url: `https://github.com/o/r/actions/runs/${run.id}`, run_attempt: 2, ...run }));
  const newest = await decideFor(pr, { runs });
  assert.equal(newest.decision.reuse, true, newest.decision.reason);
  assert.equal(newest.decision.url, 'https://github.com/o/r/actions/runs/100/attempts/2');
  const failedLast = await decideFor(pr, { runs: [...runs, { ...runs[1], id: 110, conclusion: 'failure', html_url: 'https://github.com/o/r/actions/runs/110' }] });
  assert.equal(failedLast.decision.reason, 'round 1 (https://github.com/o/r/actions/runs/110/attempts/2) is completed, failure, not a success');
});

test('refuses before asking GitHub anything unless the push only adds ledger lines to a pull request of this repository', async () => {
  const pr = pullRequest();
  const cases = [
    [{ env: { ROUND1_REUSE: 'off' } }, 'turned off by the repository variable ROUND1_REUSE'],
    [{ env: { GITHUB_EVENT_NAME: 'push' } }, 'a push run, not a pull request run'],
    [{ env: { GITHUB_EVENT_NAME: 'workflow_dispatch' } }, 'a workflow_dispatch run, not a pull request run'],
    [{ event: { action: 'opened' } }, 'the pull request was opened, not pushed to'],
    [{ event: { action: 'reopened' } }, 'the pull request was reopened, not pushed to'],
    [{ event: { pull_request: { ...eventOf(pr).pull_request, head: { sha: pr.head, ref: 'feature', repo: { full_name: 'fork/r' } } } } },
      'the pull request comes from another repository'],
    [{ event: { before: '0'.repeat(39) } }, 'the push names no previous head'],
    [{ event: { pull_request: { ...eventOf(pr).pull_request, base: { ref: 'main' } } } }, 'the pull request names no head or base revision'],
  ];
  for (const [state, reason] of cases) {
    const { decision, calls } = await decideFor(pr, state);
    assert.deepEqual(decision, { reuse: false, reason });
    assert.deepEqual(calls, []);
  }
  // A release pull request: its version or release records change.
  for (const [file, text] of [['kit/package.json', '{"version":"1.5.0"}\n'], ['docs/releases/v1.5.0.md', '# 1.5.0\n']]) {
    const release = pullRequest();
    release.git('checkout', '-q', release.before);
    release.write(file, text);
    release.before = release.commit('the release');
    release.write(EVENTS, '{"event":"merge"}\n{"event":"approve"}\n');
    release.head = release.commit('ledger: approve');
    const { decision, calls } = await decideFor(release);
    assert.equal(decision.reason, `a release pull request (it changes ${file}), so its head runs everything`);
    assert.deepEqual(calls, []);
  }
  // Round 1 itself: new code, so nothing to ask GitHub.
  const code = pullRequest();
  code.write('runtime/src/lib.rs', 'fn d() {}\n');
  code.head = code.commit('more code');
  code.before = code.git('rev-parse', 'HEAD~1');
  const { decision, calls } = await decideFor(code);
  assert.equal(decision.reason, 'files other than ledger/events.jsonl changed since the previous head (runtime/src/lib.rs)');
  assert.deepEqual(calls, []);
});

test('refuses unless round 1 ran green, lately, on the same tree, and ran every carried job in full', async () => {
  const pr = pullRequest();
  const tree = pr.local.tree(pr.before);
  const record = overrides => [{ annotation_level: 'notice', title: RECORD_TITLE,
    message: formatRecord({ tree, head: pr.before, games: 'true', legacy_door: 'true', reuse: false, ...overrides }) }];
  const scope = { id: 7, name: SCOPE_JOB, conclusion: 'success', steps: [] };
  const carried = CARRIED_NAMES.map((name, index) => fullJob(name, 10 + index));
  const url = 'https://github.com/o/r/actions/runs/100/attempts/1';
  const cases = [
    [{ runs: [] }, `no earlier run of this pull request tested the previous head ${pr.before.slice(0, 12)}`],
    [{ round1: { id: 200 } }, `no earlier run of this pull request tested the previous head ${pr.before.slice(0, 12)}`],
    [{ round1: { head_branch: 'other' } }, `no earlier run of this pull request tested the previous head ${pr.before.slice(0, 12)}`],
    [{ round1: { status: 'in_progress', conclusion: null } }, `round 1 (${url}) is in_progress, not a success`],
    [{ round1: { conclusion: 'cancelled' } }, `round 1 (${url}) is completed, cancelled, not a success`],
    [{ round1: { updated_at: new Date(NOW - (MAX_AGE_MINUTES + 1) * 60_000).toISOString() } }, `round 1 (${url}) finished more than ${MAX_AGE_MINUTES} minutes ago`],
    [{ round1: { updated_at: 'yesterday' } }, `round 1 (${url}) finished more than ${MAX_AGE_MINUTES} minutes ago`],
    [{ jobs: carried }, `round 1 (${url}) has no regression scope job`],
    [{ annotations: [] }, `round 1 (${url}) left 0 tested-tree records, not one`],
    [{ annotations: [...record(), ...record()] }, `round 1 (${url}) left 2 tested-tree records, not one`],
    [{ annotations: [{ annotation_level: 'warning', title: RECORD_TITLE, message: record()[0].message }] }, `round 1 (${url}) left 0 tested-tree records, not one`],
    [{ annotations: [{ annotation_level: 'notice', title: RECORD_TITLE, message: 'tree=x' }] }, "round 1's tested-tree record does not read: tree=x"],
    [{ annotations: record({ head: 'c'.repeat(40) }) }, `round 1 recorded head cccccccccccc, not the previous head ${pr.before.slice(0, 12)}`],
    [{ annotations: record({ tree: 'd'.repeat(40) }) }, 'round 1 tested a merge with another main than this head holds (main moved, or the head was behind it)'],
    [{ annotations: record({ reuse: true }) }, 'round 1 itself carried an earlier result'],
    [{ annotations: record({ games: 'false' }) }, 'round 1 decided a different set of browser QA jobs'],
    [{ annotations: record({ legacy_door: 'false' }) }, 'round 1 decided a different set of browser QA jobs'],
    [{ jobs: [scope, ...carried.slice(1)] }, `round 1's ${CARRIED_NAMES[0]} did not run to success`],
    [{ jobs: [scope, ...carried.map((job, index) => (index === 3 ? { ...job, conclusion: 'skipped' } : job))] }, `round 1's ${CARRIED_NAMES[3]} did not run to success`],
    [{ jobs: [scope, ...carried.map((job, index) => (index === 7 ? { ...job, steps: [{ name: CARRY_STEP, conclusion: 'success' }] } : job))] },
      `round 1's ${CARRIED_NAMES[7]} did not run in full`],
    [{ jobs: [scope, ...carried.map((job, index) => (index === 2 ? { ...job, steps: [] } : job))] }, `round 1's ${CARRIED_NAMES[2]} did not run in full`],
  ];
  for (const [state, reason] of cases) assert.deepEqual((await decideFor(pr, state)).decision, { reuse: false, reason });
  // This run's own scope must match round 1's.
  assert.equal((await decideFor(pr, { env: { GAMES: 'false' } })).decision.reason, 'round 1 decided a different set of browser QA jobs');
});

test('main leaves the tested tree, decides, and writes its outputs in one write at the end', async () => {
  const pr = pullRequest();
  const dir = mkdtempSync(path.join(tmpdir(), 'ci-reuse-main-'));
  temporary.push(dir);
  const files = { GITHUB_EVENT_PATH: path.join(dir, 'event.json'), GITHUB_OUTPUT: path.join(dir, 'output'), GITHUB_STEP_SUMMARY: path.join(dir, 'summary') };
  writeFileSync(files.GITHUB_EVENT_PATH, JSON.stringify(eventOf(pr)));
  const run = async api => {
    writeFileSync(files.GITHUB_OUTPUT, '');
    writeFileSync(files.GITHUB_STEP_SUMMARY, '');
    const logs = [];
    const result = await main({ env: { ...ENV, ...files }, now: NOW, git: pr.local, api, log: line => logs.push(line) });
    return { ...result, logs, outputs: readFileSync(files.GITHUB_OUTPUT, 'utf8'), summary: readFileSync(files.GITHUB_STEP_SUMMARY, 'utf8') };
  };
  const headTree = pr.local.tree('HEAD');

  const carried = await run(github(pr).api);
  assert.equal(carried.decision.reuse, true);
  assert.deepEqual(carried.logs, [
    `::notice title=${RECORD_TITLE}::tree=${headTree} head=${pr.head} games=true legacy_door=true reuse=true`,
    `::notice title=${DECISION_TITLE}::Carrying round 1's result: ${carried.decision.reason}.`,
  ]);
  assert.equal(carried.outputs, [
    'reuse=true', 'url=https://github.com/o/r/actions/runs/100/attempts/1', `tree=${pr.local.tree(pr.before)}`,
    'finished=2026-11-13T01:55:00Z', `names=${CARRIED_NAMES.join('; ')}`, '',
  ].join('\n'));
  assert.match(carried.summary, /^### Round-1 carry\n\nCarrying: round 1 /);

  // GitHub failing runs everything, and still leaves the record a later push can carry from.
  const failed = await run(async () => { throw new Error('GitHub answered 503 for /repos/o/r/actions/workflows/runtime.yml/runs'); });
  assert.deepEqual(failed.decision, { reuse: false, reason: 'the decision failed (GitHub answered 503 for /repos/o/r/actions/workflows/runtime.yml/runs)' });
  assert.deepEqual(failed.logs, [
    `::warning title=${DECISION_TITLE}::Running everything: the decision failed (GitHub answered 503 for /repos/o/r/actions/workflows/runtime.yml/runs)`,
    `::notice title=${RECORD_TITLE}::tree=${headTree} head=${pr.head} games=true legacy_door=true reuse=false`,
    `::notice title=${DECISION_TITLE}::Running everything: the decision failed (GitHub answered 503 for /repos/o/r/actions/workflows/runtime.yml/runs).`,
  ]);
  assert.equal(failed.outputs, 'reuse=false\nurl=\ntree=\nfinished=\nnames=\n');

  // An unreadable event leaves no record, so the next push runs everything too.
  writeFileSync(files.GITHUB_EVENT_PATH, '{');
  const unreadable = await run(github(pr).api);
  assert.equal(unreadable.decision.reuse, false);
  assert.match(unreadable.logs[0], /^::warning title=Round-1 carry::Running everything: the decision failed \(/);
  assert.equal(unreadable.logs[1], `::warning title=${DECISION_TITLE}::This run left no tested-tree record, so the next push runs everything.`);
  assert.equal(unreadable.outputs, 'reuse=false\nurl=\ntree=\nfinished=\nnames=\n');
});
