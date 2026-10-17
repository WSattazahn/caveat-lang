// The runtime identity `caveat-lang mcp` reports, from the package's own
// build record: its reactive WebAssembly hash is the file the Worker bundles.
export function runtimeIdentity(info) {
  return {
    reactiveWasmSha256: info.runtimeArtifacts['pkg-reactive/caveat_runtime_bg.wasm'],
    revision: info.revision, clean: info.clean, compiled: info.compiled, host: info.host,
  };
}
