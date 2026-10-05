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

## Integer clocks

Status: specified for rc.15 (owner decision of 2026-10-04 on
[integer tick time](../docs/design/integer-tick-time.md)). The runtime
implements it from rc.15's PR 3; until then a program declaring `integer` is
rejected when it loads.

A program can declare its clock **integer**, so that time is counted in whole
units and every schedule applies on the exact event:

```caveat
event step dt min 0 max 100;
clock step every 16 integer;         -- dt and elapsed() count whole units
on taste qualify taste_$m with taste_faded after 60000;
fn seconds(t) = t / 1000;
bind hud.time = seconds(elapsed());
```

The unit is the program's own: a millisecond, a frame, a turn. The runtime
does not know it, as it does not know seconds for a binary64 clock.
**A program without `integer` is unchanged**: its clock, timing, saves and
scenario outcomes are exactly as specified above.

- **Declaration.** `clock EVENT every STEP integer` requires a whole `STEP`
  and whole `min` and `max` bounds on the clock event's `dt`. The reserved
  `tick` event cannot be an integer clock, because its bounds lie within
  `0..0.1`; a program names its own clock event. A program that declares an
  integer clock otherwise is rejected when it loads.
- **Input.** A clock event's `dt` must be a whole number. A fraction inside the
  declared bounds is refused as `input/payload_invalid`, as a fraction for a
  typed parameter is; a value outside the bounds is still
  `input/bound_exceeded` ([Dispatch 0.1](caveat-dispatch-0.1.md)). The
  bounds are checked first: a whole number outside them and a fraction
  outside them are both `input/bound_exceeded`, and only a fraction inside
  them is `input/payload_invalid`.
- **Reading.** `elapsed()` is a whole number of units within
  `±(2^53 − 1)` (`±9007199254740991`), the integers binary64 and a JSON host
  hold exactly, so every addition is exact. A clock event whose `dt` would take
  the reading outside that range is refused as `evaluation/bound_exceeded`,
  with everything the event did rolled back. Unlike the binary64 clock's
  nonfinite refusal, this one is reachable: steps of `1e12` reach it in about
  9,000 events.
- **Delays.** A `qualify … after` delay must be a whole number in
  `0..2^53 − 1`. A delay written as a number literal is checked when the
  program loads. A computed delay is checked when the rule runs: one that is
  not a whole number in that range refuses the event as
  `evaluation/expression` ([Dispatch 0.1](caveat-dispatch-0.1.md)). A
  schedule is due on the first clock event at which
  `elapsed() - scheduled_at >= after`, as above, and the comparison is exact:
  both readings are whole and within range, and a difference too large to
  hold exactly is at least `2^53`, which no delay reaches, or at most `-2^53`,
  which is below every delay. The rounding paragraphs of
  [Renewal 0.1](caveat-renewal-0.1.md) do not apply.
- **Records.** The snapshot's and journal's `elapsed` and each pending
  scheduled qualification's `scheduled_at` and `after` hold whole units. The
  snapshot's `clock` metadata adds `"integer": true` for an integer clock and
  is unchanged otherwise. The view and save formats keep their shape.
- **Restore.** For an integer clock, restore refuses a saved `elapsed`,
  journal `elapsed`, `scheduled_at` or `after` that is not a whole number in
  its range ([Save 0.1](caveat-save-0.1.md)). As with every restore check, a
  value that passes is one the program could hold, not one events are shown
  to have produced.

Opting in edits the source, so it changes the program's `source_id`, and a
save of the program without `integer` does not restore into it, as for any
source edit. The [authoring guide](../docs/AI_AUTHORING.md#read-the-session-clock)
describes how a host converts its frame time to whole units.

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

- 2026-10-05 (rc.15, specified): integer clocks. `clock EVENT every STEP
  integer` counts time in whole units of the program's choosing, refuses a
  fractional `dt` as `input/payload_invalid` and a reading outside
  `±(2^53 − 1)` as `evaluation/bound_exceeded`, and makes every schedule apply
  on the exact event (Version Lab finding F267, Glowcap round 7). Programs
  without `integer` are unchanged. The runtime implements it from rc.15's PR 3.

- 2026-10-04: the clock contract states when a scheduled qualification is due,
  in binary64 (Version Lab finding F267). The runtime is unchanged.

- 2026-10-03: the accumulator section says the clock stays finite and loses
  precision near the binary64 limit. It used to say an accumulation that
  becomes nonfinite rejects the event, which implied an overflow that
  declared bounds cannot reach, while the runtime added `dt` with no check.
  The runtime now checks the sum and refuses a nonfinite one as
  `evaluation/bound_exceeded`; no supported path reaches it. Version Lab
  finding F151 measured the saturation on every published version.
