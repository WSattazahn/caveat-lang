# caveat-lang

A bundled WebAssembly runtime, session library and scenario runner for reactive
Caveat programs in Node and browsers. Requires Node 20 or later for the CLI.

The selected release candidate is **`caveat-lang@0.1.0-rc.1`**, with the command
`caveat` and npm publication tag `next`. It is not published: `private: true`
stays set until publishing is explicitly authorized. The package can be
installed from a tested tarball without Rust:

```sh
npm init -y
npm install ./caveat-lang-0.1.0-rc.1.tgz
npx --no-install caveat test my.scenarios.json
```

Start with [Getting started](docs/GETTING_STARTED.md): it goes from an empty
directory to a tested program in about fifteen minutes. The
[documentation index](docs/README.md) lists the authoring guide, the language
reference and the thermostat example that ship in the package. The
repository's test suites are not included.

## Run scenario files

From an installed tarball:

```sh
npx --no-install caveat test --json a.scenarios.json b.scenarios.json
npx --no-install caveat test --runtime path/to/pkg-reactive file.scenarios.json
```

From a repository checkout, build the runtime first:

```sh
npm run build
node kit/bin/caveat.mjs test examples/thermostat_history.scenarios.json
```

Files follow [Scenarios 0.1](https://github.com/WSattazahn/caveat-lang/blob/ab3b0d3/spec/caveat-scenarios-0.1.md). Every file is
validated before anything runs. Exit status is 0 when every scenario passes,
1 when one fails, and 2 when a file is invalid or the runtime cannot load.

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
npx --no-install caveat explain program.cav events.jsonl
npx --no-install caveat explain --json program.cav events.jsonl
```

The events file has one JSON object per line, `{"event": "read", "payload":
{"value": 17}}`; the payload may be left out when an event takes none. With no
events file, the program's initial state is explained. Exit status is 0 when
the explanation is complete, 1 when an event failed fatally (the explanation is
of the session before it), and 2 when the program or the events file cannot be
used. From code, `explain(snapshot, events)` in `caveat-lang/explain` returns
the same report as data, and `formatExplanation` renders it as text.

## Use a session from code

With the package installed, save the
[thermostat example](https://github.com/WSattazahn/caveat-lang/blob/ab3b0d3/examples/thermostat_history.cav)
as `thermostat_history.cav` alongside this code:

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

Rejections are values, following the
[dispatch outcome contract](https://github.com/WSattazahn/caveat-lang/blob/ab3b0d3/spec/caveat-dispatch-0.1.md). Everything else
throws a `CaveatError` with a `kind`:

| `kind` | Meaning |
| --- | --- |
| `load` | The source did not load; the message is the runtime's diagnostic. |
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

## Tests

These commands run from a repository checkout, after `npm run build`.
`npm run test:kit` covers:

- validation and matching rules, including repeated elements in `$set` and
  `$includes`, and `before` defined only after the first `send`;
- the session library: typed outcomes, payload refusal, fatal and trapped
  sessions, and a runtime that predates `dispatch_outcome`;
- injected runtime faults: a rejection that changes state, a restore that
  loses history, a resumed session that diverges or disagrees, grounds outside
  lineage, fatal reports, unrecognised outcomes and traps. Each is caught and
  the misbehaving session is named;
- the [spec evidence](https://github.com/WSattazahn/caveat-lang/blob/ab3b0d3/experiments/scenario-format/README.md) fixtures and
  the CLI's exit codes;
- `explain`: the report matches the snapshot decision by decision, and the
  command lists refusals, stops at a fatal event and refuses bad input;
- the packaged documentation: every source exists, every link in the kit's
  own documents resolves inside the package, the authoring guide's script
  runs, and the getting-started guide, followed step by step, prints what it
  shows. The package test follows the guide again from the installed tarball.

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
install: the `caveat` command (`test` with exit statuses 0, 1 and 2, and
`explain`), the library imported as `caveat-lang/node`, `caveat-lang/session`,
`caveat-lang/scenarios` and `caveat-lang/explain`, and the browser check above.
The tarball holds the command, the four library files,
the runtime, this README, the license and the notices, and nothing else.

The package is MIT licensed. Packing copies the repository's `LICENSE` and
`THIRD_PARTY_NOTICES.md` (the crates compiled into the runtime) into the
tarball, and the test checks both arrive unchanged.

CI already runs this packaging test against the runtime built by its Linux
core job. It retains the exact tested `.tgz`, the test report with its SHA-256,
`build-info.json` and `SHA256SUMS` together as an artifact. A local Windows run
checks packaging locally; the release candidate must use the tested Linux
artifact from the intended release commit. The current private tarball is for
evaluation. Once publishing is authorized, commit the removal of `private`,
rebuild and retest the candidate on Linux, then publish that exact tested
tarball under the `next` tag. See the
[consolidation plan](https://github.com/WSattazahn/caveat-lang/blob/ab3b0d3/docs/CONSOLIDATION_PLAN.md)
for the remaining release gates.

## Not yet

- `init`, `validate` and `replay` commands.
- Publication on npm.
- Host conformance tests: the host library is the only future producer of
  `origin: "host"`.
