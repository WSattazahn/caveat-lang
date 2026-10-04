// Run after npm run build: every repository program's interface
// (spec/caveat-interface-0.1.md) is the same text from the native runtime and
// from both WASM builds, and a program that does not load gets the same load
// error from each.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const run = (command, args) => {
  const result = spawnSync(command, args, { cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
  if (result.error) throw result.error;
  return result;
};

const built = run('cargo', ['build', '--quiet', '--manifest-path', 'runtime/Cargo.toml', '--bin', 'caveat']);
assert.equal(built.status, 0, built.stderr);
const native = fileURLToPath(new URL(`../runtime/target/debug/caveat${process.platform === 'win32' ? '.exe' : ''}`, import.meta.url));

async function wasm(directory) {
  const namespace = await import(new URL(`../dist/${directory}/caveat_runtime.js`, import.meta.url).href);
  await namespace.default({ module_or_path: await readFile(new URL(`../dist/${directory}/caveat_runtime_bg.wasm`, import.meta.url)) });
  assert.equal(typeof namespace.WebReactiveSession.interface, 'function', `${directory} has no interface export`);
  return namespace.WebReactiveSession;
}
const builds = { full: await wasm('pkg'), reactive: await wasm('pkg-reactive') };

const files = run('git', ['ls-files', '*.cav']).stdout.split('\n').filter(Boolean);
let loaded = 0;
let refused = 0;
for (const file of files) {
  // The bundle the CLI loads: imports resolved, a file without them unchanged.
  const linked = run(native, ['--link', file]);
  if (linked.status !== 0) continue;
  const expected = run(native, ['--interface', file]);
  for (const [name, Session] of Object.entries(builds)) {
    let text;
    let error;
    try { text = Session.interface(linked.stdout); } catch (thrown) { error = String(thrown?.message ?? thrown); }
    if (expected.status === 0) {
      assert.equal(error, undefined, `${file}: ${name} refused what native loads`);
      assert.equal(`${text}\n`, expected.stdout, `${file}: ${name} differs from native`);
      assert.equal(Session.interface(linked.stdout), text, `${file}: ${name} is not deterministic`);
      assert.equal(JSON.parse(text).schema, 'caveat-interface/0.1');
    } else {
      assert.equal(`load error: ${error}\n`, expected.stderr, `${file}: ${name} load error differs from native`);
    }
  }
  if (expected.status === 0) loaded += 1; else refused += 1;
}
assert(loaded >= 100, `only ${loaded} programs loaded`);
console.log(`interface: ${loaded} programs identical on native and both WASM builds; ${refused} that do not load refused alike`);
