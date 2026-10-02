// Evaluator validation only. These tests are not fresh-agent study runs.
import assert from 'node:assert/strict';
import { mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { loadRuntimeFromDirectory, defaultRuntimeDirectory } from '../../../kit/lib/node.mjs';
import * as scenarioApi from '../../../kit/lib/scenarios.mjs';
import { checkProgram, checkHost, checkScenarios, verifySubmission } from './verify.mjs';
import { attempt } from './evaluator/gold-host.mjs';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const runtime = await loadRuntimeFromDirectory();
const source = await readFile(new URL('./evaluator/gold-tracker.cav', import.meta.url), 'utf8');
const hostSource = await readFile(new URL('./evaluator/gold-host.mjs', import.meta.url), 'utf8');
const suite = JSON.parse(await readFile(new URL('./evaluator/gold.scenarios.json', import.meta.url), 'utf8'));
const starter = await readFile(new URL('../../../kit/examples/agent-evidence/assessment.cav', import.meta.url), 'utf8');
const failed = result => result.cases.filter(item => !item.pass);
const rejectsCase = (result, name) => {
  const declaration = result.cases.find(item => item.name === 'declarations-and-event-contract');
  if (declaration) assert.equal(declaration.pass, true, 'the mutant must load with the required declarations before testing behavioral rejection');
  assert.equal(result.pass, false);
  assert.ok(failed(result).some(item => item.name === name), JSON.stringify(failed(result), null, 2));
};

test('correct private tracker passes every semantic check', async () => {
  const result = await checkProgram(runtime, source);
  assert.equal(result.cases.length, 15);
  assert.equal(result.pass, true, JSON.stringify(failed(result), null, 2));
});

test('valid correction may reopen because the withdrawn reading rather than the correction evidence', async () => {
  const alternative = source.replace('reopen answer because correction;', 'reopen answer because latest(checks);');
  assert.notEqual(alternative, source);
  const session = runtime.open(alternative);
  try {
    session.dispatch('observe', { value: 85 }); session.dispatch('assess');
    const corrected = session.dispatch('correct').snapshot;
    assert.deepEqual(corrected.withdrawals.map(item => [item.evidence, item.because]), [['checks@1', 'correction']]);
    assert.deepEqual(corrected.decision_journal.find(item => item.change === 'reopened').because, ['checks@1']);
  } finally { session.close(); }
  const result = await checkProgram(runtime, alternative);
  assert.equal(result.pass, true, JSON.stringify(failed(result), null, 2));
});

test('mutant that fabricates template support cannot pass neutral observation checks', async () => {
  const mutant = source.replace('reveal tool;', 'reveal tool supports ready;');
  assert.notEqual(mutant, source);
  rejectsCase(await checkProgram(runtime, mutant), 'threshold-70-neutral-grounds');
});

test('mutant that commits during observation fails observation and assessment separation', async () => {
  const mutant = `${source}\non observe commit answer because enough using latest(checks);\n`;
  rejectsCase(await checkProgram(runtime, mutant), 'threshold-70-neutral-grounds');
});

test('mutant with only opposing-reading reopening fails the every-reading contract', async () => {
  const mutant = source.replace('reopened by checks;', 'reopened by checks opposing ready;');
  assert.notEqual(mutant, source);
  rejectsCase(await checkProgram(runtime, mutant), 'every-reading-reopens-85');
  rejectsCase(await checkProgram(runtime, mutant), 'every-reading-reopens-92');
});

test('correction still requires a reopening event and the exact withdrawal reason', async () => {
  const noReopen = source.replace('on correct when committed(answer) and not reopened(answer) and rests_on_withdrawn(answer)\n    reopen answer because correction;\n', '');
  assert.notEqual(noReopen, source);
  rejectsCase(await checkProgram(runtime, noReopen), 'correction-withdrawal-revision-frozen-history');
  const wrongWithdrawalReason = source.replace('withdraw latest(checks) because correction;', 'withdraw latest(checks) because tool;');
  assert.notEqual(wrongWithdrawalReason, source);
  rejectsCase(await checkProgram(runtime, wrongWithdrawalReason), 'correction-withdrawal-revision-frozen-history');
});

test('correct host performs required operations and obtains fresh approval', async () => {
  const result = await checkHost(runtime, starter, attempt);
  assert.equal(result.cases.length, 10);
  assert.equal(result.pass, true, JSON.stringify(failed(result), null, 2));
});

test('correct private registered scenarios run including historical assertions and resume', async () => {
  const result = await checkScenarios(scenarioApi, runtime, suite, source, 'tracker.cav');
  assert.equal(result.pass, true, JSON.stringify(result, null, 2));
  assert.equal(result.count, 7);
  assert.equal(result.meetsStructuralMinimum, true);
  assert.ok(result.report.scenarios.some(item => item.resumes > 0));
});

test('mutant that substitutes a bare value loses exact decision grounds', async () => {
  const mutant = source.replace('commit answer because enough using latest(checks)', 'commit answer because enough using 85');
  assert.notEqual(mutant, source);
  rejectsCase(await checkProgram(runtime, mutant), 'threshold-70-neutral-grounds');
});

test('mutant missing the weak-reading policy refusal is rejected', async () => {
  const mutant = source.replace('on assess when latest(checks) < 70 reject "Weak reading.";\n', '');
  assert.notEqual(mutant, source);
  rejectsCase(await checkProgram(runtime, mutant), 'threshold-69-neutral-grounds');
});

test('mutant that checks template stale instead of selected archived reading is rejected', async () => {
  const mutant = source.replace('has_caveat(selected, stale)', 'carries(tool, stale)');
  assert.notEqual(mutant, source);
  rejectsCase(await checkProgram(runtime, mutant), 'old-clean-reading-remains-eligible-after-stale');
});

test('mutant missing correction withdrawal is rejected', async () => {
  const mutant = source.replace('on correct withdraw latest(checks) because correction;\n', '');
  assert.notEqual(mutant, source);
  rejectsCase(await checkProgram(runtime, mutant), 'correction-withdrawal-revision-frozen-history');
});

test('mutant that reuses visible old approval after rejected operations is rejected', async () => {
  const unsafe = async (session, score) => {
    session.dispatch('observe', { confidence: score });
    session.dispatch('assess');
    return { ok: session.snapshot().bindings.assessment.verdict === 'approved' };
  };
  const result = await checkHost(runtime, starter, unsafe);
  rejectsCase(result, 'primary-2-score-101');
  rejectsCase(result, 'primary-3-score-92');
});

test('host cannot pass by returning the expected boolean sequence without operations', async () => {
  let index = 0;
  const result = await checkHost(runtime, starter, async () => ({ ok: [true, false, false, false, true][index++ % 5] }));
  rejectsCase(result, 'primary-1-score-85');
});

test('scenario evaluator refuses alternate sources and preserves failed assertions', async () => {
  const external = structuredClone(suite);
  external.scenarios[0].source = '../other.cav';
  const blocked = await checkScenarios(scenarioApi, runtime, external, source, 'tracker.cav');
  assert.equal(blocked.pass, false);
  const wrong = structuredClone(suite);
  wrong.scenarios[1].steps.push({ expect: { '/decision_series/answer/current': 'answer@99' } });
  const bad = await checkScenarios(scenarioApi, runtime, wrong, source, 'tracker.cav');
  assert.equal(bad.pass, false);
  assert.equal(bad.report.failed, 1);
});

test('full scorer preserves first failures, records hashes, and leaves manual scores unset', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'caveat-study-evaluator-'));
  try {
    await writeFile(path.join(directory, 'tracker.cav'), source);
    await writeFile(path.join(directory, 'first.cav'), source.replace('commit answer because enough using latest(checks)', 'commit answer because enough using 85'));
    await writeFile(path.join(directory, 'tracker.scenarios.json'), JSON.stringify(suite));
    await writeFile(path.join(directory, 'first.scenarios.json'), JSON.stringify({ ...suite, source: 'first.cav' }));
    await writeFile(path.join(directory, 'host.mjs'), hostSource);
    const result = await verifySubmission({ submissionDirectory: directory, packageDirectory: path.join(root, 'kit'), runtimeDirectory: defaultRuntimeDirectory() });
    assert.equal(result.programs.first.pass, false);
    assert.equal(result.programs.tracker.pass, true);
    assert.equal(result.automated.programSemanticsScore, 1);
    assert.equal(result.automated.finalObjectivesPass, true);
    assert.match(result.artifacts['first.cav'].sha256, /^[a-f0-9]{64}$/);
    assert.notEqual(result.artifacts['first.cav'].sha256, result.artifacts['tracker.cav'].sha256);
    assert.ok(Object.values(result.manual).every(value => value === null));
    assert.equal(result.artifacts['FEATURE_REQUEST.md'].present, false);
    assert.equal(result.candidate.starterUnchanged, true);
  } finally {
    // mkdtemp supplied this exact path in the OS temp directory; no user files.
    assert.equal(path.dirname(directory), path.resolve(os.tmpdir()));
    assert.ok(path.basename(directory).startsWith('caveat-study-evaluator-'));
    await rm(directory, { recursive: true, force: true });
  }
});
