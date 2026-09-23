# CAVEAT Scenarios 0.1

A scenario file states what a reactive program should do: events to send and
what should follow. The developer kit's runner executes it through the
[dispatch outcome contract](caveat-dispatch-0.1.md). On every step the runner
also checks what every Caveat program owes: rejected events leave nothing
behind, a save resumes to the same session, and grounds stay within lineage.
A file states only what is specific to its program.

The requirements come from the
[coverage inventory](../docs/SCENARIO_COVERAGE_INVENTORY.md) of the existing
harnesses. Only reactive programs are in scope.

## File

A scenario file is UTF-8 JSON, conventionally `<program>.scenarios.json`.

```json
{
  "schema": "caveat-scenarios/0.1",
  "source": "trail_rescue.cav",
  "scenarios": [
    { "id": "R01", "title": "…", "steps": [] }
  ]
}
```

| Field | Meaning |
| --- | --- |
| `schema` | Required; exactly `"caveat-scenarios/0.1"`. |
| `source` | Required; one program file, relative to this file. |
| `scenarios` | Required; a non-empty array. |
| `id` | Required; unique within the file. |
| `title` | Required; what the scenario shows. |
| `source` (scenario) | Optional; a different program for this scenario, such as a committed variant. |
| `steps` | Required; a non-empty array, run in order. |
| `note` | Optional on the file, scenarios and steps; ignored. JSON has no comments. |

Unknown fields are errors at every level. An invalid file runs nothing.

## Steps

Each step has exactly one of `send`, `expect`, `same_as`, `checkpoint`,
`resume` or `size`, plus an optional `note`.

### `send`

```json
{ "send": "read", "payload": { "value": 17 } }
{ "send": "advance", "payload": { "dt": 0.5 }, "repeat": 40 }
{ "send": "plan", "payload": { "target": "reed" }, "rejected": "Reconsider the existing plan first." }
{ "send": "advance", "payload": { "dt": 31 }, "rejected": { "origin": "input", "code": "bound_exceeded" } }
```

`send` names the event. An omitted `payload` is `{}`. Any other JSON value,
including `null`, an array or a scalar, is serialized with standard JSON and
sent. Wrong types, missing or extra parameters and unknown members can
therefore be tested. Malformed JSON text and duplicate keys cannot be written
in a parsed scenario file; they stay in runtime conformance tests. A payload
number that parses as non-finite, such as `1e999`, makes the file invalid,
because standard JSON would silently send it as `null`. Negative zero is sent
as `0`. `repeat` sends the same event 1 to 10,000 times, and every repetition
must have the expected outcome.

Without `rejected`, the event must be accepted. Otherwise:

| `rejected` | Matches |
| --- | --- |
| `true` | A policy rejection, any message. |
| `"<text>"` | A policy rejection whose message is exactly this text. |
| `{"origin": …, "code"?: …, "message"?: …}` | That origin, and the code if given. `message` is allowed only with `"origin": "policy"`. |

A bare expected rejection matches only `origin: "policy"`, so a payload
refusal can never pass for a Caveat rule. `origin` is `policy`, `input`,
`evaluation` or `limit`. `host` is invalid in 0.1, because the runner sends
events straight to the core. Codes are compared as text, so a file may name a
code added after this document.

A fatal outcome or an escaped exception never matches anything. It fails the
scenario and ends every session in it; the runner does not read state from a
session after a fatal.

### `expect`

```json
{ "expect": {
    "/bindings/heating/text": "0%",
    "/commitment_grounds/heating@2/evidence": ["temperature@2"],
    "/commitment_bases/heating@2/provenance/evidence": ["temperature@1", "temperature@2"]
} }
```

Each key is a JSON Pointer ([RFC 6901](https://www.rfc-editor.org/rfc/rfc6901))
into the current snapshot. The view's fields are the snapshot's fields, except
`/schema`, which names each document's own schema. A pointer that works on the
rest of the view works here too. The snapshot adds `elapsed`,
`qualified_values`, `commitment_bases` and the rest of the lineage.

- Scalars must be equal.
- Arrays must have the same length and match element by element, in order.
- Objects match partially: listed members must match, and other members are
  ignored. This applies at every depth.
- A pointer that does not exist fails, unless the expectation is `$absent`.

A matcher is an object whose only member's name starts with `$`:

| Matcher | Matches |
| --- | --- |
| `{"$exact": value}` | Equal, with no extra members at any depth. |
| `{"$set": [...]}` | An array with exactly these elements in any order, counting repeats. |
| `{"$includes": [...]}` | An array that contains these elements anywhere, counting repeats. |
| `{"$absent": true}` | The pointer, or this object member, does not exist. |

Elements of `$set` and `$includes` are compared exactly, and both count
repeats: `["a", "b"]` does not match `{"$set": ["a", "a"]}`, and
`{"$includes": ["a", "a"]}` needs two occurrences of `"a"`.

Order is significant unless `$set` or `$includes` says otherwise. Journals, revisions and
reopenings are ordered; a harness that sorted every list once hid an ordering
bug. A literal object whose only member starts with `$` is written with
`$exact`. An object that mixes a matcher with other members is invalid.

### `checkpoint` and `same_as`

```json
{ "checkpoint": "first" }
{ "same_as": "first", "paths": ["/commitment_bases/heating@1", "/commitment_grounds/heating@1"] }
```

`checkpoint` stores the current snapshot under a new name. `same_as` requires
the values at `paths` to equal those in a stored snapshot exactly. Every path
must exist in both snapshots; a path missing from either fails, so two absent
values never count as equal. `paths` must be a non-empty array. A whole
snapshot always differs after an accepted event, because its sequence
advances.

Besides checkpoints, two names are defined:

- `initial`: the snapshot when the session was created.
- `before`: the primary session's snapshot just before the most recent `send`
  step, before its first repetition. It exists only after the scenario's first
  `send` step. A file that names `before` earlier, in `same_as` or in a `size`
  bound, is invalid.

A checkpoint must be defined by an earlier step of the same scenario. Its name
must be new and cannot be `initial` or `before`.

### `resume`

```json
{ "resume": true }
```

The runner saves the session, restores the save text with the scenario's
source, and requires an identical snapshot and view. The restored session
continues as the primary session. Sessions from before the resume keep
receiving every later event, and all of them must agree on each outcome and
on the snapshot after each accepted event. Failures name the session that
disagreed.

### `size`

```json
{ "size": { "save": { "max": 4096 } } }
{ "size": { "snapshot": { "max_growth": 512, "since": "start" } } }
```

Sizes are UTF-8 bytes of the compact JSON serialization of the parsed save or
snapshot, so runtime formatting does not count. `max` is an absolute limit.
`max_growth` is a limit on snapshot growth relative to a stored snapshot.

## What the runner checks on every step

These checks run without being written. Every scenario starts a new session.

1. Every dispatch's outcome matches the expected outcome.
2. After every rejected dispatch, the save, view and snapshot are identical to
   those before it, in every live session.
3. At `resume`, the restored snapshot and view equal the saved session's. After
   later events, every live session agrees on the outcome and snapshot.
4. After the last step, a final save and restore reproduces the snapshot.
5. Grounds are within lineage for every value and commitment: each name in
   `value_grounds` and `commitment_grounds` appears in the matching
   `qualified_values` or `commitment_bases` provenance. This runs at creation,
   after each accepted dispatch and after each restore. A violation is
   reported as a runtime invariant failure, not a program failure.
6. A source that fails to load fails the scenario at step 0 with the runtime's
   diagnostic.

The first failure ends its scenario; other scenarios still run. After a
WebAssembly trap the runner loads a fresh runtime before the next scenario,
because a trap leaves the whole instance in an unknown state. The runner adds
no randomness and reads no clock, so time passes only through the program's
own events.

## Results

A human report shows one line per scenario. A failure gives the scenario, the
step number, the failure kind and the session, then what differed: a pointer
with expected and actual values, or the expected and actual outcome. For
example:

```text
FAIL T01 step 5 expect [primary]: /bindings/heating/text expected "50%", actual "0%"
FAIL R01 step 8 send [primary]: expected a policy rejection; got input/bound_exceeded "dt must be finite and in 0..30"
```

Step 0 is creating the session, and the step after the last is the final
restore. Sessions are named `primary` (the newest), `shadow N` (the Nth
session created, so `shadow 1` is the session the scenario started with), or
`final restore`.

| Kind | Category | Failure |
| --- | --- | --- |
| `send` | `expectation` | The outcome differs from the expected outcome. |
| `expect`, `same_as`, `size` | `expectation` | A value or size differs. |
| `fatal` | `fatal` | A fatal outcome or escaped exception. |
| `load` | `load` | The source did not load or could not be read. |
| `atomicity` | `runtime-invariant` | A rejected event changed the save, view or snapshot. |
| `agreement` | `runtime-invariant` | A resumed session's outcome or snapshot differs from the primary's. |
| `resume`, `final-resume` | `runtime-invariant` | A restore changed the snapshot or view, or failed. |
| `grounds` | `runtime-invariant` | A grounded name is not in its lineage. |

The machine report, schema `caveat-scenario-report/0.1`, covers one run of
one or more files. It records the outcome schema, the runtime build identity,
and for each file the SHA-256 of each source it read and, per scenario,
`pass`, event, rejection and resume counts, and the first failure with the
fields above: `step`, `kind`, `category`, `session`, and `path`, `expected`,
`actual`, `outcome`, `repetition` or `message` as they apply.

Exit status is 0 when every scenario passes, 1 when any fails, and 2 when a
file is invalid or cannot be read, or the runtime cannot load. Every file is
validated first, and nothing runs with status 2.

## Examples

Both files below passed a throwaway prototype runner against the clean
compiled build of `541e09d`. Nine altered copies failed as intended (a wrong
value, list order, a bare `rejected` on a payload refusal, a wrong policy
message, a fatal expected as a rejection, a size bound and others), and five
invalid files were refused. Three more files exercised `repeat`, `$set`,
growth bounds and a state-bound `evaluation` rejection. The prototype, these
files and their recorded results are kept in
[`experiments/scenario-format`](../experiments/scenario-format/README.md).
The [kit runner](../kit/README.md) implements this document and runs those
files in its tests. Trail Rescue's 24 and Glowcap's 38 scenarios are
[converted](../experiments/scenario-conversion/README.md) as its acceptance
check.

`thermostat_history.scenarios.json`, the authoring guide's walkthrough:

```json
{
  "schema": "caveat-scenarios/0.1",
  "source": "thermostat_history.cav",
  "scenarios": [
    {
      "id": "T01",
      "title": "Each reading revises heating; earlier decisions keep what they knew",
      "steps": [
        { "send": "read", "payload": { "value": 17 } },
        { "expect": {
            "/bindings/heating/text": "100%",
            "/commitment_grounds/heating@1": { "evidence": ["temperature@1"], "caveats": ["calibration_offset"] }
        } },
        { "checkpoint": "first" },
        { "send": "read", "payload": { "value": 25 } },
        { "expect": {
            "/bindings/heating/text": "0%",
            "/bindings/change/text": "Rising",
            "/commitment_grounds/heating@2/evidence": ["temperature@2"],
            "/commitment_bases/heating@2/provenance/evidence": ["temperature@1", "temperature@2"]
        } },
        { "same_as": "first", "paths": ["/commitment_bases/heating@1", "/commitment_grounds/heating@1"] },
        { "resume": true },
        { "send": "read", "payload": { "value": 17 } },
        { "expect": {
            "/bindings/heating/text": "100%",
            "/decision_series/heating/current": "heating@3",
            "/decision_journal": [
              { "commitment": "heating@1", "change": "committed", "because": ["temperature@1"] },
              { "commitment": "heating@1", "change": "reopened", "because": ["temperature@2"] },
              { "commitment": "heating@2", "change": "committed", "value": 0 },
              { "commitment": "heating@2", "change": "reopened", "because": ["temperature@3"] },
              { "commitment": "heating@3", "change": "committed", "caveats": ["calibration_offset"] }
            ]
        } }
      ]
    },
    {
      "id": "T02",
      "title": "Invalid readings are refused by the core and leave nothing behind",
      "steps": [
        { "send": "read", "payload": { "value": 41 }, "rejected": { "origin": "input", "code": "bound_exceeded" } },
        { "send": "read", "rejected": { "origin": "input", "code": "payload_invalid" } },
        { "send": "warm", "rejected": { "origin": "input", "code": "unknown_event" } },
        { "expect": { "/sequence": 0, "/bindings/heating/text": "Awaiting a reading", "/decision_journal": [] } }
      ]
    }
  ]
}
```

The second decision is grounded on the second reading alone, while its
lineage holds both. The first decision's basis is unchanged by later readings.

`trail_rescue.scenarios.json`, with Caveat events rather than the page's
adapter events:

```json
{
  "schema": "caveat-scenarios/0.1",
  "source": "trail_rescue.cav",
  "scenarios": [
    {
      "id": "R01",
      "title": "A scouted plan goes stale and is reopened on the scout's own evidence",
      "steps": [
        { "send": "observe", "payload": { "target": "stone", "method": "scout", "condition": 0 } },
        { "send": "plan", "payload": { "target": "stone" } },
        { "expect": {
            "/bindings/decision/state": "committed",
            "/commitment_grounds/route@1/evidence": ["seen_scout_stone_1"]
        } },
        { "resume": true },
        { "send": "plan", "payload": { "target": "reed" }, "rejected": "Reconsider the existing plan first." },
        { "send": "advance", "payload": { "dt": 30 } },
        { "expect": {
            "/elapsed": 30,
            "/bindings/decision/state": "reopened",
            "/bindings/stone/status": "unknown",
            "/decision_journal/1": { "change": "reopened", "elapsed": 30, "because": ["seen_scout_stone_1"], "caveats": ["stale"] }
        } },
        { "send": "advance", "payload": { "dt": 31 }, "rejected": { "origin": "input", "code": "bound_exceeded" } },
        { "send": "rescue", "rejected": true }
      ]
    }
  ]
}
```

## Not in 0.1

- Host conformance tests. They belong to the host library, which will be the
  only producer of `origin: "host"`.
- Seeded properties, fuzzing, benchmarks and browser flows. The inventory
  keeps these as custom code.
- Numeric tolerance, phase filtering, and shared or included steps.
- Save compatibility across runtime releases. The save contract governs it.
- Programs split across imported modules. The runner loads one source file.

## Versioning

The schema versions this format independently of the outcome, save and
package versions. Because unknown fields and matchers are errors, a new
optional field or matcher is compatible: older runners refuse files that use
it instead of misreading them. Changing the meaning of an existing field,
matcher or check requires a new schema.
