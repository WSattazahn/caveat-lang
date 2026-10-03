# CAVEAT Routed Repetition 0.1

**Status:** implemented. The pass is in `runtime/src/repeat.rs`, and C003
skips routed blocks in `runtime/src/reactive_check.rs`. Sections 1 to 8 are
tested in `runtime/tests/routed_repetition.rs` and `runtime/tests/check.rs`.
The converted agent ledger passes the ledger's 11 scenarios
(`kit/test/commands.test.mjs`). Section 9 lists what routing does not
prevent. Of those limits, only entities of KIND in another part has a test.
A runtime older than this profile refuses a `routed by` header as a
malformed `for` block.

The [agent ledger](../experiments/agent-ledger/ledger-identifiers.cav) writes
one [repetition](caveat-repetition-0.1.md) block over its pull requests. All
31 of its rules begin the same way:

```caveat
on approved when target == $index and commit != $p_head reject "A go-ahead for a commit that is not the head.";
on approved when target == $index sample $p_approvals = commit supports $p_permitted;
```

A `for` block is a text expansion. Each member gets a copy of every rule, and
every copy runs on every event its guard allows. `target == $index` is what
keeps a copy with its own pull request. Without it, an event about one pull
request acts for all of them: one go-ahead is recorded in every pull request's
stream. In a study of the ledger, each of the 31 rules was written again
without its selection, one at a time. Every such program loaded and checked
clean until [C003](caveat-check-0.1.md) reported them.

C003 finds a missing selection. This profile writes it. The block names, once,
the event parameter that says which member an event is about, and the
expansion adds the selection to each rule on such an event.

This is routing assistance, not a proof that members are isolated. A rule in a
routed block can still name another member's streams. Section 9 lists what
else it does not guarantee.

## 1. The header

```caveat
for pr as $p routed by target {
    on approved when commit != $p_head reject "A go-ahead for a commit that is not the head.";
    on approved sample $p_approvals = commit supports $p_permitted;
};
```

```text
for KIND as $NAME routed by P { BODY };
```

This is a Repetition 0.1 block with one more clause. P is the name of an event
parameter: a plain name, with no `$` and no dots. Everything Repetition 0.1
says still holds: `$NAME` and `$index`, member-major expansion, one part at a
time, no nesting.

P is written, not inferred. An event can name two members of one kind, as
`event stacked target kind pr, base kind pr;` would, and an inferred P would
pick one without saying so. A written P is also checked against every event
the block's rules are on (section 7).

A block routes by one parameter. A rule is about one member, and the events a
block reacts to name it the same way. Two parameters would need a rule for an
event that has both, and whichever it chose would not show in the text. An
event that names a member of the kind only by another parameter is an error:
the author routes the block by that parameter, or keeps the rules on it in a
plain block. Renaming the event's parameter instead would change what every
host sends and every recorded history holds.

A plain block is unchanged. Its header is still exactly `for KIND as $NAME`,
so every existing program expands byte for byte as before. Routing is a
profile of its own rather than a new section of Repetition 0.1, because it
reads event declarations, which Repetition 0.1 never does, and it has errors
of its own.

## 2. Which rules are routed

A rule is *routed* when all of these hold:

- it is an `on EVENT` rule written in the block's body;
- EVENT is written without `$NAME` or `$index`;
- EVENT declares a parameter `P kind KIND`.

This is decided from EVENT alone. Every such rule is routed. There is no
exception for one rule, and the pass does not read what the rule does:

- A `reject` is routed. It then refuses an event only when the member the
  event names meets its guard. Unrouted, any member's copy whose guard held
  would refuse the event, whichever member it was about. That is why the
  ledger routes its rejects by hand today.
- A `call` is routed like any other effect. A procedure's steps are not rules
  and are not changed. They run when the routed rule calls them.
- A rule with no guard gets one.
- A guard that already selects the member keeps its selection, and the rule
  is routed as well, with no attempt to remove the repeat. Selecting the same
  member twice does not change the condition, but the extra comparison still
  counts toward the loader's expression limits (section 3).
  The pass reads a guard only to find where it ends and whether it has an
  `or` (section 3). It does not look for a selection, because a selection has
  many forms, and one it missed would expand differently from one it found.
  When converting a block, delete the hand-written selection.
- Any other comparison with P stays, and is read together with the route. In
  a routed block, `target == target.pr21` limits a rule to pr21's own copy,
  and `target != $index` means the rule never runs. Such a rule means
  something else in a plain block, where `target == target.pr21` lets every
  member's copy run on pr21's events. Converting a block changes it.

A routed rule must mention `$NAME` or `$index`, quoted text included and
comments not, as C003 reads a rule. One that mentions neither is an error
(section 7). Its copies are all the same rule. In
a plain block every copy runs on each event. Routed, only one would. What it
did would then depend on the header, and C003 does not check such a rule, so
dropping the clause would multiply it with no report. Written once, outside
the block, it runs once per event either way:
`on observe set observations = observations + 1;` belongs there.

Two kinds of rule in the block are not routed. They expand as written, as in a
plain block:

- A rule on an event that has no parameter P and no parameter of kind KIND,
  such as `tick`, or a `stop` for the whole repository. The event is not about
  one member, so there is nothing to select, and every member's copy runs on
  it by design. C003 does not check these rules, for the same reason.
- A rule on an event written with `$NAME` or `$index`, such as
  `on absorb_$m`. Each copy runs on its own member's event, so the event
  already selects the member.

Any other rule on an event is an error (section 7).

A statement that is not an `on` rule expands as written: the declarations of
claims, evidence, readings, decisions, renewables, states and events,
`define`, `bind`, `cue`, `proc` and the rest. None of them is on an event, so
there is no member to select. A define that a routed rule reads is inlined, as
one operand, into a guard that already has its route. A binding and a
procedure step are the same in a routed block as in a plain one.

## 3. Where the route goes

A routed rule is expanded as if `P == $index` had been written at the front of
its guard.

| Written in the block | Expanded as if written |
| --- | --- |
| `on E EFFECT` | `on E when P == $index EFFECT` |
| `on E when G EFFECT` | `on E when P == $index and G EFFECT` |
| the same, with an `or` in G outside parentheses | `on E when P == $index and (G) EFFECT` |

Repetition 0.1 turns `$index` into the member's position among the part's
top-level `entity` statements of kind KIND. A `P kind KIND` parameter takes an
entity's position among every entity of KIND in the loaded program
([typed parameters](caveat-typed-parameters-0.1.md)). The two are the same
count when every entity of KIND is a top-level `entity` statement in the
block's part. Repetition 0.1 reads each of those as the loader does, one with
a comment inside it or a last one without its `;` included. Within the part,
an entity of KIND in a `for` block is an error (section 7). For the parts of a
bundle, see section 9.

The route is placed in each member's copy of the rule, with `$NAME` and
`$index` already replaced as Repetition 0.1 replaces them. That copy is the
text the loader reads. The rule as written in the block is not: in it,
`sample $p_approvals = …` is not a complete effect until `$p_approvals` is a
name, and `examine $r_fog cost $index` is not one until `$index` is a number.

A rule has a guard when the word after EVENT is `when`. Then `P == N and ` goes
directly before the guard's first word, after `when` and whatever space or
line break follows it, where N is the member's position. Otherwise
` when P == N` goes directly after EVENT. Neither needs the rest of the rule
to be read, so every routed rule gets its route.

G is the guard as the loader reads it: the words after `when`, up to the word
where the loader finds the rule's effect. An effect word can also be a name
inside a guard, so the loader takes the first effect word outside parentheses
that begins a complete effect, and the pass finds the effect the same way, in
the copy. A qualified name such as `glow::level` is read as the one name
linking makes of it, so its `::` neither ends the guard nor hides an `or`. When
G has an `or` outside parentheses, `(` goes directly before G's first word and
`)` directly after its last.

`or` is the operator as the loader's expression reader reads it: the word
`or`, whether spaces surround it or a parenthesis touches it. `a or b`,
`a or(b)`, `(a)or b` and `(a)or(b)` each have one. An `or` does not count
inside parentheses, quoted text or a comment, inside a longer name such as
`order`, or inside a define the guard names, which is read as one operand.

Nothing else changes: spacing, line breaks, comments, quoted text and the
guard itself are kept. Routing adds no line, so a diagnostic in the expanded
text has the line it has in the expansion of the same block without the
clause.

The route comes first. `and` reads left to right and stops at its first false
operand, so in the copy for any other member the rest of the guard is not
reached, as with the hand-written `target == $index and …` today. It is also
the place and the form the ledger's hand-written selections have, so a
converted block expands to the same text (section 5).

The parentheses keep the guard one operand. `and` binds tighter than `or`, so
`P == N and A or B` would mean `(P == N and A) or B`, and B alone would run
the rule for every member. `not`, comparisons and arithmetic bind tighter than
`and`, and a chain of `and` means the same however it is grouped, so no other
guard needs them.

"Means the same" is about the condition only. The route is one more
comparison and one more `and`, and the parentheses one more level, and all of
it counts toward the loader's limits on expression size and nesting like any
text the author wrote. A chain of `and` nests one level per operand, so a
guard that is at the nesting limit as written fails to load once routed, and a
hand-written selection left in place adds a level more:

```text
expression exceeds nesting limit 64
```

The pass does not remove repeats or regroup a guard to stay under a limit. It
adds exactly the text in the table above.

A copy in which no word begins a complete effect cannot be loaded, whatever
the pass does. It still gets its route, at the front, and loading refuses it.

## 4. Order

A routed block expands member-major, like any `for` block: the whole body for
the first member, then for the second, in declaration order. Each copy keeps
its place, and the rules keep the order they are written in. Routing changes
guards only. It adds, removes and moves no statement.

It follows no dependency between rules and creates nothing at run time. The
member set is the declared entities, fixed when the source is written, and a
route compares an event parameter with a number fixed when the source is
expanded.

## 5. The agent ledger

Today the ledger writes the selection on each of its 31 rules:

```caveat
for pr as $p {
    …
    on approved when target == $index and commit != $p_head reject "A go-ahead for a commit that is not the head.";
    on approved when target == $index sample $p_approvals = commit supports $p_permitted;
    …
    on merge when target == $index and $p_revisions == 0
        commit $p_merge because enough using latest($p_checks)
        permitted by latest($p_approvals) for $p_head;
    …
};
```

Routed, the header says it once:

```caveat
for pr as $p routed by target {
    …
    on approved when commit != $p_head reject "A go-ahead for a commit that is not the head.";
    on approved sample $p_approvals = commit supports $p_permitted;
    …
    on merge when $p_revisions == 0
        commit $p_merge because enough using latest($p_checks)
        permitted by latest($p_approvals) for $p_head;
    …
};
```

Both expand, for `pr26`, the second pull request declared, to:

```caveat
    on approved when target == 2 and commit != pr26_head reject "A go-ahead for a commit that is not the head.";
    on approved when target == 2 sample pr26_approvals = commit supports pr26_permitted;
    …
    on merge when target == 2 and pr26_revisions == 0
        commit pr26_merge because enough using latest(pr26_checks)
        permitted by latest(pr26_approvals) for pr26_head;
```

The conversion adds `routed by target` to the header, deletes
`target == $index and ` from 20 rules, and deletes ` when target == $index`
from the other 11. Every event the block's rules are on declares
`target kind pr`, so all 31 rules are routed. Every rule mentions `$p`, and
the four pull requests are top-level `entity` statements, so no error applies.
No guard has an `or`, so none gets parentheses. The converted ledger expands
to exactly the text today's ledger expands to, byte for byte: the same rules,
in the same order, with the same guards. This was checked with the
evaluation's prototype of the pass.

The same expansion is not the same source. A save records its program's
`source_id`, and a restore refuses a save made by a different program
([save](caveat-save-0.1.md)). The converted ledger is a different text from
today's, so a save made by today's ledger does not restore into it, even
though every rule is the same once expanded. The same is already true of a
plain block and its expansion written out by hand. Routing does not change how
a program's source is identified or what restore accepts. This profile adds
no canonical source identity and no migration of saves.

## 6. A rule for every member

A routed block has no way to leave one rule unrouted. A rule that should run
for every member on an event that names one, such as Trail Rescue's reset of
every tunnel's tallies on each observation, is written in a plain block over
the same kind, with the allow comment C003 asks for:

```caveat
for tunnel as $t routed by target {
    state $t_reported = 0 min 0 max 1;
    state $t_support = 0;
    on observe when method == method.report and $t_reported == 1 reject "That visitor has already reported.";
    on observe when method == method.report set $t_reported = 1;
};

for tunnel as $t {
    # caveat check: allow unrouted-member-rule
    on observe set $t_support = 0;
};
```

An exception inside the block would be new rule syntax for something the
language already writes, and the case is rare: two rules in the repository's
programs. Written this way, the intent shows twice. The rule is outside the
routed block, and it carries the allow comment.

Moving a rule moves it in the order. In Trail Rescue's one plain block, each
tunnel's copy of the reset runs right after that tunnel's own rules. Split as
above, every copy runs after every tunnel's routed rules on `observe`. Rules
on other events are not affected. Where one rule on the event reads what
another writes, the new order is the author's to check, as for any moved rule.

## 7. Errors

Routing runs with Repetition 0.1: once per part, before names are rewritten.
EVENT is looked up among the `event` declarations of the block's own part (a
single-file program is one part). A declaration a `for` block writes counts
only when Repetition 0.1 expands that whole block, for every member, or would
but for its body's last statement without its `;`: that block's declarations
count as its copies read, and loading then stops at the block, with Repetition
0.1's error. A declaration that is the part's last statement counts without
its `;`, as the loader reads it. A program declares every event it reacts to,
`tick` included: `game/glowcap.cav` writes `event tick dt min 0 max 0.1;`. The
prelude holds functions only. Like Repetition 0.1, which iterates only kinds
declared in its own part, routing reads nothing else.

An error stops loading at the first one found. Blocks are read in source
order, and in each block the checks run in this order:
1. the header, and Repetition 0.1's checks of the binding, of nesting, of
   the kind and of the body's last `;`;
2. an entity of KIND in a `for` block;
3. each rule, in the order written;
4. whether the block routes any rule;
5. Repetition 0.1's check that every `$` name is bound, as each member's copy
   is written.

In a bundle an error is prefixed with the part's name, as Repetition 0.1's
errors are. HEADER below is the block's header, from `for` up to but not
including `{`, with each run of whitespace written as one space and none at
either end; a comment in it is whitespace. A routed header whose binding is
not `$NAME` is a malformed header, not Repetition 0.1's binding error.

- **A malformed header.** The header's fifth word is `routed`, but the header
  is not `for KIND as $NAME routed by P` with P a plain name.

  ```text
  expected `for KIND as $NAME routed by PARAM { ... }`, found: HEADER
  ```

- **A block that routes nothing.** No rule in the block is routed.

  ```text
  `HEADER` routes no rule: no rule in it is on an event that declares `P kind KIND`
  ```

- **P of another form.** EVENT declares P, but not as `P kind KIND`. FORM is
  the parameter as declared after its name, such as `id`, `in failed passed`,
  `min 0 max 3` or `kind repo`.

  ```text
  `on EVENT` in `HEADER`: `EVENT` declares `P FORM`, not `P kind KIND`; keep the rules on `EVENT` in a plain for block
  ```

- **The member named by another parameter.** EVENT has no parameter P, but
  has one parameter Q of kind KIND:

  ```text
  `on EVENT` in `HEADER`: `EVENT` names a KIND by `Q`, not by `P`; route the block by `Q`, or keep the rules on `EVENT` in a plain for block
  ```

  or several, Q1 to Qn in the order EVENT declares them:

  ```text
  `on EVENT` in `HEADER`: `EVENT` names a KIND by `Q1`, …, `Qn`, not by `P`; route the block by one of them, or keep the rules on `EVENT` in a plain for block
  ```

  A header with a mistyped P, such as `routed by targt` on the ledger, reaches
  this error at its first rule, and the message names the parameter it meant.

- **An event the pass cannot read.** EVENT is not declared in the part, such
  as an event a module declares and this part imports.

  ```text
  `on EVENT` in `HEADER`: no event `EVENT` is declared in this part, so the block cannot tell whether it names a KIND
  ```

- **A routed rule that mentions no binding.** The rule is routed, and mentions
  neither `$NAME` nor `$index`. `$NAME` below is the block's binding as
  written, such as `$p`.

  ```text
  `on EVENT` in `HEADER` mentions neither `$NAME` nor `$index`; its copies are all the same rule, so write it once, outside the block
  ```

- **An entity of KIND in a `for` block.** A `for` block in the part declares
  an entity of kind KIND. ENTITY is its name as written there, such as
  `$a_pr`.

  ```text
  `HEADER`: `$index` does not count entity `ENTITY` of kind KIND, declared in a for block, but `P` does; declare it at the top level of this part
  ```

For example, with `event approved target id;`, the ledger's routed block is
refused with:

```text
`on approved` in `for pr as $p routed by target`: `approved` declares `target id`, not `target kind pr`; keep the rules on `approved` in a plain for block
```

Each of these is an error rather than a quiet choice:

- A header that routes nothing says something untrue about the block. A plain
  block says the same thing truly.
- A P of another form holds something other than a member's position, so
  comparing it with `$index` would mean something else. Leaving the rule
  unrouted would contradict the header.
- An event that names a member by another parameter is about one member.
  Leaving its rules unrouted is the slip this profile exists to prevent, and
  routing them by Q would route by a parameter the header does not name.
- An event the pass cannot read may or may not name a member. Either guess
  could be wrong, with nothing to show it.
- A rule that mentions no binding runs once per event routed and once per
  member plain, and only the header would show which. Outside the block it
  means the same either way.
- An entity of KIND written in a `for` block makes a member's `$index`
  differ from its position in P, so the route would select another member's
  copy, or none. Correcting either count would change Repetition 0.1 or typed
  parameters, which this profile does not do. A plain block with a
  hand-written `target == $index` has the same miscount on such a program
  today. `caveat check` reports it where the entity comes before a
  top-level member, so that a member's copy acts for another member (C004
  in [Check 0.1](caveat-check-0.1.md)).

In the evaluation of a prototype, 20 edits that renamed an event's subject
parameter or gave it another kind were all refused by the errors for P of
another form and for a member named by another parameter. Without them, 11 of
the 20 loaded, and 2 of those passed every recorded history.

## 8. Check

`caveat check` reads a routed block as the `for` block it is. Each statement
in it is located where it is written in the block, and C001 and C002 treat it
as [Check 0.1](caveat-check-0.1.md) says they treat any `for` block. C002
reads its rules expanded, route included, and a route reads no stream.

C003 does not check a rule in a routed block. Each rule there is routed, or is
on an event C003 does not check (one that names no member of the kind, or one
written with `$NAME` or `$index`), or stops loading. C003 must not read the
expanded copies instead: there the route is `P == 2`, a comparison with a
number, which C003 does not count as a selection.

C004 does not check a rule in a routed block either. A block routed over KIND
loads only when no `for` block declares an entity of KIND (section 7), so in a
single-file program, the only kind check reads, each member's `$index` is its
number in P.

Plain blocks are checked exactly as before, including a plain block over the
same kind beside a routed one. A define written in a routed block counts for
C003 like a define in any block over the same kind. Two things follow:

- A rule for every member, moved to a plain block (section 6), is reported
  until it carries the allow comment.
- If `routed by P` is dropped from a header, the block is plain, and C003
  reports each of its rules that has no selection C003 recognizes. Two kinds
  of rule change meaning with no report. One has a top-level conjunct that
  C003 counts as routing but that is not `P == $index` alone. It compares
  P with something other than `$index`, such as `target == target.pr21` or
  `target == $p_base`, or it compares another parameter of KIND, by itself
  or in a chain of `or`, such as `base == $index` or
  `target == $index or base == $index`. Plain, a member's copy runs whenever
  that conjunct holds, whichever member P names. The other is a rule
  C003 cannot read as written, such as one with `examine $r_fog cost $index`.
  A rule that mentions no binding cannot be in a routed block (section 2), so
  dropping the clause cannot multiply one.

## 9. What it does not guarantee

Routing decides when a member's copy of a rule runs: only on an event that
names that member by P. It does not decide what the copy touches. None of the
following is prevented or reported.

- **Another member, named explicitly.** Another member's names are ordinary
  names in a routed block. `on merge when $p_green == 1 set pr21_green = 0;`
  written in the block runs in the copy of whichever pull request is merged,
  and writes pr21's state. `pr21_checks` can be read in a guard, in `using`,
  in `because` or in `permitted by`, and `target.pr21` can be compared
  anywhere.
- **A wrong P.** The pass checks that each event the block's rules are on
  either names a KIND by P or names none. It does not check that P names the
  member the rules are about. If every such event also had `base kind pr`,
  `routed by base` would be accepted where `target` was meant.
- **Entities of KIND in another part.** A kind is not namespaced
  ([caveat-0.5](caveat-0.5-draft.md)), so a module in a bundle may declare
  entities of KIND too. P counts them, and `$index` does not. A member's
  position in P can then differ from its `$index`, and the route selects
  another member's copy, or none. The pass reads only its own part, so it
  cannot see them. A hand-written `target == $index` has the same limit.
- **A selection through state.** After `on read set current = target;`, a
  rule on an event that names no member, guarded by `current == $index`, is
  not routed. Its selection is the author's, as in a plain block. A member's
  position kept in a state, such as a stacked pull request's base, is a
  number, and the pass cannot see which member it means.
- **Procedures and defines.** The route gates the rule that calls a
  procedure or reads a define. It does not limit what the procedure writes or
  what the define reads: either may name any member.
- **Rules elsewhere.** Rules outside the block, rules in a plain block and
  rules on events that name no member of the kind run as written.
- **A rule about another member's event.** A rule meant to run for one member
  when an event names another, such as every stacked pull request reconsidering
  its decision when its base gets a new commit, cannot be written in a routed
  block: the route limits every copy to the member the event names. It is
  written in a plain block, with its selection by hand. When another `kind`
  parameter of the same event names the other member, the draft
  [member symbols](caveat-member-symbols-0.1.md) profile, not implemented,
  proposes a reference that names that member's symbol at dispatch; a member
  kept in a state, such as a stacked pull request's base, stays outside it.

Routing serves the common case, where a rule is about the member its event
names. It does not make members private to their rules, and it does not make
cross-member access impossible.
