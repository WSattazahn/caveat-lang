# CAVEAT Lineage Compaction 0.1 — what a record leaves behind when it departs

Status: specified for rc.16, not implemented. It is the rule that
[departure](../docs/design/save-forgetting.md) and series windows build on
(owner, 2026-10-05 14:45 UTC; marker shape and riders 16:34 UTC, see
[Decision](#decision)).

[Windows 0.1](caveat-windows-0.1.md) lets a history retire its oldest records.
A retired record leaves the live graph but stays in the session, so a
windowed program's save still grows. Departure, the second half of the
owner's 2026-10-04 decision, lets a retired record leave the session. Under
the [save-forgetting](../docs/design/save-forgetting.md) plan, a record could
depart only when nothing cited it. Round 7's C3 program showed that this
never happens for a record that a value has read.

This profile replaces that pinning rule with one sentence: **a value's own
citations stay exact and pin their records; inherited bases and read lineage
compact to one range per history.** When a record departs, every lineage and
inherited basis that named it keeps a **departure marker** for its history in
place of the name, so `explain` still says what a value rested on. When a
retired record departs, and the archive the host drains, are the
[departure specification](caveat-departure-0.1.md)'s to define. This profile defines what every citation keeps and
what the archive entry must name.

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
  the 66,464 bytes of `states`. The `trust` series' selection and reopening
  qualifications held the same 295 names.

A rule that departs records but keeps their names in lineages cannot make this
save flat. The names, not the records, are what grow. A marker per departed
record is compact per record but not per reference, so it moves the linear
term from `graph` into `states`.

## What stays exact

A value's own citations stay exact names and **pin** their records: a record
they name does not depart while they name it. They are:

- a state's **grounds** ([Explanations 0.2](caveat-explanations-0.2.md)), what
  its value is based on;
- the **grounds of the revision in force** of each decision series: what its
  `using` read, with its retained caveats (`commitment_grounds`);
- the `because` of each **journal entry still in the journal's window**.

A grounds' own reads, the names its expression, guard or predecessor check
actually read, are always exact and pin. A grounds can also carry names it
inherits through a selection or reopening qualification: [Reactive 0.5](caveat-reactive-0.5.md)
has later `latest`, `has_sample` and current-series predicates inherit a
skipped sample's or skipped series commit's guard. Those inherited names are
not the value's own reads. They pin nothing and compact like lineage, so a
grounds may hold a marker, and only through such a qualification.

These are the reasons the language promises to keep. They are expected to be
bounded by the source, not the run: a grounds replaces the one before at each
`set`, and a history read supplies only live records. In C3's saves at 10, 30
and 60 cycles they pin five retired records each time: `trust@1`'s grounds
(`witness_cave`) and four recent witnesses. The departure implementation counts pinned
records across the corpus, so a program whose own citations grow with play is
reported, not assumed away. The count includes a program with a guarded
commit whose guard skips forever, so that inherited selection qualifications
are shown to compact rather than pin. A record that the session must
still name for its own reasons is pinned too: a live record, the current
occurrence of a renewable evidence, a record with a scheduled qualification
still pending, and a commitment that is the current entry of its series.

Everything else that names a departed record compacts: a state's
**lineage**, a commitment's **basis** (including what a revision inherits from
its predecessor through the check that the predecessor was reopened), the
grounds of revisions no longer in force, a series' selection and reopening
qualifications, withdrawal and permission records.

## Departure markers

A departure marker stands for the departed records of one history that a
compacting provenance named:

```json
{"history": "witness_cave", "read": 41, "from": 1, "through": 59, "departed_at": 412}
```

- `history` is the windowed history's name: a reading stream, a renewable
  evidence, a decision series once series windows exist, or `journal`.
- `from` and `through` are the lowest and highest occurrence numbers among
  the departed records it stands for. Each is a record the provenance named. A
  history's first occurrence is number 1 (`witness_cave` is
  `witness_cave@1`).
- `read` is how many departed records of the range the provenance named.
- `departed_at` is the sequence of the event at which the latest of them
  departed.

A provenance holds **at most one marker per history**, beside its exact
names. Its exact names are the records still in the session, live or retired.
A pinned record keeps its exact name everywhere it is cited, including inside
the numeric range of a marker, which never counts it.

**Departure.** When a record departs, every provenance that names it drops
the name and adds it to its marker for that history. If it has no marker for
that history yet, it gets one with `read` 1 and `from` and `through` set to the
record's number. Otherwise `read` grows by one, `from` becomes the lower number
and `through` the higher. In both cases `departed_at` becomes the departing
event's sequence. A record departs in exactly one event, so every provenance
that named it is compacted in that same event, and `read` is exact when it is
written.

**Combining provenance.** Where a read combines provenance (a union of
lineages, a state read in an expression, a revision's basis taking its
predecessor's), markers for the same history join by `from = min`,
`through = max`, `departed_at = max` and `read = max`. The overlap of two
markers is not recorded, so the larger count is the most that can be said: a
combined marker read **at least** `read` of its range. The join is
associative, commutative and idempotent, so a combined provenance does not
depend on evaluation order, and incremental evaluation ([Incremental
evaluation 0.1](caveat-incremental-evaluation-0.1.md)) stays exact.

**Meaning.** A marker says the value read at least `read` of the departed
records numbered `from` to `through`, including those two, and nothing else of
that history that has departed. It does not say which records inside the range.
Lineage answers *what could have influenced this?*, and the range with its
count answers it within the range. A state that read every departed record of
a history, as C3's journal slots do, has `read` equal to the range's departed
count, and is exact. Grounds narrow lineage. A grounds marker is within the
lineage's marker for the same history: its `from` is not lower, its `through`
is not higher, and its `departed_at` is not later. So **grounds ⊆ lineage**
still holds ([Explanations 0.2](caveat-explanations-0.2.md)).

**Caveats.** A provenance's caveats are names of declared caveats, a set that
cannot grow with play. They are kept exactly. A departed record's
qualifications stay in every provenance that carried them.

**Bounded.** A compacting provenance holds at most one marker per windowed
history, and a marker's size depends on numbers, not on how many records it
covers. Pinned records are bounded by the source. A windowed history keeps at
most `N` live records plus the retired records that have not yet departed. In
a program where every growing history has a window, therefore, nothing in the
session grows with the length of play except the digits of sequence and
occurrence numbers.

## The archive entry

The departure specification defines the archive that the host drains. Under
this profile, each departed record's archive entry also names the provenances
that held it when it departed: the states, commitments, journal entries,
withdrawals and permission records. Their number is bounded by the program's
declarations and windows. So `explain` over the session together with a
drained archive is exact: the range is what the session can say on its own,
and the archive says which records were read. Nothing is erased, only moved.

## Explain, dependents and the view

- **`explain`** shows each marker after the exact names. When `read` equals
  the departed records in the range, it shows "read all 59 of departed
  `witness_cave@1` to `@59`, the last at 412". Otherwise it shows "read at
  least 41 of departed `witness_cave@1` to `@59`, the last at 412". When `from`
  equals `through`, it shows the one name. Exact names are shown as before,
  and retired names keep `retired_at`. Given a drained archive, the kit's
  `explain` names the records exactly.
- **`dependents`** of a departed record cannot be answered from the session,
  which no longer holds it. Given a departed name, it marks the name departed
  and lists the provenances whose marker for that history covers its number,
  as "may rest on". A drained archive answers exactly.
- **`caveat-reactive-view/0.1`** serializes the same provenance type as the
  save: `binding_explanations` and each decision series'
  `selection_qualifications` are provenances (`ReactiveView` and
  `DecisionSeries` in `runtime/src/reactive.rs`). Its provenance objects
  therefore carry `departed` beside `evidence` and `caveats` as the save does,
  absent when there are none, so a view's bytes are unchanged for every
  program without a window. View 0.2 decides only where a marker sits in
  ordered grounds.

## Save and restore

A save writes a provenance's markers under `departed`, beside `evidence` and
`caveats`. The field is absent when there are none. A departed record leaves
the save's `retired` map, its graph node and every exact citation. A record
of a windowed history is departed when its number is below the oldest live
record's and it is not in `retired`. Numbering continues as in Windows 0.1,
and a name is never reused. Saves of programs without a window never have
`departed`, and the schema stays `caveat-reactive-save/0.1`. A runtime that
predates this profile refuses a save with `departed`, as it refuses any field
the schema lacks.

A grounds' split between its own reads and the names it inherits through a
qualification must survive save and restore. `Provenance` is a flat set
(`runtime/src/reactive_expr.rs`: `evidence` and `caveats`), so a saved grounds
alone does not say which names were read. Restore cannot rederive the split:
the qualifications that supplied the inherited names may have cleared since.
A restored session that pinned every name would keep records the original
let depart, and the two would diverge. So a save writes a grounds' inherited
names under `inherited`, beside `evidence`.

The split also travels through reads. When two provenances merge, a name is
inherited in the result only if neither side read it itself. A read takes a
reading stream's or decision series' selection qualifications, and a
reopening qualification, as inherited whatever they hold. Every other value
it reads passes its names on with their split as it holds them. So a `set`
that reads a state whose lineage inherited a name writes that name into its
grounds as inherited, and a restore that lost the state's list would write it
there as an own read, which pins. The split must therefore survive on every
provenance a later read can reach, not only on the grounds that pin.

In a save of a program with a window, `inherited` may appear on exactly these
provenances, and on no other:

1. a state's lineage (`states.S.lineage`);
2. a state's grounds (`states.S.grounds`);
3. a commitment's basis (`commitment_bases.C.provenance`);
4. a commitment's grounds (`commitment_grounds.C`);
5. a reading's provenance (`reading_streams.S.occurrences[i].provenance`);
6. a reading stream's selection qualifications
   (`reading_streams.S.selection_qualifications`);
7. a decision series' selection qualifications
   (`decision_series.D.selection_qualifications`);
8. a scheduled qualification's guard (`scheduled_qualifications[i].guard`);
9. an observation qualification (`observation_qualifications.E`);
10. an examination qualification (`examination_qualifications.C`);
11. a reopening qualification (`reopening_qualifications.C`);
12. a predicate qualification (`predicate_qualifications.P.N`);
13. a cue's qualification (`cue_qualifications[i]`).

These are all the provenances the save schema holds. A later profile that
adds a provenance to the save adds it to this list. Only 2, and 4 for a
commitment in force, pin, and only their own reads. The other holders never
pin, so the list adds no pin and no category to Departure 0.1's attribution.

Only a program with a window keeps the split at all. Without a window nothing
departs, inherited names merge as any others, and those saves and snapshots
stay byte for byte as before. The field is absent when a provenance inherited
nothing. A state whose grounds carry `inherited` writes its grounds even when
their names equal the lineage's. A name that is both an own read and
inherited is an own read, and `inherited` lists only names that are not own
reads. Snapshots show `inherited` wherever the session holds it.

When an inherited name departs, it leaves both `evidence` and `inherited` and
joins the grounds' marker for its history. Every name in `inherited` is
therefore still in `evidence`, and every marker in a grounds is inherited, as
the rule in "What stays exact" requires. An own read never departs while it
is cited, because it pins. The next `set` replaces the whole grounds, its
`inherited` list and markers included, as it replaces the names today.

A windowed save written by rc.15 has no `inherited` field, and none of its
records have departed. Restore reads each missing list as empty, so every
name in every holder counts as an own read. That can keep records longer
than a session started on this profile would. It never loses one, and no
rc.15 session departed anything, so there is no original for the restored
session to diverge from. The difference is between versions, not between a
session on this profile and its own restore, so the save sweep's property
holds.

Only grounds pin, so the excess, called over-pinning here, arises in a
grounds in one of two ways. Either the grounds was itself restored without
its list, or a `set` or `commit` after the restore read a holder restored
without its list and took the holder's names as own reads. Selection and
reopening qualifications (6, 7 and 11) never do the second, because a read
takes them as inherited anyway. How long it lasts, holder by holder:

- **A state's grounds (2)** pins its restored names until the state's next
  `set` replaces the grounds.
- **A state's lineage (1)** passes its names on as own reads until the
  state's next `set` replaces it.
- **A commitment's grounds (4) in a decision series** pin their restored
  names until a later revision supersedes the commitment. Its basis (3)
  passes its names on as own reads for as long as a read reaches it.
- **A commitment's grounds and basis outside a series** are frozen. Its
  grounds over-pin for the rest of the session, by at most the names they
  held at restore.
- **A reading (5)** is read only while it is live. It passes its names on
  until it retires.
- **The qualifications 8, 9, 10, 12 and 13** pass their names on for as long
  as they hold them.

Over-pinning has one bound in every case. It names only records the restored
save already held, because only holders restored without their lists carry
a misread name, and they were all written before the restore. A read can
copy a misread name into a holder written after the restore, but it cannot
add a new one. A record that a session creates after the restore is never
over-pinned, so over-pinning never grows with play. The holders above say
when each source stops. Because a name can be copied, no fixed number of
`set`s is guaranteed to end it, so the departure implementation measures
the restored size instead of predicting it. Because the field is
absent when empty, an rc.15 save and a save on this profile with no inherited
names look the same, and restore reads them the same way.

Restore refuses a save when:

- a marker names a history without a window, or one the program does not
  declare;
- a marker's `from` is below 1 or above its `through`, or its `from` or
  `through` is not a departed record of its history;
- a marker's `read` is below 1, or above the number of departed records from
  `from` to `through`;
- a marker's `departed_at` is 0 or past the save's `sequence`;
- a provenance holds two markers for one history, or names a departed record
  exactly;
- a grounds marker is not within the lineage's marker for its history, or a
  commitment's grounds marker is not within its basis's;
- a name in `inherited` is not also in the same provenance's `evidence`;
- `inherited` appears in a save of a program without a window. The 13
  holders above are every provenance the schema has, so a field anywhere
  else is already an unknown field, and restore refuses it as such.

As with every restore check, a save that passes holds records the program
could hold, not records events are shown to have produced
([Save 0.1](caveat-save-0.1.md)). Restore never authenticates history.

## Series revisions and the journal

**Series revisions.** Since [Reactive 0.5](caveat-reactive-0.5.md), a
revision's basis holds its predecessor's basis, so the series chain is
another lineage that grows with play. That inherited basis compacts, while the
revision in force keeps its own grounds exact. Series windows, which follow
this profile, retire and depart revisions. A later revision's basis then holds
one marker for the series, such as `trust@1` to `@200`, and markers for the
histories its predecessors read. What a revision rests on is the same rule
for every series, windowed or not. Only which records are still in the session
depends on the window, and that is exactly what a marker reports. This is the
revision-lineage rule the rc.15 record named for rc.16, stated for every
provenance at once.

Series windows get their own fixture when they land, because C3's `trust`
series never gets past one revision. The kit's `boss-stance` example revises
`stance` on every switch. Run long enough for revisions to retire and depart,
the revision in force must still explain with its own grounds exact and its
inherited chain as one range.

**The journal.** Under `journal window N`, a retired journal entry departs by
the same rule. An entry still in the window keeps its `because` exact. That
pins what it cites, which is bounded by the window. Its basis compacts. Nothing
else cites journal entries.

## Gate

The registered fixture decides when departure is done. It is measured as
round 7 measured it: round 7's C3 program with windows on its renewable
evidences and its journal, run on a runtime build right after departure, over
`longplay.mjs`'s 60 regrowth cycles. The registered figure is the adapter's
save bytes, not the raw save. The gate passes when the adapter bytes at 30 and
60 cycles are the same, apart from the digits of growing clock and sequence
numbers. Smaller is not enough. The record gives the raw bytes beside the
adapter bytes, so the tables line up with the rc.15 record's:

| Adapter save bytes | 10 cycles | 30 cycles | 60 cycles |
| --- | ---: | ---: | ---: |
| C3 as written (`limit 64`), round 7's figures, reproduced on rc.15 | 3,714 | 8,790 | 16,310 |
| C3 with both windows, rc.15 build (retirement only) | 4,582 | 11,870 | 22,182 |

Round 7's other registered figures were C4's 40,661 bytes and TypeScript's
345–514.

`experiments/lineage-compaction/simulate.mjs` simulates this over rc.15's
real C3 saves. It departs every retired record that no value's own citation
pins, replaces departed names with markers in every compacting provenance in
the save, and drops the departed records. Its byte counts are of compact
re-serialized JSON, slightly smaller than the runtime's text:

| Save bytes (JSON as re-serialized) | 10 cycles | 30 cycles | 60 cycles |
| --- | ---: | ---: | ---: |
| rc.15 with windows | 39,126 | 113,758 | 225,929 |
| one marker per departed record | 36,277 | 101,849 | 200,550 |
| one marker per history (this profile) | 16,495 | 16,799 | 16,800 |

Under this profile's markers, every part of the save is the same size at 30
and 60 cycles, give or take a digit: `states` 9,280 bytes, `graph` 3,130, the
journal 1,074. The `trust` series has one revision in this play, so C3 needs
no series window. Its retired names sat in the series' selection and
reopening qualifications, which compact like any other inherited provenance.
This is a simulation over raw saves, not a runtime measurement or the
adapter's figure, and the gate is measured on the runtime.

For every program without a window, the implementation's gate is the seeded
save sweep (`runtime/examples/save_sweep.rs`) over the repository's reactive
programs, with digests identical before and after.

## Decision

The owner chose **one range per history** on 2026-10-05 at 16:34 UTC, over a
marker per departed record and over exact markers for the newest records with
a range for older ones. The reviewer's reasons, adopted by the owner:

- A marker per record is compact per node but not per reference. C3's twelve
  journal-slot states each reference every occurrence, so it moves the linear
  term from `graph` into `states`.
- A range is the only form whose size is fixed by the source, at one marker per
  history per lineage, rather than by the run.
- The newest-exact form keeps exact names only for the latest departures, which
  are the records the archive received last. It costs a second marker form, a
  new parameter and two shapes in every reader (`explain`, the restore check,
  the Lean model). Anyone who needs the exact names gets them from the archive.

The newest-exact form is bounded too, and it also joins: any record among the
newest of a union is among the newest of one side. The choice rests on the
reasons above, not on a failure to merge.

The owner added two riders and an archive requirement. All three are part of
this profile:

1. **The marker carries the count read** (`read`), so a value that read a
   subset is not recorded as having read the whole range.
2. **The revision in force keeps its own grounds exact and pinned**. Only the
   inherited predecessor bases and state lineage compact. This profile states
   it for every value: a value's own citations stay exact.
3. **The archive entry names the provenances that held the record when it
   departed**, so `explain` with a drained archive stays exact.

Order: this specification, then departure, then the C3 gate measured on a
runtime build, 60 cycles with the adapter save flat. Series windows, View 0.2
and the rest of rc.16 wait until that gate is measured. The owner set the
order at 16:34 UTC with series windows before the gate, on numbers that showed
C3's `trust` series growing. The rerun above shows it does not, and at 16:40
UTC the owner moved the gate ahead of series windows: a series-windows PR
cannot change what C3's gate measures.

Three details differ from the owner's message, to fit the language:

- The message's restore bound `to < departed count` became "`from` and
  `through` are departed records". Occurrences number from 1, so `@1..@59`
  with 59 departed has `through` equal to the count, and pinned records mean
  the departed records of a history need not be a prefix.
- The message's "`because` basis" of the revision in force is its grounds,
  what `using` read. `because REASON` on a commit names a reason, not
  citations.
- `read` joins by maximum and reads "at least" after a combination, because
  the overlap of two markers is not recorded.

A candidate for rc.16's Lean model, from the owner's 17:00 UTC review:
**compaction never overstates**. At departure a marker's `read` is exact, and
after any join it is at most the true number of departed records the
provenance read in its range.

## Changes

- 2026-10-05 (rc.16, specified): new profile. A value's own citations (state
  grounds, the revision in force's grounds, in-window journal entries'
  `because`) stay exact and pin their records. Lineage and inherited bases
  replace departed names with one marker per history (`read`, `from`,
  `through`, `departed_at`), joined order-independently. The profile also
  covers the archive entry's holders, the `explain` and `dependents` forms, the
  save field and restore checks, series revisions and the journal under the
  same rule, the C3 gate, and the simulation behind the marker shape. The
  owner chose the shape and its riders at 16:34 UTC. The owner's 17:00 UTC
  review fixed two points before merge: a grounds' own reads pin, while
  markers it inherits through a qualification pin nothing and are allowed;
  and View 0.1 provenance objects carry `departed` as the save does. At
  17:08 UTC the owner added the saved split of a grounds' own and inherited
  names (`inherited`), its restore check, and how restore reads a windowed
  rc.15 save. Not implemented.
- 2026-10-05 (rc.16, the split travels through reads): implementing
  departure showed that a read carries a value's inherited names into what it
  writes. A restore that kept `inherited` only on a state's grounds and the
  grounds of a revision in force would read every other holder's names as own
  reads, and a later `set` would pin records the original lets depart. The
  runtime's restore test failed in exactly this way. A save of a program with
  a window now writes `inherited` on a closed list of 13 holders, which is
  every provenance the save schema has. Restore checks each list against its
  own `evidence`. The rc.15-save paragraph now gives, holder by holder, how
  long over-pinning lasts and the bound that holds in every case. Only
  grounds pin, so Departure 0.1's attribution gains no category. Programs
  without a window keep no split, so their saves and snapshots are unchanged.
  The owner checked the diagnosis and set these requirements at 18:21 UTC.
