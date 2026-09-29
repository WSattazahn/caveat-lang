// The harness's own logic, without a runtime: statistics, inputs and their
// hashes, target parsing and the adapter rewrite.
//
//   node --test experiments/performance-0.1/harness.test.mjs
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import test from 'node:test';
import { quantile, spread, summarize, summarizeRounds, toMicroseconds } from './lib/stats.mjs';
import { glowcapAdapterEvent, loadManifest, loadWorkload, removeBindings, repositoryRoot, scaleGlowcap } from './lib/workloads.mjs';
import { parseTarget } from './lib/targets.mjs';
import { coresDuring, parseCpuSets, performanceCores } from './lib/environment.mjs';
import { maskLabel, maskOf, maskProcessors, parseMask, parseMaskSet } from './lib/pinning.mjs';
import { ADAPTER_LOCATIONS, rewriteAdapter } from './lib/adapter.mjs';
import { SUITES } from './lib/suites.mjs';
import { JOB_GUARD, covering, judge, settled, toSamples } from './lib/guard.mjs';
import { CLASS_ORDER, CLASS_RULES, changedParts, classify, glowcapCategory, signature } from './lib/classes.mjs';

test('quantiles follow the published rule: samples[floor(p * (n - 1))]', () => {
  const sorted = [1, 2, 3, 4];
  assert.equal(quantile(sorted, 0.5), 2, 'the median of an even count is the lower median');
  assert.equal(quantile(sorted, 0.95), 3);
  assert.equal(quantile(sorted, 1), 4);
  const summary = summarize([4, 1, 3, 2]);
  assert.deepEqual([summary.n, summary.median, summary.min, summary.max, summary.mean], [4, 2, 1, 4, 2.5]);
  assert.equal(summarize([]), null);
});

test('rounds pool, skip missing slots and summarize segments by event position', () => {
  const result = summarizeRounds([[10, null, 30, 40], [20, 25, null, 60]], [{ name: 'head', from: 0, to: 2 }]);
  assert.equal(result.pooled.n, 6);
  assert.deepEqual(result.rounds.map((round) => round.n), [3, 3]);
  assert.equal(result.segments.head.n, 3);
  assert.equal(result.segments.head.median, 20);
  assert.deepEqual(toMicroseconds([1500, null], 'ns'), [1.5, null]);
});

test('spread reports the median, range and range share across runs', () => {
  assert.deepEqual(spread([40, 44, 42]), { runs: 3, median: 42, min: 40, max: 44, rangePercent: 9.5 });
  assert.equal(spread([null, undefined]), null);
});

test('every workload loads and matches the hashes recorded in inputs/workloads.json', async () => {
  const manifest = await loadManifest();
  for (const { id } of manifest.workloads) {
    const workload = await loadWorkload(manifest, id);
    assert.ok(workload.eventsPerPass > 0, id);
    for (const episode of workload.episodes) {
      for (const [event, payload] of episode) {
        assert.equal(typeof event, 'string');
        assert.doesNotThrow(() => JSON.parse(payload), `${id}: payload is JSON`);
      }
    }
  }
});

test('the primary workload is the published benchmark stream', async () => {
  const workload = await loadWorkload(await loadManifest(), 'glowcap-replay');
  const [episode] = workload.episodes;
  assert.equal(episode.length, 10_000);
  assert.deepEqual(episode.slice(0, 4), [
    ['absorb', '{"target":"cave","sort":"glowcap"}'],
    ['absorb', '{"target":"pool","sort":"duskcap"}'],
    ['taste', '{"target":"ruin","sort":"glowcap"}'],
    ['tick', '{"dt":0.05}'],
  ]);
  assert.ok(episode.slice(3).every(([event, payload]) => event === 'tick' && payload === '{"dt":0.05}'));
  // The harness's own stream, as experiments/glowcap/harness.mjs bench() builds it.
  const replay = [{ type: 'absorb', id: 'cave', kind: 'glowcap' }, { type: 'absorb', id: 'pool', kind: 'duskcap' }, { type: 'taste', id: 'ruin', kind: 'glowcap' }];
  while (replay.length < 10_000) replay.push({ type: 'tick', dt: 0.05 });
  assert.deepEqual(episode.map(glowcapAdapterEvent), replay);
  const segments = workload.segments.map(({ name, from, to }) => [name, from, to]);
  assert.deepEqual(segments, [['observations', 0, 3], ['decay', 3, 603], ['transition', 603, 1300], ['steady', 1300, 10_000]]);
});

test('truncation keeps the verified stream prefix and clips segments', async () => {
  const workload = await loadWorkload(await loadManifest(), 'glowcap-replay', { maxEvents: 700 });
  assert.equal(workload.episodes[0].length, 700);
  assert.deepEqual(workload.segments.map(({ name, to }) => [name, to]), [['observations', 3], ['decay', 603], ['transition', 700]]);
});

test('a scaled Glowcap adds mushrooms after grove and nothing else', async () => {
  const source = await readFile(path.join(repositoryRoot, 'experiments/glowcap/caveat5/glowcap.cav'), 'utf8');
  const scaled = scaleGlowcap(source, 2);
  assert.equal(scaled.length - source.length, 'entity patch1 kind mushroom at garden;\n'.length * 2);
  assert.ok(scaled.includes('entity grove kind mushroom at garden;\nentity patch1 kind mushroom at garden;\nentity patch2 kind mushroom at garden;\n'));
  assert.throws(() => scaleGlowcap('entity cave kind mushroom at garden;\n', 1), /grove/);
});

test('the unbound Glowcap drops exactly the bind statements', async () => {
  const source = await readFile(path.join(repositoryRoot, 'experiments/glowcap/caveat5/glowcap.cav'), 'utf8');
  const unbound = removeBindings(source);
  const lines = (text) => text.split('\n');
  const bound = lines(source).filter((line) => line.trimStart().startsWith('bind '));
  assert.equal(bound.length, 39);
  assert.deepEqual(lines(source).filter((line) => !line.trimStart().startsWith('bind ')), lines(unbound));
  assert.throws(() => removeBindings('state x = 0;'), /no one-line bind/);
});

test('targets parse as LABEL=KIND:DIR', () => {
  const target = parseTarget('rc4=package:C:/somewhere/package');
  assert.equal(target.label, 'rc4');
  assert.equal(target.kind, 'package');
  assert.throws(() => parseTarget('rc4:C:/x'), /LABEL=tree:DIR/);
});

test('performance cores are the highest efficiency class', () => {
  assert.deepEqual(performanceCores('0:0:1,1:1:1,2:2:0,10:10:1'), [0, 1, 10]);
  assert.deepEqual(performanceCores('0:0:1:2:0,1:1:1:2:0,2:2:0:1:0,13:13:1:3:0'), [0, 1, 13], 'with scheduling class and cache');
  assert.equal(performanceCores(null), null);
});

test('CPU sets parse into one record per logical processor, old and new text', () => {
  assert.deepEqual(parseCpuSets('13:13:1:3:0'), [{ logicalProcessor: 13, core: 13, efficiencyClass: 1, schedulingClass: 3, lastLevelCache: 0 }]);
  assert.deepEqual(parseCpuSets('2:2:0'), [{ logicalProcessor: 2, core: 2, efficiencyClass: 0, schedulingClass: null, lastLevelCache: null }]);
  assert.equal(parseCpuSets(null), null);
});

test('masks parse, print and expand to logical processors', () => {
  assert.equal(parseMask('0x3C00'), 0x3C00n);
  assert.equal(parseMask('3C00'), 0x3C00n);
  assert.equal(parseMask('none'), null);
  assert.equal(maskLabel(0xC00000n), '0xC00000');
  assert.equal(maskLabel(null), 'none');
  assert.deepEqual(maskProcessors(0x3C00n), [10, 11, 12, 13]);
  assert.equal(maskOf([10, 11, 12, 13]), 0x3C00n);
  assert.deepEqual(parseMaskSet('0x3,0xC00000').map(maskLabel), ['0x3', '0xC00000']);
  assert.throws(() => parseMaskSet('0x3,0x3'), /repeats/);
  assert.throws(() => parseMaskSet('0x3,none'), /needs masks/);
  assert.equal(parseMaskSet(null), null);
});

test('the busy processors of a job and their clock come from the samples taken while it ran', () => {
  const monitor = { processors: [0, 1], counters: ['total', 'processor 0', 'processor 1', 'processor 0 MHz', 'processor 1 MHz'] };
  const at = (seconds) => new Date(2026, 8, 29, 0, 0, seconds).toLocaleString('en-US', { hour12: false }).replace(',', '');
  const rows = [[at(0), '5', '90', '3', '5100', '800'], [at(1), '5', '95', '1', '5150', '900'], [at(9), '5', '0', '0', '800', '800']];
  const stopped = { rows };
  const from = new Date(2026, 8, 29, 0, 0, 0).getTime();
  const result = coresDuring(monitor, stopped, from, from + 2000);
  assert.equal(result.samples, 2);
  assert.equal(result.cores[0].busyPercent, 92.5);
  assert.equal(result.cores[0].mhzWhileBusy, 5125);
  assert.equal(result.cores[1].mhzWhileBusy, null, 'never more than half busy');
  assert.equal(coresDuring(null, stopped, 0, 1), null);
});

test('the job guard reads samples by position, covers a job from start to end and judges its load', () => {
  const counters = ['total', 'processor 10', 'processor 11', 'processor 10 MHz', 'processor 11 MHz', 'process System', 'process MsMpEng'];
  const at = (seconds) => new Date(2026, 8, 29, 0, 0, seconds).toLocaleString('en-US', { hour12: false }).replace(',', '');
  const base = new Date(2026, 8, 29, 0, 0, 0).getTime();
  // 24 logical processors; the benchmark keeps processor 10 busy.
  const row = (seconds, total, system, defender = '0') => [at(seconds), String(total), '100', '10', '4200', '800', String(system), defender];
  const samples = toSamples([row(1, 7, 5), row(2, 7, 5, '-1'), row(3, 7, 5), row(4, 16.5, 110), row(5, 7, 5), ['cut off']], counters);
  assert.equal(samples.length, 5, 'an incomplete row is dropped');
  assert.equal(samples[1].processes.MsMpEng, null, 'a process with no instance reads -1: missing, not zero');
  assert.equal(samples[1].since, samples[0].at, 'a sample covers the time since the one before it');
  const glitch = toSamples([row(1, '-1', 5), row(2, 7, 5)], counters);
  assert.equal(glitch[0].total, null, 'a failed total reading is missing');
  assert.equal(judge(glitch, { processors: [10, 11], logicalProcessors: 24 }).meanTotal, 7);
  assert.equal(settled([...glitch, ...glitch]), false, 'a missing total does not show quiet');
  const quiet = covering(samples, base + 1500, base + 2800);
  assert.equal(quiet.samples.length, 2);
  assert.ok(quiet.covered);
  const verdict = judge(quiet.samples, { processors: [10, 11], logicalProcessors: 24, covered: quiet.covered });
  assert.ok(verdict.kept, verdict.reasons.join('; '));
  assert.equal(verdict.maxOutsideCores, 0.58);
  const tick = covering(samples, base + 2500, base + 4200);
  const busy = judge(tick.samples, { processors: [10, 11], logicalProcessors: 24, covered: tick.covered });
  assert.equal(busy.kept, false);
  assert.deepEqual(busy.reasons, ['System at 110% of a core > 30%', '2.9 cores busy outside the pinned processors > 2']);
  assert.equal(judge([], { logicalProcessors: 24 }).kept, false, 'no samples, no verdict');
  assert.equal(covering(samples, base + 4500, base + 9000).covered, false, 'the end is not covered yet');
  // The replicate's first job: two samples, 14.7% and 17.4% total.
  const burst = toSamples([row(1, 14.7, 20, '15'), row(2, 17.4, 20, '15')], counters);
  assert.deepEqual(judge(burst, { processors: [10, 11], logicalProcessors: 24 }).reasons.slice(0, 1), ['mean total CPU 16% > 12%']);
  assert.equal(settled(samples.slice(0, 3)), true);
  assert.equal(settled(samples.slice(2, 5)), false);
  assert.equal(JOB_GUARD.attempts, 5);
});

test('the adapter rewrite finds each published location exactly once', async () => {
  const text = await readFile(path.join(repositoryRoot, 'experiments/glowcap/caveat5/adapter.mjs'), 'utf8');
  const rewritten = rewriteAdapter(text, { glueUrl: 'file:///r/glue.js', wasmUrl: 'file:///r/runtime.wasm', programUrl: 'file:///p/glowcap.cav' });
  for (const location of Object.values(ADAPTER_LOCATIONS)) assert.ok(!rewritten.includes(location));
  assert.ok(rewritten.includes('"file:///r/glue.js"'));
  // Nothing but the three locations changed.
  const strip = (value) => value.replace(/"file:\/\/\/[^"]+"|'[^']*dist\/pkg-reactive[^']*'|'\.\/glowcap\.cav'|, import\.meta\.url/g, '');
  assert.equal(strip(rewritten), strip(text));
  assert.throws(() => rewriteAdapter(text.replace(ADAPTER_LOCATIONS.glue, "'elsewhere.js'"), { glueUrl: '', wasmUrl: '', programUrl: '' }), /expected/);
});

test('event classes follow their rules in order, and every effect kind has one', () => {
  const accepted = (effects, changed = []) => ({ outcome: 'accepted', effects, changed });
  assert.equal(classify({ outcome: 'rejected', effects: [], changed: [] }), 'refused');
  assert.equal(classify(accepted(['reveal', 'commit', 'reopen'])), 'commit+reopen');
  assert.equal(classify(accepted(['reveal', 'commit'])), 'commit');
  assert.equal(classify(accepted(['qualify', 'reopen'])), 'reopen');
  assert.equal(classify(accepted(['renew', 'reveal'])), 'evidence');
  assert.equal(classify(accepted(['sample'])), 'evidence');
  assert.equal(classify(accepted(['qualify'], ['states.x'])), 'qualify');
  assert.equal(classify(accepted(['renew'])), 'renew');
  assert.equal(classify(accepted([], ['states.glow'])), 'state-changing');
  assert.equal(classify(accepted([])), 'idle');
  assert.throws(() => classify(accepted(['teleport'])), /unknown effect kind/);
  assert.deepEqual(CLASS_ORDER, CLASS_RULES.map(([name]) => name));
  assert.equal(signature('advance', 'reopen', ['reopen', 'qualify', 'reopen']), 'advance reopen [qualify,reopen]');
  assert.equal(signature('merge', 'refused', [], 'reject'), 'merge refused [] reject');
});

test('only volatile fields and clock states leave an event idle', () => {
  const before = { sequence: 1, elapsed: 0, last_event: 'tick', effects: [], states: { now: 1, glow: 3 }, graph: { a: 1 } };
  assert.deepEqual(changedParts(before, { ...before, sequence: 2, elapsed: 0.05, states: { now: 1.05, glow: 3 } }, ['now']), []);
  assert.deepEqual(changedParts(before, { ...before, states: { now: 1, glow: 2 } }, ['now']), ['states.glow']);
  assert.deepEqual(changedParts(before, { ...before, graph: { a: 2 } }, ['now']), ['graph']);
  assert.equal(glowcapCategory('tick', 'idle'), 'idle tick');
  assert.equal(glowcapCategory('tick', 'state-changing'), 'state-changing tick');
  assert.equal(glowcapCategory('absorb', 'commit'), 'observation (commit)');
});

test('every suite names only workloads the manifest has', async () => {
  const ids = new Set((await loadManifest()).workloads.map((workload) => workload.id));
  for (const suite of Object.values(SUITES)) for (const job of suite.jobs) assert.ok(ids.has(job.workload), job.workload);
});

// A smoke suite (one repeat, any power plan) truncates its streams; any other
// suite is a measurement: full streams, with the baseline's passes.
test('smoke suites are marked and short; the others measure as the baseline does', () => {
  for (const [name, suite] of Object.entries(SUITES)) {
    assert.equal(Boolean(suite.smoke), name.endsWith('smoke'), name);
    if (suite.smoke) {
      assert.ok(suite.maxEvents > 0, name);
      continue;
    }
    assert.equal(suite.maxEvents, null, name);
    assert.equal(suite.repeatCap, null, name);
    for (const key of ['rounds', 'warmup', 'publishedRounds']) assert.equal(suite[key], SUITES.baseline[key], `${name}: ${key}`);
  }
  assert.deepEqual(SUITES['view-path-smoke'].jobs, SUITES['view-path'].jobs);
});
