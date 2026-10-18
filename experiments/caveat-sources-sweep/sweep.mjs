// Corpus sweep for Caveat Sources 0.1 (spec/caveat-caveat-sources-0.1.md,
// "Compatibility"). Builds the runtime at the revision before the change and
// at this checkout, drives every tracked Caveat program with the same seeded
// events on both (runtime/examples/caveat_sweep.rs), and compares every
// state's caveats after every event, in its value and its grounds, and every
// view binding's. Each difference is counted by direction:
//   - a gain: the current build carries a caveat the baseline did not;
//   - a loss: the baseline carried a caveat the current build does not.
// The profile only adds caveats, so the sweep fails on any loss, and on any
// event whose outcome differs. It also reports the static population: the
// programs that both `qualify` and read a decision's record.
// Run from any directory: node experiments/caveat-sources-sweep/sweep.mjs [--write]
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { copyFileSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const baselineRevision = '1be361066abb83685026a51c48c496079ea27181';
const [runs, events] = [8, 150];
const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../..');

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, maxBuffer: 1 << 30, ...options });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${command} ${args.join(' ')}: ${result.stderr}`);
  return result;
}

const programs = run('git', ['ls-files', '-z', '--', '*.cav'], { encoding: 'utf8' })
  .stdout.split('\0').filter(Boolean).sort();
assert.ok(programs.length > 0, 'the tracked Caveat corpus must not be empty');

// The static population, from source text with `#` comments removed.
const code = path => readFileSync(join(root, path), 'utf8').replace(/#[^\n]*/g, '');
const readsRecord = text => /\b(committed|reopened)\(/.test(text) || /\b(history_at|basis_of)\(/.test(text);
const qualifies = text => /(^|[^\w])qualify\s+[\w(]/m.test(text.replace(/\bunqualify\b/g, ''));
const takesBack = text => /\bunqualify\b/.test(text);
const population = {
  can_gain: programs.filter(path => readsRecord(code(path)) && qualifies(code(path))),
  take_back_and_record: programs.filter(path => readsRecord(code(path)) && takesBack(code(path))),
};

const baseline = mkdtempSync(join(tmpdir(), 'caveat-sources-baseline-'));
const archive = run('git', ['archive', baselineRevision, 'runtime', 'game', 'rust-toolchain.toml']).stdout;
run('tar', ['-xf', '-', '-C', baseline], { input: archive });
copyFileSync(join(root, 'runtime/examples/caveat_sweep.rs'), join(baseline, 'runtime/examples/caveat_sweep.rs'));
const build = ['build', '--locked', '--release', '--manifest-path', 'runtime/Cargo.toml', '--example', 'caveat_sweep'];
run('cargo', build, { cwd: baseline, stdio: ['ignore', 'ignore', 'inherit'] });
run('cargo', build, { stdio: ['ignore', 'ignore', 'inherit'] });
const extension = process.platform === 'win32' ? '.exe' : '';
const sweep = directory => run(join(directory, `runtime/target/release/examples/caveat_sweep${extension}`),
  [String(runs), String(events)], { input: programs.join('\n') + '\n', encoding: 'utf8' }).stdout;

// One map per build: `PATH RUN STEP` to its event line and its caveat lines.
function steps(output) {
  const result = new Map();
  let key = null;
  for (const line of output.split('\n')) {
    if (!line) continue;
    if (line.startsWith('  ')) {
      const [state, place, caveats] = line.trim().split(' ');
      result.get(key).caveats.set(`${state} ${place}`, new Set(caveats.split(',')));
      continue;
    }
    const [path, runIndex, step, ...rest] = line.split(' ');
    key = `${path} ${runIndex} ${step}`;
    result.set(key, { path, event: rest.join(' '), caveats: new Map() });
  }
  return result;
}
const before = steps(sweep(baseline));
const after = steps(sweep(root));
assert.deepEqual([...after.keys()], [...before.keys()], 'both builds ran the same events');

const gained = new Map();
const lost = [];
for (const [key, now] of after) {
  const then = before.get(key);
  assert.equal(now.event, then.event, `${key}: outcome differs`);
  for (const place of new Set([...then.caveats.keys(), ...now.caveats.keys()])) {
    const old = then.caveats.get(place) ?? new Set();
    const current = now.caveats.get(place) ?? new Set();
    for (const caveat of old) if (!current.has(caveat)) lost.push(`${key} ${place} ${caveat}`);
    for (const caveat of current) {
      if (old.has(caveat)) continue;
      gained.set(now.path, (gained.get(now.path) ?? 0) + 1);
    }
  }
}

const results = {
  baseline_revision: baselineRevision,
  programs: programs.length,
  runs,
  events,
  static: {
    can_gain: population.can_gain.length,
    take_back_and_record: population.take_back_and_record,
  },
  gains: {
    programs: gained.size,
    readings: [...gained.values()].reduce((sum, count) => sum + count, 0),
    by_program: Object.fromEntries([...gained].sort()),
  },
  losses: lost.length,
};
console.log(JSON.stringify(results, null, 2));
if (process.argv.includes('--write')) {
  writeFileSync(join(here, 'RESULTS.json'), JSON.stringify(results, null, 2) + '\n');
}
assert.deepEqual(lost.slice(0, 20), [], `${lost.length} caveats lost`);
