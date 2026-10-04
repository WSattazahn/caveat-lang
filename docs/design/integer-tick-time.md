# Design note: integer tick time

Status: a design note for the owner's decision (rc.13 plan, PR 10). Nothing
here is implemented or promised, and no specification changes with it.
Receipts are `path: Lx–Ly` at `main` `8b1d2f3`.

## The problem

Glowcap round 7's third work item (`experiments/glowcap/RESULTS.md:
L924–927`): every Caveat program faded a taste one tick later than the
TypeScript programs at an exact 60-second boundary (Version Lab finding F267).
The session clock is a binary64 sum of each clock event's `dt`, and a
scheduled qualification stores the reading when it was scheduled and is due on
the first clock event at which the current reading minus that stored reading
reaches the delay (`spec/caveat-elapsed-0.1.md: L32–35`,
`spec/caveat-renewal-0.1.md: L85–105`). Scheduled at `2.800000000000001`, a
delay of 60 is not due after 960 steps of `0.0625`, which sum to exactly 60;
it applies on the 961st step. At large readings it can be several events off,
either way.

By the owner's card, rc.12 documented the rule and changed nothing
(`docs/releases/rc12-development.md: L437–449`). It declined counting time
per schedule: that changes which event existing schedules apply on, including in
saved sessions, and still rounds for steps that are not powers of two. It
named integer tick time as the design to consider if a program needs timing
exact to the event (`docs/releases/rc12-development.md: L450–455`). The
authoring guide advises power-of-two steps (`docs/AI_AUTHORING.md: L239–243`),
which a host does not always control: Glowcap's `tick` carries any `dt` in
`0..0.1`.

The TypeScript programs were not exact either. They counted `dt` per timer
from zero, so with `0.0625` steps their sum was exact and with `0.1` steps it
would not have been. What a program needing exact timing wants is that the
clock and every delay are whole numbers, so that every sum and difference is
exact.

## What an integer clock would mean

A program opts in on its clock declaration. Syntax is illustrative only:

```caveat
event step dt min 0 max 100;
clock step every 1 integer;          -- dt and elapsed() count whole units
on taste qualify taste_$m with taste_faded after 60000;
fn seconds(t) = t / 1000;
bind hud.time = seconds(elapsed());
```

The unit is the program's own (a millisecond, a frame, a turn); the runtime
does not know it, as it does not know seconds today. For such a clock:

- **Input.** A clock event's `dt` must be a whole number inside its declared
  bounds. A fraction is refused as `input/payload_invalid`, as a fraction for
  a typed parameter already is (`spec/caveat-dispatch-0.1.md: L110`). The
  reserved `tick` event cannot be an integer clock, because its bounds lie
  within `0..0.1` (`spec/caveat-reactive-0.2.md: L170–177`); a program names
  its own clock event.
- **Reading.** `elapsed()` is a whole number of units. It stays within
  `±(2^53 − 1)`, the integers binary64 and a JSON host hold exactly; a clock
  event that would leave that range is refused as `evaluation/bound_exceeded`,
  the code a nonfinite clock already uses (`spec/caveat-elapsed-0.1.md:
  L51–53`). Unlike today's refusal, this one is reachable: a `dt` up to `1e12`
  can get there in about 9,000 events.
- **Delays.** `after` must be a whole number of units, checked when the
  program loads when it is a constant. A schedule is due on the first clock
  event at which the reading minus `scheduled_at` is at least the delay, the
  same rule as now, and with whole numbers the subtraction is exact. The
  rounding paragraphs of Renewal 0.1 do not apply.
- **Records.** Journal `elapsed`, `scheduled_at` and the snapshot's `elapsed`
  hold whole units. The view and save formats are unchanged in shape.
- **Restore.** For an integer clock, a saved `elapsed` or `scheduled_at` that
  is not a whole number in range is refused. As with every restore check
  under A3, this says the value is one the program could hold, not that
  events produced it.

Changes to specifications if chosen: Elapsed 0.1 (clock contract and
accumulator section), Renewal 0.1 (scheduled qualification), Reactive 0.2
(`clock` declaration), Dispatch 0.1 (the two refusals above) and Save 0.1
(the restore check). Programs that do not opt in keep the binary64 clock,
and their timing, saves and scenarios are unchanged.

## How an existing program opts in

1. Choose the unit so that every delay and every `dt` the host sends is a
   whole number of it. Glowcap's `tick` in `0..0.1` seconds becomes a
   `step` event with `dt` in `0..100` milliseconds, and `after 60` becomes
   `after 60000`.
2. Convert in source wherever time is shown or compared: bindings and guards
   that read `elapsed()` or a `dt` divide by the unit.
3. The host sends whole units. If its frame time is not a whole number of
   units (16.67 ms), it rounds once, at its own boundary, and carries the
   remainder to the next frame. Rounding then happens in one visible place,
   not in the clock's subtraction.
4. Saves do not carry over. The edit changes the program's `source_id`, so a
   save of the old program does not restore into the new one, as for any
   source edit (`spec/caveat-routed-repetition-0.1.md: L282–289`). A host
   that must keep a session replays its recorded events with `dt` converted,
   which works only when every recorded `dt` was a whole number of the new
   unit.

## Options

- **A. Opt-in integer clock.** As above.
- **B. A counting clock.** The clock counts accepted clock events and ignores
  `dt` for time: `elapsed()` is the number of clock events, and `after 960`
  means 960 of them. Simpler than A, with no unit or input rule, and exact by
  construction. It suits fixed-step hosts only: Glowcap's variable `dt` could
  not use it, and a host catching up after a stall must dispatch every missed
  step.
- **C. No change.** Keep the binary64 clock and the documented rule. A program
  needing exact timing counts whole ticks in a state of its own and qualifies
  immediately when a guard on that counter holds. That gives up what makes
  `qualify … after` useful in Glowcap, a schedule bound to the occurrence that
  was current when it was scheduled (`spec/caveat-renewal-0.1.md: L69–73`),
  and needs a counter per pending timer.

Counting time per schedule on the binary64 clock was considered and declined
in rc.12 for the reasons above; it is not offered again here.

## Decision

**Question:** should a Caveat program be able to declare a clock whose time is
counted in whole units, so that schedules fire on the exact event?

- **A. Opt-in integer clock.** A clock declared integer takes whole-number
  `dt` in a unit the program chooses; delays and readings are exact up to
  2^53 units. Existing programs are unchanged; opting in changes the source,
  so old saves do not restore into it.
- **B. Counting clock.** An opt-in clock that counts its events instead of
  summing `dt`. Exact and simpler, but only for hosts that dispatch fixed
  steps.
- **C. No change.** Keep binary64 time and its documented rule; exact timing
  stays the author's job, with a counter in state.

**Recommendation: A.** It removes F267's cause rather than documenting it, it
serves variable-step hosts as well as fixed ones (B's case is A with
`dt` fixed at 1), and it changes nothing for a program that does not declare
it. Its cost is a new refusal for fractional `dt`, a reachable range limit,
and the unit conversions an author writes when opting in. It is not planned
for rc.13.

## Owner's decision (2026-10-04)

**A, the opt-in integer clock.** The owner picked A on the card. B is A with
`dt` fixed at 1. C would give each author a counter per timer and lose a
schedule bound to the occurrence it was scheduled against. One condition
applies to the specification:

- A constant `after` is checked when the program loads. A computed delay that
  is not a whole number needs a named refusal at dispatch, defined in the same
  specification. The host-side rule (round once at the host's boundary and
  carry the remainder, as in step 3 above) goes in the authoring guide
  (`docs/AI_AUTHORING.md`), where hosts with 16.67 ms frames such as
  Glowcap's handle it.

Nothing changes in rc.13. When these designs are built, the clock goes first,
because it is the smaller change. The save windows
([save-forgetting.md](save-forgetting.md)) follow with their own
specification.
