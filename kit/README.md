# CAVEAT Language (`caveat-lang`)

<!-- caveat-package:identity -->
Package: **`caveat-lang@0.1.0-rc.11`**.
<!-- /caveat-package:identity -->

When evidence changes, reconsider the decision while keeping its original
reasons. CAVEAT Language carries evidence and caveats through computed values
and records every decision revision. This package includes the WebAssembly
runtime, Node/browser library and CLI; no Rust installation is needed.
Requires Node 20 or later for the CLI.

The package provides the `caveat-lang` command and supported `caveat`
shorthand. In an empty directory, install the exact version named above and
run its installation check, agent demo and starter:

<!-- caveat-package:starter -->
```sh
npm init -y
npm install caveat-lang@0.1.0-rc.11
npx --no-install caveat-lang --version
npx --no-install caveat-lang doctor
npx --no-install caveat-lang demo agent
npx --no-install caveat-lang init
npx --no-install caveat-lang test umbrella.scenarios.json
npx --no-install caveat-lang explain umbrella.cav events.jsonl
```
<!-- /caveat-package:starter -->

The two scenarios pass. The explanation shows `umbrella@1 = 70` reopened by
`sky`, still based on `rain_chance@1` with its `forecast_is_old` caveat. Follow
[the agent quickstart](docs/AGENT_START.md) to adapt this into an integration.

To use a verified tarball you were given, pass its path or URL to `npm install`
instead. The [release records](https://github.com/WSattazahn/caveat-lang/releases)
record which versions have been published and their verification receipts.
This document describes the package version printed above; it does not claim
that a development checkout has been published.

Introduced in rc.7, `caveat-lang` is the unambiguous command; `caveat` remains
supported shorthand.

See [project names and CLI commands](docs/NAMES.md) for the distinction from
other projects named Caveat. Historical rc.6 has only the `caveat` command.

Start with [Getting started](docs/GETTING_STARTED.md): it goes from an empty
directory to a tested program in about fifteen minutes. [Caveat on one
page](docs/REFERENCE.md) gives the language in brief, and the [worked
example](docs/WORKED_EXAMPLE.md) takes one program through every command. The
[documentation index](docs/README.md) lists the authoring guide, the language
reference and the thermostat example that ship in the package. The
repository's test suites are not included.

To call Caveat from an agent or a tool, start with the [agent-evidence
example](examples/agent-evidence/README.md), in the package from
0.1.0-rc.6. It drives `caveat serve` from Python and judges each attempt by
the current assessment. If that integration exposes a capability Caveat is
missing, please
[open an agent feature request](https://github.com/WSattazahn/caveat-lang/issues/new?template=agent-feature-request.md)
for the project owner to evaluate. Check existing issues and include the exact
version, a small reproduction, and expected versus actual behavior. Share only
sanitized examples and output.

## Check the installation and see an agent decision

With this package installed (these commands were introduced in rc.7):

```sh
npx --no-install caveat-lang doctor
npx --no-install caveat-lang demo agent
```

`doctor --json` reports the same checks as text: Node support, package identity,
measured runtime hashes, local build metadata, a synthetic observation and
save/restore test, and executable ownership on PATH. Build metadata is reported,
not authenticated. Shell aliases, functions and cached shell lookups are outside
that inspection. A missing PATH command or a competing `caveat` produces a
warning; use `caveat-lang`. Failed checks exit 1. `--runtime DIRECTORY` selects
a trusted runtime directory, as with the other commands.

`demo agent --json` returns each real snapshot and explanation from the shipped
[assessment program](examples/agent-evidence/assessment.cav). Illustrative inputs
85 and 92 produce two revisions: a correction withdraws the first observation
and reopens its assessment, and the replacement assessment uses the new reading.
The earlier grounds remain recorded. The demo writes no files and does not
measure a model's confidence or authorize an external action.

## Use authoring tools from an MCP host

This package can run `caveat-lang mcp` for the five existing
authoring operations. It takes inline source and uses a fresh subprocess for
each call. See [MCP setup and limits](docs/MCP.md), including its explicit
2025-11-25 stdio compatibility profile. Published rc.6 does not include it.

## Run scenario files

From this installed package (use `caveat` instead for historical rc.6):

```sh
npx --no-install caveat-lang test --json a.scenarios.json b.scenarios.json
npx --no-install caveat-lang test --runtime path/to/pkg-reactive file.scenarios.json
```

From a repository checkout, build the runtime first:

```sh
npm run build
node kit/bin/caveat.mjs test examples/thermostat_history.scenarios.json
```

Files follow [Scenarios 0.1](docs/reference/spec/caveat-scenarios-0.1.md).
Every file is validated before anything runs. Exit status is 0 when every
scenario passes, 1 when one fails, and 2 when a file is invalid or the runtime
cannot load.

```text
PASS R01  A scouted plan goes stale and is reopened on the scout's own evidence  (6 events, 3 rejected, 1 resumes)
FAIL T01 step 5 expect [primary]: /bindings/heating/text expected "50%", actual "0%"
FAIL R01 step 8 send [primary]: expected a policy rejection; got input/bound_exceeded "dt must be finite and in 0..30"
```

## Ask why

`caveat explain` sends a list of events to a program and shows what it decided
and why: each decision with its value, whether it is still in force, what it is
based on, what else could have influenced it, and every time it was committed
or reopened; each piece of evidence with its caveats; and the evidence behind
each displayed value. Refused events are listed and change nothing.

```sh
npx --no-install caveat-lang explain program.cav events.jsonl
npx --no-install caveat-lang explain --json program.cav events.jsonl
```

The events file has one JSON object per line, `{"event": "read", "payload":
{"value": 17}}`; the payload may be left out when an event takes none. With no
events file, the program's initial state is explained. Exit status is 0 when
the explanation is complete, 1 when an event failed fatally (the explanation is
of the session before it), and 2 when the program or the events file cannot be
used. From code, `explain(snapshot, events)` in `caveat-lang/explain` returns
the same report as data, and `formatExplanation` renders it as text.

`caveat dependents` asks the reverse question: what rests on one piece of
evidence, reading stream or caveat. It lists:

- the decisions based on it, or that it could have influenced;
- the decision changes it caused;
- the values based on it;
- the displayed values that cite it.

A reading stream, or the evidence it reads from, stands for every reading in
it.

```sh
npx --no-install caveat-lang dependents program.cav sky events.jsonl
npx --no-install caveat-lang dependents --json program.cav forecast_is_old events.jsonl
```

The exit statuses are those of `explain`, and 2 also means the program
declares no such name. From code, `dependents(snapshot, name)` in
`caveat-lang/explain` returns the report.

## Check, replay, start

```sh
npx --no-install caveat-lang validate program.cav
npx --no-install caveat-lang check program.cav
npx --no-install caveat-lang replay program.cav events.jsonl
npx --no-install caveat-lang init my-project
```

- `validate` loads a program and lists its events, reading streams, decision
  series and displayed values. `--json` prints them as data. It exits with 0
  if the program loads and 2 if it does not.
- `check` loads a program and reports patterns worth a second look. Each
  warning has a stable code, a line and a suggestion; for example, rules
  repeated across events that a procedure could share. Warnings are advice: it
  exits with 0 whatever it finds, 1 with `--strict` when there is a warning,
  and 2 if the program does not load. A comment
  `# caveat check: allow CODE` on the line above silences a pattern that is
  intended. `--json` prints the report.
  [Check 0.1](docs/reference/spec/caveat-check-0.1.md) lists the checks and
  their limits.
- `replay` prints one JSON record per line. First comes the initial snapshot.
  Then each event gets a record with its line in the file and its outcome,
  plus the snapshot after it if it was accepted. A fatal event's record is
  last, and `replay` then exits with 1.
- `init` writes the getting-started program, its scenarios and an events
  file. It refuses to overwrite any of them.

## Drive a session from another program

`caveat serve program.cav` keeps one session open. It reads requests from
standard input, one JSON object per line, and answers each on standard output.
The operations are `dispatch`, `snapshot`, `explain`, `dependents`, `save`,
`restore` and `close`.

```text
> {"id":1,"op":"dispatch","event":"read_forecast","payload":{"chance":70}}
< {"id":1,"ok":true,"outcome":"accepted","sequence":1}
> {"id":2,"op":"dependents","of":"sky"}
< {"id":2,"ok":true,"report":{"schema":"caveat-dependents/0.1",…}}
```

A malformed request gets an error response and changes nothing. A fatal
outcome ends the server with status 1. [Serve 0.1](docs/reference/spec/caveat-serve-0.1.md)
is the protocol. From code, `createServer` in `caveat-lang/serve` handles
lines without any I/O.

The [agent-evidence example](examples/agent-evidence/README.md), in the
package from 0.1.0-rc.6, drives a session from Python. It shows how to tell a
handled request from an accepted event, and both from a decision that permits
what the application intends.

## Use a session from code

The package ships the
[thermostat example](docs/reference/examples/thermostat_history.cav). With the
package installed, copy it from
`node_modules/caveat-lang/docs/reference/examples/thermostat_history.cav` to
`thermostat_history.cav` alongside this code:

```js
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const source = await readFile(new URL('./thermostat_history.cav', import.meta.url), 'utf8');
const runtime = await loadRuntimeFromDirectory();   // uses the bundled runtime
const session = runtime.open(source);

const outcome = session.dispatch('read', { value: 17 });
// {outcome: "accepted", snapshot} or {outcome: "rejected", origin, code, message}
if (outcome.outcome === 'rejected' && outcome.origin === 'policy') console.log(outcome.message);

const saved = session.save();                        // text; keep it with the exact source
const resumed = runtime.restore(source, saved);
console.log(resumed.view().bindings.heating.text);
session.close();
resumed.close();
```

In the repository, import from `./kit/lib/node.mjs` instead; its default runtime
is the build in `dist/pkg-reactive`.

A host that redraws from the view after every event can call
`session.dispatchView(event, payload)` instead of `dispatch` and then `view`.
It runs the same event and returns `{outcome: "accepted", view}`, where `view`
is what `session.view()` then returns, or the same rejected outcome as
`dispatch`. The runtime does not build the snapshot for it. Its payload checks
and failures are `dispatch`'s, and after the same events either way the session
is the same.

Rejections are values, following the
[dispatch outcome contract](docs/reference/spec/caveat-dispatch-0.1.md).
Everything else throws a `CaveatError` with a `kind`:

| `kind` | Meaning |
| --- | --- |
| `load` | The source did not load; the message is the runtime's diagnostic. `dispatchView` also throws it on a runtime build that predates it, and the session is untouched. |
| `restore` | The save does not restore with this source. |
| `payload` | The payload cannot be sent unchanged as JSON (a non-finite number, `undefined`, a hole in an array, a class instance). The session is untouched. |
| `fatal` | An unclassified runtime error, an unrecognised outcome, runtime text that is not JSON, or a trap. The session refuses every later call and is never touched again. |
| `closed` | The session was closed. |

A WebAssembly trap in any session, including during `close()`, marks the
whole runtime instance trapped: every session from it refuses further calls,
`close()` no longer frees them, and `open` and `restore` refuse. Runtimes
loaded from the same module share one instance, so they share that state too.
To recover, load again: `loadRuntimeFromDirectory()` always gives a fresh
instance, and `loadRuntime()` given a module URL imports a fresh copy once the
earlier instance has trapped.

In a browser, `lib/session.mjs` and `lib/scenarios.mjs` need no Node APIs.
Serve the installed files over HTTP and load the runtime from URLs (adjust
these paths to your server layout):

```js
import { loadRuntime } from './node_modules/caveat-lang/lib/session.mjs';
const runtime = await loadRuntime({
  module: new URL('./node_modules/caveat-lang/runtime/caveat_runtime.js', location.href).href,
  wasm: new URL('./node_modules/caveat-lang/runtime/caveat_runtime_bg.wasm', location.href),
});
```

## TypeScript and bundlers

Every library entry point carries its declarations (`lib/*.d.mts`), named
by the `types` condition of its `exports` entry. They are new in rc.6;
rc.5 has none. TypeScript finds them with `"moduleResolution"` set to
`"bundler"`, `"node16"` or `"nodenext"`, and nothing needs declaring by hand.
They cover the session library, the dispatch outcomes, `CaveatError` and its
kinds, the snapshot and view, and every report. The runtime module ships
wasm-bindgen's `runtime/caveat_runtime.d.ts`, with a reference to the
`esnext.disposable` TypeScript library required by its `Symbol.dispose`
member. The generated declaration loads that library itself; a consumer
using `ES2022` does not need to add it to its own `lib` list. Release-preparation
compiler probes use TypeScript 6.0.3; a minimum supported compiler version
has not been established.

In a bundled page, import the runtime module yourself and pass its namespace:

```ts
import { loadRuntime, type DispatchViewOutcome } from 'caveat-lang/session';
import * as runtimeModule from 'caveat-lang/runtime/caveat_runtime.js';

const runtime = await loadRuntime({ module: runtimeModule });
const source = await (await fetch('/thermostat_history.cav')).text();
const outcome: DispatchViewOutcome = runtime.open(source).dispatchView('read', { value: 17 });
```

The bundler follows that import like any other. The runtime finds its
`.wasm` with `new URL(…, import.meta.url)`, which webpack and Vite emit beside
the bundle. Given a URL instead, `loadRuntime` imports it with the browser's
own `import()`, which webpack and Vite are told to leave alone. A namespace
cannot be imported afresh, so after a trap only a URL gives a new instance.

## Tests

These commands run from a repository checkout, after `npm run build`.
`npm run test:kit` covers:

- validation and matching rules, including repeated elements in `$set` and
  `$includes`, and `before` defined only after the first `send`;
- the session library: typed outcomes, payload refusal, fatal and trapped
  sessions, and a runtime that predates `dispatch_outcome`;
- `dispatchView` against `dispatch` and `view`, event by event: the same
  outcome, view, save and snapshot for every refusal code, fatal outcomes,
  every tracked program on a seeded random stream, and the Glowcap, agent
  ledger and Trail Rescue streams;
- injected runtime faults: a rejection that changes state, a restore that
  loses history, a resumed session that diverges or disagrees, grounds outside
  lineage, fatal reports, unrecognised outcomes and traps. Each is caught and
  the misbehaving session is named;
- the [spec evidence](https://github.com/WSattazahn/caveat-lang/blob/ab3b0d3/experiments/scenario-format/README.md) fixtures and
  the CLI's exit codes;
- `explain`: the report matches the snapshot decision by decision, and the
  command lists refusals, stops at a fatal event and refuses bad input;
- `dependents`: the answer is checked against the snapshot's grounds and
  lineage for evidence, reading streams and caveats, and an unknown name is
  refused;
- `serve`: dispatch, explain, dependents, save and restore over the protocol;
  malformed requests answered without effect; a fatal outcome ending the
  server; and the process itself on standard input and output;
- the agent-evidence example: its program loads with no warnings, and its
  Python caller's tests pass against this checkout's `caveat serve`. They need
  Python 3.9 or later;
- `validate`, `replay` and `init`: exit statuses, line numbers, typed
  parameters, and `init`'s files staying identical to the guide's;
- the TypeScript declarations, by name: every export, method and member,
  every field of the snapshot, view, outcomes and reports, every option and
  runtime call, and every code and kind, against the modules, real sessions,
  the runtime's Rust structs and the documents that list them. Parameter and
  return types are not compared;
- the packaged documentation: every source exists, every link in the kit's
  own documents resolves inside the package, every link a copied document
  keeps resolves inside it and every link it rewrites names a repository
  file, the authoring guide's script runs, and the getting-started guide,
  followed step by step, prints what it shows. The package test follows the
  guide again from the installed tarball.

`npm run test:scenario-conversion` runs the
[converted Trail Rescue and Glowcap suites](https://github.com/WSattazahn/caveat-lang/blob/ab3b0d3/experiments/scenario-conversion/README.md).

`npm run test:kit-browser` runs the session library and the scenario runner
in Chromium: the runtime loaded from URLs, typed outcomes, refused input that
leaves state unchanged, save and restore, `elapsed()`, and scenario files read
with `fetch`, including a failing one. Set `PLAYWRIGHT_CHANNEL=chrome` or
`msedge` to use an installed browser.

## Packing

`npm run test:kit-package` copies the reactive runtime and its build info into
`kit/runtime/`, packs the kit, and removes `kit/runtime/` again so development
never uses a stale copy. It installs the tarball offline into a fresh consumer
directory under `test-results/kit-package/` and uses it only through the
install: the `caveat` command (`test` with exit statuses 0, 1 and 2,
`explain`, `dependents`, `validate`, `replay`, `serve` and `init`), the
library imported as `caveat-lang/node`, `caveat-lang/session`,
`caveat-lang/scenarios`, `caveat-lang/explain` and `caveat-lang/serve`, and
the browser check above. It also copies the agent-evidence example out of the
install and runs its Python tests against the installed command. The tarball
holds the command, the public library files, bounded authoring bridge and their declarations, `init`'s
templates, the example, the runtime and its declarations, the documentation,
the license and the notices, and nothing else. In the documents copied from
the repository, links that leave the package point to GitHub at the build's
revision.

The package is MIT licensed. Packing copies the repository's `LICENSE` and
`THIRD_PARTY_NOTICES.md` (the crates compiled into the runtime) into the
tarball, and the test checks both arrive unchanged.

Exact installation examples and the package identity are generated from
`kit/package.json`. After changing that manifest, run
`node scripts/kit-docs.mjs --write` from the repository root.
`node scripts/kit-docs.mjs --check` and the packaging gate refuse stale blocks
or moving publication notices in the kit's own guides. The installed-package
test runs the README commands against the same tarball, including doctor, demo
and the starter.

CI already runs this packaging test against the runtime built by its Linux
core job. It retains the exact tested `.tgz`, the test report with its SHA-256,
`build-info.json` and `SHA256SUMS` together as an artifact. A local Windows run
checks packaging locally; the release candidate must use the tested Linux
artifact from the intended release commit. Before publication, the
[packed dependency security gate](docs/reference/docs/PACKAGE_SECURITY.md) must
pass against that exact tarball and retain its npm/Rust advisory receipts.
A release publishes that one tested
tarball twice, unchanged: to npm and attached to the GitHub pre-release. No
second build is made for the registry. Each release record names the npm
dist-tags its version was published under. See the
[consolidation plan](https://github.com/WSattazahn/caveat-lang/blob/main/docs/CONSOLIDATION_PLAN.md#candidate-and-release-gates)
for the release gates.

## Not yet

- A stable release. This package version is a release candidate; its API may
  change before 1.0.
- Host conformance tests: the host library is the only future producer of
  `origin: "host"`.
