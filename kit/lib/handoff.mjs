// Handing a session from one agent to another. The runtime does not know who
// sends an event, so a host that lets a main agent and its sub-agents share
// one program keeps the boundary itself: scopeSession gives an agent a session
// that refuses every event outside its list, and keeps the events it
// accepted; replayOnto sends one agent's accepted events to another agent's
// session and reports the ones that no longer apply. Neither changes what the
// program means. See kit/examples/handoff/README.md.
import { DISPATCH_SCHEMA, payloadText } from './session.mjs';

function eventList(events) {
  if (!Array.isArray(events)) throw new TypeError('events must be an array of event names');
  const seen = new Set();
  for (const name of events) {
    if (typeof name !== 'string' || !name) throw new TypeError('each event must be a non-empty string');
    if (seen.has(name)) throw new TypeError(`duplicate event ${name}`);
    seen.add(name);
  }
  return Object.freeze([...events]);
}

// The JSON the session was sent, so a later replay sends the same payload.
const copy = payload => JSON.parse(payloadText(payload));

export class ScopedSession {
  #session;
  #events;
  #allowed;
  #accepted = [];

  constructor(session, events) {
    if (session === null || typeof session?.dispatch !== 'function') throw new TypeError('session must be a CaveatSession');
    this.#session = session;
    this.#events = eventList(events);
    this.#allowed = new Set(this.#events);
  }

  get events() { return this.#events; }
  get state() { return this.#session.state; }
  // A fresh copy each call: the events this scope sent that were accepted, in order.
  get accepted() { return this.#accepted.map(({ event, payload }) => ({ event, payload: structuredClone(payload) })); }

  // An event outside the list is refused here, as an event the program does
  // not have, and never reaches the session. The payload is checked first,
  // so a payload dispatch would refuse is refused the same way.
  #run(event, payload, send) {
    if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
    const sent = copy(payload);
    if (!this.#allowed.has(event)) {
      return {
        schema: DISPATCH_SCHEMA, outcome: 'rejected', origin: 'input', code: 'unknown_event',
        message: `${event} is not one of this session's events: ${this.#events.join(', ')}`,
      };
    }
    const outcome = send();
    if (outcome.outcome === 'accepted') this.#accepted.push({ event, payload: sent });
    return outcome;
  }

  dispatch(event, payload = {}) { return this.#run(event, payload, () => this.#session.dispatch(event, payload)); }
  dispatchView(event, payload = {}, options = {}) { return this.#run(event, payload, () => this.#session.dispatchView(event, payload, options)); }
  dispatchViewDelta(event, payload = {}) { return this.#run(event, payload, () => this.#session.dispatchViewDelta(event, payload)); }

  // Reading the session is open to every agent: the ledger is what it hands over.
  snapshot() { return this.#session.snapshot(); }
  snapshotText() { return this.#session.snapshotText(); }
  view(options) { return this.#session.view(options); }
  viewText(options) { return this.#session.viewText(options); }
  save(options) { return this.#session.save(options); }
}

// A session for one agent: only `events` may be sent through it.
export function scopeSession(session, events) {
  return new ScopedSession(session, events);
}

// Sends each event to `session` in order, as `accepted` lists them. A refused
// event is a conflict: it changes nothing, and the next one is still sent.
// The result depends on the order: replaying A onto B can differ from B onto A.
export function replayOnto(session, events) {
  if (!Array.isArray(events)) throw new TypeError('events must be an array of {event, payload}');
  const accepted = [];
  const conflicts = [];
  events.forEach((entry, index) => {
    if (entry === null || typeof entry !== 'object' || typeof entry.event !== 'string') {
      throw new TypeError(`events[${index}] must be {event, payload}`);
    }
    const payload = entry.payload ?? {};
    const outcome = session.dispatch(entry.event, payload);
    if (outcome.outcome === 'accepted') accepted.push({ event: entry.event, payload: copy(payload) });
    else conflicts.push({ index, event: entry.event, payload: copy(payload), outcome: { schema: outcome.schema, outcome: outcome.outcome, origin: outcome.origin, code: outcome.code, message: outcome.message } });
  });
  return { accepted, conflicts };
}
