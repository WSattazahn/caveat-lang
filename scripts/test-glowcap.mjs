// Drives game/glowcap.cav through the lean reactive runtime and checks what a
// player reads on web/glowcap.html: labels, beliefs, the trust journal, late
// caveats and rejections, all through web/glowcap-explain.js. No browser.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import init, { WebReactiveSession } from '../dist/pkg-reactive/caveat_runtime.js';
import { beliefCard, glossary, journalLines, lateCaveats, mushroomCards, rejection } from '../web/glowcap-explain.js';

const root = new URL('../', import.meta.url);
await init({ module_or_path: await readFile(new URL('dist/pkg-reactive/caveat_runtime_bg.wasm', root)) });
const source = await readFile(new URL('dist/glowcap.cav', root), 'utf8');

const session = new WebReactiveSession(source);
const snapshot = JSON.parse(session.snapshot());
const words = glossary(snapshot);
const ids = snapshot.world.entities.filter((e) => e.kind === 'mushroom').map((e) => e.id);
assert.deepEqual(ids, ['cave', 'pool', 'ruin', 'grove']);

let view = JSON.parse(session.view());
const act = (event, payload) => { view = JSON.parse(session.dispatch_view(event, JSON.stringify(payload))); };
const wait = (seconds) => { for (let i = 0; i < seconds * 20; i += 1) act('tick', { dt: 0.05 }); };
const card = (id) => mushroomCards(view, words, ids).find((c) => c.id === id);

// Nothing is known yet, and nothing is claimed.
assert.equal(card('pool').label, 'Glowing mushroom');
assert.deepEqual(card('pool').because, []);
assert.equal(beliefCard(view, words).title, 'No belief yet');

// One glowcap: a belief, and a decision that remembers what it rests on.
act('absorb', { target: 'cave', sort: 'glowcap' });
assert.equal(card('pool').label, 'Probably a glowcap');
assert.deepEqual(card('pool').because, ['You absorbed the cave mushroom.']);
assert.deepEqual(beliefCard(view, words).because, ['You absorbed the cave mushroom.']);
assert.deepEqual(journalLines(view, words).map((l) => [l.text, l.because]), [
  ['Started trusting glowing mushrooms', ['You absorbed the cave mushroom.']],
]);

// A look-alike: the label cites the counterexample, trust reopens.
act('absorb', { target: 'pool', sort: 'duskcap' });
assert.equal(card('ruin').label, 'Could be a duskcap — taste first');
assert.deepEqual(card('ruin').because, ['You absorbed the pool mushroom.']);
assert.equal(beliefCard(view, words).title, 'Uncertain');
assert.deepEqual(journalLines(view, words).at(-1).text, 'Stopped trusting glowing mushrooms');

// The glow wears off; a taste in the dark carries its caveat.
wait(31);
act('taste', { target: 'ruin', sort: 'glowcap' });
assert.equal(card('ruin').label, 'Probably a glowcap (tasted in the dark)');
assert.deepEqual(card('ruin').because, ['You tasted the ruin mushroom.']);
assert.deepEqual(card('ruin').caveats, ['Tasted in the dark; taste can be wrong without light.']);

// A minute later the taste fades. The caveat arrives late and reaches every
// value built on the taste.
wait(59);
const lateEvents = [];
for (let i = 0; i < 40 && lateEvents.length === 0; i += 1) {
  act('tick', { dt: 0.05 });
  lateEvents.push(...lateCaveats(view, words));
}
assert.deepEqual(lateEvents, [{ evidence: 'You tasted the ruin mushroom.', caveat: 'The taste has faded from memory.' }]);
assert.equal(card('ruin').label, 'Probably a glowcap (taste has faded)');
assert.deepEqual(card('ruin').caveats.sort(), [
  'Tasted in the dark; taste can be wrong without light.',
  'The taste has faded from memory.',
].sort());
assert.ok(beliefCard(view, words).caveats.includes('The taste has faded from memory.'));
// The decision already made keeps what it knew.
assert.deepEqual(journalLines(view, words)[0].caveats, []);

// Absorbing the tasted glowcap is the second glowcap since the last
// contradiction: trust returns, grounded on those two, faded taste and all.
act('absorb', { target: 'ruin', sort: 'glowcap' });
const regained = journalLines(view, words).at(-1);
assert.equal(regained.text, 'Trusted glowing mushrooms again');
assert.deepEqual(regained.because, ['You tasted the ruin mushroom.', 'You absorbed the ruin mushroom.']);
assert.ok(regained.caveats.includes('The taste has faded from memory.'));

// Not allowed is not allowed: the event is rejected and nothing changes.
const before = session.view();
assert.throws(() => act('absorb', { target: 'ruin', sort: 'glowcap' }), (error) => rejection(error) === 'already absorbed');
assert.throws(() => act('absorb', { target: 'meadow', sort: 'glowcap' }), /does not accept meadow/);
assert.equal(session.view(), before);

session.free();
console.log('Glowcap explainer checks pass: labels, belief, journal, late caveats and rejections all come from the runtime.');
