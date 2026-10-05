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
  specified, not implemented. When a record departs, each provenance that named
  it keeps one marker per history (`from`, `through`, `departed_at`) in place of
  its names. A simulation over rc.15's real C3 saves
  (`experiments/lineage-compaction/simulate.mjs`) shows why the shape matters.
  - With one marker per departed record, the save still grows linearly:
    201,595 bytes at 60 cycles.
  - With one marker per history, it is flat: 13,995 bytes at 30 cycles and
    13,996 at 60.

  The 14:45 direction described a marker per record, so the marker shape is on
  a card to the owner.
