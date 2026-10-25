# Authoring and checking Caveat programs

An agent can use Caveat through the same source, interpreter, and tests as a
human developer. The `caveat-lang` package runs programs and checks them against
scenario files without Rust; a repository checkout also has a native replay
command. These tools do not authenticate evidence, inspect a model's internal
reasoning, or authorize external actions.

If a real integration exposes a missing capability, please
[open an agent feature request](https://github.com/WSattazahn/caveat-lang/issues/new?template=agent-feature-request.md)
for the project owner to evaluate. Check related issues and the existing
language and kit facilities first. Include a small reproduction, exact version,
and expected versus actual behavior. Distinguish a language limitation from
authored policy, invalid input or host permissions. Share only with the user's
permission when the example contains private material; synthetic examples work too.

## What the language refuses to do for you

Programmers coming from general-purpose languages tend to expect the runtime to
do these things. Caveat leaves each one to a rule the author writes.

- **A caveat carries no logic.** `caveat C consequence …; C qualifies E;`
  says that evidence `E` carries `C`, and nothing more. A caveat never fires,
  expires or checks a condition. Whatever should make `C` matter, such as a
  time, a count or a reading, is a rule: `on EVENT when … qualify E with C`,
  or `qualify E with C after N`. Its consequence level changes no outcome
  either: the runtime reports it in the snapshot's `symbols` and no rule or
  outcome reads it. To make a caveat change what happens, write a rule that
  reads it, such as
  `on approve when has_caveat(tests, C) reject "tests carry C";` or
  `on recheck reopen D because caveated(tests, C);`.
- **There are no records, lists or loops.** State is a number and `fn` is
  one pure expression. Several similar things, such as four mushrooms or six
  sensors, are entities of one kind, and their rules are written once in a
  `for KIND as $NAME { … };` block, which expands to one copy per member
  ([repetition](../spec/caveat-repetition-0.1.md)).
- **There is no fallback.** Nothing is withdrawn, reopened or replaced
  because something else happened. A withdrawal is an explicit effect with a
  reason (`withdraw E because F`), and a reopening is explicit and has a
  reason too (`reopen D because F`). A reopened decision keeps the grounds it
  was made on; the next commit of the series gets its own.
- **Grounds are frozen per revision.** `commitment_grounds` holds, for each
  revision such as `stance@1` and `stance@2`, the evidence and caveats it
  rested on when it was committed. Later evidence never rewrites them.

## What grows the save

A state's numeric slot holds one current number, but its grounds and lineage
can retain growing history. Evidence occurrences, reading occurrences, renewals
and journal entries also consume storage. A history declared with `limit`
retains its recorded items and refuses additions at its capacity. A `window`
retires older records; rc.16 departure and collection can remove eligible
retired records from the session while the host retains their archive.

Windows do not give a universal save bound: exact own grounds, reachable
withdrawal-reason chains and conservative retention can keep older records.
Compaction can bound particular lineage representations without bounding every
required dependency. Live retained memory, temporary peaks, save size and host
archive storage are separate costs. See the
[history integration guide](../kit/docs/HISTORY.md) for draining, exact versus
conservative explanations and save/restore responsibilities.

A save names everything it holds, so a small program's state save can be
several times the size of its events. `session.save({ compact: true })` writes
a [replay save](../spec/caveat-replay-save-0.1.md), the session's accepted
events, while they fit in 4,096 bytes and that is smaller, and otherwise the
state save; `runtime.restore` takes either. Restoring a replay save dispatches
its events again, so it costs more as they grow. A runtime before 1.1.0 cannot
restore one.

`session.save({ compact: true, positional: true })` writes the
[positional replay save](../spec/caveat-replay-save-0.2.md) instead: each event
as its position and its parameters' values, written as one base-36 digit each
when every value is a whole number below 36, so it stays a replay save for
several times as many events. Prefer it to keeping
your own event log. A runtime before it cannot restore one.

Its bound is counted on the list form, where each value takes its digits and
a comma, so it holds about 4,096 / (2 × (parameters + 1)) events whose values
are single digits. After that the save is the state save, which is far
larger. For the [authoring card](../kit/docs/AUTHORING_CARD.md)'s program, the
positional save holds 512 three-parameter events (2,150 bytes); after 514 the
save is a 34,488-byte state save, about 64 bytes more for each further event
(measured on 1.2.0). Events a host can send without end, such as flag and
unflag, need a cap when its store has a size limit.

A decision series cannot have a window: `decisions NAME window N` is refused
when the program loads, and series windows wait until after 1.0
([Windows 0.1](../spec/caveat-windows-0.1.md#decision-series-not-in-rc15)).
Every revision stays in the save with its frozen grounds, and each revision's
lineage holds its predecessor's basis, so a program that recommits for as long
as it runs grows its save with every recommit. Cap the series: its `limit` is
the cap, and a guard such as `when history_count(trust) < 64` keeps the commit
from being refused as `limit/history_limit`, which would refuse everything
else the event did. Decide what the program shows once the cap is reached.
The cap bounds the series, not its cost: up to that many revisions, each with
its grounds, stay in every save. All three Caveat authors in Glowcap round 8
capped a recommitted series this way, at 64 or 256
([results](../experiments/glowcap/RESULTS.md#round-8-fresh-authors-blind-requests-rc17)).

So let rules decide when
inputs become evidence, and do not let the frame rate decide. For example,
`on hit set hits = hits + 1;` followed by
`on hit when hits >= 3 and not observed(rhythm) reveal rhythm supports …;`
records one observation, while sampling every frame records one occurrence
per frame. Glowcap round 7's save-bound failures came from this
([results](../experiments/glowcap/RESULTS.md)).

One decision series per exclusive choice keeps a choice exclusive. Put the
choice in the series' value (`using` 1 for parry, 2 for dodge), and the
runtime refuses a second commit while one is in force as
`evaluation/decision_in_force`, until a rule reopens it with a reason
([boss-stance](../kit/examples/boss-stance/boss.cav)).

## What a guess gets wrong

A model that was told only what Caveat is for, and asked to guess its syntax
(Glowcap round 8's learnability probe), wrote these. Each is a habit from
general-purpose languages:

- `caveat NAME;`. A caveat declares its consequence:
  `caveat NAME consequence LEVEL;`.
- Records, lists, loops and text in state. Use entities of one kind and a
  `for` block, as above.
- `because` lists and caveat unions computed by hand. A value already carries
  them; cite it with `because`, and run `caveat explain` to see them.

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

An expectation compares scalars exactly. An array must have the same length
and match element by element, in order, so an expected list of grounds names
every one in the order the snapshot lists it. An object matches partially:
the members it lists must match, and others are ignored, at every depth. To
match an array in any order write `{"$set": [...]}`, and to require only some
elements write `{"$includes": [...]}`
([Scenarios 0.1](../spec/caveat-scenarios-0.1.md)).

A bare `"rejected": true` expects any `policy` refusal: the program's own
`reject`, or a commit refused as `not_permitted`. Name the code to tell them
apart: `{"origin": "policy", "code": "reject"}`. A refusal the runtime makes
before any rule runs, such as a payload outside its declared range, has origin
`input` and must be named: `{"origin": "input", "code": "bound_exceeded"}`. Origins and codes are listed
in [Dispatch outcomes 0.1](../spec/caveat-dispatch-0.1.md). A `sample` or
`commit` on a history that already holds its declared limit is refused as
`{"origin": "limit", "code": "history_limit"}`, and the session continues, so a
program needs its own guard only to give its own message. The same holds for a
`commit` to a decision series whose current revision has not been reopened: it
is refused as `{"origin": "evaluation", "code": "decision_in_force"}`. So is
an event that acts on evidence no event has observed (`unobserved_evidence`),
reopens a decision never committed (`not_committed`), fails an arithmetic or
history read such as a division by zero or `latest` of an empty history
(`expression`), or fails a `require` (`requirement_failed`). A clock event
whose sum would make the session clock nonfinite is refused as
`{"origin": "evaluation", "code": "bound_exceeded"}`, and a `qualify … after`
while 4,096 qualifications are already waiting as
`{"origin": "limit", "code": "scheduled_limit"}`. An event that would give a
value a lineage of more than 1,024 identifiers is refused as
`{"origin": "limit", "code": "provenance_limit"}`, and one in which an
expression's evaluation would visit more than 65,536 nodes, as a
`fold_history` over a long stream with a heavy step can, as
`{"origin": "limit", "code": "evaluation_limit"}` (before 1.4, both were
fatal). Other runtime errors are deliberately unclassified: they are fatal,
and a fatal outcome never matches an expected rejection.

A plain `commit`, made without a `decisions` declaration, can be made only
once: committing it again, even after it was reopened, is refused as
`{"origin": "evaluation", "code": "already_committed"}` (before 1.1, it was
fatal). Declare a decision series (`decisions NAME limit N;`) when a decision
may be made again after it is reopened, or guard the rule with
`on EVENT when committed(NAME) reject "…";` to give your own message.

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
`session.save()` returns text that `runtime.restore(source, saved)` resumes,
and `session.save({ compact: true })` a smaller save for a short session (see
[What grows the save](#what-grows-the-save)). Use the compact save rather than
writing your own event log: it holds exactly the events the runtime accepted,
and `runtime.restore` takes it only with the source it came from.

Choose the call by what the host reads after each event. `dispatchView`
returns the view (displayed values, decisions and cues) and is the cheapest:
the runtime does not build the snapshot. Read the view it returns rather than
calling `view()` after it, which builds the same view again. When the host
reads only a few of the view's fields, `dispatchView(event, payload, { fields })`
builds just those: `{ fields: ['bindings', 'binding_explanations'] }`, or add
`commitments`, `decision_series` and `decision_journal` for `decisionHistory`.
Name the fields by default. `relations`, `binding_explanations` and
`decision_journal` grow with the session, and building and parsing a view the
host does not read can cost more than the event itself.
`dispatchViewDelta`
returns what changed in a View 0.2. It builds the whole view after each event
and compares it with the one it kept from the last, so it costs about what the
full view does and is not a cheaper way to read it. `dispatch` builds the full snapshot, the
largest thing the runtime returns; use it when the host needs the journal,
grounds or graph. `session.snapshot()` straight after `dispatch` reuses that
snapshot instead of building another. A decision's basis is in the view:
`decisionHistory(view, name)` lists what each revision was committed on, so an
adapter that reads bases does not need `dispatch` and the snapshot for it.

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

`caveat-map validate` is another native command of a repository checkout; the
npm package does not ship it. It checks declared map structure and reports
`validation_scope: "declared_map"`. It does not validate reactive execution.
Use `caveat-reactive validate`, or `caveat validate` from the package, for
reactive source. Pure function extraction
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
   warning is advice, not a failure. `caveat test` runs the same check on each
   program a scenario file names and prints its warnings with the run, without
   stopping it. To make a warning fail a gate, run `caveat check --strict`,
   which exits 1 when there is one. Glowcap round 8's mutation harness
   counted only programs that failed to load, so a C006 warning on each of
   its 215 M5 mutants went uncounted.
   After a rewrite, run `caveat check --unused` too: it reports a claim,
   evidence, caveat or state that nothing else in the program names, which
   the plain check does not.
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

Symbol arguments are bare declared names. An `e evidence` parameter cannot take
`latest(soil)`: pass `soil` to `s readings` and use `latest(s)` in the body.
The [paired procedure examples](../kit/examples/procedure-handles/README.md)
test this distinction, including the actual reopening reasons. Replacing the
reading with its evidence template would name different evidence.

Each call passes names fixed in the source, so the program loads one copy of
the procedure for each set of names, named `record[soil, uncover]` in
diagnostics. That copy runs exactly as the rules would if written out by hand.
Each stream keeps the evidence, caveats and limit of its own declaration: here
only aerial readings carry `uncalibrated`. A procedure's steps run in order
and see the effects of earlier steps in the same event, and a `reject` inside
one refuses the whole event. When a change adds another instrument, it adds
declarations and a call, not another copy of the rules. See
[procedure symbols](../spec/caveat-procedure-symbols-0.1.md).

A call passes its guard's and every argument's evidence and caveats to
everything the procedure does, samples included, even when the body never
reads that argument ([evaluation order](../spec/caveat-reactive-0.7.md#evaluation-order-and-qualification)).
Passing a control flag such as `frozen` into a procedure that samples
therefore puts the flag's lineage on every new reading. If the flag was set
under a guard that read a withdrawn report, each later reading then carries
`withdrawn` as well. Pass a procedure only what its steps read. Keep a
`sample` in a procedure that does not take the flag, or in its own rule, and
test the flag in the calling rule's `when`:

```caveat
proc take(s readings, outcome) {
    sample s = outcome supports passing;
};
on report when frozen == 0 call take(checks, outcome);
```

The guard's lineage still reaches what the call does, as it does for any
guarded effect, but only for that one event.

## Compare several candidates

When a decision compares several candidates, such as hosts, packages or
runtimes, that each get the same readings, declare the candidates once and let
`for KIND as $NAME` write a stream and a routing rule for each. One event then
carries the candidate's name and lands in that candidate's stream:

```caveat
claim fit;
evidence probe from "a version probe";
place machine kind host;
entity n20 kind runtime at machine;
entity n22 kind runtime at machine;
event observe_present which kind runtime, value min 0 max 1;

for runtime as $r {
  readings $r_present from probe limit 4;
  on observe_present when which == which.$r sample $r_present = value supports fit;
};
```

This declares `n20_present` and `n22_present`, and
`{"event":"observe_present","payload":{"which":"n22","value":1}}` records
`n22_present@1`. A new candidate is one more `entity` line, not another copy of
the rules. The `place` line only gives the entities somewhere to be; outside a
game it has no other meaning. Use a procedure, as above, when the inputs are
different events rather than one event naming its subject. See
[repetition](../spec/caveat-repetition-0.1.md).

When a rule is about a pair, such as a candidate and one of its checks, put a
block over the second kind inside the block over the first, from 1.2.0. The
inner block expands in each copy of the outer one, and its body may use both
names; `$index` is the inner member's position:

```caveat
for component as $c routed by component {
  for slot as $k {
    state $c_ack_$k = 0 min 0 max 1;
    on acknowledge when decision == $index set $c_ack_$k = 1;
  };
};
```

This declares `api_ack_d1` through `worker_ack_d8`, and the route selects the
component on each `acknowledge`. See
[nested repetition](../spec/caveat-nested-repetition-0.1.md).

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

A `qualify … after` applies on the first clock event at which the clock has
moved by at least its delay in binary64, so rounding can move it by an event.
For timing exact to the event, use clock steps that are powers of two, such as
`0.0625`; otherwise expect a schedule to apply within an event of its nominal
time.

From rc.15, a program that needs timing exact to the event can declare an
[integer clock](../spec/caveat-elapsed-0.1.md#integer-clocks) instead. (The
runtime accepts the declaration from rc.15's PR 3; check the package version.)

1. Choose the unit so that every delay and every `dt` the host sends is a
   whole number of it. Glowcap's `tick` in `0..0.1` seconds becomes a `step`
   event with `dt` in `0..100` milliseconds, `clock step every 16 integer`,
   and `after 60` becomes `after 60000`. `tick` itself cannot be an integer
   clock.
2. Convert in source wherever time is shown or compared: bindings and guards
   that read `elapsed()` or a `dt` divide by the unit, as in
   `fn seconds(t) = t / 1000;`. A computed `after` must come out whole, or the
   event is refused as `evaluation/expression`.
3. The host sends whole units. If its frame time is not a whole number of
   units (16.67 ms), it rounds once, at its own boundary, and carries the
   remainder into the next frame: keep `carry += frame_ms`, send
   `dt = Math.floor(carry)` and subtract what was sent. A fractional `dt` is
   refused as `input/payload_invalid`, so rounding never happens inside the
   clock.
4. Saves do not carry over: the edit changes `source_id`. A host that must
   keep a session replays its recorded events with `dt` converted, which works
   only when every recorded `dt` was a whole number of the new unit.

The reading stays within `±(2^53 − 1)` units; a clock event that would leave
that range is refused as `evaluation/bound_exceeded`. At a millisecond unit
that is about 285,000 years.

A countdown a view shows is a place where this rule becomes visible. A
binding such as `bind m.fades_in = ceil(60 - (elapsed() - tasted_at));` over
a fractional clock can show one second more at a boundary than a timer
counted per event, because the accumulated sum lands just short of the whole
second. In Glowcap round 8, two Caveat programs that counted down from the
accumulated clock differed from TypeScript programs counting per timer this
way after ticks of 0.05, 0.1 or 0.2 seconds. The round inferred the cause;
on rc.17, that binding with steps of 0.1 showed one second more than a
millisecond count on 13 of 600 steps. A countdown that must agree to the tick
uses an integer clock.

Without `integer`, the clock is finite binary64 and reading it has no state duration cap. Copying
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

A reading found wrong after later readings arrived is no longer the latest.
Name it by its position instead, counted from 1, and ask whether the decision
in force rested on it (from 1.1.0):

```caveat
event retract i min 1 max 64;
on retract withdraw reading(checks, i) because recheck;
on retract when committed(merge) and not reopened(merge)
    and in_basis(merge, reading(checks, i))
    reopen merge because reading(checks, i);
```

`reading(STREAM, N)` names the stream's Nth reading wherever `latest(STREAM)`
does in `withdraw`, `qualify`, `unqualify` and `reopen … because`, and in
`observed`, `withdrawn` and `carries`. An N that names no reading the stream
holds refuses an effect as `expression` and makes a predicate false.
`carries(latest(STREAM), C)` asks whether the current reading carries `C`,
for example in a guard that refuses to flag a reading twice. To ask whether
it carries any caveat at all, write `carries_any(latest(STREAM))`, or
`carries_any(latest(STREAM), material)` for any of consequence `material` or
above, `withdrawn` included, rather than listing caveats by name: a caveat
declared later is then covered too (1.4 draft,
[carries any](../spec/caveat-carries-any-0.1.md)).
`in_basis(DECISION, E)` is true when the decision's current revision rests on
`E`, `latest(STREAM)` or `reading(STREAM, N)`; `in_basis(revision(SERIES, K), …)`
asks the same of the Kth revision. A later reading is never in an earlier
decision's basis. See [reading index](../spec/caveat-reading-index-0.1.md) and
[in basis](../spec/caveat-in-basis-0.1.md).

When a reading's source said something in words, such as a version range,
keep the words with the number: `sample node_min = minimum supports compatible
noting range;`, where `range` is an `id` parameter. Decisions still compare the
number. See [reading notes](../spec/caveat-reading-notes-0.1.md).

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

A trigger fires on every reading, including one that repeats the last. When a
host re-checks by taking every reading again, write `changed` before the
stream (from 1.1.0, [Changed Triggers 0.1](../spec/caveat-changed-triggers-0.1.md)):
the trigger then fires only on a reading whose value differs from the stream's
previous one, so the journal names the reading that changed.

```caveat
decisions install limit 4 reopened by changed n20_present;
```

When carrying out a decision can't be undone, such as dropping a table or
publishing a package, declare it irreversible and name what it rules out:

```caveat
decisions drop_old_table limit 1 irreversible forecloses rollback;
```

Its commit is refused as `{"origin": "evaluation", "code": "caveated_irreversible"}`
when it rests on a caveat of consequence `material` or worse that it does not
name in `retaining`, so a withdrawn reading never grounds it. Once it is made,
a commit of `rollback` is refused as `foreclosed`, and the message names
`drop_old_table@1`. It may still be reopened: that records doubt and keeps the
foreclosure. Caveat records the declaration; it does not check that the action
really can't be undone. See [foreclosure](../spec/caveat-foreclosure-0.1.md).

That check reads only the commit's grounds. Evidence that a `when` guard or a
`permitted by` grant read is not checked, so a payout gated on
`support >= 3` can rest on an unchecked email without naming it. Write
`irreversible strict` (from 1.4, [strict irreversible](../spec/caveat-strict-irreversible-0.1.md))
to check every caveat in the commit's lineage:

```caveat
decisions payout limit 1 irreversible strict;
```

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

## Keep a decision in force for a while

From 1.2.0, a hold keeps a decision series' current revision in force without
dropping what would have reopened it. While it is on hold, a reopening by a
rule or a `reopened by` trigger is deferred and recorded; `release` ends the
hold and reopens the revision because of every deferred cause:

```caveat
decisions gate limit 8 reopened by tests;
on freeze when on_hold(gate) reject "already frozen";
on freeze hold gate because team;
on unfreeze release gate because team;
```

Do not write a freeze as a state that guards each reopening rule: a guarded
rule drops the reopening, and a trigger has no guard. A hold never keeps a
revision that rests on withdrawn evidence: a reopening of it goes ahead. See
[holds](../spec/caveat-holds-0.1.md).

## Record how a decision turned out

From 1.3.0, `resolve` records an outcome on the revision it judges, so a
review can tell a sound basis that met bad luck from an unsound one. It
changes no basis, reopens nothing and takes no stance; `materialized` names
the revision's retained caveats that came true:

```caveat
decisions pilot limit 4;
on close resolve pilot = users because latest(usage) materialized thin_demand;
on late resolve revision(pilot, k) = users because latest(usage);
```

Do not record an outcome as a reading that `supports` the decision's claim:
an outcome is what happened, not evidence for the claim. Read it back with
`outcome(pilot)`, guarded by `resolved(pilot)`, next to `latest(pilot)` to
score a forecast. A wrong outcome is corrected by withdrawing its evidence,
then resolving again. See [outcomes](../spec/caveat-outcomes-0.1.md).

## Citations and named expressions

Grounds and caveats are not bookkeeping the author writes. A value carries the
evidence and caveats it was computed from, a commit freezes its grounds, and
the decision journal records each change, so a `because` list or a caveat
union kept by hand, in the source or in the host, is a second copy that can
drift from the first.

A `because` citation is an expression whose provenance the value or its
conditions actually read. A bare claim such as `intent_clear` is a known
symbol of the wrong kind, not an unknown name. Cite the state, `latest(STREAM)`
or an appropriate evidence-bearing expression. Choosing a valid expression
does not bypass the check against the original lineage.

`define freshness_last = latest(freshness);` names an expression. It does
not take parentheses or parameters. `fn` is for pure functions with explicit
arguments; it cannot capture state, history or evidence. Pass values in when
that is the computation you intend.

For scenario checks, assert the decision and numeric score structurally.
A display prefix such as `Proceeding` alone cannot distinguish confidence
68 from 12. Exact text is appropriate when the text itself is the contract.
No new string matcher is part of this candidate.

## Consulting a source without taking a stance

In rc.7, `on consult reveal memory;` observes the current evidence occurrence
without claiming that it supports or opposes anything. `observed(memory)` then
becomes true and `qualified(value, memory)`, late qualification and withdrawal
can use it. Sampling from a source still does not observe the source itself.

A neutral reveal does not assess, reopen a decision, or infer trustworthiness.
Existing stance-bearing reveals remain supported. The
[neutral reveal contract](../spec/caveat-neutral-reveal-0.1.md) describes
idempotence, guard provenance, renewal and old-save compatibility. Explanations
show neutral evidence as observed with no stance, rather than inventing a claim.

A reading that bears on no claim, such as a tool's version number, is a
neutral sample (from 1.1.0, [Neutral Sample 0.1](../spec/caveat-neutral-sample-0.1.md)):

```caveat
evidence node_version from "node --version on the build machine";
readings node from node_version limit 8;
event inventory major min 0 max 99;
on inventory sample node = major;
```

`node@1` is observed and can be read, qualified, withdrawn and cited like any
reading, without a claim declared only to give it something to support. A
`reopened by node` trigger fires on it; a trigger filtered by
`supporting|opposing CLAIM` does not.

## Repeated assessments and late qualifications

The [agent-evidence starter](../kit/examples/agent-evidence/README.md#assessing-again)
separates duplicate calls, new evidence for the same decision, corrections and
new tasks. Its executable examples compare opposing-only and every-observation
reopening policies. Neither policy is the default for every application.

Its [qualification example](../kit/examples/agent-evidence/QUALIFICATION.md)
shows why a template, current template-dependent value, archived reading and
frozen decision grounds can carry different caveats. Reassessment does not
resample. Its custom caller checks exact required occurrence IDs and makes
memory carry-over an explicit application policy. An accepted operation and
a newly committed decision alone do not establish the right grounds.

For hypotheses, see [branching with a separate runtime](../kit/examples/agent-evidence/BRANCHING.md).
A second session sharing a WASM instance cannot isolate a trap.

## Record "not ready" with its reasons

A gate whose honest answer is "not ready" can keep that answer with its
grounds as a decision of its own. `citing` gives a decision grounds without a
value ([Citing 0.1](../spec/caveat-citing-0.1.md)): its grounds are what each
citation reads, and its value is `null`.

```caveat
decisions ship limit 4;
decisions hold limit 4;
on assess when ci == 1 and build == 1 commit ship because enough using ci + build;
on assess when not (ci == 1 and build == 1) commit hold because enough citing ci, build;
```

Here `ci` and `build` hold `qualified(…)` readings of the `ci_run` and
`build_check` evidence. After a failed CI run and a passing build, `explain`
prints `hold@1 = null` and `based on build_check, ci_run`. The verdict is which
series is committed, here `hold` and not `ship`: bind what a reader sees to
`committed(hold)`. A citing decision has no value to read, so `latest(hold)`
refuses as `evaluation/expression`. A commit with neither `using` nor `citing`
has no grounds at all: `explain` prints "based on nothing".

Before 1.1, the same hold was written `using ci + build`, a sum that only read
both readings. That still works, but its value is not a verdict and readers
have to be told to ignore it.

## Record a caveat you chose not to pursue

Give the reason its own evidence, and name it after the caveat in
`retaining` ([Retention Reasons 0.1](../spec/caveat-retention-reasons-0.1.md)).
Each caveat keeps its own reason, so one decision can leave several caveats
for different reasons:

```caveat
evidence scope_note from "the release plan: the game does not gate 1.0";
evidence d1_only from "draft 179 only matters for option B";
caveat presentation_gaps consequence low;
caveat pr179_drift consequence low;
decisions defer_polish limit 2;
on triage reveal scope_note supports can_wait;
on triage reveal d1_only supports can_wait;
on triage commit defer_polish because enough
  retaining presentation_gaps because scope_note, pr179_drift because d1_only;
```

`explain` then prints `based on d1_only, scope_note` and
`retaining: pr179_drift because d1_only, presentation_gaps because scope_note`
for `defer_polish@1`, and `dependents PROGRAM presentation_gaps` finds the
decision. A reason must be observed when the commit runs, and it joins the
decision's grounds: withdrawing it makes `rests_on_withdrawn(defer_polish)`
true. `because` after the series takes only `enough`, `budget` or `deadline`,
which are stopping rules, so "out of scope for now" is written
`because enough`. A caveat retained without a reason is listed only when no
grounds observation carries it; a caveat the evidence brought with it is
shown beside that evidence instead.

Before 1.1, the reason went into the grounds with
`using qualified(1, scope_note)`, and two caveats left for two reasons needed
two decisions. That still works, but nothing in the record pairs a caveat with
its reason.

## Act on text from a source that may turn out bad

Draft for 1.4 ([Untrusted Sources 0.1](../spec/caveat-untrusted-sources-0.1.md)).
An agent that acts on text it read, such as an instruction in an email, gives
that text a caveat and every action resting on it carries the caveat. Three
forms keep such a program short:

```caveat
caveat untrusted consequence high;
evidence mail from "the newsletter as the agent read it";
evidence mail_bad from "a finding that the newsletter is malicious";
evidence user_ok from "the user confirming one pending send";
renewable user_ok limit 64;
untrusted qualifies mail;
readings asks from mail limit 8;
decisions add_event limit 8;
decisions send limit 1 irreversible;
on confirm when observed(user_ok) renew user_ok;
on confirm reveal user_ok;
on act commit send because enough using latest(asks)
    retaining untrusted because once user_ok;
on found reveal mail_bad;
on found withdraw every mail because mail_bad;
on found when committed(add_event) and not reopened(add_event) and rests_on_withdrawn(add_event)
    reopen add_event because mail_bad;
```

- `because once user_ok`: a confirmation backs one commitment. A second send
  before the next `confirm` is refused as `evaluation/reason_spent`. Without
  `once`, one confirmation covers every later send.
- `withdraw every mail`: the text and every reading sampled from it are
  withdrawn in one step, not only the latest.
- `caveat undo-list PROGRAM EVENTS`, or `undoList(snapshot)` from
  `caveat-lang/explain`, lists every revision, current or past, whose grounds
  hold withdrawn evidence. The program reopens the current one; the list is
  for the host to undo what already went out, and marks irreversible ones.

When an action takes two sources, such as an instruction from one email and a
recipient from another, nest a routed `for` per event parameter; each
decision's grounds then name both readings
([section 1](../spec/caveat-untrusted-sources-0.1.md#1-one-action-on-several-sources)).

When several sources vote and a decision should rest on the ones that agreed,
add one `only_if(AGREES, VOTE)` term per source to its `using`. A condition in
`if(...)` would make every vote it read a ground, dissenters included.
`only_if`'s condition is lineage only, as a rule's `when` is. Draft for 1.4
([Only If 0.1](../spec/caveat-only-if-0.1.md); the
[model-votes experiment](../experiments/model-votes/README.md)).

## What a withdrawal adds to a decision's grounds

A withdrawal needs a reason: `withdraw latest(checks) because recheck`. Every
`withdrawn(...)` read carries that reason as evidence, so a decision whose
`using` asks whether a reading is withdrawn is grounded on the reason too:

<!-- withdrawal-grounds -->
```caveat
scene "A withdrawn check counts as zero.";
claim fine;
evidence probe from "the probe";
evidence recheck from "a recheck found the result wrong";
readings checks from probe limit 4;
decisions gate limit 4 reopened by checks;
event report value min 0 max 1;
event misread;
event decide;
on report sample checks = value;
on misread reveal recheck;
on misread withdraw latest(checks) because recheck;
on decide commit gate because enough
    using if(withdrawn(latest(checks)), 0 * latest(checks), latest(checks));
```

After `report 1`, `misread` and `decide`, `gate@1` is grounded on `checks@1`
and `recheck`, with the `withdrawn` caveat. Two things follow:

- A host that shows a decision's basis as a list of reports gets the reason
  in that list. Keep only the names of readings (`STREAM@N`) when the basis
  means reports, rather than leaving the reason out of the program.
- The withdrawn reading stays in the grounds only if `using` reads it.
  `if(withdrawn(latest(checks)), 0, latest(checks))` would be grounded on
  `recheck` alone; `0 * latest(checks)` keeps the reading there.

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

## Revise a program while the record continues

A program is the policy, and an event history is the record. When the policy
changes and the record goes on, keep both versions of the program. A replay
uses one program for every event, so `explain` under the revised program
reports that program's source hash for every decision, including those first
made under the earlier one. The events still give the same grounds when the
revision leaves those rules alone; a `same_as` scenario can confirm it.

Keep each program version beside the range of events it governed, for example
`policy/v1.cav` for events 1 to 13 and `policy/v2.cav` from event 14, and
record that boundary next to the event file. To show what the earlier program
decided, replay the earlier range under it:

```sh
head -n 13 events.jsonl > v1.events.jsonl
npx --no-install caveat-lang explain policy/v1.cav v1.events.jsonl
```

A save binds the exact source it was made under, and restore refuses any other
([save](../spec/caveat-save-0.1.md)), so a save cannot carry a session across a
revision. Replay the event history under the new program instead.
