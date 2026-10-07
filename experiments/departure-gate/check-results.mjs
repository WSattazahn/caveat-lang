// Registered departure gates; measurements are separate commands so failures
// retain their full receipts. No thresholds are fitted to a particular run.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
const json = file => JSON.parse(readFileSync(file, 'utf8'));
const lines = file => readFileSync(file, 'utf8').trim().split(/\r?\n/);
const baselineSave = lines('test-results/rc15-save-sweep.txt');
const currentSave = lines('test-results/rc16-save-sweep.txt');
const population = lines('test-results/rc16-nonwindow-population.txt');
assert.equal(baselineSave.length, population.length, 'baseline includes the entire registered population');
assert.equal(currentSave.length, population.length, 'current build includes the entire registered population');
assert.ok(currentSave.every((line, index) => line.startsWith(population[index] + ' ')));
assert.ok(currentSave.every(line => !line.includes(' unreadable ')));
assert.deepEqual(currentSave, baselineSave, 'no-window outcomes, saves, restores and refusal digests must match rc.15');
const baselineOutcomes = lines('test-results/rc15-outcome-sweep.txt');
const currentOutcomes = lines('test-results/rc16-outcome-sweep.txt');
const programs = new Set(currentOutcomes.map(line => line.split(' ')[0]));
const windowPopulation = lines('test-results/rc16-window-population.txt');
assert.deepEqual([...programs].sort(), [...windowPopulation].sort(), 'the complete windowed corpus must run');
for (const name of ['c3-windows', 'guarded-skip', 'kit-explain-window', 'kit-types-window', 'runtime-windows-world']) {
  assert.ok(windowPopulation.includes(`experiments/departure-gate/${name}.cav`), `missing registered window fixture: ${name}`);
}
assert.equal(currentOutcomes.length, windowPopulation.length * 32, '32 seeded runs per fixture');
assert.ok(currentOutcomes.every(line => line.includes('fatal=false')));
assert.deepEqual(currentOutcomes, baselineOutcomes, 'windowed dispatch outcomes must match retirement-only rc.15');
const baseline = json('test-results/c3-published-rc15.json');
const current = json('test-results/c3-reviewed.json');
assert.equal(current.source_sha256, baseline.source_sha256);
assert.equal(current.outcomes.accepted + current.outcomes.refused, 44460);
assert.deepEqual(current.outcomes, baseline.outcomes);
assert.equal(current.sizes[30].structure_sha256, current.sizes[60].structure_sha256,
  'live-save structure must stay fixed from 30 to60 cycles');
assert.equal(current.sizes[30].adapter, current.sizes[60].adapter,
  'registered C3 adapter save gate: same bytes at30 and60 cycles');
assert.ok(current.dispatch_us.cycles_51_60.median < 1000, 'registered C3 median dispatch gate is under1ms');
assert.equal(current.archive.undrained_after_drain, 0);
assert.equal(current.archive.items, current.archive.records + current.archive.provenance_nodes);
const restore = json('test-results/cross-restore-reviewed.json');
assert.equal(restore.checkpoints.every_state_set.states_written.length, restore.state_count);
assert.ok(restore.state_count > 0);
const pins = json('test-results/pinned-reviewed.json');
assert.equal(pins.reports.length, 5);
assert.equal(current.runtime.reactiveWasmSha256, restore.runtime.reactiveWasmSha256);
assert.equal(current.runtime.reactiveWasmSha256, pins.runtime.reactiveWasmSha256);
assert.ok(pins.reports.every(report => report.checkpoints > 0 && report.runs === 8 && report.events_per_run === 4000));
const digest = rows => createHash('sha256').update(rows.join('\n') + '\n').digest('hex');
console.log(JSON.stringify({ passed: true,
  no_window: { programs: population.length, executed: currentSave.filter(line => line.includes(' saves=')).length,
    skipped: currentSave.filter(line => line.endsWith(' skipped')).length, runs: 8, events: 150, sha256: digest(currentSave) },
  windowed: { programs: [...programs], runs_per_program: 32, events_per_run: 400, sha256: digest(currentOutcomes) },
  c3: { sizes: current.sizes, outcomes: current.outcomes, dispatch_us: current.dispatch_us, archive: current.archive },
  cross_restore: restore, pin_attribution: pins }, null, 2));
