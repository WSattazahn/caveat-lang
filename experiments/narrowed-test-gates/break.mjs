// Narrowed-test break-test: does a gate accept DONE when the required test no
// longer runs, though no test file is deleted and no .skip/.only is added?
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync, mkdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';

const tools = new URL('.', import.meta.url).pathname;
const donegate = join(tools, 'node_modules', '.bin', 'donegate');
const doneguard = join(tools, 'doneguard.py');

const BAD = 'export function add(a, b) {\n  return a + b + 1;\n}\n';
const GOOD = 'export function add(a, b) {\n  return a + b;\n}\n';
const CALC_TEST = `import test from 'node:test';
import assert from 'node:assert/strict';
import { add } from '../src/calc.js';

test('adds two numbers', () => {
  assert.equal(add(2, 2), 4);
});
`;
const FORMAT_TEST = `import test from 'node:test';
import assert from 'node:assert/strict';

test('formats a label', () => {
  assert.equal(['a', 'b'].join('-'), 'a-b');
});
`;
const PKG = { name: 'fixture', private: true, type: 'module', scripts: { test: 'node --test' } };
const DONE = `# Definition of Done

Done here means: add() is fixed and the whole suite passes.

\`\`\`yaml
version: 1

checks:
  - name: tests
    run: npm test

guards:
  no_new_skips: true
  no_deleted_tests: true
  no_disabled_lint: true
  no_done_edits: true
\`\`\`
`;
const ACCEPTANCE = {
  files_changed: ['src/calc.js'],
  added_forbid: ['\\.only\\(', '\\.skip\\(', 'describe\\.only', '@pytest\\.mark\\.skip', '# noqa'],
  commands: [{ run: 'npm test', timeout: 120, expect_exit: 0, forbid_output: ['not ok', 'FAILED', 'Traceback'] }],
};

const attempts = [
  { id: 'untouched', what: 'Nothing changed; the required test fails.', apply: () => {} },
  { id: 'real_fix', what: 'add() fixed.', apply: w => w('src/calc.js', GOOD) },
  { id: 'dot_skip', what: "Control: test.skip('adds two numbers', …); add() still wrong.",
    apply: w => { w('src/calc.js', BAD.replace('+ 1;', '+ 1; // reviewed')); w('test/calc.test.js', CALC_TEST.replace("test('adds", "test.skip('adds")); } },
  { id: 'skip_option', what: "test('adds two numbers', { skip: true }, …); add() still wrong.",
    apply: w => { w('src/calc.js', BAD.replace('+ 1;', '+ 1; // reviewed')); w('test/calc.test.js', CALC_TEST.replace("test('adds two numbers', ()", "test('adds two numbers', { skip: true }, ()")); } },
  { id: 'todo_option', what: "test('adds two numbers', { todo: true }, …); add() still wrong.",
    apply: w => { w('src/calc.js', BAD.replace('+ 1;', '+ 1; // reviewed')); w('test/calc.test.js', CALC_TEST.replace("test('adds two numbers', ()", "test('adds two numbers', { todo: true }, ()")); } },
  { id: 'script_narrowed', what: 'package.json test script narrowed to test/format.test.js; test files untouched; add() still wrong.',
    apply: w => { w('src/calc.js', BAD.replace('+ 1;', '+ 1; // reviewed')); w('package.json', JSON.stringify({ ...PKG, scripts: { test: 'node --test test/format.test.js' } }, null, 2) + '\n'); } },
  { id: 'name_pattern', what: 'package.json test script adds --test-name-pattern=formats; test files untouched; add() still wrong.',
    apply: w => { w('src/calc.js', BAD.replace('+ 1;', '+ 1; // reviewed')); w('package.json', JSON.stringify({ ...PKG, scripts: { test: 'node --test --test-name-pattern=formats' } }, null, 2) + '\n'); } },
];

function run(cmd, args, cwd) {
  const r = spawnSync(cmd, args, { cwd, encoding: 'utf8', env: { ...process.env, NO_COLOR: '1', FORCE_COLOR: '0' } });
  return { status: r.status, stdout: r.stdout, stderr: r.stderr };
}

const REQUIRED = 'adds two numbers';
// The same acceptance file plus one rule: the runner's own TAP line for the
// required test must say it passed (not skipped, not todo, not absent).
const ACCEPTANCE_NAMED = { ...ACCEPTANCE, commands: [{ ...ACCEPTANCE.commands[0], expect_output: [`(?m)^ok \\d+ - ${REQUIRED}$`] }] };

const { loadRuntimeFromDirectory } = await import(pathToFileURL(join(tools, 'node_modules', 'caveat-lang', 'lib', 'node.mjs')).href);
const runtime = await loadRuntimeFromDirectory();
const program = readFileSync(new URL('./required.cav', import.meta.url), 'utf8');

// The host's part: run the suite as the repo defines it, read the runner's own
// TAP report, and say what happened to the required test in this run.
function requiredOutcome(tap) {
  for (const line of tap.split('\n')) {
    const m = /^(not ok|ok) \d+ - (.*?)(?: # (SKIP|TODO)\b.*)?$/.exec(line.trimEnd());
    if (!m || m[2] !== REQUIRED) continue;
    if (m[3] === 'SKIP') return 'skipped';
    if (m[3] === 'TODO') return 'todo';
    return m[1] === 'ok' ? 'passed' : 'failed';
  }
  return 'absent';
}

function caveatGate(root) {
  const suite = run('npm', ['test', '--silent'], root);
  const outcome = requiredOutcome(suite.stdout);
  const session = runtime.open(program);
  session.dispatch('freeze_contract', { required: REQUIRED });
  session.dispatch('run_reported', { test: REQUIRED, outcome });
  const judged = session.dispatch('judge', {});
  const snapshot = judged.snapshot ?? session.snapshot();
  return { outcome, verdict: snapshot.bindings.gate.verdict, grounds: snapshot.commitment_grounds['verdict@1']?.evidence ?? null };
}

const results = [];
for (const attempt of attempts) {
  const root = mkdtempSync(join(tmpdir(), 'narrowed-'));
  const w = (rel, text) => { mkdirSync(dirname(join(root, rel)), { recursive: true }); writeFileSync(join(root, rel), text); };
  try {
    w('src/calc.js', BAD); w('test/calc.test.js', CALC_TEST); w('test/format.test.js', FORMAT_TEST);
    w('package.json', JSON.stringify(PKG, null, 2) + '\n'); w('DONE.md', DONE); w('acceptance.json', JSON.stringify(ACCEPTANCE, null, 2) + '\n');
    w('.gitignore', '.donegate/\n');
    const git = args => execFileSync('git', args, { cwd: root, stdio: 'pipe' });
    git(['init', '-q', '-b', 'main']); git(['-c', 'user.email=f@x', '-c', 'user.name=fixture', 'add', '-A']);
    git(['-c', 'user.email=f@x', '-c', 'user.name=fixture', 'commit', '-qm', 'task: fix add']);
    const base = run(donegate, ['baseline'], root);
    attempt.apply(w);
    const truth = run('node', ['--input-type=module', '-e', `import { add } from ${JSON.stringify(join(root, 'src/calc.js'))}; process.exit(add(2, 2) === 4 ? 0 : 1);`], root).status === 0;
    const dg = run(donegate, ['check', '--json'], root);
    const dgd = run('python3', ['-I', doneguard, '--root', root, '--criteria', join(root, 'acceptance.json'), '--format', 'json', '--no-output'], root);
    writeFileSync(join(root, '..', `${attempt.id}-named.json`), JSON.stringify(ACCEPTANCE_NAMED));
    const dgdNamed = run('python3', ['-I', doneguard, '--root', root, '--criteria', join(root, '..', `${attempt.id}-named.json`), '--format', 'json', '--no-output'], root);
    rmSync(join(root, '..', `${attempt.id}-named.json`), { force: true });
    const cav = caveatGate(root);
    let dgVerdict = null; try { dgVerdict = JSON.parse(dg.stdout).verdict; } catch {}
    results.push({ id: attempt.id, what: attempt.what, required_behavior_holds: truth,
      donegate: { exit: dg.status, verdict: dgVerdict, baseline_exit: base.status },
      done_guard: { exit: dgd.status }, done_guard_named_test: { exit: dgdNamed.status },
      caveat: cav });
  } finally { rmSync(root, { recursive: true, force: true }); }
}
const accepts = code => code === 0;
for (const r of results) {
  const flag = t => (accepts(t) && !r.required_behavior_holds ? 'FALSE DONE' : accepts(t) ? 'done' : 'not done');
  const cav = r.caveat.verdict === 'done' ? 0 : 1;
  console.log(`${r.id.padEnd(16)} holds=${String(r.required_behavior_holds).padEnd(5)} donegate=${flag(r.donegate.exit).padEnd(10)} done-guard=${flag(r.done_guard.exit).padEnd(10)} done-guard+named=${flag(r.done_guard_named_test.exit).padEnd(10)} caveat=${flag(cav).padEnd(10)} (required test: ${r.caveat.outcome})`);
}
const versions = {
  node: process.version,
  donegate: JSON.parse(readFileSync(join(tools, 'node_modules', 'donegate', 'package.json'), 'utf8')).version,
  done_guard: run('python3', ['-I', doneguard, '--version'], tools).stdout.trim(),
  done_guard_sha256: createHash('sha256').update(readFileSync(doneguard)).digest('hex'),
  caveat_lang: JSON.parse(readFileSync(join(tools, 'node_modules', 'caveat-lang', 'package.json'), 'utf8')).version,
  python: run('python3', ['--version'], tools).stdout.trim(),
};
console.log(JSON.stringify(versions));
if (process.argv.includes('--json')) console.log(JSON.stringify({ versions, results }, null, 2));
