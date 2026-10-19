# CAVEAT In Basis 0.1 (draft) — asking whether evidence is in a decision's basis

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.1: implemented on `main`, not yet in a release. It becomes stable when a 1.x release ships it. It adds to
[Past Revisions 0.1](caveat-past-revisions-0.1.md),
[Latest 0.1](caveat-latest-0.1.md) and
[Reading Index 0.1](caveat-reading-index-0.1.md); it changes no program that
loads today.

A decision keeps its grounds: the evidence it was made on.
`rests_on_withdrawn(SERIES)` asks whether any of them is withdrawn, but nothing
asked whether one given report is among them. When a report arrives or is
found wrong, an author wants to know whether it changes the decision in force:
did the decision rest on it, or on something else? Without a way to ask, an
author kept a state per report, set beside each commit, to remember what the
decision used. Before this profile the question did not load:

```text
basis.cav does not load: unknown source function in_basis
```

(`caveat-lang@1.0.2`, `caveat check`, on the program below.) This profile asks
it directly:

```caveat
claim passing;
evidence runner from "the test runner";
readings checks from runner limit 8;
decisions ship limit 8;
state second_counts = 0;
event report v min 0 max 1;
event decide;
event ask;
on report sample checks = v supports passing;
on decide commit ship because enough using latest(checks);
on ask set second_counts = if(in_basis(ship, latest(checks)), 1, 0);
```

After `report, decide, report, ask`, `second_counts` is 0: `ship@1` rests on
`checks@1`, and the current reading is `checks@2`.

## Syntax

```text
in_basis(DECISION, REFERENCE)
in_basis(revision(SERIES, K), REFERENCE)
```

`DECISION` is a decision series or a commitment, and `SERIES` a decision
series; `K` is a numeric expression. `REFERENCE` is one of:

- `EVIDENCE`, declared evidence: its current occurrence, as `observed(EVIDENCE)`
  reads it;
- `latest(NAME)`: a stream's current reading, or the current occurrence of
  declared evidence, as in [Latest 0.1](caveat-latest-0.1.md);
- `occurrence(EVIDENCE, N)` or `reading(STREAM, N)`: the held occurrence or
  reading that `N` names, as in
  [Earlier Occurrence 0.1](caveat-earlier-occurrence-0.1.md) and
  [Reading Index 0.1](caveat-reading-index-0.1.md).

It is a boolean expression and stands wherever one does. A name that is not
a decision, a `revision` of something that is not a decision series, or a
reference its profile does not accept does not load.

A 1.0 program may define a source function named `in_basis`. Where it does,
every `in_basis(…)` in it calls that function, as in 1.0.

## Meaning

`in_basis(DECISION, REFERENCE)` asks about the decision's current revision,
the one `rests_on_withdrawn(DECISION)` reads: for a series its current
revision, reopened or not, and for a commitment the commitment once made.
`in_basis(revision(SERIES, K), REFERENCE)` asks about `SERIES@K`, the `K`th
revision, from 1, current or past.

The answer is true when that revision's grounds hold the evidence the
reference names now. Grounds are what the decision was made on: the evidence
its `using` value or its citations rest on, as recorded when it was made
([Explanations 0.2](caveat-explanations-0.2.md#grounds-of-a-commitment),
[Citing 0.1](caveat-citing-0.1.md)). They do not change after, so a
later reading, a withdrawal or a take-back never adds to them or removes from
them.

The answer is false, and nothing is refused, when the decision has not been
made, `K` names no revision made so far, or the reference names nothing held:
a stream with no reading, or an index naming no held occurrence.

The answer rests on the revision's record: its lineage and grounds are those
`committed(…)` asked of that revision gives. A false answer about a revision
not made carries none. `K` and `N` are evaluated whatever they name, as an
index is wherever it appears.

`in_basis(revision(SERIES, K), …)` keeps every revision's grounds of `SERIES`,
as `rests_on_withdrawn(revision(SERIES, K))` does
([Past Revisions 0.1](caveat-past-revisions-0.1.md)).

## Snapshot, save and views

The form adds no field and no refusal code, and changes no snapshot, view or
save of a program that does not use it. A program that asks about a past
revision keeps that series' revision grounds, which its save holds as Past
Revisions 0.1 already states.

`caveat check`'s C008 does not report `in_basis(…)`: it warns where 1.1 reads
a record differently from 1.0.x, and no 1.0.x runtime loads the form
([Check 0.1](caveat-check-0.1.md)).

## Corpus sweep

No tracked program writes `in_basis` or defines a function of that name;
before this profile the form did not load. The save sweep
(`runtime/examples/save_sweep.rs`, 8 runs of 150 seeded events) over all
tracked programs gives the same digest for every program as `main` before this
change.

## Implementation

The parser reads `in_basis(` as the form, and as a call when what follows does
not parse as one; expansion turns the form back into a call when the program
defines `in_basis`. The host resolves the revision with `held_revision` and the
reference as the selectors do, then looks the evidence up in the revision's
recorded grounds. Tests: `runtime/tests/in_basis.rs`.

## Changes

- 2026-10-19: drafted and implemented for 1.1.
