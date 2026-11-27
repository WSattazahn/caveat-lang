import assert from 'node:assert/strict';
import test from 'node:test';
import { verifyReplay } from './replay-check.mjs';

const snapshot = sequence => ({ sequence, bindings: { ledger: { release: 'none yet' } } });
const initial = { record: 'initial', sequence: 0, snapshot: snapshot(0) };
const accepted = { record: 'event', line: 1, event: 'ci_run', payload: { green: 1 }, outcome: 'accepted', sequence: 1, snapshot: snapshot(1) };
const second = { record: 'event', line: 2, event: 'approve', payload: { item: 393 }, outcome: 'accepted', sequence: 2, snapshot: snapshot(2) };
const input = (...records) => records.map(({ event, payload }) => JSON.stringify({ event, payload })).join('\n');
const oneEvent = input(accepted);
const twoEvents = input(accepted, second);
const output = (...records) => ({ status: 0, stdout: records.map(record => JSON.stringify(record)).join('\n'), stderr: '' });

test('a failed spawn cannot pass even when its status is zero and stdout empty', () => {
  assert.throws(() => verifyReplay({ ...output(), error: new Error('spawnSync EPERM') }, oneEvent), /could not run.*EPERM/);
  assert.throws(() => verifyReplay({ ...output(initial, accepted), signal: 'SIGTERM' }, oneEvent), /stopped on SIGTERM/);
  assert.throws(() => verifyReplay({ ...output(initial, accepted), status: 2 }, oneEvent), /exited with status 2/);
});

test('empty, partial, malformed and unknown replay output cannot pass', () => {
  assert.throws(() => verifyReplay(output(), oneEvent), /initial record/);
  assert.throws(() => verifyReplay(output(initial), oneEvent), /0 of 1/);
  assert.throws(() => verifyReplay(output(initial, accepted), twoEvents), /1 of 2/);
  assert.throws(() => verifyReplay(output(initial, accepted, accepted), oneEvent), /2 of 1/);
  assert.throws(() => verifyReplay({ ...output(initial), stdout: '{' }, oneEvent), SyntaxError);
  assert.throws(() => verifyReplay(output(initial, { record: 'unexpected' }), oneEvent), /unrecognized/);
  assert.throws(() => verifyReplay(output(initial, null), oneEvent), /unrecognized/);
  assert.throws(() => verifyReplay(output(initial), '\n \n'), /event record is empty/);
});

test('a refused or fatal event fails even if the subprocess exits successfully', () => {
  assert.throws(() => verifyReplay(output(initial, { ...accepted, outcome: 'rejected', origin: 'policy', code: 'reject', message: 'no' }), oneEvent), /was rejected/);
  assert.throws(() => verifyReplay(output(initial, { ...accepted, record: 'fatal' }), oneEvent), /was fatal/);
  assert.throws(() => verifyReplay({ ...output(initial, accepted), status: 1 }, oneEvent), /exited with status 1/);
});

test('every input event must have an accepted replay record', () => {
  assert.deepEqual(verifyReplay(output(initial, accepted, second), twoEvents), [accepted, second]);
});

test('same-count duplicates, reordered events and changed identities cannot pass', () => {
  assert.throws(() => verifyReplay(output(initial, accepted, accepted), twoEvents), /does not match recorded line 2/);
  assert.throws(() => verifyReplay(output(initial, second, accepted), twoEvents), /does not match recorded line 1/);
  for (const change of [{ line: 2 }, { event: 'main_run' }, { payload: { green: 0 } }, { payload: undefined }]) {
    assert.throws(() => verifyReplay(output(initial, { ...accepted, ...change }), oneEvent), /does not match recorded line 1/);
  }
});

test('accepted events must carry the next sequence and a matching ledger snapshot', () => {
  for (const change of [{ sequence: 2 }, { snapshot: undefined }, { snapshot: [] }, { snapshot: snapshot(0) }]) {
    assert.throws(() => verifyReplay(output(initial, { ...accepted, ...change }), oneEvent), /invalid sequence or snapshot/);
  }
  for (const bindings of [undefined, [], {}, { ledger: null }, { ledger: [] }, { ledger: {} }]) {
    assert.throws(() => verifyReplay(output(initial, { ...accepted, snapshot: { sequence: 1, bindings } }), oneEvent), /no ledger bindings/);
  }
  assert.throws(() => verifyReplay(output({ ...initial, sequence: 1 }, accepted), oneEvent), /invalid sequence or snapshot/);
  assert.throws(() => verifyReplay(output({ record: 'initial', sequence: 0 }, accepted), oneEvent), /invalid sequence or snapshot/);
  assert.throws(() => verifyReplay(output(initial, accepted, { ...second, sequence: 1, snapshot: snapshot(1) }), twoEvents), /invalid sequence or snapshot/);
});

test('expected events use the CLI parser, physical lines and default payload', () => {
  const defaultPayload = { ...accepted, line: 2, event: 'reproduce', payload: {} };
  const afterBlank = { ...second, line: 4, payload: { item: 393, nested: { a: 1, b: 2 } } };
  const text = '\r\n{"event":"reproduce"}\r\n \r\n{"event":"approve","payload":{"nested":{"b":2,"a":1},"item":393}}\r\n';
  assert.deepEqual(verifyReplay(output(initial, defaultPayload, afterBlank), text), [defaultPayload, afterBlank]);
  assert.throws(() => verifyReplay(output(initial, { ...defaultPayload, line: 1 }, afterBlank), text), /does not match recorded line 2/);
  assert.throws(() => verifyReplay(output(initial, accepted), '{"event":"ci_run","unknown":1}'), /unknown field/);
  assert.throws(() => verifyReplay(output(initial, accepted), '{'), /not JSON/);
});
