// Private stdio MCP adapter. Every tool owns one disposable subprocess/runtime.
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { TextDecoder } from 'node:util';
import { defaultRuntimeDirectory } from './node.mjs';
import { validateAuthoringArguments } from './authoring.mjs';

export const MCP_PROTOCOL_VERSION = '2025-11-25';
const MESSAGE_BYTES = 4 * 1024 * 1024;
const STDERR_BYTES = 64 * 1024;
const MAX_DEPTH = 64;
const MAX_IDS = 4096;
const WORKER = fileURLToPath(new URL('./authoring-worker.mjs', import.meta.url));
const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const idValid = id => typeof id === 'string' || Number.isSafeInteger(id);
const keyOf = id => createHash('sha256').update(JSON.stringify(id)).digest('hex');
const sourceSchema = { type: 'string', maxLength: 1024 * 1024, description: 'Inline Caveat source; no file or URL lookup.' };
const eventSchema = {
  type: 'object', required: ['event'], additionalProperties: false,
  properties: { event: { type: 'string', minLength: 1 }, payload: { type: 'object' } },
};
const eventsSchema = { type: 'array', maxItems: 1000, items: eventSchema };
const tool = (name, description, properties, required = ['source']) => ({
  name, description,
  inputSchema: { type: 'object', additionalProperties: false, properties: { source: sourceSchema, ...properties }, required },
  annotations: { readOnlyHint: true, destructiveHint: false, idempotentHint: true, openWorldHint: false },
});
export const MCP_TOOLS = Object.freeze([
  tool('caveat_validate', 'Validate inline Caveat source using the installed runtime.', {}),
  tool('caveat_check', 'Check inline Caveat source for authoring diagnostics.', { strict: { type: 'boolean' } }),
  tool('caveat_test', 'Run scenarios against inline source. Every source reference must be inline.cav.', { scenarios: { type: 'object' } }, ['source', 'scenarios']),
  tool('caveat_explain', 'Explain supplied events and their evidence/decision history.', { events: eventsSchema }),
  tool('caveat_dependents', 'Inspect what depends on evidence, a reading stream, or a caveat.', { subject: { type: 'string', minLength: 1 }, events: eventsSchema }, ['source', 'subject']),
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
        result: toolResult(bridgeError('output_limit', 'Serialized response exceeds 4 MiB'), true) });
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
  const rpcError = (id, code, message) => send({ jsonrpc: '2.0', id, error: { code, message } });
  function startTool(id, name, args) {
    const job = { id, key: keyOf(id), child: null, suppressed: false, failure: null };
    active = job;
    let child;
    try {
      const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !['NODE_OPTIONS', 'NODE_PATH'].includes(key.toUpperCase())));
      child = spawnWorker(process.execPath, ['--max-old-space-size=128', WORKER, runtimeDirectory], {
        stdio: ['pipe', 'pipe', 'pipe'], shell: false, windowsHide: true, env,
      });
      job.child = child;
    } catch (error) {
      active = null;
      return send({ jsonrpc: '2.0', id, result: toolResult(bridgeError('worker_failure', `Cannot launch authoring worker: ${error.message}`), true) });
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
      await send({ jsonrpc: '2.0', id, result });
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
        protocolVersion: MCP_PROTOCOL_VERSION, capabilities: { tools: {} },
        serverInfo: { name: 'caveat-lang', version: manifest.version, title: 'CAVEAT Language' },
        instructions: 'Inline authoring operations only. Evidence and reports are supplied data; no file access or external permission is implied.',
      } });
      return;
    }
    if (!['tools/list', 'tools/call'].includes(method)) { await rpcError(id, -32601, 'Method not found'); return; }
    if (state !== 'ready') { await rpcError(id, -32600, 'Initialize, then send notifications/initialized before using tools'); return; }
    if (method === 'tools/list') {
      if (!paramsEmpty(params)) await rpcError(id, -32602, 'tools/list takes no cursor or other parameters');
      else await send({ jsonrpc: '2.0', id, result: { tools: MCP_TOOLS } });
      return;
    }
    if (!hasOnly(params, ['name', 'arguments', '_meta']) || !toolNames.has(params.name) || !object(params.arguments)) {
      await rpcError(id, -32602, 'tools/call requires a supported name and arguments object'); return;
    }
    try { validateAuthoringArguments(params.name, params.arguments); }
    catch (error) {
      await send({ jsonrpc: '2.0', id, result: toolResult(bridgeError(error.kind === 'limit' ? 'limit' : 'input', error.message), true) });
      return;
    }
    if (active) { await send({ jsonrpc: '2.0', id, result: toolResult(bridgeError('busy', 'One authoring operation is active; retry after it finishes'), true) }); return; }
    await startTool(id, params.name, params.arguments);
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
