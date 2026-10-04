# Design note: a save that forgets what the program forgot

Status: a design note for the owner's decision (rc.13 plan, PR 10). Nothing
here is implemented or promised, and no specification changes with it.
Receipts are `path: Lx–Ly` at `main` `8b1d2f3`.

## The problem

Glowcap round 7's first work item
(`experiments/glowcap/RESULTS.md: L917–920`): CR13 asks the slime to remember
only its six most recent observations, and asks that the 4,096-byte save
bound hold "for play of any length". Every Caveat author wrote the window in
source, so the belief forgot. Nothing forgot in the session. The save kept
every relation, renewal occurrence, journal entry and revision basis, and it
grew (`experiments/glowcap/RESULTS.md: L697–718`):

| Regrowth cycles | C1 | C3 | C4 | TypeScript (T1–T3) |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 2,349 | 3,714 | 5,869 | 338–506 |
| 60 | 6,753 | 16,310 | 40,661 | 345–514 |

C1, C3 and C4 compressed the save in their adapters, which slowed the growth
without bounding it. C2 saved its accepted events instead, and its resume
took 112 seconds after 60 cycles. C3 measured its uncompressed save growing
roughly quadratically, which it attributes to each trust revision's basis
including the earlier ones (`experiments/glowcap/round7/runs/C3/notes.md:
L120–131`). C4 named the missing capability as "bounded or compactable
history" (`experiments/glowcap/round7/runs/C4/notes.md: L59`). The TypeScript
authors bounded their state by dropping what nothing still cited, and kept a
short id for anything a frozen basis or the journal still named
(`experiments/glowcap/round7/runs/T1/notes.md: L43`).

This is the language working as specified. The record keeps everything:

- "Capacity limits reject an event atomically rather than evicting evidence
  silently" (`docs/CAVEAT_ESSENCE.md: L49`).
- Reading streams and decision series: "There is no eviction, silent history
  truncation, or published identity reuse" (`spec/caveat-reactive-0.5.md:
  L210–223`).
- The decision journal is append-only (`spec/caveat-decision-journal-0.1.md:
  L48–52`) and has no limit of its own.
- [Withdrawal](../../spec/caveat-withdrawal-0.1.md) records that an
  observation is no longer stood behind and removes nothing
  (`spec/caveat-withdrawal-0.1.md: L21–23, L59–62`).

So the session's record is bounded today only by declared limits (256 records
per stream or series and 1,024 in all, 1,024 occurrences per renewable
evidence), and reaching a limit refuses the event (`limit/history_limit`,
`limit/renewal_limit`). Neither gives play of any length: the save grows to
the limits, then the program stops accepting the events that would grow it.

## What "forgotten" could mean

Three meanings, from least to most change:

1. **Out of the belief.** The program stops counting the observation. Authors
   can already write this (C2's six-slot shift register,
   `experiments/glowcap/round7/runs/C2/notes.md: L52`). It changes nothing in
   the record.
2. **Out of the live graph.** The observation no longer satisfies
   `observed(...)`, its relation no longer counts in any new read, and new
   values cannot cite it. Records that already cite it (a frozen basis, a
   journal entry, a state's lineage) keep citing it. The record still holds
   it.
3. **Out of the session.** The observation's records leave what `save()`
   writes. Only this bounds the save.

The invariants above allow the first meaning. The second and third need the
owner's decision, because they replace "nothing is evicted" with "eviction is
declared, recorded and never retroactive".

## Rules any option must keep

Whatever is chosen:

- **Authored, never silent.** Nothing leaves the live graph or the session
  unless the source declares the policy that removes it. Programs without such
  a declaration behave, save and restore exactly as now.
- **Never retroactive.** A frozen basis, its grounds and a written journal
  entry are not edited. If they cite an occurrence, that occurrence stays in
  the session until nothing in the session cites it ("pinned").
- **Transactional.** Forgetting happens inside an accepted event. A refused or
  failed event forgets nothing, and occurrence numbering never reuses a name.
- **Reported.** The event's `effects` report what left, as `renew` and
  `withdraw` are reported now.
- **Restore stays what it is.** Under A3, restore refuses only what no source
  mechanism can produce. It does not verify that a save is the original
  account of a session, and a compacted save is no different: restore cannot
  know what was forgotten, only whether what remains is consistent.

## Option A: declared windows with an archive hand-off (recommended)

A history declaration can carry a window in place of a limit. Syntax is
illustrative only:

```caveat
renewable taste_$m window 2;      -- the current life and the one before
decisions trust window 8;         -- the latest eight revisions
journal window 32;                -- the latest 32 journal entries
```

When an event would go past a window, the oldest record **retires** instead of
the event being refused. A retired occurrence leaves the live graph (meaning 2)
at once. Its records leave the session (meaning 3) once nothing in the session
pins it: no state lineage or grounds, no frozen basis of a revision inside its
series' window, no journal entry inside the journal's window, no pending
scheduled qualification and no withdrawal record. Each retirement and each
departure is an effect of the event that caused it.

What leaves is handed to the host, not erased. The session keeps an
append-only **archive** of departed records, which the host drains (for
example `session.drain_archive()`, returning records in the order they left).
A host that needs the full history stores the drained records next to its
saves. A host that does not, discards them. Either way `save()` holds only
what is still in the session, so a program whose every growing history has a
window has a save bounded by its windows, whatever the length of play.

- **`explain`.** A value cannot cite a departed record, because a citation
  pins what it cites. `explain` of a retired but pinned occurrence shows it
  with the sequence at which it retired. Given a departed name such as
  `taste_cave@3` (recognizable because its number is below the stream's
  retired count), `explain` reports it as archived rather than unknown, and
  the kit's `explain` can read a drained archive to show it in full.
- **`dependents`.** Nothing in the session rests on a departed record, by the
  pinning rule. A query returns empty arrays and marks the subject archived.
  For a retired but pinned occurrence it lists what pins it, which is exactly
  the question "why is this still here".
- **Restore validation.** The save gains, per windowed history, the count of
  records that have left; entry N of a stream becomes `STREAM@(retired + N)`.
  New refusals: a count past what the declaration allows to remain, a gap or
  reuse in numbering, and any citation of a name below the count that no
  pinned record holds. Restore cannot tell whether the archive the host kept
  matches the save and does not claim to.
- **Migration.** Adding a window edits the source, so its `source_id` changes
  and saves of the old program do not restore into it; this is already true of
  any edit (`spec/caveat-routed-repetition-0.1.md: L282–289`). Programs
  without a window write the same `caveat-reactive-save/0.1` saves. A windowed
  save carries fields 0.1 lacks, so a runtime that predates windows refuses it
  (F247's rule, `spec/caveat-save-0.1.md: L289–298`) rather than misreading it.
  Whether that warrants a new schema name is an implementation detail.

Open questions for implementation: whether CR13's window across all
observations (one window over several evidence names) is needed in the runtime
or stays in source, where the round's authors already wrote it; and how a
decision series' revision chain stays bounded when, as C3 reports, a basis can
include its predecessor's.

## Option B: retire in place

The same declared windows, with meaning 2 only. A retired occurrence leaves
the live graph and its records stay in the session, marked retired. `explain`
and `dependents` work unchanged on everything, restore validation gains only
a check that a retired record has no live relation, and there is no archive
for hosts to handle. The save keeps growing, more slowly, because a retired
record is smaller than a live one. Play stays bounded by the declared limits.
This answers "the slime forgets" in the runtime and leaves CR13's save bound
unmet.

## Option C: keep everything, and say so

No runtime change. The specifications and the authoring guide state that the
session record grows with every observation and decision change until a
declared limit refuses the event, that a save is bounded only by those limits,
and that a host needing a smaller save stores accepted events and replays them
(C2's resume cost) or compresses the save text (C1, C3, C4). A program can
express forgetting in its belief (meaning 1) but cannot meet a byte bound for
play of any length, and Glowcap-like requests stay a recorded limit of the
language.

## Decision

**Question:** what may Caveat forget, so that a long-running program can keep
a bounded save?

- **A. Windows with an archive hand-off.** Declared windows retire old records;
  unpinned records leave the save and go to a host-drained archive. Saves stay
  bounded for any length of play; the "nothing is evicted" invariant becomes
  "eviction is declared, recorded and never retroactive".
- **B. Retire in place.** Declared windows take old records out of the live
  graph, but the save keeps them. Belief-level forgetting moves into the
  runtime; saves still grow until a limit refuses events.
- **C. Keep everything.** No change; document that saves grow until declared
  limits refuse events, and that hosts bound storage by replay or compression.

**Recommendation: A.** It is the only option that meets round 7's request, and
it does so without erasing anything: what leaves the save is handed to the
host in order, the pinning rule keeps every frozen basis and journal entry
explainable while the session cites it, and programs that declare no window
are unchanged. Its cost is a change to a stated invariant, which is why it is
the owner's call. It would need its own specification and review before any
implementation, and is not planned for rc.13.
