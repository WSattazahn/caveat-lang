import { createHash } from 'node:crypto';
import { tasks } from './private/oracles.mjs';
import { casesFor, oracleStep } from './checks.mjs';

export function describeCorpus() {
  const report = { schema: 1, source: 'Unchanged v1 A/B oracle and registered event sequences', tasks: {} };
  for (const id of ['A', 'B']) {
    const task = tasks[id];
    const cases = casesFor(task, id);
    const summary = { heldOutScenarios: task.cases.length, seededSequences: 100, cases: cases.length,
      casesSha256: createHash('sha256').update(JSON.stringify(cases)).digest('hex'),
      steps: 0, events: 0, accepted: 0, rejected: 0, resumeSteps: 0, maximumElapsed: 0 };
    for (const test of cases) {
      let state = task.initial();
      for (const event of test.events) {
        summary.steps++;
        if (event.resume === true) { summary.resumeSteps++; continue; }
        const result = oracleStep(task, state, event);
        state = result.state;
        summary.events++; summary[result.accepted ? 'accepted' : 'rejected']++;
        summary.maximumElapsed = Math.max(summary.maximumElapsed, state.elapsed);
      }
    }
    report.tasks[id] = summary;
  }
  return report;
}
