# CAVEAT Nested Repetition 0.1 (draft) — a `for` block inside a `for` block

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.2: implemented on its branch, not yet on `main` or in a release. It becomes stable when a 1.x release ships it. It adds to
[Repetition 0.1](caveat-repetition-0.1.md) and
[Routed Repetition 0.1](caveat-routed-repetition-0.1.md); it changes no
program that loads today.

A [`for` block](caveat-repetition-0.1.md) writes a rule once for every member
of one kind. Some rules are about a pair: a component and one of its decision
numbers, a host and one of its checks. Repetition 0.1 refuses a block inside a
block, so an author writes the inner loop by hand, once per member of the outer
kind:

```caveat
for slot as $k {
    state api_ack_$k = 0 min 0 max 1;
    state web_ack_$k = 0 min 0 max 1;
    state worker_ack_$k = 0 min 0 max 1;
    on acknowledge when decision == $index and component == component.api set api_ack_$k = 1;
    on acknowledge when decision == $index and component == component.web set web_ack_$k = 1;
    on acknowledge when decision == $index and component == component.worker set worker_ack_$k = 1;
    bind api.listed_$k = api_ack_$k;
    bind web.listed_$k = web_ack_$k;
    bind worker.listed_$k = worker_ack_$k;
};
```

Each rule is written three times, and a fourth component means finding and
copying each of them. This profile lets the block that already ranges over the
components hold the block over the slots:

```caveat
for component as $c routed by component {
    for slot as $k {
        state $c_ack_$k = 0 min 0 max 1;
        on acknowledge when decision == $index set $c_ack_$k = 1;
        bind $c.listed_$k = $c_ack_$k;
    };
};
```

It expands to the nine states, rules and bindings above, with the component
selected by the route, as section 3 says.

## 1. Syntax

A statement in a `for` block's body may be a `for` block, plain or
[routed](caveat-routed-repetition-0.1.md), written exactly as at the top level:

```text
for KIND as $NAME [routed by P] { … for KIND2 as $NAME2 [routed by P2] { BODY2 }; … };
```

Blocks may nest to any depth. A nested block is a statement of its enclosing
body, so it ends with `;` like every body statement, the last one too
([Repetition 0.1](caveat-repetition-0.1.md), section 2): written as the body's
last statement without its `;`, it is refused with that section's error. Every
other rule of Repetition 0.1 and Routed Repetition 0.1 about a block's header,
body, braces, kind and members holds for a nested block as for any block.

## 2. Expansion

A nested block expands inside each copy of its enclosing block. The whole
expansion is outer-member-major: the enclosing block's body is emitted for its
first member, with the nested block's copies, in its members' order, where the
nested block is written; then for its second member; and so on. That is the
order a hand-unrolled source already has, so converting one keeps which `on`
rule fires first and which `bind` wins.

Inside a nested block's body:

- `$NAME` of every enclosing block means that block's member, and the nested
  block's own `$NAME2` means its own.
- `$index` is the position of the innermost block's member. An enclosing
  block's position is not written in a nested body; where one is needed, the
  enclosing body names it once, as `define $c_position = $index;`, or the rule
  compares by name, as `component == component.$c` does.
- A `$` word is read as Repetition 0.1 reads it, across every block in scope:
  the longest binding that prefixes it wins, whichever block binds it. So with
  `$c` outside and `$cx` inside, `$cx_n` is the inner member's name followed by
  `_n`. A `$` word that no block in scope binds is an error, as before.

These are refused, each with its own message:

| Source | Message |
| --- | --- |
| a nested block that binds a name an enclosing block binds, as `for slot as $c` inside `for component as $c` | `` `for slot as $c` binds $c, which an enclosing for block binds too; give the nested block another name `` |
| a member of a nested block's kind with `$` in its name | `` `HEADER`: entity `NAME` has a `$` in its name, which the enclosing for block would read as a binding; rename it, or do not nest the block `` |

A nested block may range over the same kind as an enclosing one, giving one
copy per ordered pair of members, the same member twice included.

Expansion is still a source-to-source pre-pass. Its output is ordinary
statements, every one of which the author could have written by hand.

## 3. Routing

A routed block routes the rules of the blocks nested in it too. Each block
reads the rules in its body after the blocks nested in it have expanded, and
decides which are routed by Routed Repetition 0.1's rules, against its own
kind and parameter. A rule that two routed blocks route gets each route,
outermost first:

| Written | Expanded, for the first component and the second slot |
| --- | --- |
| `for component as $c routed by component { for slot as $k routed by slot { on pick when G EFFECT; }; };` | `on pick when component == 1 and slot == 2 and G EFFECT` |

Section 3 of Routed Repetition 0.1 is unchanged: each route goes at the front
of the guard as it stands when that block expands, so the enclosing block's
route ends up first, and a guard with an `or` is parenthesized by the
innermost route that reaches it.

A routed rule must mention its own block's `$NAME` or `$index`
([Routed Repetition 0.1](caveat-routed-repetition-0.1.md), section 2). A
binding of an enclosing block does not count: it is the same in each of the
block's own copies, so it does not tell them apart. A nested rule that a
routed enclosing block routes must mention that block's `$NAME`. One more rule
is refused:

| Source | Message |
| --- | --- |
| in a routed block, a rule on an event written only with an enclosing block's binding, as `on pick_$c` inside `for slot as $k routed by slot` | `` `on pick_$c` in `HEADER`: the event is named with an enclosing for block's binding only, so this block cannot tell whether it names a KIND; write the rule in the enclosing block `` |

An event written with the block's own binding is the member's own and is not
routed, as before.

## 4. Check

[`caveat check`](caveat-check-0.1.md) reads a nested block as a block of its
own once per member of each enclosing block, and the enclosing block with the
nested block's copies in its body. So C003 reports a rule in a nested block
that does not select the nested block's member, or that does not select the
enclosing plain block's member, and C004 reads each block's `$index` as its
own. A routed block is still skipped by C003, at any depth.

## 5. What it does not do

- It adds no runtime semantics, as Repetition 0.1 adds none. A nested block is
  not a loop over values, and the members of every kind are fixed when the
  source is written.
- It gives no name to an enclosing block's position (section 2).
- A block still iterates a kind declared in its own part.
- A diagnostic after a nested block reports the line in the expanded text, as
  for any block.

## Compatibility with other additions

- [Procedures](caveat-procedure-symbols-0.1.md): a `proc` in a nested body is
  declared once per pair, as in any block body.
- Foreclosure, a 1.2 draft on its own branch: a series a nested block
  declares may be irreversible, as anywhere.

## Corpus sweep

`node experiments/nested-repetition-sweep/sweep.mjs` reads every tracked Caveat
program and writes
[`RESULTS.json`](../experiments/nested-repetition-sweep/RESULTS.json). It reads
source text only and runs no program. At this profile's revision:

- 196 programs write 58 `for` blocks, in 29 programs. None nests one, so no
  program reaches the new form;
- no block's body writes a statement once per member of another kind, and no
  statement anywhere is written once per pair of members of two kinds: the
  corpus has no hand-written nested block to convert. The sweep finds such a
  body when one is added: a block over decision numbers that writes each rule
  once per component, as in the example above, is reported with each of its
  statement forms;
- 50 groups of top-level statements are written once per member of one kind,
  95 statement forms in all, in programs that predate Repetition 0.1 or keep
  their copies by hand. A single block covers those; they are listed for
  comparison and are not this profile's.

No accepted program changes.

## Implementation

- `runtime/src/repeat.rs` expands a nested block inside its enclosing block's
  body before the enclosing block reads its routed rules and makes its copies,
  keeping the enclosing bindings as written for it. It refuses the forms in
  sections 2 and 3. A block's events are read with its nested blocks expanded,
  so a routed block elsewhere in the part reads the events they declare.
- `runtime/src/reactive_check.rs` reads nested blocks as section 4 says.
- `runtime/tests/nested_repetition.rs` covers the expansion order, three
  levels, a binding an enclosing one prefixes, the refusals, routing through a
  nested block, two routes, a rule that does not tell its own copies apart, an
  event named with an enclosing binding, events declared in a nested block,
  and C003 on a nested rule. The tests in `runtime/tests/repetition.rs` that
  expected a nested block to be refused now expect it to expand, or to be
  refused for a body statement without its `;`.
- The language reference and the authoring guide describe the form.

## Changes

- 2026-10-19: specified and implemented for 1.2.
