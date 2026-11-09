import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { CaveatError, createRuntime, loadRuntime, payloadText } from '../lib/session.mjs';
import { faulty, Real, real, thermostat } from './helpers.mjs';

test('dispatch returns accepted and rejected outcomes as values', () => {
  const session = real.open(thermostat);
  try {
    const accepted = session.dispatch('read', { value: 17 });
    assert.equal(accepted.outcome, 'accepted');
    assert.equal(accepted.snapshot.bindings.heating.text, '100%');
    assert.deepEqual(session.dispatch('read', { value: 41 }), {
      schema: 'caveat-dispatch/0.1', outcome: 'rejected', origin: 'input', code: 'bound_exceeded', message: 'value must be finite and in 0..40',
    });
    assert.equal(session.dispatch('warm').code, 'unknown_event');
    assert.equal(session.dispatch('read', null).code, 'payload_invalid');
  } finally { session.close(); }
});

test('a save restores with the same source', () => {
  const first = real.open(thermostat);
  first.dispatch('read', { value: 17 });
  const second = real.restore(thermostat, first.save());
  try {
    assert.deepEqual(second.snapshot(), first.snapshot());
    assert.deepEqual(second.dispatch('read', { value: 25 }), first.dispatch('read', { value: 25 }));
  } finally { first.close(); second.close(); }
});

test('a negative zero the program computes survives a save', () => {
  // The kit sends a payload's -0 as 0, but the program computes -2 * 0 = -0.
  // The save used to leave x out as equal to the 0 it loads, and hud.angle
  // kept showing atan2(0, -1).
  const source = 'state x = 0 min -9 max 9; event e v min -9 max 9; on e set x = v * 0; bind hud.angle = atan2(x, -1);';
  const first = real.open(source);
  let second;
  try {
    assert.equal(first.dispatch('e', { v: -2 }).outcome, 'accepted');
    assert.ok(Object.is(first.snapshot().values.x, -0));
    assert.equal(first.view().bindings.hud.angle, -Math.PI);
    second = real.restore(source, first.save());
    // Strict deep equality tells -0 from 0.
    assert.deepEqual(second.snapshot(), first.snapshot());
    assert.deepEqual(second.view(), first.view());
  } finally { first.close(); second?.close(); }
});

test('load and restore failures are typed', () => {
  assert.throws(() => real.open('this is not caveat'), error => error instanceof CaveatError && error.kind === 'load');
  assert.throws(() => real.restore(thermostat, '{"not":"a save"}'), error => error instanceof CaveatError && error.kind === 'restore');
  assert.throws(() => real.restore(thermostat, {}), TypeError);
});

test('a save whose graph holds a renewal its renewals do not list fails to restore', () => {
  // Issue #43: accepted, it made the next renewal a fatal outcome instead.
  const source = 'claim safe; evidence bite from "a bite"; renewable bite limit 3; event re; on re renew bite;';
  const session = real.open(source);
  try {
    assert.equal(session.dispatch('re').outcome, 'accepted');
    const saved = JSON.parse(session.save());
    assert.deepEqual(saved.renewals, { bite: ['bite', 'bite@2'] });
    const edited = JSON.stringify({ ...saved, renewals: {}, effects: [] });
    assert.throws(() => real.restore(source, edited), error => error instanceof CaveatError && error.kind === 'restore'
      && error.message === 'cannot restore save: renewable bite occurrence bite@2 is not in its renewals');
  } finally { session.close(); }
});

const sampling = 'claim safe; evidence plain from "a plain reading"; readings flow from plain limit 4; event rd x min -10 max 10; on rd sample flow = x supports safe;';

test('a save whose graph holds a reading its stream does not list fails to restore', () => {
  // Accepted, it made the next sample a fatal outcome instead.
  const session = real.open(sampling);
  try {
    for (const x of [1, 2]) assert.equal(session.dispatch('rd', { x }).outcome, 'accepted');
    const saved = JSON.parse(session.save());
    const { flow } = saved.reading_streams;
    assert.deepEqual(flow.occurrences.map(reading => reading.id), ['flow@1', 'flow@2']);
    const edited = JSON.stringify({ ...saved, reading_streams: { flow: { ...flow, current: 'flow@1', occurrences: flow.occurrences.slice(0, 1) } } });
    assert.throws(() => real.restore(sampling, edited), error => error instanceof CaveatError && error.kind === 'restore'
      && error.message === 'cannot restore save: reading stream flow occurrence flow@2 is not in its occurrences');
  } finally { session.close(); }
});

test('a save whose stream lists other than its readings in order fails to restore', () => {
  // Accepted, a stream listing its template could make the next event on its latest reading fatal.
  const session = real.open(sampling);
  try {
    assert.equal(session.dispatch('rd', { x: 1 }).outcome, 'accepted');
    const saved = JSON.parse(session.save());
    const { flow } = saved.reading_streams;
    const edited = JSON.stringify({ ...saved, reading_streams: { flow: { ...flow, current: 'plain', occurrences: [...flow.occurrences, { ...flow.occurrences[0], id: 'plain' }] } } });
    assert.throws(() => real.restore(sampling, edited), error => error instanceof CaveatError && error.kind === 'restore'
      && error.message === 'cannot restore save: reading stream flow occurrence plain is out of order');
  } finally { session.close(); }
});

test('a save whose sequence no session reaches fails to restore', () => {
  // Accepted at u64::MAX, it made the next event fatal: "reactive event sequence exhausted".
  const session = real.open(thermostat);
  try {
    assert.equal(session.dispatch('read', { value: 17 }).outcome, 'accepted');
    const saved = JSON.parse(session.save());
    const at = sequence => JSON.stringify({ ...saved, sequence: 'here' }).replace('"sequence":"here"', `"sequence":${sequence}`);
    for (const sequence of ['18446744073709551615', String(2 ** 53)]) {
      assert.throws(() => real.restore(thermostat, at(sequence)), error => error instanceof CaveatError && error.kind === 'restore'
        && error.message === 'cannot restore save: sequence must be at most 9007199254740991', sequence);
    }
    // The largest integer a JSON host reads exactly still restores, and plays on.
    const resumed = real.restore(thermostat, at(Number.MAX_SAFE_INTEGER));
    try {
      assert.equal(resumed.dispatch('read', { value: 25 }).snapshot.sequence, 2 ** 53);
    } finally { resumed.close(); }
  } finally { session.close(); }
});

test('a save relating kinds of node no event relates fails to restore', () => {
  // Accepted, go@1 relying on a claim made the next revision of go a fatal outcome instead.
  const source = 'claim safe; evidence seen from "seen"; decisions go limit 3; event see; event redo; on see reveal seen supports safe; on see commit go because enough using qualified(1, seen); on redo reopen go because seen; on redo commit go because enough using qualified(2, seen);';
  const session = real.open(source);
  try {
    assert.equal(session.dispatch('see').outcome, 'accepted');
    const saved = JSON.parse(session.save());
    assert.deepEqual(saved.graph.relations, [['seen', 'supports', 'safe'], ['go@1', 'relies_on', 'seen']]);
    const edited = JSON.stringify({ ...saved, graph: { ...saved.graph, relations: [['seen', 'supports', 'safe'], ['go@1', 'relies_on', 'safe']] } });
    assert.throws(() => real.restore(source, edited), error => error instanceof CaveatError && error.kind === 'restore'
      && error.message === 'cannot restore save: relation go@1 relies_on safe: safe is a claim, not evidence');
  } finally { session.close(); }
});

test('payloads that JSON would change are refused before the session is touched', () => {
  for (const bad of [NaN, Infinity, -Infinity, undefined, () => 1, new Date(0), new Map(), { a: [1, NaN] }, { a: undefined }, 10n]) {
    assert.throws(() => payloadText(bad), error => error instanceof CaveatError && error.kind === 'payload', String(bad));
  }
  assert.equal(payloadText({ a: [1, 'x', null, true, { b: -0 }] }), '{"a":[1,"x",null,true,{"b":0}]}');
  const { runtime, sessions } = faulty();
  const session = runtime.open(thermostat);
  assert.throws(() => session.dispatch('read', { value: NaN }), error => error.kind === 'payload');
  assert.deepEqual(sessions[0].calls, [], 'the runtime was not called');
  assert.equal(session.dispatch('read', { value: 17 }).outcome, 'accepted', 'the session is still usable');
  session.close();
});

test('after a fatal outcome every call fails without reaching the runtime', () => {
  const { runtime, sessions } = faulty({
    dispatch() { throw JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'fatal', code: 'unclassified', message: 'boom' }); },
  });
  const session = runtime.open(thermostat);
  assert.throws(() => session.dispatch('read', { value: 17 }), error => error.kind === 'fatal' && error.report.code === 'unclassified');
  for (const call of ['snapshot', 'view', 'save', 'snapshotText', 'viewText']) {
    assert.throws(() => session[call](), error => error.kind === 'fatal', call);
  }
  assert.throws(() => session.dispatch('read', { value: 17 }), error => error.kind === 'fatal');
  session.close();
  assert.deepEqual(sessions[0].calls, ['dispatch'], 'not even free()');
  assert.equal(session.state, 'closed');
});

test('a closed session refuses calls', () => {
  const session = real.open(thermostat);
  session.close();
  session.close();
  assert.throws(() => session.snapshot(), error => error.kind === 'closed');
});

test('a trap marks the whole runtime unusable', () => {
  const { runtime } = faulty({ dispatch() { throw new WebAssembly.RuntimeError('unreachable'); } });
  const session = runtime.open(thermostat);
  assert.throws(() => session.dispatch('read', { value: 17 }), error => error.kind === 'fatal');
  assert.equal(runtime.trapped, true);
  assert.throws(() => runtime.open(thermostat), error => error.kind === 'fatal' && /trapped/.test(error.message));
});

// Glowcap round 8, item 7: a host that opened sessions and never closed them
// ran the runtime out of memory, and the only message was "unreachable".
test('round8_item7_unfreed_sessions: a trap from exhausted memory names the cause and the open sessions', () => {
  let exhausted = false;
  class Exhausting {
    constructor() { if (exhausted) throw new WebAssembly.RuntimeError('unreachable'); }
    static restore() { return new Exhausting(); }
    static out_of_memory() { return exhausted; }
    free() {}
  }
  const runtime = createRuntime(Exhausting);
  const kept = [runtime.open(thermostat), runtime.open(thermostat), runtime.open(thermostat)];
  kept[2].close();
  assert.equal(runtime.openSessions, 2, 'a closed session is no longer counted');
  exhausted = true;
  assert.throws(() => runtime.open(thermostat), {
    name: 'CaveatError',
    kind: 'fatal',
    message: 'the runtime ran out of WebAssembly memory with 2 session(s) open (trap: unreachable). '
      + 'Close each session with close() once it is no longer needed. This runtime instance is now unusable; load it again.',
  });
  assert.equal(runtime.trapped, true);
  assert.equal(kept.length, 3);
});

test('round8_item7_unfreed_sessions: any other trap keeps its own message', () => {
  const { runtime } = faulty({ dispatch() { throw new WebAssembly.RuntimeError('unreachable'); } });
  const session = runtime.open(thermostat);
  assert.throws(() => session.dispatch('read', { value: 17 }), error => error.kind === 'fatal' && error.message === 'unreachable');
});

// D070 review: restore takes a state save of any size, so a restore can run
// out of memory however few sessions are open, and the message blamed the
// open sessions and said to close them. A stand-in runtime whose memory runs
// out at the call named `at`; it never reads the save.
function exhaustingAt(at) {
  let exhausted = false;
  const trap = call => { if (exhausted && call === at) throw new WebAssembly.RuntimeError('unreachable'); };
  class Exhausting {
    constructor() { trap('open'); }
    static restore() { trap('restore'); return new Exhausting(); }
    static check() { trap('check'); return '{}'; }
    static check_options() { trap('check_options'); return '{}'; }
    static interface() { trap('interface'); return '{}'; }
    static out_of_memory() { return exhausted; }
    dispatch_outcome() { trap('dispatch'); return '{}'; }
    save() { trap('save'); return '{}'; }
    free() {}
  }
  return { runtime: createRuntime(Exhausting), exhaust: () => { exhausted = true; } };
}

// The message for a restore or check of a save that ran out of memory with
// `others` other sessions open.
const savingMessage = (during, length, others) => `the runtime ran out of WebAssembly memory while ${during} a save of ${length} characters, `
  + `with ${others} other session(s) open (trap: unreachable). `
  + `${during === 'checking' ? 'Checking' : 'Restoring'} a save needs memory that grows with its length. `
  + (others > 0 ? 'Close each session with close() once it is no longer needed. ' : '')
  + 'This runtime instance is now unusable; load it again.';

// The message for any other call that ran out of memory with `open` sessions open.
const openSessionsMessage = open => `the runtime ran out of WebAssembly memory with ${open} session(s) open (trap: unreachable). `
  + 'Close each session with close() once it is no longer needed. This runtime instance is now unusable; load it again.';

test('d070_restore_out_of_memory: a restore or check of a save that runs out of memory names it and the save\'s length', () => {
  const saved = 'x'.repeat(12345);
  for (const [at, call, during] of [
    ['restore', runtime => runtime.restore(thermostat, saved), 'restoring'],
    // check restores the save itself, then reads it again inside check_options.
    ['restore', runtime => runtime.check(thermostat, { records: true, save: saved }), 'checking'],
    ['check_options', runtime => runtime.check(thermostat, { records: true, save: saved }), 'checking'],
  ]) {
    for (const others of [0, 1]) {
      const { runtime, exhaust } = exhaustingAt(at);
      const open = Array.from({ length: others }, () => runtime.open(thermostat));
      exhaust();
      assert.throws(() => call(runtime), error => {
        assert.ok(error instanceof CaveatError, at);
        assert.equal(error.kind, 'fatal', at);
        assert.equal(error.message, savingMessage(during, 12345, others), at);
        return true;
      });
      assert.equal(runtime.trapped, true, at);
      for (const session of open) {
        assert.throws(() => session.snapshot(), error => error.kind === 'fatal' && /trapped/.test(error.message), at);
      }
    }
  }
});

// The README's leak pattern, a batch of saves resumed and never closed: a
// small save's restore runs out because of the sessions, and the message
// still says to close them.
test('d070_restore_out_of_memory: a restore that runs out among unclosed sessions still says to close them', () => {
  const { runtime, exhaust } = exhaustingAt('restore');
  const saved = 's'.repeat(2219);
  const kept = Array.from({ length: 1756 }, () => runtime.restore(thermostat, saved));
  assert.equal(runtime.openSessions, 1756);
  exhaust();
  assert.throws(() => runtime.restore(thermostat, saved), {
    name: 'CaveatError',
    kind: 'fatal',
    message: 'the runtime ran out of WebAssembly memory while restoring a save of 2219 characters, '
      + 'with 1756 other session(s) open (trap: unreachable). Restoring a save needs memory that grows with its length. '
      + 'Close each session with close() once it is no longer needed. This runtime instance is now unusable; load it again.',
  });
  assert.equal(kept.length, 1756);
});

test('d070_restore_out_of_memory: a check without a save keeps the open-sessions message', () => {
  const { runtime, exhaust } = exhaustingAt('check_options');
  runtime.open(thermostat);
  exhaust();
  assert.throws(() => runtime.check(thermostat, { records: true }), {
    name: 'CaveatError',
    kind: 'fatal',
    message: openSessionsMessage(1),
  });
  assert.equal(runtime.trapped, true);
});

// Every call that neither restores nor checks a save keeps the message as it
// was: open, the program load check does before it restores a save,
// interface, and a session's dispatch and save.
test('d070_restore_out_of_memory: every other call keeps the open-sessions message', () => {
  const saved = 'x'.repeat(12345);
  for (const [at, call, open] of [
    ['open', runtime => runtime.open(thermostat), 1],
    ['open', runtime => runtime.check(thermostat, { records: true, save: saved }), 1],
    ['interface', runtime => runtime.interface(thermostat), 1],
    ['dispatch', (_runtime, session) => session.dispatch('read', { value: 17 }), 1],
    ['save', (_runtime, session) => session.save(), 1],
  ]) {
    const { runtime, exhaust } = exhaustingAt(at);
    const session = runtime.open(thermostat);
    exhaust();
    assert.throws(() => call(runtime, session), {
      name: 'CaveatError',
      kind: 'fatal',
      message: openSessionsMessage(open),
    }, at);
    assert.equal(runtime.trapped, true, at);
  }
});

test('the real runtime reports whether it has run out of memory', () => {
  assert.equal(Real.out_of_memory(), false);
});

test('a runtime without dispatch_outcome is refused at load', async () => {
  const old = { default: async () => {}, WebReactiveSession: class { dispatch() {} } };
  await assert.rejects(loadRuntime({ module: old, wasm: null }), error => error.kind === 'load' && /predates/.test(error.message));
  assert.equal(typeof createRuntime(class {}).open, 'function');
});

// Regressions from review of 603a20d.

test('a trap in one session makes its siblings unusable, and closing never calls into the instance', () => {
  const { runtime, sessions } = faulty({
    dispatch(session, event, payload) {
      if (session.number === 1) throw new WebAssembly.RuntimeError('unreachable');
      return session.inner.dispatch_outcome(event, payload);
    },
  });
  const first = runtime.open(thermostat);
  const second = runtime.open(thermostat);
  assert.equal(second.dispatch('read', { value: 17 }).outcome, 'accepted');
  assert.throws(() => first.dispatch('read', { value: 17 }), error => error.kind === 'fatal');
  const before = sessions[1].calls.length;
  for (const call of ['snapshot', 'view', 'save']) {
    assert.throws(() => second[call](), error => error.kind === 'fatal' && /trapped/.test(error.message), call);
  }
  assert.throws(() => second.dispatch('read', { value: 25 }), error => error.kind === 'fatal');
  first.close();
  second.close();
  assert.equal(sessions[1].calls.length, before, 'the sibling made no call after the trap');
  assert.ok(!sessions[0].calls.includes('free') && !sessions[1].calls.includes('free'), 'no free() on a trapped instance');
});

test('a trap during free() marks the runtime trapped for every session', () => {
  const { runtime, sessions } = faulty({ free() { throw new WebAssembly.RuntimeError('unreachable'); } });
  const first = runtime.open(thermostat);
  const second = runtime.open(thermostat);
  first.close();
  assert.equal(runtime.trapped, true);
  assert.throws(() => second.snapshot(), error => error.kind === 'fatal' && /trapped/.test(error.message));
  second.close();
  assert.deepEqual(sessions[1].calls.filter(call => call === 'free'), [], 'the second session is not freed');
  assert.throws(() => runtime.open(thermostat), error => error.kind === 'fatal');
});

for (const call of ['snapshot', 'snapshotText', 'view', 'viewText', 'save']) {
  test(`malformed JSON from ${call}() is fatal and ends the session`, () => {
    const hook = call.replace('Text', '');
    const { runtime, sessions } = faulty({ [hook]: () => '{"truncated":' });
    const session = runtime.open(thermostat);
    assert.throws(() => session[call](), error => error instanceof CaveatError && error.kind === 'fatal' && /not JSON/.test(error.message));
    assert.equal(session.state, 'fatal');
    const calls = sessions[0].calls.length;
    assert.throws(() => session.dispatch('read', { value: 17 }), error => error.kind === 'fatal');
    assert.equal(sessions[0].calls.length, calls, 'no call after the fatal read');
  });
}

test('holes in payload arrays are refused instead of becoming null', () => {
  const holes = [[1, undefined, 3], { a: new Array(2) }, [new Array(1)]];
  delete holes[0][1];
  for (const sparse of holes) {
    assert.throws(() => payloadText(sparse), error => error.kind === 'payload' && /hole/.test(error.message), String(sparse));
  }
  assert.equal(payloadText([null, 1]), '[null,1]', 'an explicit null is still allowed');
});

// Regressions from review of 4e2ef26.

test('loaders over the same instance share its trap, and loading again after a trap gives a fresh instance', async () => {
  const url = new URL('../../dist/pkg-reactive/caveat_runtime.js', import.meta.url).href;
  const wasm = await readFile(new URL('../../dist/pkg-reactive/caveat_runtime_bg.wasm', import.meta.url));
  const first = await loadRuntime({ module: url, wasm });
  const second = await loadRuntime({ module: url, wasm });
  const one = first.open(thermostat);
  const two = second.open(thermostat);
  // A trap inside the shared instance, reached through the first loader.
  const namespace = await import(url);
  const { prototype } = namespace.WebReactiveSession;
  const original = prototype.dispatch_outcome;
  prototype.dispatch_outcome = () => { throw new WebAssembly.RuntimeError('unreachable'); };
  try {
    assert.throws(() => one.dispatch('read', { value: 17 }), error => error.kind === 'fatal');
  } finally {
    prototype.dispatch_outcome = original;
  }
  assert.equal(second.trapped, true, 'the other loader sees the trap');
  assert.throws(() => two.snapshot(), error => error.kind === 'fatal' && /trapped/.test(error.message));
  assert.throws(() => second.open(thermostat), error => error.kind === 'fatal');
  one.close();
  two.close();

  const fresh = await loadRuntime({ module: url, wasm });
  assert.equal(fresh.trapped, false);
  const three = fresh.open(thermostat);
  assert.equal(three.dispatch('read', { value: 17 }).outcome, 'accepted', 'the fresh instance works');
  assert.equal(first.trapped, true, 'the trapped instance stays trapped');
  three.close();
  await assert.rejects(loadRuntime({ module: namespace, wasm }), error => error.kind === 'fatal' && /module URL/.test(error.message));
});

test('runtimes created from the same session class share one lifecycle', () => {
  const { runtime, sessions } = faulty();
  runtime.open(thermostat).close();
  const Class = sessions[0].constructor;
  const a = createRuntime(Class);
  const b = createRuntime(Class);
  a.markTrapped();
  assert.equal(b.trapped, true);
  assert.equal(runtime.trapped, true);
  assert.throws(() => b.open(thermostat), error => error.kind === 'fatal');
});

test('a compact save is smaller and restores the same session', () => {
  const first = real.open(thermostat);
  for (const value of [17, 21, 25]) first.dispatch('read', { value });
  const compact = first.save({ compact: true });
  assert.equal(JSON.parse(compact).schema, 'caveat-replay-save/0.1');
  assert.ok(compact.length < first.save().length);
  const second = real.restore(thermostat, compact);
  try {
    assert.deepEqual(second.snapshot(), first.snapshot());
    assert.equal(second.save(), first.save());
    assert.deepEqual(second.dispatch('read', { value: 30 }), first.dispatch('read', { value: 30 }));
    // A session restored from a state save holds no replay save.
    const third = real.restore(thermostat, first.save());
    assert.equal(third.save({ compact: true }), third.save());
    third.close();
  } finally { first.close(); second.close(); }
});

test('a positional compact save is smaller still and restores the same session', () => {
  const first = real.open(thermostat);
  for (const value of [17, 21, 25]) first.dispatch('read', { value });
  const named = first.save({ compact: true });
  const positional = first.save({ compact: true, positional: true });
  assert.equal(JSON.parse(positional).schema, 'caveat-replay-save/0.2');
  assert.ok(positional.length < named.length);
  // Asking for it leaves the named compact save as it was.
  assert.equal(first.save({ compact: true }), named);
  const second = real.restore(thermostat, positional);
  try {
    assert.deepEqual(second.snapshot(), first.snapshot());
    assert.equal(second.save({ compact: true }), named);
    assert.deepEqual(second.dispatch('read', { value: 30 }), first.dispatch('read', { value: 30 }));
  } finally { first.close(); second.close(); }
});

test('a compact save on a runtime build that predates it throws a load error and changes nothing', () => {
  const { runtime } = faulty();
  const session = runtime.open(thermostat);
  session.dispatch('read', { value: 17 });
  assert.throws(() => session.save({ compact: true }), error => error.kind === 'load' && /predates/.test(error.message));
  assert.throws(() => session.save({ compact: true, positional: true }), error => error.kind === 'load' && /predates/.test(error.message));
  assert.equal(session.state, 'open');
  assert.equal(typeof session.save(), 'string');
  session.close();
});

test('an accepted snapshot is the one the event produced, however late it is read', () => {
  const lazy = real.open(thermostat);
  const eager = real.open(thermostat);
  try {
    const first = lazy.dispatch('read', { value: 17 });
    const expected = eager.dispatch('read', { value: 17 }).snapshot;
    lazy.dispatch('read', { value: 25 });
    assert.deepEqual(first.snapshot, expected);
    assert.equal(first.snapshot, first.snapshot, 'parsed once');
    assert.deepEqual(JSON.parse(JSON.stringify(first)), { schema: 'caveat-dispatch/0.1', outcome: 'accepted', snapshot: expected });
    eager.dispatch('read', { value: 25 });
    assert.deepEqual({ ...lazy.dispatch('read', { value: 20 }) }.snapshot, eager.dispatch('read', { value: 20 }).snapshot);
    first.snapshot = 1;
    assert.equal(first.snapshot, 1);
  } finally { lazy.close(); eager.close(); }
});

test('snapshot() after dispatch reuses the event\'s snapshot until the session changes', () => {
  const { runtime, sessions } = faulty();
  const session = runtime.open(thermostat);
  const outcome = session.dispatch('read', { value: 17 });
  assert.deepEqual(session.snapshot(), outcome.snapshot);
  assert.notEqual(session.snapshot(), session.snapshot(), 'a fresh object each time');
  assert.equal(sessions[0].calls.includes('snapshot'), false);
  session.dispatchView('read', { value: 18 });
  assert.equal(session.snapshot().sequence, 2);
  assert.equal(sessions[0].calls.filter(call => call === 'snapshot').length, 1);
  session.close();
});

test('a rejected event keeps the snapshot the session had', () => {
  const session = real.open(thermostat);
  try {
    session.dispatch('read', { value: 17 });
    assert.equal(session.dispatch('read', { value: 41 }).outcome, 'rejected');
    assert.deepEqual(session.snapshot(), JSON.parse(session.snapshotText()));
    session.dispatchView('read', { value: 18 });
    assert.equal(session.dispatch('warm').outcome, 'rejected');
    assert.deepEqual(session.snapshot(), JSON.parse(session.snapshotText()));
    assert.equal(session.snapshot().sequence, 2);
  } finally { session.close(); }
});
