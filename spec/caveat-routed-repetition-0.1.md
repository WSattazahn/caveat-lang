# CAVEAT Routed Repetition 0.1

**Status:** implemented. The pass is in `runtime/src/repeat.rs`, and C003
skips routed blocks in `runtime/src/reactive_check.rs`. Sections 1 to 8 and
10 are tested in `runtime/tests/routed_repetition.rs` and
`runtime/tests/check.rs`. The converted agent ledger passes the ledger's 11
scenarios (`kit/test/commands.test.mjs`), and Before the Rain
(`game/before_the_rain.cav`) passes its 12 (`npm run test:before-the-rain`).
Section 9 lists what routing does not prevent. Of those limits, only
entities of KIND in another part has a test. A runtime older than this
profile refuses a `routed by` header as a malformed `for` block, and one
older than section 10 refuses `$Q` as not bound.

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

A routed rule can still name a second member that its event names, by that
parameter's own name: `$shown` for `shown kind exhibit` (section 10). The
header routes by one parameter, and the text shows which parameter names the
second member.

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

`$Q` (section 10) names another member, so it is not a mention.

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
only when Repetition 0.1 expands that block's statements other than its `on`
rules, for every member, or would but for its body's last statement without
its `;`: that block's declarations count as its copies read, and loading then
stops at the block, with Repetition 0.1's error. A rule declares no event,
and one that names a member by `$Q` is bound only once its event is read
(section 10), so the rules are left out. A declaration that is the part's last
statement counts without its `;`, as the loader reads it. A program declares
every event it reacts to, `tick` included: `game/glowcap.cav` writes
`event tick dt min 0 max 0.1;`. The prelude holds functions only. Like
Repetition 0.1, which iterates only kinds declared in its own part, routing
reads nothing else.

An error stops loading at the first one found. Blocks are read in source
order, and in each block the checks run in this order:
1. the header, and Repetition 0.1's checks of the binding, of nesting, of
   the kind and of the body's last `;`;
2. an entity of KIND in a `for` block;
3. each rule, in the order written;
4. whether the block routes any rule;
5. section 10, rule by rule in the order written: in each rule, each `$`
   word in the order written, then the rule as a whole;
6. the work of each event that a rule of section 10 is on, in the order of
   those rules;
7. Repetition 0.1's check that every `$` name is bound, as each member's copy
   is written.

A plain block has checks 1, 5 and 7, and check 5 only reads a `$` word in an
`on` rule that neither `$NAME` nor `$index` begins. Such a word is refused
either way, so a plain block that loads is read as it always was.

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

Section 10 adds the errors below, in the order checks 5 and 6 find them. In
each, EVENT is written without `$` and is declared in the part. $WORD is the
whole `$` word as written, such as `$shown_points`. Q is a parameter of
EVENT, `$Q` is Q with a `$`, and K is Q's kind.

- **A word either could name.** In a routed rule, `$NAME` or `$index` begins
  $WORD, and so does a `kind` parameter Q of EVENT that is longer, or as long
  and not P.

  ```text
  `$WORD` in `on EVENT` in `HEADER` could be `$NAME` or the K `EVENT` names by `Q`; rename `$NAME` so that only one of them begins it, or write the rule in a block routed by `Q`
  ```

  Where `$index` begins the word, the message names `$index` and does not
  suggest renaming it. Where Q is P, it does not suggest routing by Q. Where
  both hold, it suggests writing `$NAME` for the K that P names.

- **P named by its parameter.** In a routed rule, neither `$NAME` nor
  `$index` begins $WORD, and the longest `kind` parameter of EVENT that
  begins it is P.

  ```text
  `$WORD` in `on EVENT` in `HEADER`: the block is routed by `P`, so the KIND it names is `$NAME`; write `$NAME` for `$P`
  ```

- **A parameter of another form.** In a routed rule, no binding and no
  `kind` parameter of EVENT begins $WORD, and another parameter Q does, the
  longest if several do. FORM is as for P of another form.

  ```text
  `$WORD` in `on EVENT` in `HEADER`: `EVENT` declares `Q FORM`, and a `$` names a member only by a `kind` parameter
  ```

- **A parameter outside a routed rule.** In an `on` rule of a plain block,
  or in a rule of a routed block that is not routed, neither `$NAME` nor
  `$index` begins $WORD, and a `kind` parameter of EVENT does. Anywhere
  else, such as in a declaration or in a rule on an event written with `$`,
  the word is refused as not bound, as before.

  ```text
  `$WORD` in `on EVENT` in `HEADER`: a `$` names the member a parameter names only in a routed rule (spec/caveat-routed-repetition-0.1.md section 10), and this rule is not routed
  ```

- **A rule that names members by two parameters.** Q1 and Q2 are the
  parameters in the order the rule first names a member by them:

  ```text
  `on EVENT` in `HEADER` names members by `$Q1` and `$Q2`; a rule names a member by one parameter at most, so write it in a block routed by one of them
  ```

  With more, the message names each, as `$Q1`, `$Q2` and `$Q3`.

- **A rule that names members only by a parameter.** The rule names a member
  by `$Q`, and neither `$NAME` nor `$index` begins any of its words.

  ```text
  `on EVENT` in `HEADER` names members only by `$Q`, and neither `$NAME` nor `$index`; each KIND's copy of it would be the same rule, so write it in a block routed by `Q`
  ```

- **A kind with no member in the part.** No top-level `entity` statement of
  the part declares an entity of kind K.

  ```text
  `$Q` in `on EVENT` in `HEADER`: no entity is declared `kind K` at the top level of this part, so `$Q` names no member
  ```

- **An entity of K in a `for` block.** A `for` block in the part declares an
  entity of kind K. ENTITY is its name as written there.

  ```text
  `$Q` in `on EVENT` in `HEADER`: `Q` counts entity `ENTITY` of kind K, declared in a for block, and `$Q` does not; declare it at the top level of this part
  ```

- **More work than one event can do.** N is the number of rules the block
  writes on EVENT, counted as section 10 counts them, and it is more than
  4096.

  ```text
  `HEADER` writes N rules on `EVENT`, and one event does at most 4096 steps of work; each copy spends a step whether its route holds or not
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
- A word that either could name named the block's member before, and a
  parameter's name begins it now too. Either could be meant, and nothing
  would show which.
- In the only copy that runs, the member P names is the block's own, so
  `$P` would write W×|KIND| copies to say what `$NAME` says once.
- A parameter of another form holds a number or a choice, not a member's
  position.
- A rule that is not routed would run once for each block member, for the
  member Q names. No game has needed that, and C003 would then have to read
  `$Q`.
- A rule that names members by two parameters would multiply its copies
  again, and no game has needed it.
- A rule that names members only by a parameter is the same in every block
  member's copy, as a routed rule that mentions no binding is.
- A kind with no member in the part would give the rule no copy, and it would
  vanish without a report.
- An entity of K in a `for` block makes Q count a member that `$Q` does not,
  the miscount refused above for P.
- An event with more work than it can do would be refused each time it is
  sent, as `limit/work_limit`. Refused at load, the block says so once.

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

A rule that names a member by its parameter is routed, so C003 and C004 do
not check it. C002 reads its copies. Check reads `$Q` as a name the source
does not use, as it reads `$NAME` and `$index`.

Plain blocks are checked exactly as before, including a plain block over the
same kind beside a routed one. A define written in a routed block counts for
C003 like a define in any block over the same kind. Two things follow:

- A rule for every member, moved to a plain block (section 6), is reported
  until it carries the allow comment.
- If `routed by P` is dropped from a header, the block is plain, and C003
  reports each of its rules that has no selection C003 recognizes. Two kinds
  of rule change meaning with no report. One has a top-level conjunct that
  C003 counts as a selection but that compares P with something other than
  `$index`, such as `target == target.pr21` or `target == $p_base`. Plain,
  every member's copy runs when that comparison holds. The other is a rule
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
- **A wrong Q.** A rule that names a member by `$Q` (section 10) names it by
  the parameter written. The text shows which parameter that is, and nothing
  checks that it is the one meant.
- **Members of Q's kind the rule did not mean.** `$Q` covers every member of
  Q's kind. Where a rule means only some of them, such as the traces among
  Before the Rain's exhibits, a rule that refuses the others must come first.
- **Entities of KIND in another part.** A kind is not namespaced
  ([caveat-0.5](caveat-0.5-draft.md)), so a module in a bundle may declare
  entities of KIND too. P counts them, and `$index` does not. A member's
  position in P can then differ from its `$index`, and the route selects
  another member's copy, or none. The pass reads only its own part, so it
  cannot see them. A hand-written `target == $index` has the same limit. Q
  counts the entities of its kind in other parts as P does, and the copies of
  a rule that names a member by `$Q` have the same limit.
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
  written in a plain block, with its selection by hand.

Routing serves the common case, where a rule is about the member its event
names. It does not make members private to their rules, and it does not make
cross-member access impossible.

## 10. A member named by its parameter

[Before the Rain](../game/before_the_rain.cav), a game built on this profile,
has an event that names two exhibits:

```caveat
event confront about kind exhibit, shown kind exhibit, recants in no yes;
```

`about` is the witness whose account is confronted, and `shown` is the trace
shown to them. A recant takes the account back because of that trace, and a
trace that says what the account says cannot bring one. Routing by `about`
covers the witness. The trace has no name in the source, and
`withdraw E because R` needs R written there. So the game named each trace by
hand, and each witness to compare with:

```caveat
// What was shown must be an examined trace that says otherwise.
for exhibit as $s {
    on confront when shown == $index and $index <= 3 reject "Show them a trace.";
    on confront when shown == $index and $s_points == 0 reject "That trace shows nothing yet.";
    on confront when shown == $index and about == 1 and x_farmer_points == $s_points reject "That trace agrees with them.";
    on confront when shown == $index and about == 2 and x_farmhand_points == $s_points reject "That trace agrees with them.";
    on confront when shown == $index and about == 3 and x_neighbour_points == $s_points reject "That trace agrees with them.";
};

// A recant takes the account back because of the trace that was shown.
for exhibit as $w routed by about {
    on confront when recants == recants.yes and shown == 4 withdraw ev_$w because ev_x_prints;
    on confront when recants == recants.yes and shown == 5 withdraw ev_$w because ev_x_can;
    on confront when recants == recants.yes and shown == 6 withdraw ev_$w because ev_x_lamp;
    on confront when recants == recants.yes and shown == 7 withdraw ev_$w because ev_x_hay;
};
```

Seven of these rules name a witness or a trace by hand, one for each of 3
witnesses and 4 traces, and they grow with both. The game's author wrote the
form they wanted: `because ev_$shown`. With this section, the seven rules are
two, and their number does not grow:

```caveat
// What was shown must be an examined trace.
for exhibit as $s routed by shown {
    on confront when $index <= 3 reject "Show them a trace.";
    on confront when $s_points == 0 reject "That trace shows nothing yet.";
};

// A recant takes the account back because of the trace that was shown, and
// only a trace that says otherwise can bring one.
for exhibit as $w routed by about {
    on confront when $w_points == $shown_points reject "That trace agrees with them.";
    on confront when recants == recants.yes withdraw ev_$w because ev_$shown;
};
```

In a routed rule (section 2), `$Q` names the member that the rule's event
names by its parameter Q, where the event declares `Q kind K` and Q is not P.
K may be KIND, as in Before the Rain, or another kind. `$Q` composes as
`$NAME` does: `ev_$shown` becomes `ev_x_can`, `$shown_points` becomes
`x_can_points`, `$shown.x` becomes `x_can.x`, and `"$shown"` becomes
`"x_can"`. `$Q` is bound nowhere else: not in a plain block, not in a rule
that is not routed, and not in a declaration, a `define`, a `bind`, a `cue`,
a `proc` body or a rule on an event written with `$NAME` or `$index`. There a
`$` word that no binding of the block's own begins is refused, as before
(section 7). A rule's text runs from its `on` to its `;`, comments inside it
included. A comment on the line above a rule, or after its `;`, is not part of
the rule, and a `$Q` there is refused as not bound. A routed rule may pass the
members' symbols to a procedure, as in `call recant(ev_$w, ev_$shown)`, at the
cost described below.

The pass reads each `$` word of a routed rule in order, comments blanked and
quoted text included. The longest of `$NAME` and `$index` that begins the
word binds it, as before. If neither begins it, the longest `kind` parameter
of the event that begins it binds it as `$Q`. A word that `$NAME` or `$index`
begins stays theirs, unless a `kind` parameter of the event begins it too and
is longer, or as long and not P. Then either could be meant, and the rule is
refused (section 7): `$shown_n` in a block bound `$s` and routed by `about` is
one. A binding named exactly P, as in `for exhibit as $about routed by about`,
keeps its word, since both name the same member. A rule names members by one
parameter at most. It must still mention `$NAME` or `$index`, and `$Q` is not
a mention.

In each block member's copy, a rule that uses `$Q` is written once for each
member of K, in declaration order, as if written:

| Written in the block | Expanded as if written |
| --- | --- |
| `on E EFFECT` | `on E when P == I and Q == N EFFECT` |
| `on E when G EFFECT` | `on E when P == I and Q == N and G EFFECT` |
| the same, with an `or` in G outside parentheses | `on E when P == I and Q == N and (G) EFFECT` |

I is the block member's position, as in section 3. N is the member's position
among the part's top-level `entity` statements of K, the count `$index` uses.
The route is placed as section 3 places `P == I`, in the copy with `$Q`
already replaced. In the copy for the farmhand's account and the can, the
second rule above is:

```caveat
on confront when about == 2 and shown == 5 and recants == recants.yes withdraw ev_x_farmhand because ev_x_can;
```

That is the rule the game wrote by hand, with its selections first.

The copies go one after another where the rule is, separated by one space.
A rule on one line adds no line, so its copies share the line it is written
on. A diagnostic gives the line in the expanded text, as Repetition 0.1
section 4 says. The block still expands member-major: the whole body for the
block's first member, with every copy of such a rule where the rule is, then
the whole body for the next.

The copies have no order that can show. On any event, P names one member and
Q names one, so at most one copy of a written rule passes its route. Every
other copy stops at a route, since `and` stops at its first false operand. A
route compares only event parameters, and the runtime builds those as plain
values, with no provenance, so a route that does not hold has none. A skipped
effect is retained only under a guard with provenance, so a copy that stops
adds nothing to lineage or grounds. The copy that runs has the grounds, lineage,
withdrawal record and journal entry of the rule written by hand for that pair.
This rests on event parameters carrying no provenance. If a later profile
gives them provenance, the test
`a_copy_that_its_route_stops_adds_nothing_to_lineage` in
`runtime/tests/routed_repetition.rs` fails, and this section must be
revisited.

This is not a nested block, and section 1 still holds. The header routes by
one parameter. Section 1's objection to a second was that its choice would
not show in the text, and here the second member is named in the text, by its
own parameter. The binding is the parameter's name, so two parameters of one
kind, such as `about` and `shown`, cannot swap roles without it showing. A
`for` block inside another is still refused.

A rule that uses `$Q` is written W×|K| times, for a block of W members. Each
copy spends a step of the event's work whether its route holds or not, as
every rule on the event does. In Before the Rain, the rules on `confront` went
from 86 to 135, and an accepted `confront` took 10 to 14 µs longer, 7 to 10
percent, in three runs of the WebAssembly runtime under Node. A copy of a rule
whose effect is a `call` costs more when its route stops it: it still walks
every step of the procedure ([procedures](caveat-reactive-0.7.md)), which the
count below does not include. Such a program can load and then have every
event refused as `limit/work_limit`. Evidence and other symbol arguments also
specialize the procedure once for each pair of members, at most 1024.

The pass counts that work when the program loads. For each event that a rule
using `$Q` is on, it counts the block's rules on the event: W×|K| for each
rule that uses `$Q`, and W for each other rule. A count over 4096, the most
steps one event can do, is refused (section 7). So 64 members of one kind with
one such rule on an event write 4096 rules, which load and run, and 65 members
write 4225, which are refused. With one more rule of the block on the event,
63 members write 4032 and load, and 64 write 4160 and are refused. The count
is a floor: rules outside the block, and the steps of a procedure that a copy
calls, count only when the event runs, and the runtime then refuses the event
as `limit/work_limit`.

When K is KIND, some copies pair a member with itself, and `$shown` covers
every exhibit, the three accounts too. The rules that refuse what the rule
does not mean must come first. A `withdraw` or a `qualify` that names
evidence never observed fails the event, so Before the Rain keeps the block
routed by `shown` before the block routed by `about`. Merged into the
exhibit block above both, the withdrawal would run before those refusals,
and a trace that was never examined would fail the event.

Three ways to write these rules without this section were measured, and each
costs more:

- A block routed the other way, by `shown`, names the witness by hand in
  each rule instead: min(W, T) rules, each naming a member by hand.
- A relay state, set to the witness's points in the block routed by `about`
  and compared in the block routed by `shown`, costs W + 2 rules. A state
  cannot name evidence, so each witness's withdrawal is still written by
  hand, and the state is carried in every save.
- A procedure for each witness, called from the block routed by `shown`,
  costs W + 2 rules too, and it widens lineage: the trace shown enters the
  witness's predicate qualifications, binding explanations and qualified
  values even without a recant. In 3,000 random histories, the snapshots
  after 506, 1,328 and 947 of 15,530 accepted events differed in those.

This section does not name a member:

- in a plain block, or in a rule that is not routed;
- in a declaration, a `define`, a `bind` or a `proc`;
- by two parameters in one rule;
- by an `in`, numeric or `id` parameter;
- held in a state;
- of a kind declared in another part.

Nor does it write a rule about either of two members, such as Before the
Rain's hold on the exhibits named by `first` and `second`. Such a rule
selects its member by hand, in a plain block, and how `caveat check` reads
that selection is a separate question.

## Changes

- 2026-09-29: a routed rule names the member its event names by another
  `kind` parameter Q as `$Q` (section 10). Before the Rain wrote 7 rules for
  3 witnesses and 4 traces, and their number grew with both. `$shown` writes
  them as 2. `$Q` was refused before as not bound. Every program that loaded
  before still loads, and expands byte for byte as before, except one now
  refused: a routed rule with a word that `$NAME` or `$index` begins and a
  `kind` parameter of its event begins too, longer, or as long and not P,
  such as `$shown_n` in a block bound `$s` and routed by `about`. It read as
  the member's name followed by `hown_n`. No program in the repository has
  one. Of the programs that did not load, a `$` word in a routed rule that a
  `kind` parameter of its event begins is refused with a message that says
  why, where it was refused as not bound. Elsewhere, and for other parameters,
  the message is the one before. Event declarations are read without the block's
  rules, so a program that does not load may report another error first.
  Nothing changes in the runtime, snapshots or saves. In 3,000 random
  histories of 30 events, Before the Rain rewritten this way left the same
  snapshot, source identity aside, after every event as the game written by
  hand. Every refusal was the same apart from rule numbers in messages.
