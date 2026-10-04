# Authoring bridge interface review

Reviewed October 1, 2026 before implementation. The owner authorized this as the
third adoption follow-up, after doctor/demo and onboarding. A separate reviewer
read the public protocol and existing kit APIs; this document records the
selected interface and acceptance requirements.

## Boundary

`caveat-lang mcp` is a local stdio server for five authoring operations. It reads
inline Caveat source supplied by its host. It exposes no user-file paths, URLs,
persistent sessions, save handles, host commands, networking or hypothetical
operation. Startup may select a trusted runtime with `--runtime DIRECTORY`;
individual tool calls cannot change it. It does not authenticate evidence or
permit external action.

The first implementation (rc.7) supported only the **2025-11-25 stdio profile**.
From rc.12 the bridge is dual-era: it also speaks the stateless
[2026-07-28 revision](https://modelcontextprotocol.io/specification/2026-07-28/changelog),
and keeps 2025-11-25 through that revision's twelve-month deprecation window.
Server capabilities are only `tools: {}` in both.

- **2026-07-28.** A request whose `_meta` carries
  `io.modelcontextprotocol/protocolVersion` is served statelessly, with no
  prior handshake. `io.modelcontextprotocol/clientCapabilities` is required;
  a missing or malformed field returns -32602. Methods are
  [`server/discover`](https://modelcontextprotocol.io/specification/2026-07-28/server/discover),
  tools/list and tools/call. `server/discover` answers with
  `supportedVersions` (`2026-07-28`, `2025-11-25`), capabilities and
  instructions; a version outside that list returns
  `UnsupportedProtocolVersionError` (-32022, `data.supported` and
  `data.requested`). Requests carrying `_meta` must name 2026-07-28, since
  2025-11-25 begins with initialize; only `server/discover` also accepts
  2025-11-25. Every result carries `resultType: "complete"` and
  `_meta["io.modelcontextprotocol/serverInfo"]`; `server/discover` and
  tools/list also carry `ttlMs: 3600000` and `cacheScope: "public"`. `ping`
  is not a 2026-07-28 method. No result is `input_required`: the bridge never
  asks the client for more input.
- **2025-11-25.** The
  [lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle)
  uses initialize, then notifications/initialized. Methods are initialize,
  ping, tools/list, tools/call and the initialized/cancelled notifications.
  Its responses are byte-for-byte what earlier releases sent. A client must
  accept the negotiated version or disconnect.

Both eras may share one connection. Unknown methods return method-not-found.
The request ID budget, single-flight rule, cancellation and limits below apply
to every request whatever its era. The tool list is the same, in the same
fixed order, under both.

## Tools

| Tool | Arguments |
| --- | --- |
| caveat_validate | source |
| caveat_check | source, optional strict boolean |
| caveat_test | source, scenarios object |
| caveat_explain | source, optional events array |
| caveat_dependents | source, subject, optional events array |

All argument objects reject unknown fields. `source` is text, `subject` is a
symbol name, and each event is `{event, payload?}`. Scenario data follows the
existing schema, using only the virtual source name `inline.cav`, including
any per-scenario source. No virtual file lookup may fall through to disk.

Existing library reports are retained under a `caveat-authoring/0.1` wrapper:
operation, sourceSha256, runtime identity, exitCode and report. Completed
operations retain CLI-style exit codes, including invalid source, strict-check
warnings and failed scenarios. Authored policy refusals stay recorded event
outcomes. Bridge failures carry a stable error kind and `isError: true`.
Results include structuredContent and the same JSON serialized as text, per
the [tools contract](https://modelcontextprotocol.io/specification/2025-11-25/server/tools).
Tools are read-only, non-destructive, idempotent and closed-world; these hints
do not prove the truth of an input or result.

## Resource and process ownership

These are bridge limits, not changes to language semantics:

- 4 MiB per incoming UTF-8 JSON-RPC line and serialized outgoing response.
- 1 MiB source; 1 MiB serialized event/scenario data; nesting at most 64.
- 1,000 explain/dependents events; 32 scenarios; 2,000 total scenario steps;
  1,000 expanded sends, counting repeat.
- One active tool child, no queue. Concurrent calls return a busy result.
- 10 seconds per tool, including runtime loading; 64 KiB captured child stderr.
- 4,096 request IDs per connection, after which reconnect is required.

A fresh child loads a fresh WASM instance for every tool call. Launch uses the
current Node executable and an absolute first-party worker path, without a
shell; Windows child windows stay hidden. Input travels over a pipe. Starting
with the rc.9 candidate, the bridge passes an empty environment object instead
of copying the parent's application environment. The worker needs no such
configuration: its input and runtime location are explicit. Node/libuv may add
platform support variables, notably standard Windows process, path, temporary
directory and profile entries. A real-child regression checks that an unrelated
synthetic secret and NODE_OPTIONS/NODE_PATH are absent. This is reduced exposure,
not a claim that the operating system supplies zero environment variables.
There is no shared runtime between calls. Timeout, cancellation, EOF, overflow or malformed worker output
kills and reaps that child. Node heap limits do not bound WASM memory or total
RSS: this is subprocess fault containment, not an operating-system sandbox.

The parent bounds UTF-8 framing before parsing, validates JSON-RPC single-object
envelopes and IDs, limits nesting, reserves stdout for protocol messages, and
sends no responses to notifications. Cancellation of the active call suppresses
its eventual response. Protocol errors use the standard parse/request/method/
parameter error codes. Input and result content are data, never instructions.
The [stdio transport](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)
and [cancellation contract](https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/cancellation)
define framing and cancellation behavior.

## Acceptance

Verify all five tools against direct existing APIs; actual installed stdio
startup/discovery/calls; malformed arguments/envelopes; every resource limit;
strict warnings, scenario failures, policy rejection and fatal dispatch;
virtual source path refusal; cancellation/EOF cleanup; and an injected worker
crash/timeout followed by a successful fresh call. Retain injected-fault labels.
No live desktop-host connection or public deployment is claimed by those tests.
