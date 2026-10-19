// session.dispatchView(event, payload, { fields }): the same transaction as
// dispatchView(), with only the named view fields. Each check drives twins,
// one through dispatchView() and the other with a selection, and requires the
// selected fields to be exactly the full view's, in its order, after every
// event, with the same session.
import test from 'node:test';
import assert from 'node:assert/strict';
import { CaveatError, VIEW_FIELDS, createRuntime } from '../lib/session.mjs';
import { Real, real, thermostat, trail } from './helpers.mjs';

const state = session => ({ save: session.save(), snapshot: session.snapshotText(), view: session.viewText() });

const pick = (view, fields) => Object.fromEntries(
  Object.entries(view).filter(([key]) => ['schema', 'sequence', 'last_event'].includes(key) || fields.includes(key)),
);

function step(full, lean, fields, event, payload = {}) {
  const expected = full.dispatchView(event, payload);
  const outcome = lean.dispatchView(event, payload, { fields });
  const at = `${event} ${JSON.stringify(payload)} ${JSON.stringify(fields)}`;
  if (expected.outcome === 'accepted') {
    assert.deepEqual(Object.keys(outcome.view), Object.keys(pick(expected.view, fields)), `${at}: the view's order`);
    assert.deepEqual(outcome, { ...expected, view: pick(expected.view, fields) }, at);
  } else {
    assert.deepEqual(outcome, expected, at);
  }
  assert.deepEqual(state(lean), state(full), `${at}: the same session`);
}

const SELECTIONS = [
  [],
  ['bindings'],
  ['bindings', 'binding_explanations'],
  ['relations', 'commitments'],
  ['decision_journal', 'cues', 'effects'],
  [...VIEW_FIELDS],
];

const TRAIL_EVENTS = [
  ['advance', { dt: 1 }],
  ['observe', { target: 'stone', method: 'report', condition: 1 }],
  ['plan', { target: 'stone' }],
  ['change', { target: 'stone', condition: 'blocked' }],
  ['observe', { target: 'stone', method: 'scout', condition: 2 }],
  ['plan', { target: 'reed' }],
  ['advance', { dt: 30 }],
  ['rescue', {}],
];

test('a field selection gives exactly the full view\'s fields, in its order, and the same session', () => {
  for (const fields of SELECTIONS) {
    for (const [source, events] of [
      [thermostat, [17, 25, 5, 40, 0].map(value => ['read', { value }])],
      [trail, TRAIL_EVENTS],
    ]) {
      const full = real.open(source);
      const lean = real.open(source);
      try {
        for (const [event, payload] of events) step(full, lean, fields, event, payload);
      } finally { full.close(); lean.close(); }
    }
  }
});

test('an unknown or repeated field throws TypeError before the event and leaves the session usable', () => {
  const session = real.open(thermostat);
  try {
    const before = state(session);
    for (const options of [{ fields: ['world'] }, { fields: ['bindings', 'bindings'] }, { fields: 'bindings' }, 7]) {
      assert.throws(() => session.dispatchView('read', { value: 17 }, options), TypeError, JSON.stringify(options));
      assert.deepEqual(state(session), before, JSON.stringify(options));
    }
    assert.equal(session.dispatchView('read', { value: 17 }, { fields: ['bindings'] }).outcome, 'accepted');
    assert.equal(session.dispatchView('read', { value: 18 }, {}).view.relations !== undefined, true, 'no fields: the whole view');
  } finally { session.close(); }
});

test('a runtime build without selection throws CaveatError("load"), and the session goes on', () => {
  // A stand-in for a runtime that has dispatchView but not its selection.
  class Earlier {
    constructor(source) { this.inner = new Real(source); }
    dispatch_outcome(event, payload) { return this.inner.dispatch_outcome(event, payload); }
    dispatch_view_outcome(event, payload) { return this.inner.dispatch_view_outcome(event, payload); }
    snapshot() { return this.inner.snapshot(); }
    view() { return this.inner.view(); }
    save() { return this.inner.save(); }
    free() { this.inner.free(); }
  }
  const session = createRuntime(Earlier).open(thermostat);
  try {
    const before = session.save();
    assert.throws(() => session.dispatchView('read', { value: 17 }, { fields: ['bindings'] }), error => error instanceof CaveatError && error.kind === 'load' && /predates dispatchView fields/.test(error.message));
    assert.equal(session.state, 'open');
    assert.equal(session.save(), before);
    assert.equal(session.dispatchView('read', { value: 17 }).outcome, 'accepted');
  } finally { session.close(); }
});
