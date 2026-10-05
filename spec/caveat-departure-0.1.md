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
- the grounds of the revision in force of each decision series, by its own
  reads;
- the `because` of each journal entry still in the journal's window;
- a pending scheduled qualification's target;
- the current occurrence of a renewable evidence, which is never retired;
- the grant named by a commitment's permission record.

The last pin is this profile's. A permission record names its grant as one
exact name, which has no marker form, and a commitment's permission record is
part of its frozen basis. The commitments that hold one are bounded by the
program's declarations and its series limits. Series windows, when they land,
revisit it with the revisions they depart.

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

The archive is not part of the save. A save holds what is still in the
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
  retired record that nothing in the save pins. Departure would have taken it
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

## Checks before the gate

The departure PR records two measurements as numbers, beside the gate
(owner, 2026-10-05 17:11 UTC):

- **Pinned records across the corpus.** For every windowed program in the
  repository, and for a fixture whose guarded commit skips forever, the most
  retired records held by pins at checkpoints over a long run. A program whose
  pinned set grows with play is a finding.
- **Cross-version restore.** A windowed rc.15 save of C3, restored on this
  profile and played on: the save size at restore, after the first event, and
  after every state has been set once.

## Gate

Lineage Compaction 0.1's gate: round 7's C3 program with windows, on a runtime
build, over `longplay.mjs`'s 60 cycles, the adapter's save bytes the same at 30
and 60 cycles apart from the digits of growing numbers, with the raw bytes
recorded beside them. For every program without a window, the seeded save
sweep's digests are identical before and after.

## Changes

- 2026-10-05 (rc.16, specified): new profile. Retired records that nothing
  pins depart at the end of each accepted event; departure is a reported,
  transactional effect; a host-drained archive outside the save, with each
  entry's holders; the `journal_departed` save field and restore checks; a
  permission record's grant pins. Not implemented.
