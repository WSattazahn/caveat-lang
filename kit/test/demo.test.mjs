import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { DEMO_SCHEMA, formatAgentDemo, runAgentDemo } from '../lib/demo.mjs';
import { explain } from '../lib/explain.mjs';
import { faulty, real } from './helpers.mjs';

test('the agent demo records supplied readings, reopens after correction, and preserves original grounds', async () => {
  const report = await runAgentDemo(real);
  assert.equal(report.schema, DEMO_SCHEMA);
  assert.equal(report.inputs, 'illustrative');
  assert.deepEqual(report.steps.map(step => step.event), ['observe', 'assess', 'retract', 'observe', 'assess']);
  assert.deepEqual(report.steps.map(step => step.snapshot.sequence), [1, 2, 3, 4, 5]);
  const [observed, decided, corrected, replacement, revised] = report.steps;
  assert.equal(observed.snapshot.decision_series.assessment.current, null);
  assert.deepEqual(decided.snapshot.commitment_grounds['assessment@1'], { evidence: ['observations@1'], caveats: [] });
  assert.equal(decided.snapshot.commitment_bases['assessment@1'].value, 85);
  assert.equal(corrected.explanation.decisions[0].revisions[0].status, 'reopened');
  assert.deepEqual(corrected.snapshot.withdrawals.map(item => [item.evidence, item.because]), [['observations@1', 'recheck']]);
  assert.equal(replacement.snapshot.decision_series.assessment.current, 'assessment@1');
  assert.deepEqual(revised.snapshot.reading_streams.observations.occurrences.map(item => item.id), ['observations@1', 'observations@2']);
  assert.equal(revised.snapshot.decision_series.assessment.current, 'assessment@2');
  assert.deepEqual(revised.snapshot.commitment_grounds['assessment@2'], { evidence: ['observations@2'], caveats: [] });
  assert.equal(revised.snapshot.commitment_bases['assessment@2'].value, 92);
  assert.deepEqual(revised.explanation.decisions[0].revisions.map(item => item.status), ['superseded', 'in force']);
  assert.deepEqual(revised.snapshot.commitment_grounds['assessment@1'], decided.snapshot.commitment_grounds['assessment@1']);
  assert.deepEqual(revised.snapshot.commitment_bases['assessment@1'], decided.snapshot.commitment_bases['assessment@1']);
  assert.equal(report.preservation.unchanged, true);
  assert.equal(report.preservation.commitment, 'assessment@1');
  for (const step of report.steps) {
    assert.deepEqual(step.explanation, explain(step.snapshot, step.explanation.events));
  }
  const source = await readFile(new URL('../examples/agent-evidence/assessment.cav', import.meta.url), 'utf8');
  assert.equal(report.source.sha256, createHash('sha256').update(source).digest('hex'));
  assert.deepEqual(report, await runAgentDemo(real), 'same runtime and supplied inputs produce the same complete report');
});

test('agent demo text uses the runtime outcomes and names the synthetic inputs', async () => {
  const report = await runAgentDemo(real);
  const text = formatAgentDemo(report);
  assert.match(text, /Illustrative inputs supplied by this demo/);
  assert.match(text, /observations@1 = 85; supports answer_supported/);
  assert.match(text, /assessment@1 = 85; in force/);
  assert.match(text, /observations@1 withdrawn because recheck; its archive remains/);
  assert.match(text, /assessment@1 = 85; reopened/);
  assert.match(text, /assessment@2 = 92; in force/);
  assert.match(text, /assessment@1 is superseded; original grounds preserved: observations@1/);
  const altered = structuredClone(report);
  altered.steps[0].explanation.evidence[0].value = 83;
  assert.match(formatAgentDemo(altered), /observations@1 = 83;/, 'text reads the explanation instead of a stored transcript');
});

test('the agent demo closes its session on success, a refusal, or a fatal dispatch', async () => {
  const success = faulty();
  await runAgentDemo(success.runtime);
  assert.ok(success.sessions[0].calls.includes('free'));
  const refused = faulty({ dispatch(session, event, payload) {
    return event === 'retract'
      ? JSON.stringify({ schema: 'caveat-dispatch/0.1', outcome: 'rejected', origin: 'policy', code: 'reject', message: 'fixture refusal' })
      : session.inner.dispatch_outcome(event, payload);
  } });
  await assert.rejects(runAgentDemo(refused.runtime), /retract was refused \(policy\/reject\): fixture refusal/);
  assert.ok(refused.sessions[0].calls.includes('free'));
  const failed = faulty({ dispatch() { throw new Error('fixture fatal'); } });
  await assert.rejects(runAgentDemo(failed.runtime), /fixture fatal/);
  assert.ok(!failed.sessions[0].calls.includes('free'), 'fatal sessions are abandoned by the public session lifecycle');
});


test('the agent demo refuses success when a runtime snapshot loses a promised invariant', async t => {
  const cases = [
    ['changed first grounds', 2, snapshot => { snapshot.commitment_grounds['assessment@1'].evidence = ['recheck']; }, /first decision grounds/],
    ['changed original grounds at final decision', 5, snapshot => { snapshot.commitment_grounds['assessment@1'].evidence = ['recheck']; }, /original decision grounds/],
    ['changed original basis', 5, snapshot => { snapshot.commitment_bases['assessment@1'].value = 1; }, /original decision basis/],
    ['erased withdrawal', 5, snapshot => { snapshot.withdrawals = []; }, /remain withdrawn/],
    ['erased archived reading', 5, snapshot => { snapshot.reading_streams.observations.occurrences.shift(); }, /reading archive/],
    ['missing reopening', 3, snapshot => { snapshot.decision_journal = snapshot.decision_journal.filter(entry => entry.change !== 'reopened'); }, /must reopen/],
    ['missing final revision', 5, snapshot => { snapshot.decision_series.assessment.current = 'assessment@1'; }, /commit assessment@2/],
    ['wrong final grounds', 5, snapshot => { snapshot.commitment_grounds['assessment@2'].evidence = ['observations@1']; }, /revised decision grounds/],
    ['wrong final basis', 5, snapshot => { snapshot.commitment_bases['assessment@2'].value = 1; }, /revised basis/],
  ];
  for (const [name, sequence, alter, message] of cases) {
    await t.test(name, async () => {
      const injected = faulty({ dispatch(session, event, payload) {
        const result = JSON.parse(session.inner.dispatch_outcome(event, payload));
        if (result.snapshot?.sequence === sequence) alter(result.snapshot);
        return JSON.stringify(result);
      } });
      await assert.rejects(runAgentDemo(injected.runtime), error => {
        assert.match(error.message, /^Agent demo invariant failed:/);
        assert.match(error.message, message);
        return true;
      });
      assert.ok(injected.sessions[0].calls.includes('free'));
    });
  }
});
