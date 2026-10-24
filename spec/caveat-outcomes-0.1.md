# CAVEAT Outcomes 0.1 (draft) — recording how a decision turned out

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.3: implemented on its branch, not yet on `main` or in a release. It becomes stable when a 1.x release ships it. It adds to
[Reactive 0.5](caveat-reactive-0.5.md),
[Past Revisions 0.1](caveat-past-revisions-0.1.md),
[Withdrawal 0.1](caveat-withdrawal-0.1.md) and the
[Decision Journal 0.1](caveat-decision-journal-0.1.md); it changes no
program that loads today.

Caveat keeps what each decision was based on, frozen when it was made. That
is half of what it takes to judge a decision apart from how it turned out
(Annie Duke calls the mistake "resulting"). The other half is the outcome,
recorded against the decision it judges, so that a review can tell a sound
basis that met bad luck from an unsound one.

Before this profile, an outcome was a reading in a stream of its own, and
nothing linked it to the revision it judged
([prototype](../experiments/outcomes-sweep/README.md#the-prototypes)):

```caveat
readings outcomes from usage_feed limit 8;
on close sample outcomes = users supports pilot_working;
```

The host paired `outcomes@1` with `pilot@1` by hand. The reading had to
claim a stance it did not have, because an outcome is not evidence for or
against the claim the decision was about: it is what happened after. And the
premortem's risk that came true could only be marked by qualifying the
plan's evidence with a second caveat, which changed what every later reading
of that evidence carried.

This profile records the outcome on the revision:

```caveat
decisions pilot limit 4;
on close resolve pilot = users because latest(usage) materialized thin_demand;
```

`pilot@1` now holds its outcome, 15, because of `usage@2`, and records that
its retained caveat `thin_demand` came true. Its basis, its caveats and its
status are what they were. For the pilot in
[the tests](../runtime/tests/outcomes.rs), after a plan scored 7, a week of 12
users and a close at 15, `explain` prints:

```text
    pilot@1 = 7  reopened
      based on plan_review (caveats: thin_demand)
      plan_review: observed at #1 plan {"score":7}
      #1 plan: committed because plan_review (caveats: thin_demand)
      #2 weekly: reopened because usage@1
      #3 close: resolved = 15 because usage@2; materialized: thin_demand
```

## 1. Syntax

```text
resolve SERIES = VALUE because SELECTOR [materialized CAVEAT, CAVEAT, …]
resolve revision(SERIES, INDEX) = VALUE because SELECTOR [materialized CAVEAT, CAVEAT, …]
resolved(SERIES)
resolved(revision(SERIES, INDEX))
outcome(SERIES)
outcome(revision(SERIES, INDEX))
```

`resolve` is an effect, written wherever `reopen` is: in a rule, after its
guard, or as a step of a [procedure](caveat-procedure-symbols-0.1.md).

- `SERIES` must be a decision series (`decisions SERIES limit N`).
- `revision(SERIES, INDEX)` names a revision as
  [Past Revisions 0.1](caveat-past-revisions-0.1.md) does: revision 1 is the
  first committed, and `INDEX` is a numeric expression.
- `VALUE` is a numeric expression.
- `SELECTOR` is any selector `reopen` takes: an evidence name, an
  occurrence, `latest(STREAM)` or `caveated(STATE, CAVEAT)`.
- Each `CAVEAT` is a declared caveat, named at most once.

`resolved(…)` is a predicate, written wherever `reopened(SERIES)` is.
`outcome(…)` is a number, written wherever `latest(SERIES)` is. Each takes a
series or a `revision(…)` selector, which before this profile stood only
inside `rests_on_withdrawn`.

These do not load:

| Source | Message |
| --- | --- |
| `resolve go = 1 because e;` where `go` is not a series | `resolve names a decision series; go is not one` |
| `resolved(go)` where `go` is not a series | `resolved names a decision series; go is not one` |
| `outcome(go)` where `go` is not a series | `outcome names a decision series; go is not one` |
| `materialized c` where `c` is not a caveat | `unknown caveat c` |
| the same caveat twice after `materialized` | `duplicate materialized caveat c` |

`resolve` is an effect word only when it is followed by a name and `=`, or
by `revision(`. `resolve`, `resolved`, `outcome` and `materialized` stay
legal names: a state, event or stream may be called any of them, and a
program that defines its own `fn resolved` or `fn outcome` keeps calling it,
as a 1.0 program that defines `fn in_basis` does.

## 2. Meaning

### An outcome

An outcome is a fact about the world after a decision: a number, the
evidence it was read from, and the caveats of the decision that it shows
came true. It is not evidence for or against a claim, so it takes no stance.

`resolve TARGET = VALUE because E materialized C, …` gives the revision
`TARGET` names an outcome:

- **value**: `VALUE`, evaluated as the effect runs;
- **because**: the evidence `E` names, in observation order, as `reopen`
  would cite it;
- **materialized**: the caveats named, in source order;
- the event's sequence and name;
- **lineage**: `VALUE`'s lineage, `E`'s lineage and the effect's guard,
  frozen, as a reopening's basis is.

`resolve SERIES` names the series' current revision. `resolve
revision(SERIES, K)` names revision `K`, current or superseded: outcomes
often arrive after the next decision has been made.

### What resolving changes, and what it does not

Resolving records the outcome on the revision and adds one entry to the
decision journal (section 3). Nothing else changes:

- The revision's value, basis, grounds and retained caveats stay as they
  were. An outcome never rewrites a basis and never drops a caveat.
- Resolving does not reopen the revision, and does not commit, hold or
  release one. A revision in force stays in force; a reopened one stays
  reopened. `committed`, `reopened`, `on_hold` and `rests_on_withdrawn`
  answer as before.
- Resolving qualifies nothing. `materialized` records that a caveat came
  true for this revision; it adds no relation to the caveat or to any
  evidence, so later readings of the evidence carry what they carried.

A rule that wants a bad outcome to reopen a decision, or to change a
policy, says so with its own effects, as it would for any other reading.

### When a resolve is refused

The event is refused, and records nothing, when:

| The `resolve` | Origin and code | Message |
| --- | --- | --- |
| names a series with no revision, or an `INDEX` that is not a whole number from 1 to the series' revision count | `evaluation/not_committed` | `cannot resolve pilot: it has no revision 3` |
| names a revision that already has an outcome, unless the outcome is withdrawn (below) | `evaluation/already_resolved` | `pilot@1 is already resolved at #3; withdraw its evidence to correct it` |
| names a caveat after `materialized` that the revision does not retain | `evaluation/not_retained` | `pilot@1 does not retain late_launch, so it cannot have materialized` |
| cites evidence that is not observed, or `latest` of a stream with no reading | `evaluation/unobserved_evidence` | as for `reopen` |
| cites `caveated(STATE, CAVEAT)` and finds nothing | `evaluation/empty_caveated_selection` | as for `reopen` |

`VALUE` and `INDEX` fail as any expression fails, with
`evaluation/expression`. The caveats a revision retains are those `explain`
lists after `retaining:` for it: its commitment's retained caveats.

`already_resolved` and `not_retained` are new codes. They are allowed
because the syntax is new ([compatibility](../docs/COMPATIBILITY.md),
"Rejection origins and codes").

### Correcting an outcome

An outcome is not overwritten. A wrong one is corrected as wrong evidence
is: by withdrawing the evidence it was read from
([Withdrawal 0.1](caveat-withdrawal-0.1.md)). An outcome is **withdrawn**
when some evidence in its `because` is withdrawn now.

- A withdrawn outcome is still the revision's outcome. `resolved(…)` stays
  true, and `outcome(…)` reads its value with the `withdrawn` caveat, as
  `history_at` reads a withdrawn reading.
- A `resolve` of a revision whose outcome is withdrawn is accepted, and its
  outcome replaces the withdrawn one. The journal keeps the entries of both.

### Reading an outcome

- `resolved(SERIES)` is true when the current revision has an outcome.
  `resolved(revision(SERIES, K))` is true when revision `K` has one; an
  `INDEX` that names no revision gives false, as in `rests_on_withdrawn`.
  When true, its grounds are the outcome's lineage.
- `outcome(SERIES)` is the current revision's outcome value, and
  `outcome(revision(SERIES, K))` revision `K`'s. Its lineage is the
  outcome's frozen lineage, with each `because` evidence's caveats as they
  are now, so a later qualification or withdrawal of that evidence reaches
  it ([Late Qualification 0.1](caveat-late-qualification-0.1.md)). A series
  or revision with no outcome refuses the event as `evaluation/expression`,
  as `history_at` past a history's records does: guard it with `resolved`.

A revision's own value is read as before: `latest(SERIES)` for the current
one, `history_at(SERIES, K - 1)` for revision `K`. Together they pair a
forecast with what happened, which is what calibration needs:

```caveat
claim canary_passes;
evidence forecaster from "release agent";
evidence canary from "canary monitor";
evidence scorer from "Brier score of a resolved forecast";
readings canary_runs from canary limit 64;
readings errors from scorer limit 64;
decisions forecast limit 64;
event predict p min 0 max 1;
event report passed min 0 max 1;
on predict reveal forecaster supports canary_passes;
on predict commit forecast because enough using qualified(p, forecaster);
on report sample canary_runs = passed;
on report resolve forecast = passed because latest(canary_runs);
on report sample errors = (latest(forecast) - outcome(forecast)) * (latest(forecast) - outcome(forecast));
```

The host no longer pairs `forecast@N` with a reading. Each error rests on
the forecast it scores and the run that resolved it.

## 3. Journal

A program with a `resolve` records one more change in the
[decision journal](caveat-decision-journal-0.1.md), of the revision
resolved:

| `change` | `because` | more fields |
| --- | --- | --- |
| `resolved` | the outcome's evidence | `outcome`: the value; `materialized`: the caveats named, left out when none |

`value` is the revision's own frozen value, as on its other entries, and
`caveats` are those of the outcome's evidence at that moment. A program
without a `resolve` records only what it recorded before.

## 4. Save

A revision in a save carries one more field, left out when it has no
outcome:

| Field | Meaning |
| --- | --- |
| `outcome` | `value`, `because`, `materialized` (left out when empty), `sequence`, `event` and the frozen `lineage` |

A save of a program without a `resolve` is byte for byte as before. Restore
refuses an outcome no source mechanism can produce: an outcome on a revision
of a series that no rule or procedure of the source resolves; a `because`
name that is not observed evidence, or is listed twice; a `materialized`
caveat the revision does not retain, or listed twice; a `sequence` after the
save's own, or before the revision's commit. A journal entry `resolved` must
be of a revision of the series, and its event must reach a `resolve` of it.
Restore does not check that the value is the one the event computed: a save
binds its source, not its history ([A3](../docs/RESTORE_TRUST_BOUNDARY.md)).

The outcome's evidence stays pinned against
[departure](caveat-departure-0.1.md) while the revision holds it, as a
hold's reasons are. A series has no window and a revision holds at most one
outcome, so the pins are bounded by the series' `limit`.

## 5. Views and explain

- `caveat-reactive-view/0.1` and `/0.2` show the journal entries and the
  revision field above; a program without a `resolve` shows exactly what it
  showed before.
- `caveat explain` prints the journal entry under the revision, as in the
  example at the top. `caveat-explain/0.1` gains an optional `outcome` on a
  revision, left out when it has none.

## What it does not do

- It does not resolve a one-off commitment. An outcome judges a revision of
  a series, so a decision made once is declared `decisions NAME limit 1`.
- It does not take a named result (`pass`, `fail`). An outcome is a number,
  as a reading and a `using` value are; a program names results with
  numbers, as it names choices today.
- It does not fold over outcomes. A program scores each outcome as it
  arrives, as in the example above, and folds the scores with
  `fold_history`. A fold over revisions paired with their outcomes may
  follow, once a program needs one.
- It does not list, under `caveat dependents`, the revisions where a caveat
  materialized. The journal and `explain` show it; `caveat-dependents/0.1`
  is unchanged.
- It does not judge the decision. It keeps the basis and the outcome side by
  side, so that a person, or a later profile, can.

## Corpus sweep

`node experiments/outcomes-sweep/sweep.mjs` reads every tracked Caveat
program and writes [`RESULTS.json`](../experiments/outcomes-sweep/RESULTS.json).
At this profile's revision:

- 197 programs; none defines `fn resolved` or `fn outcome`, and none writes
  `resolve` followed by a name and `=` or by `revision(`;
- `outcome` is a name in 8 programs: an event `outcome` in the six
  agent-authoring v1 runs, and a state `outcome` in the trail-rescue game.
  None calls `outcome(`;
- `resolve` appears in one program, `game/the_last_beacon.cav`, as the
  game format's top-level `resolve ACTION as RESULT when …` directive. It is
  not a rule effect and is followed by `as`, not `=`;
- 147 decision series in 130 programs: the revisions an outcome can judge.

## Implementation

- `runtime/src/reactive.rs` parses `resolve` as an effect only in the
  positions section 1 names, records the outcome on the revision and the
  `resolved` journal entry, and refuses as section 2 says. `select_because`
  is the selector `reopen` already read, shared.
- `runtime/src/reactive_expr.rs` parses `resolved(…)` and `outcome(…)`, and
  hands a program's own `fn resolved` or `fn outcome` its calls.
- `runtime/src/reactive_save.rs` restores and validates outcomes as section
  4 says; `runtime/src/reactive_departure.rs` pins an outcome's evidence.
- `runtime/src/link.rs` protects `resolve` and `materialized` by position in
  a module.
- `runtime/tests/outcomes.rs` covers an outcome that changes nothing else, a
  refused second outcome and its correction after a withdrawal, refusals for
  a missing revision and an unretained caveat, a late outcome, reading an
  outcome that is not there, the load errors, a program's own names, saves
  with and without an outcome, forged outcomes refused on restore, a
  withdrawn outcome's caveat, the calibration example, and a procedure
  resolving in a windowed program.
- The kit's types, `explain` and the starter's assessment carry the new
  journal change and revision field.

## Changes

- 2026-10-24: drafted and implemented for 1.3.
