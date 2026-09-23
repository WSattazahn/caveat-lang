# Caveat developer kit (unpublished)

A session library and scenario runner for reactive Caveat programs in Node.
It lives in this repository as a private package. Its name, version and
bundled runtime are release decisions in the
[consolidation plan](../docs/CONSOLIDATION_PLAN.md). Until then it uses the
repository build, so run `npm run build` first.

## Run scenario files

```sh
node kit/bin/caveat.mjs test examples/thermostat_history.scenarios.json
node kit/bin/caveat.mjs test --json a.scenarios.json b.scenarios.json
node kit/bin/caveat.mjs test --runtime path/to/pkg-reactive file.scenarios.json
```

Files follow [Scenarios 0.1](../spec/caveat-scenarios-0.1.md). Every file is
validated before anything runs. Exit status is 0 when every scenario passes,
1 when one fails, and 2 when a file is invalid or the runtime cannot load.

```text
PASS R01  A scouted plan goes stale and is reopened on the scout's own evidence  (6 events, 3 rejected, 1 resumes)
FAIL T01 step 5 expect [primary]: /bindings/heating/text expected "50%", actual "0%"
FAIL R01 step 8 send [primary]: expected a policy rejection; got input/bound_exceeded "dt must be finite and in 0..30"
```

## Use a session from code

```js
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from './kit/lib/node.mjs';

const source = await readFile('examples/thermostat_history.cav', 'utf8');
const runtime = await loadRuntimeFromDirectory();   // dist/pkg-reactive by default
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

Rejections are values, following the
[dispatch outcome contract](../spec/caveat-dispatch-0.1.md). Everything else
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
`close()` no longer frees them, and `open` and `restore` refuse. Load another
with `loadRuntimeFromDirectory()`.

In a browser, `lib/session.mjs` and `lib/scenarios.mjs` need no Node APIs.
Load the runtime from URLs:

```js
import { loadRuntime } from './node_modules/caveat-kit/lib/session.mjs';
const runtime = await loadRuntime({
  module: new URL('./node_modules/caveat-kit/runtime/caveat_runtime.js', location.href).href,
  wasm: new URL('./node_modules/caveat-kit/runtime/caveat_runtime_bg.wasm', location.href),
});
```

## Tests

`npm run test:kit` covers:

- validation and matching rules, including repeated elements in `$set` and
  `$includes`, and `before` defined only after the first `send`;
- the session library: typed outcomes, payload refusal, fatal and trapped
  sessions, and a runtime that predates `dispatch_outcome`;
- injected runtime faults: a rejection that changes state, a restore that
  loses history, a resumed session that diverges or disagrees, grounds outside
  lineage, fatal reports, unrecognised outcomes and traps. Each is caught and
  the misbehaving session is named;
- the [spec evidence](../experiments/scenario-format/README.md) fixtures and
  the CLI's exit codes.

`npm run test:scenario-conversion` runs the
[converted Trail Rescue and Glowcap suites](../experiments/scenario-conversion/README.md).

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
install: the `caveat` command (exit statuses 0, 1 and 2), the library imported
as `caveat-kit/node`, `caveat-kit/session` and `caveat-kit/scenarios`, and the
browser check above. The tarball holds the command, the three library files,
the runtime and this README, and nothing else.

This is a packaging test, not a release. The package has no license, and its
name and version are unset. A release packs the verified Linux runtime.

## Not yet

- `init`, `validate` and `replay` commands.
- A license, and a package name and version chosen for release.
- Host conformance tests: the host library is the only future producer of
  `origin: "host"`.
