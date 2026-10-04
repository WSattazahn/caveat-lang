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

test('F260: restore refuses a qualification no mechanism of the source can make', () => {
  // There is no static qualifies relation or qualify event effect for phantom.
  const original = runtime.open(graphSource);
  try {
    assert.equal(original.dispatch('observe').outcome, 'accepted');
    const saved = JSON.parse(original.save());
    assert.deepEqual(saved.graph.relations, [['sensor', 'supports', 'ready']]);
    saved.graph.relations.push(['phantom', 'qualifies', 'sensor']);
    refuses(graphSource, saved,
      /relation phantom qualifies sensor: no rule, declaration, reading, renewal or withdrawal of this program qualifies sensor with phantom/);
    assert.equal(original.dispatch('decide').outcome, 'accepted');
    assert.deepEqual(original.snapshot().commitment_grounds['go@1'], { evidence: ['sensor'], caveats: [] });
  } finally { original.close(); }
});

test('F260: a qualification a rule of the source can make restores as live and later freezes in grounds', () => {
  const source = `${graphSource}event doubt;\non doubt when observed(sensor) qualify sensor with phantom;\n`;
  const original = runtime.open(source);
  let resumed;
  let again;
  try {
    assert.equal(original.dispatch('observe').outcome, 'accepted');
    assert.equal(original.dispatch('doubt').outcome, 'accepted');
    const saved = original.save();
    assert.deepEqual(JSON.parse(saved).graph.relations,
      [['sensor', 'supports', 'ready'], ['phantom', 'qualifies', 'sensor']]);
    resumed = runtime.restore(source, saved);
    assert.equal(resumed.view().bindings.hud.carries, true);
    assert.deepEqual(resumed.snapshot().relations.find(edge => edge.relation === 'qualifies'),
      { from: 'phantom', relation: 'qualifies', to: 'sensor', origin: 'live' });
    assert.equal(original.dispatch('decide').outcome, 'accepted');
    assert.equal(resumed.dispatch('decide').outcome, 'accepted');
    const grounds = { evidence: ['sensor'], caveats: ['phantom'] };
    assert.deepEqual(resumed.snapshot().commitment_grounds['go@1'], grounds);
    assert.deepEqual(resumed.snapshot(), original.snapshot());
    again = runtime.restore(source, resumed.save());
    assert.deepEqual(again.snapshot(), resumed.snapshot());
  } finally { original.close(); resumed?.close(); again?.close(); }
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

test('F115: restore refuses a declared cue the last event cannot emit', () => {
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

test('F248: restore refuses an effect the last event cannot make', () => {
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

test('findings 123, 188-191: restore refuses attention no examination of the source could leave', () => {
  const session = runtime.open(effectSource);
  let resumed;
  let again;
  try {
    assert.equal(session.dispatch('observe').outcome, 'accepted');
    const unexamined = JSON.parse(session.save());
    assert.equal(unexamined.graph.attention, undefined);
    for (const [attention, message] of [
      [{ stale: 'examined' }, /examined caveats cost at least 1 attention to examine, more than the 0 the budget spent/],
      [{ stale: 'deferred' }, /caveat stale is deferred, which no reactive event leaves/],
      [{ stale: 'examining' }, /caveat stale is examining, which no reactive event leaves/],
    ]) {
      const saved = structuredClone(unexamined);
      saved.graph.attention = attention;
      refuses(effectSource, saved, message);
    }
    const noBudget = runtime.open(graphSource);
    try {
      assert.equal(noBudget.dispatch('observe').outcome, 'accepted');
      const saved = JSON.parse(noBudget.save());
      saved.graph.attention = { phantom: 'examined' };
      refuses(graphSource, saved, /caveat phantom is examined, but this program has no attention budget/);
    } finally { noBudget.close(); }
    // A genuine examination restores, plays on and restores again.
    assert.equal(session.dispatch('quiet').outcome, 'accepted');
    const examined = session.save();
    assert.deepEqual(JSON.parse(examined).graph.attention, { stale: 'examined' });
    resumed = runtime.restore(effectSource, examined);
    assert.deepEqual(resumed.snapshot(), session.snapshot());
    assert.deepEqual(resumed.dispatch('ring'), session.dispatch('ring'));
    again = runtime.restore(effectSource, resumed.save());
    assert.deepEqual(again.snapshot(), session.snapshot());
  } finally { session.close(); resumed?.close(); again?.close(); }
});

test('F330: restore refuses a caveat no mechanism of the source can attach to a provenance', () => {
  // Nothing attaches phantom to a value; stale qualifies sensor by declaration.
  const graph = runtime.open(graphSource);
  const sparse = runtime.open(sparseSource);
  let resumed;
  try {
    assert.equal(graph.dispatch('observe').outcome, 'accepted');
    assert.equal(graph.dispatch('decide').outcome, 'accepted');
    const decided = JSON.parse(graph.save());
    decided.commitment_bases['go@1'].provenance.caveats = ['phantom'];
    refuses(graphSource, decided, /commitment go@1: phantom cannot qualify any of its evidence/);
    assert.equal(sparse.dispatch('observe', { value: 85 }).outcome, 'accepted');
    const genuine = sparse.save();
    // serial rests on no evidence, so it can carry no caveat that needs one.
    const serial = JSON.parse(genuine);
    serial.states.serial.lineage = { evidence: [], caveats: ['stale'] };
    refuses(sparseSource, serial, /state serial: stale cannot qualify any of its evidence/);
    resumed = runtime.restore(sparseSource, genuine);
    assert.deepEqual(resumed.snapshot(), sparse.snapshot());
  } finally { graph.close(); sparse.close(); resumed?.close(); }
});

test('F247: restore refuses a field the schema lacks in a nested record', () => {
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

test('F248: restore refuses an effect the restored graph does not hold', () => {
  const session = runtime.open(effectSource);
  try {
    assert.equal(session.dispatch('observe').outcome, 'accepted');
    assert.equal(session.dispatch('quiet').outcome, 'accepted');
    for (const [effect, message] of [
      [{ kind: 'examine', caveat: 'sensor', cost: 1 }, /examine effect sensor must name a declared caveat/],
      [{ kind: 'examine', caveat: 'stale', cost: 2 }, /examine effect spends more attention than the budget spent/],
      [{ kind: 'qualify', evidence: 'stale', caveat: 'sensor' }, /qualify effect stale must name a declared evidence/],
    ]) {
      const saved = JSON.parse(session.save());
      saved.effects.push(effect);
      refuses(effectSource, saved, message);
    }
    assert.equal(session.dispatch('ring').outcome, 'accepted');
  } finally { session.close(); }
});
