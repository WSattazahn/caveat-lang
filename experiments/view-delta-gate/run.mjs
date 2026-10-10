// The View 0.2 delta gate (spec/caveat-view-0.2.md, "The delta gate"): every
// tracked Caveat program, driven with seeded events through the delta path,
// with each delta applied to the held view and compared with the full view.
// Run from any directory: node experiments/view-delta-gate/run.mjs
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const recorded = JSON.parse(readFileSync(join(root, 'experiments/view-delta-gate/population.json'), 'utf8'));
const output = join(root, 'test-results');
mkdirSync(output, { recursive: true });

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, maxBuffer: 64 * 1024 * 1024, encoding: 'utf8', ...options });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${command} ${args.join(' ')} failed:\n${result.stderr ?? ''}`);
  return result.stdout;
}

const programs = run('git', ['ls-files', '-z', '--', '*.cav']).split('\0').filter(Boolean).sort();
assert.ok(programs.length > 0, 'the tracked Caveat corpus must not be empty');
run('cargo', ['build', '--locked', '--release', '--manifest-path', 'runtime/Cargo.toml', '--example', 'view_delta_sweep'], { stdio: ['ignore', 'inherit', 'inherit'] });
const extension = process.platform === 'win32' ? '.exe' : '';
const binary = join(root, `runtime/target/release/examples/view_delta_sweep${extension}`);
const rows = run(binary, [String(recorded.runs), String(recorded.events)], { input: programs.join('\n') + '\n' })
  .trim().split(/\r?\n/).filter(Boolean);
writeFileSync(join(output, 'view-delta-sweep.txt'), rows.join('\n') + '\n');
assert.equal(rows.length, programs.length);

const skipped = [];
const totals = {};
let detected = 0;
let tried = 0;
for (const [index, row] of rows.entries()) {
  const [path, ...fields] = row.split(' ');
  assert.equal(path, programs[index]);
  if (fields.join(' ') === 'skipped') { skipped.push(path); continue; }
  const values = Object.fromEntries(fields.map(field => field.split('=')));
  assert.equal(values.mismatched, '0', `${path}: a delta did not give the full view`);
  assert.equal(values.fatal, '0', `${path}: a fatal dispatch`);
  const [found, total] = values.controls.split('/').map(Number);
  assert.equal(found, total, `${path}: a corrupted delta went undetected`);
  detected += found; tried += total;
  for (const [name, value] of Object.entries(values)) {
    if (name !== 'controls') totals[name] = (totals[name] ?? 0) + Number(value);
  }
}
const unnamed = skipped.filter(path => !(path in recorded.skipped));
assert.deepEqual(unnamed, [], 'programs skipped that population.json does not name');
const checkedPrograms = rows.length - skipped.length;
assert.ok(checkedPrograms >= recorded.minimum_checked_programs, `only ${checkedPrograms} programs checked`);
assert.ok(totals.checked >= recorded.minimum_checked_events, `only ${totals.checked} accepted events checked`);
assert.ok(tried > 0, 'no negative control ran');

const report = {
  passed: true,
  revision: run('git', ['rev-parse', 'HEAD']).trim(),
  runs: recorded.runs, events: recorded.events,
  programs: rows.length, checked_programs: checkedPrograms,
  skipped: skipped.length, skipped_programs: skipped,
  accepted_events_checked: totals.checked, refused: totals.refused, restores_checked: totals.restored,
  negative_controls: { detected, tried },
  // Measured separately, summed over every event of every run.
  dispatch_ms: Math.round(totals.dispatch_ns / 1e6),
  delta_path_ms: Math.round(totals.delta_ns / 1e6),
  full_view_ms: Math.round(totals.view_ns / 1e6),
  apply_ms: Math.round(totals.apply_ns / 1e6),
  full_view_bytes: totals.view_bytes, delta_bytes: totals.delta_bytes,
};
writeFileSync(join(output, 'view-delta-gate.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
