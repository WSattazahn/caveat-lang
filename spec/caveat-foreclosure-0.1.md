# CAVEAT Foreclosure 0.1 — decisions that close off options

**Status:** stable from 1.2.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); drafted for 1.2. It adds to
[Reactive 0.5](caveat-reactive-0.5.md),
[Reopening Triggers 0.1](caveat-reopening-triggers-0.1.md) and
[Dispatch 0.1](caveat-dispatch-0.1.md); it changes no program that loads
today.

Some decisions can't be undone. Dropping a table, publishing a package or
spending money changes the world, and reopening the decision afterwards
changes only the record. Caveat keeps why each decision was made and lets it
reopen, but it can't tell such a decision from any other, or say what making
it ruled out. Today an author writes the policy by hand
([prototype](../experiments/foreclosure-sweep/README.md#the-prototype)):

```caveat
decisions drop_old_table limit 1;
decisions rollback limit 4;
on drop when not has_sample(full_runs) reject "irreversible: needs a complete dry run";
on drop commit drop_old_table because enough using latest(full_runs) permitted by latest(grants);
on roll_back when committed(drop_old_table) reject "foreclosed by drop_old_table";
on roll_back commit rollback because enough using 1;
```

The policy works, and `caveat test` passes on it. What the runtime can't
see is the point of it. The foreclosure is a string: `explain` shows
`drop_old_table@1` and, apart from it, a `roll_back` "refused (policy)",
with nothing linking the two. The rule that an irreversible commit needs a
complete dry run is one guard among many, and a reader can't tell it is the
one that matters. `caveat check` reports C002 `no-reopening-path` for
`drop_old_table` and offers a reopening path or an `allow` comment, when the
real reason is that the world can't be put back.

This profile lets the declaration say it:

```caveat
decisions drop_old_table limit 1 irreversible forecloses rollback;
decisions rollback limit 4;
on drop commit drop_old_table because enough
  using if(has_sample(full_runs), latest(full_runs), latest(sampled_runs))
  permitted by latest(grants);
on roll_back commit rollback because enough using 1;
```

With only a sampled dry run, the drop rests on `sampled_runs@1`, which carries
`sampled_only` (`high`), and is refused as `evaluation/caveated_irreversible`.
With a complete one it rests on `full_runs@1`, which carries no caveat, and is
accepted.

After the drop, `explain` prints:

```text
  drop_old_table: 1 of at most 1, irreversible, forecloses rollback
  rollback: 0 of at most 4, foreclosed by drop_old_table@1
```

and a `roll_back` event is refused as `evaluation/foreclosed`:
`rollback is foreclosed by drop_old_table@1`.

## Syntax

```text
decisions NAME limit 1 irreversible [forecloses ACTION, ACTION, …] [reopened by TRIGGER, TRIGGER, …];
```

`TRIGGER` is as in [Reopening Triggers 0.1](caveat-reopening-triggers-0.1.md).
Each `ACTION` is a declared decision series or a plain commitment that a
`commit` in the program names, after procedure calls and `for` blocks are
expanded.

The words are read only in this position, after `limit N`, so they reserve
nothing. A series, stream or claim named `irreversible` or `forecloses` keeps
loading, and `decisions irreversible limit 1 irreversible;` declares an
irreversible series named `irreversible`. Only a module of a draft 0.5 bundle
([Caveat 0.5](caveat-0.5-draft.md)) may not declare them, as for every clause
word, since its rewriter reads words, not positions.

These do not load, each with its own message:

| Source | Message |
| --- | --- |
| `irreversible` with a limit other than 1 | `an irreversible series holds one revision: decisions NAME limit 1 irreversible` |
| `forecloses` without `irreversible` before it | `forecloses follows irreversible: decisions NAME limit 1 irreversible forecloses ACTION, …` |
| an `ACTION` that is not a series and that no `commit` names | `NAME forecloses ACTION, which is not a decision` |
| `ACTION` equal to `NAME` | `NAME cannot foreclose itself` |
| the same `ACTION` twice | `duplicate foreclosed decision ACTION` |

A declaration without either word keeps today's message, `decisions expects
NAME limit CAPACITY [reopened by STREAM, …]`, byte for byte.

## Meaning

### An irreversible decision

`irreversible` says that carrying out the decision changes something the
program can't change back. The runtime records that the author said so. It
does not check that the external action really can't be undone, and it does
not carry the action out. Caveat records the evidence supplied to it; it does
not authorize external actions.

An irreversible series is an ordinary decision series of one revision, with
two differences.

**Its commit names the caveats it accepts.** An executed `commit` of an
irreversible series is refused as `evaluation/caveated_irreversible` when its
grounds carry a caveat of consequence `material`, `high` or `catastrophic`
that its `retaining` clause does not name. The grounds are those the commit
records today ([Explanations 0.2](caveat-explanations-0.2.md), "Grounds of a
commitment"): its `using` and `citing` content and its retained caveats. The
guard, a `permitted by` grant and other lineage are not grounds and are not
read. A caveat of consequence `negligible` or `low` is not read either.

The message names each caveat in grounds order:
`irreversible drop_old_table rests on sampled_only (high), which its commit does not retain`.

So the prototype's guard becomes the default: a drop grounded on a sampled
dry run carries `sampled_only`, and is refused unless the author writes
`retaining sampled_only` to decide with that caveat attached, on purpose.
`withdrawn` is `material` and can't be retained
([Withdrawal 0.1](caveat-withdrawal-0.1.md)), so an irreversible decision is
never made on withdrawn evidence.

An irreversible commit with no `using`, `citing` or retained caveat has empty
grounds and is not refused by this rule. Irreversibility is about the world,
not certainty, so the rule asks for named caveats, not for evidence.

**Reopening doesn't make room.** An irreversible series may be reopened, by an
authored `reopen` or a declared trigger, exactly as any series: the reopening
records that the decision is in question, keeps its basis and caveats, and
`reopened(NAME)` is true. Its one revision stays the only one. A later commit
is refused as `limit/history_limit`, as for any full series. The record can be
questioned; the act it records can't be repeated.

### What a decision forecloses

Once a revision of an irreversible series `D` is committed, each `ACTION` it
`forecloses` is closed: an executed `commit` of `ACTION` is refused as
`evaluation/foreclosed`, with the message `ACTION is foreclosed by D@1`. When
several committed series foreclose the same action, the message names the one
committed first, by event sequence and then by series name.

- **Only later commits.** A foreclosure closes the commits after it. A
  revision of `ACTION` already committed keeps its status, its basis and its
  caveats, and may be reopened as before. Whether `D` may be committed while
  `ACTION` is in force is the author's policy, written as a guard.
- **Reopening `D` doesn't reopen the option.** `D` stays committed when it is
  reopened, and its foreclosure stands. Reopening questions the decision; it
  does not undo what the decision did.
- **Within an event.** Commits run in rule order. A commit of `ACTION` before
  the commit of `D` in the same event is accepted; one after it is refused,
  and the event publishes nothing.
- **Order of checks.** `foreclosed` is checked first, before every other
  check of the commit, since no other change could let it through.
  `caveated_irreversible` is checked once the grounds are known, after the
  `permitted by` check, so an unpermitted commit is still `not_permitted`.

A foreclosure is not evidence. It adds no relation to the graph, no reading
and no caveat, and it rewrites no basis.

### Codes

Both codes are new, for new syntax, as
[Compatibility](../docs/COMPATIBILITY.md#rejection-origins-and-codes) allows.
Both have origin `evaluation`: the commit ran, and the series' state refused
it, as for `decision_in_force`. Neither is `policy`, because the program wrote
no clause for it, so a bare expected rejection in a scenario does not match
it. A program that declares neither word never produces either code.

| Origin | Code | Meaning |
| --- | --- | --- |
| `evaluation` | `foreclosed` | An executed `commit` names an action that a committed irreversible decision forecloses. |
| `evaluation` | `caveated_irreversible` | An executed `commit` of an irreversible series rests on a caveat of consequence `material` or above that its `retaining` clause does not name. |

## Check

C002 `no-reopening-path` does not report an irreversible series. Its fix, a
reopening path, would never lead to a new revision; the declaration states
the reason the series holds one. Every other warning is unchanged, and a
program that declares neither word gets the same report, byte for byte.

## Snapshot, save, explain and views

- The `caveat-reactive/0.1` snapshot gains `irreversible`: by irreversible
  series, the actions it forecloses in source order (an empty list when it
  forecloses none). It is absent when no series is irreversible, so every
  program that loads today keeps its snapshot byte for byte.
- A save holds nothing new: the declaration comes from the source, which the
  save binds by digest ([Save 0.1](caveat-save-0.1.md#source-digest)).
  Restoring refuses a save whose journal commits a foreclosed action after the
  commitment that forecloses it, and one holding a second revision of an
  irreversible series. As with every restore check, a save that passes holds
  records the program could hold; restore never authenticates history.
- `caveat-explain/0.1` gives a series `irreversible: true` and `forecloses`
  when declared, and gives a foreclosed series, or a foreclosed plain
  commitment it lists, `foreclosed_by`: the committed revisions that foreclose
  it, by event sequence and then name.
  Each is absent otherwise. The text adds `, irreversible`,
  `, forecloses A, B` and `, foreclosed by D@1` to the series line, in that
  order.
- `dependents` is unchanged. A decision name is still not a subject.
- `caveat-reactive-view/0.1` and `/0.2` are unchanged: a foreclosure is not a
  relation.

## Compatibility with other additions

- [Retention Reasons 0.1](caveat-retention-reasons-0.1.md):
  `retaining C because E` names `C` for this rule, as `retaining C` does.
- [Citing 0.1](caveat-citing-0.1.md): cited content is grounds, so its
  caveats are read.
- The 1.1 trigger `reopened by changed STREAM`
  ([#223](https://github.com/WSattazahn/caveat-lang/issues/223)) may be
  declared on an irreversible series, as any trigger.
- [Procedures](caveat-procedure-symbols-0.1.md): a procedure that takes a
  `decisions` parameter may be passed an irreversible series. The rules apply
  to the commits it makes, after expansion.
- [Holds 0.1](caveat-holds-0.1.md): an irreversible series may be held and
  released like any series. A hold defers its reopenings and a release applies
  them; a hold never keeps it in force on withdrawn evidence. A hold never
  touches a foreclosure: what an irreversible revision forecloses stays
  refused while it is on hold, after its release, and after a held
  foreclosed series is released and reopened.
- [Late Qualification 0.1](caveat-late-qualification-0.1.md): `retaining`
  cannot name a caveat the program takes back with `unqualify` (an existing
  load rule). An irreversible commit resting on such a caveat, when it is
  material or above, goes ahead only after the take-back.
- `unqualify` cannot name a reading stream's template evidence (an existing
  load rule), so `withdrawn` is the only late caveat a stream reading can
  carry into an irreversible commit's grounds.

## Questions settled

The design note left two questions, and the profile settles four more.

1. **A guide note first?** No. It was proposed for the 1.0 soak, which has
   ended. The authoring guide describes the declared form when the runtime
   ships it.
2. **Series or value?** A series. A decision forecloses another decision, not
   one value of it (`rollback for table == "orders"`). Values would need
   [member symbols](caveat-member-symbols-0.1.md), which stays a draft until a
   second real program needs it. A guard still narrows a foreclosure by value.
3. **May an irreversible decision reopen?** Yes. The design note proposed
   refusing it. But a decision that can't be undone can still be wrong, and
   reopening is how a program records that. Reopening keeps its meaning,
   "this is in question", and loses only the new revision it would allow.
4. **What counts as stronger evidence?** Named caveats. The runtime can't
   tell how good evidence is; it can tell which caveats a commit rests on.
   `material` is the threshold because it is the level that "matters" and the
   level of `withdrawn`, so a withdrawn reading can never ground an
   irreversible commit. `retaining` lets the author accept a caveat
   knowingly, as Caveat's commitments always could.
5. **May a reversible decision foreclose?** No, not in this profile. While a
   reversible decision is in force, closing another is mutual exclusion, and
   it would lift when the decision reopens. That is a different relation. A
   later profile may add it; loosening a load error is an addition.
6. **Option accounting?** Not in this profile. Counting the open options and
   showing how a commit changes the count needs a real program where the count
   changes a decision.

## Not in this profile

- A `foreclosed(ACTION)` predicate. `committed(D)` already guards a rule.
- A `caveat check` warning for an irreversible commit whose grounds can carry
  an unretained caveat. The runtime refusal is the guarantee; a warning can
  follow if programs need it.
- `dependents` of a decision name, which would also have to report what reads
  `latest(SERIES)` for every series.
- Foreclosing an option value, or foreclosing for a period.
- Checking that an external action really can't be undone.

## Invariants kept

From [Caveat's invariants](../docs/CAVEAT_ESSENCE.md):

- A foreclosure deletes no evidence and rewrites no earlier basis.
- An irreversible decision keeps its grounds and caveats, and may be reopened
  with its cause recorded.
- A refused event publishes nothing, so a refused irreversible commit leaves
  every option open.
- The runtime records that a decision was declared irreversible. It does not
  authenticate that claim or authorize the action.

## Proofs

`proofs/lean/Caveat` models evidence, grounds and frozen bases. Both rules
refuse a commit before it records anything, so the frozen-basis and rollback
theorems cover them without change. The implementation adds runtime tests.

## Corpus sweep

`node experiments/foreclosure-sweep/sweep.mjs` reads every tracked Caveat
program and writes
[`RESULTS.json`](../experiments/foreclosure-sweep/RESULTS.json). It reads
source text only and runs no program. At this profile's revision:

- 191 programs declare 138 decision series, in 129 programs. None writes
  `irreversible` or `forecloses` in a declaration, and no declared name is
  spelled like either word, so no program reaches the new form;
- 4 series are declared `limit 1`. Three have no reopening path:
  `publish` in `experiments/agent-ledger/release.cav`, `stabilization` in
  `experiments/release-readiness/release-readiness.cav` and `ruling` in
  `game/before_the_rain.cav`. Publishing a package is the clearest case of an
  action that can't be undone. Source text can't say whether the others
  are, so these are candidates, not mistakes;
- no tracked program rejects one decision's commit because another is
  committed, or guards a commit with `not committed` of another decision. The
  prototype that does lives outside the corpus, and the sweep finds its
  `roll_back` rule when the prototype is added to it.

No accepted program changes.

## Implementation

- The runtime parses the two words after `limit N`, applies the load rules,
  and adds the two checks to `commit` in the order above. A foreclosed
  action's refusal names the foreclosing revision. Restore checks that no
  foreclosed action was committed after its foreclosure; a second revision of
  an irreversible series already fails restore's series limit check. The
  snapshot carries `irreversible`.
- `runtime/tests/foreclosure.rs` covers the prototype's migration, a
  foreclosed plain commitment, the refusal of a caveated irreversible commit
  and its acceptance with `retaining`, a `low` caveat that is not read, the
  permission check coming first, a withdrawn ground, a reopened irreversible
  series that keeps its foreclosure, the in-event order, the load rules, the
  words as names, a procedure parameter, C002, and a save and restore round
  trip with restore's check.
- `runtime/tests/foreclosure.rs` also covers holds: a deferred reopening of
  an irreversible series and its release, a withdrawal breaking through the
  hold, a hold before the commit, a held foreclosed series released and
  reopened, and a save with a hold.
- `runtime/tests/foreclosure_invariants.rs`, from the bug hunt, runs 300
  random histories of 60 steps with save and restore in place, beside a twin
  program without the two words, and checks each rule above on every step.
- The kit's types add the two codes and the snapshot and explain fields, and
  `explain` prints them. Dispatch 0.1, Compatibility, the language reference
  and the authoring guide describe the form.

## Changes

- 2026-10-18: specified for 1.2, from the
  [design note](../experiments/foreclosure-sweep/README.md).
- 2026-10-18: implemented on its branch.
- 2026-10-18: the bug hunt's random check added; no finding.
- 2026-10-19: checked with Holds 0.1 and Nested Repetition 0.1 on `main`.
