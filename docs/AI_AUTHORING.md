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

For integrations that need to distinguish authored refusals from invalid inputs
and execution failures, use the [structured dispatch API](../spec/caveat-dispatch-0.1.md).
`dispatch_outcome` returns accepted/rejected reports. Bare rejection assertions
mean an authored policy `reject`; other origins must be named. Every thrown
error is fatal and requires discarding the session. Some existing runtime errors
are deliberately still unclassified and fatal in this API; legacy dispatch
methods retain their existing behavior. The initial code catalog is in the spec.

Use `#` or `//` for line comments outside strings; `--` is not a comment marker.
The [source-text profile](../spec/caveat-text-0.1.md) specifies comments and quoting.

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

Plan numeric ranges before writing state accumulators. State and numeric
event bounds must be finite, ordered, and within the inclusive hard limit
`-1e12..1e12`; an explicit larger `max` is invalid even when the initial value
fits. For a state accumulator such as `total = total + dt`, choose its units,
maximum supported duration, and representation so every update fits the
declared range. If the required duration would overflow it, redesign the
representation before coding. Exceeding the range rejects the whole event,
without advancing that accumulator. For session time, read `elapsed()` rather
than maintaining a second clock in bounded state. Validate early and replay
boundary cases.

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

## Read the session clock

```caveat
event advance dt min 0 max 3600;
clock advance every 1;
bind hud.elapsed = elapsed();
```

[`elapsed()`](../spec/caveat-elapsed-0.1.md) reads the runtime's existing session
time in seconds. It starts at zero and adds each accepted clock event's `dt`
before scheduled qualifications and rules run. An explicit `clock` selects the
time event; otherwise `tick` advances it. With neither, it remains zero.
Rejected events roll back the clock along with every other effect.

Use it in guards, bindings, `define` expressions and procedures, or pass it to
a pure function such as `time_text(elapsed())`. Pure functions cannot capture
the clock themselves. A clock read introduces no evidence of its own; the
usual qualification of enclosing expressions and guards still applies.

The clock is finite binary64 and reading it has no state duration cap. Copying
it into a state still enforces that state's bounds. Use direct reads for
display and current-time comparisons; keep bounded state only for values the
policy needs to retain. Saves already preserve the clock, and restored
bindings read that saved value. A custom clock admitting negative `dt` can
move backward, so assume monotonic time only when its declared bounds ensure it.

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

## Evidence from fresh authors

The [first authoring pilot](AI_AUTHORING_PILOT.md) supplied this guide, language
references and a task to one fresh agent. Its first candidate passed; the
original source and independent verification are retained.

A later [six-context study](../experiments/agent-authoring/v1/RESULTS.md) used
three preregistered tasks, two authors per task, three source versions and
twelve runtime checks per author. Authors received no private-test feedback.
Two first submissions and all six final submissions passed the registered
corpus, including exact grounds, journal order, atomic rejection and restore.
All four first-source failures involved the numeric declaration ceiling; the
guidance above was clarified afterward, with the original packet preserved.

Two final sources still reject valid advances beyond their accumulated clock
cap, and the other two clock representations have untested large-time paths.
Those limits stay in the results. The evidence supports authoring these
specified policies with self-directed repair; it does not establish complete
correctness, reliability across models, or an authoring cost advantage. Keep
independent policy checks and explicit duration/capacity contracts in the loop.

A [four-context follow-up](../experiments/agent-authoring/v2/RESULTS.md) repeated
the two clock tasks with `elapsed()` and the updated guide. Three first sources
and all four final sources passed the registered corpus. Every final source
read the runtime clock directly, removing the separate clock representations
seen in v1. The one first-source error used unsupported `--` comments; its
author repaired it without private feedback. The comment reminder above was
added after this cohort and is not part of its frozen packet. These are small,
repeated-task results for the runtime and documentation together; they do not
isolate causality or establish reliability on new tasks.
