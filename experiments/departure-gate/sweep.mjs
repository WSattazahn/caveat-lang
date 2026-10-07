// Compare both runtimes against identical current source bytes on this host.
// Run from any directory: node experiments/departure-gate/sweep.mjs
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { copyFileSync, closeSync, mkdirSync, mkdtempSync, openSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const baselineRevision = '3a88ba0f80d563b4493840dc7bd7e195329b8302'; // v0.1.0-rc.15, peeled commit
const output = join(root, 'test-results');
const originalWindows = [
  'c3-windows.cav', 'guarded-skip.cav', 'kit-explain-window.cav',
  'kit-types-window.cav', 'runtime-windows-world.cav',
].map(name => `experiments/departure-gate/${name}`);

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

function hasWindow(source) {
  // Ignore comments and strings, so prose saying "window" does not omit a
  // program. Windows belong only to readings, renewables and the journal.
  const code = source.replace(/\/\/[^\r\n]*|\/\*[\s\S]*?\*\/|"(?:\\.|[^"\\])*"/g, ' ');
  return /\b(?:readings|renewable|journal)\b[^;{}]*\bwindow\s+\d+\b/.test(code);
}

function lines(text) { return text.trim().split(/\r?\n/).filter(Boolean); }
function digest(text) { return createHash('sha256').update(text).digest('hex'); }

function sweep(binary, programs, args, name) {
  console.error(`${name}: ${programs.length} programs, ${args[0]} runs, ${args[1]} events per run`);
  const result = run(binary, args.map(String), { input: programs.join('\n') + '\n', encoding: 'utf8' });
  const rows = lines(result.stdout);
  writeFileSync(join(output, `${name}.txt`), rows.join('\n') + '\n');
  writeFileSync(join(output, `${name}-stderr.txt`), result.stderr);
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
const allPrograms = [...new Set([...tracked, ...additions])].sort();
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
logged('cargo', buildArgs, 'compatibility-rc15-build.log', baseline);
logged('cargo', buildArgs, 'compatibility-rc16-build.log');
const extension = process.platform === 'win32' ? '.exe' : '';
const executable = (directory, example) => join(directory, `runtime/target/release/examples/${example}${extension}`);
const oldSave = sweep(executable(baseline, 'save_sweep'), noWindow, [8, 150], 'rc15-save-sweep');
const newSave = sweep(executable(root, 'save_sweep'), noWindow, [8, 150], 'rc16-save-sweep');
assert.equal(newSave.length, noWindow.length);
assert.ok(newSave.every((row, index) => row.startsWith(`${noWindow[index]} `)));
assert.ok(newSave.every(row => row.endsWith(' skipped') || row.endsWith(' fatal=0')));
assert.deepEqual(newSave, oldSave, 'no-window outcome, save and restore digests differ');
const oldOutcome = sweep(executable(baseline, 'outcome_sweep'), windowed, [32, 400], 'rc15-outcome-sweep');
const newOutcome = sweep(executable(root, 'outcome_sweep'), windowed, [32, 400], 'rc16-outcome-sweep');
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
