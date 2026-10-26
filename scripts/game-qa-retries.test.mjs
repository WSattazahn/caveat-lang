import test from 'node:test';
import assert from 'node:assert/strict';
import { retriedCommand, tallyRetries, overThreshold, issueTitle, report } from './game-qa-retries.mjs';

const warn = message => ({ annotation_level: 'warning', message });

test('reads the command from each retry warning and nothing else', () => {
  assert.equal(retriedCommand("'npm run test:beacon' failed (exit 1); trying once more"), 'npm run test:beacon');
  assert.equal(retriedCommand('playwright install webkit failed or stalled; trying once more'), 'playwright install webkit');
  assert.equal(retriedCommand('No files were found with the provided path: test-results/. No artifacts will be uploaded.'), null);
  assert.equal(retriedCommand(undefined), null);
});

test('counts retries per job and command, ignoring other annotations', () => {
  const beacon = { name: 'The Last Beacon browser QA', html_url: 'https://example.test/1', annotations: [
    warn("'npm run test:beacon' failed (exit 1); trying once more"),
    warn('No files were found with the provided path: test-results/.'),
    { annotation_level: 'failure', message: "'npm run test:beacon' failed (exit 1); trying once more" },
  ] };
  const tally = tallyRetries([beacon, beacon, beacon, { name: 'Light the Way (webkit)', annotations: [
    warn("'npm run test:rescue' failed (exit 124); trying once more")] }]);
  assert.deepEqual(tally.map(({ job, command, count }) => ({ job, command, count })), [
    { job: 'The Last Beacon browser QA', command: 'npm run test:beacon', count: 3 },
    { job: 'Light the Way (webkit)', command: 'npm run test:rescue', count: 1 },
  ]);
  assert.deepEqual(overThreshold(tally).map(entry => entry.count), [3]);
  assert.deepEqual(overThreshold(tally.map(entry => ({ ...entry, count: 2 }))), []);
});

test('opens an issue once, then comments on the open one', async () => {
  const entry = { job: 'Light the Way (webkit)', command: 'npm run test:rescue', count: 4, examples: [] };
  const calls = [];
  let issues = [];
  const api = async (path, options = {}) => {
    calls.push([options.method || 'GET', path]);
    if (path.startsWith('/repos/o/r/issues?')) return issues;
    if (path === '/repos/o/r/issues') return { number: 7 };
    return {};
  };
  const log = () => {};
  assert.deepEqual((await report({ api, repository: 'o/r', tally: [entry], log })).map(a => a.action), ['opened']);
  issues = [{ number: 7, title: issueTitle(entry) }, { number: 8, title: issueTitle(entry), pull_request: {} }];
  assert.deepEqual(await report({ api, repository: 'o/r', tally: [entry], log }),
    [{ action: 'commented', number: 7, title: issueTitle(entry) }]);
  assert.deepEqual(calls.filter(([method]) => method === 'POST').map(([, path]) => path),
    ['/repos/o/r/issues', '/repos/o/r/issues/7/comments']);
  assert.deepEqual(await report({ api, repository: 'o/r', tally: [{ ...entry, count: 2 }], log }), []);
});
