// The engineering-readiness example: its program checks clean, its scenarios
// pass, and its pilot runs through the starter with a restart, a refusal, a
// storage failure and the archive.
import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { real } from './helpers.mjs';
import { formatFileReport, parseScenarioFile, runScenarioFile } from '../lib/scenarios.mjs';
import { sourceDigest } from '../lib/starter.mjs';
import { runPilot } from '../examples/readiness/run.mjs';
import { spawnSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
import { createServer } from '../lib/serve.mjs';

const folder = fileURLToPath(new URL('../examples/readiness/', import.meta.url));
const source = await readFile(path.join(folder, 'readiness.cav'), 'utf8');

test('the readiness program checks clean and passes its scenarios', async () => {
  assert.deepEqual(real.check(source).diagnostics, []);
  const outcome = await runScenarioFile(parseScenarioFile(await readFile(path.join(folder, 'readiness.scenarios.json'), 'utf8')), {
    runtime: real, file: 'readiness.scenarios.json', readSource: relative => readFile(path.resolve(folder, relative), 'utf8'),
  });
  assert.equal(outcome.failed, 0, formatFileReport(outcome));
  assert.equal(outcome.passed, 4);
});

test('the readiness pilot keeps qualification, reopening and the unrelated decision apart through restarts and the archive', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-readiness-'));
  try {
    const report = await runPilot({ runtime: real, source, directory: path.join(directory, 'store') });
    const steps = Object.fromEntries(report.steps.map(({ step, ...facts }) => [step, facts]));
    assert.deepEqual(steps.approve.release, { revision: 'release@1', status: 'in_force', change: 'committed', because: ['test_result'], caveats: [] });
    assert.deepEqual(steps.reopen.release, { revision: 'release@1', status: 'reopened', change: 'reopened', because: ['test_result'], caveats: ['wrong_configuration'] });
    assert.equal(steps.reopen.permits, false);
    assert.deepEqual(steps.unchanged.dependency_signoff, steps.unrelated.dependency_signoff);
    assert.equal(steps.unchanged.dependency_signoff.status, 'in_force');
    assert.deepEqual([steps.refusal.origin, steps.refusal.code], ['policy', 'reject']);
    assert.deepEqual(steps.storage_failure, { accepted: true, durable: false, flushed: true });
    assert.deepEqual(steps.new_approval.release.because, ['test_result@11']);
    assert.deepEqual(steps.history.release_1.grounds, ['test_result']);
    assert.deepEqual(steps.history.release_1.history.map(change => change.caveats), [[], ['wrong_configuration']]);
    assert.deepEqual(steps.history.departed, ['test_result@2', 'test_result@3']);
    assert.equal(report.identity.source_sha256, sourceDigest(source));
    assert.equal(steps.history.source_sha256, sourceDigest(source));
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test('the readiness pilot refuses a directory that already holds something', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-readiness-'));
  try {
    await runPilot({ runtime: real, source, directory });
    await assert.rejects(runPilot({ runtime: real, source, directory }), /is not empty/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

// Issue #199: a decision's grounds name `test_result@2`, and only the event
// that observed it says which revision that run was for. explain reports
// that event and its payload beside each ground.
const READINESS_EVENTS = [
  ['propose', { revision: 'rev-1234' }],
  ['test_reported', { revision: 'rev-1234', passed: 1 }],
  ['approve_release', {}],
  ['test_configuration_wrong', {}],
  ['propose', { revision: 'rev-5678' }],
  ['test_reported', { revision: 'rev-5678', passed: 1 }],
  ['approve_release', {}],
];

test('issue_199_explain_names_the_event_and_payload_behind_each_ground', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-issue-199-'));
  try {
    const events = path.join(directory, 'events.jsonl');
    await writeFile(events, READINESS_EVENTS.map(([event, payload]) => JSON.stringify({ event, payload })).join('\n'));
    const cli = fileURLToPath(new URL('../bin/caveat.mjs', import.meta.url));
    const program = path.join(folder, 'readiness.cav');
    const text = spawnSync(process.execPath, [cli, 'explain', program, events], { encoding: 'utf8' });
    assert.equal(text.status, 0, text.stderr);
    assert.match(text.stdout, /based on test_result\n {6}test_result: observed at #2 test_reported \{"revision":"rev-1234","passed":1\}\n/);
    assert.match(text.stdout, /based on test_result@2\n {6}test_result@2: observed at #6 test_reported \{"revision":"rev-5678","passed":1\}\n/);
    const json = spawnSync(process.execPath, [cli, 'explain', '--json', program, events], { encoding: 'utf8' });
    assert.equal(json.status, 0, json.stderr);
    const report = JSON.parse(json.stdout);
    const release = report.decisions.find(series => series.name === 'release');
    assert.deepEqual(release.revisions.map(revision => revision.observed_by), [
      [{ evidence: 'test_result', sequence: 2, event: 'test_reported', payload: { revision: 'rev-1234', passed: 1 } }],
      [{ evidence: 'test_result@2', sequence: 6, event: 'test_reported', payload: { revision: 'rev-5678', passed: 1 } }],
    ]);
    // The field is additive: the events are listed as before, without the
    // effects explain read them from.
    assert.deepEqual(Object.keys(report.events[1]).sort(), ['event', 'outcome', 'payload']);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test('issue_199_a_ground_observed_before_the_events_listed_says_so', () => {
  const server = createServer({ runtime: real, source, program: 'readiness.cav' });
  const send = request => server.handle(JSON.stringify(request)).response;
  for (const [event, payload] of READINESS_EVENTS.slice(0, 2)) send({ op: 'dispatch', event, payload });
  // A restored session lists only the events sent since: the first test run
  // was observed before them.
  send({ op: 'restore', save: send({ op: 'save' }).save });
  for (const [event, payload] of READINESS_EVENTS.slice(2)) send({ op: 'dispatch', event, payload });
  const release = send({ op: 'explain' }).report.decisions.find(series => series.name === 'release');
  assert.deepEqual(release.revisions.map(revision => revision.observed_by), [
    [{ evidence: 'test_result', sequence: null, event: null, payload: null }],
    [{ evidence: 'test_result@2', sequence: 6, event: 'test_reported', payload: { revision: 'rev-5678', passed: 1 } }],
  ]);
  server.close();
});
