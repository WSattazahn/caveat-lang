// Programs whose saves a reviewed fix changed on purpose. Each must still
// give the same refusals and fatal outcomes as rc.15; only its save and
// restore digest may differ. D014 (experiments/agent-decisions/README.md):
// a late `qualify` reaches a guard's stored record, so values read through it
// carry the caveat; it adds caveats and changes no outcome. Windowed programs
// it changes are absent: their sweep compares outcomes only, which match.
export const reviewedSaveChanges = new Set([
  'experiments/agent-adoption/v1/evaluator/gold-tracker.cav',
  'experiments/agent-authoring/rc7/trial01/final/task.cav',
  'experiments/agent-authoring/rc7/trial01/first/task.cav',
  'experiments/agent-decisions/001-departure-reason-mechanism/decision.cav',
  'experiments/agent-decisions/retest-rc19/d001-undeclared.cav',
  'kit/examples/agent-evidence/grounded_assessment.cav',
  'kit/examples/agent-evidence/qualification.cav',
  'kit/examples/reviewer-decision/rely.cav',
  'kit/test/fixtures/saves/programs/frost.cav',
  'runtime/tests/fixtures/f268-rc11-wide-grounds.cav',
]);

// A reviewed change keeps everything but the digest.
const withoutDigest = row => row.replace(/^(\S+) [0-9a-f]{16} /, '$1 ');
export const comparableSave = rows =>
  rows.map(row => reviewedSaveChanges.has(row.split(' ')[0]) ? withoutDigest(row) : row);
