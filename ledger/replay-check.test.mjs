import assert from 'node:assert/strict';
import test from 'node:test';
import { verifyReplay } from './replay-check.mjs';

const initial = { record: 'initial', sequence: 0 };
const accepted = { record: 'event', line: 1, event: 'ci_run', outcome: 'accepted', sequence: 1 };
const output = (...records) => ({ status: 0, stdout: records.map(record => JSON.stringify(record)).join('\n'), stderr: '' });

test('a failed spawn cannot pass even when its status is zero and stdout empty', () => {
  assert.throws(() => verifyReplay({ ...output(), error: new Error('spawnSync EPERM') }, 1), /could not run.*EPERM/);
  assert.throws(() => verifyReplay({ ...output(initial, accepted), signal: 'SIGTERM' }, 1), /stopped on SIGTERM/);
  assert.throws(() => verifyReplay({ ...output(initial, accepted), status: 2 }, 1), /exited with status 2/);
});

test('empty, partial, malformed and unknown replay output cannot pass', () => {
  assert.throws(() => verifyReplay(output(), 1), /initial record/);
  assert.throws(() => verifyReplay(output(initial), 1), /0 of 1/);
  assert.throws(() => verifyReplay(output(initial, accepted), 2), /1 of 2/);
  assert.throws(() => verifyReplay(output(initial, accepted, accepted), 1), /2 of 1/);
  assert.throws(() => verifyReplay({ ...output(initial), stdout: '{' }, 1), SyntaxError);
  assert.throws(() => verifyReplay(output(initial, { record: 'unexpected' }), 1), /unrecognized/);
  assert.throws(() => verifyReplay(output(initial), 0), /event record is empty/);
});

test('a refused or fatal event fails even if the subprocess exits successfully', () => {
  assert.throws(() => verifyReplay(output(initial, { ...accepted, outcome: 'rejected', origin: 'policy', code: 'reject', message: 'no' }), 1), /was rejected/);
  assert.throws(() => verifyReplay(output(initial, { ...accepted, record: 'fatal' }), 1), /was fatal/);
  assert.throws(() => verifyReplay({ ...output(initial, accepted), status: 1 }, 1), /exited with status 1/);
});

test('every input event must have an accepted replay record', () => {
  assert.deepEqual(verifyReplay(output(initial, accepted), 1), [accepted]);
});
