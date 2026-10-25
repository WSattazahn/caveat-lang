// Runs every tracked Caveat program and scenario file under two CLIs and
// compares what they print: `validate --json` for each program, `test --json`
// for each scenario file. Untrusted Sources 0.1 adds only forms that refuse
// to load today, so a released CLI and this checkout must agree on all of them.
// Usage: node experiments/untrusted-sources-sweep/compare.mjs <baseline caveat.mjs> [--write]
// The baseline is an installed package's bin/caveat.mjs, such as caveat-lang@1.3.0's.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../..');
const baseline = process.argv[2];
assert.ok(baseline, 'name the baseline CLI');
const current = join(root, 'kit', 'bin', 'caveat.mjs');
const tracked = pattern => {
  const listed = spawnSync('git', ['ls-files', '-z', '--', pattern], { cwd: root, encoding: 'utf8' });
  assert.equal(listed.status, 0, listed.stderr);
  return listed.stdout.split('\0').filter(Boolean).sort();
};
const run = (cli, args) => {
  const result = spawnSync(process.execPath, [cli, ...args], { cwd: root, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 });
  // A scenario report names the runtime that ran it; that is what differs by
  // design. Durations differ from run to run.
  let stdout = result.stdout.replace(/"(duration_ms|elapsed_ms|ms)": [\d.]+/g, '"$1": 0');
  if (args[0] === 'test') {
    try {
      const report = JSON.parse(stdout);
      delete report.runtime;
      stdout = JSON.stringify(report);
    } catch { /* not JSON: compared as printed */ }
  }
  return { status: result.status, stdout, stderr: result.stderr };
};

const differences = [];
const counts = { programs: 0, programs_loading: 0, scenario_files: 0, scenario_files_passing: 0 };
for (const program of tracked('*.cav')) {
  counts.programs += 1;
  const [a, b] = [baseline, current].map(cli => run(cli, ['validate', '--json', program]));
  if (b.status === 0) counts.programs_loading += 1;
  if (a.status !== b.status || a.stdout !== b.stdout) differences.push({ kind: 'validate', file: program, baseline: a.status, current: b.status });
}
for (const file of tracked('*.scenarios.json')) {
  counts.scenario_files += 1;
  const [a, b] = [baseline, current].map(cli => run(cli, ['test', '--json', file]));
  if (b.status === 0) counts.scenario_files_passing += 1;
  if (a.status !== b.status || a.stdout !== b.stdout) differences.push({ kind: 'test', file, baseline: a.status, current: b.status });
}
const results = {
  note: 'Untrusted Sources 0.1: every tracked program validated and every tracked scenario file run under a released CLI and this checkout. See README.md.',
  revision: spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).stdout.trim(),
  baseline: run(baseline, ['--version']).stdout.trim(),
  ...counts,
  differences,
};
const json = `${JSON.stringify(results, null, 2)}\n`;
if (process.argv.includes('--write')) writeFileSync(join(here, 'COMPARE.json'), json);
process.stdout.write(json);
