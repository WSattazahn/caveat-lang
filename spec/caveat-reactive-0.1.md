# CAVEAT Reactive Profile 0.1

The reactive profile runs a continuous game or other event-driven simulation
from CAVEAT source. Numeric rules share the existing epistemic graph: observed
evidence, examined caveats, commitments, and reopening can change motion and
other calculations. A host supplies bounded inputs and displays snapshots.

`ReactiveSession` is the native entry point; `WebReactiveSession` exposes it to
WebAssembly. Existing sequential `GameSession` programs keep their behavior.
The legacy evaluator alone does not run reactive events.

## State and inputs

```caveat
state speed = 2;
state boat_x = ferry.x min -9.5 max 13;
state hull = 3 min 0 max 3;

event start;
event aim x min -100 max 100, z min -100 max 100, active min 0 max 1;
event tick dt min 0 max 0.1;
```

State initializers are numeric expressions. They can reference previously
declared states and read-only source positions. Explicit state bounds are
optional; the default is `-1e12..1e12`. This is also the hard inclusive limit:
explicit state bounds and numeric event parameter bounds must have finite
endpoints satisfying `-1e12 <= min <= max <= 1e12`. An explicit declaration
cannot widen that limit.

Values are finite double-precision numbers. State initializers must fit their
declared range. An assignment outside that range rejects the entire event
atomically, including any earlier effects; values are not silently clamped.
Use the `clamp` function when saturation is intended.

Each event declares all its numeric parameters and their required inclusive
bounds. Empty events accept `{}`. Event payloads must contain exactly those
parameters: unknown, missing, duplicate, nonnumeric, nonfinite, and out-of-range
values are errors. Parameter names cannot shadow state. Names and declarations
are checked before execution.

`tick` has a reserved input contract: its only parameter is `dt`, and its bounds
must lie within `0..0.1` seconds. The host determines how often to dispatch it;
the source determines what time changes. The engine does not read a wall clock
or generate random numbers.

## Source coordinates

```caveat
place sea kind sea;
entity reef kind reef at sea;
position sea at 0 0 0;
position reef at -1 0 45;

state reef_distance = 100;
on tick set reef_distance = sqrt((boat_x - reef.x) * (boat_x - reef.x)
                               + (boat_z - reef.z) * (boat_z - reef.z));
```

Every checked `position ID at X Y Z` declaration creates read-only numeric
identifiers `ID.x`, `ID.y`, and `ID.z`. Physics and the renderer therefore use
the same coordinates from `MapWorld.presentation`. Reading an unknown position
or assigning to a coordinate is rejected. This profile uses numeric event rules
in place of sequential action plans. Action plans and sequential choice
statements cannot be mixed into a reactive program; topology and presentation
declarations remain validated.

## Rules and expressions

```caveat
on aim set target_x = x;
on tick when phase == 1 set boat_x = boat_x + velocity * dt;
on tick when hull <= 0 set phase = 3;
```

A rule has one effect and optionally a boolean condition:

```text
on EVENT [when BOOLEAN_EXPRESSION] EFFECT;
```

All rules for the dispatched event run in source order against an evolving
transactional copy. A later condition can see an earlier state assignment,
observation, examination, commitment, or reopening from the same event.
Rules are evaluated once; there are no implicit loops or recursive dispatches.

Numeric expressions support literals, state, event parameters, source
coordinates, parentheses, unary signs, `+`, `-`, `*`, `/`, and these functions:

| Function | Meaning |
| --- | --- |
| `abs(x)` | Absolute value |
| `min(a, b)`, `max(a, b)` | Smaller or larger value |
| `clamp(x, low, high)` | Inclusive saturation |
| `sqrt(x)` | Square root of a nonnegative value |
| `sin(x)`, `cos(x)` | Trigonometric functions, radians |
| `elapsed()` | Current session clock in seconds; see [Elapsed 0.1](caveat-elapsed-0.1.md) |

`elapsed()` reads the existing runtime clock, initially zero, without adding
evidence or grounds. Clock events accumulate their `dt` before due scheduled
qualifications and rules, and a rejected event rolls that update back. Reads
are not subject to state bounds; assigning the result to a state still is.
The later [Elapsed 0.1](caveat-elapsed-0.1.md) profile specifies clock selection,
pure-function boundaries, incremental bindings and save/restore behavior.

Boolean expressions support `true`, `false`, `==`, `!=`, `<`, `<=`, `>`, `>=`,
`and`, `or`, `not`, and the graph predicates below. Conditions must be boolean;
assignments must be numeric. `and` and `or` short-circuit. Standard arithmetic
precedence applies; use parentheses to make combined conditions explicit.

Division by zero, nonfinite results, negative square roots, and inverted clamp
bounds are runtime errors. Expression parsing limits token count and nesting;
the interpreter executes parsed expressions and never evaluates host code.

## Reasoning changes the simulation

```caveat
budget 1;
claim clear_course;
evidence chart from "old chart";
evidence reef_reading from "lookout";
caveat chart_omission consequence high;
chart supports clear_course;
chart_omission qualifies chart;

on start when not committed(course)
  commit course because enough retaining chart_omission;
on tick when light_near_reef == 1 and not observed(reef_reading)
  reveal reef_reading opposes clear_course;
on tick when observed(reef_reading) and not examined(chart_omission)
  examine chart_omission cost 1;
on tick when observed(reef_reading) and committed(course) and not reopened(course)
  reopen course because reef_reading;
on tick when reopened(course) set speed = cautious_speed;
```

The predicates read the actual reached graph:

| Predicate | True when |
| --- | --- |
| `observed(EVIDENCE)` | A reached `supports` or `opposes` edge originates at that evidence |
| `examined(CAVEAT)` | Live attention is `Examined` |
| `committed(ACTION)` | That named commitment exists, including if reopened |
| `reopened(ACTION)` | That commitment exists and is open |

Declaring evidence does not observe it. Opposing evidence counts as observation;
the engine retains the earlier supporting edge and its caveats. These predicates
do not collapse uncertainty into a Boolean claim of truth.

The graph effects are:

```text
reveal EVIDENCE supports CLAIM
reveal EVIDENCE opposes CLAIM
examine CAVEAT cost UNSIGNED_INTEGER
commit ACTION because enough|budget|deadline [retaining CAVEAT, CAVEAT]
reopen ACTION because EVIDENCE
```

Graph references are checked for the appropriate types. Repeated identical
revelations and reopening edges are idempotent. Examination consumes the existing
attention ledger; authors can guard it with `not examined(...)` to pay once.
Insufficient attention rejects the event. Committing an already existing action
is an error; `not committed(...)` supports a one-time commitment.

A reactive reopening requires an existing commitment and observed evidence.
Reopening preserves retained caveats and records the evidence responsible.
Different newly observed evidence can add further reopening edges.

## Atomicity and determinism

The engine validates the complete payload before running a rule. It publishes
numeric updates, attention spending, graph changes, effect reports, and the event
sequence together. Any later rule failure discards all earlier effects from the
same event. Direct host calls have the same enforcement as UI input.

Replaying the same valid input sequence against the same source on the same
engine produces the same snapshots. Source order resolves otherwise competing
assignments. The host does not supply scores, collisions, or outcome decisions
unless the source explicitly declares such inputs.

Compiled rule expressions are immutable and shared between transactional
copies; a tick does not reparse source. Numeric state and graph state are copied
for commit/rollback. There is no built-in random source, event recursion,
automatic save system, or general-purpose file/network I/O in this profile.

## Host API and snapshots

```rust
let mut session = ReactiveSession::from_source(source)?;
let initial = session.snapshot();
let next = session.dispatch_json("tick", r#"{"dt":0.033}"#)?;
```

The native `dispatch` alternative accepts a `BTreeMap<String, f64>`.
`WebReactiveSession` has a source constructor, `snapshot() -> JSON`, and
`dispatch(event, payload_json) -> JSON` or an error. Constructing a fresh session
resets the program to its source-authored initial state.

`caveat-reactive/0.1` snapshots contain:

- `values`: numeric state by source identifier.
- `events`: declared event names and parameter bounds, for input adapters.
- `world`: the checked `MapWorld`, including source presentation.
- `symbols`, `relations`, `commitments`, `budget`: current epistemic state.
- `effects`: graph effects newly produced by the last accepted event.
- `labels`, `scenes`: authored presentation text.
- `sequence`, `last_event`, `source_id`: event count and source identity.

Relations preserve insertion order; names and numeric state serialize in stable
order. A failed event leaves the preceding snapshot unchanged. Event metadata
lets hosts constrain pointers or sliders without duplicating source limits.

## Executable example

[`game/light_the_way.cav`](../game/light_the_way.cav) owns ferry velocity,
steering, wind, reef distances, collision cooldown, hull damage, progress,
passenger rescue, and win/loss conditions. It also owns sightings that oppose
the old chart, paid caveat examination, retained uncertainty, and reopening that
changes approach speed. JavaScript renders those results and supplies pointer
and bounded time inputs.
