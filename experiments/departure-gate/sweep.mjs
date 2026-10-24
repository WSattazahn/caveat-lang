// Compare both runtimes against identical current source bytes on this host.
// Run from any directory: node experiments/departure-gate/sweep.mjs
import assert from 'node:assert/strict';
import { reviewedSaveChanges, comparableSave } from './reviewed-save-changes.mjs';
import { createHash } from 'node:crypto';
import { copyFileSync, closeSync, mkdirSync, mkdtempSync, openSync, readFileSync, writeFileSync } from 'node:fs';
import { availableParallelism, tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const baselineRevision = '3a88ba0f80d563b4493840dc7bd7e195329b8302'; // v0.1.0-rc.15, peeled commit
const output = join(root, 'test-results');
const originalWindows = [
  'c3-windows.cav', 'guarded-skip.cav', 'kit-explain-window.cav',
  'kit-types-window.cav', 'runtime-windows-world.cav',
].map(name => `experiments/departure-gate/${name}`);
// Programs written in syntax rc.15 cannot load have no rc.15 behavior to
// preserve. Each is named here, and must use that syntax.
const newerThanBaseline = {
  'experiments/take-back-sweep/programs/guarded.cav': /\bunqualify\b/,
  'experiments/take-back-sweep/programs/round8.cav': /\bunqualify\b/,
  'experiments/take-back-sweep/programs/windowed.cav': /\bunqualify\b/,
  'experiments/stabilization-trial/trial.cav': /\bunqualify\b/,
  'docs/releases/v1.0.1-d012.cav': /\bunqualify\b/,
  'docs/releases/v1.0.3-d020.cav': /\bunqualify\b/,
  'kit/examples/procedure-handles/handles.cav': /\bs readings\)/,
  'experiments/false-completion-delivery/delivery.cav': /\bnoting\b/,
  'experiments/narrowed-test-gates/required.cav': /\bnoting\b/,
};


function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, maxBuffer: 64 * 1024 * 1024, ...options });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${command} ${args.join(' ')} failed (${result.signal ?? 'exit'}):\n${result.stderr ?? ''}`);
  return result;
}

function logged(command, args, log, cwd = root) {
  console.error(`${command} ${args.join(' ')} (log: ${log})`);
  const fd = openSync(join(output, log), 'w');
  try { run(command, args, { cwd, stdio: ['ignore', fd, fd] }); }
  finally { closeSync(fd); }
}

// The asynchronous form of run, so independent builds and sweep shards can
// share the host's cores. Each result is checked exactly as run checks it.
function runAsync(command, args, { input, cwd = root, stdio } = {}) {
  return new Promise((resolveRun, reject) => {
    const child = spawn(command, args, { cwd, stdio: stdio ?? ['pipe', 'pipe', 'pipe'] });
    const stdout = [], stderr = [];
    child.stdout?.on('data', chunk => stdout.push(chunk));
    child.stderr?.on('data', chunk => stderr.push(chunk));
    child.on('error', reject);
    child.on('close', (status, signal) => {
      const result = { status, signal, stdout: Buffer.concat(stdout).toString('utf8'), stderr: Buffer.concat(stderr).toString('utf8') };
      try {
        assert.equal(status, 0, `${command} ${args.join(' ')} failed (${signal ?? 'exit'}):\n${result.stderr}`);
        resolveRun(result);
      } catch (error) { reject(error); }
    });
    if (child.stdin) child.stdin.end(input ?? '');
  });
}

async function loggedAsync(command, args, log, cwd = root) {
  console.error(`${command} ${args.join(' ')} (log: ${log})`);
  const fd = openSync(join(output, log), 'w');
  try { await runAsync(command, args, { cwd, stdio: ['ignore', fd, fd] }); }
  finally { closeSync(fd); }
}

// Run at most `limit` tasks at once; results keep the order of `tasks`.
async function pool(tasks, limit) {
  const results = new Array(tasks.length);
  let next = 0;
  const worker = async () => { while (next < tasks.length) { const index = next++; results[index] = await tasks[index](); } };
  await Promise.all(Array.from({ length: Math.min(limit, tasks.length) }, worker));
  return results;
}

function hasWindow(source) {
  // Ignore comments and strings, so prose saying "window" does not omit a
  // program. Windows belong only to readings, renewables and the journal.
  const code = source.replace(/\/\/[^\r\n]*|\/\*[\s\S]*?\*\/|"(?:\\.|[^"\\])*"/g, ' ');
  return /\b(?:readings|renewable|journal)\b[^;{}]*\bwindow\s+\d+\b/.test(code);
}

function lines(text) { return text.trim().split(/\r?\n/).filter(Boolean); }
function digest(text) { return createHash('sha256').update(text).digest('hex'); }

// The sweep examples handle each program independently, in input order, so
// every program is a separate task: running them on all cores and joining
// the output in program order gives the same bytes as one sequential process.
function sweepTasks(binary, programs, args, name) {
  console.error(`${name}: ${programs.length} programs, ${args[0]} runs, ${args[1]} events per run`);
  return programs.map(program => () => runAsync(binary, args.map(String), { input: program + '\n' }));
}

function record(name, results) {
  const rows = lines(results.map(result => result.stdout).join(''));
  writeFileSync(join(output, `${name}.txt`), rows.join('\n') + '\n');
  writeFileSync(join(output, `${name}-stderr.txt`), results.map(result => result.stderr).join(''));
  assert.ok(rows.every(row => !row.includes(' unreadable ')), `${name}: unreadable source`);
  return rows;
}

if (process.argv.includes('--help')) {
  console.log('Build locked release runtimes at pinned rc.15 and the current checkout, then compare all tracked .cav programs plus new departure-gate/002-decision fixtures on identical current source bytes. Requires git, tar, Cargo and the pinned Rust toolchain. Writes test-results/; retains a temporary rc.15 checkout for diagnosis.');
  process.exit(0);
}
assert.equal(process.argv.length, 2, 'No arguments are accepted (except --help).');
mkdirSync(output, { recursive: true });
const tracked = run('git', ['ls-files', '-z', '--', '*.cav'], { encoding: 'utf8' }).stdout.split('\0').filter(Boolean);
// Include this deliverable before staging without sweeping unrelated local
// directories such as a user's preserved rc11 checkout.
const additions = run('git', ['ls-files', '-z', '--others', '--exclude-standard', '--', 'experiments/departure-gate', 'experiments/agent-decisions/002-departure-continuation'], { encoding: 'utf8' }).stdout.split('\0').filter(path => path.endsWith('.cav'));
for (const [path, syntax] of Object.entries(newerThanBaseline)) {
  assert.ok(tracked.includes(path) || additions.includes(path), `missing registered newer program: ${path}`);
  assert.match(readFileSync(join(root, path), 'utf8'), syntax, `${path} no longer uses syntax newer than rc.15`);
}
const allPrograms = [...new Set([...tracked, ...additions])].filter(path => !(path in newerThanBaseline)).sort();
assert.ok(allPrograms.length > 0, 'the tracked Caveat corpus must not be empty');
const sourceDigest = () => digest(JSON.stringify(allPrograms.map(path => [path, digest(readFileSync(join(root, path)))])));
const corpusDigest = sourceDigest();
const noWindow = allPrograms.filter(path => !hasWindow(readFileSync(join(root, path), 'utf8')));
const windowed = allPrograms.filter(path => hasWindow(readFileSync(join(root, path), 'utf8')));
assert.ok(noWindow.length > 0, 'the no-window corpus must not be empty');
for (const path of originalWindows) assert.ok(windowed.includes(path), `missing registered window fixture: ${path}`);
writeFileSync(join(output, 'rc16-nonwindow-population.txt'), noWindow.join('\n') + '\n');
writeFileSync(join(output, 'rc16-window-population.txt'), windowed.join('\n') + '\n');

const baseline = mkdtempSync(join(tmpdir(), 'caveat-rc15-compatibility-'));
console.error(`Pinned rc.15 source retained at ${baseline}`);
const present = spawnSync('git', ['cat-file', '-e', `${baselineRevision}^{commit}`], { cwd: root, encoding: 'utf8' });
if (present.error) throw present.error;
if (present.status !== 0) {
  // CI's shallow checkout may not contain this immutable baseline object.
  // Fetch only that commit; never switch or reset the working branch.
  logged('git', ['fetch', '--no-tags', '--depth=1', 'origin', baselineRevision], 'compatibility-rc15-fetch.log');
}
// A source archive is independent of the active checkout and is not a Git
// worktree. Both binaries below nevertheless read programs from root.
const archive = run('git', ['archive', baselineRevision, 'runtime', 'game', 'rust-toolchain.toml']).stdout;
run('tar', ['-xf', '-', '-C', baseline], { input: archive });
for (const example of ['save_sweep', 'outcome_sweep']) {
  copyFileSync(join(root, `runtime/examples/${example}.rs`), join(baseline, `runtime/examples/${example}.rs`));
}
const buildArgs = ['build', '--locked', '--release', '--manifest-path', 'runtime/Cargo.toml', '--example', 'save_sweep', '--example', 'outcome_sweep'];
// The two builds are independent, and each spends most of its time in one
// single-codegen-unit LTO step, so they run side by side.
await Promise.all([
  loggedAsync('cargo', buildArgs, 'compatibility-rc15-build.log', baseline),
  loggedAsync('cargo', buildArgs, 'compatibility-rc16-build.log'),
]);
const extension = process.platform === 'win32' ? '.exe' : '';
const executable = (directory, example) => join(directory, `runtime/target/release/examples/${example}${extension}`);
const sweeps = [
  ['rc15-save-sweep', executable(baseline, 'save_sweep'), noWindow, [8, 150]],
  ['rc16-save-sweep', executable(root, 'save_sweep'), noWindow, [8, 150]],
  ['rc15-outcome-sweep', executable(baseline, 'outcome_sweep'), windowed, [32, 400]],
  ['rc16-outcome-sweep', executable(root, 'outcome_sweep'), windowed, [32, 400]],
];
const tasks = sweeps.map(([name, binary, programs, args]) => sweepTasks(binary, programs, args, name));
const results = await pool(tasks.flat(), availableParallelism());
let offset = 0;
const [oldSave, newSave, oldOutcome, newOutcome] = sweeps.map(([name], index) => {
  const own = results.slice(offset, offset + tasks[index].length);
  offset += tasks[index].length;
  return record(name, own);
});
assert.equal(newSave.length, noWindow.length);
assert.ok(newSave.every((row, index) => row.startsWith(`${noWindow[index]} `)));
assert.ok(newSave.every(row => row.endsWith(' skipped') || row.endsWith(' fatal=0')));
for (const path of reviewedSaveChanges) assert.ok(noWindow.includes(path), `reviewed save change names no gated program: ${path}`);
assert.deepEqual(comparableSave(newSave), comparableSave(oldSave), 'no-window outcome, save and restore digests differ');
assert.equal(newOutcome.length, windowed.length * 32);
assert.ok(newOutcome.every((row, index) => row.startsWith(`${windowed[Math.floor(index / 32)]} run=${index % 32} `)));
assert.ok(newOutcome.every(row => row.includes('fatal=false')));
assert.deepEqual(newOutcome, oldOutcome, 'windowed dispatch outcomes differ');
assert.equal(sourceDigest(), corpusDigest, 'Caveat source bytes changed during the sweep; rerun on a stable corpus');
const report = {
  passed: true, baseline_revision: baselineRevision, baseline_source: baseline,
  current_revision: run('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).stdout.trim(),
  platform: process.platform, architecture: process.arch,
  source_bytes: 'Both binaries read the same current checkout; no cross-platform saved digest comparison.',
  corpus_sha256: corpusDigest,
  sweep_examples: Object.fromEntries(['save_sweep', 'outcome_sweep'].map(name => [name, digest(readFileSync(join(root, `runtime/examples/${name}.rs`)))])),
  no_window: { programs: noWindow.length, executed: newSave.filter(row => row.includes(' saves=')).length, skipped: newSave.filter(row => row.endsWith(' skipped')).length, runs: 8, events: 150, sha256: digest(newSave.join('\n') + '\n') },
  windowed: { programs: windowed.length, rows: newOutcome.length, runs: 32, events: 400, sha256: digest(newOutcome.join('\n') + '\n') },
};
writeFileSync(join(output, 'compatibility-summary.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
