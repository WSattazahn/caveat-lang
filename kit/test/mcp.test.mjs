// MCP protocol and supervisor checks. Worker fault fixtures below are injected;
// they do not claim naturally occurring WASM traps or operating-system sandboxing.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { PassThrough } from 'node:stream';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { MCP_PROTOCOL_VERSION, MCP_TOOLS, serveMcp } from '../lib/mcp.mjs';

// The package gate creates/removes kit/runtime while packing. Repository tests
// must hold the durable build path, not capture that transient staging folder.
// Installed-package tests separately verify the bundled-runtime default.
const RUNTIME = fileURLToPath(new URL('../../dist/pkg-reactive/', import.meta.url));
const SOURCE = 'evidence memory from "lookup"; event consult; on consult reveal memory;';
const request = (id, method, params) => ({ jsonrpc: '2.0', id, method, ...(params === undefined ? {} : { params }) });
const notification = (method, params) => ({ jsonrpc: '2.0', method, ...(params === undefined ? {} : { params }) });
const call = (id, name = 'caveat_validate', args = { source: SOURCE }) => request(id, 'tools/call', { name, arguments: args });
function connection(options = {}) {
  const { outputHighWaterMark, ...serverOptions } = options;
  const input = new PassThrough();
  const output = new PassThrough(outputHighWaterMark === undefined ? {} : { highWaterMark: outputHighWaterMark });
  const messages = [], waiters = [];
  let pending = '';
  output.on('data', data => {
    pending += data.toString('utf8');
    for (;;) {
      const newline = pending.indexOf('\n');
      if (newline < 0) break;
      const value = JSON.parse(pending.slice(0, newline)); pending = pending.slice(newline + 1);
      messages.push(value);
      for (const waiter of [...waiters]) {
        if (waiter.test(value)) { waiters.splice(waiters.indexOf(waiter), 1); waiter.resolve(value); }
      }
    }
  });
  const done = serveMcp({ runtimeDirectory: RUNTIME, input, output, ...serverOptions });
  const wait = id => {
    const found = messages.find(message => Object.is(message.id, id));
    if (found) return Promise.resolve(found);
    let waiter;
    const received = new Promise(resolve => { waiter = { test: value => Object.is(value.id, id), resolve }; waiters.push(waiter); });
    let timer;
    const timed = new Promise((_, reject) => { timer = setTimeout(() => reject(new Error(`No MCP response for ${id}`)), 12000); });
    return Promise.race([received, timed])
      .finally(() => { clearTimeout(timer); const at = waiters.indexOf(waiter); if (at >= 0) waiters.splice(at, 1); });
  };
  const send = value => input.write(`${JSON.stringify(value)}\n`);
  const initialize = async (version = MCP_PROTOCOL_VERSION) => {
    send(request('init', 'initialize', { protocolVersion: version, capabilities: {}, clientInfo: { name: 'fixture', version: '1' } }));
    const response = await wait('init');
    send(notification('notifications/initialized'));
    return response;
  };
  return { input, output, messages, send, wait, initialize, done,
    close: async () => { input.end(); const code = await done; output.destroy(); return code; } };
}

function injected(code) {
  const children = [];
  let spawned = 0;
  const spawnWorker = (command, args, options) => {
    assert.equal(args.at(-1), RUNTIME, 'real workers retain the durable repository runtime path');
    assert.equal(options.shell, false);
    assert.equal(options.windowsHide, true);
    assert.deepEqual(options.env, {}, 'the bridge does not explicitly inherit application environment');
    const child = spawned++ === 0 ? spawn(process.execPath, ['-e', code], options) : spawn(command, args, options);
    children.push(child);
    return child;
  };
  return { children, spawnWorker };
}
const noLongerRunning = child => {
  assert.ok(child.exitCode !== null || child.signalCode !== null, 'the child exited before the slot/connection was released');
  assert.throws(() => process.kill(child.pid, 0), error => error.code === 'ESRCH');
};

test('MCP negotiates its explicit profile, initializes before tools, and lists only inline authoring tools', async () => {
  const client = connection();
  try {
    client.send(request(1, 'tools/list'));
    assert.equal((await client.wait(1)).error.code, -32600);
    client.send(request(2, 'ping'));
    assert.deepEqual((await client.wait(2)).result, {});
    const initialized = await client.initialize('2026-07-28');
    assert.equal(initialized.result.protocolVersion, MCP_PROTOCOL_VERSION);
    assert.deepEqual(initialized.result.capabilities, { tools: {} });
    client.send(request(3, 'tools/list'));
    assert.deepEqual((await client.wait(3)).result.tools, MCP_TOOLS);
    assert.deepEqual(MCP_TOOLS.map(item => item.name), ['caveat_validate', 'caveat_check', 'caveat_test', 'caveat_explain', 'caveat_dependents']);
    for (const tool of MCP_TOOLS) {
      assert.equal(tool.inputSchema.additionalProperties, false);
      assert.equal(tool.annotations.readOnlyHint, true);
      assert.equal(tool.annotations.openWorldHint, false);
      assert.equal(Object.hasOwn(tool.inputSchema.properties, 'path'), false);
    }
    client.send(request(4, 'initialize', {}));
    assert.equal((await client.wait(4)).error.code, -32600);
  } finally { assert.equal(await client.close(), 0); }
});

test('all five tools execute fresh real workers and retain completed CLI-style outcomes', async () => {
  const client = connection();
  try {
    await client.initialize();
    const cases = [
      ['caveat_validate', { source: SOURCE }],
      ['caveat_check', { source: SOURCE, strict: true }],
      ['caveat_test', { source: SOURCE, scenarios: { schema: 'caveat-scenarios/0.1', source: 'inline.cav', scenarios: [{ id: 'S1', title: 'neutral', steps: [{ send: 'consult' }] }] } }],
      ['caveat_explain', { source: SOURCE, events: [{ event: 'consult' }] }],
      ['caveat_dependents', { source: SOURCE, subject: 'memory', events: [{ event: 'consult' }] }],
    ];
    for (const [index, [name, args]] of cases.entries()) {
      client.send(call(index, name, args));
      const response = await client.wait(index);
      assert.equal(response.error, undefined, JSON.stringify(response));
      assert.equal(response.result.isError, false, JSON.stringify(response));
      const value = response.result.structuredContent;
      assert.equal(value.schema, 'caveat-authoring/0.1');
      assert.equal(value.operation, name.slice(7));
      assert.equal(value.exitCode, 0, JSON.stringify(value));
      assert.deepEqual(JSON.parse(response.result.content[0].text), value);
      assert.match(value.runtime.reactiveWasmSha256, /^[a-f0-9]{64}$/);
    }
    client.send(call(9, 'caveat_validate', { source: 'not valid Caveat;' }));
    const invalid = (await client.wait(9)).result;
    assert.equal(invalid.isError, false);
    assert.equal(invalid.structuredContent.exitCode, 2);
    client.send(call(10, 'caveat_explain', { source: 'event refuse; on refuse reject "no";', events: [{ event: 'refuse' }] }));
    assert.equal((await client.wait(10)).result.structuredContent.report.events[0].outcome.outcome, 'rejected');
  } finally { assert.equal(await client.close(), 0); }
});

test('real authoring worker receives no unrelated secret or Node preload configuration', async () => {
  const names = ['CAVEAT_MCP_TEST_SECRET', 'NODE_OPTIONS', 'NODE_PATH'];
  const previous = new Map(names.map(name => [name, process.env[name]]));
  process.env.CAVEAT_MCP_TEST_SECRET = 'synthetic-test-value-not-a-credential';
  process.env.NODE_OPTIONS = '--trace-warnings'; // Harmless if inherited; no loader is executed.
  process.env.NODE_PATH = 'caveat-synthetic-unused-module-directory';
  let launches = 0;
  // Inspect the actual child environment before importing the real first-party
  // worker. The wrapper changes no tool input, report or runtime behavior.
  const inspectThenRun = `
    import assert from 'node:assert/strict';
    import { pathToFileURL } from 'node:url';
    for (const name of ${JSON.stringify(names)}) {
      assert.equal(Object.hasOwn(process.env, name), false, name + ' reached the worker');
    }
    await import(pathToFileURL(process.argv[1]).href);
  `;
  const client = connection({ spawnWorker(command, args, options) {
    launches++;
    assert.equal(command, process.execPath);
    assert.deepEqual(options.env, {});
    assert.equal(options.shell, false);
    assert.equal(options.windowsHide, true);
    assert.equal(args[0], '--max-old-space-size=128');
    assert.equal(args[1], fileURLToPath(new URL('../lib/authoring-worker.mjs', import.meta.url)));
    assert.equal(args[2], RUNTIME);
    return spawn(command, [args[0], '--input-type=module', '-e', inspectThenRun, ...args.slice(1)], options);
  } });
  try {
    await client.initialize();
    client.send(call(1));
    const result = (await client.wait(1)).result;
    assert.equal(result.isError, false, JSON.stringify(result));
    assert.equal(result.structuredContent.report.loads, true);
    assert.equal(result.structuredContent.exitCode, 0);
    assert.equal(launches, 1);
  } finally {
    try { assert.equal(await client.close(), 0); }
    finally {
      for (const [name, value] of previous) {
        if (value === undefined) delete process.env[name];
        else process.env[name] = value;
      }
    }
  }
});

test('a worker launch failure is reported and a later real call still succeeds', async () => {
  let launches = 0;
  const client = connection({ spawnWorker(command, args, options) {
    assert.deepEqual(options.env, {});
    if (launches++ === 0) throw new Error('injected spawn failure');
    return spawn(command, args, options);
  } });
  try {
    await client.initialize(); client.send(call(1));
    const failed = (await client.wait(1)).result;
    assert.equal(failed.isError, true);
    assert.equal(failed.structuredContent.error.kind, 'worker_failure');
    client.send(call(2));
    assert.equal((await client.wait(2)).result.isError, false);
  } finally { assert.equal(await client.close(), 0); }
});
test('bad requests, unsupported methods, unknown fields and virtual file escapes return errors without spawning', async () => {
  let spawns = 0;
  const client = connection({ spawnWorker() { spawns++; throw new Error('must not spawn'); } });
  try {
    await client.initialize();
    for (const [index, value, code] of [
      [1, request(1, 'resources/list'), -32601],
      [2, request(2, 'tools/call', { name: 'unknown', arguments: {} }), -32602],
      [3, call(3, 'caveat_validate', { source: SOURCE, path: 'private.cav' }), 'input'],
      [4, call(4, 'caveat_check', { source: SOURCE, strict: 'yes' }), 'input'],
      [5, call(5, 'caveat_test', { source: SOURCE, scenarios: { source: '../outside.cav', scenarios: [] } }), 'input'],
      [6, request(6, 'tools/list', { cursor: 'not-supported' }), -32602],
    ]) {
      client.send(value); const response = await client.wait(index);
      if (typeof code === 'number') assert.equal(response.error.code, code);
      else { assert.equal(response.result.isError, true); assert.equal(response.result.structuredContent.error.kind, code); }
    }
    client.send(call(7, 'caveat_explain', { source: SOURCE, events: Array.from({ length: 1001 }, () => ({ event: 'consult' })) }));
    assert.equal((await client.wait(7)).result.structuredContent.error.kind, 'limit');
    assert.equal(spawns, 0);
    const count = client.messages.length;
    for (const value of [notification('unknown'), notification('tools/call', { name: 'caveat_validate', arguments: { source: SOURCE } }), notification('notifications/cancelled', { requestId: 'absent' })]) client.send(value);
    client.send(request(8, 'ping')); await client.wait(8);
    assert.equal(client.messages.length, count + 1, 'notifications get no responses and do not run tools');
    client.send(request(8, 'ping'));
    await delay(20);
    assert.equal(client.messages.at(-1).error.code, -32600);
  } finally { assert.equal(await client.close(), 0); }
});

test('byte framing handles split UTF-8 and rejects invalid JSON/UTF-8/depth while staying usable', async () => {
  const client = connection();
  try {
    const bytes = Buffer.from(`${JSON.stringify(request('caf\u00e9', 'ping'))}\n`);
    const accent = bytes.indexOf(Buffer.from('\u00e9'));
    client.input.write(bytes.subarray(0, accent + 1)); client.input.write(bytes.subarray(accent + 1));
    assert.deepEqual((await client.wait('caf\u00e9')).result, {});
    for (const [bytes, code] of [[Buffer.from('{}\n'), -32600], [Buffer.from('not json\n'), -32700], [Buffer.from([0xff, 10]), -32700], [Buffer.from(`${'['.repeat(65)}0${']'.repeat(65)}\n`), -32600]]) {
      const before = client.messages.length;
      client.input.write(bytes);
      for (let n = 0; n < 100 && client.messages.length === before; n++) await delay(1);
      assert.equal(client.messages.at(-1).error.code, code);
    }
    client.send(request('healthy', 'ping'));
    assert.deepEqual((await client.wait('healthy')).result, {});
  } finally { assert.equal(await client.close(), 0); }
});

test('oversized and unterminated input lines end the connection with bounded output', async () => {
  for (const input of [Buffer.alloc(4 * 1024 * 1024 + 1, 65), Buffer.from('{')]) {
    const client = connection();
    client.input.end(input);
    assert.equal(await client.done, 1);
    assert.equal(client.messages.length, 1);
    assert.ok(JSON.stringify(client.messages[0]).length < 500);
    client.output.destroy();
  }
});

test('the request ID budget is bounded and requires reconnection after 4096 IDs', async () => {
  const client = connection();
  for (let id = 0; id <= 4096; id++) client.send(request(id, 'ping'));
  assert.equal(await client.done, 1);
  assert.equal(client.messages.length, 4097);
  assert.match(client.messages.at(-1).error.message, /reconnect/);
  client.output.destroy();
});

test('one worker runs at a time; cancellation suppresses its response and reaps before recovery', async () => {
  const fixture = injected('process.stdin.resume(); setInterval(() => {}, 1000);');
  const client = connection({ spawnWorker: fixture.spawnWorker });
  try {
    await client.initialize();
    client.send(call(1));
    client.send(call(2));
    assert.equal((await client.wait(2)).result.structuredContent.error.kind, 'busy');
    client.send(notification('notifications/cancelled', { requestId: 1, reason: 'test cancellation' }));
    await once(fixture.children[0], 'close');
    noLongerRunning(fixture.children[0]);
    client.send(call(3));
    assert.equal((await client.wait(3)).result.isError, false);
    assert.equal(client.messages.some(message => message.id === 1), false);
  } finally { assert.equal(await client.close(), 0); }
});

test('EOF kills and reaps the active worker and sends no cancelled result', async () => {
  const fixture = injected('process.stdin.resume(); setInterval(() => {}, 1000);');
  const client = connection({ spawnWorker: fixture.spawnWorker });
  await client.initialize();
  client.send(call(1));
  client.send(request(2, 'ping')); await client.wait(2);
  assert.equal(await client.close(), 0);
  noLongerRunning(fixture.children[0]);
  assert.equal(client.messages.some(message => message.id === 1), false);
});

test('injected crash, timeout, malformed output, stdout overflow and stderr overflow allow a fresh real call', async () => {
  for (const [code, expected] of [
    ['process.stdin.resume(); process.stdin.on("end", () => process.exit(17));', 'worker_output'],
    ['process.stdin.resume(); setInterval(() => {}, 1000);', 'timeout'],
    ['process.stdin.resume(); process.stdin.on("end", () => process.stdout.write("not JSON\\n"));', 'worker_output'],
    ['process.stdin.resume(); process.stdout.write("x".repeat(4*1024*1024+1)); setInterval(() => {},1000);', 'output_limit'],
    ['process.stdin.resume(); process.stderr.write("x".repeat(64*1024+1)); setInterval(() => {},1000);', 'stderr_limit'],
  ]) {
    const fixture = injected(code);
    const client = connection({ spawnWorker: fixture.spawnWorker, timeoutMs: 1500 });
    try {
      await client.initialize(); client.send(call(1));
      assert.equal((await client.wait(1)).result.structuredContent.error.kind, expected);
      noLongerRunning(fixture.children[0]);
      client.send(call(2));
      assert.equal((await client.wait(2)).result.isError, false);
    } finally { assert.equal(await client.close(), 0); }
  }
});

test('serialized MCP output is bounded even when duplication of a valid worker report would exceed 4 MiB', async () => {
  const fixture = injected(`let input=''; process.stdin.on('data',chunk=>input+=chunk); process.stdin.on('end',()=>{
    const value=JSON.parse(input); process.stdout.write(JSON.stringify({schema:'caveat-authoring/0.1',operation:'validate',
    sourceSha256:require('node:crypto').createHash('sha256').update(value.arguments.source).digest('hex'),runtime:{},exitCode:0,report:{text:'x'.repeat(2300000)}})); });`);
  const client = connection({ spawnWorker: fixture.spawnWorker });
  try {
    await client.initialize(); client.send(call(1));
    const response = await client.wait(1);
    assert.equal(response.result.structuredContent.error.kind, 'output_limit');
    assert.equal(response.result.isError, true);
    assert.ok(Buffer.byteLength(JSON.stringify(response)) < 4 * 1024 * 1024);
    client.send(call(2));
    const recovered = await client.wait(2);
    assert.equal(recovered.result.isError, false, JSON.stringify(recovered));
  } finally { assert.equal(await client.close(), 0); }
});


test('backpressured output cannot block cancellation or EOF cleanup', { timeout: 5000 }, async () => {
  for (const action of ['cancel', 'EOF']) {
    const fixture = injected('process.stdin.resume(); setInterval(() => {}, 1000);');
    const client = connection({ spawnWorker: fixture.spawnWorker, outputHighWaterMark: 1 });
    try {
      await client.initialize();
      client.output.pause();
      client.send(call(1)); client.send(request(2, 'ping'));
      for (let n = 0; n < 100 && !fixture.children.length; n++) await delay(1);
      assert.equal(fixture.children.length, 1);
      assert.equal(client.output.writableNeedDrain, true);
      const closed = once(fixture.children[0], 'close');
      if (action === 'cancel') client.send(notification('notifications/cancelled', { requestId: 1 }));
      else client.input.end();
      await closed;
      noLongerRunning(fixture.children[0]);
      if (action === 'cancel') {
        client.output.resume(); await client.wait(2);
        client.send(call(3)); assert.equal((await client.wait(3)).result.isError, false);
      } else {
        assert.equal(await client.done, 0, 'shutdown does not require the reader to resume');
      }
      assert.equal(client.messages.some(message => message.id === 1), false);
    } finally {
      client.output.resume(); assert.equal(await client.close(), 0);
    }
  }
});
