import test from 'node:test';
import assert from 'node:assert/strict';
import { CaveatError, createRuntime } from '../lib/session.mjs';
import { savedEvents } from '../lib/saves.mjs';

// A bridge recorder isolates the public option and lifetime contract from
// compiler behavior, which the native and compiler-compatibility tests cover.
function recordingRuntime({ older = false, fail = null } = {}) {
  const calls = [];
  class Bridge {
    static restore(...args) { calls.push(['restore', ...args]); return new Bridge(); }
    static restore_options(...args) {
      calls.push(['restore_options', ...args]);
      if (fail) throw fail;
      return new Bridge();
    }
    free() { calls.push(['free']); }
  }
  if (older) delete Bridge.restore_options;
  return { runtime: createRuntime(Bridge), calls };
}

test('restore forwards each explicit binding choice and retains the exact save text', () => {
  const saved = '{"states":{"n":{"value":-0.0}}}\n';
  for (const moduleBindings of ['scoped', 'legacy-hold-capture']) {
    const { runtime, calls } = recordingRuntime();
    const session = runtime.restore('exact source\r\n', saved, { moduleBindings });
    assert.deepEqual(calls, [['restore_options', 'exact source\r\n', saved, JSON.stringify({ moduleBindings })]]);
    assert.equal(runtime.openSessions, 1);
    session.close();
    assert.equal(runtime.openSessions, 0);
    assert.deepEqual(calls.at(-1), ['free']);
  }
});

test('ordinary restore and empty options retain the older bridge path', () => {
  for (const options of [undefined, {}, { moduleBindings: undefined }, Object.create(null)]) {
    const { runtime, calls } = recordingRuntime({ older: true });
    runtime.restore('source', 'save', options).close();
    assert.deepEqual(calls, [['restore', 'source', 'save'], ['free']]);
  }
});

test('invalid restore options fail before entering the runtime', () => {
  const bad = [null, true, 1, 'scoped', [], new Date(), () => {},
    { moduleBindings: null }, { moduleBindings: false }, { moduleBindings: 'guess' },
    { unknown: true }, { moduleBindings: 'scoped', typo: true },
    { [Symbol('choice')]: 'scoped' }, Object.create({ moduleBindings: 'scoped' })];
  for (const options of bad) {
    const { runtime, calls } = recordingRuntime();
    assert.throws(() => runtime.restore('source', 'save', options), TypeError);
    assert.deepEqual(calls, []);
    assert.equal(runtime.openSessions, 0);
  }
});

test('an older runtime cannot silently ignore an explicit binding choice', () => {
  const { runtime, calls } = recordingRuntime({ older: true });
  assert.throws(() => runtime.restore('source', 'save', { moduleBindings: 'scoped' }),
    error => error instanceof CaveatError && error.kind === 'load' && /restore_options/.test(error.message));
  assert.deepEqual(calls, []);
  assert.equal(runtime.openSessions, 0);
});

test('a binding-choice failure remains a restore error and does not open a session', () => {
  const { runtime } = recordingRuntime({ fail: 'choice conflicts with saved module bindings' });
  assert.throws(() => runtime.restore('source', 'save', { moduleBindings: 'scoped' }),
    error => error instanceof CaveatError && error.kind === 'restore' && /conflicts/.test(error.message));
  assert.equal(runtime.trapped, false);
  assert.equal(runtime.openSessions, 0);
});

test('a trap in the options bridge invalidates its runtime just like ordinary restore', () => {
  const { runtime } = recordingRuntime({ fail: new WebAssembly.RuntimeError('unreachable') });
  assert.throws(() => runtime.restore('source', 'save', { moduleBindings: 'scoped' }),
    error => error instanceof CaveatError && error.kind === 'fatal');
  assert.equal(runtime.trapped, true);
  assert.throws(() => runtime.restore('source', 'save'), error => error.kind === 'fatal');
  assert.equal(runtime.openSessions, 0);
});

test('savedEvents uses restored signatures and closes on decoding errors', () => {
  const signatures = [{ name: 'report', parameters: [
    { name: 'target', domain: { entity: { kind: 'gate', members: ['north', 'south'] } } },
    { name: 'stance', domain: { member: { members: ['clear', 'guarded'] } } },
    { name: 'who', domain: { identifier: { limit: 8 } } },
    { name: 'amount' },
  ] }];
  let restored = 0;
  let closed = 0;
  const runtime = {
    restore(_source, _save) {
      restored += 1;
      return { snapshot: () => ({ events: signatures }), close: () => { closed += 1; } };
    },
    interface() { assert.fail('fresh loading must not choose a different interpretation'); },
  };
  const expected = [{ event: 'report', payload: { target: 'south', stance: 'guarded', who: 'ocelot', amount: -0 } }];
  for (const saved of [
    '{"schema":"caveat-replay-save/0.1","events":[["report",{"target":2,"stance":2,"who":"ocelot","amount":-0.0}]]}',
    '{"schema":"caveat-replay-save/0.2","events":[0,2,2,"ocelot",-0.0]}',
  ]) assert.deepEqual(savedEvents(runtime, 'legacy source', saved), expected);
  assert.throws(() => savedEvents(runtime, 'legacy source', '{"schema":"caveat-replay-save/0.2","events":[0,3,2,"x",0]}'), /has no such member/);
  assert.throws(() => savedEvents(runtime, 'legacy source', '{"schema":"caveat-reactive-save/0.1"}'), /state save holds no events/);
  assert.throws(() => savedEvents(runtime, 'legacy source', 'not JSON'), SyntaxError);
  assert.equal(restored, 5);
  assert.equal(closed, restored);
});
