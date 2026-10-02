// Interoperability through the pinned official client, against an installed kit.
// Client dependencies are test-only; nothing is added to the shipped package.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { copyFile, mkdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const fixture = fileURLToPath(new URL('./fixtures/mcp-client/', import.meta.url));
export async function checkMcpClient({ installed, directory }) {
  await mkdir(directory, { recursive: true });
  for (const file of ['package.json', 'package-lock.json']) await copyFile(path.join(fixture, file), path.join(directory, file));
  const installedDeps = process.platform === 'win32'
    ? spawnSync('npm ci --ignore-scripts --no-audit --no-fund', { cwd: directory, encoding: 'utf8', shell: true })
    : spawnSync('npm', ['ci', '--ignore-scripts', '--no-audit', '--no-fund'], { cwd: directory, encoding: 'utf8' });
  assert.equal(installedDeps.status, 0, installedDeps.stdout + installedDeps.stderr);
  const sdk = path.join(directory, 'node_modules', '@modelcontextprotocol', 'sdk');
  const { Client } = await import(pathToFileURL(path.join(sdk, 'dist/esm/client/index.js')).href);
  const { StdioClientTransport } = await import(pathToFileURL(path.join(sdk, 'dist/esm/client/stdio.js')).href);
  const client = new Client({ name: 'caveat-package-check', version: '1.0.0' }, { capabilities: {} });
  const transport = new StdioClientTransport({ command: process.execPath,
    args: [path.join(installed, 'bin', 'caveat.mjs'), 'mcp'], cwd: directory, stderr: 'pipe' });
  let stderr = '';
  transport.stderr.on('data', chunk => { stderr = (stderr + chunk).slice(-65536); });
  const source = await readFile(path.join(installed, 'templates', 'umbrella.cav'), 'utf8');
  const scenarios = JSON.parse(await readFile(path.join(installed, 'templates', 'umbrella.scenarios.json'), 'utf8'));
  scenarios.source = 'inline.cav';
  const events = (await readFile(path.join(installed, 'templates', 'events.jsonl'), 'utf8')).trim().split(/\r?\n/).map(line => JSON.parse(line));
  const names = ['caveat_validate', 'caveat_check', 'caveat_test', 'caveat_explain', 'caveat_dependents'];
  const calls = [];
  try {
    await client.connect(transport, { timeout: 15000 });
    const listed = await client.listTools();
    assert.deepEqual(listed.tools.map(tool => tool.name).sort(), [...names].sort());
    assert.deepEqual(client.getServerCapabilities(), { tools: {} });
    for (const [name, args] of [
      ['caveat_validate', { source }], ['caveat_check', { source }],
      ['caveat_test', { source, scenarios }], ['caveat_explain', { source, events }],
      ['caveat_dependents', { source, subject: 'rain_chance', events }],
    ]) {
      const result = await client.callTool({ name, arguments: args }, undefined, { timeout: 15000 });
      assert.notEqual(result.isError, true, JSON.stringify(result));
      const value = result.structuredContent;
      assert.equal(value.schema, 'caveat-authoring/0.1');
      assert.equal(value.exitCode, 0, JSON.stringify(value));
      assert.deepEqual(JSON.parse(result.content[0].text), value);
      if (name === 'caveat_test') assert.equal(value.report.failed, 0);
      if (name === 'caveat_explain') {
        assert.equal(value.report.decisions[0].revisions[0].status, 'reopened');
        assert.deepEqual(value.report.decisions[0].revisions[0].grounds.evidence, ['rain_chance@1']);
      }
      calls.push({ name, schema: value.report.schema, exitCode: value.exitCode });
    }
    const badSource = await client.callTool({ name: 'caveat_validate', arguments: { source: 'invalid caveat;' } });
    assert.equal(badSource.structuredContent.exitCode, 2);
    const forbidden = await client.callTool({ name: 'caveat_test', arguments: {
      source, scenarios: { ...scenarios, source: '../secret.cav' },
    } });
    assert.equal(forbidden.isError, true);
    const retry = await client.callTool({ name: 'caveat_validate', arguments: { source } });
    assert.equal(retry.structuredContent.exitCode, 0);
    const metadata = JSON.parse(await readFile(path.join(sdk, 'package.json'), 'utf8'));
    return { client: `${metadata.name}@${metadata.version}`, server: client.getServerVersion(),
      calls, invalidSource: true, virtualPathRefused: true, retry: true, stderr };
  } finally { await client.close(); }
}
