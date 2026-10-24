# MCP authoring tools

<!-- caveat-package:identity -->
Package: **`caveat-lang@1.3.0`**.
<!-- /caveat-package:identity -->

This package exposes existing Caveat authoring operations to a local MCP
host. No MCP desktop host is installed by the release or development process.
The command was introduced in rc.7; historical rc.6 does not include it.
The bridge speaks MCP **2026-07-28** and, through that revision's deprecation
window, the **2025-11-25** profile. A 2026-07-28 client may call
`server/discover` first, then send `tools/list` and `tools/call` with
`io.modelcontextprotocol/protocolVersion` and
`io.modelcontextprotocol/clientCapabilities` in each request's `_meta`; no
`initialize` is needed. Each of those results carries `resultType: "complete"`
and `io.modelcontextprotocol/serverInfo`, and an unsupported version returns
`UnsupportedProtocolVersionError` (-32022) listing both supported versions. A
2025-11-25 client initializes, sends `notifications/initialized`, then lists
and calls tools exactly as in earlier releases (rc.12 added 2026-07-28).

Configure your host to run Node with the absolute path to your installed CLI:

```json
{
  "mcpServers": {
    "caveat-language": {
      "command": "node",
      "args": ["/absolute/project/node_modules/caveat-lang/bin/caveat.mjs", "mcp"]
    }
  }
}
```

Replace that path with the actual installation (Windows JSON paths can use
forward slashes). This is a host configuration example, not an automatic
installation. `node` must resolve to Node 20 or newer. Either protocol works
from the same configuration.
The process writes protocol messages on stdout and no startup banner.

## Available tools

| Tool | Inputs | Result |
| --- | --- | --- |
| caveat_validate | source | Whether the program loads and what it declares |
| caveat_check | source, optional strict | Existing diagnostics and suppressed warnings |
| caveat_test | source, scenarios | Existing scenario results and invariant checks, with the program's check warnings under `check` (rc.15) |
| caveat_explain | source, optional events | Decisions, exact grounds, caveats and history |
| caveat_dependents | source, subject, optional events | What depends on the named evidence, stream or caveat |

Choose `caveat_validate` for a load/declaration check, `caveat_check` for static
authoring diagnostics, and `caveat_test` to assert behavior after events. A
successful load or a clean check does not establish that a scenario passes.
Use `caveat_explain` to understand the resulting decisions and their reasons;
use `caveat_dependents` to trace the impact of one named evidence item, reading
stream, reading record or caveat. Each call starts fresh, so supply the event
history again when moving from an explanation to a dependency query.

`strict` defaults to `false`. With `true`, any unsuppressed diagnostic makes
`caveat_check` return `exitCode: 1`; suppressed diagnostics do not count. The
check does not send events. `caveat_test` checks the program first and stops
before scenarios if it finds an error-severity diagnostic; advisory diagnostics
are included in its report and do not stop the scenarios.

`source` is inline program text. Events are objects with `event` and optional
`payload`. `scenarios` is a [scenario document](reference/spec/caveat-scenarios-0.1.md)
whose source, including per-scenario overrides, must be `inline.cav`. The bridge
has no user-file lookup: the supplied text is its only virtual source file.
Unknown argument fields are rejected.

For example, this tool name and arguments check an event's effect. Wrap them
in your protocol's `tools/call` request; for 2026-07-28, also add the required
`_meta` fields inside the request's `params` as described above:

```json
{
  "name": "caveat_test",
  "arguments": {
    "source": "state count = 0; event increment; on increment set count = count + 1;",
    "scenarios": {
      "schema": "caveat-scenarios/0.1",
      "source": "inline.cav",
      "scenarios": [{
        "id": "increments",
        "title": "An event increments the count",
        "steps": [
          { "send": "increment" },
          { "expect": { "/values/count": 1 } }
        ]
      }]
    }
  }
}
```

For `caveat_explain` or `caveat_dependents`, `events` defaults to `[]` (the
initial state). Entries run in order; `event` names a declared event and
`payload` supplies its declared parameters, defaulting to `{}`. A rejected
event is recorded and replay continues; a fatal event ends replay at the last
good snapshot. Inspect each reported outcome. For `caveat_dependents`,
`subject` is an evidence name such as `probe`, a stream such as `readings`, a
record ID such as `readings@1`, or a caveat name; an unknown subject returns
`exitCode: 2`.

Every completed operation returns `caveat-authoring/0.1` with operation,
sourceSha256, runtime identity, exitCode and the existing report. Both the JSON
text and structuredContent carry that same object. Read exitCode: a completed
operation may report invalid source (2), failing scenarios or strict warnings
(1), or success (0). An authored event rejection is still an event outcome,
not a bridge failure. Invalid tool arguments, busy/limit conditions, runtime
loading failure and worker failure use `isError: true` and a stable error kind.
No report authorizes an external action or authenticates its evidence.

## Limits and lifecycle

Each call starts a fresh subprocess and runtime and retains no session handle,
under either protocol. `tools/list` returns the same five tools in a fixed
order; 2026-07-28 results mark it cacheable for an hour (`ttlMs: 3600000`,
`cacheScope: "public"`), since the list changes only with the installed package.
Only one call runs at once; concurrent calls receive a busy error. A tool gets
10 seconds including startup. Source and serialized event/scenario data are
limited to 1 MiB each, and protocol input/output to 4 MiB. Additional limits are
1,000 events, 32 scenarios, 2,000 scenario steps, 1,000 expanded sends, nesting
64, and 4,096 request IDs per connection (reconnect afterward). Reports that
exceed the limit fail explicitly rather than being truncated.

Cancellation, connection close, timeout and output overflow terminate the
worker. A failed call cannot poison a later call's WASM instance. This is
subprocess containment, not an OS sandbox or a bound on total process memory.
The host owns authorization and the server process. A trusted runtime override
is available at startup with `--runtime DIRECTORY`, never as a tool argument.
There are no persistent-session, file-writing, network, or whatif tools.

## Worker environment

The worker uses the running Node executable, an absolute first-party worker
path, an explicit runtime directory and stdio. Its authoring operations need no
application configuration from environment variables. The bridge therefore
passes `env: {}` when spawning each child, rather than copying its parent's
application environment. Unrelated application secrets, `NODE_OPTIONS` and
`NODE_PATH` are not explicitly forwarded by the bridge. The server's own
startup environment remains the host's responsibility.

An empty environment option is not a promise that the resulting process has
zero variables. Node and its platform support library may add process-support
variables; on Windows, libuv supplies standard entries such as `SystemRoot`,
`PATH`, temporary-directory and user-profile variables. The regression checks
inspect a real child for an unrelated synthetic secret and Node preload
controls, then run the real authoring worker. Ordinary calls and failure,
timeout, cancellation and recovery retain separate checks.

This reduces unnecessary environment exposure. It does not turn the subprocess
into an operating-system sandbox, authenticate the selected runtime, or mean
that a third-party scanner's capability notices have been cleared.
The [interface review](reference/docs/MCP_AUTHORING_DESIGN.md) defines the
protocol choices, resource limits and required regression evidence. The
[agent quickstart](AGENT_START.md) introduces the same tools through the CLI.
