# CAVEAT Reactive Profile 0.7 — reusable procedures

This profile extends [Reactive 0.6](caveat-reactive-0.6.md) with named,
bounded sequences of effects. A program can reuse input handling, observation,
and revision behavior without implementing an application-specific operation
in Rust or JavaScript. Pure functions compute values; procedures perform
effects within their calling event.

```caveat
state position = 0;
state paused = 0;
event move distance min -10 max 10;

proc advance(amount) {
    set position = position + amount;
    when position > 20 set paused = 1;
};

on move when paused == 0 call advance(distance);
```

## Declarations and calls

`proc NAME(PARAMETERS) { STEPS };` declares a procedure. Each step contains an
existing reactive effect, optionally preceded by `when BOOL`. Effects include
`set`, `sample`, `reveal`, `examine`, `commit`, `reopen`, `emit`, and
`call NAME(ARGUMENTS)`. Semicolons separate steps; the final body semicolon is
optional, consistent with the source scanner's existing final-statement rule.
Calls can appear in ordinary event rules or inside other procedures.
Definitions can refer to later procedure definitions.

Parameters and arguments are numeric. Argument count must match exactly.
There are no return values, local declarations, nested procedure declarations,
dynamic procedure names, loops, or recursive calls. Use pure `fn` declarations
for reusable calculations. Procedure names cannot duplicate one another or
shadow a source or prelude function.

A body sees its own parameters and the program's declared state, coordinates,
and graph/history queries. It cannot implicitly capture an event parameter or
another procedure's parameter. Passing those values requires an explicit
argument. Parameter names must be unique and cannot shadow global state or
coordinates. All definitions are checked, including unused definitions and
calls under false guards.

## Evaluation order and qualification

1. Evaluate the calling rule or body step's guard once against the current
   transaction state.
2. If true, evaluate all arguments once, left to right, before any body effect.
   Bind the resulting qualified numeric values to the callee's parameters.
3. Combine the guard's qualifications, inherited qualifications from enclosing
   calls, and every argument's qualifications. Carry that basis through all
   descendant effects, even when a parameter is unused.
4. Execute body steps in source order. Each body guard is evaluated immediately
   before its effect and can see state or graph changes from earlier steps.
   Parameters retain their entry values and provenance throughout the call.

An outer guard stays accepted throughout a call even if its body changes the
state that made the guard true. Refactoring separate guarded event rules into
a procedure must account for this difference: separate rules reevaluate their
guards independently. Conditions that need to respond to earlier body writes
belong on the corresponding body steps.

When a calling guard is false, no arguments or body conditions are evaluated.
The interpreter traverses possible descendant effect targets and retains the
evaluated guard's dependencies using the existing skipped-effect rules. A
skipped nested call therefore cannot erase the reason a value, observation,
examination, or history selection was left unchanged. This traversal creates
no observations, spends no attention, and emits no cues. It still consumes
bounded interpreter work.

For a false guard inside an accepted call, the inherited call basis includes
the already-evaluated arguments. Skipping that step retains this basis along
with the step's evaluated guard dependencies. Untaken argument expressions
remain unevaluated; unused arguments to an accepted call remain eager.

## One event transaction

A call does not dispatch another event. All calls share the calling event's
transaction, event identity, sequence number, effect reports, attention budget,
and work budget. Bindings are evaluated after all event rules finish, not
between body steps. Cues become visible only on successful publication.

Failure in an argument, body guard, effect, nested call, capacity check, work
limit, or final binding calculation rolls back the whole event. Numeric values,
graph relations, retained caveats, observations, commitment revisions, cues,
and occurrence identifiers all return to the preceding snapshot. Calls neither
launder provenance nor rewrite frozen earlier readings and commitment bases.

## Bounds and compatibility

| Resource | Limit |
| --- | --- |
| Procedure definitions | 128 |
| Numeric parameters per procedure | 32 |
| Declared body steps across all procedures | 4,096 |
| Nested procedure calls | 64 |
| Expanded body work per procedure | 4,096 steps |
| Work across one dispatched event | 4,096 steps |

Load-time validation rejects cycles and excessive expanded body work, even in
unused procedures. Event work counts matching top-level rules, call steps,
leaf effects, and skipped descendant traversal together. Repeated calls each
consume work; a work-limit failure is atomic. Existing expression and history
limits also apply.

Braces group procedure bodies; quoted braces, semicolons, and comment markers
remain literal text. Comments and Unicode source positions retain the
[source-text contract](caveat-text-0.1.md). Body syntax errors identify their
source line and column. Existing programs and browser bridges retain their
snapshot schema, `caveat-reactive/0.1`.

## Source consumers

[Light the Way](../game/light_the_way.cav) shares keyboard release, held-input
activation, and aim movement between source events. Its existing observation
cost, physical current, prior measurements, and navigation revisions remain
governed by the same Caveat rules.

The [thermostat](../examples/thermostat_history.cav) handles each reading through
one procedure that records evidence, summarizes the archive, explicitly reopens
the selected heating decision, and commits a new qualified basis. Rust supplies
the generic call mechanism, validation, qualification tracking, and atomicity;
neither application requires its own interpreter operation. This adds effect
reuse, not a self-hosted compiler.
