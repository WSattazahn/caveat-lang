import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { createInterface } from 'node:readline';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { runAuthoringOperation } from '../node_modules/caveat-lang/lib/authoring.mjs';
import { MCP_TOOLS } from '../node_modules/caveat-lang/lib/mcp.mjs';
import { loadRuntimeFromDirectory } from '../node_modules/caveat-lang/lib/node.mjs';
import { createMcpHandler } from '../src/handler.mjs';
import { runtimeIdentity } from '../src/identity.mjs';

const pkg = new URL('../node_modules/caveat-lang/', import.meta.url);
const manifest = JSON.parse(await readFile(new URL('package.json', pkg), 'utf8'));
const SOURCE = await readFile(new URL('templates/umbrella.cav', pkg), 'utf8');
const SCENARIOS = { ...JSON.parse(await readFile(new URL('templates/umbrella.scenarios.json', pkg), 'utf8')), source: 'inline.cav' };
const EVENTS = (await readFile(new URL('templates/events.jsonl', pkg), 'utf8')).trim().split('\n').map(line => JSON.parse(line));
const serverInfo = { name: 'caveat-lang', version: manifest.version, title: 'CAVEAT Language' };
const CALLS = [
  ['caveat_validate', { source: SOURCE }],
  ['caveat_check', { source: SOURCE, strict: true }],
  ['caveat_test', { source: SOURCE, scenarios: SCENARIOS }],
  ['caveat_explain', { source: SOURCE, events: EVENTS }],
  ['caveat_dependents', { source: SOURCE, subject: 'forecast', events: EVENTS }],
  ['caveat_validate', { source: 'event a {}\n' }],
  ['caveat_explain', { source: 'event refuse; on refuse reject "no";', events: [{ event: 'refuse' }] }],
  ['caveat_dependents', { source: SOURCE, subject: 'umbrella' }],
];

function handler(options = {}) {
  return createMcpHandler({ run: (name, args) => runAuthoringOperation(name, args), serverInfo, home: 'hello\n', ...options });
}
function post(fetch, body, headers = {}) {
  return fetch(new Request('https://caveat.example/mcp', {
    method: 'POST', body: typeof body === 'string' ? body : JSON.stringify(body),
    headers: { 'content-type': 'application/json', accept: 'application/json, text/event-stream', ...headers },
  }));
}
const call = (id, name, args) => ({ jsonrpc: '2.0', id, method: 'tools/call', params: { name, arguments: args } });
const statelessMeta = { 'io.modelcontextprotocol/protocolVersion': '2026-07-28', 'io.modelcontextprotocol/clientCapabilities': {} };

// The published stdio server, for side-by-side answers.
async function stdio(messages) {
  const child = spawn(process.execPath, [fileURLToPath(new URL('bin/caveat.mjs', pkg)), 'mcp'], { stdio: ['pipe', 'pipe', 'inherit'] });
  const lines = createInterface({ input: child.stdout });
  const replies = new Map();
  const waiting = new Map();
  lines.on('line', line => {
    const reply = JSON.parse(line);
    replies.set(reply.id, reply);
    waiting.get(reply.id)?.(reply);
  });
  const wait = id => replies.has(id) ? Promise.resolve(replies.get(id)) : new Promise(resolve => waiting.set(id, resolve));
  const send = value => child.stdin.write(`${JSON.stringify(value)}\n`);
  send({ jsonrpc: '2.0', id: 0, method: 'initialize', params: { protocolVersion: '2025-11-25', capabilities: {}, clientInfo: { name: 'test', version: '1' } } });
  await wait(0);
  send({ jsonrpc: '2.0', method: 'notifications/initialized' });
  const out = [];
  for (const [index, value] of messages.entries()) { send({ ...value, id: index + 1 }); out.push(await wait(index + 1)); }
  child.stdin.end();
  await new Promise(resolve => child.once('close', resolve));
  return out;
}

test('tools/list and every tool answer as the published stdio server does', async () => {
  const fetch = handler();
  const messages = [{ jsonrpc: '2.0', method: 'tools/list' }, ...CALLS.map(([name, args]) => call(0, name, args))];
  const expected = await stdio(messages);
  for (const [index, value] of messages.entries()) {
    const response = await post(fetch, { ...value, id: index + 1 });
    assert.equal(response.status, 200);
    assert.equal(response.headers.get('content-type'), 'application/json');
    assert.deepEqual(await response.json(), expected[index], value.params?.name ?? value.method);
  }
  assert.deepEqual(expected[0].result.tools, MCP_TOOLS);
  assert.deepEqual(expected.slice(1, 6).map(reply => reply.result.structuredContent.exitCode), [0, 0, 0, 0, 0]);
});

test('initialize answers without a session and keeps versions HTTP clients ask for', async () => {
  const fetch = handler();
  for (const [asked, answered] of [['2025-11-25', '2025-11-25'], ['2025-06-18', '2025-06-18'], ['2025-03-26', '2025-03-26'], ['2024-11-05', '2025-11-25']]) {
    const response = await post(fetch, { jsonrpc: '2.0', id: 1, method: 'initialize', params: { protocolVersion: asked, capabilities: {}, clientInfo: { name: 'c', version: '1' } } });
    assert.equal(response.headers.get('mcp-session-id'), null);
    const { result } = await response.json();
    assert.equal(result.protocolVersion, answered);
    assert.deepEqual(result.serverInfo, serverInfo);
    assert.deepEqual(result.capabilities, { tools: {} });
  }
  const initialized = await post(fetch, { jsonrpc: '2.0', method: 'notifications/initialized' });
  assert.equal(initialized.status, 202);
  assert.equal(await initialized.text(), '');
  assert.deepEqual(await (await post(fetch, { jsonrpc: '2.0', id: 'p', method: 'ping' })).json(), { jsonrpc: '2.0', id: 'p', result: {} });
});

test('2026-07-28 requests carry their version in _meta, as over stdio', async () => {
  const fetch = handler();
  const list = await (await post(fetch, { jsonrpc: '2.0', id: 1, method: 'tools/list', params: { _meta: statelessMeta } })).json();
  assert.equal(list.result.resultType, 'complete');
  assert.deepEqual(list.result.tools, MCP_TOOLS);
  assert.deepEqual(list.result._meta, { 'io.modelcontextprotocol/serverInfo': serverInfo });
  const discover = await (await post(fetch, { jsonrpc: '2.0', id: 2, method: 'server/discover', params: { _meta: statelessMeta } })).json();
  assert.deepEqual(discover.result.supportedVersions, ['2026-07-28', '2025-11-25']);
  const old = await (await post(fetch, { jsonrpc: '2.0', id: 3, method: 'tools/list', params: { _meta: { ...statelessMeta, 'io.modelcontextprotocol/protocolVersion': '2099-01-01' } } })).json();
  assert.equal(old.error.code, -32022);
  const called = await (await post(fetch, { ...call(4, 'caveat_validate', { source: SOURCE }), params: { name: 'caveat_validate', arguments: { source: SOURCE }, _meta: statelessMeta } })).json();
  assert.equal(called.result.structuredContent.report.loads, true);
});

test('bridge failures are tool errors and protocol failures are JSON-RPC errors', async () => {
  const fetch = handler();
  const bad = await (await post(fetch, call(1, 'caveat_validate', { source: 7 }))).json();
  assert.equal(bad.result.isError, true);
  assert.deepEqual(bad.result.structuredContent, { schema: 'caveat-authoring-error/0.1', error: { kind: 'input', message: 'source must be inline Caveat text' } });
  const many = await (await post(fetch, call(2, 'caveat_explain', { source: SOURCE, events: Array.from({ length: 1001 }, () => ({ event: 'leave' })) }))).json();
  assert.equal(many.result.structuredContent.error.kind, 'limit');
  const broken = createMcpHandler({ run: () => { throw new Error('the runtime instance trapped; load it again'); }, serverInfo });
  const trapped = await (await post(broken, call(3, 'caveat_validate', { source: SOURCE }))).json();
  assert.deepEqual(trapped.result.structuredContent.error, { kind: 'runtime', message: 'the runtime instance trapped; load it again' });
  assert.equal((await (await post(fetch, call(4, 'caveat_nothing', {}))).json()).error.code, -32602);
  assert.equal((await (await post(fetch, { jsonrpc: '2.0', id: 5, method: 'resources/list' })).json()).error.code, -32601);
  const parse = await post(fetch, '{"jsonrpc":');
  assert.equal(parse.status, 400);
  assert.equal((await parse.json()).error.code, -32700);
  assert.equal((await post(fetch, '[]')).status, 400);
  assert.equal((await post(fetch, `${'['.repeat(65)}${']'.repeat(65)}`)).status, 400);
  assert.equal((await post(fetch, { jsonrpc: '2.0', id: 6, result: {} })).status, 202);
});

test('a step budget bounds caveat_test only', async () => {
  const fetch = handler({ testSteps: 12 });
  const fits = await (await post(fetch, call(1, 'caveat_test', { source: SOURCE, scenarios: SCENARIOS }))).json();
  assert.equal(fits.result.structuredContent.exitCode, 0);
  const repeated = structuredClone(SCENARIOS);
  repeated.scenarios[1].steps.unshift({ send: 'leave', repeat: 2 });
  const over = await (await post(fetch, call(2, 'caveat_test', { source: SOURCE, scenarios: repeated }))).json();
  assert.equal(over.result.isError, true);
  assert.equal(over.result.structuredContent.error.kind, 'limit');
  assert.match(over.result.structuredContent.error.message, /at most 12 scenario steps/);
  const { result } = await (await post(fetch, { jsonrpc: '2.0', id: 4, method: 'tools/list' })).json();
  for (const [index, item] of result.tools.entries()) {
    if (item.name === 'caveat_test') assert.equal(item.description, `${MCP_TOOLS[index].description} This hosted server runs at most 12 scenario steps per call, counting repeated sends; split larger scenario files across calls.`);
    else assert.deepEqual(item, MCP_TOOLS[index]);
  }
  const explain = await (await post(fetch, call(3, 'caveat_explain', { source: SOURCE, events: Array.from({ length: 40 }, () => ({ event: 'leave' })) }))).json();
  assert.equal(explain.result.isError, false);
});

test('the OpenAI domain challenge is served only when set', async () => {
  const url = 'https://caveat.example/.well-known/openai-apps-challenge';
  assert.equal((await handler()(new Request(url))).status, 404);
  const response = await handler({ challenge: 'token-123' })(new Request(url));
  assert.equal(response.status, 200);
  assert.equal(response.headers.get('content-type'), 'text/plain; charset=utf-8');
  assert.equal(await response.text(), 'token-123');
});

test('HTTP edges: methods, paths, headers, size and rate limit', async () => {
  const fetch = handler();
  const get = await fetch(new Request('https://caveat.example/mcp'));
  assert.equal(get.status, 405);
  assert.equal(get.headers.get('allow'), 'POST, OPTIONS');
  assert.equal((await fetch(new Request('https://caveat.example/mcp', { method: 'DELETE' }))).status, 405);
  const preflight = await fetch(new Request('https://caveat.example/mcp', { method: 'OPTIONS' }));
  assert.equal(preflight.status, 204);
  assert.equal(preflight.headers.get('access-control-allow-origin'), '*');
  assert.match(preflight.headers.get('access-control-allow-headers'), /mcp-protocol-version/);
  assert.equal((await fetch(new Request('https://caveat.example/other'))).status, 404);
  assert.equal(await (await fetch(new Request('https://caveat.example/'))).text(), 'hello\n');
  assert.equal((await post(fetch, { jsonrpc: '2.0', id: 1, method: 'ping' }, { 'content-type': 'text/plain' })).status, 415);
  assert.equal((await post(fetch, { jsonrpc: '2.0', id: 1, method: 'ping' }, { 'mcp-protocol-version': '2099-01-01' })).status, 400);
  assert.equal((await post(fetch, { jsonrpc: '2.0', id: 1, method: 'ping' }, { 'mcp-protocol-version': '2025-06-18' })).status, 200);
  const huge = await post(fetch, JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'ping', params: { _meta: { pad: 'x'.repeat(4 * 1024 * 1024) } } }));
  assert.equal(huge.status, 413);
  const keys = [];
  let allow = 1;
  const limited = handler({ limiter: { limit: async ({ key }) => { keys.push(key); return { success: allow-- > 0 }; } } });
  assert.equal((await post(limited, { jsonrpc: '2.0', id: 1, method: 'ping' }, { 'cf-connecting-ip': '203.0.113.9' })).status, 200);
  const refused = await post(limited, { jsonrpc: '2.0', id: 2, method: 'ping' }, { 'cf-connecting-ip': '203.0.113.9' });
  assert.equal(refused.status, 429);
  assert.equal((await refused.json()).error.code, -32000);
  assert.deepEqual(keys, ['203.0.113.9', '203.0.113.9']);
});

test('the bundled runtime identity is the one the Node loader reports', async () => {
  const info = JSON.parse(await readFile(new URL('runtime/build-info.json', pkg), 'utf8'));
  const runtime = await loadRuntimeFromDirectory(fileURLToPath(new URL('runtime/', pkg)));
  assert.deepEqual(runtimeIdentity(info), runtime.identity);
});
