import { createHash } from 'node:crypto';
import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';

const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const artifactNames = [
  'pkg/caveat_runtime.js', 'pkg/caveat_runtime_bg.wasm',
  'pkg-reactive/caveat_runtime.js', 'pkg-reactive/caveat_runtime_bg.wasm',
];

async function sourceFiles(root, directory) {
  const result = [];
  for (const entry of await readdir(path.join(root, directory), { withFileTypes: true })) {
    const name = directory + '/' + entry.name;
    if (entry.isDirectory()) result.push(...await sourceFiles(root, name));
    else if (entry.isFile()) result.push(name);
    else throw new Error('Unsupported runtime source entry: ' + name);
  }
  return result;
}

// Content identity for this repository's local WASM build inputs. This is a
// build receipt, not an authenticated attestation or a proof of compilation.
export async function currentRuntimeFingerprint(root) {
  const files = [
    ...await sourceFiles(root, 'runtime/src'),
    'runtime/Cargo.toml', 'runtime/Cargo.lock', 'runtime/prelude.cav',
    'rust-toolchain.toml', 'game/the_door_round2.cav', 'web/the_door_round2.cav',
    'scripts/build-web.mjs', 'scripts/runtime-build-fingerprint.mjs',
  ];
  for (const name of ['runtime/build.rs', '.cargo/config', '.cargo/config.toml',
    'runtime/.cargo/config', 'runtime/.cargo/config.toml']) {
    try { await readFile(path.join(root, name)); files.push(name); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
  const inputs = [];
  for (const name of files.sort()) inputs.push([name, sha256(await readFile(path.join(root, name)))]);
  return sha256(JSON.stringify(inputs));
}

export async function runtimeArtifactHashes(dist) {
  const hashes = {};
  for (const name of artifactNames) hashes[name] = sha256(await readFile(path.join(dist, name)));
  return hashes;
}

export async function assertFreshRuntime(root) {
  const dist = path.join(root, 'dist');
  const info = JSON.parse(await readFile(path.join(dist, 'build-info.json'), 'utf8'));
  if (info.compiled !== true) throw new Error('A compiled runtime is required; run npm run build');
  if (info.runtimeSourceFingerprint !== await currentRuntimeFingerprint(root)) {
    throw new Error('WASM build inputs are stale or unrecorded; run npm run build');
  }
  const artifacts = await runtimeArtifactHashes(dist);
  if (!info.runtimeArtifacts || Object.keys(info.runtimeArtifacts).length !== artifactNames.length
    || artifactNames.some(name => info.runtimeArtifacts[name] !== artifacts[name])) {
    throw new Error('WASM output bytes differ from their build receipt; run npm run build');
  }
  return info;
}
