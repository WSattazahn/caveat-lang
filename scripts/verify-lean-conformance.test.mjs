import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { checkedOutput, firstDifference, modelTrace, compare, dependencyDropMutation,
  InfrastructureFailure, SemanticMismatch, runtimeTrace, verifyMutation, wasmTrace, assertSourceHashesUnchanged, checkedDecoderRejection } from './verify-lean-conformance.mjs';
import { STATES, SCHEMA } from './lean-conformance-cases.mjs';

const fixture = { id: 'test', steps: [{}] };
function trace() {
  const frame = () => ({ outcome: { outcome: 'accepted' },
    states: Object.fromEntries(STATES.map(name => [name, { value: 0,
      lineage: { evidence: ['ea'], caveats: ['ca'] }, grounds: { evidence: ['ea'], caveats: ['ca'] } }])),
    observations: ['eg', 'ea', 'eb'], decision_journal: [], effects: [] });
  const initial = frame(); initial.outcome = { outcome: 'initial' };
  return { schema: SCHEMA, id: 'test', frames: [initial, frame(), frame()] };
}

test('simultaneous lineage/grounds erasure is a semantic mismatch despite subset inclusion', () => {
  const expected = modelTrace(trace(), fixture);
  const changed = trace();
  changed.frames[2].states.x.lineage = { evidence: [], caveats: [] };
  changed.frames[2].states.x.grounds = { evidence: [], caveats: [] };
  const actual = modelTrace(changed, fixture);
  assert.throws(() => compare(expected, actual, 'test', 'mutation'), error =>
    error instanceof SemanticMismatch && /\/frames\/2\/states\/x\/grounds/.test(error.difference.path));
});

test('provenance alone is set-normalized; observation ordering remains semantic', () => {
  const expected = trace();
  const same = trace();
  same.frames[0].states.a.grounds.evidence.push('ea');
  assert.deepEqual(modelTrace(same, fixture), modelTrace(expected, fixture));
  same.frames[1].observations.reverse();
  assert.throws(() => compare(modelTrace(expected, fixture), modelTrace(same, fixture), 'test', 'reorder'), SemanticMismatch);
});

test('malformed, extra, incomplete and fatal model outputs fail infrastructure', () => {
  const mutations = [
    value => { value.extra = true; },
    value => { value.frames.pop(); },
    value => { value.frames[0].outcome = { outcome: 'fatal', code: 'unclassified' }; },
    value => { value.frames[0].states.a.value = -0; },
    value => { value.frames[0].states.a.grounds.evidence = 'ea'; },
    value => { value.frames[0].outcome.extra = true; },
    value => { value.frames[0].outcome = { outcome: 'accepted' }; },
    value => { value.frames[2].outcome = { outcome: 'initial' }; },
    value => { value.frames[1].outcome = { outcome: 'rejected', origin: 'evaluation', code: 'ungrounded_citation' }; },
    value => { value.frames[2].outcome = { outcome: 'rejected', origin: 'unknown', code: 'reject' }; },
    value => { value.frames[2].outcome = { outcome: 'rejected', origin: 'policy', code: 'unknown' }; },
    value => { value.frames[2].outcome = { outcome: 'rejected', origin: 'input', code: 'ungrounded_citation' }; },
    value => { value.frames[2].effects = [{ kind: 'reveal', evidence: 'ea', extra: true }]; },
  ];
  for (const mutate of mutations) {
    const value = trace(); mutate(value);
    assert.throws(() => modelTrace(value, fixture), InfrastructureFailure);
  }
});

test('real failed process, signal and missing binary cannot count as semantic detection', () => {
  const failed = spawnSync(process.execPath, ['-e', 'process.exit(7)'], { encoding: 'utf8' });
  for (const result of [failed, { status: null, signal: 'SIGTERM' }, { status: null, error: new Error('ENOENT') }]) {
    assert.throws(() => checkedOutput(result, 'control'), InfrastructureFailure);
  }
  assert.equal(checkedOutput({ status: 0, stdout: '{"ok":true}' }, 'ok'), '{"ok":true}');
});

test('first differing array position retains sequence order and object keys do not matter', () => {
  assert.equal(firstDifference({ a: 1, b: 2 }, { b: 2, a: 1 }), null);
  const left = Array(12).fill(0), right = [...left]; right[2] = 1; right[10] = 1;
  assert.equal(firstDifference(left, right).path, '/2');
});

test('an absent or ambiguous production mutation anchor fails instead of skipping', () => {
  assert.throws(() => dependencyDropMutation(''), InfrastructureFailure);
});


function rawTrace() {
  const records = trace().frames.map((frame, sequence) => {
    const snapshot = { schema: 'caveat-reactive/0.1', source_id: 'fixture', sequence,
      values: Object.fromEntries(STATES.map(name => [name, frame.states[name].value])),
      qualified_values: Object.fromEntries(STATES.map(name => [name, {
        value: frame.states[name].value, provenance: structuredClone(frame.states[name].lineage) }])),
      value_grounds: Object.fromEntries(STATES.map(name => [name, structuredClone(frame.states[name].grounds)])),
      observations: [...frame.observations], decision_journal: [], effects: [] };
    return { snapshot, save: JSON.stringify({ schema: 'caveat-reactive-save/0.1', source_id: 'fixture', sequence }),
      view: { schema: 'caveat-reactive-view/0.1', sequence, decision_journal: [], effects: [] } };
  });
  return { initial: records[0], events: records.slice(1).map(record => ({ ...record,
    outcome: { schema: 'caveat-dispatch/0.1', outcome: 'accepted', snapshot: structuredClone(record.snapshot) } })) };
}

test('runtime projection refuses malformed protocol instead of crediting a semantic discrepancy', () => {
  assert.deepEqual(runtimeTrace(rawTrace(), fixture), modelTrace(trace(), fixture));
  const mutations = [
    value => { value.initial.outcome = { outcome: 'initial' }; },
    value => { value.events[0].extra = true; },
    value => { delete value.events[0].outcome.schema; },
    value => { value.events[0].outcome.schema = 'other'; },
    value => { value.events[0].outcome.extra = true; },
    value => { value.events[0].outcome.snapshot.values.a = 1; },
    value => { value.events[0].outcome = { schema: 'caveat-dispatch/0.1', outcome: 'initial' }; },
    value => { value.events[1].outcome = { schema: 'caveat-dispatch/0.1', outcome: 'rejected', origin: 'policy', code: 'unknown', message: 'no' }; },
    value => { value.events[1].outcome = { schema: 'caveat-dispatch/0.1', outcome: 'rejected', origin: 'policy', code: 'reject', message: 42 }; },
    value => { value.initial.view.schema = 'other'; },
    value => { value.initial.view = []; },
    value => { value.initial.view.effects = [{ kind: 'reveal', evidence: 'ea' }]; },
    value => { value.initial.save = 'broken'; },
    value => { value.initial.save = '{}'; },
    value => { value.initial.qualified_values = {}; },
    value => { value.initial.snapshot.qualified_values.a.value = 1; },
    value => { value.initial.snapshot.qualified_values.a.extra = true; },
    value => { value.initial.snapshot.value_grounds.a.extra = []; },
    value => { delete value.events[1].snapshot.observations; delete value.events[1].outcome.snapshot.observations; },
  ];
  for (const mutate of mutations) {
    const value = rawTrace(); mutate(value);
    assert.throws(() => runtimeTrace(value, fixture), InfrastructureFailure);
  }
});

function tracked(value, evidence = [], caveats = [], groundsEvidence = evidence, groundsCaveats = caveats) {
  return { value, lineage: { evidence: [...evidence], caveats: [...caveats] },
    grounds: { evidence: [...groundsEvidence], caveats: [...groundsCaveats] } };
}
function mutationWitness(id) {
  const initial = { outcome: { outcome: 'initial' },
    states: Object.fromEntries(STATES.map(name => [name, tracked(0)])), observations: [], effects: [], decision_journal: [] };
  const seed = structuredClone(initial);
  seed.outcome = { outcome: 'accepted' };
  seed.states.a = tracked(id === 'drop-both-channels' || id === 'reorder-observations' ? 2 : 3, ['ea'], ['ca']);
  seed.states.b = tracked(-2, ['eb'], ['cb']);
  seed.states.g = tracked(id === 'skip-guard-loss' ? 0 : 1, ['eg'], ['cg']);
  seed.observations = ['eg', 'ea', 'eb'];
  seed.effects = seed.observations.map(evidence => ({ kind: 'reveal', evidence }));
  const after = structuredClone(seed); after.effects = [];
  const expected = { schema: SCHEMA, id, frames: [initial, seed, after] };
  if (id === 'drop-both-channels' || id === 'reorder-observations') {
    after.states.x = tracked(0, ['ea', 'eb'], ['ca', 'cb']);
  } else if (id === 'skip-guard-loss') {
    after.states.x = tracked(-2, ['eb'], ['cb']);
    const skipped = structuredClone(after);
    skipped.states.x = tracked(-2, ['eb', 'eg'], ['cb', 'cg'], ['eb'], ['cb']);
    expected.frames.push(skipped);
  } else if (id === 'guard-into-grounds') {
    after.states.x = tracked(3, ['ea', 'eg'], ['ca', 'cg'], ['ea'], ['ca']);
  } else if (id === 'bypass-citation') {
    expected.frames[2] = structuredClone(seed);
    expected.frames[2].outcome = { outcome: 'rejected', origin: 'evaluation', code: 'ungrounded_citation' };
    after.states.y = tracked(-2, ['eb'], ['cb']);
    expected.frames.push(after);
  }
  const actual = structuredClone(expected);
  if (id === 'drop-both-channels') actual.frames[2].states.x = tracked(0, ['eb'], ['cb']);
  if (id === 'skip-guard-loss') actual.frames[3].states.x = tracked(-2, ['eb'], ['cb']);
  if (id === 'guard-into-grounds') actual.frames[2].states.x = tracked(3, ['ea', 'eg'], ['ca', 'cg']);
  if (id === 'bypass-citation') {
    actual.frames[2].outcome = { outcome: 'accepted' }; actual.frames[2].effects = [];
    actual.frames[2].states.x = tracked(3, ['ea'], ['ca'], [], []);
    actual.frames[3].states.x = tracked(3, ['ea'], ['ca'], [], []);
  }
  if (id === 'reorder-observations') {
    for (const frame of actual.frames.slice(1)) frame.observations = ['ea', 'eg', 'eb'];
    actual.frames[1].effects = ['ea', 'eg', 'eb'].map(evidence => ({ kind: 'reveal', evidence }));
  }
  return { expected, actual };
}

test('every semantic control requires its exact intended change, not a generic mismatch', () => {
  for (const id of ['drop-both-channels', 'skip-guard-loss', 'guard-into-grounds', 'bypass-citation', 'reorder-observations']) {
    const { expected, actual } = mutationWitness(id);
    assert.ok(verifyMutation(id, expected, actual).path, id);
    assert.throws(() => verifyMutation(id, expected, expected), InfrastructureFailure, id + ': survival');
    const unrelated = structuredClone(expected); unrelated.frames[2].states.y.value = 99;
    assert.throws(() => verifyMutation(id, expected, unrelated), InfrastructureFailure, id + ': unrelated difference');
    actual.frames[2].states.y.value = 99;
    assert.throws(() => verifyMutation(id, expected, actual), InfrastructureFailure, id + ': extra difference');
  }
});

function fakeRuntime(corruptFinal = false) {
  let restores = 0;
  const saved = [];
  const closed = [];
  const session = (initial, label) => {
    let n = initial;
    return {
      snapshot: () => ({ n }), view: () => ({ n }), save: () => String(n),
      dispatch(event) {
        if (event === 'refuse') return { outcome: 'rejected', origin: 'policy', code: 'reject' };
        n += 1; return { outcome: 'accepted', snapshot: { n } };
      },
      close() { closed.push(label); },
    };
  };
  return { saved, closed, open: () => session(0, 'original'), restore(_source, save) {
    saved.push(save); restores += 1;
    return session(Number(save) + (corruptFinal && restores === 3 ? 1 : 0), 'restore' + restores);
  } };
}

test('WASM continuation restores the final nonfatal prefix and refuses a divergent final restore', async () => {
  const runtime = fakeRuntime();
  await wasmTrace(runtime, '', ['advance', 'refuse']);
  assert.deepEqual(runtime.saved, ['0', '1', '1']);
  assert.deepEqual(runtime.closed.sort(), ['original', 'restore1', 'restore2', 'restore3']);
  await assert.rejects(wasmTrace(fakeRuntime(true), '', ['advance', 'refuse']), InfrastructureFailure);
});

test('source receipt integrity refuses changed, added or removed inputs', () => {
  assert.doesNotThrow(() => assertSourceHashesUnchanged({ a: 'one', b: 'two' }, { b: 'two', a: 'one' }));
  for (const after of [{ a: 'changed', b: 'two' }, { a: 'one' }, { a: 'one', b: 'two', c: 'new' }]) {
    assert.throws(() => assertSourceHashesUnchanged({ a: 'one', b: 'two' }, after), InfrastructureFailure);
  }
});

test('decoder refusals require exact nonzero status, empty stdout and intended diagnostic', () => {
  const control = { id: 'duplicate-key', diagnostic: 'duplicate key' };
  const result = { status: 1, signal: null, stdout: '', stderr: 'caveat_compare: duplicate key' };
  assert.doesNotThrow(() => checkedDecoderRejection(result, control));
  for (const changed of [
    { status: 0 }, { status: 2 }, { stdout: '{}' }, { stderr: 'unrelated parser error' },
    { signal: 'SIGTERM' }, { error: new Error('ENOENT') },
  ]) assert.throws(() => checkedDecoderRejection({ ...result, ...changed }, control), InfrastructureFailure);
});
