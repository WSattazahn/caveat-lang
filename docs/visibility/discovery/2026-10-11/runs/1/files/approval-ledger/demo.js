import { Ledger } from './ledger.js';
const l = new Ledger();
l.approve({ id: 'deploy-42', action: 'deploy api@1.8.0 to prod', approver: 'agent-7',
  reason: 'Integration suite passed; no other blockers.',
  evidence: [{ id: 'ev-itest', source: 'testtool run #991', claim: '212/212 passed', config: { env: 'staging-old' } }] });
l.retract({ evidenceId: 'ev-itest', source: 'testtool maintainers', reason: 'Run used staging-old config, not prod-like',
  correctedClaim: 'result is not valid for prod' });
console.log(JSON.stringify(l.report('deploy-42'), null, 2));
