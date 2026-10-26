import test from 'node:test';
import assert from 'node:assert/strict';
import { BUDGETS, runTrialWithinBudget } from './run.mjs';

const trials = slowest => {
  const calls = [];
  const trial = async ({ directory }) => { calls.push(directory); return { maxSendMs: slowest[calls.length - 1] }; };
  return { calls, trial };
};

test('one stalled trial is run again in a fresh store', async () => {
  const { calls, trial } = trials([BUDGETS.sendMs + 166, 20]);
  const report = await runTrialWithinBudget({ packageDir: 'pkg', directory: '/store', trial });
  assert.equal(report.attempts, 2);
  assert.deepEqual(report.slowestByAttempt, [BUDGETS.sendMs + 166, 20]);
  assert.deepEqual(calls, ['/store/attempt-1', '/store/attempt-2']);
});

test('a slowdown in every trial fails', async () => {
  const { calls, trial } = trials([400, 380, 410]);
  await assert.rejects(runTrialWithinBudget({ packageDir: 'pkg', directory: '/store', trial }),
    /slowest send over 250 ms in all 3 trials: 400 ms, 380 ms, 410 ms/);
  assert.equal(calls.length, 3);
});

test('a trial within budget runs once', async () => {
  const { calls, trial } = trials([BUDGETS.sendMs]);
  assert.equal((await runTrialWithinBudget({ packageDir: 'pkg', directory: '/store', trial })).attempts, 1);
  assert.equal(calls.length, 1);
});
