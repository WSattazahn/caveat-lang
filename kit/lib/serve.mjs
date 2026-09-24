// One program's session behind a line protocol, so that a hook, a harness or
// another language can send events and ask questions without JavaScript. Each
// request is one JSON object on a line; each response is one JSON object on a
// line. See spec/caveat-serve-0.1.md. This module does no I/O: the caller feeds
// it lines and writes what it returns.
import { dependents, explain } from './explain.mjs';
import { CaveatError } from './session.mjs';

export const SERVE_SCHEMA = 'caveat-serve/0.1';

const FIELDS = {
  dispatch: ['event', 'payload', 'snapshot'],
  snapshot: [],
  explain: [],
  dependents: ['of'],
  save: [],
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
 * when the program does not load. `ready` is the first line to write.
 * `handle(line)` returns `{response, exit}`: write the response, then stop with
 * `exit` as the status when it is a number.
 */
export function createServer({ runtime, source, program = null }) {
  let session = runtime.open(source);
  // The events since the session opened or was restored, for explain.
  let events = [];

  const ready = { schema: SERVE_SCHEMA, ready: true, program, runtime: runtime.identity ?? {} };

  function run({ op, ...request }) {
    if (op === 'dispatch') {
      if (typeof request.event !== 'string' || !request.event) throw new RequestError('dispatch needs an "event" name');
      if (request.payload !== undefined && !isObject(request.payload)) throw new RequestError('"payload" must be an object');
      if (request.snapshot !== undefined && typeof request.snapshot !== 'boolean') throw new RequestError('"snapshot" must be true or false');
      const payload = request.payload ?? {};
      let result;
      try { result = session.dispatch(request.event, payload); } catch (error) {
        if (error instanceof CaveatError && error.kind === 'payload') throw new RequestError(error.message);
        throw error;
      }
      const { schema: _schema, snapshot, ...outcome } = result;
      events.push({ event: request.event, payload, outcome });
      const after = snapshot ?? session.snapshot();
      return { ...outcome, sequence: after.sequence, ...(request.snapshot ? { snapshot: after } : {}) };
    }
    if (op === 'snapshot') return { snapshot: session.snapshot() };
    if (op === 'explain') return { report: explain(session.snapshot(), events) };
    if (op === 'dependents') {
      if (typeof request.of !== 'string' || !request.of) throw new RequestError('dependents needs "of", a name');
      try { return { report: dependents(session.snapshot(), request.of) }; } catch (error) {
        if (error instanceof CaveatError) throw error;
        throw new RequestError(error.message);
      }
    }
    if (op === 'save') return { save: session.save() };
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
      return { response: { id, ok: true, ...result }, exit: request.op === 'close' ? 0 : null };
    } catch (error) {
      if (error instanceof RequestError) {
        return { response: { id: error.id ?? id, ok: false, error: { kind: 'request', message: error.message } }, exit: null };
      }
      // A fatal outcome makes the session unusable; the host must start again.
      const kind = error instanceof CaveatError ? error.kind : 'fatal';
      return { response: { id, ok: false, error: { kind, message: error.message } }, exit: 1 };
    }
  }

  return { ready, handle, close: () => session.close() };
}
