# CAVEAT Past Revisions 0.1 (draft) — asking whether a past revision rests on withdrawn evidence

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.1: implemented on `main`, not yet in a release. It becomes stable when a 1.x release ships it. It adds to
[Withdrawal 0.1](caveat-withdrawal-0.1.md),
[Departure 0.1](caveat-departure-0.1.md) and
[Earlier Occurrence 0.1](caveat-earlier-occurrence-0.1.md); it changes no
program that loads today.

A decision series keeps every revision: what each was committed on stays in
its frozen grounds. `rests_on_withdrawn(D)` asks whether the current
revision's grounds hold withdrawn evidence. Nothing asks it of an earlier
revision. A team that withdraws a report found wrong after the fact wants
every ship decision made on it, current or not, so someone can review what
went out. Before this profile a program could not say which: both authors in
the second usability pilot packed each decision's basis into numbers,
bitmasks or a hex digit per decision, and compared them with a hand-kept
record of withdrawals. A host can join `decisionHistory` with the
session's withdrawals, but only while the decision journal still holds the
revision's entry; a windowed journal retires it. This profile adds one
selector:

```caveat
claim passing;
evidence tests from "test runner";
renewable tests limit 8;
evidence operator from "the operator's correction";
decisions ship limit 8;
state answer = 0;
event first_report;
event report;
event decide;
event correct n min 1 max 8;
event ask k min 1 max 8;
on first_report reveal tests supports passing;
on report renew tests;
on report reveal tests supports passing;
on report when committed(ship) and not reopened(ship) reopen ship because tests;
on decide commit ship because enough using qualified(1, tests);
on correct when not observed(occurrence(tests, n)) reject "no such report";
on correct reveal operator;
on correct withdraw occurrence(tests, n) because operator;
on ask set answer = if(rests_on_withdrawn(revision(ship, k)), 1, 0);
```

After `first_report, decide, report, decide, report, decide`, the series
holds `ship@1` on `tests`, `ship@2` on `tests@2` and `ship@3` on `tests@3`.
`correct {"n": 2}` withdraws `tests@2`. Asked then, revision 2 answers 1 and
revisions 1 and 3 answer 0. `rests_on_withdrawn(ship)` answers 0, because
the current revision, `ship@3`, rests on `tests@3`.

## Syntax

```text
rests_on_withdrawn(revision(SERIES, INDEX))
```

`SERIES` is a declared decision series (`decisions SERIES limit N`). A
commitment outside a series, or a name that is not a decision, does not
load:

```text
revision(once, …) names no decision series
```

`INDEX` is a numeric expression: a literal, a state, an event parameter or
anything computed from them. Revision 1 is the first committed, and revision
`k` is the one a snapshot names `SERIES@k`: the `k`th entry of
`decision_series.SERIES.revisions`, oldest first, and the `k`th revision the
kit's `decisionHistory` lists, whether or not the journal has retired its
entries. A series has no window, so every revision it made is held and keeps
its number.

The selector stands only inside `rests_on_withdrawn`. It is not an
expression, and `committed`, `reopened`, `withdrawn` and the other
predicates do not take it; a program that writes one does not load.

## Meaning

`rests_on_withdrawn(revision(D, K))` is true when `K` is a whole number from
1 to the series' revision count and some evidence in revision `D@K`'s
frozen grounds is withdrawn now. It reads the same grounds, the same
withdrawals and the same rule as `rests_on_withdrawn(D)` reads for the
current revision ([Withdrawal 0.1](caveat-withdrawal-0.1.md)), so for the
current revision the two answer alike.

- An `INDEX` that is not a whole number, is below 1, or is above the
  number of revisions made so far names no revision, and the predicate is
  false. It does not refuse the event: like `occurrence(E, I)` in a
  predicate, an index that names nothing is an ordinary operand.
- When true, the answer carries each withdrawal's reason into the reading's
  lineage and grounds, as `rests_on_withdrawn(D)` does.
- Asking changes nothing. A revision's grounds, the journal and every
  withdrawal stay as they are; a past revision is never reopened, and its
  grounds never change.

Whether a revision was a ship or a hold is its value, read as before with
`history_at(D, K - 1)`.

## Departure

A withdrawal stands while its subject is held, and leaves when its subject
departs ([Departure 0.1](caveat-departure-0.1.md)). Departure pins the
current revision's grounds and the `because` of each journal entry still in
the journal's window, not a superseded revision's grounds. In a program with
windows, a past revision's evidence could therefore retire, depart, and take
its withdrawal with it, and `rests_on_withdrawn(revision(D, K))` would turn
false with no event about `D`. That is the change
[D017](../experiments/agent-decisions/README.md) ruled out for the current
revision.

So a series that any rule or binding reads through `revision(D, …)` pins the
grounds of **every** revision, not only the current one. The bound is the
series' `limit`, a number the source fixes; it cannot grow with play.
Departure 0.1's pins list gains this item. A series no rule or binding reads
through `revision(…)` keeps today's pins exactly, so every program that
loads today departs the same records at the same events.

## Snapshot, save and views

The selector adds no field and no refusal code.

- `caveat-reactive-save/0.1` is unchanged, and every 1.0 save restores as
  before. A save from a program that reads past revisions holds the same
  kinds of records, kept longer. Restore rebuilds its pins from the source,
  as it does today.
- `caveat-reactive-view/0.1` and `/0.2` and `caveat-explain/0.1` are
  unchanged.

## Not in this profile

- `committed`, `reopened` and `permission_withdrawn` of a past revision. A
  past revision is superseded by definition, and `history_at` reads its
  value; whether a past revision's permission grant was later withdrawn may
  follow.
- A list-valued answer. Values are numbers, so a program asks about each
  revision by index, as a host or a binding per revision does.
- Naming a revision in an effect. A past revision cannot be reopened or
  withdrawn; it is history.

## Corpus sweep

No tracked program writes `revision(`; before this profile it did not load.
The save sweep (`runtime/examples/save_sweep.rs`, 8 runs of 150 seeded
events) over all 194 tracked programs, 164 of which load as reactive
programs, gives the same digest for every program on the runtime before this
change (main `7c53f5b`) and after it: no program that loads today changes an
outcome, a save or a restored save. The departure gate in CI holds the same
programs to rc.15.

## Implementation

`rests_on_withdrawn(revision(D, K))` parses to the same selector node as
`occurrence(E, I)`, with kind `revision:rests_on_withdrawn`. The session
resolves `D#K` to `D@K` when `K` names a revision, and asks
`rests_on_withdrawn` of that revision's grounds; otherwise the answer is
false. The series a program reads this way are fixed at load, and departure
counts every revision's grounds as a pin for them. Tests:
`runtime/tests/past_revisions.rs`, and in `kit/test/decision-history.test.mjs`
the case whose revisions the journal retired.

## Changes

- 2026-10-18: drafted and implemented for 1.1, closing the first pilot gap
  ("no question over decision history").
