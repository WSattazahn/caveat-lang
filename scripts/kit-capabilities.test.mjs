import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { CAPABILITIES, DECLARED_CAPABILITIES, documentedCapabilities, scanCapabilities } from './kit-capabilities.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const kit = path.join(root, 'kit');

// The committed kit sources plus the generated runtime loader, which is only
// staged into kit/runtime while packing; its fetch( is stood in for here.
async function packedSources() {
  const manifest = JSON.parse(await readFile(path.join(kit, 'package.json'), 'utf8'));
  assert.ok(manifest.files.includes('examples/'), 'examples/ stays in the published files (rc11-development.md 4d)');
  const files = new Map([['runtime/caveat_runtime.js', 'module_or_path = fetch(module_or_path);\n']]);
  for (const name of CAPABILITIES) {
    for (const file of DECLARED_CAPABILITIES[name]) {
      if (!files.has(file)) files.set(file, await readFile(path.join(kit, file), 'utf8'));
    }
  }
  return files;
}

test('the documented table is the declared table', async () => {
  const documented = documentedCapabilities(await readFile(path.join(root, 'docs/PACKAGE_SECURITY.md'), 'utf8'));
  for (const name of CAPABILITIES) assert.deepEqual(documented[name], [...DECLARED_CAPABILITIES[name]].sort(), name);
});

test('the declared files use exactly their declared capabilities', async () => {
  const result = scanCapabilities(await packedSources());
  assert.deepEqual(result.failures, []);
});

test('an undeclared or stale capability fails the scan', async () => {
  const cases = [
    [files => files.set('lib/session.mjs', "import { readFileSync } from 'node:fs';\n"), /lib\/session\.mjs uses filesystem but is not declared/],
    [files => files.set('lib/explain.mjs', "import { spawn } from 'child_process';\n"), /lib\/explain\.mjs uses subprocess/],
    [files => files.set('lib/check.mjs', 'const debug = process.env.DEBUG;\n'), /lib\/check\.mjs uses environment/],
    [files => files.set('lib/serve.mjs', "import http from 'node:http';\n"), /lib\/serve\.mjs uses network/],
    [files => files.set('lib/scenarios.mjs', 'await fetch (url);\n'), /lib\/scenarios\.mjs uses network/],
    [files => files.set('examples/new/client.py', 'import urllib.request\n'), /client\.py uses network/],
    [files => files.set('examples/new/run.py', 'subprocess.run(["caveat"])\n'), /run\.py uses subprocess/],
    [files => files.set('lib/doctor.mjs', '// no capabilities\n'), /lib\/doctor\.mjs is declared for filesystem but does not use it/],
  ];
  for (const [mutate, expected] of cases) {
    const files = await packedSources();
    mutate(files);
    const result = scanCapabilities(files);
    assert.equal(result.passed, false, String(expected));
    assert.match(result.failures.join('\n'), expected);
  }
});

test('documentation and data files are not scanned', () => {
  const files = new Map([['docs/MCP.md', "fetch(url); process.env.X; require('child_process')"], ['templates/events.jsonl', '{"fs":"node:fs"}']]);
  const declared = Object.fromEntries(CAPABILITIES.map(name => [name, []]));
  assert.deepEqual(scanCapabilities(files, declared).failures, []);
});
