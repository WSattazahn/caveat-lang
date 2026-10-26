// Carrying a save onto a changed program: savedEvents reads a replay save's
// events back by name with the source that wrote it, and carrySave replays
// them onto the new source. The save itself is never changed, and restore
// still refuses it with any other source.
import test from 'node:test';
import assert from 'node:assert/strict';
import { real } from './helpers.mjs';
import { CaveatError } from '../lib/session.mjs';
import { carrySave, savedEvents } from '../lib/saves.mjs';

const OLD = `
place field kind gate;
entity north kind gate at field;
entity south kind gate at field;
event report gate kind gate, says in clear guarded, who id;
event wait;
event aim heading min 0 max 360;
identifiers limit 8;
for gate as $g routed by gate {
    readings $g_word from radio limit 4;
    on report when says == says.clear sample $g_word = 1;
    on report when says == says.guarded sample $g_word = 0;
};
evidence radio from "the radio";
state heading_now = 0 min 0 max 360;
on aim set heading_now = heading;
bind hud.heading = heading_now;
`;
// The patch: a new entity sorts first, an event is added, and "wait" is gone.
const NEW = OLD.replace('entity north kind gate at field;', 'entity east kind gate at field;\nentity north kind gate at field;')
  .replace('event wait;', 'event rest;');

const STORY = [
  ['report', { gate: 'south', says: 'guarded', who: 'ocelot' }],
  ['wait', {}],
  ['report', { gate: 'north', says: 'clear', who: 'eva' }],
  ['aim', { heading: 270.5 }],
];

function played(save) {
  const session = real.open(OLD);
  try {
    for (const [event, payload] of STORY) assert.equal(session.dispatch(event, payload).outcome, 'accepted');
    return session.save(save);
  } finally { session.close(); }
}

const named = STORY.map(([event, payload]) => ({ event, payload }));

test('both replay save forms read back as the events that made them, members by name', () => {
  for (const options of [{ compact: true }, { compact: true, positional: true }]) {
    const saved = played(options);
    assert.match(saved, /caveat-replay-save/);
    assert.deepEqual(savedEvents(real, OLD, saved), named);
  }
});

test('a positional save written as base-36 digits reads back too', () => {
  const source = 'event pick n min 0 max 30; event wait; state at = 0 min 0 max 30; on pick set at = n;';
  const session = real.open(source);
  try {
    for (const n of [3, 0, 29]) session.dispatch('pick', { n });
    session.dispatch('wait');
    const saved = session.save({ compact: true, positional: true });
    assert.equal(typeof JSON.parse(saved).events, 'string');
    assert.deepEqual(savedEvents(real, source, saved),
      [{ event: 'pick', payload: { n: 3 } }, { event: 'pick', payload: { n: 0 } }, { event: 'pick', payload: { n: 29 } }, { event: 'wait', payload: {} }]);
  } finally { session.close(); }
});

test('carrySave replays onto the patched program by name, and reports what no longer applies', () => {
  const saved = played({ compact: true, positional: true });
  assert.throws(() => real.restore(NEW, saved), error => error instanceof CaveatError && error.kind === 'restore');
  const { session, accepted, conflicts } = carrySave(real, OLD, saved, NEW);
  try {
    assert.deepEqual(accepted.map(item => item.event), ['report', 'report', 'aim']);
    assert.deepEqual(conflicts.map(item => [item.index, item.event, `${item.outcome.origin}/${item.outcome.code}`]), [[1, 'wait', 'input/unknown_event']]);
    // "north" is now the second gate; the report reached north, not east.
    const snapshot = session.snapshot();
    assert.equal(snapshot.reading_streams.north_word.occurrences.length, 1);
    assert.equal(snapshot.reading_streams.east_word.occurrences.length, 0);
    assert.equal(session.view().bindings.hud.heading, 270.5);
  } finally { session.close(); }
  assert.equal(real.openSessions, 0);
});

test('the save is untouched, and still restores with its own source only', () => {
  const saved = played({ compact: true });
  const copy = String(saved);
  carrySave(real, OLD, saved, NEW).session.close();
  assert.equal(saved, copy);
  real.restore(OLD, saved).close();
});

test('a save that does not restore with the stated source is refused, as restore refuses it', () => {
  const saved = played({ compact: true });
  assert.throws(() => savedEvents(real, NEW, saved), error => error instanceof CaveatError && error.kind === 'restore');
  const edited = saved.replace('"says":2', '"says":1');
  assert.notEqual(edited, saved);
  // A replay save is not authenticated (Replay Save 0.1): an edited one whose
  // events still run restores, and reads back as edited.
  assert.equal(savedEvents(real, OLD, edited)[0].payload.says, 'clear');
});

test('a state save holds no events, and is refused with what to do instead', () => {
  const saved = played();
  assert.throws(() => savedEvents(real, OLD, saved), error => error instanceof CaveatError && error.kind === 'restore' && /state save holds no events/.test(error.message));
  assert.equal(real.openSessions, 0);
});
