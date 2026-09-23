# CAVEAT Elapsed 0.1

Reactive expressions can read the session clock directly:

```caveat
event advance dt min 0 max 3600;
clock advance every 1;
bind hud.elapsed = elapsed();

fn minutes(seconds) = seconds / 60;
bind hud.minutes = minutes(elapsed());
```

`elapsed()` is a read-only, zero-argument numeric intrinsic. It returns the
existing session `elapsed` value in seconds, the same clock used by
[scheduled qualifications](caveat-renewal-0.1.md) and the
[decision journal](caveat-decision-journal-0.1.md). Authors do not need a second
state accumulator to display or compare session time.

## Clock contract

- A fresh session starts at zero. State initializers read zero.
- With `clock EVENT every STEP`, only accepted dispatches of that event add
  their supplied `dt`. An explicit clock takes precedence over `tick`; a
  separately declared `tick` does not also advance time.
- Without an explicit clock, accepted `tick` events add their `dt`. With
  neither clock nor `tick`, the value stays zero. Other events read the current
  value without advancing it.
- A clock event adds `dt` first, then applies due scheduled qualifications in
  their existing order, then runs its rules. Rules, procedure steps and final
  bindings therefore read the updated time.
- The existing input contract remains: `tick` bounds lie within `0..0.1`, while
  a custom clock uses its event's declared bounds. If those bounds admit
  negative `dt`, time may decrease. The positive `STEP` is host metadata, not
  the amount implicitly added by a dispatch.
- Any failure rejects the whole event, including its clock update, due
  qualifications, rules and bindings. A rejected event leaves `elapsed()` at
  its preceding value.

No wall clock is read, and no event is dispatched automatically. The result
is exactly the existing finite binary64 accumulator, including its ordinary
floating-point rounding. Reading it imposes no state range or new duration
cap; an accumulation that becomes nonfinite rejects the event. Assigning it
to a state still requires the result to fit that state's declared bounds and
the existing `-1e12..1e12` hard limit.

## Expressions and qualification

`elapsed()` can appear in reactive initializers, rule guards and values,
bindings, `define` expressions, and procedure guards, arguments and steps.
It can be passed explicitly to a pure function, as in `minutes(elapsed())`.
A pure function cannot capture the clock: `fn now() = elapsed();` is rejected,
including in an unused function. The intrinsic takes no arguments. An
unqualified global `fn elapsed` declaration is rejected. Modules retain their
existing function-shadowing rule: an explicit module-local `fn elapsed` owns
calls within that module. A state or parameter named `elapsed` may coexist
with the intrinsic; the identifier `elapsed` and the call `elapsed()` are distinct.

The read contributes no evidence, caveats, lineage or grounds of its own.
Ordinary propagation from enclosing expressions, qualified inputs, guards,
procedure calls and explicit citations remains unchanged. Reading time does
not assert that evidence is fresh or automatically reconsider a decision;
the source authors those policies.

## Evaluation and persistence

A binding that reads `elapsed()`, including through a `define` or a pure
function argument, depends on the runtime clock. Incremental evaluation
reevaluates that binding when the clock changes even if no state or graph
record changed. This dependency is separate from a state named `elapsed`.

The [save format](caveat-save-0.1.md) is unchanged: saves already hold the
session clock. Restore loads that value before recomputing bindings, so
`elapsed()` agrees with the restored snapshot and subsequent event behavior.
The intrinsic adds no mutable state or snapshot fields.
