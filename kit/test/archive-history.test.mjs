import test from 'node:test';
import assert from 'node:assert/strict';
import {archiveNodeId} from '../lib/archive.mjs';
import {explain, dependents, formatExplanation, formatDependents} from '../lib/explain.mjs';

const source_id = 'archive-history-fixture';
const p = (evidence = [], caveats = []) => ({evidence, caveats});
function fixture() {
  const one = {record: 'h@1', history: 'h', number: 1, retired_at: 8, departed_at: 10, holders: [],
    withdrawal: {evidence: 'h@1', because: 'h@2', sequence: 7, event: 'withdraw_pair'},
    relations: [['h@1','supports','claim'], ['h@1','relies_on','h@2'], ['q','qualifies','h@1']],
    qualifications: {observation: p(['h@2'], ['q'])}};
  const two = {record: 'h@2', history: 'h', number: 2, retired_at: 9, departed_at: 10, holders: [],
    withdrawal: {evidence: 'h@2', because: 'h@1', sequence: 7, event: 'withdraw_pair'},
    relations: [['h@2','opposes','claim']]};
  const leaves = [one, two].map(entry => {
    const node = {kind: 'provenance', operation: 'record', source_id, record: entry.record, history: entry.history, departed_at: entry.departed_at};
    node.id = archiveNodeId(node); return node;
  });
  const marker = index => ({history: 'h', read: 1, from: index + 1, through: index + 1, departed_at: 10, archive_ref: leaves[index].id});
  one.reading = {id: 'h@1', ordinal: 1, sequence: 4, event: 'sample', value: 42, relation: 'supports', claim: 'claim',
    provenance: {...p(), departed: [marker(1)]}};
  const snapshot = {schema: 'caveat-reactive/0.1', source_id, sequence: 12, elapsed: 0, windows: ['h'],
    symbols: [{name: 'claim', kind: 'claim'}, {name: 'q', kind: 'caveat'}, {name: 'reason', kind: 'evidence'}, {name: 'template', kind: 'evidence'}],
    reading_streams: {h: {template: 'template', occurrences: [{id: 'h@3', ordinal: 3}]}}, renewals: {}, retired: {},
    commitments: [{action: 'd', open: true, retained: []}], commitment_grounds: {d: p(['h@3'])},
    commitment_bases: {d: {value: 9, provenance: p(['h@3'])}}, decision_series: {}, decision_journal: [],
    observations: ['h@3'], withdrawals: [{evidence: 'h@3', because: 'reason', sequence: 11, event: 'current'}],
    relations: [{from: 'h@3', relation: 'supports', to: 'claim'}],
    qualified_values: {copied: {value: 42, provenance: {...p(), departed: [marker(0)]}}}, value_grounds: {copied: p()},
    bindings: {hud: {value: 42}}, binding_explanations: {}, binding_qualifications: {hud: {value: {...p(), departed: [marker(0)]}}}};
  return {snapshot, one, two, leaves, marker, archive: [one, two, ...leaves]};
}

test('archive history joins mutual withdrawals, once-stored relations and nested payload markers without changing live reports', () => {
  const {snapshot, archive} = fixture();
  const current = explain(snapshot), report = explain(snapshot, [], {archive});
  assert.deepEqual(report.evidence, current.evidence);
  assert.deepEqual(report.decisions, current.decisions);
  assert.equal(report.decisions[0].revisions[0].status, 'reopened');
  assert.equal(report.archive.authenticated, false);
  assert.equal(report.archive.scope, 'provided records and their referenced closure');
  assert.deepEqual(report.archive.records.map(row => [row.record, row.status]), [['h@1','complete'], ['h@2','complete']]);
  assert.ok(report.archive.records[1].dependencies.includes('h@1'), 'once-stored relation joins both endpoints');
  assert.deepEqual(report.archive.withdrawals.map(item => [item.evidence, item.because]), [['h@1','h@2'], ['h@2','h@1']]);
  assert.equal(report.archive.relations.filter(item => item.relation === 'relies_on').length, 1);
  assert.equal(report.archive.records[0].entry.reading.value, 42);
  assert.match(formatExplanation(report), /Archived history .*not authenticated or proven exhaustive/);
  assert.match(formatExplanation(report), /archived value 42/);
  assert.match(formatExplanation(report), /h@1: retired at #8, departed at #10/);
  const query = dependents(snapshot, 'h@1', {archive});
  assert.deepEqual(query.withdrawals, []);
  assert.deepEqual(query.reasonForWithdrawals, []);
  assert.deepEqual(query.archive.withdrawals.map(item => item.evidence), ['h@1']);
  assert.deepEqual(query.archive.reasonForWithdrawals.map(item => item.evidence), ['h@2']);
  assert.match(formatDependents(query), /withdrawn at #7 because h@2/);
  for (const name of ['h', 'template']) assert.equal(dependents(snapshot, name, {archive}).archive.withdrawals.length, 2);
  const caveat = dependents(snapshot, 'q', {archive}).archive;
  assert.deepEqual(caveat.withdrawals.map(item => item.evidence), ['h@1']);
  assert.deepEqual(caveat.reasonForWithdrawals.map(item => item.evidence), ['h@2']);
  assert.deepEqual(caveat.relations, [{from: 'q', relation: 'qualifies', to: 'h@1'}]);
});

test('missing leaf, missing recursive payload, conflicting duplicates and invalid sources remain unavailable', () => {
  const {snapshot, archive, one, two, leaves} = fixture();
  const wrong = {...leaves[1], source_id: 'different-program'}; wrong.id = archiveNodeId(wrong);
  for (const supplied of [archive.filter(item => item !== two), archive.filter(item => item !== leaves[1]),
    [...archive, {...two, retired_at: 6}], [one, two, leaves[0], wrong],
    archive.map(item => item === two ? {...two, withdrawal: {...two.withdrawal, because: 'absent'}} : item)]) {
    const report = explain(snapshot, [], {archive: supplied}).archive;
    assert.equal(report.records.find(row => row.record === 'h@1').status, 'unavailable');
    assert.equal(report.records.find(row => row.record === 'h@1').entry, undefined);
    assert.deepEqual(report.withdrawals, []);
    assert.deepEqual(report.relations, []);
    assert.match(formatExplanation(explain(snapshot, [], {archive: supplied})), /archive reconstruction unavailable/);
  }
  const missing = dependents(snapshot, 'h@1').archive;
  assert.ok(missing.unresolved.includes('h@1'));
  assert.deepEqual(missing.withdrawals, []);
});

test('duplicates, cycles and later copied/replaced marker references remain deterministic after JSON transfer', () => {
  const {snapshot, archive, marker} = fixture();
  const duplicate = JSON.parse(JSON.stringify([...archive, ...archive].reverse()));
  assert.deepEqual(explain(snapshot, [], {archive: duplicate}).archive, explain(snapshot, [], {archive}).archive);
  snapshot.qualified_values.copied.provenance.departed = [marker(1)];
  const moved = JSON.parse(JSON.stringify(snapshot));
  assert.deepEqual(dependents(moved, 'h@1', {archive: duplicate}).values, []);
  assert.deepEqual(dependents(moved, 'h@2', {archive: duplicate}).values[0].via, ['h@2']);
  assert.equal(explain(moved, [], {archive: duplicate}).archive.records.every(row => row.status === 'complete'), true);
});

test('consistent payload edits and omitted unrelated records cannot prove authenticity or global completeness', () => {
  const {snapshot, archive, one, two, leaves} = fixture();
  const supplied = archive.map(item => item === one ? {...one, reading: {...one.reading, value: 999}} : item);
  const history = explain(snapshot, [], {archive: supplied}).archive;
  assert.equal(history.records[0].status, 'complete', 'membership hashes deliberately do not authenticate payload bytes');
  assert.equal(history.records[0].entry.reading.value, 999);
  assert.equal(history.authenticated, false);
  assert.equal(history.scope, 'provided records and their referenced closure');
  const independent = {...one, withdrawal: {...one.withdrawal, because: 'h@1'}, relations: [], qualifications: {},
    reading: {...one.reading, provenance: p()}};
  const incoming = {...two, withdrawal: {...two.withdrawal, because: 'h@2'}, relations: [['h@2','relies_on','h@1']]};
  assert.ok(explain(snapshot, [], {archive: [independent, incoming, ...leaves]}).archive.records[0].dependencies.includes('h@2'));
  const partial = explain(snapshot, [], {archive: [independent, leaves[0]]}).archive;
  assert.equal(partial.records[0].status, 'complete', 'an omitted incoming edge cannot be inferred from a retained payload');
  assert.deepEqual(partial.relations, []);
  assert.equal(partial.scope, 'provided records and their referenced closure', 'not a negative claim about all historical edges');
});

test('malformed archive rows fail conservatively rather than throwing during sorting or formatting', () => {
  const {snapshot} = fixture();
  const archive = [{record: 'bad-one', holders: []}, {record: 'bad-two', holders: []},
    {record: 'bad-three', history: {toString: null}, number: 1, departed_at: 2, holders: []}];
  const report = explain(snapshot, [], {archive});
  assert.equal(report.archive.records.length, 3);
  assert.ok(report.archive.records.every(row => row.status === 'unavailable'));
  assert.match(formatExplanation(report), /bad-one: archive reconstruction unavailable/);
  const {one, two, leaves, marker} = fixture();
  one.reading.provenance.departed = [{...marker(1), history: {toString: null}}, {history: {toString: null}, from: {toString: null}}];
  const malformedMarkers = explain(snapshot, [], {archive: [one, two, ...leaves]}).archive;
  assert.ok(malformedMarkers.records.every(row => row.status === 'unavailable'));
});

test('future occurrences cannot bypass departure validation by naming their history journal', () => {
  for (const history of ['h', 'journal']) {
    const {snapshot} = fixture();
    snapshot.windows = [history];
    snapshot.reading_streams = {[history]: {occurrences: [{id: `${history}@3`, ordinal: 3}]}};
    const entry = {record: `${history}@99`, history, number: 99, retired_at: 8, departed_at: 10, holders: []};
    const node = {kind: 'provenance', operation: 'record', source_id, history, record: entry.record, departed_at: 10};
    node.id = archiveNodeId(node);
    assert.equal(explain(snapshot, [], {archive: [entry, node]}).archive.records[0].status, 'unavailable');
  }
});


function chronologyFixture() {
  const value = fixture();
  value.one.relations = []; value.one.qualifications = {};
  value.one.reading.provenance = p();
  value.two.relations = []; delete value.two.withdrawal;
  value.two.reading = {...value.one.reading, id: 'h@2', ordinal: 2, sequence: 7};
  const row = (archive = value.archive, name = 'h@1') => explain(value.snapshot, [], {archive}).archive.records.find(item => item.record === name);
  return {...value, row};
}

test('withdrawals require already-created reading subjects and archived or held reasons when creation is known', () => {
  for (const location of ['subject', 'archived reason', 'held reason']) {
    for (const sequence of [7, 8]) {
      const {snapshot, one, two, leaves, archive, row} = chronologyFixture();
      let supplied = archive;
      if (location === 'subject') one.reading.sequence = sequence;
      else two.reading.sequence = sequence;
      if (location === 'held reason') {
        snapshot.reading_streams.h.occurrences.unshift(two.reading);
        supplied = [one, leaves[0]];
      }
      const found = row(supplied);
      assert.equal(found.status, sequence === 7 ? 'complete' : 'unavailable', location);
      if (sequence === 8) assert.equal(found.entry, undefined);
    }
  }
});

test('frozen reading provenance uses creation time while late metadata and markers keep their own meanings', () => {
  for (const membership of ['direct', 'inherited', 'marker']) {
    for (const sequence of [4, 5]) {
      const {one, two, marker, row} = chronologyFixture();
      delete one.withdrawal;
      two.reading.sequence = sequence;
      one.reading.provenance = membership === 'marker' ? {...p(), departed: [marker(1)]}
        : {...p(['h@2']), ...(membership === 'inherited' ? {inherited: ['h@2']} : {})};
      assert.equal(row().status, sequence === 4 ? 'complete' : 'unavailable', membership);
    }
  }
  const {snapshot, one, two, leaves, row} = chronologyFixture();
  delete one.withdrawal;
  one.qualifications.observation = p(['h@2']);
  assert.equal(row().status, 'complete', 'late metadata may reference evidence created after the frozen reading');
  snapshot.reading_streams.h.occurrences.unshift({...two.reading, sequence: 11});
  assert.equal(row([one, leaves[0]]).status, 'unavailable', 'even untimed metadata cannot reference creation after departure');
});

test('journal references use their change sequence and agree with known decision revisions', () => {
  for (const field of ['because', 'permitted_by', 'revision']) {
    for (const sequence of [7, 8]) {
      const {snapshot, two, leaves, row} = chronologyFixture();
      snapshot.windows.push('journal');
      snapshot.decision_series.choice = {revisions: [{id: 'choice@1', sequence: field === 'revision' ? sequence : 7, event: 'decide'}]};
      snapshot.commitments.push({action: 'choice@1', open: true, retained: []});
      two.reading.sequence = field === 'revision' ? 7 : sequence;
      const journal = {record: 'journal@1', history: 'journal', number: 1, retired_at: 8, departed_at: 10, holders: [],
        journal_entry: {decision: 'choice', commitment: 'choice@1', change: 'reopened', sequence: 7, event: 'decide',
          because: field === 'because' ? ['h@2'] : [], caveats: [], ...(field === 'permitted_by' ? {permitted_by: 'h@2'} : {})}};
      const leaf = {kind: 'provenance', operation: 'record', source_id, history: 'journal', record: 'journal@1', departed_at: 10};
      leaf.id = archiveNodeId(leaf);
      const supplied = [two, leaves[1], journal, leaf];
      assert.equal(row(supplied, journal.record).status, sequence === 7 ? 'complete' : 'unavailable', field);
      if (field === 'revision' && sequence === 7) {
        journal.journal_entry.change = 'committed';
        assert.equal(row(supplied, journal.record).status, 'complete');
        journal.journal_entry.sequence = 8;
        assert.equal(row(supplied, journal.record).status, 'unavailable', 'the same committed revision has one frozen creation sequence');
        journal.journal_entry.sequence = 7; journal.journal_entry.event = 'other_event';
        assert.equal(row(supplied, journal.record).status, 'unavailable', 'a committed revision must name its recorded event');
      }
    }
  }
});
