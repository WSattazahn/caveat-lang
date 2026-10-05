# CAVEAT Lineage Compaction 0.1 — what a record leaves behind when it departs

Status: specified for rc.16, not implemented. It is the rule that
[departure](../docs/design/save-forgetting.md) and series windows build on
(owner, 2026-10-05 14:45 UTC). Its marker shape is the recommendation below and
is awaiting the owner's choice; see [Decision](#decision).

[Windows 0.1](caveat-windows-0.1.md) lets a history retire its oldest records.
A retired record leaves the live graph but stays in the session, so a
windowed program's save still grows. Departure, the second half of the
owner's 2026-10-04 decision, lets a retired record leave the session. Under
the [save-forgetting](../docs/design/save-forgetting.md) plan, a record could
depart only when nothing cited it. Round 7's C3 program showed that this
never happens for a record that a value has read.

This profile replaces that pinning rule. When a record departs, every lineage,
grounds, basis and journal entry that named it keeps a **departure marker**
for its history in place of the name. Citations no longer keep a record in the
session, and `explain` still says what a value rested on. When a retired
record departs, and the archive the host drains, are the departure
specification's to define. This profile defines what every citation keeps.

**A program without a window is unchanged**: no record of it ever departs, so
no marker ever appears, and its lineages, grounds, snapshots, saves, views and
scenario outcomes are exactly as before.

## The finding

Measured on rc.15 (`caveat-lang@0.1.0-rc.15`, the published package) with
round 7's C3 program (`experiments/glowcap/round7/runs/C3/impl/`), given
`renewable … window 2` on its fifteen renewable evidences and `journal
window 6`, under `longplay.mjs`'s 60 cycles:

- The adapter's save was 4,582, 11,870 and 22,182 bytes after 10, 30 and 60
  cycles, as the [rc.15 development record](../docs/releases/rc15-development.md)
  reports. The raw save at 60 cycles was 226,589 bytes, of which 22,773 were
  the `retired` map naming 1,030 retired records.
- Its 12 journal-slot states (`j1_code` to `j6_cite`) each held 295 retired
  occurrence names among 300 in their lineage. A skipped rule adds its guard to
  the target's lineage ([Explanations 0.2](caveat-explanations-0.2.md)), and
  every journal shift passes the slot's lineage on, so each slot's lineage
  holds every observation the program ever made. Those names were 63,077 of
  the 66,464 bytes of `states`.

A rule that departs records but keeps their names in lineages cannot make this
save flat. The names, not the records, are what grow.

## Departure markers

A departure marker stands for the departed records of one history that a
provenance named:

```json
{"history": "witness_cave", "from": 1, "through": 59, "departed_at": 412}
```

- `history` is the windowed history's name: a reading stream, a renewable
  evidence, a decision series once series windows exist, or `journal`.
- `from` and `through` are the lowest and highest occurrence numbers among
  the departed records the provenance named. A history's first occurrence is
  number 1 (`witness_cave` is `witness_cave@1`).
- `departed_at` is the sequence of the event at which the latest of them
  departed.

A provenance (a lineage, grounds, a frozen basis, a series' selection or
reopening qualifications, a journal entry's `because` or basis, a withdrawal
record, a permission record) holds **at most one
marker per history**, beside its exact names. Its exact names are the records
still in the session, live or retired, as before. Its markers are everything
it named that has departed.

**Departure.** When a record departs, every provenance that names it drops
the name and joins it into its marker for that history. If it has no marker
for that history yet, it gets one with `from` and `through` set to the
record's number. Otherwise `from` becomes the lower of the two numbers and
`through` the higher, and in both cases `departed_at` becomes the departing
event's sequence. A record departs in exactly one event, so every provenance
that named it is compacted in that same event.

**Combining provenance.** Where a read combines provenance (a union of
lineages, a state read in an expression, a revision's basis taking its
predecessor's), markers for the same history join by `from = min`,
`through = max` and `departed_at = max`. The join is associative, commutative
and idempotent, so a combined provenance does not depend on evaluation order,
and incremental evaluation ([Incremental evaluation
0.1](caveat-incremental-evaluation-0.1.md)) stays exact.

**Meaning.** A marker says the value read **some** of the history's records
numbered `from` to `through`, all of which have departed. It does not say which
ones. Lineage answers *what could have influenced this?*, so naming a range
that may include records the value never read over-approximates and stays a
correct answer. Grounds narrow lineage. A grounds marker is within the
lineage's marker for the same history: its `from` is not lower, its `through`
is not higher, and its `departed_at` is not later. So **grounds ⊆ lineage**
still holds ([Explanations 0.2](caveat-explanations-0.2.md)).

**Caveats.** A provenance's caveats are names of declared caveats, a set that
cannot grow with play. They are kept exactly. A departed record's
qualifications stay in every provenance that carried them.

**What does not depart.** A record that the session must still name exactly
does not depart, whatever cites it: a live record, the current occurrence of a
renewable evidence, the latest reading of a stream, a record with a scheduled
qualification still pending, and a commitment that is the current entry of its
series. When such a record departs later, the markers form then.

**Bounded.** The marker count of a provenance is at most the number of
windowed histories in the program, and a marker's size depends on numbers, not
on how many records it covers. A windowed history keeps at most `N` live
records plus the retired records that have not yet departed. In a program
where every growing history has a window, therefore, nothing in the session
grows with the length of play except the digits of sequence and occurrence
numbers.

## Explain, dependents and the view

- **`explain`** shows each marker after the exact names, as "departed
  `witness_cave@1` to `@59`, the last at 412". When `from` equals `through`,
  it shows the one name. Exact names are shown as before, and retired names
  keep `retired_at`.
- **`dependents`** of a departed record cannot be answered from the session,
  which no longer holds it. Given a name numbered at or below the departed count of
  its history, it marks the name departed and lists the provenances whose marker
  for that history covers its number, as "may rest on". This answers "what
  might have read it".
- **`caveat-reactive-view/0.1`** is unchanged for programs without a window. A
  windowed program's view gains markers only through View 0.2, which follows
  this profile and defines where a marker sits in ordered grounds. A marker
  takes the position of the first name it absorbed.

## Save and restore

A save writes a provenance's markers under `departed`, beside `evidence` and
`caveats`. The field is absent when there are none. Departure replaces the
`retired` entry of a departed record with a per-history departed count, so
the history's numbering continues (`STREAM@(departed + K)`). The
departure specification defines that field. Saves of programs without a window
never have either, and the schema stays `caveat-reactive-save/0.1`. A runtime
that predates this profile refuses a save with `departed`, as it refuses any
field the schema lacks.

Restore refuses a save when:

- a marker names a history without a window, or one the program does not
  declare;
- a marker's `from` is below 1, its `from` is above its `through`, or its
  `through` is above the history's departed count;
- a marker's `departed_at` is 0 or past the save's `sequence`;
- a provenance holds two markers for one history, or names exactly a record
  its history has departed;
- a grounds marker is not within the lineage's marker for its history, or a
  commitment's grounds marker is not within its basis's.

As with every restore check, a save that passes holds records the program
could hold, not records events are shown to have produced
([Save 0.1](caveat-save-0.1.md)). Restore never authenticates history.

## Series revisions and the journal

**Series revisions.** Since [Reactive 0.5](caveat-reactive-0.5.md), a
revision's basis holds its predecessor's basis, so the series chain is
another lineage that grows with play. Series windows, which follow this
profile, retire and depart revisions. A later revision's basis then holds one
marker for the series, such as `trust@1` to `@200`, and markers for the
histories its predecessors read. What a revision rests on is the same rule
for every series, windowed or not. Only which records are still in the session
depends on the window, and that is exactly what a marker reports.

**The journal.** Under `journal window N`, a retired journal entry departs by
the same rule. An entry still in the window keeps its `because` and basis,
with markers for the occurrences that have departed. Nothing else cites
journal entries.

## Gate

The registered fixture decides when departure is done: round 7's C3 program
with windows on its renewable evidences and its journal, under
`longplay.mjs`'s 60 cycles. Its adapter save stays flat: the bytes at 60
cycles are within 5% of those at 30, the allowance for numbers gaining digits.
Smaller is not enough.

`experiments/lineage-compaction/simulate.mjs` simulates this over rc.15's
real C3 saves. It treats every retired record as departed, replaces departed
names with markers in every provenance in the save and drops the departed
records. Its byte counts are of compact re-serialized JSON, slightly smaller
than the runtime's text:

| Save bytes (JSON as re-serialized) | 10 cycles | 30 cycles | 60 cycles |
| --- | ---: | ---: | ---: |
| rc.15 with windows | 39,126 | 113,758 | 225,929 |
| one marker per departed record | 37,296 | 102,894 | 201,595 |
| one marker per history (this profile) | 13,846 | 13,995 | 13,996 |

Under this profile's markers, every part of the save is the same size at 30
and 60 cycles, give or take a digit: `states` 7,714 bytes, `graph` 2,347, the
journal 1,074. The `trust` series has one revision in this play, so C3 needs
no series window. Its retired names sat in the series' selection and
reopening qualifications, which are provenances like any other. This is a
simulation over saves, not a runtime measurement, and the gate is measured on
the runtime.

For every program without a window, the implementation's gate is the seeded
save sweep (`runtime/examples/save_sweep.rs`) over the repository's reactive
programs, with digests identical before and after.

## Decision

The 2026-10-05 14:45 UTC direction named a marker per record: "the record's
name, its occurrence number and the sequence at which it retired". The C3
finding shows that a marker per record keeps the growth, because the growing
part is the lineage's list of names, not the records. This profile therefore
recommends one marker per history, which loses only which records inside a
departed range a value read. The records themselves go to the archive the
host drains, which the departure specification defines.

A third shape keeps exact markers for the newest `N` departed records of each
history and folds older ones into a range. It is also bounded, and it also
joins: any record among the newest `N` of a union is among the newest `N` of
one side, so the combined form is a function of the union. One marker per
history is recommended over it because it needs one marker form, not two, and
no new constant `N`. The owner's choice among the three is recorded here
before the departure implementation begins.

## Changes

- 2026-10-05 (rc.16, specified): new profile. Departure markers in place of
  departed names in every provenance, joined per history; their meaning,
  bound, `explain` and `dependents` forms, save field and restore checks; series
  revisions and journal entries under the same rule; the C3 gate and the
  simulation behind the marker shape. Not implemented.
