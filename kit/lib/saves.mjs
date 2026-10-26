// Carrying a save onto a changed program. A save restores only with the exact
// source that wrote it, so a patched program refuses every earlier save. A
// replay save is a list of accepted events: savedEvents reads them back, by
// name, with the source that wrote the save, and carrySave sends them to a
// session of the new source, which decides each one afresh and reports the
// ones that no longer apply. Neither edits a save or loosens restore: the save
// is first restored with its own source, and the new session is the one its
// events make. See kit/docs/GAMES.md, "Saves".
import { CaveatError } from './session.mjs';
import { replayOnto } from './handoff.mjs';

const REPLAY_V1 = 'caveat-replay-save/0.1';
const REPLAY_V2 = 'caveat-replay-save/0.2';

// A parameter's value as a payload names it: a typed parameter's position,
// from 1, as its member's name; an identifier as its text; a number as is.
function named(parameter, value) {
  if (parameter.type === 'entity' || parameter.type === 'member') {
    const member = Number.isInteger(value) ? parameter.members[value - 1] : undefined;
    if (member === undefined) throw new CaveatError('restore', `save names position ${value} of ${parameter.name}, which has no such member`);
    return member;
  }
  return value;
}

function payloadOf(signature, values) {
  const payload = {};
  signature.parameters.forEach((parameter, index) => { payload[parameter.name] = named(parameter, values[index]); });
  return payload;
}

// Replay Save 0.1: [event] or [event, payload], typed parameters as positions.
function fromNamed(events, signatures) {
  return events.map(([event, positions = {}]) => {
    const signature = signatures.get(event);
    if (!signature) throw new CaveatError('restore', `save names event ${event}, which the program does not have`);
    return { event, payload: payloadOf(signature, signature.parameters.map(parameter => positions[parameter.name])) };
  });
}

// Replay Save 0.2: one flat list (or base-36 text) of each event's position
// among the program's events in name order, then its parameters' values.
function fromPositional(events, ordered) {
  const values = typeof events === 'string' ? [...events].map(digit => parseInt(digit, 36)) : events;
  const accepted = [];
  for (let at = 0; at < values.length;) {
    const signature = ordered[values[at]];
    if (!signature) throw new CaveatError('restore', `save names event position ${values[at]}, which the program does not have`);
    const parameters = values.slice(at + 1, at + 1 + signature.parameters.length);
    accepted.push({ event: signature.name, payload: payloadOf(signature, parameters) });
    at += 1 + signature.parameters.length;
  }
  return accepted;
}

/**
 * The events a replay save holds, in order, each with a payload that names
 * its members, read with the source that wrote the save. Throws
 * `CaveatError("restore")` when the save does not restore with that source,
 * and when it is a state save, which holds no events.
 */
export function savedEvents(runtime, source, saved) {
  runtime.restore(source, saved).close();
  const { schema, events } = JSON.parse(saved);
  const ordered = runtime.interface(source).events;
  if (schema === REPLAY_V1) return fromNamed(events, new Map(ordered.map(signature => [signature.name, signature])));
  if (schema === REPLAY_V2) return fromPositional(events, ordered);
  throw new CaveatError('restore', 'a state save holds no events to replay: keep the accepted events beside it, or save with { compact: true } while they fit');
}

/**
 * A session of `newSource` made by sending it the events of a save written
 * with `oldSource`. Each event the new program refuses is a conflict and
 * changes nothing; the rest are still sent. The save is not changed.
 */
export function carrySave(runtime, oldSource, saved, newSource) {
  const events = savedEvents(runtime, oldSource, saved);
  const session = runtime.open(newSource);
  try {
    const { accepted, conflicts } = replayOnto(session, events);
    return { session, accepted, conflicts };
  } catch (error) {
    if (session.state === 'open') session.close();
    throw error;
  }
}
