import test from 'node:test';
import assert from 'node:assert/strict';
import { tasks } from './private/oracles.mjs';
import { assertProjection, assertAdmission, assertGrounds, oracleStep, casesFor } from './checks.mjs';

test('Scoring detects changed output, list order and acceptance', () => {
  const task = { project: state => state, projectActual: raw => raw };
  const expected = { number: 1, evidence: ['b_first', 'a_second'] };
  assertProjection(task, expected, structuredClone(expected));
  assert.throws(() => assertProjection(task, expected, { ...expected, number: 2 }));
  assert.throws(() => assertProjection(task, expected, { ...expected, evidence: [...expected.evidence].reverse() }));
  assertAdmission(true, true);
  assert.throws(() => assertAdmission(false, true));
  assert.throws(() => assertAdmission(true, false));
});

test('Grounds checking detects fabricated evidence and caveats', () => {
  const source = { evidence: ['real'], caveats: ['uncertain'] };
  const snapshot = { value_grounds: { x: structuredClone(source) }, qualified_values: { x: { provenance: source } },
    commitment_grounds: { choice: structuredClone(source) }, commitment_bases: { choice: { provenance: source } } };
  assertGrounds(snapshot);
  const fakeEvidence = structuredClone(snapshot); fakeEvidence.value_grounds.x.evidence.push('invented');
  assert.throws(() => assertGrounds(fakeEvidence));
  const fakeCaveat = structuredClone(snapshot); fakeCaveat.commitment_grounds.choice.caveats.push('invented');
  assert.throws(() => assertGrounds(fakeCaveat));
});

for (const [id, task] of Object.entries(tasks)) {
  test(`Task ${id}: deterministic cases, immutable oracle, atomic rejections, frozen journal`, () => {
    const cases = casesFor(task, id);
    assert.deepEqual(casesFor(task, id), cases);
    assert(task.cases.length >= 10);
    assert.equal(new Set(cases.map(item => item.id)).size, cases.length);
    for (const current of cases) {
      if (current.kind === 'seeded') assert.equal(current.events.length, 40);
      let state = task.initial();
      for (const event of current.events) {
        const before = task.project(state);
        if (event.resume === true) continue;
        const result = oracleStep(task, state, event);
        const after = task.project(result.state);
        assert.deepEqual(after.decision_journal.slice(0, before.decision_journal.length), before.decision_journal);
        for (const [name, grounds] of Object.entries(before.commitment_grounds)) assert.deepEqual(after.commitment_grounds[name], grounds);
        state = result.state;
      }
    }
  });
}

function reduce(id, events) {
  const task = tasks[id];
  let state = task.initial();
  for (const [event, payload = {}] of events) {
    const result = oracleStep(task, state, { event, payload });
    assert.equal(result.accepted, true, `${id}: ${event}`);
    state = result.state;
  }
  return task.project(state);
}

test('Independent spot check A: old basis expires while newer sensor reading remains fresh', () => {
  const result = reduce('A', [['read', { temperature: 1 }], ['decide'], ['advance', { dt: 1 }],
    ['read', { temperature: 8 }], ['advance', { dt: 1 }], ['decide']]);
  assert.deepEqual(result.hud, { readings: 2, temperature: 8, recommendation: 'block', decision: 'block', frozen: 0, revision: 2, elapsed: 2 });
  assert.deepEqual(result.decision_journal, [
    { decision: 'dispatch', commitment: 'dispatch@1', change: 'committed', sequence: 2, event: 'decide', elapsed: 0, value: 1, because: ['sensor_reading'], caveats: ['calibration_uncertain'] },
    { decision: 'dispatch', commitment: 'dispatch@1', change: 'reopened', sequence: 5, event: 'advance', elapsed: 2, value: 1, because: ['sensor_reading'], caveats: ['calibration_uncertain', 'stale'] },
    { decision: 'dispatch', commitment: 'dispatch@2', change: 'committed', sequence: 6, event: 'decide', elapsed: 2, value: 0, because: ['sensor_reading@2'], caveats: ['calibration_uncertain'] },
  ]);
  assert.deepEqual(result.commitment_grounds['dispatch@1'], { evidence: ['sensor_reading'], caveats: ['calibration_uncertain'] });
});

test('Independent spot check B: multiple challenges, then fresh grounds excluding revocation', () => {
  const result = reduce('B', [['submit', { code: 2 }], ['approve'], ['submit', { code: 3 }],
    ['submit', { code: 5 }], ['revoke', { code: 2 }], ['approve']]);
  assert.deepEqual(result.hud, { support: 0, opposition: 1, verdict: 'deny', decision: 'deny', frozen: 0, revision: 2, elapsed: 0 });
  assert.deepEqual(result.decision_journal.map(({ change, sequence, because, caveats }) => ({ change, sequence, because, caveats })), [
    { change: 'committed', sequence: 2, because: ['beta_allow'], caveats: ['unverified_source'] },
    { change: 'reopened', sequence: 3, because: ['alpha_deny'], caveats: ['unverified_source'] },
    { change: 'reopened', sequence: 4, because: ['challenge'], caveats: [] },
    { change: 'committed', sequence: 6, because: ['alpha_deny'], caveats: ['unverified_source'] },
  ]);
  assert.deepEqual(result.commitment_grounds['access@1'], { evidence: ['beta_allow'], caveats: ['unverified_source'] });
  assert.deepEqual(result.commitment_grounds['access@2'], { evidence: ['alpha_deny'], caveats: ['unverified_source'] });
});

test('Independent spot check C: selected diagnosis, retained repair caveat and two distinct failures', () => {
  const result = reduce('C', [['inspect', { cause: 1, score: 2 }], ['inspect', { cause: 2, score: 1 }],
    ['choose', { repair: 1 }], ['outcome', { success: 0 }], ['next'],
    ['inspect', { cause: 2, score: 2 }], ['choose', { repair: 2 }], ['outcome', { success: 0 }]]);
  assert.deepEqual(result.hud, { investigation: 2, tokens: 0, bearing: -1, motor: 2, phase: 'failed', selected: 2, frozen: 2, revision: 2 });
  assert.deepEqual(result.commitment_grounds, {
    'repair@1': { evidence: ['bearing_check'], caveats: ['alternative_cause', 'repair_unverified'] },
    'repair@2': { evidence: ['motor_check@2'], caveats: ['alternative_cause', 'repair_unverified'] },
  });
  assert.deepEqual(result.decision_journal.filter(entry => entry.change === 'reopened').map(({ because, caveats }) => ({ because, caveats })), [
    { because: ['failure'], caveats: [] }, { because: ['failure@2'], caveats: [] },
  ]);
});

test('Fractional clock expiry uses subtraction at the registered boundary', () => {
  let state = tasks.A.initial();
  for (const [event, payload] of [['advance', { dt: 0.1 }], ['advance', { dt: 0.2 }],
    ['read', { temperature: 1 }], ['decide', {}], ['advance', { dt: 1 }], ['advance', { dt: 1 }]]) {
    state = oracleStep(tasks.A, state, { event, payload }).state;
  }
  assert.equal(state.elapsed, 2.3);
  assert.equal(tasks.A.project(state).hud.decision, 'release');
  state = oracleStep(tasks.A, state, { event: 'advance', payload: { dt: 0.1 } }).state;
  assert.equal(tasks.A.project(state).hud.decision, 'review');
});
