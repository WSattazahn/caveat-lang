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

A checked rule is *routed* when a top-level conjunct of its `when` guard is
`P == E` or `E == P`, where P is such a parameter and E is one of these:

- `$index`;
- a member constant `Q.MEMBER`, where `Q` is a parameter declared
  `kind KIND` and `MEMBER` is a member of KIND;
- any expression that mentions `$NAME` or `$index`, such as `$p_base`.

The top-level conjuncts are the operands of the guard's chain of `and`.
Parentheses do not change them: `(A and B) and C` has three. A conjunct that
is the name of a [define](caveat-define-0.1.md) is replaced by the conjuncts of
its expression as written, so `define $m_here = target == $index;` followed by
`on absorb when $m_here and ...` is routed. A comparison under `or` or `not`,
or inside a function call or an `if`, is not a top-level conjunct and does not
count. Neither does a comparison with `!=`, or with a number such as
`target == 1`.

The warning is at the rule where it is written in the block, once for each
member whose copy is not routed. When EVENT is itself written with `$NAME`,
each member's copy is checked against its own event. The suggestion is to add
the selection if the rule is about the member the event names, or to allow
the rule on purpose if it should run for every member.

It does not check:

- a rule on an event with no parameter of the block's kind, such as `tick`,
  which reaches every member by design;
- a rule that mentions neither `$NAME` nor `$index`, whose copies are all the
  same;
- rules outside `for` blocks, procedure steps and bindings;
- whether the selection is the right one. `target == target.north` in the
  copy for `south` is routed, and so is one parameter compared where another
  was meant.

A rule that should run for every member, such as one that resets every
member's count on each event, is allowed on purpose with
`# caveat check: allow unrouted-member-rule` on the line above it in the
block.

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

- C003 `unrouted-member-rule`. In a study of the agent ledger, each of its 31
  member rules was written again without its selection, one at a time. Every
  such program loaded and checked clean. C003 reports all 31. In the
  repository's unchanged programs it reports only two rules in Trail Rescue
  that reset every tunnel on purpose; they are now allowed. A program that
  checked clean before may now warn, and fail with `--strict`.
