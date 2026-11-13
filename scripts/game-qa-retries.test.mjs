import test from 'node:test';
import assert from 'node:assert/strict';
import { retriedCommand, tallyRetries, overThreshold, issueTitle, issueBody, report, collect, expectedCount } from './game-qa-retries.mjs';

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

test('a run that carried round 1 over counts as if it had run, so the guard needs no more flakiness to open an issue', () => {
  const retry = warn("'npm run test:rescue' failed (exit 1); trying once more");
  const run = (annotations = [], extra = {}) => ({ name: 'Light the Way (chromium)', html_url: 'https://example.test/r', annotations, ...extra });
  const shape = tally => tally.map(({ count, runs, carried }) => ({ count, runs, carried }));
  // Two retries in four runs that ran: not over the threshold.
  const ran = tallyRetries([run([retry]), run([retry]), run(), run()]);
  assert.deepEqual(shape(ran), [{ count: 2, runs: 4, carried: 0 }]);
  assert.deepEqual(overThreshold(ran), []);
  // The same two retries when two more runs carried (and one was skipped):
  // judged as 3 retries in 6 runs, so the issue opens.
  const carried = tallyRetries([run([retry]), run([retry]), run(), run(), run([], { carried: true }), run([], { carried: true }),
    run([], { conclusion: 'skipped' })]);
  assert.deepEqual(shape(carried), [{ count: 2, runs: 6, carried: 2 }]);
  assert.equal(expectedCount(carried[0]), 3);
  assert.deepEqual(overThreshold(carried).map(entry => entry.count), [2]);
  assert.match(issueBody(carried[0]), /2 of the job's 6 runs carried round 1's result over without running \(scripts\/ci-reuse\), so the 2 retries count as 3, as if every run had run\./);
  assert.doesNotMatch(issueBody(ran[0]), /carried/);
  // Just past the threshold, the count never prints at it: 2 retries with 1
  // of 50 runs carried is 2.04, which opens the issue and prints as 2.1.
  const barely = { job: 'j', command: 'c', count: 2, runs: 50, carried: 1, examples: [] };
  assert.equal(overThreshold([barely]).length, 1);
  assert.match(issueBody(barely), /so the 2 retries count as 2\.1, as if every run had run\./);
  // A carried run's annotations are never read as retries.
  assert.deepEqual(tallyRetries([run([retry], { carried: true })]), []);
});

test('collect marks a check run carried only when its own carry step says so', async () => {
  const retry = warn("'npm run test:rescue' failed (exit 1); trying once more");
  const carriedNotice = { annotation_level: 'notice', title: 'Carried from round 1', message: 'https://example.test/runs/1 ran this job to success on tree t.' };
  const checkRun = (id, name, conclusion = 'success', annotations_count = 1) => ({ id, name, conclusion, html_url: `https://example.test/${id}`, output: { annotations_count } });
  const annotations = {
    11: [{ annotation_level: 'notice', title: 'Round-1 carry', message: 'round 1 ran these jobs' }],
    12: [carriedNotice],
    13: [retry, carriedNotice], // not a job that can carry, whatever it prints
    14: [retry], // the same job, run in full by a later attempt of the same run
    15: [{ ...carriedNotice, title: 'Round-1 carry' }],
    22: [retry],
  };
  const api = async url => {
    if (url.startsWith('/repos/o/r/actions/workflows/runtime.yml/runs?created=%3E%3D2026-11-06&')) return { workflow_runs: [{ check_suite_id: 1 }, { check_suite_id: 2 }] };
    if (url.startsWith('/repos/o/r/actions/workflows/pages.yml/runs?')) return { workflow_runs: [] };
    if (url.startsWith('/repos/o/r/check-suites/1/check-runs?filter=all&')) {
      return { check_runs: [checkRun(11, 'round-1 carry'), checkRun(12, 'Light the Way (chromium)'), checkRun(13, 'Evidence games browser QA'),
        checkRun(14, 'Light the Way (chromium)'), checkRun(15, 'native runtime')] };
    }
    if (url.startsWith('/repos/o/r/check-suites/2/check-runs?filter=all&')) {
      return { check_runs: [checkRun(21, 'round-1 carry', 'skipped', 0), checkRun(22, 'Light the Way (chromium)')] };
    }
    const id = /^\/repos\/o\/r\/check-runs\/(\d+)\/annotations\?/.exec(url)?.[1];
    if (id && annotations[id]) return annotations[id];
    throw new Error(`unexpected ${url}`);
  };
  const runs = await collect({ api, repository: 'o/r', since: new Date('2026-11-06T12:00:00Z') });
  assert.deepEqual(runs.map(({ name, conclusion, carried }) => ({ name, conclusion, carried })), [
    { name: 'round-1 carry', conclusion: 'success', carried: false },
    { name: 'Light the Way (chromium)', conclusion: 'success', carried: true },
    { name: 'Evidence games browser QA', conclusion: 'success', carried: false },
    { name: 'Light the Way (chromium)', conclusion: 'success', carried: false },
    { name: 'native runtime', conclusion: 'success', carried: false },
    { name: 'round-1 carry', conclusion: 'skipped', carried: false },
    { name: 'Light the Way (chromium)', conclusion: 'success', carried: false },
  ]);
  const tally = tallyRetries(runs);
  assert.deepEqual(tally.find(entry => entry.job === 'Light the Way (chromium)'), {
    job: 'Light the Way (chromium)', command: 'npm run test:rescue', count: 2, examples: ['https://example.test/14', 'https://example.test/22'], runs: 3, carried: 1,
  });
  assert.equal(tally.find(entry => entry.job === 'Evidence games browser QA').count, 1);
});
