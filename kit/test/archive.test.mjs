import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {archiveSha256, archiveNodeId, archiveResolver} from '../lib/archive.mjs';
import {explain, dependents, formatExplanation, formatDependents} from '../lib/explain.mjs';

const source_id = 'synthetic-archive-program';
function leaf(number, source = source_id) {
  const entry = {record: `h@${number}`, history: 'h', number, retired_at: number + 5, departed_at: number + 10, holders: []};
  const node = {kind: 'provenance', history: 'h', operation: 'record', source_id: source, record: entry.record, departed_at: entry.departed_at};
  node.id = archiveNodeId(node);
  return {entry, node};
}
function union(...nodes) {
  const node = {kind: 'provenance', history: 'h', operation: 'union', parents: [...new Set(nodes.map(node => node.id))].sort()};
  node.id = archiveNodeId(node);
  return node;
}
const one = leaf(1), two = leaf(2), three = leaf(3), four = leaf(4), five = leaf(5);
const left = union(one.node, three.node), right = union(three.node, five.node), overlap = union(left, right), replacement = union(two.node, four.node);
const archive = [one,two,three,four,five].flatMap(value => [value.entry, value.node]).concat(left, right, overlap, replacement);
function provenance(root = overlap, numbers = [1,3,5], read = 2) {
  return {evidence: [], caveats: [], departed: [{history: 'h', read, from: Math.min(...numbers), through: Math.max(...numbers), departed_at: Math.max(...numbers) + 10, archive_ref: root.id}]};
}
function snapshot(p = provenance()) {
  return {schema: 'caveat-reactive/0.1', source_id, sequence: 20, elapsed: 0, windows: ['h'],
    reading_streams: {h: {occurrences: [{id: 'h@6'}]}}, renewals: {}, retired: {},
    commitments: [{action: 'd', open: false, retained: []}], commitment_grounds: {d: p},
    commitment_bases: {d: {value: 7, provenance: p}}, decision_series: {}, decision_journal: [],
    qualified_values: {copied: {value: 7, provenance: p}}, value_grounds: {copied: p},
    bindings: {hud: {score: 7}}, binding_explanations: {hud: {score: p}}, binding_qualifications: {}, relations: [],
  };
}

test('browser SHA-256 agrees with standard vectors, UTF-8 and block boundaries', () => {
  for (const text of ['', 'abc', 'The quick brown fox jumps over the lazy dog', '温度🌡️', 'x'.repeat(55), 'x'.repeat(56), 'x'.repeat(64), 'x'.repeat(10000)]) {
    assert.equal(archiveSha256(text), createHash('sha256').update(text).digest('hex'));
  }
});

test('copy and overlapping unions reconstruct sparse exact members in decisions and displayed bindings', () => {
  const current = snapshot();
  const report = explain(current, [], {archive});
  assert.deepEqual(report.decisions[0].revisions[0].grounds.departed[0].records, ['h@1','h@3','h@5']);
  assert.deepEqual(report.displayed[0].cites.departed[0].records, ['h@1','h@3','h@5']);
  assert.equal(report.displayed[0].cites.departed[0].archive_status, 'complete');
  assert.match(formatExplanation(report), /departed h@1, h@3, h@5/);
  const found = dependents(current, 'h@3', {archive});
  assert.equal(found.decisions[0].basis, 'grounds');
  assert.equal(found.values[0].basis, 'grounds');
  assert.equal(found.displayed[0].basis, 'cites');
  assert.deepEqual(dependents(current, 'h@2', {archive}).values, []);
});

test('replacement roots discard prior membership and survive snapshot JSON transfer', () => {
  const current = JSON.parse(JSON.stringify(snapshot(provenance(replacement, [2,4]))));
  const transferredArchive = JSON.parse(JSON.stringify(archive));
  assert.deepEqual(explain(current, [], {archive: transferredArchive}).displayed[0].cites.departed[0].records, ['h@2','h@4']);
  assert.deepEqual(dependents(current, 'h@3', {archive: transferredArchive}).values, []);
  assert.equal(dependents(current, 'h@2', {archive: transferredArchive}).values[0].basis, 'grounds');
});

test('missing, conflicting, corrupt, cyclic and wrong-source archive graphs remain conservative', () => {
  const changed = structuredClone(left); changed.parents = [one.node.id, four.node.id].sort();
  const cycle = {...left, parents: [left.id, right.id].sort()};
  const wrong = leaf(1, 'other-source');
  const wrongUnion = union(wrong.node, three.node, five.node);
  const unknownParent = {...overlap, parents: [left.id, '0'.repeat(64)].sort()}; unknownParent.id = archiveNodeId(unknownParent);
  const cases = [
    [], archive.filter(entry => entry !== three.node), archive.filter(entry => entry !== three.entry),
    [...archive, changed], archive.map(entry => entry === left ? cycle : entry),
    [...archive, {...one.entry, departed_at: 999}], archive.map(entry => entry === left ? {...entry, history: 'other'} : entry),
  ];
  for (const incomplete of cases) {
    const report = explain(snapshot(), [], {archive: incomplete});
    assert.equal(report.displayed[0].cites.departed[0].records, undefined);
    assert.equal(report.displayed[0].cites.departed[0].archive_status, 'unavailable');
    assert.match(formatExplanation(report), /marker summary; exact archive reconstruction unavailable/);
    const result = dependents(snapshot(), 'h@2', {archive: incomplete});
    assert.equal(result.values[0].basis, 'may rest on');
    assert.match(formatDependents(result), /may rest on h@2/);
  }
  const wrongArchive = [...archive, wrong.entry, wrong.node, wrongUnion];
  assert.equal(archiveResolver(snapshot(), wrongArchive)(provenance(wrongUnion).departed[0]), null);
  assert.equal(archiveResolver(snapshot(), [...archive, unknownParent])(provenance(unknownParent).departed[0]), null);
});

test('identical duplicate archive chunks are idempotent and order does not select a conflicting winner', () => {
  const expected = ['h@1','h@3','h@5'];
  assert.deepEqual(archiveResolver(snapshot(), [...archive, ...structuredClone(archive)].reverse())(provenance().departed[0]), expected);
  const conflict = {...one.entry, holders: [{kind: 'state', name: 'other', in: 'lineage'}]};
  for (const entries of [[conflict, ...archive], [...archive, conflict]]) assert.equal(archiveResolver(snapshot(), entries)(provenance().departed[0]), null);
});

test('marker summary mismatch, future leaves, held records and unproved exact-looking annotations are refused', () => {
  const resolve = archiveResolver(snapshot(), archive);
  for (const patch of [{from: 2}, {through: 4}, {read: 4}, {departed_at: 14}, {read: 1.5}, {archive_ref: undefined}]) {
    assert.equal(resolve({...provenance().departed[0], ...patch}), null);
  }
  assert.equal(archiveResolver({...snapshot(), sequence: 14}, archive)(provenance().departed[0]), null);
  assert.equal(archiveResolver({...snapshot(), retired: {'h@3': 9}}, archive)(provenance().departed[0]), null);
  const spoofed = provenance(); spoofed.departed[0].records = ['h@2']; spoofed.departed[0].archive_status = 'complete';
  assert.equal(explain(snapshot(spoofed)).displayed[0].cites.departed[0].records, undefined);
  assert.equal(explain(snapshot(spoofed)).displayed[0].cites.departed[0].archive_status, 'unavailable');
  const singleton = provenance(one.node, [1], 1);
  assert.match(formatExplanation(explain(snapshot(singleton))), /departed h@1 .*exact archive reconstruction unavailable/);
});

test('deep archive DAGs are traversed iteratively and large ranges do not expand into absent records', () => {
  const entries = [];
  let root;
  for (let number = 1; number <= 12000; number++) {
    const item = leaf(number); entries.push(item.entry, item.node);
    if (!root) root = item.node;
    else { root = union(root, item.node); entries.push(root); }
  }
  const current = {...snapshot(), sequence: 13000, reading_streams: {h: {occurrences: [{id: 'h@12001'}]}}};
  const marker = {history: 'h', from: 1, through: 12000, read: 1, departed_at: 12010, archive_ref: root.id};
  assert.equal(archiveResolver(current, entries)(marker).length, 12000);
  const huge = provenance(); huge.departed[0] = {history: 'h', from: 1, through: 1000000000000, read: 2, departed_at: 10};
  assert.equal(explain(snapshot(huge)).displayed[0].cites.departed[0].departed_between, 999999999999);
});
