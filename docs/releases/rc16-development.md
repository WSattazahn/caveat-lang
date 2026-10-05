# CAVEAT 0.1.0-rc.16 development plan

Status: development open, unpublished. This file records the owner's direction
for rc.16 as given so far and the cycle's progress. Keep the "Progress"
section factual, as `rc15-development.md` does.

## Starting point

- The cycle opens at `main` = `02173f16795e3647c6c456cf51aa27453d2f1613`, the
  merge of PR #168 (the rc.15 publication record).
- rc.15 was published on 2026-10-05 through `publish-npm.yml` with provenance.
  `latest` and `next` both name it, and the MCP Registry lists
  `io.github.WSattazahn/caveat-lang` 0.1.0-rc.15 as latest
  ([rc.15 release record](v0.1.0-rc.15.md)).
- `kit/package.json` still says `0.1.0-rc.15`. The version moves to
  `0.1.0-rc.16` with the first runtime change, not with this record.
- PRs merge as merge commits, never squash. Merges reopened when `next` moved
  to rc.15 (owner, 2026-10-05 14:45 UTC: "move `next` by hand as for
  rc.13/rc.14, and only then let the thread resume merges").

## Owner direction (2026-10-05 14:45 UTC)

- **PR 1 is lineage compaction**: "rc.16 PR 1 is the lineage-compaction spec
  covering series revisions, state lineage and journal entries together;
  departure and View 0.2 follow it; the C3 longplay fixture is the gate."
- **View 0.2 is in rc.16**, as [`docs/design/view-0.2.md`](../design/view-0.2.md)
  proposes. The rider: the delta view's gate is that the full view at `N` plus
  the delta equals the full view at `N+1`, checked over the whole corpus as the
  save sweep is, not on a sample.
- The rest of the cycle follows
  [`beat-typescript.md`](../design/beat-typescript.md) as direction, not
  scope: the adapter scaffold, the reactive-only WebAssembly build and the
  dispatch path. Glowcap round 8 waits for rc.16, because the save-bound cause
  stays open until departure lands.

## Progress

- PR 1: [Lineage compaction 0.1](../../spec/caveat-lineage-compaction-0.1.md),
  specified, not implemented. A value's own citations stay exact and pin their
  records: state grounds, the grounds of the revision in force, and the
  `because` of in-window journal entries. Lineage and inherited bases replace
  departed names with one marker per history (`read`, `from`, `through`,
  `departed_at`). A departed record's archive entry names the provenances that
  held it. The owner chose this shape and its riders at 16:34 UTC, over a
  marker per record and over newest-exact markers. A simulation over rc.15's
  real C3 saves (`experiments/lineage-compaction/simulate.mjs`) gives these
  raw save bytes at 10, 30 and 60 cycles:
  - a marker per record: 36,277, 101,849 and 200,550, still linear;
  - this shape: 16,495, 16,799 and 16,800, flat.

  The simulation's first run was wrong, and it stays on record here. That
  first run compacted only the states' lineages. It reported 14, 18 and 23 KB
  raw at 10, 30 and 60 cycles for this shape, and 181 KB at 60 for a marker
  per record. It put the remaining growth down to the `trust` series, as if
  that series had many revisions without a window, and from that it concluded
  that the C3 gate needed series windows. In fact `trust` has one revision.
  The growth was 295 uncompacted names in its selection and reopening
  qualifications, which are provenances like any other. Compacting every
  provenance gave the flat figures above. The card the owner answered at 16:34
  carried the first run's numbers, and the 16:34 order put series windows
  before the gate for that reason.

  The order (owner, 16:40 UTC, gate first): this spec, departure, then the C3
  gate on a runtime build right after departure. The gate is measured as round
  7 measured it: adapter save bytes over 60 `longplay.mjs` cycles, flat between
  30 and 60, with the raw bytes recorded beside them. Two adapter tables are on
  record for comparison:
  - round 7's registered figures for C3 as written (`limit 64`), reproduced on
    rc.15: 3,714, 8,790 and 16,310;
  - the rc.15 build with both windows (retirement only): 4,582, 11,870 and
    22,182.

  Series windows, View 0.2 and the rest of rc.16 wait for the gate number.
  When series windows land, they get their own fixture: `kit/examples/boss-stance`
  (#159), run until revisions depart.

  The owner's review at 17:00 UTC fixed two points before merge. A grounds'
  own reads pin, while a marker a grounds inherits through a selection or
  reopening qualification ([Reactive 0.5](../../spec/caveat-reactive-0.5.md))
  pins nothing and is allowed, so restore no longer refuses it. View 0.1
  provenance objects carry `departed` as the save does, because the view
  serializes the same provenance type. The simulation pins every name in a
  grounds, inherited ones included, so its figures can only overstate the save.

  At 17:08 UTC the owner added one point before departure. A grounds' split
  between its own reads and its inherited names must survive save and
  restore, because a provenance is a flat set. The save therefore writes
  `inherited` beside `evidence`, only for programs with a window, and restore
  checks that every inherited name is also in `evidence`.

- PR 2: [Departure 0.1](../../spec/caveat-departure-0.1.md), specified, not
  implemented. At the end of each accepted event, every retired record that
  nothing pins departs. Its citations compact as Lineage Compaction 0.1 says,
  the session reports a `depart` effect, and the record goes to an archive that
  the host drains. The archive is not saved. Nothing cites a journal entry by
  name, so a retired journal entry departs at once, and a save writes the
  journal's departed entries as a count. A permission record's grant pins.
  The departure implementation records two numbers the owner asked for at
  17:11 UTC: the pinned records across the corpus, including a fixture whose
  guarded commit skips forever, and the save sizes when an rc.15 windowed
  save is restored and played until every state has been set once.

  The owner chose all three of the departure rules at 17:47 UTC: departure at
  the end of every accepted event, the archive kept outside the save, and a
  permission grant that pins. Three riders came with that choice. A journal
  entry's `permitted_by` pins while the entry is in the window. The grant pin
  is bounded by the program's commitments and series limits, and it exists
  because `permission_withdrawn(D)` reads the grant. The departure PR reports
  C3's dispatch microseconds beside its save bytes, with a pin check that
  considers only the candidates.

  For the rc.16 adapter scaffold, when it is built (owner, 17:47 UTC): it
  drains the archive by default, and the kit exposes the number of undrained
  entries, so a host that never drains can see what it is accumulating.

- PR 3: the `inherited` split travels through reads ([Lineage compaction
  0.1](../../spec/caveat-lineage-compaction-0.1.md), "Save and restore"),
  specified. Building departure showed that the 17:08 placement cannot keep
  a restored session equal to the original. That placement put `inherited`
  on a state's grounds and on the grounds of a revision in force. But a
  `set` whose `because` reads a state, a reading or a commitment takes that
  value's names, and a name the value inherited stays inherited in the new
  grounds. A restore that dropped a lineage's `inherited` list would read
  those names as own reads and pin them at the next `set`, so records the
  original lets depart would stay. The runtime's own restore test failed in
  exactly this way (`runtime/tests/restore_tables.rs`, "a genuine save with
  every table restores": a series' selection qualifications held
  `inherited: {gauge}` before the save and nothing after it).

  The owner checked the diagnosis at 18:21 UTC. They set three requirements:
  - the holders as a closed list in the write rule and the restore check;
  - the rc.15-save paragraph rewritten holder by holder;
  - the failing restore test checked in as a regression, with an rc.15 twin
    showing a bounded over-pin rather than a refusal.

  The spec now lists the 13 holders, which are every provenance the save
  schema has, and gives each holder's over-pinning and the common bound.
  Only grounds pin, so Departure 0.1's attribution is unchanged. The
  regression and its twin test departure itself, so they land with the
  departure implementation (PR 4), which is the first build that departs
  anything.

  The same review found that Departure 0.1's outcome sweep would pass
  vacuously, because no tracked `.cav` file declares a window. The sweep now
  names its population: the C3 program with windows, the two windowed kit
  test programs and the runtime's window tests.
