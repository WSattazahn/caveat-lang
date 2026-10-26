// Runs web/break.html's checker (web/break-check.js) on the page's two
// examples through the lean reactive runtime, and shows that each of the five
// rules fails on a snapshot doctored to break it. No browser.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import init, { WebReactiveSession } from '../dist/pkg-reactive/caveat_runtime.js';
import { PROBE, RULES, checkEvent, checkStart, probeSource, reportBody } from '../web/break-check.js';

const root = new URL('../', import.meta.url);
await init({ module_or_path: await readFile(new URL('dist/pkg-reactive/caveat_runtime_bg.wasm', root)) });

// The page's run: the program with the checker's display lines, each event
// checked against the snapshot before it.
function run(source, events) {
  const plain = JSON.parse(new WebReactiveSession(source).snapshot());
  const session = new WebReactiveSession(probeSource(source, plain));
  let before = JSON.parse(session.snapshot());
  const steps = [{ event: 'start', ...checkStart(before), snapshot: before }];
  for (const event of events) {
    const outcome = JSON.parse(session.dispatch_outcome(event, '{}'));
    const after = JSON.parse(session.snapshot());
    steps.push({ event, outcome: outcome.outcome, ...checkEvent(before, after, outcome.outcome === 'accepted'), before, snapshot: after });
    before = after;
  }
  return steps;
}

const clean = steps => {
  for (const step of steps) assert.deepEqual(step.findings, [], `${step.event}: ${JSON.stringify(step.findings)}`);
};
const status = (snapshot, revision) => (snapshot.commitments.find(item => item.action === revision)?.open ? 'reopened' : 'in force');

// The apartment: the landlord's reply withdraws what the decision rested on,
// and it reopens; the fake gym review withdraws what it never used, and
// nothing reopens.
const apartment = await readFile(new URL('web/break-apartment.cav', root), 'utf8');
const flat = run(apartment, ['gather', 'decide', 'review_flagged', 'landlord_answers', 'decide', 'traffic_rechecked']);
clean(flat);
assert.equal(status(flat[3].snapshot, 'apartment@1'), 'in force', 'the fake review reopened the decision');
assert.equal(flat[3].snapshot.bindings[PROBE].apartment, 0);
assert.equal(status(flat[4].snapshot, 'apartment@1'), 'reopened');
assert.deepEqual(flat[4].snapshot.commitment_grounds['apartment@1'], flat[2].snapshot.commitment_grounds['apartment@1'],
  'the reopened decision keeps its grounds');
assert.equal(flat[5].snapshot.decision_series.apartment.current, 'apartment@2');

// The guard example: withdrawing the evidence only the guard read leaves the
// decision in force, while what was computed under it carries `withdrawn`
// (D026 for the display read through committed(merge)).
const guard = await readFile(new URL('web/break-guard.cav', root), 'utf8');
const gated = run(guard, ['observe', 'decide', 'freeze_still_on', 'tests_misread']);
clean(gated);
const afterFreeze = gated[3].snapshot;
assert.equal(afterFreeze.bindings[PROBE].merge, 0, 'a guard-only withdrawal made the merge rest on withdrawn evidence');
assert.equal(status(afterFreeze, 'merge@1'), 'in force');
assert.ok(afterFreeze.binding_qualifications.merge.status.caveats.includes('withdrawn'), 'D026: the status display lost withdrawn');
assert.ok(afterFreeze.qualified_values.merge_copy.provenance.caveats.includes('withdrawn'));
assert.ok(!afterFreeze.value_grounds.merge_copy.caveats.includes('withdrawn'));
assert.equal(gated[4].snapshot.bindings[PROBE].merge, 1);
assert.equal(status(gated[4].snapshot, 'merge@1'), 'reopened');

// Mutation controls: each rule fails on a snapshot broken its way.
const copy = value => structuredClone(value);
const failed = (before, after, accepted = true) => new Set(checkEvent(before, after, accepted).findings.map(item => item.rule));
const landlord = flat[4];
{
  const after = copy(landlord.snapshot);
  after.commitment_grounds['apartment@1'].evidence = ['maple_rent'];
  assert.ok(failed(landlord.before, after).has('R1'), 'R1 missed a rewritten basis');
}
{
  const after = copy(landlord.snapshot);
  after.binding_qualifications.choice.text.caveats = after.binding_qualifications.choice.text.caveats.filter(c => c !== 'withdrawn');
  assert.ok(failed(landlord.before, after).has('R2'), 'R2 missed a value that lost withdrawn');
}
{
  const after = copy(gated[3].snapshot);
  after.bindings[PROBE].merge = 1;
  assert.ok(failed(gated[3].before, after).has('R3'), 'R3 missed rests_on_withdrawn true on guard-only evidence');
}
{
  const after = copy(flat[3].snapshot);
  after.commitments.find(item => item.action === 'apartment@1').open = true;
  assert.ok(failed(flat[3].before, after).has('R4'), 'R4 missed a reopening nobody wrote');
}
{
  const after = copy(flat[2].snapshot);
  after.withdrawals = [{ evidence: 'maple_pets', because: 'landlord_reply', sequence: 2, event: 'decide' }];
  assert.ok(failed(flat[2].snapshot, after, false).has('R5'), 'R5 missed a withdrawal left by a refused event');
}

// The report fills the issue template's sections and ticks what failed.
const template = await readFile(new URL('.github/ISSUE_TEMPLATE/break-report.md', root), 'utf8');
const body = reportBody({ version: 'test', source: guard, events: '{"event":"observe"}', findings: [{ rule: 'R3', kind: 'too much', at: '#3 x', detail: 'd' }] });
for (const heading of template.match(/^## .+$/gm)) assert.ok(body.includes(heading), `the report lacks ${heading}`);
assert.ok(body.includes('- [x] R3'));
assert.ok(body.includes('- [x] Reopens too much'));
assert.equal(RULES.length, 5);
console.log('break.html checker: both examples hold every rule; each rule fails on its mutation');
