// The Glowcap adapter as published, pointed somewhere else. It imports the
// runtime from ../../../dist/pkg-reactive/ and its program from beside itself;
// the benchmark rewrites exactly those three locations and nothing else, so
// the code that runs is the adapter's own.

export const ADAPTER_LOCATIONS = Object.freeze({
  glue: "'../../../dist/pkg-reactive/caveat_runtime.js'",
  wasm: "new URL('../../../dist/pkg-reactive/caveat_runtime_bg.wasm', import.meta.url)",
  program: "new URL('./glowcap.cav', import.meta.url)",
});

// glueUrl, wasmUrl, programUrl: file: URLs. Throws if any location is not
// found exactly once, which means the adapter changed.
export function rewriteAdapter(text, { glueUrl, wasmUrl, programUrl }) {
  const substitutions = [
    [ADAPTER_LOCATIONS.glue, JSON.stringify(glueUrl)],
    [ADAPTER_LOCATIONS.wasm, `new URL(${JSON.stringify(wasmUrl)})`],
    [ADAPTER_LOCATIONS.program, `new URL(${JSON.stringify(programUrl)})`],
  ];
  let rewritten = text;
  for (const [from, to] of substitutions) {
    const count = rewritten.split(from).length - 1;
    if (count !== 1) throw new Error(`adapter: expected ${from} once, found ${count}; update lib/adapter.mjs`);
    rewritten = rewritten.replace(from, () => to);
  }
  return rewritten;
}
