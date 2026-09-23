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

function validOutcome(outcome) {
  if (!outcome || outcome.schema !== DISPATCH_SCHEMA) return false;
  if (outcome.outcome === 'accepted') return isPlainObject(outcome.snapshot);
  if (outcome.outcome === 'rejected') {
    return ORIGINS.includes(outcome.origin) && typeof outcome.code === 'string' && typeof outcome.message === 'string';
  }
  return false;
}

export class CaveatSession {
  #inner;
  #runtime;
  #state = 'open';

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
    if (isTrap(error)) this.#runtime.markTrapped();
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
  // code, message}. Throws CaveatError("fatal") for everything else.
  dispatch(event, payload = {}) {
    this.#usable();
    if (typeof event !== 'string' || !event) throw new TypeError('event must be a non-empty string');
    const text = payloadText(payload);
    let raw;
    try { raw = this.#inner.dispatch_outcome(event, text); } catch (error) { throw this.#fail(error); }
    let outcome;
    try { outcome = JSON.parse(raw); } catch { throw this.#fail(new CaveatError('fatal', 'dispatch returned text that is not JSON')); }
    if (!validOutcome(outcome)) {
      // Not a fatal report either: keep what arrived for diagnostics only.
      const error = new CaveatError('fatal', `unrecognised dispatch outcome: ${String(raw).slice(0, 200)}`);
      error.received = outcome;
      throw this.#fail(error);
    }
    return outcome;
  }

  snapshotText() { return this.#json(() => this.#inner.snapshot(), 'snapshot').text; }
  snapshot() { return this.#json(() => this.#inner.snapshot(), 'snapshot').value; }
  viewText() { return this.#json(() => this.#inner.view(), 'view').text; }
  view() { return this.#json(() => this.#inner.view(), 'view').value; }

  // The save is JSON text; restore it with the exact source it came from.
  save() { return this.#json(() => this.#inner.save(), 'save').text; }

  // After a fatal outcome, or once the runtime has trapped, the session is
  // abandoned without calling into the runtime, including free(). A trap
  // during free() marks the runtime trapped for every other session.
  close() {
    if (this.#state === 'open' && !this.#runtime.trapped) {
      this.#state = 'closed';
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
    lifecycle = { trapped: false };
    lifecycles.set(SessionClass, lifecycle);
  }
  return lifecycle;
}

// SessionClass is the runtime's WebReactiveSession, or a stand-in with the same
// methods: new SessionClass(source), SessionClass.restore(source, saved),
// dispatch_outcome, snapshot, view, save and free.
export function createRuntime(SessionClass, identity = {}) {
  const lifecycle = lifecycleOf(SessionClass);
  const refuse = () => new CaveatError('fatal', 'this runtime instance trapped; load a fresh one');
  const runtime = {
    identity: Object.freeze({ ...identity }),
    get trapped() { return lifecycle.trapped; },
    markTrapped() { lifecycle.trapped = true; },
    open(source) {
      if (lifecycle.trapped) throw refuse();
      if (typeof source !== 'string') throw new TypeError('source must be text');
      try { return new CaveatSession(new SessionClass(source), runtime); } catch (error) {
        if (isTrap(error)) lifecycle.trapped = true;
        throw new CaveatError(isTrap(error) ? 'fatal' : 'load', messageOf(error));
      }
    },
    restore(source, saved) {
      if (lifecycle.trapped) throw refuse();
      if (typeof source !== 'string' || typeof saved !== 'string') throw new TypeError('source and save must be text');
      try { return new CaveatSession(SessionClass.restore(source, saved), runtime); } catch (error) {
        if (isTrap(error)) lifecycle.trapped = true;
        throw new CaveatError(isTrap(error) ? 'fatal' : 'restore', messageOf(error));
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
export async function loadRuntime({ module, wasm, identity }) {
  const url = typeof module === 'string' || module instanceof URL ? new URL(String(module), import.meta.url) : null;
  let namespace = url ? await import(url.href) : module;
  if (lifecycles.get(namespace.WebReactiveSession)?.trapped) {
    if (!url) throw new CaveatError('fatal', 'this runtime instance trapped; pass its module URL to load a fresh one');
    const fresh = new URL(url.href);
    freshInstances += 1;
    fresh.searchParams.set('caveat-instance', String(freshInstances));
    namespace = await import(fresh.href);
  }
  await namespace.default({ module_or_path: wasm });
  if (typeof namespace.WebReactiveSession?.prototype?.dispatch_outcome !== 'function') {
    throw new CaveatError('load', 'this runtime build has no dispatch_outcome; it predates the dispatch outcome contract');
  }
  return createRuntime(namespace.WebReactiveSession, identity);
}
