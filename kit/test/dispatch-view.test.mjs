// session.dispatchView(): the transaction, payload handling and failures of
// dispatch(), with the view in place of the snapshot. Each check drives twins,
// one through dispatch() and view() and the other through dispatchView(), and
// requires the same outcome and the same session after every event.
import test from 'node:test';
import assert from 'node:assert/strict';
import { CaveatError, createRuntime } from '../lib/session.mjs';
import { Real, faulty, real, thermostat, trail } from './helpers.mjs';
import { provenanceOverflow } from './fatal-fixture.mjs';

const state = session => ({ save: session.save(), snapshot: session.snapshotText(), view: session.viewText() });

// One event through both paths. Returns the new path's outcome.
function step(old, viewed, event, payload = {}) {
  const before = state(viewed);
  const expected = old.dispatch(event, payload);
  const outcome = viewed.dispatchView(event, payload);
  const at = `${event} ${JSON.stringify(payload)}`;
  if (expected.outcome === 'accepted') {
    assert.deepEqual(Object.keys(outcome), ['schema', 'outcome', 'view'], at);
    assert.deepEqual(outcome, { schema: expected.schema, outcome: 'accepted', view: old.view() }, at);
    assert.deepEqual(outcome.view, viewed.view(), `${at}: the view is what view() then returns`);
  } else {
    assert.deepEqual(outcome, expected, at);
    assert.deepEqual(state(viewed), before, `${at}: a refusal changes nothing`);
  }
  assert.deepEqual(state(viewed), state(old), `${at}: the same session`);
  return outcome;
}

function twins(source, run) {
  const old = real.open(source);
  const viewed = real.open(source);
  try { run(old, viewed); } finally { old.close(); viewed.close(); }
}

test('dispatchView returns the view the session then shows, and leaves the session dispatch() and view() leave', () => {
  twins(thermostat, (old, viewed) => {
    const accepted = step(old, viewed, 'read', { value: 17 });
    assert.equal(accepted.view.bindings.heating.text, '100%');
    assert.equal(accepted.view.schema, 'caveat-reactive-view/0.1');
    assert.equal(Object.hasOwn(accepted.view, 'world'), false, 'the view, not the snapshot');
    for (const value of [25, 5, 40, 0]) step(old, viewed, 'read', { value });
  });
  twins(trail, (old, viewed) => {
    for (const [event, payload] of [
      ['advance', { dt: 1 }],
      ['observe', { target: 'stone', method: 'report', condition: 1 }],
      ['plan', { target: 'stone' }],
      ['change', { target: 'stone', condition: 'blocked' }],
      ['observe', { target: 'stone', method: 'scout', condition: 2 }],
      ['plan', { target: 'reed' }],
      ['advance', { dt: 30 }],
      ['rescue', {}],
    ]) step(old, viewed, event, payload);
  });
});

const DECISIONS = `claim safe; evidence gauge from "a gauge";
  readings depth from gauge limit 4; decisions route limit 4;
  event read value min 0 max 9; event decide;
  on read sample depth = value supports safe;
  on decide commit route because enough using latest(depth);`;

const PERMISSION = `identifiers limit 8; claim ready; claim may_merge;
  evidence ci from "checks"; evidence go from "a go-ahead";
  readings checks from ci limit 4; readings approvals from go limit 4; decisions merge limit 2;
  state head = 0;
  event pushed commit id; event check; event approved commit id; event merge;
  on pushed set head = commit;
  on check sample checks = 1 supports ready;
  on approved sample approvals = commit supports may_merge;
  on merge commit merge because enough using latest(checks) permitted by latest(approvals) for head;`;

const LATE_FADE = `claim safe;
  evidence bite from "a bite"; evidence chart from "tidal archive";
  caveat faded consequence low; renewable bite limit 4;
  state from_chart = 0; state eaten = 0 min 0 max 9;
  event tick dt min 0 max 0.1; event eat; event regrow; event chart;
  on chart reveal chart supports safe;
  on chart set from_chart = qualified(1, chart);
  on eat reveal bite supports safe;
  on eat set eaten = eaten + 1;
  on eat qualify bite with faded after 0.15;
  on regrow renew bite;
  bind hud.text = "ok" because nothing;
  bind hud.text = "faded" when carries(bite, faded) because from_chart;`;

const tick = ['tick', { dt: 0.1 }];

// Every code of Dispatch 0.1 a program can reach. limit/depth_limit is only
// the runtime's defensive guard: a program that deep does not load, and the
// runtime's tests check that both paths classify it alike.
const REFUSALS = [
  ['policy', 'reject', 'state n = 0 min 0 max 9; event e; on e set n = n + 1; on e when n > 1 reject "Not now.";', [['e']], ['e']],
  ['policy', 'not_permitted', PERMISSION, [['pushed', { commit: 'a' }], ['check']], ['merge']],
  ['input', 'unknown_event', thermostat, [['read', { value: 17 }]], ['warm']],
  ['input', 'payload_invalid', thermostat, [['read', { value: 17 }]], ['read', { value: 'x' }]],
  ['input', 'payload_invalid', thermostat, [], ['read', null]],
  ['input', 'payload_invalid', thermostat, [], ['read', { value: 17, extra: 1 }]],
  ['input', 'bound_exceeded', thermostat, [['read', { value: 17 }]], ['read', { value: 41 }]],
  ['evaluation', 'bound_exceeded', 'state output = 0 min 0 max 1; event set_value value min 0 max 2; on set_value set output = value; bind hud.value = output;', [['set_value', { value: 0.5 }]], ['set_value', { value: 2 }]],
  ['evaluation', 'decision_in_force', DECISIONS, [['read', { value: 1 }], ['decide']], ['decide']],
  ['evaluation', 'ungrounded_citation', LATE_FADE, [['chart'], ['eat'], tick, ['regrow'], ['eat'], tick], tick],
  ['limit', 'work_limit', `state output = 0; event run; proc many() { ${'set output = output + 1;'.repeat(2048)} }; on run call many(); on run call many();`, [], ['run']],
  ['limit', 'history_limit', 'claim safe; evidence gauge from "a gauge"; readings depth from gauge limit 1; event read value min 0 max 9; on read sample depth = value supports safe;', [['read', { value: 1 }]], ['read', { value: 2 }]],
  ['limit', 'identifier_limit', 'identifiers limit 1; state head = 0; event pushed commit id; on pushed set head = commit; bind pr.head = id_text(head);', [['pushed', { commit: 'a' }]], ['pushed', { commit: 'b' }]],
  ['limit', 'renewal_limit', 'claim safe; evidence bite from "a bite"; renewable bite limit 1; event regrow; proc again() { renew bite; }; on regrow call again();', [], ['regrow']],
];

test('every refusal is the object dispatch() returns, and changes nothing', () => {
  for (const [origin, code, source, before, [event, payload]] of REFUSALS) {
    twins(source, (old, viewed) => {
      for (const [prior, priorPayload] of before) {
        assert.equal(step(old, viewed, prior, priorPayload).outcome, 'accepted', `${code}: ${prior}`);
      }
      const refused = step(old, viewed, event, payload);
      assert.deepEqual([refused.outcome, refused.origin, refused.code], ['rejected', origin, code]);
      // Refused again, the same way: the refusal left nothing behind.
      assert.deepEqual(step(old, viewed, event, payload), refused, code);
    });
  }
});

test('a fatal outcome is the report dispatch() throws, and ends the session', () => {
  for (const rule of ['on run', 'on run when output > 0']) {
    const source = `state output = 0; event run; on run set output = 1; ${provenanceOverflow(rule)}`;
    twins(source, (old, viewed) => {
      let expected;
      assert.throws(() => old.dispatch('run'), error => { expected = error; return error.kind === 'fatal'; });
      assert.throws(() => viewed.dispatchView('run'), error => {
        assert.ok(error instanceof CaveatError);
        assert.equal(error.kind, 'fatal');
        assert.equal(error.message, expected.message);
        assert.deepEqual(error.report, expected.report);
        assert.equal(error.report.code, 'unclassified');
        return true;
      });
      assert.equal(viewed.state, 'fatal');
      for (const call of ['dispatchView', 'dispatch', 'snapshot', 'view', 'save']) {
        assert.throws(() => viewed[call]('run'), error => error.kind === 'fatal', call);
      }
    });
  }
});

test('after a fatal outcome of either path, neither reaches the runtime', () => {
  for (const [path, hook] of [['dispatch', 'dispatch'], ['dispatchView', 'dispatchView']]) {
    const { runtime, sessions } = faulty({
      [hook]() { throw JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'fatal', code: 'unclassified', message: 'boom' }); },
    });
    const session = runtime.open(thermostat);
    assert.throws(() => session[path]('read', { value: 17 }), error => error.kind === 'fatal' && error.report.message === 'boom');
    for (const call of ['dispatchView', 'dispatch', 'snapshot', 'view', 'save']) {
      assert.throws(() => session[call]('read', { value: 17 }), error => error.kind === 'fatal', call);
    }
    session.close();
    assert.deepEqual(sessions[0].calls, [path], 'not even free()');
  }
});

test('a trap in dispatchView marks the whole runtime unusable', () => {
  const { runtime, sessions } = faulty({ dispatchView() { throw new WebAssembly.RuntimeError('unreachable'); } });
  const first = runtime.open(thermostat);
  const second = runtime.open(thermostat);
  assert.throws(() => first.dispatchView('read', { value: 17 }), error => error.kind === 'fatal');
  assert.equal(runtime.trapped, true);
  assert.throws(() => second.dispatchView('read', { value: 17 }), error => error.kind === 'fatal' && /trapped/.test(error.message));
  assert.throws(() => runtime.open(thermostat), error => error.kind === 'fatal');
  first.close();
  second.close();
  assert.deepEqual(sessions[1].calls, [], 'the sibling made no call');
});

test('text from dispatch_view_outcome that is not a view outcome is fatal', () => {
  const view = { schema: 'caveat-reactive-view/0.1', sequence: 1 };
  for (const [text, pattern] of [
    ['{"truncated":', /not JSON/],
    // An accepted outcome carrying the snapshot is dispatch()'s, not this one.
    [JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'accepted', snapshot: view }), /unrecognised/],
    [JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'accepted', view: [] }), /unrecognised/],
    [JSON.stringify({ schema: 'caveat-dispatch/0.2', outcome: 'accepted', view }), /unrecognised/],
    [JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'rejected', origin: 'host', code: 'reject', message: 'no' }), /unrecognised/],
    [JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'fatal', code: 'unclassified', message: 'returned' }), /unrecognised/],
  ]) {
    const { runtime, sessions } = faulty({ dispatchView: () => text });
    const session = runtime.open(thermostat);
    assert.throws(() => session.dispatchView('read', { value: 17 }), error => error instanceof CaveatError && error.kind === 'fatal' && pattern.test(error.message), text);
    assert.equal(session.state, 'fatal');
    assert.throws(() => session.view(), error => error.kind === 'fatal');
    assert.deepEqual(sessions[0].calls, ['dispatchView'], text);
  }
});

test('payloads and event names are refused as dispatch() refuses them, before the runtime is called', () => {
  const sparse = [1, 2, 3];
  delete sparse[1];
  const { runtime, sessions } = faulty();
  const session = runtime.open(thermostat);
  const failure = run => { try { run(); } catch (error) { return { type: error.constructor.name, kind: error.kind, message: error.message }; } return null; };
  for (const bad of [NaN, Infinity, () => 1, new Date(0), new Map(), { value: NaN }, { value: [1, NaN] }, { value: undefined }, sparse, 10n]) {
    const expected = failure(() => session.dispatch('read', bad));
    assert.equal(expected?.kind, 'payload', String(bad));
    assert.deepEqual(failure(() => session.dispatchView('read', bad)), expected, String(bad));
  }
  for (const event of ['', 3, null, undefined, {}]) {
    const expected = failure(() => session.dispatch(event));
    assert.equal(expected?.type, 'TypeError');
    assert.deepEqual(failure(() => session.dispatchView(event)), expected, String(event));
  }
  assert.deepEqual(sessions[0].calls, [], 'the runtime was not called');
  assert.equal(session.dispatchView('read', { value: 17 }).outcome, 'accepted', 'the session is still usable');
  session.close();
});

test('a closed session refuses dispatchView', () => {
  const session = real.open(thermostat);
  session.close();
  assert.throws(() => session.dispatchView('read', { value: 17 }), error => error instanceof CaveatError && error.kind === 'closed');
});

test('a runtime build without dispatch_view_outcome refuses dispatchView, and the session goes on', () => {
  // A stand-in for an earlier runtime: every method but the new one.
  class Earlier {
    constructor(source) { this.inner = new Real(source); }
    dispatch_outcome(event, payload) { return this.inner.dispatch_outcome(event, payload); }
    snapshot() { return this.inner.snapshot(); }
    view() { return this.inner.view(); }
    save() { return this.inner.save(); }
    free() { this.inner.free(); }
  }
  const session = createRuntime(Earlier).open(thermostat);
  try {
    const before = session.save();
    assert.throws(() => session.dispatchView('read', { value: 17 }), error => error instanceof CaveatError && error.kind === 'load' && /predates dispatchView/.test(error.message));
    assert.equal(session.state, 'open');
    assert.equal(session.save(), before);
    assert.equal(session.dispatch('read', { value: 17 }).outcome, 'accepted');
  } finally { session.close(); }
});
