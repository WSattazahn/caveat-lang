import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { CaveatError, createRuntime, loadRuntime, payloadText } from '../lib/session.mjs';
import { faulty, real, thermostat } from './helpers.mjs';

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

test('load and restore failures are typed', () => {
  assert.throws(() => real.open('this is not caveat'), error => error instanceof CaveatError && error.kind === 'load');
  assert.throws(() => real.restore(thermostat, '{"not":"a save"}'), error => error instanceof CaveatError && error.kind === 'restore');
  assert.throws(() => real.restore(thermostat, {}), TypeError);
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
