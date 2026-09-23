# Authoring and checking Caveat programs

An agent can use Caveat through the same source, interpreter, and tests as a
human developer. The current tools support validation and deterministic event
replay. They do not authenticate evidence, inspect a model's internal reasoning,
or authorize external actions.

## Run a reactive program without a browser

From the repository root:

```sh
cargo run --quiet --manifest-path runtime/Cargo.toml --bin caveat-reactive -- validate examples/thermostat_history.cav
cargo run --quiet --manifest-path runtime/Cargo.toml --bin caveat-reactive -- replay examples/thermostat_history.cav examples/thermostat_readings.jsonl
```

After building, `runtime/target/debug/caveat-reactive` is the direct executable
(`caveat-reactive.exe` on Windows). Use `-` for the event input path to read
standard input. Each event is one JSON object:

```json
{"event":"read","payload":{"value":17}}
```

`validate` emits a JSON record with kind `validated` and the complete initial
snapshot, including source identity and event signatures. It invokes
`ReactiveSession::from_source`, so invalid reactive rules and procedures fail
before a successful record is written.

`replay` emits an `initial` record, then an `event` record containing the input
line, event name, and snapshot after each successful dispatch. Output is JSONL,
with one complete JSON object per line. Blank input lines are ignored; reported
line numbers still count them. Identical source and input produce identical
snapshot records.

The first invalid envelope or failed event stops replay with exit status 1.
Standard error contains one JSON error with `stage` and `message`; input and
dispatch errors also identify the line and last committed sequence. A failed
event emits no snapshot. Earlier successful events remain in the output.
Malformed commands exit with status 2. Unknown envelope fields and duplicate
event/payload fields are rejected. Duplicate numeric keys inside a payload are
preserved until the interpreter rejects them, rather than being silently
collapsed by a JSON conversion.

Source files and individual event lines are limited to 1 MiB each. The
interpreter's expression, history, provenance, and event-work limits still
apply. A source file is read once; this is replay, not a file watcher.

`caveat-map validate` checks declared map structure and reports
`validation_scope: "declared_map"`. It does not validate reactive execution.
Use `caveat-reactive validate` for reactive source. Pure function extraction
through `SourceLibrary` has a different scope again: it validates the extracted
functions, not the rest of their containing program.

## An authoring loop

1. Read the event contract and the relevant reactive profile. Keep observations,
   proposed policies, physical conditions, and committed decisions distinct.
2. Write a small program and an explicit event history. Mark which inputs are
   observations and which are ordinary controls. An evidence source label is
   descriptive metadata, not an authentication mechanism.
3. Validate, replay, and inspect `qualified_values`, `reading_streams`,
   `commitment_bases`, `decision_series`, and `relations` in the snapshots.
4. Exercise missing observations, contradictory readings, repeated equal
   readings, reopened decisions, and failures late in an event.
5. Change one policy and replay the same inputs. Check both the changed decision
   and the records that should remain unchanged.

For the thermostat fixture, the input sequence is 17, 25, 17. The expected
heating bases are 1, 0, 1, with three distinct temperature occurrences and
three decision revisions. Each revision retains the calibration caveat. The
first revision's value and provenance stay frozen after subsequent readings.
The existing `thermostat_history` tests also demonstrate a policy based on the
recorded mean, with a different command and unchanged sensor archive.

The latest profiles describe [history computation](../spec/caveat-reactive-0.6.md)
and [procedures](../spec/caveat-reactive-0.7.md). The
[essence document](CAVEAT_ESSENCE.md) records the invariants that edits must
preserve. Runtime tests are executable examples, including adversarial inputs.

## Reconsider a decision when its evidence ages

Keep a live state for the basis you actually used. The commitment freezes its
original grounds; the separate state continues receiving late caveats:

```caveat
state plan_basis = 0;
decisions route limit 6;
on plan set plan_basis = current_assessment;
on plan commit route because enough using plan_basis;
on advance when committed(route) and not reopened(route)
    and has_caveat(plan_basis, stale)
    reopen route because caveated(plan_basis, stale);
```

`has_caveat` checks content grounds, excluding caveats found only in control
lineage. `caveated` selects the exact observed evidence in those grounds that
currently carries the caveat. It preserves old occurrence identities after
renewal and cites simultaneous causes in observation order. An empty selection
rejects atomically. An explicit extra caveat on a value is not automatically a
caveat on its evidence. See [state caveats](../spec/caveat-state-caveats-0.1.md).

The complete [Trail Rescue source](../game/trail_rescue.cav) demonstrates this
with limited scouting, conflicting reports, timed evidence, frozen decisions
and direct save/resume. Its host only translates envelopes and projects source
bindings. After `npm run build`, run `npm run test:trail-rescue` for the
registered scenarios and seeded invariant checks, and
`npm run test:trail-rescue-page` for desktop and mobile browser flows.

Journal entries expose the source clock as `elapsed` and the decision's frozen
numeric `using` value as `value`. Read `because` for acquisition order; grounds
are sets. Older saves can omit the two new fields, so generic consumers should
treat their absence as unknown. Saved records are checked for internal
consistency; an unsigned save is not proof that its history really occurred.

## What would demonstrate AI usefulness

The tooling enables an experiment; it does not establish that agents write
Caveat more reliably or cheaply than another language. A fair comparison gives
the same model the same task, time/tool budget, and independent input histories.
Compare Caveat against TypeScript with equivalent provenance support. Check
behavior, dependency preservation, valid revisions, repair attempts, token cost,
and execution overhead. Count unsupported claims about evidence as failures.

The [first authoring pilot](AI_AUTHORING_PILOT.md) supplied this guide, language
references, and a task to a fresh agent. Its first candidate passed, and the
original source and independent verification are retained. This demonstrates
documentation sufficiency for that example; broader adoption needs repeated
tasks, comparative measurements, and independent users.
