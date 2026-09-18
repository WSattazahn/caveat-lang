# CAVEAT Reactive Profile 0.2

This profile adds pure numeric functions, presentation bindings, emitted cues,
and host input/clock metadata to the event runtime described in
[Reactive Profile 0.1](caveat-reactive-0.1.md). The 0.1 document remains the
historical baseline. The snapshot schema remains `caveat-reactive/0.1`: these
changes add fields to that schema rather than replacing its existing fields.

CAVEAT source owns calculations, graph effects, and presentation decisions.
The host supplies declared input, advances time, and renders accepted snapshots.
`ReactiveSession` and `WebReactiveSession` remain the execution entry points;
the legacy sequential evaluator does not dispatch reactive events.

## Pure numeric functions

```caveat
fn square(x) = x * x;
fn distance(ax, az, bx, bz) = sqrt(square(ax - bx) + square(az - bz));
fn pulse(time, rate) = 0.5 + 0.5 * sin(time * rate);
```

Function declarations are collected globally, so definitions can call functions
declared later. Names and parameters are plain ASCII identifiers. Duplicate
definitions or parameters, intrinsic/predicate name shadowing, wrong arity,
unknown calls, and direct or indirect recursion are rejected, including in
unused functions. A function can have zero parameters.

Every parameter and result is numeric. Bodies may use parameters, literal
constants, arithmetic, numeric intrinsics, and other pure functions. They
cannot capture state, named world coordinates, or graph predicates. Pass those
numeric inputs explicitly, for example
`distance(boat_x, boat_z, reef.x, reef.z)`. Arguments must remain numeric and
finite even when a function does not use that parameter; `constant(1 / 0)`
still fails. Function calls are expanded during session construction, before
ordinary expression validation and execution.

The new intrinsic is `atan2(y, x)`, returning an angle in radians with the
argument order shown. Existing intrinsics remain `abs`, `min`, `max`, `clamp`,
`sqrt`, `sin`, and `cos`. Division by zero, nonfinite arithmetic, negative
square roots, and reversed clamp bounds remain errors.

The limits are 128 function declarations, 4096 expanded expression nodes, and
64 levels of expression/function expansion depth. Individual expression
parsing is also bounded by 1024 tokens and 65,536 source bytes. There are no
collections, modules, or general recursion in this addition.

## Presentation bindings

```caveat
bind ferry.x = boat_x;
bind ferry.rotation_y = atan2(boat_vx, -boat_speed);
bind ferry.body.rotation_z = boat_vx * -0.025;
bind hud.speed = sqrt(boat_vx * boat_vx + boat_speed * boat_speed);
bind hud.caution = examined(chart_omission);
bind hud.message = "Follow the light";
bind hud.message = "Course reopened" when reopened(course);
```

The form is:

```text
bind TARGET.PROPERTY = EXPRESSION_OR_QUOTED_TEXT [when BOOLEAN_EXPRESSION];
```

A target is a plain identifier. A property can contain dot-separated identifier
segments. Targets and properties name presentation endpoints interpreted by the
host; they do not create entities or graph nodes. Values have one of three
types: number, boolean, or quoted text. Every declaration for a given
target/property must have the same type, even when its conditions are mutually
exclusive. Conditions must be boolean.

Bindings can read numeric state, source coordinates, pure functions, and
reached graph predicates. They cannot read transient event parameters or other
bindings. They have no state or graph effects. Bindings are evaluated at session
construction and after all rules of each accepted event, against that resulting
state. Matching declarations run in source order; the last matching value for
a target/property wins. If none match, the property is omitted from that
snapshot, rather than retaining an old value.

The resulting primitive values are cached. Calling `snapshot()` copies those
values without reevaluating expressions. The JSON shape is a target map whose
property keys remain literal strings, including any dots:

```json
{"bindings":{"ferry":{"x":4,"rotation_y":0.2,"body.rotation_z":0.1},"hud":{"caution":true}}}
```

An active binding's evaluation failure rejects construction or the entire
event. A false condition skips that binding's value expression; all bindings
are still statically type checked.

## Cues and emission

```caveat
cue depart sound 392 0.14 0.06;
cue warning toast "Check the chart caveat" 1.2;
cue impact_screen flash impact 0.3;
cue impact_ring ring ferry "#ff7043" 0.4;
on start emit depart;
on tick when collision == 1 emit impact_ring;
```

Cue declarations have fixed source-authored values. `emit NAME` is an event
effect, usable with the same optional rule condition as other effects.
Duplicate cue identifiers and emission of undeclared cues are rejected.

| Declaration | Fields and bounds |
| --- | --- |
| `cue ID sound FREQUENCY DURATION GAIN` | Frequency 20..20000 Hz; gain 0..1 |
| `cue ID toast "TEXT" DURATION` | Quoted text |
| `cue ID flash TARGET DURATION` | Host presentation target identifier |
| `cue ID ring ENTITY COLOR DURATION` | Target must be a declared world entity |

Every duration is finite, strictly positive, and at most 30 seconds. A ring
color is quoted `"#RGB"` or `"#RRGGBB"`, or a decimal integer from 0 through
16777215 (`0xffffff`). Numeric hexadecimal literal syntax is not supported.
Flash targets are host identifiers and need not be world entities.

Each accepted event publishes its emitted cues in source order. Multiple
emissions, including repeated IDs, are retained. The initial cue list is empty;
the next accepted event replaces it with that event's emissions. A host should
consume cues once per new accepted `sequence`, rather than replaying them on
each snapshot read. The interpreter itself does not play audio or draw effects.

## Controls and clock metadata

```caveat
event aim x min -100 max 100, z min -100 max 100, active min 0 max 1;
event start;
event tick dt min 0 max 0.1;
control pointer = aim;
control start = start reset;
control retry = start reset;
clock tick every 0.03333333333333333;
```

`control NAME = EVENT [reset]` publishes `{event, reset}` under that control
name. Names must be unique, and events must be declared. The host maps its
pointer, keyboard, or UI action to the named control and supplies precisely the
event's declared numeric payload. This metadata does not bypass event bounds.

For a control with `reset: true`, the host constructs a fresh session from the
same source **before** dispatching the control's event. Reset is a host protocol;
dispatching the event directly does not reset the existing session.

`clock EVENT every STEP` publishes `{event, step}`. At most one clock is
allowed. Its event must declare exactly one parameter named `dt`; the finite,
positive step must fit that parameter's bounds. The reserved `tick` event
continues to require bounds within 0..0.1 seconds. A host advances the simulation
using the declared fixed step and a bounded number of catch-up dispatches; it
must not inject an accumulated, out-of-range frame interval. The runtime does
not read wall time or dispatch the clock automatically. A clock targeting an
event other than `tick` follows that event's declared `dt` bounds.

## Knowledge, attention, and physical behavior

Arithmetic results, pure functions, bindings, and cues do not assert that a
claim is true. A boolean comparison such as `distance(...) < 3` establishes a
numeric condition for a rule; it does not establish an epistemic truth value.
An author can use that condition to reveal evidence through the explicit
`reveal EVIDENCE supports|opposes CLAIM` effect.

`observed(EVIDENCE)` reads a reached supporting or opposing edge from that
evidence. Declaring evidence alone does not observe it. `examined(CAVEAT)` reads
the caveat's current attention state. Examination spends the declared attention
budget and marks the caveat examined; it does **not** discharge the caveat,
remove its `qualifies` relation, delete retained uncertainty, or erase opposing
evidence. Insufficient budget rejects the event. Reopening a commitment retains
its caveats and records the observed evidence that caused reopening.

Physical forces and a controller's response are separate source rules. For
example, a current can continue to add drift whether or not its evidence has
been observed. Observation, examination, or reopening can then change a
source-authored steering correction or approach speed. Knowledge changes that
controlled response; observation does not itself remove the physical current.
This distinction is expressed by the game source, not enforced by a built-in
physics engine.

## Transaction and snapshot contract

An event validates its payload, then applies matching rules in source order to
a transactional copy. Later rules can read earlier numeric or graph changes.
After those rules, the runtime recomputes bindings before publishing anything.
Any failure, including a binding's arithmetic failure, rolls back numeric
updates, graph changes, attention spending, emitted cues, effect reports,
bindings, and the event sequence together. The preceding snapshot remains
unchanged; a host must not present speculative cues from a rejected event.

The additive snapshot fields are:

| Field | Value |
| --- | --- |
| `bindings` | Cached primitive values, indexed by target then property |
| `cues` | Cues emitted by the last accepted event; each includes `kind` and `id` |
| `controls` | Control name to `{event, reset}` metadata |
| `clock` | `{event, step}` metadata, or `null` |

Existing `values`, `events`, `world`, graph fields, `effects`, labels, scenes,
`source_id`, `sequence`, and `last_event` keep their roles. Source is parsed and
validated during construction; accepted event processing uses the compiled
expressions. The game can therefore keep motion, collisions, graph decisions,
UI values, and cue selection in CAVEAT while its host handles input and display.
This profile does not make the implementation self-hosting or add general
file, network, collection, or module facilities.
