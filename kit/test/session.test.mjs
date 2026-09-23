import test from 'node:test';
import assert from 'node:assert/strict';
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
