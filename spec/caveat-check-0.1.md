# CAVEAT Check 0.1

`caveat check PROGRAM` loads a program and reports patterns worth a second
look. A warning describes what the text of the program shows. It is not a
proven fault, and it never changes what the program means or does. The check
does not rewrite a program, reopen a decision or change a policy.

Use `caveat validate` to learn whether a program loads, `caveat test` to learn
whether it does what its scenarios say, and `caveat explain` to see why a
session is in the state it is in. A clean check proves none of those.

## Running it

```sh
npx --no-install caveat check [--json] [--strict] PROGRAM
```

| Exit status | When |
| --- | --- |
| 0 | The program loads. This holds whether or not there are warnings. |
| 1 | `--strict` was given and there is at least one warning. |
| 2 | The program does not load, cannot be read, or is a bundle. |

A program that does not load gets the loader's diagnostic, as from
`caveat validate`. Check reads a single-file program. It does not read a
[module bundle](caveat-0.5-draft.md).

## Warnings

Every warning has a stable code and name, a severity (always `warning` in
0.1), a location, a message and a suggestion. The location is the one-based
line and Unicode column where the statement it is about starts. For a
statement written inside a `for` block
([repetition](caveat-repetition-0.1.md)), that is where the statement is
written in the block. Each member it expands to gets its own warning at that
place.

### C001 `repeated-rules`

It reports the rules on one event when they repeat the rules on an earlier
event, which may make them candidates for one
[procedure](caveat-procedure-symbols-0.1.md).

For each event, its top-level `on` rules are taken in source order, as
written. Two events' rule lists match when all of these hold:

- they have the same number of rules, at least two;
- each rule has the same words in the same order, apart from names and
  numbers;
- each differing name is renamed consistently: one name always stands for the
  same name in the other list, and in both directions;
- every name that differs is one a procedure can take as a parameter. That
  means two reading streams, two decision series, two pieces of evidence, two
  claims, two caveats, or a parameter of each event.

Numbers may differ anywhere. Quoted text, operators and every other word must
be identical. The warning is at the first rule of the later event. It lists
the names that differ, and it points back to the earliest event whose rules
match.

It does not compare:

- rules in a `proc`;
- rules in a `for` block, which are already written once;
- part of an event's rules against part of another's;
- rules on the same event.

A state name that differs means no warning, because a procedure cannot take a
state. The check compares text, not meaning. Whether one procedure keeps what
both events mean is for the author to judge.

### C002 `no-reopening-path`

It reports a decision series that is decided on readings when nothing in the
program reopens it.

A series is *decided on readings* when a `commit` of it reads a reading
stream. That covers the commit's own `when` guard and its `using` expression
reading the stream with `latest`, `history_count`, `history_at`, a fold,
`has_sample` or `withdrawn(latest(...))`.

A *reopening path* is any `reopen` of the series, in a rule or in a
procedure, or a declared [reopening trigger](caveat-reopening-triggers-0.1.md)
on it. Both commits and reopenings are found after procedure calls and `for`
blocks are expanded. A reopening inside a procedure that takes the series as a
`decisions` parameter therefore counts.

A series may not be committed again until its current revision is reopened
([reactive 0.5](caveat-reactive-0.5.md)). So with no reopening path, a series
holds at most its first revision, and later readings never reconsider it. The
program may intend exactly that. The warning is at the series' `decisions`
declaration. It asks the author to confirm the intent, and suggests a trigger
or a reopening rule if later readings should reconsider it.

Only the commit's own guard and `using` expression count. These are not
followed:

- a stream read in the condition of the rule that calls the procedure;
- a state whose value came from a reading;
- a permission grant, which is not grounds
  ([permission](caveat-permission-0.1.md)).

Every `reopen` counts, whether or not its guard can ever be true.

### C003 `unrouted-member-rule`

It reports a rule written for each member of a `for` block, on an event that
names a member, when its guard does not select which member that is.

A `for` block is a text expansion ([repetition](caveat-repetition-0.1.md)).
Each member gets its own copy of every rule in the block, and every copy runs
on every event its guard allows. A copy stays with its own member only if its
guard selects the member the event names, conventionally with
`target == $index`. Without it, an event about one member acts for all of
them, for example recording one pull request's go-ahead in every pull
request's stream.

A rule is checked when all of these hold:

- it is an `on EVENT` rule written in a block `for KIND as $NAME { ... }`;
- EVENT declares at least one parameter `P kind KIND`;
- the rule mentions `$NAME` or `$index` anywhere, quoted text included.

A *selection* is `P == E` or `E == P`, where P is such a parameter and E is
one of these:

- `$index`;
- a member constant `Q.MEMBER`, where `Q` is a parameter declared
  `kind KIND` and `MEMBER` is a member of KIND;
- any expression that mentions `$NAME` or `$index`, such as `$p_base`.

A checked rule is *routed* when a top-level conjunct of its `when` guard is a
selection, or is a chain of `or` every disjunct of which is a selection. The
disjuncts may compare different parameters. After
`event hold suspect kind suspect, first kind exhibit, second kind exhibit;`,
a rule on `hold` in a block over exhibit, guarded by
`(first == $index or second == $index) and $e_points == 0`, is routed: each
copy runs only when the event names its member, by either parameter.

The top-level conjuncts are the operands of the guard's chain of `and`, and
the disjuncts of a conjunct are the operands of its chain of `or`.
Parentheses do not change them: `(A and B) and C` has three conjuncts, and
`(A or B) or C` has three disjuncts. A conjunct that is the name of a
[define](caveat-define-0.1.md) is replaced by the conjuncts of its expression
as written, so `define $m_here = target == $index;` followed by
`on absorb when $m_here and ...` is routed. A disjunct that is the name of a
define is replaced by the disjuncts of its expression in the same way. The
defines read this way are every define outside a block, and every define
written in a block over the same KIND, whatever its binding. A define written
out by hand for one member, such as `define north_here = target == 1;`, is
read as written, with its number, and so does not count.

A chain of `or` with any disjunct that is not a selection does not count, as
in `target == $index or celsius > 50`, `target == $index or target != $index`,
or `first == $index or suspect == $index`, where `suspect` is of another kind.
Neither does one with a disjunct that is itself a chain of `and`, as in
`(first == $index and $e_points > 0) or second == $index`. A comparison
under `not`, or inside a function call or an `if`, is not a top-level
conjunct or a disjunct of one, and does not count. Neither does a comparison
with `!=`, or with a number such as `target == 1`.

The warning is at the rule where it is written in the block, once for each
member whose copy is not routed. The suggestion is to add
the selection if the rule is about the member the event names, or to allow
the rule on purpose if it should run for every member.

It does not check:

- a rule on an event with no parameter of the block's kind, such as `tick`,
  which reaches every member by design;
- a rule on an event written with `$NAME` or `$index`, such as
  `on absorb_$m`. Each member's copy runs on that member's own event, so the
  event already selects the member;
- a rule that mentions neither `$NAME` nor `$index`, whose copies are all the
  same;
- a rule that loads only once `$index` is a number, such as
  `examine X cost $index`, because it cannot be read as written;
- rules outside `for` blocks, procedure steps and bindings;
- a rule in a [routed](caveat-routed-repetition-0.1.md) block;
- whether the selection is the right one. `target == target.north` in the
  copy for `south` is routed, and so is one parameter compared where another
  was meant, alone or in a chain of `or`.

A selection made another way is not recognized, and the rule is reported. For
example, after `on read set current = target;`, the guard
`current == $index` does select the member, but not in a form above. Allow
such a rule on purpose.

A rule that should run for every member, such as one that resets every
member's count on each event, is allowed on purpose with
`# caveat check: allow unrouted-member-rule` on the line above it in the
block.

### C004 `shifted-member-index`

It reports a rule written for each member of a `for` block that selects its
member with `$index`, when the event numbers that member otherwise.

`$index` is a member's position among the part's top-level `entity`
statements of the kind ([repetition](caveat-repetition-0.1.md)). A parameter
`P kind KIND` numbers every entity of KIND in the loaded program, in the order
declared, and that includes an entity a `for` block declares, where the block
is expanded. When such an entity comes before a top-level member, P numbers
that member, and each member after it, higher than its `$index`. The member's
copy of `P == $index` then holds when the event names another entity. In

```caveat
entity east kind plot at field;
entity z kind zone at field;
for zone as $z {
    entity north kind plot at field;
};
entity south kind plot at field;
event read target kind plot;
for plot as $p {
    state $p_n = 0;
    on read when target == $index set $p_n = $p_n + 1;
};
```

`target` numbers east 1, north 2 and south 3, and south's `$index` is 2. So
`read north` counts for south, and `read south` for no plot.

A rule is checked when all of these hold:

- it is an `on EVENT` rule written in a block `for KIND as $NAME { ... }`
  that is not [routed](caveat-routed-repetition-0.1.md);
- EVENT declares at least one parameter `P kind KIND`;
- a top-level conjunct of its `when` guard is `P == E` or `E == P`, where P
  is such a parameter and E reads `$index`, itself or through a define, such
  as `$index`, `$index + 1`, `round($index)`, or `$p_slot` after
  `define $p_slot = $index;`. Or the conjunct is a chain of `or` every
  disjunct of which is a selection, as C003 defines it, and one of its
  disjuncts is such a comparison, as in `from == $index or to == $index`.

The conjuncts and their disjuncts are read as C003 reads them, defines
included, so `define $p_here = target == $index;` followed by
`on read when $p_here` is checked. A define E reads through is one outside a
block, or one written in a block over the same KIND, whatever its binding, as
for C003. A rule on an event written with `$NAME` or `$index`, such as
`on $p_read`, is checked too: its copy still compares P with `$index`. So is
a rule whose guard has another conjunct that selects the member, as in
`to == to.$p and (from == $index or to == $index)`. C003 does not need the
chain of `or` there, and C004 still checks each of its comparisons.

In each member's copy, E comes to a number: the check works it out from the
member's `$index`, numbers, defines, and calls to functions, the prelude's
and the program's. The copy runs when P names the entity that P gives that
number. Each such comparison in the guard is checked on its own: each
top-level conjunct that is one, and each disjunct that is one in a chain of
`or` of selections. A chain holds when any of its disjuncts does, so each
disjunct is a way the copy runs. In
`(from == $index or to == $index) and via == $index`, and with its conjuncts
in the other order, the comparisons of `from`, `to` and `via` are each
checked. A copy is reported for a comparison when all of these hold:

- P numbers some top-level member of KIND otherwise than its `$index`;
- the entity the copy runs for is not the member itself;
- it is not the member whose `$index` that number is.

So the copy for `south`, with `$index` 2, of `target == $index` above is
reported: it runs for north, which P numbers 2. So is the copy for `east` of
`target == $index + 1` there: it compares `target` with 2, the `$index` of
south, and runs for north. A copy that the numbering does not change is not
reported, such as one that compares with the number of a member before every
entity a block declares. Neither is one whose E comes to the member's own
number in P, as `$index + 1` does for each member when one entity that a
block declares comes before them all.

The warning is at the rule where it is written in the block, once for each
comparison reported in each member's copy. With
`from == $index or to == $index`, or with `from == $index and to == $index`,
a member's copy can be reported for `from` and again for `to`. In one
copy, two comparisons of the same P with the same number, as in
`to == $index or to == $index`, are reported once. The message names the
entity the copy runs for, and the member whose `$index` the number is: the
member itself, where E comes to the member's `$index`, and otherwise another
member, with the number. It names the entities of KIND that P counts before
that member and `$index` does not, and lists each, as related, where its
block declares it. When every entity of KIND that a `for` block declares
comes after every top-level member, the two numberings agree, and nothing is
reported.

The suggestion is to select the member by name, as in
`target == target.$p`, if the rule is about the member the event names: a
name selects the block's own member however the entities are counted. A rule
that means to compare with `$index` is allowed on purpose with
`# caveat check: allow shifted-member-index` on the line above it in the
block.

It does not check:

- a rule in a routed block. A routed block refuses an entity of its kind in
  a `for` block ([routed repetition](caveat-routed-repetition-0.1.md)
  section 7), so its `$index` and P agree;
- a selection by name or by a member constant, such as `target == target.$p`
  or `target == target.east`;
- `$index` reached some other way: through a state, as in
  `target == $p_slot` after `state $p_slot = $index;`, or in a comparison
  that is neither a top-level conjunct nor a disjunct of a chain of `or` of
  selections. C003 reports such a rule, as it does
  `target == $index or $p_n > 5`, unless another conjunct selects the member
  or the rule is on the member's own event. Then nothing reports it: in
  `to == to.$p and (from == $index or $p_n > 5)`, `from == $index` is not
  checked, even where `from` numbers the members otherwise than `$index`;
- an E that reads a state, another parameter or the graph, such as
  `$index + $p_offset` after `state $p_offset = 0;`. Which entity its copy
  runs for can change as the program runs, so the check works out no number
  for it;
- a number that is no member's `$index`, or that P gives no entity;
- a rule that loads only once `$index` is a number, as for C003.

An entity of KIND that a `for` block declares gets no copy of a block over
KIND, so no copy selects it by name: a copy that selects its member by name,
as with `target == target.$p`, does not run on an event that names it. A
copy that selects its member by `$index`, or selects none, can still run on
such an event: in the example above, `read north` runs south's copy. That
the entity gets no copy is not reported. A block that declares an entity
usually handles it there, by name, as in
`for zone as $z { entity north kind plot at field; on read when target == target.north set north_n = north_n + 1; };`,
and a warning would fall on those programs too.

### C006 `citation-unreachable`

It reports a `bind` whose `because` names something that its value and its
`when` condition never read. Such a binding is refused at dispatch as
`evaluation/ungrounded_citation` whenever it supplies the shown value
([Explanations 0.1](caveat-explanations-0.1.md)), which can be long after the
program loads.

```caveat
bind journal.text = journal_text(journal_kind, journal_who) because journal_cite;
```

The check compares names after `define`s and pure function calls are
expanded. For the citation, it collects the states, evidence, caveats,
reading streams, decision series and `elapsed()` it names. For the binding,
it collects the same from the value and the condition, taken or not. Entity
coordinates are left out. A citation that names at least one of these and
shares none with its binding is reported. The warning is at the `bind`. A
binding in a `for` block gets one warning per member, all at the template.

The comparison is by name, so it can be wrong both ways:

- A citation that shares a name with its binding can still be refused, for
  example when the shared read is on a branch the value does not take.
- A citation that shares no name can still be grounded, when a state the
  binding reads carries the evidence or caveats the citation brings. An allow
  comment silences that case.

In round 7 of the [glowcap experiment](../experiments/glowcap/RESULTS.md),
two of four authors wrote this shape at CR16. Each first run was refused as
`ungrounded_citation`. C006 reports both first runs at the binding that was
refused, and neither of the versions that passed. No other program in the
repository reports it.

## Allowing a pattern on purpose

A comment on the line directly above the statement a warning is about
silences that warning:

```caveat
# caveat check: allow no-reopening-path
decisions season_plan limit 1;
```

After the comment marker come the words `caveat check: allow`, then the
checks by code or name, separated by commas or spaces. Each of those words
stands alone, separated by whitespace, so `allowance C002` and `allow,C002`
are not the directive. `//` comments work the same way. Only the line directly
above counts, and only a check the comment names is silenced. A silenced
warning is still reported, as allowed, so nothing is hidden. `--strict`
ignores it.

## The report

`--json` prints:

```json
{
  "schema": "caveat-check/0.1",
  "program": "instruments.cav",
  "loads": true,
  "strict": false,
  "diagnostics": [
    {
      "code": "C001",
      "name": "repeated-rules",
      "severity": "warning",
      "line": 12,
      "column": 1,
      "message": "the 2 rules on `drone_read` repeat the rules on `probe_read` (line 10), with `aerial` for `soil`",
      "suggestion": "If both events should keep following the same rules, …",
      "related": [{ "line": 10, "column": 1, "note": "the rules on `probe_read`" }]
    }
  ],
  "suppressed": []
}
```

`suppressed` holds silenced warnings, in the same form. A program that does not
load gives `{"schema": "caveat-check/0.1", "program": …, "loads": false,
"error": …}`. Messages and suggestions are text for people and may change;
codes, names, locations and the fields above are stable. From a script,
`runtime.check(source)` in `caveat-lang/session` returns the same report
without `program`, `loads` and `strict`. It throws a `CaveatError` of kind
`load` for a program that does not load.

## Changes

- 2026-10-05 (rc.15): C006 `citation-unreachable`, advisory. Round 7's C2 and
  C3 authors each wrote, at CR16, a journal binding that cites a state its
  text never reads, and each first run was refused at dispatch as
  `evaluation/ungrounded_citation`. C006 reports both at that binding. No
  repository program reports it. A program that checked clean before may now
  warn, and fail with `--strict`. `caveat test` now runs this check before
  the first scenario ([Scenarios 0.1](caveat-scenarios-0.1.md)).

- C003 `unrouted-member-rule`. In a study of the agent ledger, each of its 31
  member rules was written again without its selection, one at a time. Every
  such program loaded and checked clean. C003 reports all 31. In the
  repository's programs it reports only two rules in Trail Rescue, which reset
  every tunnel's tallies on each observation on purpose. They carry no allow
  comment, because the Trail Rescue dispatch audit pins that file byte for
  byte. A program that checked clean before may now warn, and fail with
  `--strict`.
- C004 `shifted-member-index`. A plain block that selects its member with
  `target == $index` acts for another member when an entity of its kind that
  a `for` block declares comes before a top-level member, as in the example
  above, and nothing reported it. In an evaluation of 23 programs, C004
  reports each of the 6 in which a member's copy acts for another member,
  and none of the 17 others: those in which every such entity comes after the
  top-level members, those that select by name, and those in which the
  declared entity only gets no copy. Of 14 more, from its review, it reports
  each of the 11 in which a copy that selects by `$index` acts for another
  member, one through `define $p_slot = $index;` and one allowed by its
  comment, and none of the 3 in which every copy acts for its own member, one
  through `$index + 1`. No program in the repository declares an entity in a
  `for` block, so none of them warns. A program that checked clean before may
  now warn, and fail with `--strict`.
- 2026-09-29: C003 counts a chain of `or` every disjunct of which is a
  selection as routing. Before, a comparison under `or` never counted. A rule
  that selects its member by either of two parameters of the kind, such as
  `(first == $index or second == $index)` on a hold that presents two
  exhibits, was reported for every member, and the allow comment the
  suggestion offered, for a rule that should run for every member, did not
  fit it. In Before the Rain, a game written in Caveat outside this
  repository, C003 reported each of the 5 rules on `hold` in its exhibit
  block for each of its 7 exhibits, 35 warnings, and nothing else. It now
  checks clean, with no C004. A chain of `or` with any disjunct that is not a
  selection is still reported. Dropping `routed by P` from a block with a
  rule guarded by `P == $index or Q == $index` is no longer reported
  ([routed repetition](caveat-routed-repetition-0.1.md) section 8). C004
  checks every comparison of P with an E that reads `$index`: each top-level
  conjunct that is one, and each disjunct that is one in a chain of `or` of
  selections, in whatever order they are written. It checks such a chain
  even where C003 does not need it, because another conjunct selects the
  member or the rule is on the member's own event. Before, it checked only
  the first top-level conjunct that was one, and no chain of `or`. Every C004
  reported before is reported now, and the same P compared with the same
  number twice in one copy is reported once. Of the repository's 106 `.cav`
  files, the 80 that load report exactly what they did before, 7 C001 and 4
  C003 warnings: none selects a member under `or`, and none declares an
  entity in a `for` block. A program that warned before may now check clean.
  A program that checked clean before may now warn, and fail with `--strict`,
  where a comparison that C004 did not read selects by a shifted `$index`.
  With north declared in a block before east and south, each of these rules
  got no C004 before, and now gets it for east and for south.
  `via == $index + 1 and from == $index` gets it on `from`; with its
  conjuncts swapped, it already did. `from == $index or to == $index` gets it
  on `from` and on `to`. It got C003 instead, or nothing where it was allowed
  on purpose. `to == to.$p and (from == $index or to == $index)` gets it on
  `from` and on `to`. It got nothing: C003 found its selection in
  `to == to.$p`. `from == $index or to == $index` on the member's own event,
  such as `on $p_move`, gets it on `from` and on `to`. It got nothing: C003
  does not check a rule on the member's own event. A rule that got C004
  before may get more: `from == $index and to == $index` got it on `from`,
  and now gets it on `to` as well.
