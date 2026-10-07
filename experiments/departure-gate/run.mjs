// Run the registered departure gates on the local build. Never publishes.
import { spawnSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync, copyFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
const root = fileURLToPath(new URL('../../', import.meta.url));
process.chdir(root);
assert.ok(process.argv.slice(2).every(x => x === '--skip-sweeps'), 'only --skip-sweeps is supported');
mkdirSync('test-results', { recursive: true });
const fixture = 'experiments/departure-gate/fixtures/';
const provenance = JSON.parse(readFileSync(fixture + 'rc15-c3-provenance.json', 'utf8'));
const hash = p => createHash('sha256').update(readFileSync(p)).digest('hex');
assert.equal(hash(fixture + 'rc15-c3-save.json'), provenance.save_sha256);
assert.equal(hash('experiments/departure-gate/c3-windows.cav'), provenance.source_sha256);
function run(script, args = [], output) {
  const result = spawnSync(process.execPath, [script, ...args], { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024, timeout: 1800000 });
  if (output) writeFileSync(output, result.stdout ?? ''); else process.stdout.write(result.stdout ?? '');
  process.stderr.write(result.stderr ?? '');
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${script} failed`);
}
if (!process.argv.includes('--skip-sweeps')) run('experiments/departure-gate/sweep.mjs');
copyFileSync(fixture + 'rc15-c3-measurement.json', 'test-results/c3-published-rc15.json');
run('experiments/departure-gate/c3.mjs', ['--package=kit'], 'test-results/c3-reviewed.json');
run('experiments/departure-gate/cross-restore.mjs', ['--package=kit'], 'test-results/cross-restore-reviewed.json');
run('experiments/departure-gate/pinned.mjs', ['--package=kit'], 'test-results/pinned-reviewed.json');
run('experiments/departure-gate/check-results.mjs', [], 'test-results/departure-gates.json');
const report = JSON.parse(readFileSync('test-results/departure-gates.json', 'utf8'));
console.log(JSON.stringify({ passed: report.passed, no_window: report.no_window, windowed: report.windowed,
  c3: { sizes: report.c3.sizes, outcomes: report.c3.outcomes, dispatch_us: report.c3.dispatch_us, archive: report.c3.archive },
  restored_states: report.cross_restore.state_count, pins: report.pin_attribution.reports.map(({program,maxima})=>({program,maxima})) }, null, 2));
