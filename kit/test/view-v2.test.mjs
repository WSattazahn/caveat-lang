// View 0.2 through the kit: view({ schema: '0.2' }) and dispatchViewDelta()
// (spec/caveat-view-0.2.md). The applier below is written from the spec's
// "The delta" table, not from the runtime's builder.
import test from 'node:test';
import assert from 'node:assert/strict';
import { CaveatError } from '../lib/session.mjs';
import { faulty, real, thermostat, trail } from './helpers.mjs';

const byAction = (a, b) => Buffer.compare(Buffer.from(a.action), Buffer.from(b.action));

function apply(view, delta) {
  assert.equal(delta.schema, 'caveat-reactive-view-delta/0.2');
  assert.equal(delta.since, view.sequence, 'the delta applies to this view');
  const next = structuredClone(view);
  Object.assign(next, { sequence: delta.sequence, last_event: delta.last_event, cues: delta.cues, effects: delta.effects });
  for (const name of ['bindings', 'binding_explanations']) {
    for (const [target, values] of Object.entries(delta[name].set)) Object.assign(next[name][target] ??= {}, values);
    for (const [target, property] of delta[name].removed) {
      delete next[name][target][property];
      if (Object.keys(next[name][target]).length === 0) delete next[name][target];
    }
  }
  const removed = new Set(delta.commitments.removed);
  const commitments = new Map(next.commitments.filter(c => !removed.has(c.action)).map(c => [c.action, c]));
  for (const commitment of delta.commitments.set) commitments.set(commitment.action, commitment);
  next.commitments = [...commitments.values()].sort(byAction);
  for (const name of ['commitment_grounds', 'decision_series']) {
    Object.assign(next[name], delta[name].set);
    for (const key of delta[name].removed) delete next[name][key];
  }
  for (const name of ['decision_journal', 'relations']) {
    const drop = new Set(delta[name].removed);
    next[name] = [...next[name].filter((_, index) => !drop.has(index)), ...delta[name].appended];
  }
  return next;
}

test('view() stays View 0.1 and view({ schema: "0.2" }) is View 0.2', () => {
  const session = real.open(thermostat);
  try {
    session.dispatch('read', { value: 17 });
    const old = session.view();
    assert.equal(old.schema, 'caveat-reactive-view/0.1');
    assert.deepEqual(session.view({ schema: '0.1' }), old);
    const next = session.view({ schema: '0.2' });
    assert.equal(next.schema, 'caveat-reactive-view/0.2');
    assert.equal(JSON.parse(session.viewText({ schema: '0.2' })).schema, 'caveat-reactive-view/0.2');
    for (const commitment of next.commitments) {
      assert.equal('open' in commitment, false);
      assert.equal(typeof commitment.reopened, 'boolean');
    }
    assert.deepEqual(next.bindings, old.bindings);
    assert.throws(() => session.view({ schema: '0.3' }), TypeError);
    assert.equal(session.state, 'open');
  } finally { session.close(); }
});

test('dispatchViewDelta keeps a held view equal to the full view, event by event', () => {
  const session = real.open(trail);
  try {
    let held = session.view({ schema: '0.2' });
    const events = [
      ['observe', { target: 'stone', method: 'report', condition: 1 }],
      ['advance', { dt: 5 }],
      ['plan', { target: 'stone' }],
      ['observe', { target: 'stone', method: 'scout', condition: 2 }],
      ['change', { target: 'stone', condition: 'blocked' }],
      ['observe', { target: 'reed', method: 'report', condition: 0 }],
      ['advance', { dt: 30 }],
      ['plan', { target: 'stone' }],
      ['rescue', {}],
      ['rescue', {}],
    ];
    let accepted = 0;
    for (const [event, payload] of events) {
      const outcome = session.dispatchViewDelta(event, payload);
      const full = session.view({ schema: '0.2' });
      if (outcome.outcome === 'accepted') {
        accepted += 1;
        held = apply(held, outcome.delta);
      } else {
        assert.equal(outcome.delta, undefined);
      }
      assert.deepEqual(held, full, `after ${event}`);
    }
    assert.ok(accepted >= 3);
  } finally { session.close(); }
});

test('a refused event returns what dispatch() returns, with no delta', () => {
  const one = real.open(thermostat);
  const two = real.open(thermostat);
  try {
    assert.deepEqual(one.dispatchViewDelta('read', { value: 99 }), two.dispatch('read', { value: 99 }));
    assert.deepEqual(one.dispatchViewDelta('nonexistent'), two.dispatch('nonexistent'));
    assert.equal(one.state, 'open');
  } finally { one.close(); two.close(); }
});

test('a runtime that predates View 0.2 refuses it and the session stays usable', () => {
  const { runtime } = faulty();
  const session = runtime.open(thermostat);
  try {
    assert.throws(() => session.view({ schema: '0.2' }), error => error instanceof CaveatError && error.kind === 'load');
    assert.throws(() => session.dispatchViewDelta('read', { value: 17 }), error => error instanceof CaveatError && error.kind === 'load');
    assert.equal(session.dispatch('read', { value: 17 }).outcome, 'accepted');
  } finally { session.close(); }
});
