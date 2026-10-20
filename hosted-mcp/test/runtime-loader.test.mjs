// The Worker replaces a runtime instance that trapped instead of refusing
// every later call until Cloudflare recycles the isolate.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import { loadRuntime } from '../node_modules/caveat-lang/lib/session.mjs';
import { factorySource } from '../scripts/runtime-factory.mjs';
import { createBindings } from '../src/generated/runtime-factory.mjs';
import { runtimeIdentity } from '../src/identity.mjs';
import { createRuntimeLoader } from '../src/runtime-loader.mjs';

const pkg = new URL('../node_modules/caveat-lang/', import.meta.url);
const wasm = new WebAssembly.Module(await readFile(new URL('runtime/caveat_runtime_bg.wasm', pkg)));
const identity = runtimeIdentity(JSON.parse(await readFile(new URL('runtime/build-info.json', pkg), 'utf8')));
const SOURCE = await readFile(new URL('templates/umbrella.cav', pkg), 'utf8');

function sequenceAfterOneEvent(runtime) {
  const session = runtime.open(SOURCE);
  try {
    const event = session.snapshot().events?.[0] ?? null;
    return { loads: true, event, sequence: session.snapshot().sequence };
  } finally { session.close(); }
}

test('each factory call gives bindings with their own instance', () => {
  const first = createBindings();
  const second = createBindings();
  assert.notEqual(first.WebReactiveSession, second.WebReactiveSession);
  first.initSync({ module: wasm });
  second.initSync({ module: wasm });
  assert.notEqual(first.initSync({ module: wasm }), second.initSync({ module: wasm }));
});

test('a trapped runtime is replaced on the next load, and only then', async () => {
  const loader = createRuntimeLoader({ createBindings, wasm, identity, loadRuntime });
  const first = await loader.load();
  assert.deepEqual(first.identity, identity);
  const before = sequenceAfterOneEvent(first);
  const again = await loader.load();
  assert.equal(loader.replaced, 0);
  assert.equal(again.trapped, false);

  first.markTrapped();
  assert.throws(() => first.open(SOURCE), /trapped/);
  const fresh = await loader.load();
  assert.equal(loader.replaced, 1);
  assert.equal(fresh.trapped, false);
  assert.deepEqual(sequenceAfterOneEvent(fresh), before);
  assert.throws(() => again.open(SOURCE), /trapped/, 'runtimes of the trapped instance stay refused');
});

test('the generator refuses glue it does not understand', () => {
  assert.throws(() => factorySource('import x from "y";\nexport default x;\n'), /imports a module/);
  assert.throws(() => factorySource('export * from "y";\n'), /unexpected export form/);
  assert.throws(() => factorySource('function f() {}\nexport { f as initSync };\nexport default f;\n'), /does not export WebReactiveSession/);
});
