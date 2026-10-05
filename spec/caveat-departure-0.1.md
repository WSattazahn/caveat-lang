# CAVEAT Departure 0.1 — when a retired record leaves the session

Status: specified for rc.16, not implemented. It is the second half of the
owner's 2026-10-04 decision ([save forgetting](../docs/design/save-forgetting.md),
option A, meaning 3), after [Windows 0.1](caveat-windows-0.1.md) and
[Lineage Compaction 0.1](caveat-lineage-compaction-0.1.md). The order is the
owner's (2026-10-05 16:40 UTC): lineage compaction, then departure, then the
C3 gate on a runtime build.

Windows 0.1 retires a windowed history's oldest records: they leave the live
graph and stay in the session. Lineage Compaction 0.1 says what every citation
keeps once a record has departed. This profile says **when** a retired record
departs, what the session reports, and the **archive** the host drains.

**A program without a window is unchanged**: nothing of it retires, so nothing
departs, the archive stays empty, and its snapshots, views, saves and scenario
outcomes are exactly as before.

## When a record departs

At the end of every accepted event, after all of its effects, each retired
record that nothing pins **departs**. The pins are Lineage Compaction 0.1's
("What stays exact"):

- a state's grounds, by its own reads;
- the grounds of each commitment in force, by their own reads: the revision in
  force of each decision series, and every commitment outside a series, which
  is in force once made;
- the `because` of each journal entry still in the journal's window, and its
  `permitted_by`, the concrete grant the journal copies
  ([Decision journal 0.1](caveat-decision-journal-0.1.md));
- a pending scheduled qualification's target;
- the current occurrence of a renewable evidence, which is never retired;
- the grant named by a commitment's permission record;
- the reason named by a withdrawal still in the session.

The pins on a permission grant and a withdrawal's reason are this
profile's. Each is pinned because a predicate still reads it, not because of
its form: a marker could stand for one record, with `from` equal to
`through`.

- **A permission grant** stays exact while the commitment that holds its
  record is held, because `permission_withdrawn(D)` reads the grant's
  withdrawal ([Permission 0.1](caveat-permission-0.1.md)). The bound is a
  number the source fixes. Each held commitment has at most one grant, so
  pinned grants are at most the program's commitments outside a series, plus
  the sum of its decision series' limits, which [Reactive 0.5](caveat-reactive-0.5.md)
  caps at 1,024 together with its reading capacities. It cannot grow with
  play. A journal entry's `permitted_by` names the same grant, and pins it
  while the entry is in the window, so a series window that departs a revision
  leaves no exact citation of a departed grant.
- **A withdrawal's reason** stays exact while the withdrawal stands, because
  the withdrawal predicates read it. "Each predicate carries the withdrawal's
  reason as its evidence" ([Withdrawal 0.1](caveat-withdrawal-0.1.md)), so
  `withdrawn(E)`, `withdrawn(latest(S))` and `rests_on_withdrawn(D)` supply
  the reason into a guard's lineage, a `set`'s grounds and a `because`
  citation. A read supplies a record, never a marker: markers arise only when
  a record departs after it was read. If the reason had departed, the
  predicate would have nothing to carry, and a `because` citing the reason
  would be refused as `evaluation/ungrounded_citation`, a refusal departure
  caused. The bound is one pinned reason per standing withdrawal. A withdrawal
  stands only while its subject is held, and it leaves when its subject
  departs (below).

Both pins serve one invariant: **departure never changes an outcome.** A
program accepts and refuses exactly the events it would with retirement alone,
with the same refusal origins and codes. Departure shows only in the size of
the save, `depart` effects, markers in explanations, and the archive.

A renewable evidence's first occurrence is the evidence the program declares,
such as `witness_cave`, which is `witness_cave@1`. The program's own
declarations name it and relate it, so it never departs: once retired, it
stays retired. That keeps at most one retired record per renewable evidence
in the session, a number the source fixes. Every other record of a windowed
history is created by an event and can depart.

A record retires and departs in the same event when nothing pins it. A record
that a pin held departs at the end of the first event after which nothing
pins it, such as the event whose `set` replaces the grounds that read it.
Restore departs nothing: a restored session's unpinned retired records depart
at the end of its next accepted event.

A refused or failed event departs nothing. Departure rolls back with the rest
of the event, as retirement does ([Dispatch 0.1](caveat-dispatch-0.1.md)), and
the archive keeps only what accepted events departed.

Nothing pins a retired journal entry, because nothing cites a journal entry by
name. A journal entry therefore departs in the event that retires it, and the
journal's departed entries are always its oldest.

## What departure does

In the event in which a record departs:

1. every provenance that names it compacts, as Lineage Compaction 0.1's
   "Departure" says: the name leaves `evidence` (and `inherited`), and joins
   the provenance's marker for its history;
2. the record leaves the session: its graph node and every relation to or
   from it, its entry in `retired`, its observation, examination and
   predicate qualifications, its place in the observation order, its
   reading-stream or renewal entry, or its journal entry;
3. a withdrawal whose subject is the record leaves with it. The withdrawal
   explains why the record stopped counting, and the record no longer counts
   in any read;
4. the record's archive entry is appended to the archive.

The occurrence numbering of Windows 0.1 continues: the next record of a
history is numbered one past its newest, whatever has departed. A name is
never reused. A windowed history's ceiling of 65,536 records counts the
records the session holds, live and retired, so departure lifts it for the
records that leave.

## Effects

Departure is an effect of the event that caused it. `effects` reports each
departure after every other effect of the event, by history name and then by
number:

```json
{"kind": "depart", "history": "witness_cave", "record": "witness_cave@3"}
```

A program without a window never reports `depart`.

## The archive

The session keeps an append-only **archive** of departed records, in the order
they departed. The host drains it: the runtime's `drain_archive()`, and the
kit session's `drainArchive()`, return the entries departed since the last
drain and empty the archive. A host that needs the full history stores what it
drains beside its saves. A host that does not can discard it.

The archive is a handover buffer, not session state. No read, predicate,
explanation or restore depends on it, so a save, a snapshot and a session
fed the same events are the same whatever the host has drained. The archive
is not part of the save. A save holds what is still in the
session, so a program whose every growing history has a window has a save
bounded by its windows, whatever the length of play. A host that saves
without draining keeps the undrained entries only in memory. Restore starts
with an empty archive. Until it is drained, the archive grows in memory with
play, which is the price of handing records over rather than erasing them.

An archive entry holds what the record was when it departed:

```json
{
  "record": "witness_cave@3",
  "history": "witness_cave",
  "number": 3,
  "retired_at": 120,
  "departed_at": 130,
  "relations": [["witness_cave@3", "supports", "glowcap_safe"]],
  "holders": [
    {"kind": "state", "name": "j1_code", "in": "lineage"},
    {"kind": "series", "name": "trust", "in": "selection_qualifications"}
  ]
}
```

- `record`, `history` and `number` name it; `retired_at` and `departed_at` are
  the sequences of the events that retired it and that it departed in.
- `relations` lists the relations that left with it, as the save writes them.
- `qualifications` maps `observation`, `examination` and each predicate to the
  provenance it carried, when it had any.
- `reading` is a reading-stream occurrence's record as the save writes it;
  `journal_entry` is a journal entry's; `withdrawal` is the withdrawal that
  left with it.
- `holders` lists every provenance that named it when it departed, as Lineage
  Compaction 0.1's "The archive entry" requires: `kind` is `state`,
  `commitment`, `series`, `stream`, `scheduled`, `cue` or `qualification`;
  `name` is the state, commitment, series, stream, scheduled target, cue index
  or qualified record; `in` is the field (`lineage`, `grounds`, `basis`,
  `selection_qualifications`, `reopening_qualifications`, `guard`, and so on).
  Bindings are computed again from states rather than saved, so they are not
  holders.

So `explain` over the session together with a drained archive names every
record a value read. The session alone gives the range and count.

## Snapshot, view and save

- The snapshot's `retired` map no longer names a departed record. Markers
  appear in provenances as Lineage Compaction 0.1 says.
- `caveat-reactive-view/0.1` reports `depart` effects. Its provenance objects
  carry `departed` as the save does.
- A save writes the journal's departed entries as their count, as
  `journal_departed`, absent when there are none. Entry `K` of the saved
  `decision_journal` is `journal@(journal_departed + K)`. Reading streams,
  renewals and the graph are written without their departed records. Their
  remaining records keep their names, so a stream's or renewal's departed
  records are the numbers below its oldest live record that are not in
  `retired`.
- The save's `effects`, the last event's, may name a departed record in a
  `retire` or `depart` effect. Those are reports, not citations.

Restore refuses a save when:

- `journal_departed` appears without `journal window`, or the journal's
  retired entries are not exactly the oldest held ones beyond its window;
- a windowed history's live records are not its newest `min(N, newest)`
  numbers without a gap. Below its oldest live record, a number may be held
  retired or departed, which replaces Windows 0.1's refusal of a gap there: a
  pinned retired record can be older than a departed one;
- the save has a departed record (a marker, `journal_departed`, or a number
  below a history's oldest live record that is not held) and also holds a
  retired record that nothing in the save pins, other than a renewable
  evidence's declared first occurrence. Departure would have taken it
  at the end of the event the save follows;
- a relation, qualification, withdrawal, observation order entry or exact
  citation names a departed record;

and in every case Lineage Compaction 0.1's restore list applies. A save from a
session with no departure, such as rc.15's windowed saves, may hold unpinned
retired records; they depart at the next accepted event. As with every
restore check, a save that passes holds records the program could hold, not
records events are shown to have produced ([Save 0.1](caveat-save-0.1.md)).
Restore never authenticates history and cannot tell whether a host's drained
archive matches the save.

A reopening qualification without a `reopens` relation is no longer refused
in one case. Save 0.1 refuses a reopening qualification unless a `reopens`
relation names its commitment, because no source mechanism produced one
otherwise. Departure now does: when a reopened commitment's cause departs,
the `reopens` relation leaves with the record (step 2 above), the commitment
stays open, and its reopening qualification holds the cause as a marker. So
restore accepts a reopening qualification whose commitment is open and whose
provenance holds a marker, and refuses every other reopening qualification
that no `reopens` relation names, as before. Like every restore check, this
asks only whether a source mechanism can produce the record. It does not
show that departure produced it.

## Checks before the gate

The departure PR records two measurements as numbers, beside the gate
(owner, 2026-10-05 17:11 UTC):

- **Pinned records across the corpus.** For every windowed program in the
  repository, and for a fixture whose guarded commit skips forever, the most
  retired records held by pins at checkpoints over a long run. A program whose
  pinned set grows with play is a finding.
- **Cross-version restore.** A windowed rc.15 save of C3, written by the
  published `caveat-lang@0.1.0-rc.15`, restored on this profile and played
  on: the save size at restore, after the first event, and after every state
  has been set once (owner, 17:39 UTC).
- **Dispatch time.** C3's dispatch microseconds beside its save bytes, next to
  rc.15's, since dispatch is a registered gate (under 1 ms) and the
  [beat-typescript](../docs/design/beat-typescript.md) target is under 50 µs
  (owner, 17:47 UTC). The pin check at the end of an event is incremental: it
  considers only the candidates, which are the records retired in the event
  and the retired records whose pin the event released (a `set` replacing
  grounds, a journal entry retiring, a scheduled qualification applying, a
  commit superseding a revision). It never scans the session.

The pinned-record count is attributed by the provenance that pins: state
grounds, commitment in force, journal entry, scheduled qualification,
permission grant and withdrawal reason. A journal entry in the window pins
what it cites, including names a commitment inherited at the time, so some
records appear under "journal entry" that would compact under "state
grounds". That is bounded by `journal window N` and is not a defect (owner,
17:39 UTC).

## Gate

Two sweeps gate the departure implementation:

- **The digest sweep.** For every program without a window, the seeded save
  sweep's digests are identical before and after.
- **The outcome sweep.** For every windowed program below, over the same
  seeded runs, the sequence of dispatch outcomes (accepted, or the refusal's
  origin and code) is the same on this profile as on rc.15's retirement-only
  build (owner, 17:49 UTC). A missing pin shows up there as a refusal rc.15
  did not produce. None of the corpus's tracked `.cav` files declares a
  window, so the sweep names its population:
  - round 7's C3 program with windows, derived as
    `experiments/lineage-compaction/simulate.mjs` derives it (`renewable …
    window 2` for each `limit 64`, and `journal window 6`), over
    `longplay.mjs`'s 60 cycles and over seeded runs;
  - the windowed programs in `kit/test/explain.test.mjs` and
    `kit/test/types.test.mjs`;
  - the program in `runtime/tests/windows.rs`.

  An empty population would let the gate pass without testing anything, so
  the sweep's record lists each program and its number of runs. A windowed
  program added to the tracked corpus later joins it.

Then Lineage Compaction 0.1's gate: round 7's C3 program with windows, on a runtime
build, over `longplay.mjs`'s 60 cycles, the adapter's save bytes the same at 30
and 60 cycles apart from the digits of growing numbers, with the raw bytes
recorded beside them.

## Changes

- 2026-10-05 (rc.16, specified): new profile. Retired records that nothing
  pins depart at the end of each accepted event; departure is a reported,
  transactional effect; a host-drained archive outside the save, with each
  entry's holders; the `journal_departed` save field and restore checks; a
  permission record's grant and a withdrawal's reason pin; a renewable
  evidence's declared first occurrence never departs. The owner chose eager
  departure, the archive outside the save and the pinned grant at 17:47 UTC,
  with riders: a journal entry's `permitted_by` pins, the grant pin's bound and
  reason, the archive as a handover buffer, and C3 dispatch time measured with
  an incremental pin check. At 17:49 UTC the owner gave the withdrawal
  reason's pin its reason, stated the invariant that departure never changes
  an outcome, and added the outcome sweep. Not implemented.
- 2026-10-05 (rc.16, outcome sweep population): the outcome sweep named
  "every windowed program in the corpus", but no tracked `.cav` file declares
  a window, so as written it would pass on an empty population (owner, 18:21
  UTC). It now names its programs, and its record lists them with their run
  counts.
- 2026-10-05 (rc.16, a reopening whose cause departed): implementing
  departure showed that a reopened commitment whose cause departs keeps its
  reopening qualification but loses the `reopens` relation that Save 0.1's
  restore check requires. Restore now accepts such a qualification when its
  commitment is open and it holds a marker, because departure is a source
  mechanism that produces it. Every other reopening qualification that no
  `reopens` relation names is still refused.
