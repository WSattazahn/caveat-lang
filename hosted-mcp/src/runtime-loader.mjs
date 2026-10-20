// Hands each operation a runtime, replacing one that trapped. A trap leaves a
// WebAssembly instance in an unknown state, and the package's loadRuntime can
// only replace it given a module URL, which a bundle has none of. Here a fresh
// copy of the bindings (src/generated/runtime-factory.mjs) is instantiated from
// the same compiled module, with its own memory; the trapped one is dropped.
export function createRuntimeLoader({ createBindings, wasm, identity, loadRuntime }) {
  let bindings = createBindings();
  // Instantiated now, which is isolate start-up in a Worker.
  bindings.initSync({ module: wasm });
  let latest = null;
  let replaced = 0;
  return {
    async load() {
      if (latest?.trapped) {
        bindings = createBindings();
        bindings.initSync({ module: wasm });
        replaced += 1;
      }
      latest = await loadRuntime({ module: bindings, wasm, identity });
      return latest;
    },
    // How many trapped instances were replaced; for tests.
    get replaced() { return replaced; },
  };
}
