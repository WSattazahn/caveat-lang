// Decisions that rest on several models' votes (experiments/model-votes):
// the scenarios pass, and the undo list separates the decision a distrusted
// model's vote tipped from the one that holds without it.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { real, sourceReader } from './helpers.mjs';
import { undoList } from '../lib/explain.mjs';
import { formatFileReport, parseScenarioFile, runScenarioFile } from '../lib/scenarios.mjs';

const folder = new URL('../../experiments/model-votes/', import.meta.url);
const source = await readFile(new URL('votes.cav', folder), 'utf8');

test('the model-votes scenarios pass', async () => {
  const doc = parseScenarioFile(await readFile(new URL('votes.scenarios.json', folder), 'utf8'));
  const result = await runScenarioFile(doc, { runtime: real, file: 'votes.scenarios.json', readSource: sourceReader({ 'votes.cav': source }) });
  assert.equal(result.failed, 0, formatFileReport(result));
  assert.equal(result.passed, doc.scenarios.length);
});

test('the undo list reopens the decision the distrusted vote tipped and keeps the other', async () => {
  const session = real.open(source);
  const events = (await readFile(new URL('demo.events.jsonl', folder), 'utf8')).trim().split('\n').map(line => JSON.parse(line));
  for (const { event, payload } of events) assert.equal(session.dispatch(event, payload).outcome, 'accepted');
  const report = undoList(session.snapshot());
  const status = Object.fromEntries(report.decisions.map(item => [item.series, item.status]));
  assert.deepEqual(status, { case_a_approve: 'reopened', case_b_approve: 'in force' });
  assert.deepEqual(report.withdrawals.map(item => item.evidence), ['m2_out', 'case_a_m2_vote@1', 'case_b_m2_vote@1']);
});
