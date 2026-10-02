// CLI demonstration using the shipped starter and the public session/explain
// APIs. Inputs are illustrative, fixed values; no model or tool is scored.
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { explain } from './explain.mjs';
import { same } from './scenarios.mjs';

export const DEMO_SCHEMA = 'caveat-demo/0.1';

function requireInvariant(condition, message) {
  if (!condition) throw new Error(`Agent demo invariant failed: ${message}`);
}

// A successful demonstration must establish its advertised result. Accepted
// dispatch alone does not prove that the earlier decision or archive survived.
function verifyDemo(steps) {
  const [observed, decided, corrected, replacement, revised] = steps.map(step => step.snapshot);
  const firstGrounds = { evidence: ['observations@1'], caveats: [] };
  const lastGrounds = { evidence: ['observations@2'], caveats: [] };
  requireInvariant(observed.decision_series.assessment.current === null, 'observing must not commit a decision');
  requireInvariant(decided.decision_series.assessment.current === 'assessment@1', 'the first assessment must be assessment@1');
  requireInvariant(same(decided.commitment_grounds['assessment@1'], firstGrounds), 'first decision grounds must be exactly observations@1');
  requireInvariant(same(decided.commitment_bases['assessment@1'], { value: 85, provenance: firstGrounds }), 'first decision basis must retain supplied value 85 and its provenance');
  const originalReading = observed.reading_streams.observations.occurrences[0];
  requireInvariant(originalReading?.id === 'observations@1' && originalReading.value === 85, 'first reading must archive supplied value 85');
  const withdrawal = [{ evidence: 'observations@1', because: 'recheck', sequence: 3, event: 'retract' }];
  for (const snapshot of [corrected, replacement, revised]) {
    requireInvariant(same(snapshot.withdrawals, withdrawal), 'the original reading must remain withdrawn with its correction reason');
    requireInvariant(same(snapshot.reading_streams.observations.occurrences[0], originalReading), 'the withdrawn reading archive must remain unchanged');
    requireInvariant(same(snapshot.commitment_grounds['assessment@1'], firstGrounds), 'original decision grounds must remain unchanged');
    requireInvariant(same(snapshot.commitment_bases['assessment@1'], decided.commitment_bases['assessment@1']), 'original decision basis must remain unchanged');
  }
  for (const step of [steps[2], steps[3]]) {
    const series = step.explanation.decisions.find(item => item.name === 'assessment');
    requireInvariant(series.current === 'assessment@1' && series.revisions[0].status === 'reopened', 'correction must reopen assessment@1 until the next assessment');
    requireInvariant(step.snapshot.decision_journal.some(entry => entry.change === 'reopened' && entry.commitment === 'assessment@1' && entry.event === 'retract' && same(entry.because, ['recheck'])), 'reopening must record the correction reason');
  }
  requireInvariant(revised.decision_series.assessment.current === 'assessment@2', 'the new assessment must commit assessment@2');
  requireInvariant(same(revised.commitment_grounds['assessment@2'], lastGrounds), 'revised decision grounds must be exactly observations@2');
  requireInvariant(same(revised.commitment_bases['assessment@2'], {
    value: 92, provenance: { evidence: ['observations@1', 'observations@2', 'recheck'], caveats: [] },
  }), 'revised basis must retain supplied value 92 and its full lineage');
  requireInvariant(same(revised.reading_streams.observations.occurrences.map(item => [item.id, item.value]), [['observations@1', 85], ['observations@2', 92]]), 'both supplied readings must remain in the archive');
  const revisions = steps.at(-1).explanation.decisions.find(item => item.name === 'assessment').revisions;
  requireInvariant(same(revisions.map(item => [item.id, item.status]), [['assessment@1', 'superseded'], ['assessment@2', 'in force']]), 'the first revision must be superseded by the new in-force decision');
}

/** Runs a fresh, disposable session; never writes source or session files. */
export async function runAgentDemo(runtime) {
  const file = 'examples/agent-evidence/assessment.cav';
  const source = await readFile(new URL(`../${file}`, import.meta.url), 'utf8');
  const session = runtime.open(source);
  const events = [];
  const steps = [];
  try {
    for (const [event, payload] of [
      ['observe', { confidence: 85 }], ['assess', {}], ['retract', {}],
      ['observe', { confidence: 92 }], ['assess', {}],
    ]) {
      const result = session.dispatch(event, payload);
      if (result.outcome !== 'accepted') {
        throw new Error(`Agent demo ${event} was refused (${result.origin}/${result.code}): ${result.message}`);
      }
      const { snapshot, ...outcome } = result;
      events.push({ event, payload, outcome });
      steps.push({ event, payload, snapshot, explanation: explain(snapshot, [...events]) });
    }
    verifyDemo(steps);
    const initial = steps[1].explanation.decisions.find(item => item.name === 'assessment');
    const commitment = initial.current;
    const originalGrounds = steps[1].snapshot.commitment_grounds[commitment];
    const finalGrounds = steps.at(-1).snapshot.commitment_grounds[commitment];
    return {
      schema: DEMO_SCHEMA,
      demo: 'agent',
      inputs: 'illustrative',
      runtime: { ...runtime.identity },
      source: { file, sha256: createHash('sha256').update(source).digest('hex') },
      steps,
      preservation: { commitment, originalGrounds, finalGrounds, unchanged: same(originalGrounds, finalGrounds) },
    };
  } finally {
    session.close();
  }
}

const names = items => items.length ? items.join(', ') : 'none';

/** Text is derived from the same snapshots/explanations returned as JSON. */
export function formatAgentDemo(report) {
  const lines = [
    'CAVEAT Language: agent demo',
    'Illustrative inputs supplied by this demo; the scores are not measured confidence.',
    `Program: ${report.source.file}`,
  ];
  for (const [index, step] of report.steps.entries()) {
    const payload = Object.keys(step.payload).length ? ` ${JSON.stringify(step.payload)}` : '';
    lines.push('', `${index + 1}. ${step.event}${payload}`);
    if (step.event === 'observe') {
      const reading = step.explanation.evidence.filter(item => item.value !== null).at(-1);
      lines.push(`   ${reading.id} = ${reading.value}; ${reading.relation} ${reading.claim}`);
    }
    if (step.event === 'retract') {
      const withdrawn = step.snapshot.withdrawals.at(-1);
      lines.push(`   ${withdrawn.evidence} withdrawn because ${withdrawn.because}; its archive remains.`);
    }
    const decision = step.explanation.decisions.find(item => item.name === 'assessment');
    const current = decision.revisions.find(item => item.id === decision.current);
    if (current) {
      lines.push(`   ${current.id} = ${current.value}; ${current.status}`);
      lines.push(`   Grounds: ${names(current.grounds.evidence)}; caveats: ${names(current.grounds.caveats)}`);
    } else lines.push('   No decision yet: observing is separate from assessing.');
  }
  const { preservation } = report;
  const original = report.steps.at(-1).explanation.decisions
    .find(item => item.name === 'assessment').revisions.find(item => item.id === preservation.commitment);
  lines.push('', `${original.id} is ${original.status}; original grounds ${preservation.unchanged ? 'preserved' : 'CHANGED'}: ${names(preservation.finalGrounds.evidence)}.`);
  lines.push('Caveat records supplied evidence and authored policy; it does not establish that either is true.');
  return lines.join('\n');
}
