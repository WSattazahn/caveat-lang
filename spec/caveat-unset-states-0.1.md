# CAVEAT Unset States 0.1 — a state with no value until a rule sets it

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.5: it becomes stable when a 1.x release ships it. It adds to
[Reactive 0.1](caveat-reactive-0.1.md), [Dispatch 0.1](caveat-dispatch-0.1.md),
[Save 0.1](caveat-save-0.1.md) and [Check 0.1](caveat-check-0.1.md); it
changes no program that loads today.

Every state starts with a value ([Reactive 0.1](caveat-reactive-0.1.md),
"State and inputs"), so a program cannot say "nothing has set this yet". A
program that needs to stands in a number for it, and every rule that reads the
state must then know which number means "missing".

The third agent test (2026-10-31) showed what that costs. A part's rating was
the lowest capacity among the reports on it, and a part with no report was
UNRATED. A bridge was UNRATED if any of its parts was. The programs wrote
UNRATED as a large number (`1000`) or as a separate `_unrated` flag. One
inherited program took a bridge's rating as the `min` of its parts'. Against
`1000`, the `min` of the rated parts always wins, so a bridge with an
unreported part got a rating. Every event was accepted, every scenario its
author had written passed, and the record replayed. The wrong rule ran until
a later notice changed an answer that depended on it, and the next agent found
and fixed it. The runtime could not refuse the rule, because nothing in the
program said that `1000` was not a capacity.

This profile lets the program say so. A state declared `unset` has no value
until a rule sets it. Reading it before then refuses the event, so the first
event that runs such a rule fails, with the rule and the state named.

## Syntax

```caveat
state part_deck unset;
state part_deck unset min 0 max 200;

on report_deck set part_deck = capacity;
on withdraw_deck unset part_deck;
on rate when is_set(part_deck) and is_set(part_pier)
  set bridge = min(part_deck, part_pier);
```

```text
state NAME unset [min LOW max HIGH];
unset STATE                      -- an effect, like set
is_set(STATE)                    -- a boolean expression
```

- `unset` in a declaration takes the place of `= INITIALIZER`. Bounds are as
  for any state.
- The effect `unset STATE` and the predicate `is_set(STATE)` name a state
  declared `unset`. Naming any other state does not load, with the message
  "`STATE` always has a value; declare it `state STATE unset;` to use
  `unset`/`is_set`".
- A state declared `unset` may not appear in another state's initializer:
  an initializer runs before any rule.

Before this profile, `state NAME unset;` and a rule effect beginning `unset`
did not load. A program that defines `fn is_set` keeps its function, as
[Choices 0.1](caveat-choices-0.1.md) does for `choose`.

## Meaning

A state declared `unset` starts unset. `set STATE = EXPRESSION` gives it a
value, as for any state; `unset STATE` takes the value away again. Both are
effects, so a refused event changes neither ([Dispatch 0.1](caveat-dispatch-0.1.md),
"Atomicity").

Reading an unset state anywhere except inside `is_set(...)` refuses the event
as `evaluation/unset_state`. The refusal names the state and the rule that
read it. This covers every place a state is read: a rule's `when` condition,
a `set` expression, a commit's `using` and `citing`, a reading's value, and a
function's arguments, which [D034](../docs/LIMITS.md) evaluates once whether
the body uses them or not.

`is_set(STATE)` is true when STATE has a value. It never refuses. With `and`
and `or`, which short-circuit, `is_set(a) and a > 3` reads `a` only when it is
set. `if` evaluates only the branch it takes, so `if(is_set(a), a, 0)` reads
`a` only when it is set.

**Grounds and lineage.** `is_set(STATE)` reads STATE as reading it would: its
lineage, and where the runtime computes grounds, its grounds. A state with no
value has no lineage and no grounds. `unset STATE` takes all three away and
leaves the state as the program loaded it; a rule skipped by its guard adds
nothing to a state with no value.

**Bounds.** An unset state has no value to check. A `set` must fit the
declared bounds, as for any state.

### The bridge, written with this profile

```caveat
state deck unset;
state pier unset;
state bridge unset;

on report_deck when is_set(deck) set deck = min(deck, capacity);
on report_deck when not is_set(deck) set deck = capacity;
on report_pier when is_set(pier) set pier = min(pier, capacity);
on report_pier when not is_set(pier) set pier = capacity;

on rate when is_set(deck) and is_set(pier) set bridge = min(deck, pier);
on rate when not (is_set(deck) and is_set(pier)) unset bridge;

on permit when not committed(yes) and is_set(bridge) and bridge >= weight
  commit yes because enough;
on permit when not committed(refer) and not is_set(bridge)
  commit refer because enough;
```

The inherited wrong rule, `on rate set bridge = min(deck, pier);`, refuses the
first `rate` sent before both parts have a report: `evaluation/unset_state`,
naming `pier` and that rule. A scenario that sends a
report for one part and then `rate` fails at once, not after the news.

## What the runtime reports

| Surface | An unset state |
| --- | --- |
| Snapshot | is left out of `values`, `qualified_values` and `value_grounds`, and listed, in name order, under `unset`. `unset` is left out when no state is unset, so a program that declares none writes the same bytes as before. The view holds no state values and is unchanged. |
| Interface | its state entry gains `"unset": true`. The field is left out for every other state, so the interface of a program that declares none is unchanged. |
| State save | nothing new. A save holds the states that differ from what the program gives them when it loads, and a state with no value is as it loaded, so the save leaves it out and restore leaves it unset. A state that has a value is saved and restored as any state is. |
| `explain` | lists no value for it, as it lists none for a state it does not know. |
| Scenarios | an `expect` reads `/unset` like any other snapshot field: `{"/unset": ["pier"]}`, or `{"/values/pier": {"$absent": true}}`. |

**Restore** gains no refusal: no save field is new. A replay save replays the
events, and the states have values or not as the events leave them.

## Check

C013 `unset-read-unguarded` (always runs): a rule reads a state declared
`unset` as a value, and nothing guards the read. A read is guarded by an
`is_set(STATE)` operand of the rule's `when` condition, of an `and` to its left,
or of an `if` condition whose branch it sits in, and by a `not is_set(STATE)`
operand of an `or` to its left or of an `if` condition whose `else` branch it
sits in. The warning names the rule and the state. It reads the rule as
written: a read inside a define or a function's body is not seen. As with every check, it describes the
text; a rule that runs only after another rule sets the state is correct and
still warned. Programs that declare no `unset` state get no new warning, so
`check --strict` passes every program that passes it today.

## Why not a value that spreads

SQL's `NULL` and IEEE's `NaN` let an unknown value flow through arithmetic:
`min(deck, unknown)` would be unknown, and the bridge would be UNRATED with no
guard written. That is the handbook's rule, and it would have made the
inherited program right without anyone noticing it had been wrong. It also
gives every comparison a third answer, which every condition, commit and
scenario would have to handle, and it would let a wrong rule hide in a
different way: `unknown >= weight` must be either true or false when it
decides a commit, and either choice is silent. This profile keeps every value
a finite number, as Reactive 0.1 promises, and makes a missing one loud. A
program states what missing means with `is_set`.

## Why not refuse at load

Whether a state is set depends on which events have arrived, not on the
program's text, so the load cannot know. As elsewhere in Caveat, a failure
that depends on what the session has seen refuses the event, and only an
error in the program's shape stops it loading. C013 is the load-time half: it points at the
rules that could refuse.

## Not in this profile

- Making `unset` the default for states. That changes every program, and is
  listed for 2.0 ([2.0 candidates](../docs/2.0-CANDIDATES.md), item 7).
- Unset event parameters. A payload already must carry every parameter.
- An unset value for a whole kind (`for part as $p: …`). Each state is
  declared on its own, as today.

## Implementation

- Runtime: `Directive::State`'s `unset`, `StateCell::unset`, `Effect::Unset`,
  the `is_set` predicate (`reactive_expr::IS_SET`), the refusal in
  `unset_read`, `RejectionCode::UnsetState`, the snapshot's `unset` and the
  interface's; C013 in `runtime/src/reactive_check.rs`. `unset` is a reserved word
  in `runtime/src/link.rs`, so a module cannot declare a name `unset`.
- Tests: `runtime/tests/unset_states.rs`. A state declared `unset` starts
  with no value, takes one from `set`, loses it to `unset`, and keeps its
  bounds. Each kind of read before a set (a `when` condition, a `set`, a
  `because`, a function argument the body does not use, a commit's `using`,
  a reading) refuses as `evaluation/unset_state` and changes nothing, and the
  refusal names the state and the rule. `is_set` never refuses, and guards
  through `and`, `or` and `if`. The grounds and lineage of `is_set` and of
  `unset`; save and restore, with an unset state left out of the save; `fn
  is_set` kept; a program with no unset state writes no `unset`; the third
  test's inherited bridge rule refuses its first `rate`, and C013 names it.
  The existing suites hold every other program's bytes.

## Changes

- 0.1 (draft): first version. Found by the third agent test (gap 6).
- 2026-11-01: implemented for 1.5. A save needs no `unset` list after all:
  a state with no value is as the program loaded it, so a save leaves it out
  like any unchanged state, and restore gains no refusal. Scenarios read
  `/unset` with the existing `expect`. `explain` lists no value for an unset
  state rather than saying "unset".
