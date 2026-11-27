# CAVEAT Choices 0.1 — `lowest`, `highest` and `choose`, whose comparison is lineage, not grounds

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Proposed for 1.5 and not implemented: it becomes stable when a 1.x release ships it. It adds to
[Only If 0.1](caveat-only-if-0.1.md) and [Reactive 0.7](caveat-reactive-0.7.md);
it changes no program that loads today.

A value's grounds are what it rests on, and its lineage is everything that
could have influenced it ([Only If 0.1](caveat-only-if-0.1.md)). Inside an
expression, everything an `if(...)` reads is grounds, its condition too. `min`
and `max` are source functions written with `if`
([Reactive 0.4](caveat-reactive-0.4.md)), so `min(a, b)` rests on both `a`
and `b`, whichever is smaller.

The third agent test (2026-10-31) showed what that costs. A bridge's rating
was the `min` of its parts' ratings, and each part's rating the `min` of the
inspection reports on it. Every report on a bridge became a ground of the
bridge's rating, and of every route, permit and convoy answer built on it:
one answer rested on 94 reports and one record listed 24 retracted reports
under answers they could not have changed. Retracting a report that was not
the lowest on its part made `rests_on_withdrawn` true for every answer
downstream, although none of their values moved. The agents and their
readers filtered these by hand.

Only If 0.1 left this open: "A choice between two values (`if` with the
condition as lineage only)". This profile adds that choice, and the two
comparisons most programs need it for.

## Syntax

```caveat
choose(CONDITION, YES, NO)
lowest(A, B)
highest(A, B)
```

CONDITION is a boolean expression; YES, NO, A and B are numeric. Each form
may appear wherever a numeric expression may, nested in any other.

## Meaning

- `choose(CONDITION, YES, NO)` is YES when CONDITION holds and NO otherwise.
  Only the chosen branch is evaluated, as with `if`.
- `lowest(A, B)` is `choose(A < B, A, B)`, and `highest(A, B)` is
  `choose(A > B, A, B)`. Both arguments are evaluated. The value is that of
  `min(A, B)` and `max(A, B)` for every input, ties included: on a tie the
  value is B's.

**Grounds.** CONDITION's reads are not grounds, as with `only_if`. Where the
runtime computes grounds (a commit's `using` and `citing`, a state's
grounds), `choose` adds the chosen branch's grounds and nothing else. So
`lowest(A, B)` rests on whichever argument it returned: A when A is smaller,
B otherwise, including on a tie. The other argument was read only to compare,
and stays out. This holds through nesting: `lowest(r1, lowest(r2, r3))` rests
on the one report whose value it returned.

**Lineage.** Everything read is lineage: CONDITION, the chosen branch, and
for `lowest` and `highest` both arguments. `explain` lists the evidence read
only to compare under "could also have been influenced by".

Everything that depends on grounds follows, with no change of its own:
`rests_on_withdrawn`, `in_basis`, the pins that keep a ground's record, the
undo list and the caveats a decision carries. Withdrawing a report that was
not the lowest leaves a `lowest` over it, and every decision built on it,
without a withdrawn ground. Its value does not change either, because the
report was not the one returned. Withdrawing the report that was returned
makes `rests_on_withdrawn` true downstream, which is when the answer may be
wrong.

A program that does not use these forms is unchanged: `min`, `max` and `if`
keep their grounds. A program that defines `fn choose`, `fn lowest` or
`fn highest` keeps its function. Its calls are that function's, with the
arguments as written. Without such a function, the forms did not load before
this profile.

## Why not change `min` and `max`

`min` and `max` would give the narrower grounds with no new names, but the
grounds of every program that uses them would change, and with them its
`rests_on_withdrawn` answers, undo lists and View output. The
[1.x promise](../docs/COMPATIBILITY.md) keeps those bytes. A program opts in
by writing `lowest` where it means "rests on the smallest".

## Ties

On a tie both arguments give the same value, so removing either alone would
not change it, and neither is strictly what the value rests on. The profile
keeps one: B, the one `min` returns. That keeps `lowest` a drop-in for `min`
in value, and gives a tie a single, fixed ground, so a save restores to the
same grounds. A program that wants both on a tie writes `min`.

## Not in this profile

- A `lowest` or `highest` over a whole kind, such as
  `lowest(for report as $r: ...)`. A program nests two-argument calls. From
  1.5 a function's argument is expanded once ([D034](../docs/LIMITS.md)), so
  nesting grows linearly.
- Grounds that leave out a decision's upstream values. A decision that uses
  another keeps that decision's grounds, as today.

## Proofs to add with the implementation

`runtime/tests/choices.rs`: the value of `lowest` and `highest` equals `min`
and `max` on random finite inputs, ties included; grounds and lineage of a
commit and of a state; withdrawal of the returned argument and of the other
one; nesting; `choose` not evaluating the branch it does not take; save and
restore; and `fn choose`, `fn lowest` and `fn highest` kept. The third test's
bridge record, rewritten with `lowest`, lists only the returned report under
each part.

## Changes

- 0.1 (draft): first version.
