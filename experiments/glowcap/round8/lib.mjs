// Phases and scenarios for round 8: the inherited scenarios (../scenarios.mjs
// and ../round7/scenarios-r7.mjs, base to CR16) and the blind phases' own
// (./scenarios-r8.mjs, CR17-CR20). Phase texts base to CR16 are round 7's
// registered files (../round7/phases/, CR10 as amended); CR17-CR20 are
// ./phases/ (PROTOCOL.md, row 3).
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { SCENARIOS as INHERITED, EXPLANATION_KEYS, PHASES as INHERITED_PHASES } from '../scenarios.mjs';
import { SCENARIOS as ROUND7 } from '../round7/scenarios-r7.mjs';
import { SCENARIOS as ROUND8 } from './scenarios-r8.mjs';

export { canonical, runScenario, toEvent } from './exec.mjs';
export { EXPLANATION_KEYS };
export const BLIND = ['cr17', 'cr18', 'cr19', 'cr20'];
export const PHASES = [...INHERITED_PHASES, 'cr13', 'cr14', 'cr15', 'cr16', ...BLIND];

const here = path.dirname(fileURLToPath(import.meta.url));
export const phaseText = (phase) => path.join(here, BLIND.includes(phase) ? 'phases' : '../round7/phases', `${phase}.md`);

export async function allScenarios() {
  return [...INHERITED, ...ROUND7, ...ROUND8];
}

export function applies(scenario, phase) {
  const at = PHASES.indexOf(phase);
  const since = PHASES.indexOf(scenario.since ?? 'base');
  const until = PHASES.indexOf(scenario.until ?? PHASES.at(-1));
  if (at < 0 || since < 0 || until < 0) throw new Error(`unknown phase in ${scenario.id}`);
  return at >= since && at <= until;
}
