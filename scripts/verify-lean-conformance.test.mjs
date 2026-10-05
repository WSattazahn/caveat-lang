import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { checkedOutput, firstDifference, modelTrace, compare, dependencyDropMutation, unselectedBranchMutation,
  InfrastructureFailure, SemanticMismatch, runtimeTrace, verifyMutation, wasmTrace, assertSourceHashesUnchanged, checkedDecoderRejection,
  lateModelFrame, lateRuntimeFrame, lateRequest, verifyLateMutation, commitmentQualificationMutation,
  reopenModelFrame, reopenRuntimeFrame, reopenRequest, verifyReopenMutation, reopenMutation } from './verify-lean-conformance.mjs';
import { STATES, SCHEMA } from './lean-conformance-cases.mjs';
import { LATE_SCHEMA, LATE_STATES, lateCases } from './lean-late-qualification-cases.mjs';
import { REOPEN_SCHEMA, REOPEN_STATES, reopenCases } from './lean-reopening-cases.mjs';

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


test('compiled mutation anchors fail closed when evaluator structure is absent or ambiguous', () => {
  const source = readFileSync(new URL('../runtime/src/reactive_expr.rs', import.meta.url), 'utf8');
  for (const mutate of [dependencyDropMutation, unselectedBranchMutation]) {
    assert.notEqual(mutate(source), source);
    assert.throws(() => mutate(''), InfrastructureFailure);
    assert.throws(() => mutate(source + source), InfrastructureFailure);
  }
  assert.throws(() => dependencyDropMutation(source, 'arbitrary'), InfrastructureFailure);
});

test('branch controls require exact missing or injected provenance on both selections', () => {
  for (const yes of [true, false]) {
    for (const id of ['branch-condition-loss', 'branch-selected-loss', 'branch-unselected-injection']) {
      const { expected } = mutationWitness('guard-into-grounds');
      expected.frames[1].states.g.value = yes ? 1 : 0;
      expected.frames[2].states.g.value = yes ? 1 : 0;
      const selected = yes ? ['ea', 'ca'] : ['eb', 'cb'];
      const unused = yes ? ['eb', 'cb'] : ['ea', 'ca'];
      expected.frames[2].states.x = tracked(yes ? 3 : -2, [selected[0], 'eg'].sort(), [selected[1], 'cg'].sort());
      const actual = structuredClone(expected), x = actual.frames[2].states.x;
      for (const field of ['lineage', 'grounds']) {
        if (id === 'branch-unselected-injection') {
          x[field].evidence.push(unused[0]); x[field].evidence.sort();
          x[field].caveats.push(unused[1]); x[field].caveats.sort();
        } else {
          const removed = id === 'branch-condition-loss' ? ['eg', 'cg'] : selected;
          x[field].evidence = x[field].evidence.filter(v => v !== removed[0]);
          x[field].caveats = x[field].caveats.filter(v => v !== removed[1]);
        }
      }
      assert.ok(verifyMutation(id, expected, actual).path);
      assert.throws(() => verifyMutation(id, expected, expected), InfrastructureFailure);
      const unrelated = structuredClone(expected); unrelated.frames[2].states.y.value = 99;
      assert.throws(() => verifyMutation(id, expected, unrelated), InfrastructureFailure);
      actual.frames[2].states.x.value = 999;
      assert.throws(() => verifyMutation(id, expected, actual), InfrastructureFailure);
    }
  }
});

const lateFixture = () => lateCases().find(item => item.id === 'late-commitment-basis');
function lateAfter() {
  const tracked = (value, evidence, caveats) => ({ value, lineage: { evidence, caveats }, grounds: { evidence, caveats } });
  return {
    outcome: { outcome: 'accepted' },
    states: Object.fromEntries(LATE_STATES.map(name => [name, tracked(1, ['ea'], ['ca', 'deep', 'late', 'meta'])])),
    commitments: {
      plan: { basis: { value: 1, evidence: ['ea', 'eb'], caveats: ['ca', 'cb'] }, grounds: { evidence: ['ea'], caveats: ['ca'] } },
      hold: { basis: { value: null, evidence: [], caveats: [] }, grounds: { evidence: [], caveats: [] } },
    },
    observations: ['eg', 'ea', 'eb'],
    effects: [{ kind: 'qualify', evidence: 'ea', caveat: 'late' }],
  };
}
const lateDocument = after => ({ schema: LATE_SCHEMA, id: 'late-commitment-basis', after });

function lateRecord(after, outcome = { schema: 'caveat-dispatch/0.1', outcome: 'accepted' }) {
  const snapshot = { schema: 'caveat-reactive/0.1', source_id: 'source', sequence: 4,
    values: Object.fromEntries(LATE_STATES.map(name => [name, after.states[name].value])),
    qualified_values: Object.fromEntries(LATE_STATES.map(name => [name, { value: after.states[name].value, provenance: after.states[name].lineage }])),
    value_grounds: Object.fromEntries(LATE_STATES.map(name => [name, after.states[name].grounds])),
    commitment_bases: Object.fromEntries(Object.entries(after.commitments).map(([name, c]) =>
      [name, { value: c.basis.value, provenance: { evidence: c.basis.evidence, caveats: c.basis.caveats } }])),
    commitment_grounds: Object.fromEntries(Object.entries(after.commitments).map(([name, c]) => [name, c.grounds])),
    observations: after.observations, effects: after.effects ?? [], decision_journal: [] };
  const view = { schema: 'caveat-reactive-view/0.1', sequence: 4, effects: snapshot.effects, decision_journal: [] };
  const save = JSON.stringify({ schema: 'caveat-reactive-save/0.1', source_id: 'source', sequence: 4 });
  return { snapshot, save, view, outcome: outcome.outcome === 'accepted' ? { ...outcome, snapshot } : outcome };
}

test('late model and runtime projections agree on shape and refuse unregistered outcomes', () => {
  const fixture = lateFixture();
  const model = lateModelFrame(lateDocument(lateAfter()), fixture);
  assert.equal(firstDifference(model, lateRuntimeFrame(lateRecord(lateAfter()))), null);
  const fatal = lateAfter(); fatal.outcome = { outcome: 'fatal', code: 'unclassified' }; delete fatal.effects;
  assert.throws(() => lateModelFrame(lateDocument(fatal), fixture), InfrastructureFailure);
  const policy = lateAfter(); policy.outcome = { outcome: 'rejected', origin: 'policy', code: 'reject' }; delete policy.effects;
  assert.throws(() => lateModelFrame(lateDocument(policy), fixture), InfrastructureFailure);
  const refused = lateAfter(); refused.outcome = { outcome: 'rejected', origin: 'evaluation', code: 'unobserved_evidence' };
  assert.throws(() => lateModelFrame(lateDocument(refused), fixture), InfrastructureFailure, 'a refusal reports no effects');
  delete refused.effects;
  assert.deepEqual(lateModelFrame(lateDocument(refused), fixture).outcome, refused.outcome);
  const extra = lateAfter(); extra.decision_journal = [];
  assert.throws(() => lateModelFrame(lateDocument(extra), fixture), InfrastructureFailure);
  const foreign = lateAfter(); foreign.commitments.invented = foreign.commitments.hold;
  assert.throws(() => lateModelFrame(lateDocument(foreign), fixture), InfrastructureFailure);
  assert.throws(() => lateModelFrame({ ...lateDocument(lateAfter()), id: 'other' }, fixture), InfrastructureFailure);
  const reveal = lateAfter(); reveal.effects = [{ kind: 'reveal', evidence: 'ea' }];
  assert.throws(() => lateRuntimeFrame(lateRecord(reveal)), InfrastructureFailure);
  const skewed = lateRecord(lateAfter()); delete skewed.snapshot.commitment_grounds.hold;
  assert.throws(() => lateRuntimeFrame(skewed), InfrastructureFailure);
});

test('late requests carry the runtime state before the step and the declared qualifier chain', () => {
  const before = lateAfter();
  const request = lateRequest(lateFixture(), 0, lateRecord(before));
  assert.equal(request.schema, LATE_SCHEMA);
  assert.deepEqual(request.qualification, { evidence: 'ea', caveat: 'late', qualifiers: ['meta', 'deep'], guard: 'g' });
  assert.deepEqual(Object.keys(request.before).sort(), ['commitments', 'observations', 'states']);
  assert.deepEqual(request.before.commitments.plan.basis, before.commitments.plan.basis);
  assert.throws(() => lateRequest(lateFixture(), 1, lateRecord(before)), InfrastructureFailure);
});

test('late mutation controls require exactly the intended commitment change', () => {
  const fixture = lateFixture();
  const q = fixture.qualifications[0];
  const expected = lateModelFrame(lateDocument(lateAfter()), fixture);
  const basis = structuredClone(expected);
  basis.commitments.plan.basis.caveats = ['ca', 'cb', 'deep', 'late', 'meta'];
  assert.ok(verifyLateMutation('qualify-reaches-commitment-basis', expected, basis, q).path);
  const records = structuredClone(basis);
  records.commitments.plan.grounds.caveats = ['ca', 'deep', 'late', 'meta'];
  assert.ok(verifyLateMutation('qualify-reaches-commitment-records', expected, records, q).path);
  for (const [id, actual] of [['qualify-reaches-commitment-basis', basis], ['qualify-reaches-commitment-records', records]]) {
    assert.throws(() => verifyLateMutation(id, expected, expected, q), InfrastructureFailure, id + ': survival');
    const unrelated = structuredClone(expected); unrelated.states.b.value = 99;
    assert.throws(() => verifyLateMutation(id, expected, unrelated, q), InfrastructureFailure, id + ': unrelated difference');
    const extra = structuredClone(actual); extra.states.b.value = 99;
    assert.throws(() => verifyLateMutation(id, expected, extra, q), InfrastructureFailure, id + ': extra difference');
  }
  assert.throws(() => verifyLateMutation('qualify-reaches-commitment-records', expected, basis, q), InfrastructureFailure);
  assert.throws(() => verifyLateMutation('qualify-reaches-commitment-basis', expected, records, q), InfrastructureFailure);
});

test('the late-qualification mutation anchor fails closed', () => {
  const source = readFileSync(new URL('../runtime/src/reactive.rs', import.meta.url), 'utf8');
  for (const field of ['basis', 'records']) {
    assert.notEqual(commitmentQualificationMutation(source, field), source);
    assert.throws(() => commitmentQualificationMutation('', field), InfrastructureFailure);
    assert.throws(() => commitmentQualificationMutation(source + source, field), InfrastructureFailure);
  }
  assert.throws(() => commitmentQualificationMutation(source, 'grounds'), InfrastructureFailure);
});

const reopenFixture = () => reopenCases().find(item => item.id === 'reopen-retains-caveats');
function reopenAfter() {
  return {
    outcome: { outcome: 'accepted' },
    commitments: {
      plan: { basis: { value: 3, evidence: ['ea'], caveats: ['ca', 'late'] }, grounds: { evidence: ['ea'], caveats: ['ca', 'late'] },
        retained: ['ca', 'late'], reopened_by: ['eb'] },
    },
    observations: ['eg', 'ea', 'eb'],
    journal: [
      { commitment: 'plan', change: 'committed', because: ['ea'], caveats: ['ca', 'late'] },
      { commitment: 'plan', change: 'reopened', because: ['eb'], caveats: ['cb', 'late', 'meta'] },
    ],
    effects: [{ kind: 'reopen', action: 'plan', because: 'eb' }],
  };
}
const reopenDocument = after => ({ schema: REOPEN_SCHEMA, id: 'reopen-retains-caveats', after });

function reopenRecord(after, outcome = { schema: 'caveat-dispatch/0.1', outcome: 'accepted' }) {
  const tracked = { value: 1, provenance: { evidence: [], caveats: [] } };
  const snapshot = { schema: 'caveat-reactive/0.1', source_id: 'source', sequence: 3,
    values: Object.fromEntries(REOPEN_STATES.map(name => [name, 1])),
    qualified_values: Object.fromEntries(REOPEN_STATES.map(name => [name, tracked])),
    value_grounds: Object.fromEntries(REOPEN_STATES.map(name => [name, { evidence: [], caveats: [] }])),
    commitments: Object.entries(after.commitments).map(([name, c]) =>
      ({ action: name, open: c.reopened_by.length > 0, retained: c.retained, reopened_by: c.reopened_by })),
    commitment_bases: Object.fromEntries(Object.entries(after.commitments).map(([name, c]) =>
      [name, { value: c.basis.value, provenance: { evidence: c.basis.evidence, caveats: c.basis.caveats } }])),
    commitment_grounds: Object.fromEntries(Object.entries(after.commitments).map(([name, c]) => [name, c.grounds])),
    observations: after.observations, effects: after.effects ?? [],
    decision_journal: after.journal.map(entry => ({ decision: entry.commitment, sequence: 2, event: 'decide', ...entry })) };
  const view = { schema: 'caveat-reactive-view/0.1', sequence: 3, effects: snapshot.effects, decision_journal: snapshot.decision_journal };
  const save = JSON.stringify({ schema: 'caveat-reactive-save/0.1', source_id: 'source', sequence: 3 });
  return { snapshot, save, view, outcome: outcome.outcome === 'accepted' ? { ...outcome, snapshot } : outcome };
}

test('reopening model and runtime projections agree on shape and refuse unregistered outcomes', () => {
  const fixture = reopenFixture();
  const model = reopenModelFrame(reopenDocument(reopenAfter()), fixture);
  assert.equal(firstDifference(model, reopenRuntimeFrame(reopenRecord(reopenAfter()))), null);
  const fatal = reopenAfter(); fatal.outcome = { outcome: 'fatal', code: 'unclassified' }; delete fatal.effects;
  assert.throws(() => reopenModelFrame(reopenDocument(fatal), fixture), InfrastructureFailure);
  const policy = reopenAfter(); policy.outcome = { outcome: 'rejected', origin: 'policy', code: 'reject' }; delete policy.effects;
  assert.throws(() => reopenModelFrame(reopenDocument(policy), fixture), InfrastructureFailure);
  for (const code of ['unobserved_evidence', 'not_committed']) {
    const refused = reopenAfter(); refused.outcome = { outcome: 'rejected', origin: 'evaluation', code };
    assert.throws(() => reopenModelFrame(reopenDocument(refused), fixture), InfrastructureFailure, 'a refusal reports no effects');
    delete refused.effects;
    assert.deepEqual(reopenModelFrame(reopenDocument(refused), fixture).outcome, refused.outcome);
  }
  const extra = reopenAfter(); extra.states = {};
  assert.throws(() => reopenModelFrame(reopenDocument(extra), fixture), InfrastructureFailure);
  const foreign = reopenAfter(); foreign.commitments.invented = foreign.commitments.plan;
  assert.throws(() => reopenModelFrame(reopenDocument(foreign), fixture), InfrastructureFailure);
  const change = reopenAfter(); change.journal[1].change = 'withdrawn';
  assert.throws(() => reopenModelFrame(reopenDocument(change), fixture), InfrastructureFailure);
  assert.throws(() => reopenModelFrame({ ...reopenDocument(reopenAfter()), id: 'other' }, fixture), InfrastructureFailure);
  const qualify = reopenAfter(); qualify.effects = [{ kind: 'qualify', evidence: 'ea', caveat: 'late' }];
  assert.throws(() => reopenRuntimeFrame(reopenRecord(qualify)), InfrastructureFailure);
  const skewed = reopenRecord(reopenAfter()); delete skewed.snapshot.commitment_grounds.plan;
  assert.throws(() => reopenRuntimeFrame(skewed), InfrastructureFailure);
});

test('reopening requests carry the runtime state before the step, the guard and the declared caveats', () => {
  const before = reopenAfter();
  const request = reopenRequest(reopenFixture(), 0, reopenRecord(before));
  assert.equal(request.schema, REOPEN_SCHEMA);
  assert.deepEqual(request.reopening, { commitment: 'plan', evidence: 'eb', caveats: ['cb', 'late', 'meta'], guard: true });
  assert.deepEqual(Object.keys(request.before).sort(), ['commitments', 'journal', 'observations']);
  assert.deepEqual(request.before.commitments.plan.retained, ['ca', 'late']);
  assert.throws(() => reopenRequest(reopenFixture(), 1, reopenRecord(before)), InfrastructureFailure);
  const guarded = reopenCases().find(item => item.id === 'reopen-guard-skipped');
  const off = reopenRecord(before); off.snapshot.values.g = 0; off.snapshot.qualified_values.g = { ...off.snapshot.qualified_values.g, value: 0 };
  assert.equal(reopenRequest(guarded, 0, off).reopening.guard, false);
});

test('reopening mutation controls require exactly the intended change', () => {
  const fixture = reopenFixture();
  const r = fixture.reopenings[0];
  const expected = reopenModelFrame(reopenDocument(reopenAfter()), fixture);
  const retained = structuredClone(expected); retained.commitments.plan.retained = [];
  assert.ok(verifyReopenMutation('reopen-drops-retained', expected, retained, r).path);
  const cause = structuredClone(expected); cause.journal[1].caveats = [];
  assert.ok(verifyReopenMutation('reopen-drops-cause-caveats', expected, cause, r).path);
  for (const [id, actual] of [['reopen-drops-retained', retained], ['reopen-drops-cause-caveats', cause]]) {
    assert.throws(() => verifyReopenMutation(id, expected, expected, r), InfrastructureFailure, id + ': survival');
    const unrelated = structuredClone(expected); unrelated.observations = ['ea'];
    assert.throws(() => verifyReopenMutation(id, expected, unrelated, r), InfrastructureFailure, id + ': unrelated difference');
    const extra = structuredClone(actual); extra.commitments.plan.reopened_by = [];
    assert.throws(() => verifyReopenMutation(id, expected, extra, r), InfrastructureFailure, id + ': extra difference');
  }
  assert.throws(() => verifyReopenMutation('reopen-drops-retained', expected, cause, r), InfrastructureFailure);
  assert.throws(() => verifyReopenMutation('reopen-drops-cause-caveats', expected, retained, r), InfrastructureFailure);
});

test('the reopening mutation anchors fail closed', () => {
  const source = readFileSync(new URL('../runtime/src/reactive.rs', import.meta.url), 'utf8');
  for (const field of ['retained', 'cause']) {
    assert.notEqual(reopenMutation(source, field), source);
    assert.throws(() => reopenMutation('', field), InfrastructureFailure);
    assert.throws(() => reopenMutation(source + source, field), InfrastructureFailure);
  }
  assert.throws(() => reopenMutation(source, 'grounds'), InfrastructureFailure);
});
