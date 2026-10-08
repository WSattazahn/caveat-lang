// Candidate-side M/A regressions from the bounded 590fae5 adversarial pass.
// These are new source-derived probes, not replicas of unavailable Muse findings.
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import test from 'node:test';
import {real} from './helpers.mjs';
import {explain, formatExplanation, dependents} from '../lib/explain.mjs';
import {archiveNodeId} from '../lib/archive.mjs';

const copy = value => structuredClone(value);
const markerSource = await readFile(new URL('fixtures/adversarial/marker-journal.cav', import.meta.url), 'utf8');
const nestedSource = await readFile(new URL('fixtures/adversarial/nested-withdrawal.cav', import.meta.url), 'utf8');
const accepted = (session, event) => assert.equal(session.dispatch(event).outcome, 'accepted', event);
const paths = [['commitment_grounds', 'trust'], ['commitment_bases', 'trust', 'provenance']];
const at = (value, path) => path.reduce((item, key) => item[key], value);
const allMarkers = value => {
  const markers = [];
  function walk(item) {
    if (!item || typeof item !== 'object') return;
    if (item.archive_status) markers.push(item);
    for (const child of Object.values(item)) walk(child);
  }
  walk(value);
  return markers;
};
function originalMarker() {
  const session = real.open(markerSource);
  try {
    for (const event of ['record', 'advance', 'skip', 'decide']) accepted(session, event);
    return {text: session.save(), archive: session.drainArchive()};
  } finally { session.close(); }
}

const markerMutants=[
  ['M01','unknown history',m=>m.history='absent'],
  ['M02','declared non-history',m=>m.history='sensor'],
  ['M03','from below one',m=>m.from=0],
  ['M04','inverted range',m=>m.from=m.through+1],
  ['M05','live occurrence endpoint',m=>m.through=2],
  ['M06','zero read',m=>m.read=0],
  ['M07','read exceeds departed range',m=>m.read=2],
  ['M08','zero departure sequence',m=>m.departed_at=0],
  ['M09','future departure sequence',m=>m.departed_at=999],
  ['M10','fractional range',m=>m.from=1.5],
  ['M11','malformed archive reference',m=>m.archive_ref='not-a-digest'],
];

const invalids = markerMutants.map(([id, title, mutate]) => [id, title, doc => {
  for (const path of paths) mutate(at(doc, path).departed[0]);
}]);
invalids.push(['M12', 'duplicate same-history markers', doc => {
  for (const path of paths) at(doc, path).departed.push(copy(at(doc, path).departed[0]));
}]);
invalids.push(['M13', 'grounds marker outside basis', doc => { delete at(doc, paths[1]).departed; }]);

test('adversarial M00: genuine marker save restores and continues with the same ordered archive', () => {
  const a = real.open(markerSource);
  let b;
  try {
    for (const event of ['record', 'advance', 'skip', 'decide']) accepted(a, event);
    a.drainArchive();
    const text = a.save();
    b = real.restore(markerSource, text);
    assert.equal(b.save(), text);
    assert.deepEqual(a.snapshot(), b.snapshot());
    assert.equal(a.undrained, 0);
    assert.equal(b.undrained, 0);
    accepted(a, 'continue'); accepted(b, 'continue');
    assert.equal(a.save(), b.save());
    assert.deepEqual(a.snapshot(), b.snapshot());
    assert.deepEqual(a.drainArchive(), b.drainArchive());
  } finally { a.close(); b?.close(); }
});
for (const [id, title, mutate] of invalids) {
  test(`adversarial ${id}: ${title} refuses with a restore error`, () => {
    const {text} = originalMarker();
    const doc = JSON.parse(text);
    assert.equal(at(doc, paths[0]).departed.length, 1);
    mutate(doc);
    assert.throws(() => {
      const unexpected = real.restore(markerSource, JSON.stringify(doc));
      unexpected.close();
    }, error => error.kind === 'restore');
  });
}
for (const [id, title, mutate] of [
  ['M14', 'well-formed unresolved reference', marker => { marker.archive_ref = 'f'.repeat(64); }],
  ['M15', 'omitted optional reference', marker => { delete marker.archive_ref; }],
]) {
  test(`adversarial ${id}: ${title} restores with a conservative explanation`, () => {
    const {text, archive} = originalMarker(), doc = JSON.parse(text);
    for (const path of paths) mutate(at(doc, path).departed[0]);
    const session = real.restore(markerSource, JSON.stringify(doc));
    try {
      const report = explain(session.snapshot(), [], {archive});
      const relevant = allMarkers(report).filter(marker => marker.history === 'r'
        && marker.archive_ref === at(doc, paths[0]).departed[0].archive_ref);
      assert.ok(relevant.length);
      assert.ok(relevant.every(marker => marker.archive_status === 'unavailable' && !marker.records));
      assert.match(formatExplanation(report), /exact archive reconstruction unavailable/);
    } finally { session.close(); }
  });
}

const variants=[
  ['A00',a=>a],
  ['A01',a=>a.filter(x=>!(x.kind!=='provenance'&&x.record==='a@2'))],
  ['A02',a=>a.filter(x=>!(x.kind==='provenance'&&x.record==='a@2'))],
  ['A03',a=>[...a,{...copy(a.find(x=>x.kind!=='provenance'&&x.record==='a@2')),retired_at:0}]],
  ['A04',a=>a.map(x=>x.kind==='provenance'&&x.record==='a@2'?{...x,source_id:'different-source'}:x)],
  ['A05',a=>{a.find(x=>x.withdrawal).withdrawal.evidence='r@1';return a;}],
  ['A06',a=>{delete a.find(x=>x.withdrawal).withdrawal.because;return a;}],
  ['A07',a=>{a.find(x=>x.withdrawal).withdrawal.because='absent';return a;}],
  ['A08',a=>{a.find(x=>x.withdrawal).withdrawal.sequence=999;return a;}],
  ['A09',a=>{a.find(x=>x.withdrawal).withdrawal.sequence=0;return a;}],
  ['A10',a=>{a.find(x=>x.withdrawal).withdrawal.sequence=1.5;return a;}],
  ['A11',a=>{for(const x of a)if(x.record==='a@2'){x.departed_at=999;if(x.kind==='provenance')x.id=archiveNodeId(x);}return a;}],
  ['A12',a=>{a.find(x=>x.kind!=='provenance'&&x.record==='a@2').retired_at=999;return a;}],
  ['A13',a=>{a.find(x=>x.kind!=='provenance'&&x.record==='a@2').relations.push(['a@2','invalid','seen']);return a;}],
  ['A14',a=>{a.find(x=>x.kind!=='provenance'&&x.record==='a@2').relations.push(['template','supports','seen']);return a;}],
  ['A15',a=>{a.find(x=>x.record==='r@1'&&x.reading).reading.provenance.evidence=null;return a;}],
  ['A16',a=>{const leaf=copy(a.find(x=>x.kind==='provenance'&&x.record==='a@2'));leaf.source_id='conflict';return [...a,leaf];}],
  ['A17',a=>[...a,...copy(a)].reverse()],
  ['A18',a=>{a.find(x=>x.record==='r@1'&&x.reading).reading.value=999;return a;}],
  ['A19',a=>[{...copy(a.find(x=>x.kind!=='provenance'&&x.record==='a@2')),retired_at:0},...a]],
  ['A20',a=>{const leaf=copy(a.find(x=>x.kind==='provenance'&&x.record==='a@2'));leaf.source_id='conflict';return [leaf,...a];}],
];

const variantTitles = [
  "complete",
  "missing-reason-record",
  "missing-reason-node",
  "conflicting-reason-record",
  "foreign-source-node",
  "wrong-subject",
  "missing-reason-field",
  "absent-reason",
  "future-withdrawal",
  "zero-withdrawal",
  "fractional-withdrawal",
  "future-departure",
  "inverted-retirement",
  "invalid-relation-kind",
  "unrelated-edge",
  "malformed-nested-provenance",
  "conflicting-leaf",
  "identical-duplicates",
  "consistent-payload-edit",
  "reversed-conflicting-record",
  "reversed-conflicting-leaf"
];

for (const drain of ['immediate', 'delayed']) {
  for (const where of ['live', 'restored']) {
    for (const [id, mutate] of variants) {
      test(`adversarial ${id}: ${variantTitles[Number(id.slice(1))]} ${drain}/${where}`, () => {
        const original = real.open(nestedSource), archive = [];
        let restored;
        try {
          accepted(original, 'create');
          if (drain === 'immediate') archive.push(...original.drainArchive());
          accepted(original, 'advance');
          archive.push(...original.drainArchive());
          const text = original.save();
          restored = real.restore(nestedSource, text);
          const snapshot = (where === 'live' ? original : restored).snapshot();
          const supplied = mutate(copy(archive));
          const report = explain(snapshot, [], {archive: supplied});
          assert.equal(report.archive.authenticated, false);
          assert.equal(report.archive.scope, 'provided records and their referenced closure');
          const row = report.archive.records.find(item => item.record === 'r@1');
          assert.ok(row);
          const independent = report.archive.records.find(item => item.record === 'z@1');
          assert.equal(independent?.status, 'complete');
          assert.equal(independent.entry.reading.value, 7);
          if (['A00', 'A17', 'A18'].includes(id)) {
            assert.equal(row.status, 'complete');
            assert.equal(row.entry.reading.value, id === 'A18' ? 999 : 1);
            assert.ok(report.archive.withdrawals.some(item => item.evidence === 'a@2' && item.because === 'a@2'));
            assert.match(formatExplanation(report), /not authenticated or proven exhaustive/);
          } else {
            assert.equal(row.status, 'unavailable');
            assert.equal(row.entry, undefined);
            assert.match(formatExplanation(report), /archive reconstruction unavailable/);
          }
          assert.deepEqual(report.evidence, explain(snapshot).evidence);
          assert.deepEqual(report.decisions, explain(snapshot).decisions);
          assert.deepEqual(dependents(snapshot, 'a@2', {archive: supplied}).withdrawals, []);
          assert.equal(original.save(), text);
          assert.equal(restored.save(), text);
        } finally { original.close(); restored?.close(); }
      });
    }
  }
}
