# Authoring and checking Caveat programs

An agent can use Caveat through the same source, interpreter, and tests as a
human developer. The `caveat-lang` package runs programs and checks them against
scenario files without Rust; a repository checkout also has a native replay
command. These tools do not authenticate evidence, inspect a model's internal
reasoning, or authorize external actions.

## Check a program with the kit

Install the package into a project directory: run `npm init -y`, then
`npm install` the tarball or the published package. The getting-started guide
that ships with the package (`docs/GETTING_STARTED.md`) walks through a first
program. The one-page reference (`docs/REFERENCE.md`) gives the syntax,
outcomes and common mistakes in brief, and the worked example
(`docs/WORKED_EXAMPLE.md`) takes one program through every command.

State what a program should do in a scenario file
([Scenarios 0.1](../spec/caveat-scenarios-0.1.md)) and run it:

```sh
npx --no-install caveat test my.scenarios.json
```

Each scenario starts a new session, sends events and checks expectations:
JSON Pointer paths into the snapshot, an expected acceptance or rejection for
every event, `checkpoint` and `same_as` for records that must not change, and
`resume` to continue from a save. Without being asked, the runner also checks
that every rejected event changed nothing, that a resumed session agrees with
the one it came from, that a final save restores, and that grounds stay within
lineage. It exits 0 when every scenario passes, 1 when one fails and 2 when a
file is invalid; a failure names the step, the path, and the expected and
actual values.

A bare `"rejected": true` expects any `policy` refusal: the program's own
`reject`, or a commit refused as `not_permitted`. Name the code to tell them
apart: `{"origin": "policy", "code": "reject"}`. A refusal the runtime makes
before any rule runs, such as a payload outside its declared range, has origin
`input` and must be named: `{"origin": "input", "code": "bound_exceeded"}`. Origins and codes are listed
in [Dispatch outcomes 0.1](../spec/caveat-dispatch-0.1.md). A `sample` or
`commit` on a history that already holds its declared limit is refused as
`{"origin": "limit", "code": "history_limit"}`, and the session continues, so a
program needs its own guard only to give its own message. Some runtime errors
are deliberately unclassified: they are fatal, and a fatal outcome never
matches an expected rejection.

To see what a program does after each event, open a session from a script:

```js
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

const source = await readFile('thermostat_history.cav', 'utf8');
const runtime = await loadRuntimeFromDirectory();
const session = runtime.open(source);
const outcome = session.dispatch('read', { value: 17 });
console.log(outcome.outcome, session.snapshot().decision_journal);
session.close();
```

`runtime.open` throws a `CaveatError` of kind `load` with the runtime's
diagnostic when a program does not load. `dispatch` returns
`{outcome: "accepted", snapshot}` or `{outcome: "rejected", origin, code,
message}`; anything else throws, and the session must then be discarded.
`session.save()` returns text that `runtime.restore(source, saved)` resumes.

## Replay with the native command line (repository checkout)

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

Use `#` or `//` for line comments outside strings; `--` is not a comment marker.
The [source-text profile](../spec/caveat-text-0.1.md) specifies comments and quoting.

1. Read the event contract and the relevant reactive profile. Keep observations,
   proposed policies, physical conditions, and committed decisions distinct.
2. Write a small program and a scenario file with an explicit event history.
   Mark which inputs are observations and which are ordinary controls. An
   evidence source label is descriptive metadata, not an authentication
   mechanism. When several inputs follow the same rules, write the rules once
   as a procedure ([below](#share-rules-with-a-procedure)).
3. Run the scenarios, and inspect `qualified_values`, `reading_streams`,
   `commitment_bases`, `decision_series`, and `relations` in the snapshots,
   from a script or with the native replay. Run `caveat check` on the source
   for patterns worth a second look ([check](../spec/caveat-check-0.1.md)); a
   warning is advice, not a failure.
4. Exercise missing observations, contradictory readings, repeated equal
   readings, reopened decisions, and failures late in an event.
5. Change one policy and run the same scenarios. Check both the changed
   decision and the records that should remain unchanged.

Plan numeric ranges before writing state accumulators. State and numeric
event bounds must be finite, ordered, and within the inclusive hard limit
`-1e12..1e12`; an explicit larger `max` is invalid even when the initial value
fits. For a state accumulator such as `total = total + dt`, choose its units,
maximum supported duration, and representation so every update fits the
declared range. If the required duration would overflow it, redesign the
representation before coding. Exceeding the range rejects the whole event,
without advancing that accumulator. For session time, read `elapsed()` rather
than maintaining a second clock in bounded state. Load the program early and
test boundary cases.

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

## Share rules with a procedure

When several inputs follow the same rules, such as two instruments measuring
the same thing, write the rules once as a `proc` and call it once for each
input:

```caveat
claim frost_risk;
evidence probe from "a soil probe";
evidence drone from "a survey drone";
caveat uncalibrated consequence material;
uncalibrated qualifies drone;
readings soil from probe limit 12;
readings aerial from drone limit 12;
decisions uncover limit 4;
event probe_read celsius min -40 max 60;
event drone_read celsius min -40 max 60;

proc record(s readings, d decisions, celsius) {
    when celsius <= 2 sample s = celsius supports frost_risk;
    when celsius > 2 sample s = celsius opposes frost_risk;
    when celsius <= 2 and committed(d) and not reopened(d)
        reopen d because latest(s);
};
on probe_read call record(soil, uncover, celsius);
on drone_read call record(aerial, uncover, celsius);
```

A parameter followed by `readings` names a reading stream, and one followed by
`decisions` names a decision series. `evidence`, `claim` and `caveat` name
those symbols. A parameter with no kind is a number, frozen when the procedure
is called.

Each call passes names fixed in the source, so the program loads one copy of
the procedure for each set of names, named `record[soil, uncover]` in
diagnostics. That copy runs exactly as the rules would if written out by hand.
Each stream keeps the evidence, caveats and limit of its own declaration: here
only aerial readings carry `uncalibrated`. A procedure's steps run in order
and see the effects of earlier steps in the same event, and a `reject` inside
one refuses the whole event. When a change adds another instrument, it adds
declarations and a call, not another copy of the rules. See
[procedure symbols](../spec/caveat-procedure-symbols-0.1.md).

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

When an observation turns out to be wrong, such as a test result read
incorrectly, withdraw it rather than recording a contrary one:

```caveat
on misread reveal recheck supports misreading;
on misread withdraw latest(checks) because recheck;
on misread when committed(merge) and not reopened(merge) and rests_on_withdrawn(merge)
    reopen merge because recheck;
```

The withdrawal is recorded with its reason. Current values, and any later read
of the reading, carry a `withdrawn` caveat. The decision keeps what it was made
on, and reopens only because a rule says so. See
[withdrawal](../spec/caveat-withdrawal-0.1.md).

When a decision also needs someone's permission, say what permits it rather
than folding the permission into the value it rests on:

```caveat
on approved sample approvals = commit supports may_merge;
on merge commit merge because enough using latest(checks)
    permitted by latest(approvals) for head;
```

Without a go-ahead for this head, the merge is refused as
`{"origin": "policy", "code": "not_permitted"}`. The decision records its grant
apart from its grounds. The check is made when the decision is made. See
[permission](../spec/caveat-permission-0.1.md).

When every new reading of some kind must reopen a decision, declare it on the
series instead of writing a reopening rule after each such reading:

```caveat
decisions merge limit 8 reopened by pushes, checks opposing ready;
```

Any further condition, such as "unless it was merged elsewhere", still needs its
own rule. See [reopening triggers](../spec/caveat-reopening-triggers-0.1.md).

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

Two later trials gave fresh agents only the packaged release candidate and a
task nobody had attempted. In [v3](../experiments/agent-authoring/v3/RESULTS.md)
two authors wrote programs, and two more agents each carried out a sealed
change request on one of them; all four passed their registered cases.
[v4](../experiments/agent-authoring/v4/RESULTS.md) repeated that design with a
new task and eight agents on separate machines, using two models; all eight
passed. Neither trial estimates how often an agent succeeds. The procedure
section above was added after v4, in which every author wrote parallel
instruments' rules out by hand.
