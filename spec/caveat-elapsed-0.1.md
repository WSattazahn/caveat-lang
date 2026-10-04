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
- A scheduled qualification is due on the first clock event at which
  `elapsed() - scheduled_at >= after` in binary64, where `scheduled_at` is the
  reading when it was scheduled. Rounding can make that an event later or
  earlier than exact arithmetic would ([Renewal 0.1](caveat-renewal-0.1.md)).
- The existing input contract remains: `tick` bounds lie within `0..0.1`, while
  a custom clock uses its event's declared bounds. If those bounds admit
  negative `dt`, time may decrease. The positive `STEP` is host metadata, not
  the amount implicitly added by a dispatch.
- Any failure rejects the whole event, including its clock update, due
  qualifications, rules and bindings. A rejected event leaves `elapsed()` at
  its preceding value.

No wall clock is read, and no event is dispatched automatically. The result
is exactly the existing binary64 accumulator, including its ordinary
floating-point rounding. Reading it imposes no state range or new duration
cap. The accumulator stays finite: each step is a `dt` within its event's
declared bounds, which lie within `-1e12..1e12`, and near the largest finite
binary64 value such a step is less than half the spacing between neighboring
values, so the sum rounds back to that value. The clock loses precision
there; it does not overflow. A clock event whose sum would nonetheless be
nonfinite is refused as `evaluation/bound_exceeded`, and restore refuses a
nonfinite saved clock, so neither dispatch nor restore reaches that refusal.
Assigning `elapsed()` to a state still requires the result to fit that
state's declared bounds and the existing `-1e12..1e12` hard limit.

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

## Changes

- 2026-10-04: the clock contract states when a scheduled qualification is due,
  in binary64 (Version Lab finding F267). The runtime is unchanged.

- 2026-10-03: the accumulator section says the clock stays finite and loses
  precision near the binary64 limit. It used to say an accumulation that
  becomes nonfinite rejects the event, which implied an overflow that
  declared bounds cannot reach, while the runtime added `dt` with no check.
  The runtime now checks the sum and refuses a nonfinite one as
  `evaluation/bound_exceeded`; no supported path reaches it. Version Lab
  finding F151 measured the saturation on every published version.
