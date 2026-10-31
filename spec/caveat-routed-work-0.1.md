# CAVEAT Routed Work 0.1 — a rule on another route costs no work

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable) until a 1.x release ships it. Draft for 1.5. It adds one statement, `work skips other routes;`, which does not load in 1.4, so every program that loads today keeps its output bytes and exit statuses. Tested in `runtime/tests/routed_work.rs`; the regression case for the finding is `gap_2b_routed_copies_for_other_members_cost_no_work`.

## Why

[Reactive 0.7](caveat-reactive-0.7.md) bounds each dispatched event at 4,096
steps of work. It counts every top-level rule on the event, whatever its guard
finds. A [routed block](caveat-routed-repetition-0.1.md) over N members gives
each of its routed rules N copies on the same event, and at most one of them
can run. One routed rule over 4,200 members is therefore refused with
`limit/work_limit`, though 4,199 of its copies could never run. A plain block
with the selection written by hand, as in
`on retract when report == report.$r …`, is refused the same way. In the third
agent test, a program with 130 report members could not retract a report:
each retraction counted every member's copies of its rules, and passed 4,096
steps.

The limit bounds the work an event does. A copy whose own guard excludes it,
from the payload alone, does none. Counting it anyway is what 1.x does, and
keeps doing, so a program opts out:

```caveat
work skips other routes;
```

## Syntax

```text
work skips other routes;
```

The statement is exactly these four words. A program declares it at most
once, anywhere among its statements. It reserves nothing: any other statement
that begins with `work` reads as before, so `work supports wet;` is still a
relation about evidence named `work`, and a state, stream, event or parameter
may still be named `work`, `skips`, `other` or `routes`.

These do not load:

| Source | Message |
| --- | --- |
| the statement twice | `work skips other routes is declared more than once` |
| the statement in a module of a draft 0.5 bundle ([Caveat 0.5](caveat-0.5-draft.md)) | ``module M declares `work skips other routes;`; how an event's work is counted is the program's to say, not a module's`` |

An event's work is counted across the whole program, so the program declares
how, as it declares its attention budget, and the declaration covers the
rules its modules contribute. A module may still use the four words as names.

## Routes

A top-level rule's **route** is the run of terms its guard evaluates first
that have the form `NAME == VALUE`, joined by `and`, however parenthesized.
The run ends at the first term of any other form. In each term:

- `NAME` is a parameter of the rule's event.
- `VALUE` is a number literal or a [threshold](caveat-thresholds-0.1.md),
  under any number of `+` and `-` signs, such as `2`, `-1` or `-margin`; or,
  with no sign, a name the program fixes when it loads: a typed parameter's
  member such as `report.r5`
  ([Typed Parameters 0.1](caveat-typed-parameters-0.1.md)), or a source
  coordinate such as `reef.x` ([Reactive 0.1](caveat-reactive-0.1.md)). A name
  that a state or a parameter of the event answers is not such a value, and
  neither is a signed name such as `-reef.x`.

Routes are read after `define`s are expanded, so a define written in a guard
reads as the expression it names. A function call is a term of another form,
whatever its body.

A routed copy's guard starts with its route, `P == $index`
([Routed Repetition 0.1](caveat-routed-repetition-0.1.md), section 3). A guard
written by hand, such as `target == $index and …` in a plain block or
`report == report.r5 and …`, has one too. A guard that starts with a term of
any other form has none: `$index == target`, `target != 2`, `target == n` where
`n` is a state, `loud > 0`, and `n >= 0 and target == 2`.

When the payload gives a term's parameter a value other than the term's, the
rule is **routed elsewhere**. Its guard is then false and rests on nothing,
since a payload parameter and a fixed value are plain: evaluating it would
find exactly that. `and` stops at its first false operand, so nothing after
that term would be evaluated either.

## Work

In a program that declares `work skips other routes;`, a rule routed
elsewhere costs no work. Neither the step [Reactive 0.7](caveat-reactive-0.7.md)
counts for a top-level rule nor the steps of the skipped procedure effects it
traverses are counted.

Everything else about the rule is unchanged. It is a skipped rule, and its
effect is retained as any skipped effect whose guard rests on nothing is. That
includes, in a program with a [window](caveat-windows-0.1.md), the withdrawal
collector's reading of the effect and of each step of a procedure it calls
([Withdrawal Collection 0.1](caveat-withdrawal-collection-0.1.md)).

Every other rule is counted as before. That covers a rule whose route the
payload matches, whether the rest of its guard then holds or not, and a rule
with no route. Inside a call that runs, every step is counted as before: a
step's guard is not a top-level guard, and it reads its procedure's
arguments, which carry their basis ([Reactive 0.7](caveat-reactive-0.7.md)).
The limit itself is unchanged: 4,096 steps across one dispatched event.

A program without the declaration counts every rule, as 1.4 does.

## Compatibility

A program without the declaration is unchanged, byte for byte. A runtime
before 1.5 refuses the declaration as a statement it cannot parse, so no
program that loads in 1.4 has it.

Adding the declaration to a program changes its source's identity, which is
its `source_id`, its SHA-256 and the archive references made under them, and
the events that the program without it refuses with `work_limit`. From the
same state, every other event has the same outcome, state, view and save,
apart from that identity. An event the program without the declaration
refuses with `work_limit` may, with it, be accepted; or be refused by what the
exhausted budget used to stop short of, such as a later rule, a later step of
a procedure or a binding after the rules, with that refusal's own origin and
code; or be refused with `work_limit` later in the event. Once one event's
outcome differs, the two sessions' states may differ, and so may the outcomes
of the events after it.

A save binds the exact source it was made under, so a save made without the
declaration does not restore onto a source that adds it
([COMPATIBILITY.md](../docs/COMPATIBILITY.md#saves-archives-and-source)).

## Time

An event's time still grows with the number of rules on it. A rule routed
elsewhere is passed over without evaluating its guard, but it is not free of
time. In a program with a window, the collector still reads its effect and
every step of the procedures it calls, and the work limit no longer bounds
that reading. The program's size does: a procedure expands to at most 4,096
steps ([Reactive 0.7](caveat-reactive-0.7.md)).

## Questions settled

1. **Opt-in, not a default.** The [versioning
   policy](../docs/COMPATIBILITY.md#versioning-policy) keeps any change to
   what an accepted program refuses for 2.0, and a save restored under a later
   1.x gives the outcomes the earlier runtime would have given
   ([Saves, archives and source](../docs/COMPATIBILITY.md#saves-archives-and-source)).
   Counting rules routed elsewhere as free by default would turn some
   `work_limit` refusals into acceptances. Making it the default is a
   candidate for 2.0, which would keep the statement loading.
2. **The program's, not a block's.** A plain block with its selection written
   by hand, and a rule outside any block, run into the limit as a routed block
   does, and an event has one budget across the program.
3. **Only leading `NAME == VALUE` terms.** Their result follows from the
   payload alone and rests on nothing, so skipping the rule is exactly what
   evaluating its guard would find. A term that reads a state, a history or a
   function call can rest on something, and a skipped effect retains what its
   guard rests on.
4. **No new code or field.** A refusal keeps its code. No view, save,
   explanation or check result names the declaration.

## Not in this profile

- Routing the steps of a procedure.
- A way to raise the limit, such as `work limit N;`.

## Corpus sweep

The repository tracks 218 `.cav` files, and 187 of them load as reactive
programs. None declares the statement. Against main at 4491c16, the runtime
of this profile gives every one of them the same output: the 2,393,600
dispatches of `runtime/examples/outcome_sweep.rs` and the 2,751,335 lines of
`runtime/examples/caveat_sweep.rs`. With rules routed elsewhere counted as
free in every program, as if each declared the statement, the output was the
same as well: none of their events reaches the limit.
