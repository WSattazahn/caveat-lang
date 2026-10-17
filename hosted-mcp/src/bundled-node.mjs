// Stands in for the package's lib/node.mjs in the Worker bundle (wrangler.toml
// aliases ./node.mjs here). A Worker has no runtime directory to read: the
// WebAssembly module is compiled at upload and imported with the bindings.
import * as bindings from 'caveat-lang/runtime/caveat_runtime.js';
import wasm from 'caveat-lang/runtime/caveat_runtime_bg.wasm';
import info from 'caveat-lang/runtime/build-info.json';
import { loadRuntime } from 'caveat-lang/session';
import { runtimeIdentity } from './identity.mjs';

const identity = runtimeIdentity(info);
// Instantiate while the isolate starts, which Workers do not count against a
// request's CPU time; loadRuntime then finds the bindings ready.
bindings.initSync({ module: wasm });

export function defaultRuntimeDirectory() { return 'bundled'; }

// The bindings exist once per isolate; each operation still opens its own
// session. A runtime that trapped is refused here and reported as a runtime error.
export function loadRuntimeFromDirectory() {
  return loadRuntime({ module: bindings, wasm, identity });
}
