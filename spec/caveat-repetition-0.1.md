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
statement, a last block may leave out its `;`. Nothing but comments may come
between the body's `}` and the block's end. A brace in the body's unquoted
text pairs only within its statement. The body ends at its first `}` outside
quoted text, comments and procedure bodies that does not close a `{` earlier
in its own statement, so a pair in unquoted provenance, such as
`from see{appendix}b;`, stays in its statement, and in `from a}b;` the `b`
is text after the body. A body statement that leaves a `{` open, as
`from x{;` does, is refused, and the error gives the line and column of
that `{`: a `}` in a later statement does not close it. Quoted, as in
`from "x{";`, the text is the statement's own. Two bindings are available
inside the body:

- `$NAME` — the member's name.
- `$index` — its one-based position in the kind's declaration order.

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
  other `}` ends the body. So a body statement that leaves a `{` open is
  refused: "line L, column C: for block `for plot as $p`:
  `evidence $p_a from x{` has a `{` that its statement does not close; a
  brace in unquoted text pairs only within its statement, so quote the
  text", with the line and column of that `{`. With `see{appendix;`, `for{ supports $p_c;` or
  `entity $p_hut{` in a body, which leave it open, the program did not load
  before either. Where a `}` in a later statement of the body closed it,
  Repetition paired the two, and the program loaded as written: a body of
  `evidence $p_a from x{; evidence $p_b from y};` over north and south
  declared north_a, north_b, south_a and south_b, with provenance `x{` and
  `y}`. Such a body is now refused, for any number of members, because that
  `y}` ends the body; quoted, as `from "x{"` and `from "y}"`, it loads as it
  did. A `}` that closes no `{` of its statement is refused too when
  anything but comments follows it before the block's end, as text after
  the body. In a body's last statement without its `;`, as in `from a}b` or
  `from x }` right before the body's own `}`, Repetition took the block's
  last `}` for the body's end, and the program loaded as written; it is now
  refused as text after the body. The brace in a name such as
  `entity north{` hid that entity too: the loader declared it, and a `kind`
  parameter counted it. It is now a member, with its copy and its `$index`,
  as is every name the loader declares, such as `no-rth` or `1north`: the
  loader does not check a name against the identifier syntax. The name
  reaches its copy as written. Where the copy needs a reactive identifier,
  as in `state $p_n`, the program does not load, and the error gives the
  copy's line in the expanded text: "invalid reactive identifier north{_n".
  Below a block, such a program loaded before, without copies for north{
  and the members after it; it now does not load, and its saves cannot be
  restored. In quoted text, in provenance, or in a claim's or evidence's
  name, the copy loads as the same text written by hand does. An entity
  declared in a block's body is no member, whatever its name. A routed
  block refuses one of its kind, and a plain block has the miscount that
  section 7 of [routed Repetition 0.1](caveat-routed-repetition-0.1.md)
  describes: it gives the entity no copy while a `kind` parameter counts
  it, so where the body selects its member with `target == $index`, an
  event that names the entity, or a member after it, updates another
  member's copy, or none, and nothing reports it. `entity north{` there
  leaves its `{` open and is refused; the program did not load before.
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
  Text between a block's body and its end, such as a statement after a
  block whose `;` was left out, is refused: "for block has text after its
  body: ...". Repetition dropped it, silently when a `;` followed it. A last
  statement after an empty statement, as in `};;` followed by `entity south
  kind plot at field`, was not read either: its entity got no copy while a
  `kind` parameter counted it, so an event that named it changed nothing,
  silently, and a last block there was not expanded. Both are now read.
