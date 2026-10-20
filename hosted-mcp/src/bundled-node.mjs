// Stands in for the package's lib/node.mjs in the Worker bundle (wrangler.toml
// aliases ./node.mjs here). A Worker has no runtime directory to read: the
// WebAssembly module is compiled at upload and imported with the bindings.
import wasm from 'caveat-lang/runtime/caveat_runtime_bg.wasm';
import info from 'caveat-lang/runtime/build-info.json';
import { loadRuntime } from 'caveat-lang/session';
import { createBindings } from './generated/runtime-factory.mjs';
import { runtimeIdentity } from './identity.mjs';
import { createRuntimeLoader } from './runtime-loader.mjs';

const loader = createRuntimeLoader({ createBindings, wasm, identity: runtimeIdentity(info), loadRuntime });

export function defaultRuntimeDirectory() { return 'bundled'; }

// The bindings exist once per isolate; each operation still opens its own
// session. After a trap the next operation gets a fresh instance.
export function loadRuntimeFromDirectory() {
  return loader.load();
}
