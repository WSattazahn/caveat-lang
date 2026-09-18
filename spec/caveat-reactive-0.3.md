# CAVEAT Reactive Profile 0.3 — qualified computation

This profile extends [Reactive Profile 0.2](caveat-reactive-0.2.md) with numeric
values that carry their evidence and caveats through computation. It also records
the value and qualifications used by a commitment. The runtime remains a Rust
implementation; this increment does not introduce modules or self-hosting.

## Constructing a qualified value

```caveat
claim mild_current;
evidence forecast from "morning tidal archive";
evidence foam_reading from "light-observed foam drift";
caveat unmeasured_surge consequence high;
forecast supports mild_current;
unmeasured_surge qualifies forecast;
unmeasured_surge qualifies foam_reading;

fn correction(estimate, fraction) = -estimate * fraction;
state estimate = qualified(0, forecast);
state steering_plan = correction(estimate, 0.65);
event inspect speed min 0 max 4;

on inspect reveal foam_reading opposes mild_current;
on inspect set estimate = qualified(speed, foam_reading);
on inspect set steering_plan = correction(estimate, 0.65);
on inspect when not committed(counter_steer) commit counter_steer because enough using steering_plan;
```

The expression form is:

```text
qualified(NUMERIC_EXPRESSION, EVIDENCE[, ADDITIONAL_CAVEAT...])
```

The first argument is a numeric expression. Remaining arguments are declared
symbol identifiers, not strings or numeric expressions. The evidence must already
have a reached `supports` or `opposes` relation when the constructor executes.
Merely declaring it is insufficient. Construction preserves the numeric result
and any qualifications already carried by that expression, then adds the cited
evidence, its inherited caveats, and any explicit additional caveats.

Inherited caveats include incoming `qualifies` relations on the evidence and on
the claims it supports or opposes. Qualifications of those caveats and of explicit
additional caveats are followed transitively. Cycles are visited once. Examined caveats still qualify their
targets. Evidence references retain their canonical provenance in the graph's
evidence symbols; a constructor cannot replace that source string.

This operation records an authored basis for a number. It does not prove that the
evidence establishes that number, resolve contradictory claims, or assign a
confidence score. The author still defines how an observation becomes a numeric
estimate.

## Propagation through computation

Reactive numeric state is internally stored as a numeric value together with its
provenance. A numeric-only snapshot is a compatibility projection, not the
authoritative state used by subsequent execution.

Provenance contains two deduplicated, deterministically ordered sets:

- Evidence identifiers actually used as a basis.
- Caveat identifiers retained by those dependencies.

Unary operations preserve provenance. Arithmetic, comparisons, and numeric
intrinsics union all evaluated operands' provenance. Cancellation does not erase
it: `estimate * 0` and `estimate - estimate` still depend on `estimate`. `min`,
`max`, and `clamp` retain every evaluated argument's basis, including the bounds.

Pure numeric functions accept and return qualified numbers without special
parameter annotations. All eagerly evaluated arguments contribute, including
arguments the body ignores. Nested calls preserve the same rule. Function bodies
remain unable to capture global state or evidence; a qualified argument supplies
the dependency explicitly.

Boolean expressions carry provenance too. `and` and `or` retain the provenance of
operands actually evaluated. A short-circuited operand is neither evaluated nor
claimed as an observation. Existing arithmetic errors, finite-number checks,
function expansion limits, and numeric bounds still apply.

Each value is limited to 1,024 total evidence and caveat identifiers and 65,536
UTF-8 bytes across their names. Exceeding a bound is an error; provenance is never
silently truncated.

## Decisions and control flow

A successful state assignment combines its expression's provenance with the
provenance of its rule condition. A constant selected by a qualified condition
therefore remains qualified.

When a `set` rule's condition is false, its evaluated condition provenance is
added to the existing target's provenance: the decision to leave that value in
place depended on the condition. Its numeric value remains unchanged. A later
independent, unconditional assignment can replace a value and its provenance;
this cannot rewrite previously recorded commitment bases or graph history.

Bindings union the evaluated candidate conditions for each target/property with
the selected value's provenance. Thus a default followed by a conditional
override cannot hide the qualification governing which value appears. Emitted
cues carry the successful emitting rule's condition provenance.

Graph predicates preserve their limited meanings:

- A true `observed(evidence)` carries that observed basis and its qualifications.
  A false result does not manufacture an observation of that evidence.
- `examined(caveat)` carries the caveat; examination does not discharge it.
- Predicates about an existing commitment carry its stored basis and retained
  caveats. Reopening dependencies include the evidence that reopened it.

Guard dependencies also cross graph effects. A newly reached evidence relation
records the guard that allowed its revelation. A later `qualified` constructor or
true `observed` query inherits that dependency. Examination and reopening retain
their successful guards as well. Otherwise an author could lose a qualified
condition by turning its result into a graph fact and reading that fact back.

A false guard on a graph effect records a query dependency without creating an
observation or commitment. For example, if a qualified condition prevents a
revelation, a subsequent `not observed(evidence)` retains that condition's basis
while still excluding the unobserved evidence itself. The same rule applies to
skipped examination, commitment, and reopening. A first independent successful
transition clears the earlier absence dependency for that query; previously
committed decisions remain frozen. Repeating an already reached reveal or reopen
edge does not invent a new observation occurrence or replace its original basis.

This profile tracks dependencies of reached execution. It does not create
immutable identities for every repeated sensor reading or infer evidence from
arbitrary external event parameters.

## Committing with a qualified basis

```caveat
on inspect commit counter_steer because enough using steering_plan;
```

The general form is:

```text
commit ACTION because REASON [using NUMERIC_EXPRESSION] [retaining CAVEAT, ...]
```

Existing reasons and legacy forms remain valid. `using` evaluates a numeric
basis. Its caveats, the successful condition's caveats, and explicitly retained
caveats are unioned and attached with real `retains` graph edges. The source need
not repeat a derived value's inherited caveats.

The commitment records the evaluated number and its provenance at the moment of
commitment. Subsequent assignments cannot revise that historical basis. Its
`relies_on` edges point from the commitment to the observed evidence it used.
These edges describe reliance; they do not assert that the evidence is true or
add a `supports` relation. Qualification impact traversal can follow this
dependency from evidence to the decision that relied on it.

Reopening preserves the earlier number, evidence basis, and retained caveats.
Further action can use a new commitment. Retaining caveats does not itself block
an action, and creating a qualified value does not spend examination budget.
Explicit `examine` effects retain their existing costs.

Legacy `retaining A` remains an explicit retained edge to `A`; its qualifications
are still present in the graph. A qualified constructor's inherited closure,
including qualifications of explicit constructor caveats, is carried into a
commitment automatically when the derived value is used.

## Atomic publication and snapshots

Qualification construction, propagation, commitment bases, reliance edges,
binding metadata, and cue metadata participate in the existing event
transaction. An unobserved evidence reference, invalid calculation, exceeded
bound, or later binding failure rolls back the complete event.

Existing numeric `values`, primitive `bindings`, and `cues` remain compatible with
the browser host. The additive fields are:

| Field | Contents |
| --- | --- |
| `qualified_values` | Every state as `{value, provenance: {evidence, caveats}}` |
| `commitment_bases` | Commitment name to frozen `{value, provenance}`; value is `null` without `using` |
| `binding_qualifications` | Target/property to `{evidence, caveats}` |
| `cue_qualifications` | Provenance entries in the same order as `cues`, including repeated cue IDs |
| `observation_qualifications` | Guard dependencies of newly reached evidence relations, keyed by evidence |
| `examination_qualifications` | Dependencies of successful examinations, keyed by caveat |
| `reopening_qualifications` | Dependencies of successful reopening, keyed by commitment |
| `predicate_qualifications` | Dependencies of skipped graph effects, grouped by query kind and target |

Evidence identifiers resolve to the existing `symbols[].source` provenance.
`relations` includes `relies_on` edges. The additive snapshot schema remains
`caveat-reactive/0.1`; the profile version documents language capabilities rather
than breaking existing rendering consumers.

## The game application

The crosscurrent is an external field defined by strength, position, and time.
Light observation supplies a qualified foam measurement. Source functions infer
a steering plan from that measurement, and the revised commitment automatically
retains its basis and caveats. Later motion uses the retained plan rather than
re-reading the world's current strength as if the navigator knew it perfectly.

A changed navigation plan can move the ferry to a different position in the
physical field. A resulting force calculation can therefore inherit the
position's computational provenance. That does not mean observation changed the
external field: comparisons of physical force must hold position and time fixed.
Keeping the physical model independent of knowledge is a source-design invariant
verified by the game's tests, not a general prohibition enforced by the compiler.
