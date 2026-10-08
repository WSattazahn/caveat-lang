// Candidate P probes plus authentic rc.15 save continuation. No runtime repair.
// See fixtures/adversarial/README.md for fixture provenance and the missing Muse cases.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFile} from 'node:fs/promises';
import test from 'node:test';
import {real} from './helpers.mjs';
import {explain} from '../lib/explain.mjs';

const source = await readFile(new URL('fixtures/adversarial/post-collection.cav.txt', import.meta.url), 'utf8');
const oldSave = await readFile(new URL('fixtures/adversarial/post-collection.rc15.save.json', import.meta.url), 'utf8');
const provenance = JSON.parse(await readFile(new URL('fixtures/adversarial/provenance.json', import.meta.url), 'utf8'));
const sha = value => createHash('sha256').update(value).digest('hex');
assert.equal(sha(source), provenance.files['post-collection.cav.txt'].sha256);
assert.equal(sha(oldSave), provenance.files['post-collection.rc15.save.json'].sha256);

const trace = ['start', 'advance', 'skip', 'decide'];
const markerPaths = [['commitment_grounds', 'trust'], ['commitment_bases', 'trust', 'provenance']];
const at = (value, keys) => keys.reduce((item, key) => item[key], value);
const withdrawal = value => value.withdrawals.find(item => item.evidence === 'b@2');
const markerEdit = (value, edit) => { for (const keys of markerPaths) edit(at(value, keys)); };
const withdrawnEdge = edge => edge[0] === 'withdrawn' && edge[1] === 'qualifies' && edge[2] === 'b@2';
const mutants = [
  ['P10', 'duplicate same-history marker after genuine collection', value => markerEdit(value, p => p.departed.push(structuredClone(p.departed[0])))],
  ['P11', 'marker endpoint names still-held r@2', value => markerEdit(value, p => { p.departed[0].through = 2; })],
  ['P12', 'marker archive reference is uppercase hex', value => markerEdit(value, p => { p.departed[0].archive_ref = 'A'.repeat(64); })],
  ['P13', 'remaining withdrawal subject replaced with departed a@2', value => { withdrawal(value).evidence = 'a@2'; }],
  ['P14', 'remaining withdrawal reason replaced with departed a@2', value => { withdrawal(value).because = 'a@2'; }],
  ['P15', 'remove only remaining withdrawal ledger record', value => { value.withdrawals = value.withdrawals.filter(item => item.evidence !== 'b@2'); }],
  ['P16', 'remove only remaining withdrawal graph edge', value => { value.graph.relations = value.graph.relations.filter(edge => !withdrawnEdge(edge)); }],
  ['P17', 'duplicate remaining withdrawal ledger record', value => { value.withdrawals.push(structuredClone(withdrawal(value))); }],
  ['P18', 'remaining withdrawal has zero sequence', value => { withdrawal(value).sequence = 0; }],
  ['P19', 'remaining withdrawal names declared non-producing event', value => { withdrawal(value).event = 'continue'; }],
];

const accepted = (session, event) => {
  const result = session.dispatch(event);
  assert.equal(result.outcome, 'accepted', `${event}: ${JSON.stringify(result)}`);
  return result;
};
const recordItems = archive => archive.filter(item => item.kind !== 'provenance');
const assertHeld = save => {
  assert.deepEqual(withdrawal(save), {evidence: 'b@2', because: 'b@2', sequence: 1, event: 'start'});
  assert.ok(Object.hasOwn(save.retired, 'b@2'));
  assert.ok((save.states.held.grounds ?? save.states.held.lineage).evidence.includes('b@2'));
  assert.equal(save.graph.relations.filter(withdrawnEdge).length, 1);
};
function build() {
  const session = real.open(source), events = [];
  try {
    for (const event of trace) events.push({event, result: accepted(session, event)});
    return {session, events};
  } catch (error) { session.close(); throw error; }
}
function assertCollected(save, archive, events) {
  assertHeld(save);
  assert.ok(!Object.hasOwn(save.retired, 'a@2'));
  assert.ok(!save.withdrawals.some(item => item.evidence === 'a@2'));
  const advance = events.find(item => item.event === 'advance').result.snapshot;
  assert.ok(advance.effects.some(item => item.kind === 'depart' && item.record === 'a@2'));
  assert.ok(!advance.effects.some(item => item.kind === 'depart' && item.record === 'b@2'));
  const departed = recordItems(archive).find(item => item.record === 'a@2');
  assert.deepEqual(departed.withdrawal, {evidence: 'a@2', because: 'a@2', sequence: 1, event: 'start'});
  for (const keys of markerPaths) {
    const markers = at(save, keys).departed;
    assert.equal(markers.length, 1);
    assert.deepEqual({history: markers[0].history, from: markers[0].from, through: markers[0].through, read: markers[0].read},
      {history: 'r', from: 1, through: 1, read: 1});
    assert.match(markers[0].archive_ref, /^[0-9a-f]{64}$/);
  }
  assert.ok(archive.some(item => item.record === 'r@1' && item.kind !== 'provenance'));
}

test('adversarial P00/P01: real mixed collection, exact restore, failed final binding and final-root release', () => {
  const original = build(), reference = build();
  let restored;
  try {
    const text = original.session.save(), snapshot = original.session.snapshot();
    const archive = reference.session.drainArchive();
    assert.equal(reference.session.save(), text);
    assertCollected(JSON.parse(text), archive, original.events);
    const history = explain(snapshot, [], {archive}).archive;
    assert.ok(history.records.every(item => item.status === 'complete'));
    assert.equal(history.authenticated, false);
    restored = real.restore(source, text);
    assert.equal(restored.save(), text);
    assert.deepEqual(restored.snapshot(), snapshot);
    assert.equal(restored.undrained, 0);
    assert.equal(original.session.undrained, archive.length);
    for (const session of [original.session, restored]) {
      const pending = session.undrained, result = session.dispatch('fail');
      assert.equal(result.outcome, 'rejected');
      assert.equal(result.origin, 'evaluation');
      assert.equal(session.save(), text);
      assert.deepEqual(session.snapshot(), snapshot);
      assert.equal(session.undrained, pending);
    }
    assert.deepEqual(original.session.drainArchive(), archive);
    assert.deepEqual(restored.drainArchive(), []);
    for (const event of ['continue', 'release']) {
      accepted(original.session, event); accepted(restored, event);
      assert.equal(restored.save(), original.session.save());
      assert.deepEqual(restored.snapshot(), original.session.snapshot());
      const nextArchive = original.session.drainArchive();
      assert.deepEqual(restored.drainArchive(), nextArchive);
      if (event === 'release') assert.deepEqual(recordItems(nextArchive).map(item => item.record), ['b@2']);
    }
  } finally { restored?.close(); original.session.close(); reference.session.close(); }
});

for (const [id, title, mutate] of mutants) {
  test(`adversarial ${id}: ${title} refuses`, () => {
    const {session, events} = build();
    try {
      const original = JSON.parse(session.save()), doc = structuredClone(original);
      assertCollected(original, session.drainArchive(), events);
      mutate(doc);
      assert.notDeepEqual(doc, original);
      assert.throws(() => {
        const unexpected = real.restore(source, JSON.stringify(doc)); unexpected.close();
      }, error => error.kind === 'restore');
    } finally { session.close(); }
  });
}

test('adversarial P02: authentic rc.15 save migrates only after an accepted event', () => {
  const session = real.restore(source, oldSave);
  try {
    const before = session.save(), snapshot = session.snapshot(), doc = JSON.parse(before);
    assertHeld(doc);
    assert.ok(Object.hasOwn(doc.retired, 'a@2'), 'restore must not migrate by collecting');
    assert.equal(session.undrained, 0);
    const failure = session.dispatch('fail');
    assert.equal(failure.outcome, 'rejected');
    assert.equal(failure.origin, 'evaluation');
    assert.equal(session.save(), before);
    assert.deepEqual(session.snapshot(), snapshot);
    assert.equal(session.undrained, 0);
    const continued = accepted(session, 'continue');
    assertHeld(JSON.parse(session.save()));
    assert.ok(continued.snapshot.effects.some(item => item.kind === 'depart' && item.record === 'a@2'));
    const archive = session.drainArchive();
    const moved = recordItems(archive).find(item => item.record === 'a@2');
    assert.deepEqual(moved.withdrawal, {evidence: 'a@2', because: 'a@2', sequence: 1, event: 'start'});
    const again = real.restore(source, session.save());
    try {
      assert.equal(again.save(), session.save());
      assert.deepEqual(again.snapshot(), session.snapshot());
    } finally { again.close(); }
  } finally { session.close(); }
});
