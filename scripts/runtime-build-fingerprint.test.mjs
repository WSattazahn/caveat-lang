import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { assertFreshRuntime, currentRuntimeFingerprint, runtimeArtifactHashes } from './runtime-build-fingerprint.mjs';

async function fixture(t) {
  const root = await mkdtemp(path.join(tmpdir(), 'caveat-build-inputs-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const names = [
    'runtime/src/lib.rs', 'runtime/Cargo.toml', 'runtime/Cargo.lock', 'runtime/prelude.cav',
    'rust-toolchain.toml', 'game/the_door_round2.cav', 'web/the_door_round2.cav',
    'scripts/build-web.mjs', 'scripts/runtime-build-fingerprint.mjs',
    'dist/pkg/caveat_runtime.js', 'dist/pkg/caveat_runtime_bg.wasm',
    'dist/pkg-reactive/caveat_runtime.js', 'dist/pkg-reactive/caveat_runtime_bg.wasm',
  ];
  for (const name of names) {
    await mkdir(path.dirname(path.join(root, name)), { recursive: true });
    await writeFile(path.join(root, name), name);
  }
  const info = { compiled: true, runtimeSourceFingerprint: await currentRuntimeFingerprint(root),
    runtimeArtifacts: await runtimeArtifactHashes(path.join(root, 'dist')) };
  await writeFile(path.join(root, 'dist/build-info.json'), JSON.stringify(info));
  return { root, info };
}

test('accepts a compiled receipt only while source and output bytes match', async t => {
  const { root, info } = await fixture(t);
  assert.deepEqual(await assertFreshRuntime(root), info);
  await writeFile(path.join(root, 'runtime/src/new.rs'), 'new module');
  await assert.rejects(assertFreshRuntime(root), /inputs are stale/);
});

test('detects runtime, embedded-source, lockfile and build-logic changes', async t => {
  const { root } = await fixture(t);
  for (const name of ['runtime/src/lib.rs', 'game/the_door_round2.cav',
    'web/the_door_round2.cav', 'runtime/Cargo.lock', 'scripts/build-web.mjs',
    'scripts/runtime-build-fingerprint.mjs']) {
    const file = path.join(root, name);
    const original = await readFile(file);
    await writeFile(file, 'changed');
    await assert.rejects(assertFreshRuntime(root), /inputs are stale/, name);
    await writeFile(file, original);
  }
  await mkdir(path.join(root, '.cargo'));
  await writeFile(path.join(root, '.cargo/config.toml'), 'new build options');
  await assert.rejects(assertFreshRuntime(root), /inputs are stale/);
});

test('rejects replaced output bytes even with unchanged source receipt', async t => {
  const { root } = await fixture(t);
  await writeFile(path.join(root, 'dist/pkg-reactive/caveat_runtime_bg.wasm'), 'stale wasm');
  await assert.rejects(assertFreshRuntime(root), /output bytes differ/);
});

test('refuses assemble-only and older or incomplete receipts', async t => {
  const { root, info } = await fixture(t);
  for (const replacement of [
    { ...info, compiled: false }, { compiled: true },
    { ...info, runtimeArtifacts: {} },
  ]) {
    await writeFile(path.join(root, 'dist/build-info.json'), JSON.stringify(replacement));
    await assert.rejects(assertFreshRuntime(root), /compiled runtime|inputs are stale|output bytes differ/);
  }
});
