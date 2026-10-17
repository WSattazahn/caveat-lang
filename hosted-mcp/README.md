# Hosted Caveat MCP server

The `caveat-lang` MCP tools (`caveat_validate`, `caveat_check`, `caveat_test`,
`caveat_explain`, `caveat_dependents`) over Streamable HTTP, as a Cloudflare
Worker at `https://caveat-mcp.waltersattazahn.workers.dev/mcp`. ChatGPT and
Codex reach it through the Caveat plugin (`plugins/caveat/.codex-plugin/mcp.json`).

It serves the published npm package, pinned in `package.json`, not this
checkout: its tools, schemas, argument checks, operations and WebAssembly
runtime are the package's own files. Answers match `npx caveat-lang mcp` over
stdio; `test/handler.test.mjs` checks that side by side, and
`test/worker.test.mjs` runs the bundled Worker in local workerd.

## How it differs from stdio

- Stateless HTTP: each POST carries one JSON-RPC message and gets one JSON
  response. There is no session ID, no server-to-client stream (GET answers
  405) and no batching. Tools can be listed and called without `initialize`.
- `initialize` answers with the version the client asks for when it is
  2025-11-25, 2025-06-18 or 2025-03-26, else 2025-11-25. 2026-07-28 requests
  carry their version in `_meta`, as over stdio.
- Every call opens a fresh session, but the WebAssembly instance lives as long
  as the isolate. If it traps, calls report a runtime error until Cloudflare
  replaces the isolate.
- Limits: 4 MiB per request and response, the package's argument limits, and
  60 requests a minute per IP address in each Cloudflare location (a rate
  limiting binding, best effort). `caveat_test` runs at most 16 scenario steps
  per call, counting repeated sends (`testSteps` in `src/worker.mjs`), to stay
  within the free plan's CPU; larger files get a `limit` error naming the local
  server. Cloudflare stops a call that runs out of CPU.
- Nothing is stored or logged; see `web/plugin-privacy.html` and
  `web/plugin-terms.html`.

## Free plan fit

The free plan allows 3 MB compressed script and 10 ms CPU per request.

- Bundle: 3,247 KiB, 866 KiB gzip (`npm run check`).
- CPU (`npm run measure`, Node, one runtime per process as in the Worker):
  once warm, validate, check, explain and dependents take under 1 ms each for
  the starter program, and `caveat_test` with the starter's two scenarios
  (12 steps) about 6 ms, 7.4 ms at the 95th percentile. The first call of a
  kind in a new isolate is slower (about 40 ms for `caveat_test`, under 10 ms
  for the rest) and can exceed the free limit, as can larger scenario files.
  The runtime is instantiated when the isolate starts, which is not counted
  against a request. The deployed Worker's real CPU times are in the
  Cloudflare dashboard.

## Commands

```
npm ci
npm test
npm run measure
npm run check       # bundle without deploying
npx wrangler deploy # needs wrangler login on the owner's account
```

To serve a new release, change the `caveat-lang` version in `package.json`,
run `npm install`, `npm test` and deploy.
