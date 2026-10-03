// Characterize the current save/0.1 boundary. These accepted edits are not
// authenticated history or promises that future save schemas accept them.
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rmdir, unlink } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { loadRuntimeFromDirectory } from '../lib/node.mjs';
import { saveTextExample, source as zeroSource } from '../examples/agent-evidence/save-text.mjs';

const runtime = await loadRuntimeFromDirectory();
const empty = { evidence: [], caveats: [] };
const graphSource = `
claim ready;
evidence sensor from "sensor";
caveat phantom consequence low;
decisions go limit 2;
event observe;
event decide;
on observe reveal sensor supports ready;
on decide commit go because enough using qualified(1, sensor);
bind hud.carries = carries(sensor, phantom);
`;
const sparseSource = `
claim ready;
evidence sensor from "sensor";
caveat stale consequence low;
stale qualifies sensor;
state score = 0 min 0 max 100;
state serial = 0 min 0 max 10;
decisions before limit 1;
decisions after limit 1;
event observe value min 0 max 100;
event decide;
on observe reveal sensor supports ready;
on observe set score = qualified(value, sensor);
on observe set serial = 1;
on observe commit before because enough using score;
on decide commit after because enough using score;
bind hud.score = score;
`;

const effectSource = `
budget 2;
claim ready;
evidence sensor from "sensor";
caveat stale consequence low;
cue ping toast "Ping" 1;
event observe;
event ring;
event quiet;
proc chime() { emit ping; };
on observe reveal sensor supports ready;
on ring call chime();
on quiet examine stale cost 1;
`;

function refuses(source, edited, message = /cannot restore save:/) {
  assert.throws(() => {
    const unexpected = runtime.restore(source, JSON.stringify(edited));
    unexpected.close();
  }, error => error.kind === 'restore' && /cannot restore save:/.test(error.message)
    && message.test(error.message));
}

test('restore accepts one structurally valid unauthored qualification as live and later freezes it in grounds', () => {
  // There is no static qualifies relation or qualify event effect in this source.
  const original = runtime.open(graphSource);
  let editedSession;
  let resumed;
  try {
    assert.equal(original.dispatch('observe').outcome, 'accepted');
    const saved = JSON.parse(original.save());
    assert.deepEqual(saved.graph.relations, [['sensor', 'supports', 'ready']]);
    assert.equal(original.view().bindings.hud.carries, false);
    saved.graph.relations.push(['phantom', 'qualifies', 'sensor']);
    editedSession = runtime.restore(graphSource, JSON.stringify(saved));
    assert.equal(editedSession.view().bindings.hud.carries, true);
    assert.deepEqual(editedSession.snapshot().relations.find(edge => edge.relation === 'qualifies'),
      { from: 'phantom', relation: 'qualifies', to: 'sensor', origin: 'live' });
    assert.equal(original.dispatch('decide').outcome, 'accepted');
    assert.equal(editedSession.dispatch('decide').outcome, 'accepted');
    assert.deepEqual(original.snapshot().commitment_grounds['go@1'], { evidence: ['sensor'], caveats: [] });
    const grounds = { evidence: ['sensor'], caveats: ['phantom'] };
    assert.deepEqual(editedSession.snapshot().commitment_bases['go@1'], { value: 1, provenance: grounds });
    assert.deepEqual(editedSession.snapshot().commitment_grounds['go@1'], grounds);
    assert.deepEqual(editedSession.snapshot().decision_journal[0].caveats, ['phantom']);
    resumed = runtime.restore(graphSource, editedSession.save());
    assert.deepEqual(resumed.snapshot(), editedSession.snapshot());
  } finally { original.close(); editedSession?.close(); resumed?.close(); }
});

test('restore refuses unknown endpoints and wrong endpoint kinds for an injected qualification', () => {
  const session = runtime.open(graphSource);
  try {
    assert.equal(session.dispatch('observe').outcome, 'accepted');
    for (const edge of [
      ['missing', 'qualifies', 'sensor'],
      ['phantom', 'qualifies', 'missing'],
      ['ready', 'qualifies', 'sensor'],
      ['phantom', 'qualifies', 'ready'],
    ]) {
      const saved = JSON.parse(session.save());
      saved.graph.relations.push(edge);
      refuses(graphSource, saved);
    }
  } finally { session.close(); }
});

test('omitting one changed sparse state restores its initializer without rewriting earlier grounds', () => {
  const original = runtime.open(sparseSource);
  let restored;
  let resumed;
  try {
    assert.equal(original.dispatch('observe', { value: 85 }).outcome, 'accepted');
    const before = original.snapshot();
    const grounds = { evidence: ['sensor'], caveats: ['stale'] };
    assert.deepEqual(before.qualified_values.score, { value: 85, provenance: grounds });
    assert.deepEqual(before.value_grounds.score, grounds);
    const saved = JSON.parse(original.save());
    assert.deepEqual(Object.keys(saved.states).sort(), ['score', 'serial']);
    delete saved.states.score;
    restored = runtime.restore(sparseSource, JSON.stringify(saved));
    const after = restored.snapshot();
    assert.equal(after.values.serial, 1, 'the other changed state remains saved');
    assert.deepEqual(after.qualified_values.score, { value: 0, provenance: empty });
    assert.deepEqual(after.value_grounds.score, empty);
    assert.equal(restored.view().bindings.hud.score, 0);
    assert.deepEqual(after.commitment_bases, before.commitment_bases);
    assert.deepEqual(after.commitment_grounds, before.commitment_grounds);
    assert.deepEqual(after.decision_journal, before.decision_journal);
    assert.deepEqual(after.commitments, before.commitments);
    assert.equal(restored.dispatch('decide').outcome, 'accepted');
    const decided = restored.snapshot();
    assert.deepEqual(decided.commitment_bases['after@1'], { value: 0, provenance: empty });
    assert.deepEqual(decided.commitment_grounds['after@1'], empty);
    assert.deepEqual(decided.commitment_bases['before@1'], before.commitment_bases['before@1']);
    assert.deepEqual(decided.commitment_grounds['before@1'], grounds);
    assert.deepEqual(decided.decision_journal[0], before.decision_journal[0]);
    resumed = runtime.restore(sparseSource, restored.save());
    assert.deepEqual(resumed.snapshot(), decided);
  } finally { original.close(); restored?.close(); resumed?.close(); }
});

test('sparse saves still require the states map and valid declared entries with values', () => {
  const session = runtime.open(sparseSource);
  try {
    assert.equal(session.dispatch('observe', { value: 85 }).outcome, 'accepted');
    for (const edit of [
      saved => { delete saved.states; },
      saved => { delete saved.states.score.value; },
      saved => { saved.states.unknown = { value: 0 }; },
      saved => { saved.states.score.value = 101; },
    ]) {
      const saved = JSON.parse(session.save());
      edit(saved);
      refuses(sparseSource, saved);
    }
  } finally { session.close(); }
});

test('the executable host preserves save text; JSON normalization changes signed-zero decisions', async () => {
  const directory = await mkdtemp(path.join(tmpdir(), 'caveat-save-text-'));
  const checkpoint = path.join(directory, 'checkpoint.json');
  let normalized;
  try {
    const result = await saveTextExample(runtime, checkpoint);
    assert.equal(await readFile(checkpoint, 'utf8'), result.savedText);
    assert.equal(result.loadedText, result.savedText);
    assert.match(result.savedText, /"value":-0\.0/);
    assert.equal(result.negativeZero, true);
    assert.equal(result.angle, -Math.PI);
    assert.equal(result.decision, -1);
    const parsed = JSON.parse(result.savedText);
    assert.equal(Object.is(parsed.states.x.value, -0), true);
    const rewritten = JSON.stringify(parsed);
    assert.notEqual(rewritten, result.savedText);
    assert.equal(Object.is(JSON.parse(rewritten).states.x.value, 0), true);
    normalized = runtime.restore(zeroSource, rewritten);
    assert.equal(Object.is(normalized.snapshot().values.x, 0), true);
    assert.equal(normalized.view().bindings.hud.angle, Math.PI);
    assert.equal(normalized.dispatch('decide').outcome, 'accepted');
    assert.equal(normalized.snapshot().commitment_bases['direction@1'].value, 1);
  } finally {
    normalized?.close();
    await unlink(checkpoint).catch(error => { if (error.code !== 'ENOENT') throw error; });
    await rmdir(directory);
  }
});

test('restore refuses a declared cue the last event cannot emit', () => {
  const session = runtime.open(effectSource);
  let resumed;
  try {
    assert.equal(session.dispatch('ring').outcome, 'accepted');
    const rung = JSON.parse(session.save());
    assert.deepEqual(rung.cues, ['ping']);
    resumed = runtime.restore(effectSource, JSON.stringify(rung));
    assert.deepEqual(resumed.snapshot().cues, session.snapshot().cues);
    assert.equal(session.dispatch('quiet').outcome, 'accepted');
    const quiet = JSON.parse(session.save());
    assert.equal(quiet.cues, undefined);
    quiet.cues = ['ping'];
    quiet.cue_qualifications = [{}];
    refuses(effectSource, quiet, /cue ping is not one the last event can emit/);
    assert.equal(resumed.dispatch('quiet').outcome, 'accepted');
    assert.equal(session.dispatch('ring').outcome, 'accepted');
  } finally { session.close(); resumed?.close(); }
});

test('restore refuses an effect the last event cannot make', () => {
  const session = runtime.open(effectSource);
  let resumed;
  try {
    assert.equal(session.dispatch('quiet').outcome, 'accepted');
    const examined = session.save();
    assert.deepEqual(JSON.parse(examined).effects, [{ kind: 'examine', caveat: 'stale', cost: 1 }]);
    resumed = runtime.restore(effectSource, examined);
    assert.deepEqual(resumed.snapshot().effects, session.snapshot().effects);
    assert.equal(session.dispatch('observe').outcome, 'accepted');
    const observed = JSON.parse(session.save());
    observed.effects.push({ kind: 'examine', caveat: 'stale', cost: 1 });
    refuses(effectSource, observed, /examine effect is not one the last event can make/);
    assert.equal(resumed.dispatch('observe').outcome, 'accepted');
    assert.equal(session.dispatch('ring').outcome, 'accepted');
  } finally { session.close(); resumed?.close(); }
});

test('restore refuses a field the schema lacks in a nested record', () => {
  const session = runtime.open(sparseSource);
  try {
    assert.equal(session.dispatch('observe', { value: 85 }).outcome, 'accepted');
    for (const edit of [
      saved => { saved.decision_journal[0].zz = 1; },
      saved => { saved.commitment_bases['before@1'].zz = 1; },
      saved => { saved.decision_series.before.revisions[0].zz = 1; },
      saved => { saved.effects[0].zz = 1; },
    ]) {
      const saved = JSON.parse(session.save());
      edit(saved);
      refuses(sparseSource, saved, /unknown field `zz`/);
    }
    assert.equal(session.dispatch('decide').outcome, 'accepted');
  } finally { session.close(); }
});
