// The runner's automatic checks, each shown to catch an injected runtime fault
// and to name the session that misbehaved.
import test from 'node:test';
import assert from 'node:assert/strict';
import { formatFileReport, runScenarioFile } from '../lib/scenarios.mjs';
import { Real, faulty, real, scenarioFile, sourceReader, thermostat } from './helpers.mjs';

const readSource = sourceReader({ 'thermostat.cav': thermostat });
const run = (runtime, steps, options = {}) => runScenarioFile(scenarioFile('thermostat.cav', steps), { runtime, readSource, ...options });
const only = async (runtime, steps) => (await run(runtime, steps)).scenarios[0];
const rejectedOutcome = text => JSON.parse(text).outcome === 'rejected';

test('an unfaulted runtime passes the same steps used below', async () => {
  const result = await only(real, [
    { send: 'read', payload: { value: 17 } },
    { resume: true },
    { send: 'read', payload: { value: 41 }, rejected: { origin: 'input', code: 'bound_exceeded' } },
    { send: 'read', payload: { value: 25 } },
  ]);
  assert.equal(result.pass, true, JSON.stringify(result.failure));
});

test('a rejected event that changes state is caught in the primary session', async () => {
  const { runtime } = faulty({
    dispatch(session, event, payload) {
      const text = session.inner.dispatch_outcome(event, payload);
      if (rejectedOutcome(text)) session.inner.dispatch_outcome('read', '{"value":20}');
      return text;
    },
  });
  const result = await only(runtime, [
    { send: 'read', payload: { value: 17 } },
    { send: 'read', payload: { value: 41 }, rejected: { origin: 'input' } },
  ]);
  assert.equal(result.pass, false);
  assert.equal(result.failure.kind, 'atomicity');
  assert.equal(result.failure.category, 'runtime-invariant');
  assert.equal(result.failure.session, 'primary');
  assert.equal(result.failure.step, 2);
  assert.match(result.failure.message, /rejected event changed the save/);
  assert.ok(result.failure.path.startsWith('/'), result.failure.path);
});

test('a rejected event that changes state only in the pre-resume session names that shadow', async () => {
  const { runtime } = faulty({
    dispatch(session, event, payload) {
      const text = session.inner.dispatch_outcome(event, payload);
      if (!session.restored && rejectedOutcome(text)) session.inner.dispatch_outcome('read', '{"value":20}');
      return text;
    },
  });
  const result = await only(runtime, [
    { send: 'read', payload: { value: 17 } },
    { resume: true },
    { send: 'read', payload: { value: 41 }, rejected: { origin: 'input' } },
  ]);
  assert.equal(result.failure.kind, 'atomicity');
  assert.equal(result.failure.session, 'shadow 1');
  assert.equal(result.failure.step, 3);
});

test('a restore that loses history is caught at the resume step', async () => {
  const { runtime } = faulty({ restore: source => new Real(source) });
  const result = await only(runtime, [
    { send: 'read', payload: { value: 17 } },
    { resume: true },
  ]);
  assert.equal(result.failure.kind, 'resume');
  assert.equal(result.failure.category, 'runtime-invariant');
  assert.equal(result.failure.session, 'primary');
  assert.equal(result.failure.step, 2);
  assert.match(result.failure.message, /restoring the save changed the snapshot/);
});

test('a restore that goes wrong only at the end is caught by the final restore', async () => {
  const { runtime } = faulty({ restore: source => new Real(source) });
  const result = await only(runtime, [{ send: 'read', payload: { value: 17 } }]);
  assert.equal(result.failure.kind, 'final-resume');
  assert.equal(result.failure.session, 'final restore');
  assert.equal(result.failure.step, 2);
});

test('a resumed session that later reaches a different snapshot is caught', async () => {
  const { runtime } = faulty({
    dispatch(session, event, payload) {
      if (!session.restored) return session.inner.dispatch_outcome(event, payload);
      const changed = JSON.parse(payload);
      if (typeof changed.value === 'number') changed.value += 1;
      return session.inner.dispatch_outcome(event, JSON.stringify(changed));
    },
  });
  const result = await only(runtime, [
    { send: 'read', payload: { value: 17 } },
    { resume: true },
    { send: 'read', payload: { value: 25 } },
  ]);
  assert.equal(result.failure.kind, 'agreement');
  assert.equal(result.failure.session, 'shadow 1');
  assert.match(result.failure.message, /different snapshot/);
  assert.ok(result.failure.path, 'names the first differing path');
});

test('a pre-resume session that disagrees on the outcome is caught and named', async () => {
  let resumed = false;
  const { runtime } = faulty({
    restore(source, saved) { resumed = true; return Real.restore(source, saved); },
    dispatch(session, event, payload) {
      if (resumed && !session.restored) {
        return JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'rejected', origin: 'policy', code: 'reject', message: 'made up' });
      }
      return session.inner.dispatch_outcome(event, payload);
    },
  });
  const result = await only(runtime, [
    { send: 'read', payload: { value: 17 } },
    { resume: true },
    { send: 'read', payload: { value: 25 } },
  ]);
  assert.equal(result.failure.kind, 'agreement');
  assert.equal(result.failure.session, 'shadow 1');
  assert.match(result.failure.message, /disagreed on the outcome/);
  assert.deepEqual(result.failure.actual, { outcome: 'rejected', origin: 'policy', code: 'reject', message: 'made up' });
});

test('grounds outside lineage after an accepted event are a runtime invariant failure', async () => {
  const { runtime } = faulty({
    dispatch(session, event, payload) {
      const outcome = JSON.parse(session.inner.dispatch_outcome(event, payload));
      if (outcome.outcome === 'accepted') {
        for (const grounds of Object.values(outcome.snapshot.commitment_grounds)) grounds.evidence.push('ghost');
      }
      return JSON.stringify(outcome);
    },
  });
  const result = await only(runtime, [{ send: 'read', payload: { value: 17 } }]);
  assert.equal(result.failure.kind, 'grounds');
  assert.equal(result.failure.category, 'runtime-invariant');
  assert.equal(result.failure.session, 'primary');
  assert.equal(result.failure.path, '/commitment_grounds/heating@1/evidence');
  assert.match(result.failure.message, /ghost/);
});

test('grounds outside lineage in a new session are caught before the first step', async () => {
  const { runtime } = faulty({
    snapshot(session) {
      const snapshot = JSON.parse(session.inner.snapshot());
      snapshot.value_grounds = { ...snapshot.value_grounds, invented: { evidence: ['ghost'], caveats: [] } };
      return JSON.stringify(snapshot);
    },
  });
  const result = await only(runtime, [{ send: 'read', payload: { value: 17 } }]);
  assert.equal(result.failure.kind, 'grounds');
  assert.equal(result.failure.step, 0);
});

test('a fatal outcome never satisfies an expected rejection and the session is not touched again', async () => {
  const { runtime, sessions } = faulty({
    dispatch() {
      throw JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'fatal', code: 'unclassified', message: 'boom' });
    },
  });
  const result = await only(runtime, [{ send: 'read', payload: { value: 17 }, rejected: { origin: 'evaluation' } }]);
  assert.equal(result.failure.kind, 'fatal');
  assert.equal(result.failure.category, 'fatal');
  assert.deepEqual(result.failure.outcome, { outcome: 'fatal', code: 'unclassified', message: 'boom' });
  const calls = sessions[0].calls;
  assert.deepEqual(calls.slice(calls.indexOf('dispatch') + 1), [], 'no call after the fatal dispatch');
});

for (const [name, altered] of [
  ['an unknown outcome schema', outcome => ({ ...outcome, schema: 'caveat-dispatch/9' })],
  ['an unknown origin', () => ({ schema: 'caveat-dispatch/0.1', outcome: 'rejected', origin: 'host', code: 'x', message: 'x' })],
  ['an unknown outcome', outcome => ({ ...outcome, outcome: 'maybe' })],
]) {
  test(`${name} is fatal, not a rejection`, async () => {
    const { runtime } = faulty({
      dispatch(session, event, payload) { return JSON.stringify(altered(JSON.parse(session.inner.dispatch_outcome(event, payload)))); },
    });
    const result = await only(runtime, [{ send: 'read', payload: { value: 41 }, rejected: { origin: 'input' } }]);
    assert.equal(result.failure.kind, 'fatal');
    assert.match(result.failure.outcome.message, /unrecognised dispatch outcome/);
  });
}

test('after a WebAssembly trap the runner reloads the runtime for later scenarios', async () => {
  let trapped = false;
  const { runtime } = faulty({
    dispatch(session, event, payload) {
      if (!trapped) { trapped = true; throw new WebAssembly.RuntimeError('unreachable'); }
      return session.inner.dispatch_outcome(event, payload);
    },
  });
  const doc = scenarioFile('thermostat.cav', [{ send: 'read', payload: { value: 17 } }]);
  doc.scenarios.push({ id: 'S2', title: 'after the trap', steps: [{ send: 'read', payload: { value: 17 } }] });
  let reloads = 0;
  const withReload = await runScenarioFile(doc, { runtime, readSource, reload: async () => { reloads += 1; return real; } });
  assert.equal(withReload.scenarios[0].failure.kind, 'fatal');
  assert.equal(withReload.scenarios[1].pass, true);
  assert.equal(reloads, 1);

  trapped = false;
  const second = faulty({
    dispatch(session, event, payload) {
      if (!trapped) { trapped = true; throw new WebAssembly.RuntimeError('unreachable'); }
      return session.inner.dispatch_outcome(event, payload);
    },
  });
  const withoutReload = await runScenarioFile(doc, { runtime: second.runtime, readSource });
  assert.equal(withoutReload.scenarios[1].pass, false);
  assert.equal(withoutReload.scenarios[1].failure.category, 'fatal');
  assert.match(withoutReload.scenarios[1].failure.message, /trapped/);
});

// Regressions from review of 603a20d.

test('malformed snapshot JSON becomes a fatal result and later scenarios still run', async () => {
  const { runtime } = faulty({
    snapshot(session) { return session.number === 1 ? 'not json' : session.inner.snapshot(); },
  });
  const doc = scenarioFile('thermostat.cav', [{ send: 'read', payload: { value: 17 } }]);
  doc.scenarios.push({ id: 'S2', title: 'next', steps: [{ send: 'read', payload: { value: 17 } }] });
  const result = await runScenarioFile(doc, { runtime, readSource });
  assert.equal(result.scenarios[0].failure.kind, 'fatal');
  assert.equal(result.scenarios[0].failure.step, 0);
  assert.equal(result.scenarios[0].failure.session, 'primary');
  assert.match(result.scenarios[0].failure.outcome.message, /not JSON/);
  assert.equal(result.scenarios[1].pass, true);
});

test('malformed view JSON during a step is a fatal result naming the session', async () => {
  const { runtime } = faulty({
    view(session) { return session.calls.includes('dispatch') ? '[' : session.inner.view(); },
  });
  const result = await only(runtime, [{ send: 'read', payload: { value: 17 } }, { send: 'read', payload: { value: 25 } }]);
  assert.equal(result.failure.kind, 'fatal');
  assert.equal(result.failure.step, 2);
  assert.equal(result.failure.session, 'primary');
});

test('a trap while releasing sessions fails that scenario and the next one reloads', async () => {
  let trapped = false;
  const { runtime } = faulty({
    free(session) {
      if (!trapped) { trapped = true; throw new WebAssembly.RuntimeError('unreachable'); }
      return session.inner.free();
    },
  });
  const doc = scenarioFile('thermostat.cav', [{ send: 'read', payload: { value: 17 } }]);
  doc.scenarios.push({ id: 'S2', title: 'next', steps: [{ send: 'read', payload: { value: 17 } }] });
  let reloads = 0;
  const result = await runScenarioFile(doc, { runtime, readSource, reload: async () => { reloads += 1; return real; } });
  assert.equal(result.scenarios[0].pass, false);
  assert.equal(result.scenarios[0].failure.kind, 'fatal');
  assert.match(result.scenarios[0].failure.outcome.message, /releasing a session/);
  assert.equal(result.scenarios[1].pass, true);
  assert.equal(reloads, 1);
});

test('a failure reading the final restore is labelled "final restore"', async () => {
  const { runtime } = faulty({
    snapshot(session) {
      if (session.restored) throw JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'fatal', code: 'unclassified', message: 'boom' });
      return session.inner.snapshot();
    },
  });
  const result = await only(runtime, [{ send: 'read', payload: { value: 17 } }]);
  assert.equal(result.failure.kind, 'fatal');
  assert.equal(result.failure.session, 'final restore');
  assert.equal(result.failure.step, 2);
});

test('a list that grew since a checkpoint is reported with an absent marker, in text and JSON', async () => {
  const doc = scenarioFile('thermostat.cav', [
    { send: 'read', payload: { value: 17 } },
    { same_as: 'initial', paths: ['/decision_journal'] },
  ]);
  const result = await runScenarioFile(doc, { runtime: real, readSource });
  const { failure } = result.scenarios[0];
  assert.equal(failure.kind, 'same_as');
  assert.equal(failure.path, '/decision_journal/0');
  assert.equal(failure.actual.commitment, 'heating@1');
  assert.match(formatFileReport(result), /\/decision_journal\/0 was \(absent\) at "initial", now \{/);
  assert.deepEqual(JSON.parse(JSON.stringify(failure)).expected, { $absent: true });
});
