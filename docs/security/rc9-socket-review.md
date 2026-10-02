# Socket capability review and worker environment hardening

Reviewed 2026-10-02 (America/Los_Angeles). This is a source and package review,
with a focused local regression result. It is not a Socket rescan, a complete
application security audit, or a declaration that historical alerts are fixed.

## Evidence identity

The reported version-history screenshot names **caveat-lang 0.1.0-rc.7** and
says **six new alerts**, but exposes only five labels. It shows **three metrics
improved** for rc.8. The version-history inspection did not expose finding IDs,
source positions or trigger expressions. Both version-specific public Alerts
views displayed **Unpopular package** under Quality at the time inspected.
Those are different observations; a version-history change summary is not an
inventory proving which findings remain or were resolved.

- [Inspected version history](https://socket.dev/npm/package/caveat-lang/versions/0.1.0-rc.7)
- [rc.7 public Alerts view](https://socket.dev/npm/package/caveat-lang/alerts/0.1.0-rc.7)
- [rc.8 public Alerts view](https://socket.dev/npm/package/caveat-lang/alerts/0.1.0-rc.8)
- [Socket's supply-chain risk guidance](https://docs.socket.dev/docs/supply-chain-risk)

Exact published tarballs compared locally:

| Version | SHA256 |
| --- | --- |
| `0.1.0-rc.7` | `0e441f896c58ba41f2741a82af40a17e74247a0c765169abda10b9ddffad5bbc` |
| `0.1.0-rc.8` | `75dab1f97a95774e4791303e38b484651379bfe922ede7805fa8dcf07828d021` |

Each tarball contained 95 files, with no added or removed file paths between
them. The bytes of all 31 JavaScript, Python and type-declaration files
(`.mjs`, `.js`, `.py`, `.mts`) matched. Documentation, package/build metadata and
WASM changed. In particular, the Node environment, filesystem, subprocess and
dynamic-loading code remained present in rc.8. The comparison inspects package
contents; it does not identify Socket's triggering expressions.

## Individual reported categories

The locations below are inspected source locations in the frozen **rc.7**
release, also present with matching bytes in **rc.8**. They are candidates for
understanding a category, not claimed Socket finding locations.

| Screenshot label | Inspected location and purpose | Trigger attribution and disposition |
| --- | --- | --- |
| Dynamic require | [`doctor.mjs`, lines 120–134](https://github.com/WSattazahn/caveat-lang/blob/v0.1.0-rc.7/kit/lib/doctor.mjs#L120) uses a locally declared boolean assertion named `require`; it is not CommonJS module loading. [`session.mjs`, lines 257 and 263](https://github.com/WSattazahn/caveat-lang/blob/v0.1.0-rc.7/kit/lib/session.mjs#L257) separately uses dynamic `import()` to load the selected runtime and replace a trapped instance. | Exact scanner trigger unavailable. Retain required runtime loading and review the selected runtime as trusted executable code. A clearer assertion name is reasonable, but no finding is declared a false positive or resolved without its location. |
| Environment variable access | [`doctor.mjs:88`](https://github.com/WSattazahn/caveat-lang/blob/v0.1.0-rc.7/kit/lib/doctor.mjs#L88) inspects PATH/PATHEXT for CLI ownership. [`mcp.mjs:136`](https://github.com/WSattazahn/caveat-lang/blob/v0.1.0-rc.7/kit/lib/mcp.mjs#L136) copied the parent's environment except `NODE_OPTIONS`/`NODE_PATH` to each worker. The Python example also reads its documented `CAVEAT_COMMAND` override. | Exact scanner trigger unavailable. Keep necessary CLI discovery/explicit caller configuration. Reduce the independent, unnecessary worker environment exposure in the rc.9 candidate as described below. No secret exfiltration was demonstrated. |
| Filesystem access | [`caveat.mjs:148`](https://github.com/WSattazahn/caveat-lang/blob/v0.1.0-rc.7/kit/bin/caveat.mjs#L148) reads selected programs/events; [`caveat.mjs:339`](https://github.com/WSattazahn/caveat-lang/blob/v0.1.0-rc.7/kit/bin/caveat.mjs#L339) writes starter files for `init`. [`node.mjs:31`](https://github.com/WSattazahn/caveat-lang/blob/v0.1.0-rc.7/kit/lib/node.mjs#L31) reads local WASM bytes. | Exact scanner trigger unavailable. These are intended CLI/loader capabilities. MCP tool arguments remain inline-only and cannot choose user file paths. This does not restrict all filesystem access by the Node process or a trusted runtime override. |
| Long strings | [`caveat.mjs:26`](https://github.com/WSattazahn/caveat-lang/blob/v0.1.0-rc.7/kit/bin/caveat.mjs#L26) contains a 3,979-character help literal. | Exact scanner trigger unavailable. Readable help text is an inspected example, not proof of what triggered this category. Do not remove useful help or declare the notice resolved merely because the version-history row changed. |
| Shell access | [`mcp.mjs:137–138`](https://github.com/WSattazahn/caveat-lang/blob/v0.1.0-rc.7/kit/lib/mcp.mjs#L137) launches a fixed Node worker using `shell: false`. [`caller.py:158`](https://github.com/WSattazahn/caveat-lang/blob/v0.1.0-rc.7/kit/examples/agent-evidence/caller.py#L158) launches the configured `caveat serve` command; its default `npx` launcher can itself use a shell. | Exact scanner trigger unavailable. Keep the bridge's fixed executable, fixed worker and no-shell launch. Process launching remains a capability and is not claimed to have disappeared. |
| Sixth alert (label not visible) | No name, finding ID, location or trigger supplied by the screenshot or recovered public view. | Unresolved attribution. Do not invent a sixth category, mark it resolved, or equate it with the current Unpopular package notice. |

The current **Unpopular package** display is a reported quality signal. Neither
this review nor the rc.9 implementation claims to change that scanner result.

## Independent rc.9 candidate change

The MCP worker reads bounded inline input, imports first-party authoring code,
and loads its explicit local runtime directory. Tracing those consumers found
no need for parent application environment variables. The bridge now passes
`env: {}` to `spawn`, retaining the fixed `process.execPath`, absolute worker
path, `shell: false`, hidden Windows window and existing process limits.

This changes what the bridge supplies, not the operating system's process
contract. On Windows, Node/libuv can insert standard process-support variables
such as SystemRoot, PATH, TEMP and user/profile values even with an empty
`env` option. That behavior was observed using Node 24.11.1 on Windows and is
visible in [libuv's Windows environment construction](https://github.com/libuv/libuv/blob/v1.51.0/src/win/process.c#L597).
The worker is still not an OS sandbox. The parent MCP server's startup
environment and a runtime override are controlled by the host.

The regression uses a synthetic unrelated secret and harmless Node option/path
values. A real child asserts their absence, then imports and executes the real
authoring worker. It does not print environment values or inject a loader.
The existing tests independently exercise all five operations, invalid input,
crashes, timeouts, cancellation/EOF, output limits, reaping and successful
subsequent calls. A launch-failure recovery case was added as well.

Local command on Windows, using the durable built runtime:

```sh
node --test kit/test/mcp.test.mjs
```

Result: **13 passed, 0 failed**, Node 24.11.1. Local receipt:
`test-results/rc9-hardening/mcp-local.tap`. This focused result is not a fresh
Rust/WASM build or a Linux result; both supported platform CI checks and the
normal release gates remain required before a candidate is called verified.

## Distinguish the verification scopes

rc.8's shipped runtime change rejects restored grounds outside saved state
lineage or frozen commitment bases; it does not remove the host capabilities
above. Its Lean laws and executable comparisons have the bounded scope in the
[rc.8 release record](../releases/v0.1.0-rc.8.md).

The retained rc.8 package audit reported no known npm or Rust dependency
advisories in its recorded scopes. The [dependency gate](../PACKAGE_SECURITY.md)
is not a Socket capability scan, source-security review, malware detector or
proof of absence of vulnerabilities. This worker hardening likewise supplies
no evidence that Socket has rescanned or cleared any reported finding.
