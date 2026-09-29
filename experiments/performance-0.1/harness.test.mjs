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
import { performanceCores } from './lib/environment.mjs';
import { ADAPTER_LOCATIONS, rewriteAdapter } from './lib/adapter.mjs';
import { SUITES } from './lib/suites.mjs';
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
  assert.equal(performanceCores(null), null);
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
