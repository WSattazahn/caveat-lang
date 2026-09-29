# CAVEAT Repetition 0.1

`game/light_the_way.cav` is 888 lines. 182 of them are the same fourteen
statements written out once per reef, byte-identical apart from the name:

```caveat
on tick when phase == 1 and paused == 0 and light_on == 1 and distance(aim_x, aim_z, reef_one.x, reef_one.z) <= beam_radius and not observed(reef_one_reading) reveal reef_one_reading opposes clear_course;
on tick when phase == 1 and paused == 0 and light_on == 1 and distance(aim_x, aim_z, reef_two.x, reef_two.z) <= beam_radius and not observed(reef_two_reading) reveal reef_two_reading opposes clear_course;
```

That is not a style problem. An author who adds a seventh reef has to find and
copy fourteen statements correctly, and a change to the beam rule has to be
made six times. The language has no way to say "for each reef", so it makes the
author be the loop.

This profile adds one: a block that is written once and expanded over the
members of a declared entity kind.

## 1. The set already exists

```caveat
entity reef_one kind reef at harbor;
entity reef_two kind reef at harbor;
```

`kind reef` is already a declared set with a declaration order. Repetition
iterates that set; it introduces no new way to group things.

Its members are the part's top-level `entity` statements of the kind. Each
statement is read as the loader reads it: a comment inside it is ignored, and
a last statement without its `;` is read, after empty statements too. A `#`
or `//` inside quoted text is not a comment. A `{` or `}` groups only a
procedure's body or a `for` block's, so in any other statement it does not
keep the `;` from ending it. A block is a statement whose first word is
`for`, so the `{` in `for{ supports c;`, about an evidence named `for{`, is
text.

An entity declared in a block's body is no member of its kind, for any
block: it gets no copy, and `$index` does not count it. A `kind` event
parameter counts it, where its block is expanded. When it comes before a
top-level member of its kind, the parameter numbers that member, and each
member after it, higher than its `$index`, so a copy that selects its member
with `target == $index` acts for another member, and
[`caveat check`](caveat-check-0.1.md) reports it (C004). A copy that selects
its member by name, with `target == target.$p`, acts for its own member
however the entities are counted, so it does not run on an event that names
the entity, which has no copy of its own. A copy that selects its member by
`$index`, or selects none, can run on such an event. Rules about the entity
are written where it is declared, such as
`on read when target == target.north` in the block that declares `north`. A
[routed](caveat-routed-repetition-0.1.md) block refuses an entity of its kind
in a `for` block.

## 2. The block

```caveat
for reef as $r {
    evidence $r_reading from $r;
    state $r_distance = 100;
    cue $r_ring ring $r "#ffe3a0" 0.75;
    on tick when phase == 1 and paused == 0 and contact == $index set hit_x = $r.x;
    bind $r.reveal.opacity = 0;
    bind $r.reveal.opacity = 0.25 when observed($r_reading);
};
```

`for KIND as $NAME { ... };` expands its body once per member of `KIND`, in
declaration order, and replaces itself with the result. Like any last
statement, a last block may leave out its `;`, though its body's last
statement may not (below). Nothing but comments may come between the body's
`}` and the block's end. A brace in the body's unquoted text pairs only
within its statement. The body ends at its first `}` outside quoted text,
comments and procedure bodies that does not close a `{` earlier in its own
statement. So a pair in unquoted provenance, such as `from see{appendix}b;`,
stays in its statement, while in `from a}b;` and in `from }{;` the `}` ends
the body, and what follows it is text after the body. A body statement that
leaves a `{` open, as `from x{;` does, is refused: a `}` in a later statement
does not close it. Either error gives the line and column of the brace.
Quoted, as in `from "x{";` or `from "}{";`, the text is the statement's own.
A quoted name keeps its quotes, so a name in a body, as in `claim $p_c}{;`,
cannot hold such a brace.

So a brace that is text in a body must be in quoted text, as in
`from "see appendix}";`, unless it pairs with a brace in its own statement.
Unquoted, any other brace in a body refuses the block. A pair split across
two statements, as in `evidence $p_a from x{; evidence $p_b from y};`, is
refused at the `{` after `x`, which its statement leaves open. A `}` that
closes no `{` of its statement ends the body where it is: what follows it up
to the block's end, other than comments, is text after the body, as `b` is in
`from a}b;`, and where nothing does, as in `from y};`, its statement is the
body's last and has no `;` (below).

Every statement in a body ends with `;`, the last one too, whatever the
number of members. The body is copied as written and no `;` is added, so a
last statement without one would run into what follows each copy: the next
member's copy, or, after the last member's, the text after the block. A body
with no statement, one that is empty or holds only comments and empty
statements, needs none. A `}` right before a statement's `;`, as in
`from y};`, ends the body there, so that statement is the body's last and
has no `;`. Such a body is refused before any copy is made, in a routed block
too:

```text
line L, column C: the last statement in the body of `HEADER`, `STATEMENT`, has no `;` before the `}` that ends the body; each member's copy of it would run into what follows the copy, so end it with `;`
```

L and C are where the statement is written, HEADER is the block's header as
in [routed Repetition 0.1](caveat-routed-repetition-0.1.md) section 7, and
STATEMENT is the statement's words, its comments read as whitespace and each
run of whitespace written as one space. The rule is the block's own: it holds
with one member, and at the end of the program, where the loader would read a
last statement without its `;`.

Two bindings are available inside the body:

- `$NAME` — the member's name.
- `$index` — its one-based position in the kind's declaration order.

In a [routed](caveat-routed-repetition-0.1.md) rule, `$Q` also names the
member that the rule's event names by another `kind` parameter Q (routed
Repetition 0.1 section 10). A word that `$NAME` or `$index` begins stays
theirs, and a routed rule in which such a parameter also begins one, longer,
or as long and not the parameter the block is routed by, is refused.

`$NAME` substitutes as text wherever it appears, so it composes into longer
identifiers (`$r_reading` becomes `reef_one_reading`), into dotted names
(`$r.x`, `$r.reveal.opacity`), and inside quoted text (`from "$r"`). A `$`
followed by a name that is not bound is an error rather than a literal, because
a typo that silently survived would produce a symbol nobody declared.

A [routed](caveat-routed-repetition-0.1.md) block selects the member an event names.

## 3. Expansion is all it is

Repetition is a source-to-source pre-pass, like linking. It runs per bundle
part, before names are rewritten, over the entity kinds declared in that same
part. The output is ordinary statements, so:

- the evaluator, the transaction model and the epistemic graph are untouched;
- every generated statement is one the author could have written by hand;
- an existing program expands to itself, byte for byte.

**Expansion is member-major.** The whole body is emitted for the first member,
then for the second, and so on. This matters because order is meaning in this
language: every matching `on` rule fires in source order and the last matching
`bind` wins. Member-major is the order a hand-unrolled block already has, so
converting one does not change behaviour.

## 4. What it does not do

A `for` block iterates a kind declared in its own part. Iterating a kind an
imported module declared is not in this profile: it would make one part's
generated names depend on another part's declarations, and
[caveat-0.5](caveat-0.5-draft.md) keeps a part's names its own.

`for` blocks do not nest. A nested one is rejected rather than given an
ordering nobody asked about.

A routed rule that names a member by its parameter is written once for each
member of that parameter's kind, where the rule is. That is not a nested
block: at most one of those copies runs on an event, so their order cannot
show. A `for` block inside another is still refused.

Expansion changes how many lines a part has, so a diagnostic pointing into or
after an expanded block reports the line in the expanded text rather than the
line the author wrote. Which *part* it is in remains clear: linking parses each
part on its own, so a parse error names the part and its own line, and the
`origin` markers of
[caveat-authorship-0.1](caveat-authorship-0.1.md) delimit the parts in the
linked text.

Repetition adds no runtime semantics and is not a loop. There is no iteration
over values, no accumulator, and no way to write a block whose member count
depends on execution. The set is fixed when the source is written, which is
what keeps the expansion checkable and the generated program ordinary.

## Changes

- 2026-09-27: a kind's members are every top-level `entity` statement of it
  that the loader declares, by the name it declares
  ([#47](https://github.com/WSattazahn/caveat-lang/issues/47)). Before this,
  Repetition read a statement with its comments left in. It skipped one with
  a comment inside it and did not read a last one without its `;`, so
  `$index` counted fewer members than a `kind` event parameter, and a block
  that selected its member with `target == $index` updated another member,
  or none. It read a comment touching the name as part of the name: with
  `entity north#first`, a copy that used `$NAME` outside quoted text did not
  load, and `"$NAME"` became `"north#first"`. This deliberately changes the
  expansion of those programs: the skipped member gets its copy, the members
  after it their right `$index`, and every copy its member's name. Every
  other program expands as before.
- 2026-09-27: a statement ends at its `;` where the loader ends it.
  Repetition counted `{` and `}` in every statement, and the loader counts
  them only in a `proc` statement, so a brace elsewhere, such as in unquoted
  evidence provenance, made Repetition read the rest of the part as one
  unterminated statement. Below a block, the entities after the brace got no
  copy while a `kind` parameter counted them, so an event that named one
  changed nothing, silently. Above a block, or in its body, the block was
  not expanded and the program did not load. Above and below a block, such
  programs now expand with a copy for every member. That includes a brace
  right after a first word `for`, as in `for{ supports c;` about an
  evidence named `for{`: that statement is no block. In a body, a brace in
  unquoted text pairs only within its statement (section 2): a `}` that
  closes a `{` earlier in its statement stays in it, as before, and any
  other `}` ends the body. Repetition counted a body's braces across its
  statements instead, and took the block statement's last `}` for the
  body's end. So these bodies are now refused, for any number of members,
  in a routed block too, and the saves of such programs cannot be restored:
  - A `{` that its statement leaves open, which a `}` in a later statement
    closed, as in `evidence $p_a from x{; evidence $p_b from y};`.
    Repetition paired the two, and over north and south the program
    declared north_a, north_b, south_a and south_b, with provenance `x{`
    and `y}`, as written. The error names the `{`: "line L, column C: for
    block `for plot as $p`: `evidence $p_a from x{` has a `{` that its
    statement does not close; a brace in unquoted text pairs only within
    its statement, so quote the text, or take the brace out of a name".
  - A `}` that closes no `{` of its statement, followed by a `{` in the
    same statement, as in `from }{;`, `from m}n{o;` or `claim $p_c}{;`.
    Repetition paired that `}` with the body's `{`, and that `{` with the
    body's `}`: over north and south, `evidence $p_a from }{;` declared
    north_a and south_a with provenance `}{`, as written. The `}` now ends
    the body, and the rest of the block is text after it: "line L, column
    C: for block has text after its body: {; }; the `}` in
    `evidence $p_a from }{` closes no `{` of its statement, so it ends the
    body: quote a brace that is text, take it out of a name, or end the
    block with `};`", with the line and column of that `}`.
  - Such a `}` in the body's last statement without its `;`, before the
    body's own `}`, as in `from a}b }`, refused the same way. With one
    member, or with a body that begins with an empty statement, which ends
    each copy's last statement, the program loaded as written. With two or
    more members otherwise, each copy's last statement read on into the
    next copy, and the program declared fewer names than written, or did
    not load.

  Quoted, as `from "x{"`, `from "}{"` or `from "a}b"`, each of these loads
  as written. Such a `}` followed by more of its statement and a `;`, as in
  `from a}b;`, is refused the same way; such a program did not load, or
  loaded other than as written. A `{` that nothing closes, as with
  `see{appendix;`, `for{ supports $p_c;`, `claim $p_c{;` or `entity $p_hut{`
  in a body, is refused with the first error, and a body whose last
  statement, without its `;`, has a `{` that the body's `}` closes, as in
  `from see{appendix }`, is refused as a block that is not closed. Such
  programs did not load before either. The brace in a name such as
  `entity north{` hid that entity too: the loader declared it, and a `kind`
  parameter counted it. It is now a member, with its copy and its `$index`,
  as is every name the loader declares, such as `no-rth` or `1north`: the
  loader does not check a name against the identifier syntax. The name
  reaches its copy as written. Where the copy needs a reactive identifier,
  as in `state $p_n`, the program does not load, and the error gives the
  copy's line in the expanded text: "invalid reactive identifier north{_n".
  Below a block, such a program loaded before, without copies for north{ and
  the members after it; it now does not load, and its saves cannot be
  restored. In quoted text, in provenance, or in a claim's or evidence's
  name, the copy loads as the same text written by hand does. An entity
  declared in a block's body is no member, whatever its name. A routed block
  refuses one of its kind, and a plain block has the miscount that section 7
  of [routed Repetition 0.1](caveat-routed-repetition-0.1.md) describes: it
  gives the entity no copy while a `kind` parameter counts it, so where the
  body selects its member with `target == $index`, an event that names the
  entity, or a member after it, updates another member's copy, or none, and
  nothing reports it. `entity north{` there leaves its `{` open and is
  refused; the program did not load before.
- 2026-09-27: a comment in a block's header is whitespace, as it is
  everywhere outside quoted text ([text 0.1](caveat-text-0.1.md)).
  Repetition read a header's words and braces with its comments left in. It
  refused `for plot # each plot` with `as $p {` on the next line as a
  malformed header, and took a `{` in a comment in the header, or a `}` in
  one after the body, for the block's own. Such blocks now expand, and a
  malformed header is shown without its comments.
- 2026-09-27: a last block without its `;` is expanded, as the loader reads
  a last statement without one ([text 0.1](caveat-text-0.1.md)). Repetition
  did not read it, and the program did not load: "cannot parse statement:
  for plot as $p {". A block that is the last statement of another block's
  body is refused as nested, where its `$` names were refused as unbound.
  Text between a block's body and its end, such as a statement after a block
  whose `;` was left out, is refused, with the line and column of the body's
  `}`: "line L, column C: for block has text after its body: ...; end the
  block with `};`". Repetition dropped it, silently when a `;` followed it.
  A last statement after an empty statement, as in `};;` followed by
  `entity south kind plot at field`, was not read either: its entity got no
  copy while a `kind` parameter counted it, so an event that named it
  changed nothing, silently, and a last block there was not expanded. Both
  are now read.
- 2026-09-27: section 1 says what an entity declared in a block's body is:
  no member, and counted by a `kind` parameter. `caveat check` reports a
  plain block whose `target == $index` acts for another member because of it
  (C004 in [Check 0.1](caveat-check-0.1.md)). Nothing about expansion or
  loading changes.
- 2026-09-28: a body's last statement must end with `;` (section 2), for any
  number of members, in a routed block and in a module too. No `;` is added,
  and a body with no statement needs none. The body was copied as written,
  so such a statement ran into what followed each copy. These programs are
  now refused, and their saves cannot be restored. Below, "before" is
  Repetition as it was before the 2026-09-27 entries, "then" is with them,
  and the members are entities north and south of kind plot unless one is
  named:
  - Two or more members. `for plot as $p { evidence $p_a from notes };`
    declared north_a with provenance `notes evidence south_a from notes`,
    and no south_a, before and then, silently. Where the copies did not read
    together as a statement, the program did not load, with the loader's
    error about the expanded text, such as "expected '=' at byte 17" for
    `{ state $p_n = 0; on read when target == $index set $p_n = 1 }`. A body
    whose first statement is empty ended every copy's last statement but the
    last one's, so at the end of the program, `{ ; evidence $p_a from notes }`
    loaded as written, before and then.
  - One member, north. The copy's last statement ran into the statement
    after the block: with `evidence tail from after;` there, north_a's
    provenance was `notes evidence tail from after`, and no tail was
    declared, before and then. At the end of the program, or with a `;`
    right after the block's, the copy loaded as written, before and then.
    It is refused all the same: the rule does not depend on what follows
    the block.
  - A `}` right before a body statement's `;`, as in
    `for plot as $p { evidence $p_a from y}; };`. That `}` ended the body,
    and ` };` was read after the block: only north_a was declared, with
    provenance `y evidence south_a from y }`, before and then.
  - A block that loaded only with the 2026-09-27 entries: one with a comment
    or a `{` in its header, a last block without its `;`, or one below a
    brace in a statement outside a block, among the shapes those entries
    list. Before, each was refused for that shape; then, each loaded as the
    examples above do. A `}` in a comment after the body, as in `} # }` and
    a `;` on the next line, loaded wrongly before as well: north_a's
    provenance was `notes } evidence south_a from notes }` before, and
    `notes evidence south_a from notes` then.

  A last block may still leave out its own `;` once its body's last
  statement has one, and a `}` in a comment, a quoted brace and a brace pair
  within a statement load as before.
- 2026-09-29: in a routed rule, `$Q` names the member that the rule's event
  names by another `kind` parameter Q, and the rule is written once for each
  member of Q's kind (routed Repetition 0.1 section 10). `$Q` was refused
  before as not bound. Every program that loaded before still loads, and
  expands byte for byte as before, except one now refused: a routed rule
  with a word that `$NAME` or `$index` begins and a `kind` parameter of its
  event begins too, longer, or as long and not the parameter the block is
  routed by, such as `$shown_n` in a block bound `$s` and routed by `about`.
  It read as the member's name followed by `hown_n`. No program in the
  repository has one. In a plain block, such a word reads as before. Of the
  programs that did not load, a `$` word in an `on` rule that a parameter of
  its event begins is refused with a message that says why, where it was
  refused as not bound.
