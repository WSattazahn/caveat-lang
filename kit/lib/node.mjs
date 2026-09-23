// Node loading for the session library: a runtime directory holds the
// wasm-bindgen output caveat_runtime.js and caveat_runtime_bg.wasm, as built
// into dist/pkg-reactive by `npm run build`.
import { existsSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { loadRuntime } from './session.mjs';

const kitRoot = fileURLToPath(new URL('../', import.meta.url));

// The packaged kit will carry its own runtime/. Until then the kit uses the
// repository's reactive build.
export function defaultRuntimeDirectory() {
  const bundled = path.join(kitRoot, 'runtime');
  if (existsSync(path.join(bundled, 'caveat_runtime_bg.wasm'))) return bundled;
  return path.join(kitRoot, '..', 'dist', 'pkg-reactive');
}

let instances = 0;

// Each call imports a separate module instance, so a runtime that trapped can
// be replaced by a fresh one.
export async function loadRuntimeFromDirectory(directory = defaultRuntimeDirectory()) {
  const wasmPath = path.join(directory, 'caveat_runtime_bg.wasm');
  const modulePath = path.join(directory, 'caveat_runtime.js');
  if (!existsSync(wasmPath) || !existsSync(modulePath)) {
    throw new Error(`no Caveat runtime in ${directory}; run npm run build or pass --runtime`);
  }
  const wasm = await readFile(wasmPath);
  // No local paths: reports that carry this identity may be committed.
  const identity = { reactiveWasmSha256: createHash('sha256').update(wasm).digest('hex') };
  // A packed kit keeps build-info.json beside the runtime; the repository
  // build keeps it one level up, in dist/.
  const buildInfo = [path.join(directory, 'build-info.json'), path.join(directory, '..', 'build-info.json')].find(existsSync);
  if (buildInfo) {
    try {
      const info = JSON.parse(await readFile(buildInfo, 'utf8'));
      Object.assign(identity, { revision: info.revision, clean: info.clean, compiled: info.compiled, host: info.host });
    } catch { /* identity stays partial */ }
  }
  instances += 1;
  const module = `${pathToFileURL(modulePath).href}?instance=${instances}`;
  return loadRuntime({ module, wasm, identity });
}
