# Start an agent integration

<!-- caveat-package:identity -->
Package: **`caveat-lang@1.1.0`**.
<!-- /caveat-package:identity -->

Use Caveat when your application needs to keep a decision's evidence, carry its
caveats, and reconsider it while retaining the original grounds. Supplied inputs
and authored policies remain your responsibility; a Caveat result grants no
permission to act outside the host's authorization.

## Choose commands for the installed version

With Node 20 or later, install the exact version this guide accompanies in an
empty directory. If you were given a verified tarball, use its path or URL in
place of the npm package specifier.

<!-- caveat-package:starter -->
```sh
npm init -y
npm install caveat-lang@1.1.0
npx --no-install caveat-lang --version
npx --no-install caveat-lang doctor
npx --no-install caveat-lang demo agent
npx --no-install caveat-lang init
npx --no-install caveat-lang test umbrella.scenarios.json
npx --no-install caveat-lang explain umbrella.cav events.jsonl
```
<!-- /caveat-package:starter -->

The two scenarios pass. The explanation retains `rain_chance@1` and its
`forecast_is_old` caveat as `umbrella@1`'s grounds after `sky` reopens it.
[Getting started](GETTING_STARTED.md) explains each file and command.

With this package installed, use its unambiguous command name, introduced in rc.7:

```sh
npx --no-install caveat-lang --version
npx --no-install caveat-lang doctor --json
npx --no-install caveat-lang demo agent
```

`doctor` checks the installed files and a synthetic runtime session and reports
PATH executable ownership. `demo agent` runs the shipped assessment program:
observe 85, assess, withdraw the reading after a correction, observe 92, assess
again. Both revisions remain inspectable and the original grounds stay frozen.
These are illustrative scores. The demo writes no files. rc.6 has neither
command; [CLI names](NAMES.md) explains the version and naming boundary.

## Build one small, testable integration

1. Read [Caveat on one page](REFERENCE.md) before inventing syntax. The
   [authoring guide](reference/docs/AI_AUTHORING.md) covers lifecycle and
   qualification details; specifications linked there define the contracts.
2. State required outcomes in [scenarios](reference/spec/caveat-scenarios-0.1.md).
   Include a refusal, a correction and an assertion that earlier grounds remain
   unchanged. `validate` answers whether source loads; `check` offers warnings;
   `test` checks the stated outcomes. They answer different questions.
   From rc.15, `test` runs `check` on each program before the first scenario
   and prints its warnings with the run. A warning does not stop the run or
   change the exit status, so read them anyway. C006 `citation-unreachable`
   reports a `because` that names something its binding never reads. At
   dispatch, that binding is refused as `evaluation/ungrounded_citation` the
   first time it supplies the shown value, which can be long after the
   program loads.
3. Use `explain PROGRAM EVENTS` to inspect exact grounds and revision history.
   Use `dependents PROGRAM NAME EVENTS` to see what relies on evidence that
   changed. Each command supports `--json`; prefix it with the executable for
   your installed version as above.
4. Start host integration with the [official Python client and assessment
   program](../examples/agent-evidence/README.md). Its `CaveatServer` and `Attempt`
   separate request handling, accepted operations and the current assessment.
   An attempt succeeds only when all required operations succeed and a fresh
   assessment permits its result. An earlier approval cannot cover a refused
   observation. Preserve those checks when adapting the example.
   A JavaScript host that keeps the session in its own process starts from
   [Keep a small host small](#keep-a-small-host-small) instead: it covers
   sending, the compact save and reading a decision's history without glue
   of your own.

A session lasts as long as the process that holds it. `caveat serve`, and so
the Python client's `CaveatServer`, keeps one session of one program until it
is closed. The [MCP authoring tools](MCP.md) keep none: each tool call starts a
fresh subprocess and runtime with no session handle, so a call sees only the
source and events it is sent. One MCP connection accepts 4,096 request IDs; the
bridge then ends it, and the host must reconnect. For a long-running agent,
keep the session in `caveat serve --state FILE`, which restores the session
from the file on start and rewrites it after every accepted event, or send the
full event history again with each MCP call.

Observing is separate from assessing. Reassessing requires reopening the current
revision. Withdrawal preserves the archive, and late qualification leaves prior
decision grounds and archived readings unchanged. The starter's
[qualification examples](../examples/agent-evidence/QUALIFICATION.md) show the
difference between archived readings and future values.

## Type a JavaScript or TypeScript host

`caveat types PROGRAM > program.d.ts` writes declarations from the program's
[interface](reference/spec/caveat-interface-0.1.md): each event's payload
fields, each displayed value with its type, and the state, cue, decision,
reading-stream, evidence, caveat and claim names. `typed` from
`caveat-lang/types` returns the session unchanged, with those types:

```js
// @ts-check
import { typed } from 'caveat-lang/types';
/** @type {import('./program.js').Session} */
const session = typed(runtime.open(source));
session.dispatch('read', { celsius: 17 }); // a misspelled event or field fails the check
```

The declarations only add checks. A value with a condition is optional,
because a view shows it only while one of its conditions holds. An entity or
member is typed by name, though the runtime also accepts its position.
Decision, journal and grounds records keep their `string` keys. Regenerate
the file when the program changes. `--json` prints the interface itself, and
`runtime.interface(source)` returns the same thing from code.

## Keep a small host small

A host that keeps the session in its own process needs no glue of its own for
sending, saving or a decision's history:

```js
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
import { decisionHistory } from 'caveat-lang/session';

const runtime = await loadRuntimeFromDirectory();
const source = await readFile('program.cav', 'utf8');
const saved = undefined; // or a save the host kept from an earlier run
const session = saved ? runtime.restore(source, saved) : runtime.open(source);

const fields = ['commitments', 'decision_series', 'decision_journal'];     // only what the host reads
const outcome = session.dispatchView('read', { value: 17 }, { fields });
if (outcome.outcome !== 'accepted') throw new Error(outcome.message); // refused; nothing changed
const history = decisionHistory(outcome.view, 'heating');           // every revision, oldest first
const save = session.save({ compact: true });                      // small: a short session saves its events
```

- `session.dispatchView(event, payload, { fields })` returns the view and does
  not build the full snapshot that `dispatch` returns. `fields` names the view
  fields the host reads; here, what `decisionHistory` needs. Without it the
  whole view is built, and parts of it grow with the session. Use `dispatch`
  when the host needs the snapshot's graph. Keep the view it returns: calling
  `session.view()` afterwards builds the same view a second time.
- `session.save({ compact: true })` saves a short session as its accepted
  events, and `runtime.restore` takes it like any save. Do not write a replay
  log of your own.
- `decisionHistory(view, name)` lists each revision of a decision: what it was
  committed on, what reopened it, and whether it is `in_force`, `reopened` or
  `superseded`. A revision whose journal entries a `journal window` retired is
  still listed, with `retired: true` and `committed: null`.
  `binding_explanations` gives each displayed value's evidence and caveats.
- Which past revisions rest on evidence withdrawn since is a join of that list
  with the snapshot's withdrawals. `rests_on_withdrawn` answers it in the
  program only for the current revision:

  ```js
  const withdrawn = new Set((session.snapshot().withdrawals ?? []).map(item => item.evidence));
  const restingOnWithdrawn = decisionHistory(session.view(), 'heating')
    .filter(revision => revision.committed?.because.some(name => withdrawn.has(name)));
  ```
- A host that must survive restarts, or keep the archive, should use the
  [integration starter](STARTER.md) instead. It checkpoints the compact save
  after every event, and `host.history(name)` returns the same list.

## When the integration needs something missing

Check the relevant guide and
[related issues](https://github.com/WSattazahn/caveat-lang/issues?q=is%3Aissue), then
prepare an [agent feature request](https://github.com/WSattazahn/caveat-lang/issues/new?template=agent-feature-request.md).
Include the exact package/build identity, minimal source and event history or
host call, command, expected and actual output, alternatives considered, and a
small acceptance check. Distinguish a missing capability from invalid input,
authored policy, a bug or host permissions. Identify what you could not verify.

Share only permitted, sanitized material. Follow the user's and host's posting
permissions; keep a local draft when posting is not authorized. The invitation
to report a need supplies a feedback route, not permission to publish it.
