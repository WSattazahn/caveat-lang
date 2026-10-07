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
- At cycle opening, `kit/package.json` said `0.1.0-rc.15`. The departure
  continuation now sets the kit and registry candidate to `0.1.0-rc.16`; this
  is an unpublished checkout version, not a publication record.
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

The dated PR 1–4 entries below record the specification decisions before
implementation. Current implementation and verification are recorded in the
2026-10-07 continuation section at the end of this file.

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

- PR 4: a reopening whose cause departed ([Departure
  0.1](../../spec/caveat-departure-0.1.md), after the restore list),
  specified. Departure takes every relation of a departed record away with
  it, including the `reopens` relation from a reopened commitment's cause.
  The commitment stays open, and its reopening qualification keeps the cause
  as a marker. Save 0.1's restore check refuses a reopening qualification
  that no `reopens` relation names, so a session's own save would not
  restore. The departure branch's test of a revision basis holding departed
  readings (`runtime/tests/departure.rs`, "departed readings become one
  marker per history") was refused with "reopening of trust@1: trust@1 is
  not a commitment a reopens relation names". Restore now also accepts a
  reopening qualification whose commitment is open and whose provenance
  holds a marker, since departure is a source mechanism that produces one.
  It still refuses every other reopening qualification that no `reopens`
  relation names. The departure implementation is now PR 5.

## Continuation for review (2026-10-07)

Claude's departure WIP `7f551e5` was preserved and continued on
`codex/rc16-continuation`. This is the departure implementation (PR 5), with
the kit/registry candidate bumped to rc.16; nothing here records npm publication.
The current source contains:

- incremental pin counts and retirement candidates, with transaction rollback
  and a one-time restore rebuild;
- cascading release of withdrawal-reason pins, so an immediate save after a
  withdrawal-triggered departure restores;
- restore checks for genuine same-event departed effects and retained retired
  permission grants, while rejecting journal ordinals forged as evidence;
- the owner's exact-archive amendment: bounded content-addressed roots in live
  markers and host-drained record/union provenance nodes. Copies, merges,
  overlap and replacement preserve exact membership with a complete matching
  archive. Missing/incomplete/corrupt proof stays explicitly conservative;
- archive-aware JS/TypeScript reports, CLI and inline-authoring replays,
  persistent serve drain/count operations and caller-supplied archive queries,
  and actual-browser reconstruction checks;
- dogfooding D001–D005 follow-through (plain decisions, reverse withdrawal
  reasons, retained-caveat wording, genuine read lineage, and support/opposition
  edges), plus sample 002 of this continuation agent's observed decision.
  Claude's unavailable private log has not been reconstructed or attributed.

The [gate directory](../../experiments/departure-gate/README.md) supplies
reproduction commands and actual receipts. Full native tests passed 849 tests;
reactive-only tests passed 837. Both strict Clippy profiles and formatting
passed. The Windows debug suites used a 16 MiB test-thread stack; the current
optimized maximum-depth test passed with the default Windows stack. The
reviewed kit suite passed 320 tests. WASM interface/dispatch, game-policy,
registered scenarios, sample 002, notices and generated docs checks passed.
The fresh installed package, copied guides, Python clients, Chrome archive
transfer and Glowcap/Trail Rescue page checks passed. The candidate tarball
is 985,936 bytes, SHA-256
`6a978f2594ae17b3dad95cd14cca921a352308ab59c4a4f233dd7368687e18e2`.
Remote CI remains an independently recorded check.

No-window comparison covered 146 programs (116 executed, 30 explicitly
skipped) over 8 seeds of 150 events. All outcome/save/restore digest rows
matched the pinned rc.15 runtime. All 13 windowed fixtures, including the new
withdrawal probes, matched rc.15 dispatch outcome rows over 32 seeds of 400
events (416 rows). These are measured populations, not proofs for all programs.

Current C3 adapter save sizes at 30 and 60 cycles are both 2,646 bytes; raw
sizes are 25,682 and 25,683 with equal normalized live-save structure. All
44,460 accepted outcomes match the published rc.15 baseline. The recorded
last-ten-cycle median is 114.7 microseconds
on this Windows Node24 host. The undrained archive contains 1,578 items
(1,011 records, 567 provenance nodes), 557,347 serialized bytes; draining it
does not change the save. This is separate historical storage and may grow.
Process-memory readings are not isolated archive heap measurements.

The exact published rc.15 C3 save restores at 226,589 raw / 22,182 adapter
bytes, then measures 92,588 / 6,530 after the first event and 26,513 / 2,698
after every one of 61 states has received a set (742 accepted events).
The seeded pin census includes the forever-skipped commit fixture and reports
the attribution categories rather than assuming pins are bounded.

### Owner review still required: withdrawal reachability

The 1,000-cycle probes found self-withdrawal retention of 999 retired records
and mutual retention of 1,998 even with the archive drained after every event.
A reachable chain retains 1,999; eliminating isolated cycles would therefore
not establish a universal bound. The owner authorized the
[design amendment and paired fixtures](../../experiments/departure-gate/WITHDRAWAL-REACHABILITY-DESIGN.md)
for review only. No collector, pin weakening, new rejection or withdrawal
semantic change was implemented. The broad save-bound argument remains
unresolved; exact own grounds and genuinely required reasons remain obligations.

The build receipts identify a compiled dirty working build based on `7f551e5`
with source fingerprint and WASM hashes, not a clean published release.
Linux, Lean, independent reproducibility and the remaining browser matrix are
remote CI/release checks. Series windows, View 0.2 and later rc.16 roadmap items
have not been folded into this departure continuation.
