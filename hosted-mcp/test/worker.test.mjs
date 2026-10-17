// The bundled Worker in local workerd answers as the published stdio server.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { after, before, test } from 'node:test';
import { unstable_dev } from 'wrangler';
import { runAuthoringOperation } from '../node_modules/caveat-lang/lib/authoring.mjs';

const pkg = new URL('../node_modules/caveat-lang/', import.meta.url);
const source = await readFile(new URL('templates/umbrella.cav', pkg), 'utf8');
const scenarios = { ...JSON.parse(await readFile(new URL('templates/umbrella.scenarios.json', pkg), 'utf8')), source: 'inline.cav' };
const events = (await readFile(new URL('templates/events.jsonl', pkg), 'utf8')).trim().split('\n').map(line => JSON.parse(line));
let worker;

before(async () => {
  worker = await unstable_dev('src/worker.mjs', { config: 'wrangler.toml', logLevel: 'error', experimental: { disableExperimentalWarning: true } });
});
after(async () => { await worker?.stop(); });

test('every tool returns the Node result from the bundled runtime', async () => {
  for (const [name, args] of [
    ['caveat_validate', { source }], ['caveat_check', { source, strict: true }], ['caveat_test', { source, scenarios }],
    ['caveat_explain', { source, events }], ['caveat_dependents', { source, subject: 'forecast', events }],
  ]) {
    const response = await worker.fetch('/mcp', { method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'tools/call', params: { name, arguments: args } }) });
    assert.equal(response.status, 200, name);
    const { result } = await response.json();
    assert.equal(result.isError, false, name);
    assert.deepEqual(result.structuredContent, await runAuthoringOperation(name, args), name);
  }
});

test('the server identifies the published package', async () => {
  const response = await worker.fetch('/mcp', { method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'initialize', params: { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'test', version: '1' } } }) });
  const { result } = await response.json();
  const { version } = JSON.parse(await readFile(new URL('package.json', pkg), 'utf8'));
  assert.deepEqual(result.serverInfo, { name: 'caveat-lang', version, title: 'CAVEAT Language' });
  assert.equal(result.protocolVersion, '2025-06-18');
});
