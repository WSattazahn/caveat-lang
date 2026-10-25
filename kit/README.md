# CAVEAT Language (`caveat-lang`)

<!-- caveat-package:identity -->
Package: **`caveat-lang@1.3.0`**.
<!-- /caveat-package:identity -->

![Mr. Caveat holding a fortune ticket whose fine print runs off the card](https://raw.githubusercontent.com/WSattazahn/caveat-lang/main/caveatism/character/mr-caveat-hero.png)

**A decision ledger for agents.** Caveat is a rules language and a runtime
for decisions that may need taking back. A value keeps the evidence and
caveats it was computed from, and a committed decision keeps that basis. When
evidence is later corrected or withdrawn, a rule you write can reopen the
decision, and the original basis stays on record. `caveat explain` prints
what each decision rested on.

This package holds the runtime (a prebuilt 3.6 MB WebAssembly file), a
Node and browser library and the CLI. It has no npm dependencies and needs
Node 20 or later; no Rust installation is needed.

## Hello, world

In an empty directory with Node 20 or later, install the package:

<!-- caveat-package:install -->
```sh
npm init -y
npm install caveat-lang@1.3.0
```
<!-- /caveat-package:install -->

Then paste this. It writes a program with one piece of evidence, one decision
resting on it, and one event that withdraws the evidence and reopens the
decision, then asks why:

<!-- hello-world -->
```sh
cat > hello.cav <<'CAV'
claim safe_to_ship;
evidence ci from "the CI run";
evidence rerun from "a rerun of CI";
readings tests from ci limit 2;
decisions ship limit 2;

event ci_passed;
event ci_was_wrong;

on ci_passed sample tests = 1 supports safe_to_ship;
on ci_passed commit ship because enough using latest(tests);
on ci_was_wrong reveal rerun opposes safe_to_ship;
on ci_was_wrong withdraw latest(tests) because rerun;
on ci_was_wrong reopen ship because rerun;
CAV
printf '%s\n' '{"event":"ci_passed"}' '{"event":"ci_was_wrong"}' > events.jsonl
npx --no-install caveat-lang explain hello.cav events.jsonl
```

Part of what it prints:

<!-- hello-world-output -->
```text
    ship@1 = 1  reopened
      based on tests@1
      tests@1: observed at #1 ci_passed
      tests@1 has since been withdrawn at #2 because rerun
      #1 ci_passed: committed because tests@1
      #2 ci_was_wrong: reopened because rerun
```

The decision to ship was reopened when the CI result it rested on was
withdrawn, and it still says what it was first based on. The rest of Caveat
is about saying which evidence carries which doubt, and which rules reopen
what. [Caveat on one page](https://github.com/WSattazahn/caveat-lang/blob/main/kit/docs/REFERENCE.md)
(about 2,700 words, also in this package at `docs/REFERENCE.md`) is the
syntax; read it before writing your own program.

## Limits

- **It does not check that evidence is true.** A caveat is what the program's
  author declares about a source. Caveat carries it faithfully, but the result
  is only as good as the sources and caveats the program declares, and it does
  not measure a model's confidence.
- **Saves are not authenticated.** A save records its source's SHA-256, so it
  will not restore against a different program. It is not signed: an edited
  save can drop a caveat or change a value and still restore. Keep saves where
  only your application writes them, or sign them yourself.
- **It does not know who sent an event.** Your application decides who may
  send what.
- **It is slower than plain code.** On a small program, a few thousand events
  per second; the CLI takes about half a second to start; loading a program
  with thousands of declarations takes seconds.
- **The proofs cover a model.** Lean theorems cover a small model of the
  language, not the shipped Rust runtime, which is checked by tests and by
  sampled agreement with the model.

[Limits](https://github.com/WSattazahn/caveat-lang/blob/main/docs/LIMITS.md)
has the measurements behind these and how to rerun them, and
[what is checked, and how](https://github.com/WSattazahn/caveat-lang/blob/main/docs/VERIFICATION.md)
says what each kind of check shows.

## Try it

The package provides the `caveat-lang` command and the shorter `caveat`. In an
empty directory, install the exact version named above and run its
installation check, agent demo and starter:

<!-- caveat-package:starter -->
```sh
npm init -y
npm install caveat-lang@1.3.0
npx --no-install caveat-lang --version
npx --no-install caveat-lang doctor
npx --no-install caveat-lang demo agent
npx --no-install caveat-lang init
npx --no-install caveat-lang test umbrella.scenarios.json
npx --no-install caveat-lang explain umbrella.cav events.jsonl
```
<!-- /caveat-package:starter -->

The two scenarios pass. The explanation shows `umbrella@1 = 70` reopened by
`sky`, still based on `rain_chance@1` with its `forecast_is_old` caveat: the
sky cleared, so the decision is open again, and what it was first based on is
still shown. Follow
[the agent quickstart](https://github.com/WSattazahn/caveat-lang/blob/main/kit/docs/AGENT_START.md) to adapt this into an integration.

[Getting started](https://github.com/WSattazahn/caveat-lang/blob/main/kit/docs/GETTING_STARTED.md) walks through those files and has you
change the program. The [worked
example](https://github.com/WSattazahn/caveat-lang/blob/main/kit/docs/WORKED_EXAMPLE.md) takes one program through every command. The
[documentation index](https://github.com/WSattazahn/caveat-lang/blob/main/kit/docs/README.md) lists the authoring guide, the language
reference and the thermostat example that ship in the package. These links
open the current documentation on GitHub; the copies that shipped with this
version are in the package's own `docs/` and `examples/` directories.

To call Caveat from an agent or a tool, start with the [agent-evidence
example](https://github.com/WSattazahn/caveat-lang/blob/main/kit/examples/agent-evidence/README.md). It drives `caveat serve` from Python and judges each attempt by
the current assessment. Other projects are also named Caveat; see
[project names and CLI commands](https://github.com/WSattazahn/caveat-lang/blob/main/kit/docs/NAMES.md).
If an integration needs something Caveat is missing, please
[open an agent feature request](https://github.com/WSattazahn/caveat-lang/issues/new?template=agent-feature-request.md)
with the exact version, a small reproduction, and expected versus actual
behavior. Share only sanitized examples and output.

## Check what you are running

- `npm ls --all` shows the package has no dependencies.
- `npm audit signatures` checks the registry signature and the provenance
  attestation, which names the GitHub Actions run that published this version.
- The `caveat-lang` command is `bin/caveat.mjs`, one readable file. It loads
  `runtime/caveat_runtime.js` and `runtime/caveat_runtime_bg.wasm`, built
  from the repository's Rust sources. `runtime/build-info.json` lists the
  revision and each file's SHA-256 under `pkg-reactive/`, and `doctor` fails
  when an installed file differs from it. It is covered by the package
  signature but is not signed on its own.
- To install a tarball you were given instead, pass its path to
  `npm install`. The [release records](https://github.com/WSattazahn/caveat-lang/releases)
  say which versions were published and how each was verified.

## Check the installation and see an agent decision

With this package installed:

```sh
npx --no-install caveat-lang doctor
npx --no-install caveat-lang demo agent
```

`doctor --json` reports the same checks as text: Node support, package identity,
measured runtime hashes compared with the ones `build-info.json` lists (a
file that differs fails), local build metadata, a synthetic observation and
save/restore test, and executable ownership on PATH. Build metadata is covered
by the package signature but is not signed on its own. Shell aliases, functions and cached shell lookups are outside
that inspection. A missing PATH command or a competing `caveat` produces a
warning; use `caveat-lang`. Failed checks exit 1. `--runtime DIRECTORY` selects
a trusted runtime directory, as with the other commands.

`demo agent --json` returns each real snapshot and explanation from the shipped
[assessment program](https://github.com/WSattazahn/caveat-lang/blob/main/kit/examples/agent-evidence/assessment.cav). Illustrative inputs
85 and 92 produce two revisions: a correction withdraws the first observation
and reopens its assessment, and the replacement assessment uses the new reading.
The earlier grounds remain recorded. The demo writes no files and does not
measure a model's confidence or authorize an external action.

## Use authoring tools from an MCP host

This package can run `caveat-lang mcp` for the five existing
authoring operations. It takes inline source and uses a fresh subprocess for
each call. See [MCP setup and limits](https://github.com/WSattazahn/caveat-lang/blob/main/kit/docs/MCP.md): it speaks MCP 2026-07-28
and the 2025-11-25 profile.

## Run scenario files

From this installed package:

```sh
npx --no-install caveat-lang test --json a.scenarios.json b.scenarios.json
npx --no-install caveat-lang test --runtime path/to/pkg-reactive file.scenarios.json
```

From a repository checkout, build the runtime first:

```sh
npm run build
node kit/bin/caveat.mjs test examples/thermostat_history.scenarios.json
```

Files follow [Scenarios 0.1](https://github.com/WSattazahn/caveat-lang/blob/main/spec/caveat-scenarios-0.1.md).
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
each displayed value. Refused events are listed and change nothing. Decisions
show the SHA-256 of the source they were made under; changes restored from a save
too old to record it show the source as "not recorded". The digest identifies the
source text; it does not authenticate a save
([Source digest](https://github.com/WSattazahn/caveat-lang/blob/main/spec/caveat-save-0.1.md#source-digest)).

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

`caveat review` asks which evidence a revision was decided on: its grounds,
and the control outside them, such as evidence only the deciding rule's
`when` condition read. It replays the events without the events that
observed them, alone and
together, and lists the evidence without which the deciding event commits
nothing, the evidence that only moved the value, and the retained caveats
the revision's outcome says came true. It gives no scores and changes no
session. It is a 1.4 draft ([Review 0.1](https://github.com/WSattazahn/caveat-lang/blob/main/spec/caveat-review-0.1.md)).

```sh
npx --no-install caveat-lang review program.cav choose@1 events.jsonl
```

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
  intended. `--json` prints the report. `--unused` also reports a claim,
  evidence, caveat or state that nothing else names, such as one a rewrite
  left behind. `--records` also reports where a program reads a decision's
  record in a place a later `qualify` reaches (1.1 changed what those reads
  carry). `--records --save SAVE` lists the values in a save written by a
  1.0.x runtime that are missing such a caveat.
  [Check 0.1](https://github.com/WSattazahn/caveat-lang/blob/main/spec/caveat-check-0.1.md) lists the checks and
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
`restore`, `drainArchive`, `undrained` and `close`.

```text
> {"id":1,"op":"dispatch","event":"read_forecast","payload":{"chance":70}}
< {"id":1,"ok":true,"outcome":"accepted","sequence":1}
> {"id":2,"op":"dependents","of":"sky"}
< {"id":2,"ok":true,"report":{"schema":"caveat-dependents/0.1",…}}
```

A malformed request gets an error response and changes nothing. A fatal
outcome ends the server with status 1. [Serve 0.1](https://github.com/WSattazahn/caveat-lang/blob/main/spec/caveat-serve-0.1.md)
is the protocol. From code, `createServer` in `caveat-lang/serve` handles
lines without any I/O.

The [agent-evidence example](https://github.com/WSattazahn/caveat-lang/blob/main/kit/examples/agent-evidence/README.md)
drives a session from Python. It shows how to tell a
handled request from an accepted event, and both from a decision that permits
what the application intends.

## Use a session from code

The package ships the
[thermostat example](https://github.com/WSattazahn/caveat-lang/blob/main/examples/thermostat_history.cav). With the
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

To store a short session, call `session.save({ compact: true })` instead of
keeping your own list of events. While the session's accepted events fit in
4,096 bytes and are smaller than the state save, it returns them as a
[replay save](https://github.com/WSattazahn/caveat-lang/blob/main/spec/caveat-replay-save-0.1.md);
otherwise it returns the state save. `runtime.restore` takes either, and
`caveat serve` answers `{"op":"save","compact":true}` the same way. A runtime
before 1.1.0 cannot restore a replay save.

In the repository, import from `./kit/lib/node.mjs` instead; its default runtime
is the build in `dist/pkg-reactive`.

A host that redraws from the view after every event can call
`session.dispatchView(event, payload)` instead of `dispatch` and then `view`.
It runs the same event and returns `{outcome: "accepted", view}`, where `view`
is what `session.view()` then returns, or the same rejected outcome as
`dispatch`. The runtime does not build the snapshot for it. Its payload checks
and failures are `dispatch`'s, and after the same events either way the session
is the same. Read the view it returns: calling `session.view()` after it builds
the same view again.

A runtime keeps the last few programs it loaded, so opening or restoring the
same source again reuses the loaded program instead of parsing and checking it
again. The session is the same either way.

[View 0.2](https://github.com/WSattazahn/caveat-lang/blob/main/spec/caveat-view-0.2.md)
is available beside it. `session.view({ schema: '0.2' })` lists each
decision's grounds evidence in the order it was first observed, and says
`reopened` where View 0.1 says `open`. `session.dispatchViewDelta(event,
payload)` runs the same event and returns `{outcome: "accepted", delta}`: only
what the event changed, which applied to the 0.2 view the host holds gives the
next one. A refusal is `dispatch`'s, with no delta. After a restore, or when a
delta's `since` is not the sequence of the view the host holds, take a full
view again. `session.view()` stays View 0.1.

Rejections are values, following the
[dispatch outcome contract](https://github.com/WSattazahn/caveat-lang/blob/main/spec/caveat-dispatch-0.1.md).
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

Close each session with `close()` once you no longer need it. A session holds
runtime memory until it is closed, and the garbage collector frees an
unclosed one only after its last reference is gone and only between
synchronous runs, so a host that opens many sessions in one run (a test
loop, a fuzzer, a batch of saves resumed and dropped) can exhaust the
runtime's WebAssembly memory. The runtime then traps, and the `fatal` error
says it ran out of memory and how many sessions were still open
(`runtime.openSessions`). Load the runtime again to continue. Code that uses
the runtime's `WebReactiveSession` directly frees a session with `free()`, and
`WebReactiveSession.out_of_memory()` tells whether a trap came from memory.

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

## Hand a session to another agent

A main agent can hand work to a sub-agent with the ledger as the save: the
sub-agent restores it, reads what each decision rests on, and works under the
program's foreclosures, grants and holds. The runtime does not know who sends
an event, so `caveat-lang/handoff` keeps each agent to its own events:
`scopeSession(session, events)` refuses any other event as
`input/unknown_event` without sending it, and keeps the events it sent that
were accepted; `replayOnto(session, events)` sends those to another session
and reports each that no longer applies. The
[handoff example](https://github.com/WSattazahn/caveat-lang/blob/main/kit/examples/handoff/README.md)
runs a main agent and two sub-agents.

## Retain history outside the live session

Windowed programs have `session.drainArchive()` and the `session.undrained`
count. Keep drained record entries and provenance nodes separately
from saves if you need exact departed history. `explain` and `dependents` accept
the accumulated archive; incomplete or conflicting evidence remains explicitly
conservative. Historical completeness is scoped to supplied records and their
referenced dependencies, not authenticated or exhaustive session history.

Current snapshots may omit legitimately departed records. Exact own grounds
and required reasons remain retention obligations, and reachable reason chains
can still grow. Window sizes alone do not bound live memory or saves. See the
[history integration guide](https://github.com/WSattazahn/caveat-lang/blob/main/kit/docs/HISTORY.md)
for archive persistence, restore, report interpretation and workload limits.

The [integration starter](https://github.com/WSattazahn/caveat-lang/blob/main/kit/docs/STARTER.md), `caveat-lang/starter` with a Node
store in `caveat-lang/starter/node`, is one host that does this. It
checkpoints every accepted event with the source's SHA-256 and keeps drained
items until its store has written them. Its guide states what happens when a
write fails and when the process restarts.

## TypeScript and bundlers

Every library entry point carries its declarations (`lib/*.d.mts`), named
by the `types` condition of its `exports` entry. TypeScript finds them with `"moduleResolution"` set to
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

## Developing Caveat

Running the test suites and packing the kit need a repository checkout; see
[developing the kit](https://github.com/WSattazahn/caveat-lang/blob/main/docs/KIT_DEVELOPMENT.md).
The package is MIT licensed and carries the repository's `LICENSE` and
`THIRD_PARTY_NOTICES.md` (the crates compiled into the runtime).

## Not yet

- Host conformance tests: the host library is the only future producer of
  `origin: "host"`.
