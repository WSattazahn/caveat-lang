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
// instances) is refused before the session is touched.
export function payloadText(payload) {
  const check = (value, at) => {
    if (value === null || typeof value === 'string' || typeof value === 'boolean') return;
    if (typeof value === 'number') {
      if (!Number.isFinite(value)) throw new CaveatError('payload', `payload ${at || 'value'} is not a finite number`);
      return;
    }
    if (Array.isArray(value)) { value.forEach((item, index) => check(item, `${at}/${index}`)); return; }
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

  #usable() {
    if (this.#state === 'open') return;
    throw new CaveatError(this.#state === 'closed' ? 'closed' : 'fatal', `session is ${this.#state === 'closed' ? 'closed' : 'unusable after a fatal outcome'}`);
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

  snapshotText() { return this.#call(() => this.#inner.snapshot()); }
  snapshot() { return JSON.parse(this.snapshotText()); }
  viewText() { return this.#call(() => this.#inner.view()); }
  view() { return JSON.parse(this.viewText()); }

  // The save is text; restore it with the exact source it came from.
  save() { return this.#call(() => this.#inner.save()); }

  // After a fatal outcome the session is abandoned without further calls into
  // the runtime, including free().
  close() {
    if (this.#state === 'open') {
      this.#state = 'closed';
      try { this.#inner.free(); } catch { /* already released */ }
    } else if (this.#state === 'fatal') {
      this.#state = 'closed';
    }
  }
}

// SessionClass is the runtime's WebReactiveSession, or a stand-in with the same
// methods: new SessionClass(source), SessionClass.restore(source, saved),
// dispatch_outcome, snapshot, view, save and free.
export function createRuntime(SessionClass, identity = {}) {
  let trapped = false;
  const runtime = {
    identity: Object.freeze({ ...identity }),
    get trapped() { return trapped; },
    markTrapped() { trapped = true; },
    open(source) {
      if (trapped) throw new CaveatError('fatal', 'runtime trapped earlier; load it again');
      if (typeof source !== 'string') throw new TypeError('source must be text');
      try { return new CaveatSession(new SessionClass(source), runtime); } catch (error) {
        if (isTrap(error)) trapped = true;
        throw new CaveatError(isTrap(error) ? 'fatal' : 'load', messageOf(error));
      }
    },
    restore(source, saved) {
      if (trapped) throw new CaveatError('fatal', 'runtime trapped earlier; load it again');
      if (typeof source !== 'string' || typeof saved !== 'string') throw new TypeError('source and save must be text');
      try { return new CaveatSession(SessionClass.restore(source, saved), runtime); } catch (error) {
        if (isTrap(error)) trapped = true;
        throw new CaveatError(isTrap(error) ? 'fatal' : 'restore', messageOf(error));
      }
    },
  };
  return runtime;
}

// module: a URL or specifier for caveat_runtime.js, or its imported namespace.
// wasm: whatever the runtime's init accepts (bytes, a URL or a Response).
export async function loadRuntime({ module, wasm, identity }) {
  const namespace = typeof module === 'string' || module instanceof URL ? await import(String(module)) : module;
  await namespace.default({ module_or_path: wasm });
  if (typeof namespace.WebReactiveSession?.prototype?.dispatch_outcome !== 'function') {
    throw new CaveatError('load', 'this runtime build has no dispatch_outcome; it predates the dispatch outcome contract');
  }
  return createRuntime(namespace.WebReactiveSession, identity);
}
