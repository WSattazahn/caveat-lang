// One way to load the reactive runtime, dispatch events, read results, and
// save or resume a session. This module has no Node imports; see node.mjs for
// loading the runtime from a directory. Outcomes follow
// spec/caveat-dispatch-0.1.md: accepted and rejected are values, and anything
// else is a CaveatError of kind "fatal" after which the session is unusable.

export const DISPATCH_SCHEMA = 'caveat-dispatch/0.1';
export const ORIGINS = Object.freeze(['policy', 'input', 'evaluation', 'limit']);

// kind: "load", "restore", "payload", "fatal" or "closed".
export class CaveatError extends Error {
  constructor(kind, message, report = null) {
    super(message);
    this.name = 'CaveatError';
    this.kind = kind;
    this.report = report;
  }
}

const messageOf = error => (typeof error === 'string' ? error : String(error?.message ?? error));

// A WebAssembly trap leaves the whole instance in an unknown state, not just
// one session. The runtime refuses further use and must be loaded again.
const isTrap = error => typeof WebAssembly !== 'undefined' && error instanceof WebAssembly.RuntimeError;

function fatalFrom(error) {
  const text = messageOf(error);
  let report = null;
  try {
    const parsed = JSON.parse(text);
    if (parsed && parsed.schema === DISPATCH_SCHEMA && parsed.outcome === 'fatal') report = parsed;
  } catch { /* not a serialized report */ }
  return new CaveatError('fatal', report ? `${report.code}: ${report.message}` : text, report);
}

function isPlainObject(value) {
  if (value === null || typeof value !== 'object') return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

// Payloads must survive standard JSON unchanged apart from negative zero, which
// JSON writes as 0. Anything else (NaN, Infinity, undefined, functions, class
// instances, holes in arrays) is refused before the session is touched.
export function payloadText(payload) {
  const check = (value, at) => {
    if (value === null || typeof value === 'string' || typeof value === 'boolean') return;
    if (typeof value === 'number') {
      if (!Number.isFinite(value)) throw new CaveatError('payload', `payload ${at || 'value'} is not a finite number`);
      return;
    }
    if (Array.isArray(value)) {
      for (let index = 0; index < value.length; index++) {
        if (!Object.hasOwn(value, index)) throw new CaveatError('payload', `payload ${at}/${index} is a hole; JSON would send null`);
        check(value[index], `${at}/${index}`);
      }
      return;
    }
    if (isPlainObject(value)) { for (const [key, item] of Object.entries(value)) check(item, `${at}/${key}`); return; }
    throw new CaveatError('payload', `payload ${at || 'value'} cannot be sent as JSON`);
  };
  check(payload, '');
  return JSON.stringify(payload);
}

// accepted names what an accepted outcome carries: dispatch() the snapshot,
// dispatchView() the view, dispatchViewDelta() the delta.
// The view fields dispatchView can select, in the view's order.
export const VIEW_FIELDS = Object.freeze([
  'bindings', 'binding_explanations', 'cues', 'effects', 'commitments',
  'commitment_grounds', 'decision_series', 'decision_journal', 'relations',
]);

// dispatchView's field selection as the JSON text the runtime takes, or null
// for the whole view. Checked before the runtime is called.
function viewFields(options) {
  if (options === undefined || options === null) return null;
  if (typeof options !== 'object') throw new TypeError('dispatchView options must be an object');
  const { fields } = options;
  if (fields === undefined) return null;
  if (!Array.isArray(fields)) throw new TypeError('fields must be an array of view field names');
  const seen = new Set();
  for (const field of fields) {
    if (!VIEW_FIELDS.includes(field)) {
      throw new TypeError(`unknown view field ${JSON.stringify(field)}; the fields are ${VIEW_FIELDS.join(', ')}`);
    }
    if (seen.has(field)) throw new TypeError(`view field ${JSON.stringify(field)} is named twice`);
    seen.add(field);
  }
  return JSON.stringify(fields);
}

function validOutcome(outcome, accepted = 'snapshot') {
  if (!outcome || outcome.schema !== DISPATCH_SCHEMA) return false;
  if (outcome.outcome === 'accepted') return isPlainObject(outcome[accepted]);
  if (outcome.outcome === 'rejected') {
    return ORIGINS.includes(outcome.origin) && typeof outcome.code === 'string' && typeof outcome.message === 'string';
  }
  return false;
}

// An accepted dispatch outcome as the runtime writes it: these fields in this
// order, the snapshot last. The snapshot's text, or null for any other text,
// which is parsed and checked in full.
const ACCEPTED_PREFIX = `{"schema":"${DISPATCH_SCHEMA}","outcome":"accepted","snapshot":{`;
function acceptedSnapshotText(raw) {
  if (typeof raw !== 'string' || !raw.startsWith(ACCEPTED_PREFIX) || !raw.endsWith('}}')) return null;
  return raw.slice(ACCEPTED_PREFIX.length - 1, -1);
}

// Whether raw is a rejected outcome as the runtime writes it.
const REJECTED_PREFIX = `{"schema":"${DISPATCH_SCHEMA}","outcome":"rejected",`;
const rejectedOutcome = raw => typeof raw === 'string' && raw.startsWith(REJECTED_PREFIX);

// {schema, outcome: "accepted", snapshot}, the snapshot parsed when first
// read. Malformed text is fatal when it is read.
function lazyAccepted(text, malformed) {
  const outcome = { schema: DISPATCH_SCHEMA, outcome: 'accepted' };
  Object.defineProperty(outcome, 'snapshot', {
    enumerable: true,
    configurable: true,
    get() {
      let snapshot;
      try { snapshot = JSON.parse(text); } catch { throw malformed(); }
      if (!isPlainObject(snapshot)) throw malformed();
      Object.defineProperty(outcome, 'snapshot', { value: snapshot, enumerable: true, configurable: true, writable: true });
      return snapshot;
    },
    set(value) {
      Object.defineProperty(outcome, 'snapshot', { value, enumerable: true, configurable: true, writable: true });
    },
  });
  return outcome;
}

export class CaveatSession {
  #inner;
  #runtime;
  #state = 'open';
  // The snapshot text the last accepted dispatch() or snapshot() read, while
  // no call can have changed the session since.
  #lastSnapshot = null;

  constructor(inner, runtime) {
    this.#inner = inner;
    this.#runtime = runtime;
  }

  get state() { return this.#state; }

  // A trap in any session of this runtime makes every session unusable.
  #usable() {
    if (this.#state === 'open' && this.#runtime.trapped) this.#state = 'fatal';
    if (this.#state === 'open') return;
    if (this.#state === 'closed') throw new CaveatError('closed', 'session is closed');
    throw new CaveatError('fatal', this.#runtime.trapped ? 'the runtime instance trapped; load it again' : 'session is unusable after a fatal outcome');
  }

  #fail(error) {
    this.#state = 'fatal';
    if (isTrap(error)) {
      this.#runtime.markTrapped();
      return new CaveatError('fatal', this.#runtime.trapMessage(error));
    }
    return error instanceof CaveatError ? error : fatalFrom(error);
  }

  #call(read) {
    this.#usable();
    try { return read(); } catch (error) { throw this.#fail(error); }
  }

  // Runtime text that must be JSON. Malformed text is fatal like any other
  // runtime fault, and is checked inside the same guard.
  #json(read, what) {
    const text = this.#call(read);
    try {
      return { text, value: JSON.parse(text) };
    } catch {
      throw this.#fail(new CaveatError('fatal', `${what}() returned text that is not JSON`));
    }
  }

  // Returns {outcome: "accepted", snapshot} or {outcome: "rejected", origin,
  // code, message}. Throws CaveatError("fatal") for everything else. An
  // accepted snapshot is parsed when it is first read, and snapshot() reuses
  // it until the session changes rather than building it again.
  dispatch(event, payload = {}) {
    this.#usable();
    if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
    const text = payloadText(payload);
    let raw;
    try { raw = this.#inner.dispatch_outcome(event, text); } catch (error) { this.#lastSnapshot = null; throw this.#fail(error); }
    const accepted = acceptedSnapshotText(raw);
    if (accepted !== null) {
      this.#lastSnapshot = accepted;
      return lazyAccepted(accepted, () => this.#fail(new CaveatError('fatal', 'dispatch returned text that is not JSON')));
    }
    let outcome;
    // A rejected event leaves the session as it was, and its snapshot with it.
    if (!rejectedOutcome(raw)) this.#lastSnapshot = null;
    try { outcome = JSON.parse(raw); } catch { throw this.#fail(new CaveatError('fatal', 'dispatch returned text that is not JSON')); }
    if (!validOutcome(outcome)) {
      // Not a fatal report either: keep what arrived for diagnostics only.
      const error = new CaveatError('fatal', `unrecognised dispatch outcome: ${String(raw).slice(0, 200)}`);
      error.received = outcome;
      throw this.#fail(error);
    }
    return outcome;
  }

  // The same transaction, payload handling and failures as dispatch(), with the
  // view in place of the snapshot: {outcome: "accepted", view}, where view is
  // what view() returns after the event, or the rejected outcome dispatch()
  // returns. The runtime does not build the snapshot. A runtime build that
  // predates it throws CaveatError("load") and the session is untouched.
  //
  // dispatchView(event, payload, { fields }) gives a view with only the named
  // fields of VIEW_FIELDS, each exactly as view() has it, plus schema,
  // sequence and last_event. The runtime builds no other field. An unknown or
  // repeated name throws TypeError, and a runtime build without selection
  // throws CaveatError("load"); either leaves the session untouched.
  dispatchView(event, payload = {}, options = {}) {
    this.#usable();
    const fields = viewFields(options);
    if (typeof this.#inner.dispatch_view_outcome !== 'function') {
      throw new CaveatError('load', 'this runtime build has no dispatch_view_outcome; it predates dispatchView');
    }
    if (fields !== null && typeof this.#inner.dispatch_view_fields_outcome !== 'function') {
      throw new CaveatError('load', 'this runtime build has no dispatch_view_fields_outcome; it predates dispatchView fields');
    }
    if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
    const text = payloadText(payload);
    this.#lastSnapshot = null;
    let raw;
    try {
      raw = fields === null
        ? this.#inner.dispatch_view_outcome(event, text)
        : this.#inner.dispatch_view_fields_outcome(event, text, fields);
    } catch (error) { throw this.#fail(error); }
    let outcome;
    try { outcome = JSON.parse(raw); } catch { throw this.#fail(new CaveatError('fatal', 'dispatchView returned text that is not JSON')); }
    if (!validOutcome(outcome, 'view')) {
      // Not a fatal report either: keep what arrived for diagnostics only.
      const error = new CaveatError('fatal', `unrecognised dispatch outcome: ${String(raw).slice(0, 200)}`);
      error.received = outcome;
      throw this.#fail(error);
    }
    return outcome;
  }

  // What dispatchView(event, payload, options) would return, run on a copy of
  // the session that is then dropped: this session, its save, its snapshot
  // and its view are unchanged, whatever the outcome. A fatal outcome throws
  // CaveatError("fatal") but leaves this session usable, since the copy
  // failed and not it; a trap still makes the whole runtime unusable. A
  // runtime build that predates it throws CaveatError("load"). See
  // spec/caveat-dispatch-0.1.md, "The what-if path".
  whatIf(event, payload = {}, options = {}) {
    this.#usable();
    const fields = viewFields(options);
    if (typeof this.#inner.what_if_view_outcome !== 'function') {
      throw new CaveatError('load', 'this runtime build has no what_if_view_outcome; it predates whatIf');
    }
    if (fields !== null && typeof this.#inner.what_if_view_fields_outcome !== 'function') {
      throw new CaveatError('load', 'this runtime build has no what_if_view_fields_outcome; it predates whatIf fields');
    }
    if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
    const text = payloadText(payload);
    let raw;
    try {
      raw = fields === null
        ? this.#inner.what_if_view_outcome(event, text)
        : this.#inner.what_if_view_fields_outcome(event, text, fields);
    } catch (error) {
      if (isTrap(error)) throw this.#fail(error);
      throw error instanceof CaveatError ? error : fatalFrom(error);
    }
    let outcome;
    try { outcome = JSON.parse(raw); } catch { throw new CaveatError('fatal', 'whatIf returned text that is not JSON'); }
    if (!validOutcome(outcome, 'view')) {
      const error = new CaveatError('fatal', `unrecognised dispatch outcome: ${String(raw).slice(0, 200)}`);
      error.received = outcome;
      throw error;
    }
    return outcome;
  }

  snapshotText() { return this.#json(() => this.#inner.snapshot(), 'snapshot').text; }
  // A fresh object each call, from the snapshot text kept until the session
  // changes. Compact text, where the runtime has it, parses faster than the
  // pretty text snapshotText() returns.
  snapshot() {
    this.#usable();
    if (this.#lastSnapshot === null) {
      const pretty = typeof this.#inner.snapshot_compact !== 'function';
      this.#lastSnapshot = this.#call(() => (pretty ? this.#inner.snapshot() : this.#inner.snapshot_compact()));
    }
    try {
      return JSON.parse(this.#lastSnapshot);
    } catch {
      this.#lastSnapshot = null;
      throw this.#fail(new CaveatError('fatal', 'snapshot() returned text that is not JSON'));
    }
  }
  // view() is View 0.1; view({ schema: '0.2' }) is View 0.2
  // (spec/caveat-view-0.2.md). A runtime build that predates View 0.2 throws
  // CaveatError("load") for 0.2.
  viewText(options) { return this.#json(this.#viewReader(options), 'view').text; }
  view(options) { return this.#json(this.#viewReader(options), 'view').value; }

  // Checked before the runtime is called, so a wrong schema or an older
  // runtime leaves the session usable.
  #viewReader(options = {}) {
    this.#usable();
    const schema = options?.schema ?? '0.1';
    if (schema === '0.1') return () => this.#inner.view();
    if (schema !== '0.2') throw new TypeError("view schema must be '0.1' or '0.2'");
    if (typeof this.#inner.view_v2 !== 'function') {
      throw new CaveatError('load', 'this runtime build has no view_v2; it predates View 0.2');
    }
    return () => this.#inner.view_v2();
  }

  // The same transaction, payload handling and failures as dispatch(), with
  // the View 0.2 delta in place of the snapshot: {outcome: "accepted", delta},
  // which applied to view({ schema: '0.2' }) before the event gives the view
  // after it, or the rejected outcome dispatch() returns, with no delta. A
  // runtime build that predates it throws CaveatError("load") and the session
  // is untouched.
  dispatchViewDelta(event, payload = {}) {
    this.#usable();
    if (typeof this.#inner.dispatch_view_delta_outcome !== 'function') {
      throw new CaveatError('load', 'this runtime build has no dispatch_view_delta_outcome; it predates dispatchViewDelta');
    }
    if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
    const text = payloadText(payload);
    this.#lastSnapshot = null;
    let raw;
    try { raw = this.#inner.dispatch_view_delta_outcome(event, text); } catch (error) { throw this.#fail(error); }
    let outcome;
    try { outcome = JSON.parse(raw); } catch { throw this.#fail(new CaveatError('fatal', 'dispatchViewDelta returned text that is not JSON')); }
    if (!validOutcome(outcome, 'delta')) {
      // Not a fatal report either: keep what arrived for diagnostics only.
      const error = new CaveatError('fatal', `unrecognised dispatch outcome: ${String(raw).slice(0, 200)}`);
      error.received = outcome;
      throw this.#fail(error);
    }
    return outcome;
  }

  // The save is JSON text; restore it with the exact source it came from.
  // save({ compact: true }) is the replay save while the session's accepted
  // events fit in one and it is smaller, otherwise the same state save
  // (spec/caveat-replay-save-0.1.md). With positional: true as well it is the
  // positional replay save, which holds more events
  // (spec/caveat-replay-save-0.2.md). A runtime build that predates either
  // throws CaveatError("load") and the session is untouched.
  save(options = {}) {
    if (options?.compact !== true) return this.#json(() => this.#inner.save(), 'save').text;
    this.#usable();
    if (options.positional === true) {
      if (typeof this.#inner.save_compact_positional !== 'function') {
        throw new CaveatError('load', 'this runtime build has no save_compact_positional; it predates positional replay saves');
      }
      return this.#json(() => this.#inner.save_compact_positional(), 'save').text;
    }
    if (typeof this.#inner.save_compact !== 'function') {
      throw new CaveatError('load', 'this runtime build has no save_compact; it predates compact saves');
    }
    return this.#json(() => this.#inner.save_compact(), 'save').text;
  }

  // The archive items produced since the last drain, and an empty
  // archive (spec/caveat-departure-0.1.md). The archive is not saved: a host
  // that keeps exact history stores departed records and provenance nodes
  // together beside its saves. A runtime that predates departure departs
  // nothing, so its archive is empty.
  drainArchive() {
    this.#lastSnapshot = null;
    if (typeof this.#inner.drain_archive !== 'function') {
      this.#usable();
      return [];
    }
    const { value } = this.#json(() => this.#inner.drain_archive(), 'drainArchive');
    if (!Array.isArray(value)) throw this.#fail(new CaveatError('fatal', 'drainArchive() returned JSON that is not a list'));
    return value;
  }

  // How many archive items (records and provenance nodes) await draining.
  get undrained() {
    if (typeof this.#inner.undrained !== 'function') {
      this.#usable();
      return 0;
    }
    return this.#call(() => this.#inner.undrained());
  }

  // After a fatal outcome, or once the runtime has trapped, the session is
  // abandoned without calling into the runtime, including free(). A trap
  // during free() marks the runtime trapped for every other session.
  close() {
    this.#lastSnapshot = null;
    if (this.#state === 'open' && !this.#runtime.trapped) {
      this.#state = 'closed';
      this.#runtime.released(this);
      try { this.#inner.free(); } catch (error) { if (isTrap(error)) this.#runtime.markTrapped(); }
    }
    this.#state = 'closed';
  }
}

// Trap state belongs to a WebAssembly instance, not to a wrapper. Every
// runtime made from the same session class shares one instance and so one
// lifecycle: a trap seen through any of them stops all of them.
const lifecycles = new WeakMap();
function lifecycleOf(SessionClass) {
  let lifecycle = lifecycles.get(SessionClass);
  if (!lifecycle) {
    // open counts the sessions made through this instance and not yet closed
    // or collected: what a trap for lack of memory reports.
    lifecycle = { trapped: false, open: 0 };
    lifecycle.collected = typeof FinalizationRegistry === 'function'
      ? new FinalizationRegistry(() => { lifecycle.open -= 1; })
      : null;
    lifecycles.set(SessionClass, lifecycle);
  }
  return lifecycle;
}

// SessionClass is the runtime's WebReactiveSession, or a stand-in with the same
// methods: new SessionClass(source), SessionClass.restore(source, saved),
// dispatch_outcome, snapshot, view, save and free, and optionally
// dispatch_view_outcome (for dispatchView), view_v2 and
// dispatch_view_delta_outcome (for View 0.2), drain_archive and undrained (for
// drainArchive and undrained), save_compact (for save({ compact: true })),
// save_compact_positional (for save({ compact: true, positional: true })),
// snapshot_compact (a faster snapshot()), SessionClass.check(source),
// optionally SessionClass.check_with(source, unused) and
// SessionClass.check_options(source, optionsJson) (for check's options), and
// SessionClass.interface(source).
export function createRuntime(SessionClass, identity = {}) {
  const lifecycle = lifecycleOf(SessionClass);
  const refuse = () => new CaveatError('fatal', 'this runtime instance trapped; load a fresh one');
  // A session made through this runtime, counted until it is closed or
  // collected.
  const opened = inner => {
    const session = new CaveatSession(inner, runtime);
    lifecycle.open += 1;
    lifecycle.collected?.register(session, undefined, session);
    return session;
  };
  // The error for a call that trapped, kind as given unless it trapped.
  const failure = (error, kind) => {
    if (!isTrap(error)) return new CaveatError(kind, messageOf(error));
    lifecycle.trapped = true;
    return new CaveatError('fatal', runtime.trapMessage(error));
  };
  const runtime = {
    identity: Object.freeze({ ...identity }),
    get trapped() { return lifecycle.trapped; },
    markTrapped() { lifecycle.trapped = true; },
    // How many sessions made through this runtime instance are neither
    // closed nor collected.
    get openSessions() { return lifecycle.open; },
    released(session) {
      if (lifecycle.collected?.unregister(session) ?? true) lifecycle.open -= 1;
    },
    // What a trap means. A runtime that can say it ran out of memory
    // (out_of_memory) is named as the cause, with the sessions still open:
    // a host that never closes its sessions exhausts memory this way, and
    // the bare trap says only "unreachable" (Glowcap round 8, item 7).
    trapMessage(error) {
      if (typeof SessionClass.out_of_memory !== 'function') return messageOf(error);
      let outOfMemory = false;
      try { outOfMemory = SessionClass.out_of_memory() === true; } catch { /* the trap stands as reported */ }
      if (!outOfMemory) return messageOf(error);
      return `the runtime ran out of WebAssembly memory with ${lifecycle.open} session(s) open `
        + `(trap: ${messageOf(error)}). Close each session with close() once it is no longer needed. `
        + 'This runtime instance is now unusable; load it again.';
    },
    open(source) {
      if (lifecycle.trapped) throw refuse();
      if (typeof source !== 'string') throw new TypeError('source must be text');
      try { return opened(new SessionClass(source)); } catch (error) {
        throw failure(error, 'load');
      }
    },
    // Advisory warnings for a program that loads (spec/caveat-check-0.1.md).
    // A program that does not load throws as open does. `{ unused: true }`
    // adds C007 and `{ records: true }` C008, each reported only when asked
    // for. `{ records: true, save }` also adds C009 for that save (its text);
    // a save that does not restore throws as restore does.
    check(source, options = {}) {
      if (lifecycle.trapped) throw refuse();
      if (typeof source !== 'string') throw new TypeError('source must be text');
      if (typeof SessionClass.check !== 'function') {
        throw new CaveatError('load', 'this runtime build has no check; it predates caveat check');
      }
      const unused = options?.unused === true;
      const records = options?.records === true;
      const save = options?.save;
      if (save !== undefined && typeof save !== 'string') throw new TypeError('save must be text');
      if (save !== undefined && !records) throw new TypeError('a save is checked with records: true');
      if (unused && typeof SessionClass.check_with !== 'function') {
        throw new CaveatError('load', 'this runtime build has no unused-declaration check; it predates caveat check --unused');
      }
      if (records && typeof SessionClass.check_options !== 'function') {
        throw new CaveatError('load', 'this runtime build has no records check; it predates caveat check --records');
      }
      if (save !== undefined) {
        // The program must load first, so its error is reported as load.
        try { new SessionClass(source).free(); } catch (error) { throw failure(error, 'load'); }
        try { SessionClass.restore(source, save).free(); } catch (error) { throw failure(error, 'restore'); }
      }
      try {
        if (records) return JSON.parse(SessionClass.check_options(source, JSON.stringify({ unused, records, save })));
        return JSON.parse(unused ? SessionClass.check_with(source, true) : SessionClass.check(source));
      } catch (error) {
        throw failure(error, 'load');
      }
    },
    // The program's events, states, bindings and declared names
    // (spec/caveat-interface-0.1.md). A program that does not load throws as
    // open does.
    interface(source) {
      if (lifecycle.trapped) throw refuse();
      if (typeof source !== 'string') throw new TypeError('source must be text');
      if (typeof SessionClass.interface !== 'function') {
        throw new CaveatError('load', 'this runtime build has no interface; it predates caveat types');
      }
      try { return JSON.parse(SessionClass.interface(source)); } catch (error) {
        throw failure(error, 'load');
      }
    },
    restore(source, saved) {
      if (lifecycle.trapped) throw refuse();
      if (typeof source !== 'string' || typeof saved !== 'string') throw new TypeError('source and save must be text');
      try { return opened(SessionClass.restore(source, saved)); } catch (error) {
        throw failure(error, 'restore');
      }
    },
  };
  return runtime;
}

let freshInstances = 0;

// module: a URL for caveat_runtime.js (relative URLs resolve against this file,
// as import() does), or its imported namespace. wasm: whatever the runtime's
// init accepts (bytes, a URL or a Response).
//
// Loading the same URL twice gives wrappers over the same instance. If that
// instance has trapped, a URL is imported again under a unique query, which
// gives a separate module with its own memory, so recovery never reuses the
// trapped instance. A namespace cannot be re-imported, so it is refused.
//
// A URL is only known at run time, so a bundler cannot follow it. The
// comments tell webpack and Vite to leave both imports to the platform's own
// import() instead of warning about an expression they cannot resolve; a
// bundled host passes the namespace it imported itself.
export async function loadRuntime({ module, wasm, identity }) {
  const url = typeof module === 'string' || module instanceof URL ? new URL(String(module), import.meta.url) : null;
  let namespace = url ? await import(/* webpackIgnore: true */ /* @vite-ignore */ url.href) : module;
  if (lifecycles.get(namespace.WebReactiveSession)?.trapped) {
    if (!url) throw new CaveatError('fatal', 'this runtime instance trapped; pass its module URL to load a fresh one');
    const fresh = new URL(url.href);
    freshInstances += 1;
    fresh.searchParams.set('caveat-instance', String(freshInstances));
    namespace = await import(/* webpackIgnore: true */ /* @vite-ignore */ fresh.href);
  }
  await namespace.default({ module_or_path: wasm });
  if (typeof namespace.WebReactiveSession?.prototype?.dispatch_outcome !== 'function') {
    throw new CaveatError('load', 'this runtime build has no dispatch_outcome; it predates the dispatch outcome contract');
  }
  return createRuntime(namespace.WebReactiveSession, identity);
}

// One journal change, copied so the caller cannot alter the view.
function journalChange(entry) {
  const change = { sequence: entry.sequence, event: entry.event, because: [...entry.because], caveats: [...entry.caveats] };
  if (entry.value !== undefined) change.value = entry.value;
  if (entry.permitted_by !== undefined) change.permitted_by = entry.permitted_by;
  return change;
}

// Every revision of one decision series, oldest first, read from a view or
// snapshot: what each was committed on and, once reopened, what reopened it.
// The current revision is in force or reopened, as its commitment says; every
// earlier one is superseded. The series lists every revision; a windowed
// journal may have retired a revision's entries, and that revision is then
// listed with `retired: true` and null for each change the journal no longer
// holds, rather than left out.
export function decisionHistory(view, decision) {
  const series = view.decision_series?.[decision];
  if (!series) throw new TypeError(`${decision} is not a decision series of this program`);
  const revisions = new Map(series.revisions.map(item => [item.id, { revision: item.id, status: 'superseded', retired: true, committed: null, reopened: null }]));
  for (const entry of view.decision_journal) {
    const revision = entry.decision === decision ? revisions.get(entry.commitment) : undefined;
    if (!revision) continue;
    if (entry.change === 'committed') {
      revision.committed = journalChange(entry);
      revision.retired = false;
    } else if (entry.change === 'reopened') {
      // A hold's `held`, `deferred` and `released` entries reopen nothing.
      revision.reopened = journalChange(entry);
    }
  }
  const current = series.current === null ? undefined : revisions.get(series.current);
  if (current) {
    const commitment = view.commitments.find(item => item.action === series.current);
    // View 0.1 and the snapshot say `open`; View 0.2 says `reopened`.
    const reopened = commitment ? commitment.open ?? commitment.reopened : true;
    current.status = reopened === false ? 'in_force' : 'reopened';
  }
  return [...revisions.values()];
}
