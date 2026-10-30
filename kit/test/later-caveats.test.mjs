// The opt-in report is a difference from frozen/retained caveat names, not
// a complete per-ground qualification history. Exercise that boundary with
// runtime-produced snapshots, including saved and departed histories.
import assert from 'node:assert/strict';
import test from 'node:test';
import { explain, formatExplanation } from '../lib/explain.mjs';
import { real } from './helpers.mjs';

function send(session, event, payload = {}) {
  const result = session.dispatch(event, payload);
  assert.equal(result.outcome, 'accepted', `${event}: ${JSON.stringify(result)}`);
  return session.snapshot();
}

function revisions(snapshot, options = {}) {
  const before = structuredClone(snapshot);
  const plain = explain(snapshot, [], options);
  const enabled = explain(snapshot, [], { ...options, laterCaveats: true });
  const withoutField = structuredClone(enabled);
  for (const decision of withoutField.decisions) {
    for (const revision of decision.revisions) delete revision.qualified_since;
  }
  assert.equal(JSON.stringify(withoutField), JSON.stringify(plain), 'only the opted-in field changes JSON');
  assert.equal(formatExplanation(withoutField), formatExplanation(plain), 'default text remains unchanged');
  assert.equal(JSON.stringify(explain(snapshot, [], { ...options, laterCaveats: false })), JSON.stringify(plain));
  assert.deepEqual(snapshot, before, 'explaining never changes the snapshot');
  return enabled.decisions.flatMap(decision => decision.revisions);
}

const TAKE_BACK = `claim supported;
evidence study from "the study";
evidence correction from "the correction";
caveat disputed consequence material;
state value = 0;
decisions conclude limit 2;
event observe; event decide; event dispute; event undoubt; event retract;
on observe reveal study supports supported;
on observe set value = qualified(1, study);
on decide commit conclude because enough using value;
on dispute qualify study with disputed;
on undoubt unqualify study with disputed;
on retract reveal correction;
on retract withdraw study because correction;`;

test('later caveats follow take-back and reapplication, preserve grounds, and survive restore', () => {
  const session = real.open(TAKE_BACK);
  try {
    send(session, 'observe');
    const frozen = send(session, 'decide').commitment_grounds['conclude@1'];
    assert.deepEqual(revisions(session.snapshot())[0].qualified_since, []);
    send(session, 'dispute');
    const expected = [{ evidence: 'study', caveats: ['disputed'] }];
    assert.deepEqual(revisions(session.snapshot())[0].qualified_since, expected);
    send(session, 'undoubt');
    assert.deepEqual(revisions(session.snapshot())[0].qualified_since, []);
    assert.equal(session.snapshot().taken_back.length, 1, 'take-back history remains');
    send(session, 'dispute');
    assert.deepEqual(revisions(session.snapshot())[0].qualified_since, expected, 'reapplied caveat appears once');
    assert.deepEqual(session.snapshot().commitment_grounds['conclude@1'], frozen);
    for (const options of [{}, { compact: true }]) {
      const restored = real.restore(TAKE_BACK, session.save(options));
      try {
        assert.deepEqual(revisions(restored.snapshot())[0].qualified_since, expected);
        assert.deepEqual(restored.snapshot().commitment_grounds['conclude@1'], frozen);
        send(restored, 'undoubt');
        assert.deepEqual(revisions(restored.snapshot())[0].qualified_since, []);
      } finally { restored.close(); }
    }
    send(session, 'retract');
    const [withdrawn] = revisions(session.snapshot());
    assert.deepEqual(withdrawn.qualified_since, expected, 'withdrawn is not duplicated as a late caveat');
    assert.equal(withdrawn.withdrawn[0].evidence, 'study');
    assert.deepEqual(withdrawn.grounds, frozen);
  } finally { session.close(); }
});

for (const retained of [false, true]) {
  test(`later caveats exclude a name already ${retained ? 'explicitly retained' : 'recorded through another ground'}`, () => {
    const source = `claim supported;
evidence alpha from "the first source"; evidence beta from "the second source";
caveat shared consequence material; caveat fresh consequence material;
${retained ? '' : 'shared qualifies beta;'}
state value = 0; decisions conclude limit 2;
event observe; event decide; event doubt; event new_doubt;
on observe reveal alpha supports supported;
on observe reveal beta supports supported;
on observe set value = qualified(1, alpha) + qualified(1, beta);
on decide commit conclude because enough using value${retained ? ' retaining shared' : ''};
on doubt qualify alpha with shared;
on new_doubt qualify alpha with fresh;`;
    const session = real.open(source);
    try {
      send(session, 'observe');
      const frozen = send(session, 'decide').commitment_grounds['conclude@1'];
      send(session, 'doubt');
      const snapshot = session.snapshot();
      assert.ok(explain(snapshot).evidence.find(item => item.id === 'alpha').caveats.includes('shared'));
      assert.deepEqual(revisions(snapshot)[0].qualified_since, [], 'not a history of each later qualification');
      send(session, 'new_doubt');
      const [revision] = revisions(session.snapshot());
      assert.deepEqual(revision.qualified_since, [{ evidence: 'alpha', caveats: ['fresh'] }]);
      assert.deepEqual(revision.grounds, frozen);
    } finally { session.close(); }
  });
}

test('later caveats distinguish renewable occurrences and rejected updates', () => {
  const source = `claim supported;
evidence study from "successive studies"; renewable study limit 3;
caveat disputed consequence material;
state value = 0; decisions conclude limit 3;
event observe; event decide; event refresh; event dispute index min 1 max 3;
on observe reveal study supports supported;
on observe set value = qualified(1, study);
on decide when committed(conclude) reopen conclude because study;
on decide commit conclude because enough using value;
on refresh renew study;
on dispute qualify occurrence(study, index) with disputed;`;
  const session = real.open(source);
  try {
    for (const event of ['observe', 'decide', 'refresh', 'observe', 'decide']) send(session, event);
    const frozen = structuredClone(session.snapshot().commitment_grounds);
    send(session, 'dispute', { index: 1 });
    let [first, second] = revisions(session.snapshot());
    assert.deepEqual(first.qualified_since, [{ evidence: 'study', caveats: ['disputed'] }]);
    assert.deepEqual(second.qualified_since, [], 'an earlier occurrence does not qualify its replacement');
    const saved = session.save();
    const rejected = session.dispatch('dispute', { index: 3 });
    assert.equal(rejected.outcome, 'rejected');
    assert.equal(session.save(), saved, 'rejected qualification preserves the complete state');
    send(session, 'dispute', { index: 2 });
    [first, second] = revisions(session.snapshot());
    assert.deepEqual(first.qualified_since, [{ evidence: 'study', caveats: ['disputed'] }]);
    assert.deepEqual(second.qualified_since, [{ evidence: 'study@2', caveats: ['disputed'] }]);
    assert.deepEqual(session.snapshot().commitment_grounds, frozen);
  } finally { session.close(); }
});

test('later caveats do not turn an archived qualification into a current relation', () => {
  const source = `claim seen; evidence glimpse from "a glimpse";
caveat disputed consequence material;
readings sighting from glimpse window 2; decisions trust limit 4; journal window 2;
event look; event decide; event dispute;
on look sample sighting = 1 supports seen;
on decide when committed(trust) reopen trust because latest(sighting);
on decide commit trust because enough using history_count(sighting);
on dispute qualify latest(sighting) with disputed;`;
  const session = real.open(source);
  try {
    for (const event of ['look', 'decide', 'dispute']) send(session, event);
    assert.deepEqual(revisions(session.snapshot())[0].qualified_since, [{ evidence: 'sighting@1', caveats: ['disputed'] }]);
    for (const event of ['look', 'look', 'decide']) send(session, event);
    const snapshot = session.snapshot();
    const archive = session.drainArchive();
    assert.ok(archive.some(entry => entry.record === 'sighting@1'), 'the former ground has departed');
    const plain = revisions(snapshot)[0];
    const supplied = revisions(snapshot, { archive })[0];
    assert.ok(plain.grounds.departed.length > 0);
    assert.deepEqual(plain.qualified_since, []);
    assert.deepEqual(supplied.qualified_since, [], 'archive content cannot claim live qualification');
    assert.ok(supplied.grounds.departed.some(marker => marker.records?.includes('sighting@1')));
    const restored = real.restore(source, session.save());
    try { assert.deepEqual(revisions(restored.snapshot(), { archive })[0].qualified_since, []); }
    finally { restored.close(); }
  } finally { session.close(); }
});
