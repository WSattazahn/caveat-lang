// Private stdio MCP adapter. Every tool owns one disposable subprocess/runtime.
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { TextDecoder } from 'node:util';
import { defaultRuntimeDirectory } from './node.mjs';
import { validateAuthoringArguments } from './authoring.mjs';

// 2025-11-25 is negotiated by initialize; 2026-07-28 requests carry their
// version and client capabilities in _meta and need no prior handshake.
export const MCP_PROTOCOL_VERSION = '2025-11-25';
export const MCP_STATELESS_PROTOCOL_VERSION = '2026-07-28';
export const MCP_SUPPORTED_VERSIONS = Object.freeze([MCP_STATELESS_PROTOCOL_VERSION, MCP_PROTOCOL_VERSION]);
const META_VERSION = 'io.modelcontextprotocol/protocolVersion';
const META_CAPABILITIES = 'io.modelcontextprotocol/clientCapabilities';
const META_CLIENT = 'io.modelcontextprotocol/clientInfo';
const META_SERVER = 'io.modelcontextprotocol/serverInfo';
const UNSUPPORTED_PROTOCOL_VERSION = -32022;
// The tool list is fixed for an installed package; restarting the server is
// the only way it changes, and it holds no user-specific data.
const TOOLS_TTL_MS = 3600000;
const INSTRUCTIONS = 'Inline authoring operations only. Evidence and reports are supplied data; no file access or external permission is implied.';
const MESSAGE_BYTES = 4 * 1024 * 1024;
const STDERR_BYTES = 64 * 1024;
const MAX_DEPTH = 64;
const MAX_IDS = 4096;
const WORKER = fileURLToPath(new URL('./authoring-worker.mjs', import.meta.url));
const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const idValid = id => typeof id === 'string' || Number.isSafeInteger(id);
const keyOf = id => createHash('sha256').update(JSON.stringify(id)).digest('hex');
const sourceSchema = {
  type: 'string', maxLength: 1024 * 1024,
  description: 'Complete inline Caveat program text, available as the virtual file inline.cav. No file path or URL lookup. At most 1 MiB of UTF-8 source; each call uses a fresh runtime.',
};
const eventSchema = {
  type: 'object', required: ['event'], additionalProperties: false,
  description: 'One event to dispatch to the supplied program, for example {"event":"read","payload":{"value":85}} when source declares read with a value parameter.',
  properties: {
    event: { type: 'string', minLength: 1, description: 'Event name declared in source, such as read or assess. Supply the name only, without Caveat syntax.' },
    payload: { type: 'object', description: 'Values keyed by the event parameter names declared in source, for example {"value":85}. Omit for an empty payload ({}). The runtime checks parameter names, types and declared bounds.' },
  },
};
const eventsSchema = {
  type: 'array', maxItems: 1000, items: eventSchema,
  description: 'Events replayed in array order from a fresh session. Omitted or [] inspects the initial state; previous calls supply no history. Rejections are recorded and replay continues; a fatal event stops replay at the last good snapshot. At most 1,000 entries and 1 MiB of serialized UTF-8 JSON.',
};
const resultDescription = 'Returns the same JSON in structuredContent and text: {schema:"caveat-authoring/0.1",operation,sourceSha256,runtime,exitCode,report}. Read exitCode even when isError is false. Bridge failures instead set isError:true with {schema:"caveat-authoring-error/0.1",error:{kind,message}}.';
const scenariosSchema = {
  type: 'object',
  description: 'Scenario document object, not JSON text. Example: {"schema":"caveat-scenarios/0.1","source":"inline.cav","scenarios":[{"id":"initial","title":"Initial sequence","steps":[{"expect":{"/sequence":0}}]}]}. Each scenario needs a unique nonempty id, a title and nonempty steps. Every source reference, including optional per-scenario source, must be inline.cav. Each step has exactly one operation: send (event name; optional payload, rejected, repeat), expect (JSON Pointer-to-value map into the snapshot; a binding declared as `bind install.state` is at /bindings/install/state), checkpoint (state name), same_as (stored state name, with paths), resume:true, or size (save/snapshot byte bounds). Sends expect acceptance unless rejected specifies a policy message, true for any policy rejection, or an object with origin and optional code/message. Object expectations match listed members; arrays match in order; $exact, $set, $includes and $absent matchers are supported. Limits: 1 MiB serialized UTF-8 JSON, 32 scenarios, 2,000 total steps and 1,000 sends after repeat expansion.',
};
const tool = (name, description, properties, required = ['source']) => ({
  name, description: `${description} ${resultDescription}`,
  inputSchema: { type: 'object', additionalProperties: false, properties: { source: sourceSchema, ...properties }, required },
  annotations: { readOnlyHint: true, destructiveHint: false, idempotentHint: true, openWorldHint: false },
});
export const MCP_TOOLS = Object.freeze([
  tool('caveat_validate', 'Load inline source with the installed runtime and list its declarations. Start here to find load errors; use caveat_check for authoring diagnostics and caveat_test to verify expected behavior. No events are dispatched. report (caveat-validate/0.1) contains program, loads, events, reading_streams, decision_series and displayed bindings, or a load error. exitCode: 0 loaded; 2 load failure.', {}),
  tool('caveat_check', 'Load a single-file program and report authoring diagnostics, such as repeated rules or a missing reopening path. Use after editing source; choose caveat_validate for a declaration summary and caveat_test for behavior assertions. A clean check does not prove behavior; bundles are unsupported. report (caveat-check/0.1) contains loads, strict, diagnostics and suppressed diagnostics, with code, location, message and suggestion, or a load error. exitCode: 0 completed without a strict failure; 1 strict with unsuppressed diagnostics; 2 load failure or bundle.', {
    strict: { type: 'boolean', description: 'Optional, defaults to false. When true, any unsuppressed diagnostic makes exitCode 1 instead of 0; suppressed diagnostics do not count. Does not change program behavior. Load failures and unsupported bundles remain exitCode 2.' },
  }),
  tool('caveat_test', 'Run scenario assertions and runtime invariant checks against inline source, with a fresh session per scenario. Use to verify expected event outcomes and snapshot values; caveat_validate only checks loading, caveat_check reports diagnostics, and caveat_explain inspects history without assertions. report (caveat-scenario-report/0.1) includes files, per-scenario results/failures, check diagnostics and passed/failed totals. exitCode: 0 all passed; 1 scenario failure; 2 invalid scenario document (report has valid:false and error) or blocking authoring diagnostics. Every source reference must be inline.cav.', { scenarios: scenariosSchema }, ['source', 'scenarios']),
  tool('caveat_explain', 'Replay optional events and explain the resulting decisions, evidence and displayed values. Use to answer why a decision was made; choose caveat_dependents to trace one evidence or caveat name forward, and caveat_test for assertions. report (caveat-explain/0.1) includes sequence, event outcomes, decisions with grounds and revision history, evidence with caveats, displayed values and available archive history. exitCode: 0 replay completed (may include rejected events); 1 fatal event with last good state; 2 source load failure (report has loads:false and error).', { events: eventsSchema }),
  tool('caveat_dependents', 'Replay optional events and find what rests on one named evidence item, reading stream or caveat. Use for impact inspection; caveat_explain instead explains all decisions and their reasons. Subjects are causes only: evidence, a reading stream or a caveat. Claims and decisions are never subjects, because a claim causes nothing and the grounds of a decision are evidence, not claims; both appear in the report as what the subject reaches. To trace from a claim, query the evidence that supports or opposes it (report.claims lists those relations). report (caveat-dependents/0.1) includes subject, kind, sequence, event outcomes, claims, decisions, changes, values, displayed values and withdrawal records; dependency entries distinguish grounds from possible influence through lineage. exitCode: 0 replay completed (may include rejected events); 1 fatal event with last good state; 2 load failure or unknown subject, with report.error.', {
    subject: { type: 'string', minLength: 1, description: 'Exact case-sensitive evidence name, reading stream name, caveat name, or evidence occurrence ID after replay. Evidence records supplied observations; a reading stream is a named history of samples; a caveat is a qualification on evidence. For example, probe selects its evidence/readings, values selects a stream, and values@1 selects one recorded reading. Use a name from source or caveat_explain; not a file path. A claim or decision name is refused like an unknown name, with exitCode 2.' },
    events: eventsSchema,
  }, ['source', 'subject']),
  tool('caveat_dispatch', 'Keep one running ledger across calls: send events to the program, starting fresh or from the save an earlier caveat_dispatch call returned, and get the new save back. Use when a task records decisions as it goes; pass report.save unchanged as save in the next call, with the same source, and send only the new events. The server keeps nothing between calls. caveat_explain instead replays a whole history from the start each time. report (caveat-dispatch-step/0.1) is the caveat-explain/0.1 report of the state after these events (its events list only this call\'s events), plus restored and save. exitCode: 0 completed (may include rejected events); 1 fatal event, with the explanation and save of the last good state; 2 source load failure, or a save that does not restore with this exact source (report has error).', {
      save: { type: 'string', maxLength: 2 * 1024 * 1024, description: 'Optional. The report.save text from the previous caveat_dispatch call, passed back unchanged. Omit to start a fresh session. A save restores only with the exact source it came from; it is not repaired or checked for history. At most 2 MiB of UTF-8.' },
      events: { ...eventsSchema, description: 'Events to send in array order after the save is restored, or from a fresh session. Omitted or [] reports the current state and returns an equivalent save. Rejections are recorded and the step continues; a fatal event stops it at the last good state. At most 1,000 entries and 1 MiB of serialized UTF-8 JSON.' },
  }),
  tool('caveat_find', 'Search past decision records of one program for decisions already made: has this been decided before, on what grounds, and does it still stand? Each record is the events of one past run, replayed in a fresh session. A decision revision is found when every word of query occurs, ignoring case, in its decision or revision name, the record name, the names, sources and claims of the evidence it rests on, its caveats, or its retention reasons. Use caveat_explain to see one record in full, and caveat_dependents to trace one evidence forward. report (caveat-find/0.1) includes words, searched (per record: revisions and any fatal event) and found (per revision: record, value, status, grounds with sources, retained caveats, withdrawn evidence, history, outcome and where each word matched). exitCode: 0 searched, even when nothing matched; 1 an event failed fatally in a record (its decisions are of the session before it); 2 source load failure.', {
    records: {
      type: 'array', minItems: 1, maxItems: 64,
      description: 'Past records of the program, each {"name":"ticket-a","events":[...]}: a unique non-empty name the report uses, and the events of that run in order, as for caveat_explain. At most 64 records and 1 MiB of serialized UTF-8 JSON in all; each record at most 1,000 events.',
      items: {
        type: 'object', required: ['name', 'events'], additionalProperties: false,
        properties: { name: { type: 'string', minLength: 1, description: 'Unique label for this record, such as a ticket or task id. Query words also match it.' }, events: eventsSchema },
      },
    },
    query: { type: 'string', minLength: 1, description: 'Words to find, separated by spaces, such as "refund email". Every word must match; case is ignored; a word matches part of a name, so refund matches refund_warranted.' },
  }, ['source', 'records', 'query']),
]);
const toolNames = new Set(MCP_TOOLS.map(item => item.name));

// Check lexical nesting before JSON.parse; strings may themselves contain JSON.
function withinDepth(text) {
  let depth = 0, quoted = false, escaped = false;
  for (const character of text) {
    if (quoted) {
      if (escaped) escaped = false;
      else if (character === '\\') escaped = true;
      else if (character === '"') quoted = false;
    } else if (character === '"') quoted = true;
    else if (character === '{' || character === '[') { if (++depth > MAX_DEPTH) return false; }
    else if (character === '}' || character === ']') depth--;
  }
  return true;
}
const hasOnly = (value, allowed) => object(value) && Object.keys(value).every(key => allowed.includes(key));
const paramsEmpty = value => value === undefined || hasOnly(value, ['_meta']);
const implementation = value => object(value) && typeof value.name === 'string' && typeof value.version === 'string';
const bridgeError = (kind, message, details = {}) => ({ schema: 'caveat-authoring-error/0.1', error: { kind, message, ...details } });
function toolResult(value, isError = false) {
  return { content: [{ type: 'text', text: JSON.stringify(value) }], structuredContent: value, isError };
}
function workerValue(text, toolName, args, code, signal) {
  if (signal || ![0, 1].includes(code)) throw new Error(`worker exited ${signal ? `with signal ${signal}` : `with status ${code}`}`);
  if (!withinDepth(text)) throw new Error('worker JSON exceeds nesting limit');
  const value = JSON.parse(text);
  if (!object(value)) throw new Error('worker output is not an object');
  if (value.schema === 'caveat-authoring-error/0.1' && object(value.error)
    && ['input', 'limit', 'runtime'].includes(value.error.kind) && typeof value.error.message === 'string') return toolResult(value, true);
  if (code !== 0 || value.schema !== 'caveat-authoring/0.1'
    || value.operation !== toolName.slice('caveat_'.length)
    || value.sourceSha256 !== createHash('sha256').update(args.source).digest('hex')
    || !object(value.runtime) || ![0, 1, 2].includes(value.exitCode) || !object(value.report)) {
    throw new Error('worker output does not match the authoring operation contract');
  }
  return toolResult(value);
}

/** CLI-private. spawnWorker and timeoutMs are test seams, never tool arguments. */
export async function serveMcp({ runtimeDirectory = defaultRuntimeDirectory(), input = process.stdin,
  output = process.stdout, spawnWorker = spawn, timeoutMs = 10000 } = {}) {
  if (!Number.isInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > 10000) throw new Error('invalid worker timeout');
  const manifest = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8'));
  const seen = new Set();
  const pending = new Set();
  let state = 'new', active = null, closing = false, exitCode = 0;
  const writes = new Set();
  let bufferedOutputBytes = 0;
  const terminate = (job, suppress = false) => {
    if (!job) return;
    if (suppress) job.suppressed = true;
    if (job.child && job.child.exitCode === null && job.child.signalCode === null) job.child.kill('SIGKILL');
  };
  const stopWorker = () => terminate(active, true);
  const abort = () => {
    if (closing) return;
    closing = true; exitCode = 1; stopWorker(); input.destroy();
    output.destroy();
    for (const receipt of writes) receipt.finish();
  };
  input.on('end', stopWorker);
  input.on('close', stopWorker);
  input.on('error', stopWorker);
  output.on('error', abort);
  output.on('close', abort);
  function send(response) {
    if (closing) return Promise.resolve();
    let serialized = JSON.stringify(response);
    if (Buffer.byteLength(serialized) > MESSAGE_BYTES) {
      serialized = JSON.stringify({ jsonrpc: '2.0', id: response.id,
        result: { ...response.result, ...toolResult(bridgeError('output_limit', 'Serialized response exceeds 4 MiB'), true) } });
      if (Buffer.byteLength(serialized) > MESSAGE_BYTES) { abort(); return Promise.resolve(); }
    }
    const line = `${serialized}\n`;
    const bytes = Buffer.byteLength(line);
    // Never let a stalled reader create an unbounded output queue or stop us
    // reading cancellation/EOF. The message limit excludes its LF delimiter.
    if (bufferedOutputBytes + bytes > MESSAGE_BYTES + 1) { abort(); return Promise.resolve(); }
    let resolve;
    const receipt = { done: new Promise(complete => { resolve = complete; }), finish: null };
    receipt.finish = error => {
      if (!writes.delete(receipt)) return;
      bufferedOutputBytes -= bytes;
      resolve();
      if (error) abort();
    };
    writes.add(receipt); bufferedOutputBytes += bytes;
    try { output.write(line, receipt.finish); }
    catch (error) { receipt.finish(error); }
    // Delivery is tracked separately so transport backpressure cannot delay
    // worker cancellation, process reaping, or another incoming control frame.
    return Promise.resolve();
  }
  const rpcError = (id, code, message, data) => send({ jsonrpc: '2.0', id, error: { code, message, ...(data === undefined ? {} : { data }) } });
  const serverInfo = { name: 'caveat-lang', version: manifest.version, title: 'CAVEAT Language' };
  // A 2026-07-28 result names its type and the server; 2025-11-25 results are unchanged.
  const stateless = value => ({ resultType: 'complete', ...value, _meta: { [META_SERVER]: serverInfo } });
  const legacy = value => value;
  function startTool(id, name, args, finish) {
    const job = { id, key: keyOf(id), child: null, suppressed: false, failure: null };
    active = job;
    let child;
    try {
      // The worker uses fixed executable/module paths, inline input and local
      // runtime files; it needs no application configuration or parent secrets.
      // Node/libuv may supply platform support variables (notably on Windows).
      child = spawnWorker(process.execPath, ['--max-old-space-size=128', WORKER, runtimeDirectory], {
        stdio: ['pipe', 'pipe', 'pipe'], shell: false, windowsHide: true, env: {},
      });
      job.child = child;
    } catch (error) {
      active = null;
      return send({ jsonrpc: '2.0', id, result: finish(toolResult(bridgeError('worker_failure', `Cannot launch authoring worker: ${error.message}`), true)) });
    }
    const chunks = [], errors = [];
    let bytes = 0, errorBytes = 0;
    const fail = (kind, message) => { job.failure ??= bridgeError(kind, message); terminate(job); };
    const timer = setTimeout(() => fail('timeout', `Authoring operation exceeded ${timeoutMs} ms`), timeoutMs);
    child.stdout.on('data', chunk => {
      if (job.failure || job.suppressed) return;
      bytes += chunk.length;
      if (bytes > MESSAGE_BYTES) { fail('output_limit', 'Authoring worker output exceeds 4 MiB'); return; }
      chunks.push(Buffer.from(chunk));
    });
    child.stderr.on('data', chunk => {
      if (job.failure || job.suppressed) return;
      errorBytes += chunk.length;
      if (errorBytes > STDERR_BYTES) { fail('stderr_limit', 'Authoring worker stderr exceeds 64 KiB'); return; }
      errors.push(Buffer.from(chunk));
    });
    child.on('error', error => { job.failure ??= bridgeError('worker_failure', `Authoring worker failed: ${error.message}`); });
    child.stdin.on('error', error => fail('worker_failure', `Cannot send authoring input: ${error.message}`));
    const closed = new Promise(resolve => child.once('close', (code, signal) => resolve({ code, signal })));
    const done = closed.then(async ({ code, signal }) => {
      clearTimeout(timer);
      if (active === job) active = null; // Only close releases the one-child slot.
      if (job.suppressed || closing) return;
      let result;
      if (job.failure) result = toolResult(job.failure, true);
      else {
        try { result = workerValue(new TextDecoder('utf-8', { fatal: true }).decode(Buffer.concat(chunks)), name, args, code, signal); }
        catch (error) {
          const stderr = Buffer.concat(errors).toString('utf8');
          result = toolResult(bridgeError('worker_output', `Invalid authoring worker result: ${error.message}`, stderr ? { stderr } : {}), true);
        }
      }
      await send({ jsonrpc: '2.0', id, result: finish(result) });
    }).catch(() => { abort(); }).finally(() => pending.delete(done));
    pending.add(done);
    try { child.stdin.end(`${JSON.stringify({ tool: name, arguments: args })}\n`); }
    catch (error) { fail('worker_failure', `Cannot send authoring input: ${error.message}`); }
    return Promise.resolve();
  }
  async function handle(bytes) {
    let text, request;
    try { text = new TextDecoder('utf-8', { fatal: true }).decode(bytes); }
    catch { await rpcError(null, -32700, 'Input is not UTF-8'); return; }
    if (!withinDepth(text)) { await rpcError(null, -32600, 'JSON nesting exceeds 64'); return; }
    try { request = JSON.parse(text); }
    catch { await rpcError(null, -32700, 'Invalid JSON'); return; }
    const notification = object(request) && request.jsonrpc === '2.0' && typeof request.method === 'string' && !Object.hasOwn(request, 'id');
    if (!hasOnly(request, ['jsonrpc', 'id', 'method', 'params']) || request.jsonrpc !== '2.0'
      || typeof request.method !== 'string' || (Object.hasOwn(request, 'params') && !object(request.params))
      || (!notification && !idValid(request.id))) {
      if (!notification) await rpcError(null, -32600, 'Expected a JSON-RPC request object with a string or safe integer ID');
      return;
    }
    const { id, method, params } = request;
    if (notification) {
      if (method === 'notifications/initialized' && state === 'initializing' && paramsEmpty(params)) state = 'ready';
      if (method === 'notifications/cancelled' && hasOnly(params, ['requestId', 'reason', '_meta'])
        && idValid(params.requestId) && (params.reason === undefined || typeof params.reason === 'string')
        && active?.key === keyOf(params.requestId)) terminate(active, true);
      return;
    }
    const key = keyOf(id);
    if (seen.has(key)) { await rpcError(id, -32600, 'Request IDs cannot be reused in this connection'); return; }
    if (seen.size >= MAX_IDS) {
      await rpcError(id, -32600, 'Request ID limit reached; reconnect');
      closing = true; exitCode = 1; terminate(active, true); return;
    }
    seen.add(key);
    // A request naming its protocol version in _meta is served statelessly
    // (2026-07-28); initialize, and requests without it, keep 2025-11-25.
    if (method === 'server/discover'
      || (method !== 'initialize' && object(params?._meta) && Object.hasOwn(params._meta, META_VERSION))) {
      const meta = object(params?._meta) ? params._meta : {};
      const version = meta[META_VERSION];
      if (typeof version !== 'string' || !object(meta[META_CAPABILITIES])) {
        await rpcError(id, -32602, `Requests without initialize need _meta ${META_VERSION} and ${META_CAPABILITIES}`); return;
      }
      if (Object.hasOwn(meta, META_CLIENT) && !implementation(meta[META_CLIENT])) {
        await rpcError(id, -32602, `_meta ${META_CLIENT} needs a string name and version`); return;
      }
      if (!MCP_SUPPORTED_VERSIONS.includes(version)) {
        await rpcError(id, UNSUPPORTED_PROTOCOL_VERSION, 'Unsupported protocol version',
          { supported: [...MCP_SUPPORTED_VERSIONS], requested: version });
        return;
      }
      if (method === 'server/discover') {
        if (!paramsEmpty(params)) await rpcError(id, -32602, 'server/discover takes only _meta');
        else await send({ jsonrpc: '2.0', id, result: stateless({
          supportedVersions: [...MCP_SUPPORTED_VERSIONS], capabilities: { tools: {} }, instructions: INSTRUCTIONS,
          ttlMs: TOOLS_TTL_MS, cacheScope: 'public',
        }) });
        return;
      }
      if (version !== MCP_STATELESS_PROTOCOL_VERSION) {
        await rpcError(id, -32602, `Protocol version ${version} begins with initialize; requests carrying _meta use ${MCP_STATELESS_PROTOCOL_VERSION}`);
        return;
      }
      if (!['tools/list', 'tools/call'].includes(method)) { await rpcError(id, -32601, 'Method not found'); return; }
      await tools(id, method, params, stateless);
      return;
    }
    if (method === 'ping') {
      if (!paramsEmpty(params)) await rpcError(id, -32602, 'ping takes no parameters');
      else await send({ jsonrpc: '2.0', id, result: {} });
      return;
    }
    if (method === 'initialize') {
      if (state !== 'new') { await rpcError(id, -32600, 'Connection is already initialized'); return; }
      if (!hasOnly(params, ['protocolVersion', 'capabilities', 'clientInfo', '_meta'])
        || typeof params.protocolVersion !== 'string' || !params.protocolVersion
        || !object(params.capabilities) || !object(params.clientInfo)
        || typeof params.clientInfo.name !== 'string' || typeof params.clientInfo.version !== 'string') {
        await rpcError(id, -32602, 'initialize requires protocolVersion, capabilities, and clientInfo'); return;
      }
      state = 'initializing';
      await send({ jsonrpc: '2.0', id, result: {
        protocolVersion: MCP_PROTOCOL_VERSION, capabilities: { tools: {} }, serverInfo, instructions: INSTRUCTIONS,
      } });
      return;
    }
    if (!['tools/list', 'tools/call'].includes(method)) { await rpcError(id, -32601, 'Method not found'); return; }
    if (state !== 'ready') { await rpcError(id, -32600, 'Initialize, then send notifications/initialized before using tools'); return; }
    await tools(id, method, params, legacy);
  }
  async function tools(id, method, params, finish) {
    if (method === 'tools/list') {
      if (!paramsEmpty(params)) await rpcError(id, -32602, 'tools/list takes no cursor or other parameters');
      else await send({ jsonrpc: '2.0', id, result: finish === stateless
        ? stateless({ tools: MCP_TOOLS, ttlMs: TOOLS_TTL_MS, cacheScope: 'public' }) : { tools: MCP_TOOLS } });
      return;
    }
    if (!hasOnly(params, ['name', 'arguments', '_meta']) || !toolNames.has(params.name) || !object(params.arguments)) {
      await rpcError(id, -32602, 'tools/call requires a supported name and arguments object'); return;
    }
    try { validateAuthoringArguments(params.name, params.arguments); }
    catch (error) {
      await send({ jsonrpc: '2.0', id, result: finish(toolResult(bridgeError(error.kind === 'limit' ? 'limit' : 'input', error.message), true)) });
      return;
    }
    if (active) { await send({ jsonrpc: '2.0', id, result: finish(toolResult(bridgeError('busy', 'One authoring operation is active; retry after it finishes'), true)) }); return; }
    await startTool(id, params.name, params.arguments, finish);
  }
  let segments = [], length = 0;
  try {
    for await (const data of input) {
      const chunk = Buffer.isBuffer(data) ? data : Buffer.from(data);
      let start = 0;
      while (start < chunk.length && !closing) {
        const newline = chunk.indexOf(10, start);
        const end = newline < 0 ? chunk.length : newline;
        const piece = chunk.subarray(start, end);
        length += piece.length;
        if (length > MESSAGE_BYTES) {
          await rpcError(null, -32600, 'Incoming JSON-RPC line exceeds 4 MiB');
          closing = true; exitCode = 1; terminate(active, true); break;
        }
        segments.push(piece);
        if (newline < 0) break;
        await handle(Buffer.concat(segments, length));
        segments = []; length = 0; start = newline + 1;
      }
      if (closing) break;
    }
    if (!closing && length) { await rpcError(null, -32700, 'Input ended before the JSON-RPC newline'); exitCode = 1; }
  } catch { exitCode = 1; }
  finally {
    closing = true;
    terminate(active, true);
    await Promise.all([...pending]);
    // Preserve responses already accepted by an ordinary writable, but do not
    // keep the server alive indefinitely after its peer has closed stdin.
    if (writes.size) {
      let timer;
      await Promise.race([
        Promise.all([...writes].map(receipt => receipt.done)),
        new Promise(resolve => { timer = setTimeout(() => {
          output.destroy();
          for (const receipt of writes) receipt.finish();
          resolve();
        }, 100); }),
      ]);
      clearTimeout(timer);
    }
    input.removeListener('end', stopWorker);
    input.removeListener('close', stopWorker);
    input.removeListener('error', stopWorker);
    output.removeListener('error', abort);
    output.removeListener('close', abort);
  }
  return exitCode;
}
