// Phases and scenarios for round 7: the inherited scenarios (../scenarios.mjs)
// and, once registered, CR13-CR16's (./scenarios-r7.mjs).
import { SCENARIOS as INHERITED, EXPLANATION_KEYS, PHASES as INHERITED_PHASES } from '../scenarios.mjs';

export { canonical, runScenario, toEvent } from './exec.mjs';
export { EXPLANATION_KEYS };
export const PHASES = [...INHERITED_PHASES, 'cr13', 'cr14', 'cr15', 'cr16'];

export async function allScenarios() {
  let added = [];
  try {
    added = (await import('./scenarios-r7.mjs')).SCENARIOS;
  } catch (error) {
    if (error.code !== 'ERR_MODULE_NOT_FOUND') throw error;
  }
  return [...INHERITED, ...added];
}

export function applies(scenario, phase) {
  const at = PHASES.indexOf(phase);
  const since = PHASES.indexOf(scenario.since ?? 'base');
  const until = PHASES.indexOf(scenario.until ?? PHASES.at(-1));
  if (at < 0 || since < 0 || until < 0) throw new Error(`unknown phase in ${scenario.id}`);
  return at >= since && at <= until;
}
