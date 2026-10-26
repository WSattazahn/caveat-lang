# CAVEAT Across Members 0.1 (draft) — a sum, or "any", over a kind's members

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable) until a 1.x release ships it. Draft for 1.5: specified on its branch, not yet in a release. It adds to [Repetition 0.1](caveat-repetition-0.1.md) and [Nested Repetition 0.1](caveat-nested-repetition-0.1.md) and changes no program that loads today: the new forms put `for` where an expression goes, which does not load in 1.4.

A [`for` block](caveat-repetition-0.1.md) writes a statement once per member
of a kind. An expression that reads every member is still written by hand.
The [intel example](../kit/examples/intel-betrayal/README.md) asks whether any
informant said a gate is clear:

```caveat
on go when gate == gate.$g and $g_ocelot_ok + $g_eva_ok == 0
    reject "None of your intel says that gate is clear.";
on go when gate == gate.$g commit $g_plan because enough using $g_ocelot_ok + $g_eva_ok;
```

Each informant is named in each sum, so a third informant means finding and
editing every one of them: the problem Repetition 0.1 solved for statements.
The [corpus sweep](#corpus-sweep) finds 14 such sums in 7 tracked programs,
among them `reef_one_seen + … + reef_six_seen` in `game/light_the_way.cav`.

This profile adds three expression forms that expand over a kind's members:

```caveat
on go when gate == gate.$g and sum(for informant as $i: $g_$i_ok) == 0
    reject "None of your intel says that gate is clear.";
on go when gate == gate.$g commit $g_plan because enough using sum(for informant as $i: $g_$i_ok);
```

## 1. The forms

```text
sum(for KIND as $NAME: EXPRESSION)
any(for KIND as $NAME: EXPRESSION)
all(for KIND as $NAME: EXPRESSION)
```

Each may stand wherever an expression may: in a guard, a `define`, a `set`,
a `bind`, a `using` basis, an argument, and inside another such form. KIND,
`as` and `$NAME` are what a `for` block's header holds; the `:` ends the
header, and EXPRESSION runs to the `)` that closes the form's `(`, counting
parentheses outside quoted text.

The word must start a name of its own: `my_sum(for …` and `glow::any(for …`
are not these forms. Whitespace may come between the word and `(`, and
between `(` and `for`.

## 2. Expansion

A form expands to EXPRESSION once per member of KIND, in declaration order,
each copy in parentheses, joined by ` + ` for `sum`, ` or ` for `any` and
` and ` for `all`, and the whole in parentheses:

```text
sum(for informant as $i: $g_$i_ok)   ->   (($g_ocelot_ok) + ($g_eva_ok))
```

Inside EXPRESSION:

- `$NAME` is the member's name and `$index` its one-based position, as in a
  block's body ([Repetition 0.1](caveat-repetition-0.1.md), section 2);
  `$index` is the form's own, and the enclosing block's position is named
  once outside it, as `define $g_position = $index;`.
- An enclosing block's `$NAME` is kept as written, for that block to replace,
  so `$g_$i_ok` becomes `north_ocelot_ok` in the `north` copy. A `$` word is
  read across every binding in scope as Nested Repetition 0.1 reads it: the
  longest binding that prefixes it wins.
- A comment is left out of the copies, which are written on one line, where
  a comment would hide the copies after it. A form written on one line moves
  no line of the program.

KIND's members are the ones Repetition 0.1 counts: the part's top-level
`entity` statements of the kind. Like a block, a form expands per bundle
part, before names are rewritten, over the kinds declared in that part.

The expansion is ordinary text the author could have written, so the
evaluator, the save formats and every view are untouched. `sum` adds numbers
and `any` and `all` join conditions, with the types and refusals the
hand-written `+`, `or` and `and` have.

## 3. Refusals

Each refuses the program at load, as a malformed `for` block does:

- a KIND no `entity` statement of the part declares;
- a header that is not `for KIND as $NAME:`, such as `as i` or a missing
  `:`;
- `$index` as the binding, or a binding an enclosing block binds;
- nothing but whitespace or comments after the `:`;
- a `(` that nothing closes;
- a `$` word no binding in scope prefixes, as in a block.

For example:

```text
no entity is declared `kind tower`, so `sum(for tower as $t: …)` has nothing to expand
expected `for KIND as $NAME: EXPRESSION` inside the parentheses, found: for informant as i: x
```

## 4. Checks

[`caveat check`](caveat-check-0.1.md) reads a program with these forms as it
reads their expansion, so a form and its hand-written expansion get the same
diagnostics.

## Not in this profile

- A `max`, `min` or count form. `sum` of a condition written as
  `if(C, 1, 0)` counts.
- A form over the members of a kind another part declares, as for blocks.
- A form over values rather than members: the set is fixed when the source is
  written, as Repetition 0.1 section 4 says.

## Corpus sweep

[`experiments/across-members-sweep/sweep.mjs`](../experiments/across-members-sweep/sweep.mjs)
reads the 208 tracked programs, comments and quoted text removed
([RESULTS.json](../experiments/across-members-sweep/RESULTS.json)):

- no program writes `sum(for`, `any(for` or `all(for` but the intel
  example, which this profile's change rewrites with two of them (its four
  scenarios unchanged), so every other program expands to itself, byte for
  byte;
- no program declares a name `sum`, `any` or `all` or calls one, so no
  meaning changes;
- 14 sums in 7 programs add one define per member of a kind by hand, such as
  `m1_support + … + m6_support` (Glowcap round 7 C3 and its fixtures) and
  `reef_one_seen + … + reef_six_seen` (`game/light_the_way.cav`). Each is the
  form's expansion and is left as written.

## Changes

- 2026-10-25: drafted from the [intel example](../kit/examples/intel-betrayal/README.md), whose sums named each informant.
