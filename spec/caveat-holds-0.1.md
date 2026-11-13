# CAVEAT Holds 0.1 — keeping a decision in force for a while

**Status:** stable from 1.2.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); drafted for 1.2. It adds to
[Reactive 0.5](caveat-reactive-0.5.md),
[Reopening Triggers 0.1](caveat-reopening-triggers-0.1.md) and the
[Decision Journal 0.1](caveat-decision-journal-0.1.md); it changes no
program that loads today.

A decision series is reopened when its rules or its
[triggers](caveat-reopening-triggers-0.1.md) say so. Sometimes a decision has
to stay in force for a while whatever arrives: a release is frozen while a
deployment runs, a plan is fixed while a team carries it out. Before this
profile, the only way to keep it was to guard every reopening rule with a
state that says so, and a trigger has no guard:

```caveat
state frozen = 0 min 0 max 1;
on freeze set frozen = 1;
on retract when frozen == 0 and committed(gate) and not reopened(gate) reopen gate because latest(tests);
```

Such a guard drops the reopening. When the freeze ends, nothing records that a
new report arrived during it, and the decision stays in force on a basis that
the program would have reconsidered. A hold keeps the decision in force and
keeps the reopenings it held back:

```caveat
decisions gate limit 8 reopened by tests;
on freeze hold gate because team;
on unfreeze release gate because team;
```

While `gate` is on hold, a new report does not reopen it; the journal records
that the reopening was deferred. When the hold is released, the deferred
reopenings apply, and `gate` is reopened because of them.

## 1. Syntax

```text
hold SERIES because SELECTOR
release SERIES because SELECTOR
on_hold(SERIES)
```

`hold` and `release` are effects. They are written wherever `reopen` is: in a
rule, after its guard, or as a step of a [procedure](caveat-procedure-symbols-0.1.md).
`SELECTOR` is any selector `reopen` takes: an evidence name, an occurrence,
`latest(STREAM)` or `caveated(STATE, CAVEAT)`. `SERIES` must be a decision
series; a one-off commitment is refused:

| Source | Message |
| --- | --- |
| `hold go because team;` where `go` is not a series | `hold names a decision series; go is not one` |
| `release go because team;` where `go` is not a series | `release names a decision series; go is not one` |
| `on_hold(go)` where `go` is not a series | `on_hold names a decision series; go is not one` |

`on_hold(SERIES)` is a predicate, written wherever `reopened(SERIES)` is. A
program that defines its own `fn on_hold` keeps calling it, as a 1.0 program
that defines `fn in_basis` does.

## 2. Meaning

A hold is of the series' current revision. Its reasons are the evidence its
selector names.

- **Hold.** `hold SERIES because E` puts the current revision on hold,
  because of `E`. A revision already on hold gains `E` as one more reason. A
  revision already reopened is not in force, so nothing is held, and the
  effect does nothing. Holding a series with no revision yet is refused like
  reopening one, with `not_committed`.
- **While on hold.** The revision stays in force: `reopened(SERIES)` stays
  false and a `commit` of the series is still refused. A `reopen` of it, by a
  rule or by a trigger, does not reopen it: its causes are **deferred**,
  each recorded once. A cause that already reopened the revision is not
  deferred again.
- **Release.** `release SERIES because E` ends the hold. The revision is then
  reopened because of every deferred cause, in the order they were observed,
  exactly as a `reopen` naming them would reopen it. With nothing deferred,
  the revision stays in force. A release of a revision that is not on hold
  does nothing.
- **A withdrawal is not held.** A hold never keeps in force a revision that
  rests on withdrawn evidence. A `reopen` of a revision on hold whose basis
  holds withdrawn evidence
  ([`rests_on_withdrawn`](caveat-withdrawal-0.1.md)) goes ahead: the hold
  ends, and the revision is reopened because of the reopening's own causes
  and every deferred one.
- **A reopening ends the hold.** Once the revision is reopened, by a release,
  through a withdrawal, or otherwise, it has no hold and nothing deferred. The
  next revision starts with none.

A hold never drops a reopening and never adds one: every cause that would
have reopened the revision without the hold reopens it once the hold ends.
What the hold changes is when.

`on_hold(SERIES)` is true while the series' current revision is on hold. Its
grounds are those of `reopened(SERIES)`: the reasons of the hold, the deferred
causes, the release, and the guards of skipped reopenings
([Reactive 0.5](caveat-reactive-0.5.md)). A hold, a deferral and a release
each join the revision's reopening record with their selector's lineage and
the effect's guard, as a reopening does.

## 3. Journal

A program with a hold records three more changes in the
[decision journal](caveat-decision-journal-0.1.md), each of the revision on
hold:

| `change` | `because` |
| --- | --- |
| `held` | the reasons this effect added to the hold |
| `deferred` | the causes this reopening deferred |
| `released` | the release's selector |

A release that reopens the revision records `released`, then `reopened`
because of the deferred causes, at the same event. The reasons of a hold and
the causes it defers stay pinned while the hold lasts
([Departure 0.1](caveat-departure-0.1.md)): a windowed record they name
cannot depart before the release cites it. A program without a hold records
only `committed` and `reopened`, as before.

## 4. Save

A decision series in a save carries two more fields, each left out when
empty:

| Field | Meaning |
| --- | --- |
| `on_hold` | the reasons the current revision is on hold |
| `deferred` | the causes deferred while it is |

A save of a program without a hold is byte for byte as before. Restore
refuses a hold no source mechanism can produce: a series held when no rule or
procedure of the source holds it, held with no current revision, or with
deferred causes and no hold; a name in either field that is not observed
evidence, or is listed twice. A journal entry `held`, `deferred` or `released`
must be of the series' current revision, and its event must reach an effect
that can make it: `hold` for `held`, `reopen` or a trigger for `deferred`,
`release` for `released`.

A hold, a deferral and a release each journal their entry in the event that
makes them, and a reopening ends a hold (section 2). So restore also reads
each revision's `held`, `deferred`, `released` and `reopened` entries in
order, and holds them, the two fields and the revision's reopening record to
each other (D062):

- A `deferred` or a `released` entry is of a revision on hold, and no `held`,
  `deferred` or `released` entry follows its revision's `reopened`. A `held`
  entry names no reason the hold has already, and a `deferred` entry no cause
  deferred already.
- A `released` entry of a hold that deferred causes is followed directly by
  the `reopened` entry that cites those causes and no other, at the same
  event.
- Any other `reopened` entry is made by a `reopen` or a trigger of its event,
  not by a `release`. Of a revision on hold it is a withdrawal breaking
  through: the program has a `withdraw`, and the entry cites every deferred
  cause.
- No revision is committed while the one before it is on hold.
- Every reopening journals a `reopened` entry, and no entry of its revision
  follows it. So the entries of a revision that the graph shows reopened end
  with its `reopened` one.
- A series' `on_hold` and `deferred` are what its current revision's entries
  leave: the reasons held and the causes deferred since the revision was
  committed or last released. A reopened revision has neither.
- A revision's reopening record has a `held`, `deferred`, `released` or
  `reopened` entry of that revision, since each of them joins it. Each such
  entry has the record, and the record holds every cause the entry cites:
  the entry joins it with the causes' lineage and pins them, so none of them
  has departed.

A windowed journal departs oldest first
([Departure 0.1](caveat-departure-0.1.md)). While it holds a revision's
`committed` entry it holds every later entry of that revision, and they say
all of its hold; so do the entries from a `released` or a `reopened` one on.
With a revision's earlier entries departed, the ones left say only part of
its hold. Restore then requires what they do say: the series holds and defers
at least what they name, a release is followed by the reopening of at least
the causes they defer, and a `reopened` entry that is the journal's first may
be its release's. That is so only of a series that a rule or procedure of the
source holds: the entries of any other series, as of a commitment that is no
series' revision, say all there is. A reopening record needs an entry of its
revision only while the journal holds the revision's `committed` entry; each
entry the journal holds needs the record whatever has departed.

Restore still does not authenticate history. It reads what the entries say,
and not what a selector could have selected. So once a hold has ended it
accepts a journal with all of the hold's entries taken out, when another
entry of the revision accounts for its reopening record. In a program that
withdraws it also accepts a journal with a `deferred` entry taken out before
the reopening that broke through the hold: that `reopened` entry then cites
one more cause, as a `reopen` naming several does.

## 5. Check

[`caveat check`](caveat-check-0.1.md) counts a `release` as a path that
reopens a series, and a `hold` as none, for C002.

## What it does not do

- It does not hold a one-off commitment, which has no series to defer into.
- It does not hold a revision on a condition. A hold lasts from `hold` to
  `release` or to a reopening, and the program says when each happens.
- It does not stop a withdrawal reaching a revision (section 2), and it does
  not authorize anything: a decision on hold is a decision in force.
- It does not end a hold on a timer. A program that wants one schedules the
  release as it schedules any rule.

## Compatibility with other additions

- [Modules](caveat-0.5-draft.md): a module may still declare `hold` or
  `release`. The rewriter reads the verb by its place, two words before a
  `because`.
- [Foreclosure 0.1](caveat-foreclosure-0.1.md): it is about what a decision
  closes off; a hold is about when a decision is reconsidered. An irreversible
  series may be held, and a hold never lifts a foreclosure.

## Corpus sweep

`node experiments/holds-sweep/sweep.mjs` reads every tracked Caveat program
and writes [`RESULTS.json`](../experiments/holds-sweep/RESULTS.json). At this
profile's revision:

- 196 programs; none defines `fn on_hold` or uses `on_hold` as a word;
- `hold` or `release` appears as a name in 15 programs: evidence, events and
  decisions called `hold` or `release` in the game, the departure fixtures,
  the readiness example and this project's ledger. None is written two words
  before a `because` as an effect;
- 14 series declare triggers, and 132 programs have a rule that reopens a
  series: the reopenings a hold can defer.

Each of the 196 programs was also loaded with the runtime before and after
this profile, and saved: the 166 that load save byte for byte the same, and
the 30 that do not load give the same error. No accepted program changes.

## Implementation

- `runtime/src/reactive.rs` parses `hold` and `release` as forms of the
  `reopen` effect, applies them as section 2 says, and adds `on_hold` and
  `deferred` to a decision series.
- `runtime/src/reactive_expr.rs` parses `on_hold(SERIES)`.
- `runtime/src/reactive_save.rs` restores and validates them as section 4
  says; `runtime/src/reactive_departure.rs` pins a hold's names.
- `runtime/src/link.rs` protects the verbs by position in a module.
- `runtime/src/reactive_check.rs` reads them for C002.
- `runtime/tests/holds.rs` covers a deferred reopening and its release, a
  release with nothing deferred, a withdrawal reaching a revision on hold, a
  hold and release made by procedure steps, a
  save with a hold, a save without one, a forged hold refused on restore, a
  hold of a one-off commitment, a program's own `fn on_hold`, and a hold
  keeping windowed readings through their retirement.
- The kit's types name the new journal changes and series fields.

## Changes

- 2026-10-19: specified and implemented for 1.2.
- 2026-10-25: D021 fixed. A reopening with several causes reopened the
  decision, and journaled it, in the order the causes arrived: a withdrawal
  breaking through a hold before the reopenings it deferred, and a release in
  the order the hold deferred them. The text above and the decision journal
  say the order they were first observed, and restore checks that order, so a
  save made after such a reopening did not restore on 1.2.0 to 1.3.0. The
  runtime now orders the causes first, so the graph's relations, the effects
  and the journal entry follow observation order. Only reopenings whose causes
  arrived out of that order change. Tests: `runtime/tests/d021_journal_order.rs`.
- 2026-11-12: D062 fixed. Restore held each series and each journal entry to
  the program, and not to each other, so it accepted a save whose journal
  left out the `held` entry of a hold in force, the `deferred` entry of a
  cause still deferred, or the `released` entry of a hold that had ended.
  Section 4 now says what a revision's entries, its series, its reopening
  record and the graph's reopening of it must agree on, and what a windowed
  journal leaves unsaid, which is nothing for a series no rule or procedure
  holds. The check runs after every other, so a save refused before is
  refused with the same message, and a save the runtime wrote restores as
  before. Tests: `runtime/tests/d062_hold_entries.rs`.
