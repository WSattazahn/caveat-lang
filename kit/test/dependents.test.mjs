// dependents: the reverse of explain. For a piece of evidence, a reading stream
// or a caveat, it finds every decision, decision change, value and displayed
// value whose grounds or lineage include it, and says which.
import assert from 'node:assert/strict';
import test from 'node:test';
import { dependents, formatDependents, DEPENDENTS_SCHEMA } from '../lib/explain.mjs';
import { real, thermostat } from './helpers.mjs';

function thermostatAfter(values) {
  const session = real.open(thermostat);
  for (const value of values) session.dispatch('read', { value });
  const snapshot = session.snapshot();
  session.close();
  return snapshot;
}

// What the snapshot itself says rests on `ids`, computed independently.
function expected(snapshot, ids, field = 'evidence') {
  const touches = provenance => (provenance?.[field] ?? []).some(name => ids.includes(name));
  const decisions = {};
  for (const [id, grounds] of Object.entries(snapshot.commitment_grounds)) {
    if (touches(grounds)) decisions[id] = 'grounds';
    else if (touches(snapshot.commitment_bases[id].provenance)) decisions[id] = 'lineage';
  }
  const values = {};
  for (const [name, value] of Object.entries(snapshot.qualified_values)) {
    if (touches(snapshot.value_grounds[name])) values[name] = 'grounds';
    else if (touches(value.provenance)) values[name] = 'lineage';
  }
  return { decisions, values };
}

const byKey = (items, key) => Object.fromEntries(items.map(item => [item[key], item.basis]));

test('evidence: grounds are told apart from what could only have influenced', () => {
  const snapshot = thermostatAfter([17, 25, 17]);
  const report = dependents(snapshot, 'temperature@1');
  assert.equal(report.schema, DEPENDENTS_SCHEMA);
  assert.equal(report.kind, 'evidence');
  assert.deepEqual(byKey(report.decisions, 'id'), { 'heating@1': 'grounds', 'heating@2': 'lineage', 'heating@3': 'lineage' });
  assert.deepEqual(byKey(report.decisions, 'id'), expected(snapshot, ['temperature@1']).decisions);
  assert.deepEqual(byKey(report.values, 'name'), expected(snapshot, ['temperature@1']).values);
  assert.deepEqual(report.decisions.map(item => item.status), ['superseded', 'superseded', 'in force']);
  // Only the first decision was made because of it.
  assert.deepEqual(report.changes.map(item => [item.commitment, item.change]), [['heating@1', 'committed']]);
  for (const item of report.displayed) {
    const cites = snapshot.binding_explanations[item.name.split('.')[0]][item.name.split('.')[1]];
    assert.equal(item.basis, cites.evidence.includes('temperature@1') ? 'cites' : 'lineage', item.name);
  }
});

test('a reading stream, or the evidence it reads from, stands for all its readings', () => {
  const snapshot = thermostatAfter([17, 25]);
  const all = ['temperature@1', 'temperature@2'];
  const stream = dependents(snapshot, 'temperature');
  assert.deepEqual(byKey(stream.decisions, 'id'), expected(snapshot, all).decisions);
  assert.deepEqual(stream.decisions.find(item => item.id === 'heating@2').via, ['temperature@2']);
  const template = snapshot.reading_streams.temperature.template;
  assert.deepEqual(dependents(snapshot, template).decisions, stream.decisions);
});

test('a caveat reaches everything based on evidence it qualifies', () => {
  const snapshot = thermostatAfter([17, 25]);
  const report = dependents(snapshot, 'calibration_offset');
  assert.equal(report.kind, 'caveat');
  assert.deepEqual(byKey(report.decisions, 'id'), expected(snapshot, ['calibration_offset'], 'caveats').decisions);
  assert.ok(report.decisions.length > 0);
  assert.match(formatDependents(report, 'thermostat', 2), /heating@1 = 1 {2}superseded {2}based on evidence with calibration_offset/);
});

test('a name the program does not declare is an error, and declared but unused is empty', () => {
  const snapshot = thermostatAfter([]);
  assert.throws(() => dependents(snapshot, 'nowhere'), /nowhere is not evidence, a reading stream or a caveat/);
  assert.throws(() => dependents(snapshot, 'heating'), /not evidence/);
  const report = dependents(snapshot, 'temperature');
  assert.deepEqual([report.decisions, report.changes, report.values, report.displayed].map(items => items.length), [0, 0, 0, 0]);
  assert.match(formatDependents(report, 'thermostat', 0),
    /^What rests on temperature in thermostat after 0 events \(sequence 0\)\n\nDecisions\n {2}nothing\n\nDecision changes\n {2}nothing\n/);
});
