// Handing a session from one agent to another. scopeSession gives an agent a
// session that refuses every event outside its list, and keeps the events it
// accepted; replayOnto sends one agent's accepted events to another agent's
// session and reports the ones that no longer apply. Neither changes what the
// program means. See kit/examples/handoff/README.md.
//
// With { agent }, the scope also says which agent sent each event, so a
// program that declares agents checks the boundary itself and records the
// sender (spec/caveat-agents-0.1.md); accepted and replayOnto carry it.
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
  #agent;
  #accepted = [];

  constructor(session, events, options = {}) {
    if (session === null || typeof session?.dispatch !== 'function') throw new TypeError('session must be a CaveatSession');
    if (options === null || typeof options !== 'object') throw new TypeError('options must be an object');
    const { agent } = options;
    if (agent !== undefined && (typeof agent !== 'string' || !agent)) throw new TypeError('agent must be a non-empty string');
    this.#session = session;
    this.#events = eventList(events);
    this.#allowed = new Set(this.#events);
    this.#agent = agent ?? null;
  }

  get events() { return this.#events; }
  // The agent this scope sends as, or null.
  get agent() { return this.#agent; }
  get state() { return this.#session.state; }
  // A fresh copy each call: the events this scope sent that were accepted, in order.
  get accepted() {
    return this.#accepted.map(({ event, payload }) => (this.#agent === null
      ? { event, payload: structuredClone(payload) }
      : { event, payload: structuredClone(payload), agent: this.#agent }));
  }

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

  // The scope's agent, when it has one, is sent with every event; options
  // cannot name another.
  #options(options = {}) {
    if (options === null || typeof options !== 'object') throw new TypeError('options must be an object');
    if (options.agent !== undefined && options.agent !== this.#agent) {
      throw new TypeError(this.#agent === null ? 'this scope sends as no agent' : `this scope sends as ${this.#agent}`);
    }
    return this.#agent === null ? options : { ...options, agent: this.#agent };
  }

  dispatch(event, payload = {}, options) {
    const sent = this.#options(options);
    return this.#run(event, payload, () => (this.#agent === null && options === undefined
      ? this.#session.dispatch(event, payload)
      : this.#session.dispatch(event, payload, sent)));
  }
  dispatchView(event, payload = {}, options = {}) {
    const sent = this.#options(options);
    return this.#run(event, payload, () => this.#session.dispatchView(event, payload, sent));
  }
  dispatchViewDelta(event, payload = {}, options) {
    const sent = this.#options(options);
    return this.#run(event, payload, () => (this.#agent === null && options === undefined
      ? this.#session.dispatchViewDelta(event, payload)
      : this.#session.dispatchViewDelta(event, payload, sent)));
  }

  // What the event would do sent through this scope, without sending it: an
  // event outside the scope is answered as dispatch would refuse it.
  whatIf(event, payload = {}, options = {}) {
    const sent = this.#options(options);
    if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
    copy(payload);
    if (!this.#allowed.has(event)) {
      return {
        schema: DISPATCH_SCHEMA, outcome: 'rejected', origin: 'input', code: 'unknown_event',
        message: `${event} is not one of this session's events: ${this.#events.join(', ')}`,
      };
    }
    return this.#session.whatIf(event, payload, sent);
  }

  // Reading the session is open to every agent: the ledger is what it hands over.
  snapshot() { return this.#session.snapshot(); }
  snapshotText() { return this.#session.snapshotText(); }
  view(options) { return this.#session.view(options); }
  viewText(options) { return this.#session.viewText(options); }
  save(options) { return this.#session.save(options); }
}

// A session for one agent: only `events` may be sent through it, each as
// `options.agent` when that is given.
export function scopeSession(session, events, options) {
  return new ScopedSession(session, events, options);
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
    const agent = entry.agent;
    if (agent !== undefined && (typeof agent !== 'string' || !agent)) throw new TypeError(`events[${index}].agent must be a non-empty string`);
    // An entry with an agent is sent as that agent, so the receiving program
    // checks and records it as the first one did.
    const outcome = agent === undefined ? session.dispatch(entry.event, payload) : session.dispatch(entry.event, payload, { agent });
    const sent = agent === undefined ? { event: entry.event, payload: copy(payload) } : { event: entry.event, payload: copy(payload), agent };
    if (outcome.outcome === 'accepted') accepted.push(sent);
    else conflicts.push({ index, ...sent, outcome: { schema: outcome.schema, outcome: outcome.outcome, origin: outcome.origin, code: outcome.code, message: outcome.message } });
  });
  return { accepted, conflicts };
}
