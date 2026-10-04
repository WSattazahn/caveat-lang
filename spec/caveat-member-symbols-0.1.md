# CAVEAT Member Symbols 0.1 (draft)

**Status:** draft, not implemented. Nothing in the runtime, kit, editor or
`caveat check` reads this profile. It records a design for the need that
[PR #68](https://github.com/WSattazahn/caveat-lang/pull/68) (`$Q`, routed
repetition section 10 on branch `feat/evidence-by-member`) tried to meet, and
why that branch is superseded rather than merged. A reference is written with
brackets, `TEMPLATE[Q]`, the form the owner chose on 2026-10-04 with the other
answers in section 12.

A `kind` parameter of an event is a member's position
([typed parameters](caveat-typed-parameters-0.1.md)). A `for` block over that
kind already declares, for every member, symbols whose names contain the
member's: `ev_$x`, `$x_points`. This profile lets a rule name, at dispatch,
the symbol that belongs to the member one of its event's `kind` parameters
names, without a copy of the rule for every pair of members.

## 1. The need

[Before the Rain](../game/before_the_rain.cav) confronts a witness with a
trace:

```caveat
event confront about kind exhibit, shown kind exhibit, recants in no yes;
```

Routing by `about` names the witness as `$w`. The trace has no name in the
source: `withdraw E because R` needs `R` written there, so the game wrote one
rule for each witness and each trace, seven rules that grow with both. PR #68
let a routed rule write `ev_$shown`, and expanded such a rule once for each
pair of members in the Repetition pre-pass. That met the need and measured,
on the same program:

- rules on `confront` 86 → 135, and an accepted `confront` 7–10 percent slower;
- the hand-written form loads for 61 exhibits and the `$Q` form for 44, because
  each stopped copy still spends a step of the event's work;
- 98 copies where 49 are meaningful, since `$shown` ranges over every exhibit,
  the witnesses too;
- one existing pattern newly refused (`$shown_n` in a block bound `$s`);
- an editor model that must replicate the expansion.

Every one of those follows from writing the rule once per pair. None follows
from naming a member by its parameter. This profile keeps the second and
drops the first.

## 2. Families

A *family* is a declaration in a `for KIND as $NAME` block whose name contains
`$NAME`. Repetition 0.1 writes it once per member, so the loaded program holds
one symbol per top-level entity of KIND in the block's part, all of one kind of
symbol. The family's *template* is the declared name with `$NAME` as its hole.

| Written in `for exhibit as $x { … }` | Template | Members |
| --- | --- | --- |
| `evidence ev_$x from "$x";` | `ev_[ ]` | `ev_x_farmer`, `ev_x_farmhand`, … |
| `state $x_points = 0 min 0 max 3;` | `[ ]_points` | `x_farmer_points`, … |
| `readings $p_checks from checks limit 8;` | `[ ]_checks` | `pr21_checks`, … |

Evidence, claims, caveats, reading streams, decision series, renewable
evidence and states can be family members. Bindings, cues, events, defines
and procedures cannot: a rule reads none of them by name. A family is keyed by
the entity's name, not by its position, so the loader can tell a member the
block did not declare (section 5) from one that it did. Nothing is declared by
this profile: the families are the ones programs already write. Symbols
declared one by one outside a `for` block are not a family, even when their
names share a prefix (section 12).

## 3. A reference

```text
TEMPLATE[Q]
```

written as the template with `[Q]` in its hole, where Q is a `kind K`
parameter of the rule's event and `TEMPLATE` is a family over K:

```caveat
for exhibit as $w routed by about {
    on confront when $w_points == [shown]_points reject "That trace agrees with them.";
    on confront when recants == recants.yes withdraw ev_$w because ev_[shown];
};
```

A reference names the symbol that belongs to the member Q names. It may
appear in an `on` rule's guard and effect wherever a written name of its
symbol's kind may appear there. For evidence: `withdraw`, `because`,
`reveal`, the evidence side of `qualify … with`, `qualified(…)`, `observed`,
`withdrawn`. For a caveat: the caveat side of `qualify … with`, `examine`,
`examined`, `carries`, `has_caveat`, `caveated`. For a reading stream or
decision series: `sample`, `latest`, `has_sample`, `history_count`,
`history_at`, `fold_history`, `permitted by latest(…)`, `commit`, `reopen`,
`committed`, `reopened`, `rests_on_withdrawn`. For a state: a read in an
expression. The place decides which kind of symbol fits, as it does for a
written name, and the loader checks it (section 5).

A reference appears nowhere else: not in a declaration, a `bind`, a `cue` or
a `proc` body, since none has an event whose parameter could name a member;
not in a `define`; and not as an argument to a procedure's symbol parameter,
since [procedure symbols](caveat-procedure-symbols-0.1.md) are specialized
when the program loads. A numeric argument may read one:
`call recant([shown]_points)` freezes the number as any argument is frozen.

A `define` may read event parameters and is inlined where it is read
([define 0.1](caveat-define-0.1.md)), so a reference in one would have a
meaning wherever an `on` rule whose event declares Q reads it. This profile
leaves it out of 0.1 (section 12): an author writes the reference in each rule.
A later profile may admit it without changing any program that loads today.

A rule may hold references to several parameters and to several families, and
a rule in a routed block, a plain block or no block at all may hold one. Two
parameters of one kind in one rule are ordinary:

```caveat
on hold when withdrawn(ev_[first]) or withdrawn(ev_[second]) reject "That account was taken back.";
```

`$NAME` and `$index` are unchanged. Brackets appear nowhere else in Caveat
source. `[Q]` is not a `$` word: Repetition 0.1
replaces `$` words before the loader reads the text, and a reference is read
by the loader. The text shows which member a rule is about and when that is
decided, which is the principle routed repetition section 1 states for P.

## 4. Dispatch

When a routed or plain rule runs, each reference is resolved before the guard
is read: Q's value is the position the payload carried, already checked to be
a whole position in range ([typed parameters](caveat-typed-parameters-0.1.md);
otherwise the event is `input/payload_invalid` before any rule runs); the
entity at that position is looked up by name in the family; the reference is
that symbol. From then on the rule is the rule an author writes by hand for
that pair. A withdrawal of unobserved evidence, a `latest` of an empty stream,
a commit on a decision in force: each fails or refuses exactly as the written
name would.

Resolution adds no step to the event's work, no provenance, no graph change
and no record. The journal, grounds, lineage, withdrawal record and effects
of the rule are those of the hand-written rule for the pair: a `kind`
parameter is a plain value with no provenance, which is the property PR #68's
test `a_copy_that_its_route_stops_adds_nothing_to_lineage` pins, and this
profile rests on it in the same way. If a later profile gives event parameters
provenance, this section must be revisited.

Snapshots, saves and the view do not change. A program that uses a reference
is a different text from one that does not, so its `source_id` differs, as for
any edit; restore accepts and refuses exactly as the [save contract](caveat-save-0.1.md)
says. A runtime older than this profile refuses a reference at load as an
unknown name.

## 5. Load-time checks

Every reference is checked when the program loads, so that nothing can fail at
dispatch that could not fail for the written name:

- Q is a parameter of the rule's event, declared `Q kind K`.
- `TEMPLATE` is a family over K: a `for K as $NAME` block in the same part
  declares a symbol whose name is the template with `$NAME` in the hole.
- The family has a member for **every** entity of K the parameter can name:
  every entity of K in the loaded program, in every part. An entity of K in
  another part, or in a `for` block, has no family member, and the reference
  is refused. Routed repetition section 9 lists these as cases where `$index`
  silently selects another member; here they are errors.
- The symbol's kind fits the place: a state is not withdrawn, evidence is not
  sampled, a stream is not a `because`.
- The reference is in an `on` rule's guard or effect, and not where section 3
  excludes it.

An error stops loading at the first one found, in source order. In a bundle an
error is prefixed with the part's name, as Repetition 0.1's errors are.

- **No such parameter.**
  ```text
  `[Q]` in `on EVENT`: `EVENT` declares no parameter `Q`
  ```
- **A parameter of another form.** FORM is the parameter as declared after
  its name, such as `id`, `in no yes` or `min 0 max 3`.
  ```text
  `[Q]` in `on EVENT`: `EVENT` declares `Q FORM`, not `Q kind KIND`; a reference names a member by a kind parameter
  ```
- **No such family.**
  ```text
  `TEMPLATE[Q]` in `on EVENT`: no `for KIND` block in this part declares `TEMPLATE` with its binding in the hole
  ```
- **A member without a symbol.** ENTITY is the entity's name as declared, and
  WHERE is `in PART` or `in a for block`.
  ```text
  `TEMPLATE[Q]` in `on EVENT`: `Q` can name `ENTITY`, declared WHERE, but `TEMPLATE` has no symbol for it
  ```
- **A symbol of the wrong kind.** PLACE names the position, such as
  `withdraw`, `because` or `latest`.
  ```text
  `TEMPLATE[Q]` in `on EVENT`: `TEMPLATE` names a SYMBOLKIND, which PLACE does not take
  ```
- **A reference outside a rule.** CONTEXT is `define NAME`, `bind NAME`,
  `cue NAME`, `proc NAME` or the declaration's first word.
  ```text
  `TEMPLATE[Q]` in CONTEXT: a reference needs the event of an `on` rule to name a member
  ```
- **A reference as a symbol argument.**
  ```text
  `TEMPLATE[Q]` as the SYMBOLKIND argument of `PROC`: a procedure takes a symbol by name when the program loads
  ```

## 6. Cost

A rule with references is one rule. It costs the event one step whether its
guard holds or not, as any rule does, and its guard and effect cost what they
cost written by hand. A block of W members with such a rule writes W copies,
as a plain or routed block does today, not W×|K|. The loader's work count, the
4,096-step event budget and the expression limits are unchanged, and no new
limit is introduced.

## 7. Before the Rain

The two blocks that PR #68 rewrote with `$shown`, written with references:

```caveat
// What was shown must be an examined trace.
for exhibit as $s routed by shown {
    on confront when $index <= 3 reject "Show them a trace.";
    on confront when $s_points == 0 reject "That trace shows nothing yet.";
};

// A recant takes the account back because of the trace that was shown, and
// only a trace that says otherwise can bring one.
for exhibit as $w routed by about {
    on confront when $w_points == [shown]_points reject "That trace agrees with them.";
    on confront when recants == recants.yes withdraw ev_$w because ev_[shown];
};
```

Seven members, four rules: 28 rules on `confront` from these blocks. The
hand-written form writes 63 (a plain block of 5 rules and a routed block of
4, each over 7 members); PR #68's form writes 112 (14, plus two rules written
once per pair of members, 98). In the farmhand's copy, the last rule at
dispatch of `confront about=2 shown=5 recants=yes` is, once `ev_[shown]` is
resolved:

```caveat
on confront when about == 2 and recants == recants.yes withdraw ev_x_farmhand because ev_x_can;
```

That is the hand-written rule without its `shown == 5` conjunct, which
compared two plain parameters and left nothing in lineage.

The hold block, which `$Q` could not write (it names a member "by two
parameters in one rule"), becomes rules outside any block:

```caveat
on hold when [first]_points == 0 or [second]_points == 0 reject "That exhibit is not established.";
on hold when withdrawn(ev_[first]) or withdrawn(ev_[second]) reject "That account was taken back.";
on hold when [first]_points != suspect or [second]_points != suspect reject "That exhibit does not point at them.";
on hold set case_basis = case_basis + qualified(0, ev_[first]) + qualified(0, ev_[second]);
on hold when first > 3 set hold_firsthand = hold_firsthand + 1;
on hold when second > 3 set hold_firsthand = hold_firsthand + 1;
```

The first rewrite (the two routed blocks) is claimed equivalent to the
hand-written pairs: one member's copy runs, and its text is the hand-written
rule. The second rewrite is **not** claimed equivalent by this draft: an `or`
guard and one `set` that reads two evidences may leave different lineage from
two copies with `first == $index or second == $index` guards and two `set`s.
Section 11 makes the differential decide, and the hold block stays as written
unless it passes.

## 8. Check

`caveat check` reads a reference as a read of every member of its family:
C002 (no reopening path) counts `latest([Q]_checks)` as a read of each
member's stream, and a decision made on it has a reopening path when any
member's stream can reopen it. C003 does not count a reference as a selection
by `$index`, and a rule outside a block that holds only references is not a
member rule: it is about the members its event names. C004 is unchanged;
references do not use `$index`. A reference to a family whose block is routed
by the same parameter, such as `ev_[about]` in a block routed by `about`,
means the block's own member and is reported as a pattern worth a second look,
under the advisory code C005, since `ev_$x` says the same thing and the route
already selects it. Like C003 and C004, C005 reports and never refuses.

## 9. What it does not do

- It does not select a member held in a state. A state is a number with
  provenance and a range of its own; which member it means is the author's,
  as in routed repetition section 9.
- It does not pass a member's symbol to a procedure's symbol parameter.
- It does not read a reference in a `bind`: bindings read no event parameter
  ([reactive 0.2](caveat-reactive-0.2.md)).
- It does not read a reference in a `define` (section 3).
- It does not make members private, prevent a rule from naming another member
  by its written name, or check that Q is the parameter the rule is about.
- It does not change which rules a routed block routes, or what a plain block
  expands to.

## 10. Lean

The verified fragment does not cover reference resolution. This profile claims
no proof; the tests in section 11 are its evidence.

## 11. Acceptance before adoption

PR #68 set its own bar: finished code, all gates passing, and adoption
deferred "pending independent evidence from 2–3 additional real programs
requiring cross-member naming." This draft keeps that bar and adds what the
design must show:

1. **Equivalence.** Before the Rain in the hand-written form and in the
   section 7 form (routed blocks only) produce byte-identical snapshots, views,
   saves, journals and `explain --json` on its 12 scenarios and on the
   3,000-random-history differential PR #68 used; zero divergences. Then the
   hold rewrite is tried under the same differential and adopted only if it
   also shows zero.
2. **Cost.** Rules on `confront` and median accepted-event time at or below
   the hand-written form, on the same machine and protocol as
   `experiments/performance-0.1/`; the exhibit count at which the program
   stops loading is unchanged from the hand-written form.
3. **Compatibility.** Every program in the repository and every registered
   study source loads and expands byte for byte as before; no existing
   `$` word changes meaning.
4. **Errors.** Each section 5 error has a test with its exact text, including
   an entity of K in another part of a bundle and one in a `for` block.
5. **Refusals.** Each dispatch failure a reference can reach (unobserved
   evidence, empty stream, decision in force, not permitted) is the same
   classified outcome as for the written name, through `dispatch`,
   `dispatchView`, serve, replay and the scenario runner.
6. **A second program.** At least one program other than Before the Rain, not
   written to exercise this profile, is simpler with it. The hold block in
   section 7 counts only if its differential passes.

## 12. Owner decisions

The draft left four questions to the owner, who answered them on 2026-10-04.
Each answer settles the design only; implementation still waits for
section 11.

- **The sigil.** Brackets: `ev_[shown]`. Reusing `$` (`ev_$shown`, the form the
  game's author first wrote) would give one sigil two binding times, and `@`
  already separates a name from its occurrence ordinal in saves. A distinct
  mark keeps section 3's principle.
- **`define` and `bind`.** Neither holds a reference in 0.1. A `bind` reads no
  event parameter. A `define` could, but is left out until a program needs it
  (section 3).
- **The advisory.** Wanted, as Check code C005 (section 8). It reports and
  never refuses.
- **Families.** Only a `for` block's template forms a family. Evidence declared
  by hand with a consistent prefix does not count: completeness (section 5) is
  checkable because a family is a block's template.

## Changes

- 2026-10-03: draft recorded. Supersedes the `$Q` design of PR #68
  (`feat/evidence-by-member`, routed repetition section 10 on that branch),
  which is closed unmerged and preserved as the measurement record. No
  runtime, kit, editor or check change accompanies this draft.
- 2026-10-04: the owner's answers to section 12 recorded. The sigil is brackets,
  `TEMPLATE[Q]`;
  `define` and `bind` hold no reference in 0.1, and section 3 now gives the
  `define` reason correctly (a define can read event parameters); the
  section 8 advisory is C005; only `for`-block templates form families. Still
  a draft, not implemented.
