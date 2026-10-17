// Streamable HTTP for the hosted Caveat MCP server. Each POST carries one
// JSON-RPC message and gets one JSON response; nothing is kept between
// requests, so there is no session ID and no server-to-client stream.
// The tools, their schemas, argument checks and protocol versions are the
// published package's own (lib/mcp.mjs and lib/authoring.mjs), so the hosted
// server answers as `caveat-lang mcp` does over stdio.
import { MCP_PROTOCOL_VERSION, MCP_STATELESS_PROTOCOL_VERSION, MCP_SUPPORTED_VERSIONS, MCP_TOOLS }
  from '../node_modules/caveat-lang/lib/mcp.mjs';
import { AUTHORING_LIMITS, AuthoringError, validateAuthoringArguments }
  from '../node_modules/caveat-lang/lib/authoring.mjs';

const META_VERSION = 'io.modelcontextprotocol/protocolVersion';
const META_CAPABILITIES = 'io.modelcontextprotocol/clientCapabilities';
const META_CLIENT = 'io.modelcontextprotocol/clientInfo';
const META_SERVER = 'io.modelcontextprotocol/serverInfo';
const UNSUPPORTED_PROTOCOL_VERSION = -32022;
const TOOLS_TTL_MS = 3600000;
const INSTRUCTIONS = 'Inline authoring operations only. Evidence and reports are supplied data; no file access or external permission is implied.';
const MESSAGE_BYTES = AUTHORING_LIMITS.messageBytes;
const MAX_DEPTH = 64;
// Earlier HTTP clients still ask for these. Their tool results are the same
// shape; a client that predates structuredContent reads the text copy.
export const HTTP_PROTOCOL_VERSIONS = Object.freeze([MCP_PROTOCOL_VERSION, '2025-06-18', '2025-03-26']);
const HEADER_VERSIONS = new Set([...HTTP_PROTOCOL_VERSIONS, MCP_STATELESS_PROTOCOL_VERSION]);
const toolNames = new Set(MCP_TOOLS.map(item => item.name));
const encoder = new TextEncoder();
const byteLength = text => encoder.encode(text).length;

const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const idValid = id => typeof id === 'string' || Number.isSafeInteger(id);
const hasOnly = (value, allowed) => object(value) && Object.keys(value).every(key => allowed.includes(key));
const paramsEmpty = value => value === undefined || hasOnly(value, ['_meta']);
const implementation = value => object(value) && typeof value.name === 'string' && typeof value.version === 'string';
const bridgeError = (kind, message) => ({ schema: 'caveat-authoring-error/0.1', error: { kind, message } });
const toolResult = (value, isError = false) => ({ content: [{ type: 'text', text: JSON.stringify(value) }], structuredContent: value, isError });

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

const CORS = {
  'access-control-allow-origin': '*',
  'access-control-allow-methods': 'POST, OPTIONS',
  'access-control-allow-headers': 'content-type, accept, mcp-protocol-version, mcp-session-id, last-event-id, authorization',
  'access-control-max-age': '86400',
};
const respond = (status, body, headers = {}) => new Response(body, { status, headers: { ...CORS, ...headers } });
const json = (status, value) => respond(status, JSON.stringify(value), { 'content-type': 'application/json' });
const rpcError = (status, id, code, message, data) =>
  json(status, { jsonrpc: '2.0', id, error: { code, message, ...(data === undefined ? {} : { data }) } });

// Scenario steps in a caveat_test call, counting each repeated send.
function scenarioSteps(scenarios) {
  let steps = 0;
  for (const scenario of Array.isArray(scenarios?.scenarios) ? scenarios.scenarios : []) {
    for (const step of Array.isArray(scenario?.steps) ? scenario.steps : []) {
      steps += object(step) && 'send' in step && Number.isInteger(step.repeat) && step.repeat > 1 ? step.repeat : 1;
    }
  }
  return steps;
}

/**
 * run(toolName, args) performs one authoring operation, as runAuthoringOperation
 * does. limiter, when given, is a Workers rate limiting binding. testSteps, when
 * given, bounds the scenario steps one caveat_test call may run, so a call stays
 * within a CPU budget, and caveat_test's description says so. challenge, when
 * given, is the token served for OpenAI's domain verification.
 */
export function createMcpHandler({ run, serverInfo, limiter = null, testSteps = null, challenge = null, path = '/mcp', home = '' }) {
  const toolList = testSteps === null ? MCP_TOOLS : MCP_TOOLS.map(item => item.name !== 'caveat_test' ? item : {
    ...item, description: `${item.description} This hosted server runs at most ${testSteps} scenario steps per call, counting repeated sends; split larger scenario files across calls.`,
  });
  const stateless = value => ({ resultType: 'complete', ...value, _meta: { [META_SERVER]: serverInfo } });
  const legacy = value => value;

  async function call(name, args) {
    try { validateAuthoringArguments(name, args); }
    catch (error) { return toolResult(bridgeError(error.kind === 'limit' ? 'limit' : 'input', error.message), true); }
    if (name === 'caveat_test' && testSteps !== null && scenarioSteps(args.scenarios) > testSteps) {
      return toolResult(bridgeError('limit', `the hosted server runs at most ${testSteps} scenario steps per caveat_test call, counting repeated sends; split the scenarios across calls, replay long histories with caveat_explain, or run npx caveat-lang mcp locally`), true);
    }
    try {
      const value = await run(name, args);
      if (byteLength(JSON.stringify(value)) > AUTHORING_LIMITS.outputBytes) throw new AuthoringError('limit', 'worker result exceeds 4 MiB UTF-8');
      return toolResult(value);
    } catch (error) {
      return toolResult(bridgeError(error instanceof AuthoringError ? error.kind : 'runtime', error.message), true);
    }
  }

  async function tools(id, method, params, finish) {
    if (method === 'tools/list') {
      if (!paramsEmpty(params)) return rpcError(200, id, -32602, 'tools/list takes no cursor or other parameters');
      return json(200, { jsonrpc: '2.0', id, result: finish === stateless
        ? stateless({ tools: toolList, ttlMs: TOOLS_TTL_MS, cacheScope: 'public' }) : { tools: toolList } });
    }
    if (!hasOnly(params, ['name', 'arguments', '_meta']) || !toolNames.has(params.name) || !object(params.arguments)) {
      return rpcError(200, id, -32602, 'tools/call requires a supported name and arguments object');
    }
    let response = JSON.stringify({ jsonrpc: '2.0', id, result: finish(await call(params.name, params.arguments)) });
    if (byteLength(response) > MESSAGE_BYTES) {
      response = JSON.stringify({ jsonrpc: '2.0', id, result: finish(toolResult(bridgeError('output_limit', 'Serialized response exceeds 4 MiB'), true)) });
    }
    return respond(200, response, { 'content-type': 'application/json' });
  }

  async function message(request) {
    const notification = object(request) && request.jsonrpc === '2.0' && typeof request.method === 'string' && !Object.hasOwn(request, 'id');
    if (!hasOnly(request, ['jsonrpc', 'id', 'method', 'params']) || request.jsonrpc !== '2.0'
      || typeof request.method !== 'string' || (Object.hasOwn(request, 'params') && !object(request.params))
      || (!notification && !idValid(request.id))) {
      // Responses from the client (to requests this server never sends) are accepted and ignored.
      if (object(request) && request.jsonrpc === '2.0' && !Object.hasOwn(request, 'method')
        && (Object.hasOwn(request, 'result') || Object.hasOwn(request, 'error'))) return respond(202, null);
      return rpcError(400, null, -32600, 'Expected one JSON-RPC request object with a string or safe integer ID');
    }
    // Nothing runs between requests, so there is nothing to initialize or cancel.
    if (notification) return respond(202, null);
    const { id, method, params } = request;
    // A request naming its protocol version in _meta is served statelessly
    // (2026-07-28); initialize, and requests without it, keep 2025-11-25.
    if (method === 'server/discover'
      || (method !== 'initialize' && object(params?._meta) && Object.hasOwn(params._meta, META_VERSION))) {
      const meta = object(params?._meta) ? params._meta : {};
      const version = meta[META_VERSION];
      if (typeof version !== 'string' || !object(meta[META_CAPABILITIES])) {
        return rpcError(200, id, -32602, `Requests without initialize need _meta ${META_VERSION} and ${META_CAPABILITIES}`);
      }
      if (Object.hasOwn(meta, META_CLIENT) && !implementation(meta[META_CLIENT])) {
        return rpcError(200, id, -32602, `_meta ${META_CLIENT} needs a string name and version`);
      }
      if (!MCP_SUPPORTED_VERSIONS.includes(version)) {
        return rpcError(200, id, UNSUPPORTED_PROTOCOL_VERSION, 'Unsupported protocol version',
          { supported: [...MCP_SUPPORTED_VERSIONS], requested: version });
      }
      if (method === 'server/discover') {
        if (!paramsEmpty(params)) return rpcError(200, id, -32602, 'server/discover takes only _meta');
        return json(200, { jsonrpc: '2.0', id, result: stateless({
          supportedVersions: [...MCP_SUPPORTED_VERSIONS], capabilities: { tools: {} }, instructions: INSTRUCTIONS,
          ttlMs: TOOLS_TTL_MS, cacheScope: 'public',
        }) });
      }
      if (version !== MCP_STATELESS_PROTOCOL_VERSION) {
        return rpcError(200, id, -32602, `Protocol version ${version} begins with initialize; requests carrying _meta use ${MCP_STATELESS_PROTOCOL_VERSION}`);
      }
      if (!['tools/list', 'tools/call'].includes(method)) return rpcError(200, id, -32601, 'Method not found');
      return tools(id, method, params, stateless);
    }
    if (method === 'ping') {
      if (!paramsEmpty(params)) return rpcError(200, id, -32602, 'ping takes no parameters');
      return json(200, { jsonrpc: '2.0', id, result: {} });
    }
    if (method === 'initialize') {
      if (!hasOnly(params, ['protocolVersion', 'capabilities', 'clientInfo', '_meta'])
        || typeof params.protocolVersion !== 'string' || !params.protocolVersion
        || !object(params.capabilities) || !object(params.clientInfo)
        || typeof params.clientInfo.name !== 'string' || typeof params.clientInfo.version !== 'string') {
        return rpcError(200, id, -32602, 'initialize requires protocolVersion, capabilities, and clientInfo');
      }
      const protocolVersion = HTTP_PROTOCOL_VERSIONS.includes(params.protocolVersion) ? params.protocolVersion : MCP_PROTOCOL_VERSION;
      return json(200, { jsonrpc: '2.0', id, result: {
        protocolVersion, capabilities: { tools: {} }, serverInfo, instructions: INSTRUCTIONS,
      } });
    }
    if (!['tools/list', 'tools/call'].includes(method)) return rpcError(200, id, -32601, 'Method not found');
    return tools(id, method, params, legacy);
  }

  async function readBody(request) {
    const declared = Number(request.headers.get('content-length'));
    if (declared > MESSAGE_BYTES) return null;
    const reader = request.body?.getReader();
    if (!reader) return new Uint8Array(0);
    const chunks = [];
    let size = 0;
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.length;
      if (size > MESSAGE_BYTES) { await reader.cancel(); return null; }
      chunks.push(value);
    }
    const bytes = new Uint8Array(size);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.length; }
    return bytes;
  }

  return async function fetch(request) {
    const url = new URL(request.url);
    if (url.pathname === '/' && request.method === 'GET' && home) {
      return respond(200, home, { 'content-type': 'text/plain; charset=utf-8' });
    }
    if (url.pathname === '/.well-known/openai-apps-challenge' && request.method === 'GET' && challenge) {
      return respond(200, challenge, { 'content-type': 'text/plain; charset=utf-8' });
    }
    if (url.pathname !== path) return respond(404, 'Not found', { 'content-type': 'text/plain; charset=utf-8' });
    if (request.method === 'OPTIONS') return respond(204, null);
    // No server-to-client stream and no session to end.
    if (request.method !== 'POST') return respond(405, 'Method not allowed', { allow: 'POST, OPTIONS', 'content-type': 'text/plain; charset=utf-8' });
    if (limiter) {
      const key = request.headers.get('cf-connecting-ip') ?? 'unknown';
      const { success } = await limiter.limit({ key });
      if (!success) return rpcError(429, null, -32000, 'Too many requests; retry in a minute', { retryAfterSeconds: 60 });
    }
    const version = request.headers.get('mcp-protocol-version');
    if (version !== null && !HEADER_VERSIONS.has(version)) {
      return rpcError(400, null, -32600, `Unsupported MCP-Protocol-Version ${version}`);
    }
    if (!(request.headers.get('content-type') ?? '').toLowerCase().startsWith('application/json')) {
      return rpcError(415, null, -32700, 'Send one JSON-RPC message as application/json');
    }
    const bytes = await readBody(request);
    if (bytes === null) return rpcError(413, null, -32600, 'Request body exceeds 4 MiB');
    let text, parsed;
    try { text = new TextDecoder('utf-8', { fatal: true }).decode(bytes); }
    catch { return rpcError(400, null, -32700, 'Input is not UTF-8'); }
    if (!withinDepth(text)) return rpcError(400, null, -32600, 'JSON nesting exceeds 64');
    try { parsed = JSON.parse(text); }
    catch { return rpcError(400, null, -32700, 'Invalid JSON'); }
    if (Array.isArray(parsed)) return rpcError(400, null, -32600, 'Batches are not supported; send one message per request');
    return message(parsed);
  };
}
