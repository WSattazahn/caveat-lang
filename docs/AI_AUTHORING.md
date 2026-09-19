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
