// Regenerates the fault fixtures from the two examples. Every fixture except
// pass-lineage-as-set and evaluation-bound (and the hand-written neg-repeat)
// must fail or be refused. The examples and these fixtures share a depth, so
// their repository-relative source paths carry over unchanged.
import { readFile, writeFile } from 'node:fs/promises';

const here = new URL('./', import.meta.url);
const t = JSON.parse(await readFile(new URL('../examples/thermostat_history.scenarios.json', here), 'utf8'));
const r = JSON.parse(await readFile(new URL('../examples/trail_rescue.scenarios.json', here), 'utf8'));
const clone = v => JSON.parse(JSON.stringify(v));
const cases = {};
{ const d = clone(t); d.scenarios[0].steps[4].expect['/bindings/heating/text'] = '50%'; cases['wrong-value'] = d; }
{ const d = clone(r); d.scenarios[0].steps[7].rejected = true; cases['bare-rejected-on-input'] = d; }
{ const d = clone(r); d.scenarios[0].steps[4].rejected = 'Some other message.'; cases['wrong-policy-message'] = d; }
{ const d = clone(r); delete d.scenarios[0].steps[4].rejected; cases['expected-accept-got-reject'] = d; }
{ const d = clone(t); const j = d.scenarios[0].steps[8].expect['/decision_journal']; [j[0], j[1]] = [j[1], j[0]]; cases['journal-order'] = d; }
{ const d = clone(t); d.scenarios[0].steps[4].expect['/commitment_bases/heating@2/provenance/evidence'] = ['temperature@2', 'temperature@1']; cases['lineage-order-matters'] = d; }
{ const d = clone(t); d.scenarios[0].steps[4].expect['/commitment_bases/heating@2/provenance/evidence'] = { $set: ['temperature@2', 'temperature@1'] }; cases['pass-lineage-as-set'] = d; }
{ const d = clone(t); d.scenarios[0].steps[5] = { same_as: 'before', paths: ['/commitment_grounds'] }; cases['same-as-before-changed'] = d; }
cases['invalid-unknown-field'] = { ...clone(t), extra: 1 };
{ const d = clone(t); d.scenarios[1].steps[0].rejected = { origin: 'host' }; cases['invalid-host-origin'] = d; }
{ const d = clone(t); d.scenarios[1].steps[0].rejected = { origin: 'input', message: 'value must be finite and in 0..40' }; cases['invalid-input-message'] = d; }
{ const d = clone(t); d.scenarios[1].steps[3].expect['/sequence'] = { $absent: false }; cases['invalid-absent-false'] = d; }
{ const d = clone(t); d.scenarios[1].steps[3].expect['/bindings'] = { heating: 1, $set: [] }; cases['invalid-mixed-matcher'] = d; }
cases['evaluation-bound'] = { schema: 'caveat-scenarios/0.1', source: 'faults.cav', scenarios: [{ id: 'F01', title: 'state bound', steps: [
  { send: 'raise', payload: { amount: 8 } }, { send: 'raise', payload: { amount: 8 }, rejected: { origin: 'evaluation', code: 'bound_exceeded' } }, { expect: { '/bindings/hud/level': 8 } }] }] };
cases['fatal-not-a-rejection'] = { schema: 'caveat-scenarios/0.1', source: 'faults.cav', scenarios: [{ id: 'F02', title: 'unclassified', steps: [
  { send: 'peek', rejected: { origin: 'evaluation' } }] }] };
cases['size-bound'] = { ...clone(t), scenarios: [{ ...clone(t.scenarios[0]), steps: [...clone(t.scenarios[0].steps.slice(0, 4)), { size: { save: { max: 200 } } }] }] };
for (const [name, doc] of Object.entries(cases)) {
  await writeFile(new URL(`neg-${name}.scenarios.json`, here), `${JSON.stringify(doc, null, 2)}\n`);
}
console.log(Object.keys(cases).join(' '));
