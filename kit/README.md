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
| `payload` | The payload cannot be sent unchanged as JSON (a non-finite number, `undefined`, a class instance). The session is untouched. |
| `fatal` | An unclassified runtime error, an unrecognised outcome, or a trap. The session refuses every later call and is never touched again. |
| `closed` | The session was closed. |

After a WebAssembly trap the whole runtime instance is refused; load another
with `loadRuntimeFromDirectory()`. `lib/session.mjs` has no Node imports, and
`loadRuntime({ module, wasm })` takes URLs, so it is written to run in a
browser, but no browser test exists yet.

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

## Not yet

- `init`, `validate` and `replay` commands.
- A packed tarball with the runtime inside it.
- Host conformance tests: the host library is the only future producer of
  `origin: "host"`.
- A browser test of the session library.
