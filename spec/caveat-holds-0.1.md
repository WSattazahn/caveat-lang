# CAVEAT Holds 0.1 (draft) — keeping a decision in force for a while

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.2: implemented on its branch, not yet on `main` or in a release. It becomes stable when a 1.x release ships it. It adds to
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
- Foreclosure, a 1.2 draft on its own branch: it is about what a decision
  closes off; a hold is about when a decision is reconsidered. They do not
  interact.

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
