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
