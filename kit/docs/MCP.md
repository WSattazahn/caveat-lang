# MCP authoring tools (rc.7; rc.8 development)

> npm publication verified 2026-10-02T15:25:02.112Z: exact **caveat-lang@0.1.0-rc.7**,
> retained Linux artifact SHA256 `0e441f896c58ba41f2741a82af40a17e74247a0c765169abda10b9ddffad5bbc`.
> `next` names rc.7; `latest` remains rc.5. This checkout stays unpublished rc.8 development.
> [Publication receipt](https://github.com/WSattazahn/caveat-lang/blob/codex/rc7-npm-publication/docs/releases/v0.1.0-rc.7-npm-publication.json).


The frozen rc.7 GitHub candidate and exact npm version are available and verified.
This checkout is unpublished rc.8 development. No MCP desktop host is installed
by the release or development process.

An installed rc.7 tarball can expose existing Caveat authoring operations to a
local MCP host. Published rc.6 does not include this command. The bridge uses
the **2025-11-25 stdio compatibility profile**, with initialization followed by
tool discovery. It does not advertise the newer 2026-07-28 profile.

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
installation. `node` must resolve to Node 20 or newer. A host that supports only
the newer protocol must support this compatibility profile before connecting.
The process writes protocol messages on stdout and no startup banner.

## Available tools

| Tool | Inputs | Result |
| --- | --- | --- |
| caveat_validate | source | Whether the program loads and what it declares |
| caveat_check | source, optional strict | Existing diagnostics and suppressed warnings |
| caveat_test | source, scenarios | Existing scenario results and invariant checks |
| caveat_explain | source, optional events | Decisions, exact grounds, caveats and history |
| caveat_dependents | source, subject, optional events | What depends on the named evidence, stream or caveat |

`source` is inline program text. Events are objects with `event` and optional
`payload`. `scenarios` is a [scenario document](reference/spec/caveat-scenarios-0.1.md)
whose source, including per-scenario overrides, must be `inline.cav`. The bridge
has no user-file lookup: the supplied text is its only virtual source file.
Unknown argument fields are rejected.

Every completed operation returns `caveat-authoring/0.1` with operation,
sourceSha256, runtime identity, exitCode and the existing report. Both the JSON
text and structuredContent carry that same object. Read exitCode: a completed
operation may report invalid source (2), failing scenarios or strict warnings
(1), or success (0). An authored event rejection is still an event outcome,
not a bridge failure. Invalid tool arguments, busy/limit conditions, runtime
loading failure and worker failure use `isError: true` and a stable error kind.
No report authorizes an external action or authenticates its evidence.

## Limits and lifecycle

Each call starts a fresh subprocess and runtime and retains no session handle.
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

The [interface review](reference/docs/MCP_AUTHORING_DESIGN.md) defines the
protocol choices, resource limits and required regression evidence. The
[agent quickstart](AGENT_START.md) introduces the same tools through the CLI.
