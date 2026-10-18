// One program's session behind a line protocol, so that a hook, a harness or
// another language can send events and ask questions without JavaScript. Each
// request is one JSON object on a line; each response is one JSON object on a
// line. See spec/caveat-serve-0.1.md. This module does no I/O: the caller feeds
// it lines and writes what it returns.
import { dependents, explain } from './explain.mjs';
import { CaveatError } from './session.mjs';

export const SERVE_SCHEMA = 'caveat-serve/0.1';

const FIELDS = {
  dispatch: ['event', 'payload', 'snapshot', 'view'],
  snapshot: [],
  view: [],
  drainArchive: [],
  undrained: [],
  explain: ['archive'],
  dependents: ['of', 'archive'],
  save: ['compact'],
  restore: ['save'],
  close: [],
};

class RequestError extends Error {}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function parseRequest(line) {
  let request;
  try { request = JSON.parse(line); } catch (error) { throw new RequestError(`not JSON: ${error.message}`); }
  if (!isObject(request)) throw new RequestError('a request must be a JSON object');
  const { id = null, op, ...rest } = request;
  if (typeof op !== 'string' || !Object.hasOwn(FIELDS, op)) {
    throw Object.assign(new RequestError(`"op" must be one of ${Object.keys(FIELDS).join(', ')}`), { id });
  }
  const extra = Object.keys(rest).find(key => !FIELDS[op].includes(key));
  if (extra) throw Object.assign(new RequestError(`${op} has no field ${extra}`), { id });
  return { id, op, ...rest };
}

/**
 * A server for `source` on `runtime` (from loadRuntime or
 * loadRuntimeFromDirectory). Opening the session throws CaveatError("load")
 * when the program does not load; with `restore`, the text of an earlier save,
 * the session starts from that save instead, and a save the runtime cannot use
 * throws CaveatError("restore"). `ready` is the first line to write.
 * `handle(line)` returns `{response, exit, changed}`: write the response, then
 * stop with `exit` as the status when it is a number. `changed` is true when
 * the request changed the session (an accepted event, or a restore), which is
 * when a host that keeps a state file rewrites it; `save()` returns that
 * file's text, compact where the runtime supports it, with `compact` saying
 * which it is.
 */
export function createServer({ runtime, source, program = null, restore = null }) {
  let session = restore === null ? runtime.open(source) : runtime.restore(source, restore);
  // The events since the session opened or was restored, for explain.
  let events = [];
  // Whether the runtime has a compact save: unknown until the first save asks.
  let compact = null;

  const ready = { schema: SERVE_SCHEMA, ready: true, program, runtime: runtime.identity ?? {} };

  function save() {
    if (compact !== false) {
      try {
        const text = session.save({ compact: true });
        compact = true;
        return { text, compact };
      } catch (error) {
        // A runtime from before compact saves says so on load; anything else is real.
        if (!(error instanceof CaveatError && error.kind === 'load')) throw error;
        compact = false;
      }
    }
    return { text: session.save(), compact };
  }

  function run({ op, ...request }) {
    if (op === 'dispatch') {
      if (typeof request.event !== 'string' || !request.event) throw new RequestError('dispatch needs an "event" name');
      if (request.payload !== undefined && !isObject(request.payload)) throw new RequestError('"payload" must be an object');
      if (request.snapshot !== undefined && typeof request.snapshot !== 'boolean') throw new RequestError('"snapshot" must be true or false');
      if (request.view !== undefined && typeof request.view !== 'boolean') throw new RequestError('"view" must be true or false');
      const payload = request.payload ?? {};
      let result;
      try { result = session.dispatch(request.event, payload); } catch (error) {
        if (error instanceof CaveatError && error.kind === 'payload') throw new RequestError(error.message);
        throw error;
      }
      const { schema: _schema, snapshot, ...outcome } = result;
      const after = snapshot ?? session.snapshot();
      events.push({ event: request.event, payload, outcome,
        ...(outcome.outcome === 'accepted' ? { sequence: after.sequence, effects: after.effects } : {}) });
      return {
        ...outcome,
        sequence: after.sequence,
        ...(request.snapshot ? { snapshot: after } : {}),
        ...(request.view ? { view: session.view() } : {}),
      };
    }
    if (op === 'snapshot') return { snapshot: session.snapshot() };
    if (op === 'view') return { view: session.view() };
    if (op === 'drainArchive') return { archive: session.drainArchive() };
    if (op === 'undrained') return { undrained: session.undrained };
    if ((op === 'explain' || op === 'dependents') && request.archive !== undefined
      && (!Array.isArray(request.archive) || !request.archive.every(isObject))) {
      throw new RequestError('"archive" must be an array of archive item objects');
    }
    if (op === 'explain') return { report: explain(session.snapshot(), events, { archive: request.archive }) };
    if (op === 'dependents') {
      if (typeof request.of !== 'string' || !request.of) throw new RequestError('dependents needs "of", a name');
      try { return { report: dependents(session.snapshot(), request.of, { archive: request.archive }) }; } catch (error) {
        if (error instanceof CaveatError) throw error;
        throw new RequestError(error.message);
      }
    }
    if (op === 'save') {
      if (request.compact !== undefined && typeof request.compact !== 'boolean') throw new RequestError('"compact" must be true or false');
      return { save: session.save({ compact: request.compact === true }) };
    }
    if (op === 'restore') {
      if (typeof request.save !== 'string') throw new RequestError('restore needs "save", the text a save returned');
      let restored;
      try { restored = runtime.restore(source, request.save); } catch (error) {
        if (error instanceof CaveatError && error.kind === 'restore') throw new RequestError(error.message);
        throw error;
      }
      session.close();
      session = restored;
      events = [];
      return { sequence: session.snapshot().sequence };
    }
    session.close();
    return {};
  }

  function handle(line) {
    let id = null;
    try {
      const request = parseRequest(line);
      id = request.id;
      const result = run(request);
      const changed = request.op === 'restore' || (request.op === 'dispatch' && result.outcome === 'accepted');
      return { response: { id, ok: true, ...result }, exit: request.op === 'close' ? 0 : null, changed };
    } catch (error) {
      if (error instanceof RequestError) {
        return { response: { id: error.id ?? id, ok: false, error: { kind: 'request', message: error.message } }, exit: null, changed: false };
      }
      // A fatal outcome makes the session unusable; the host must start again.
      const kind = error instanceof CaveatError ? error.kind : 'fatal';
      return { response: { id, ok: false, error: { kind, message: error.message } }, exit: 1, changed: false };
    }
  }

  return { ready, handle, save, close: () => session.close() };
}
