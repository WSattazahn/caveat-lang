# CAVEAT Windows 0.1 — declared windows and retirement

Status: specified for rc.15 (owner decision of 2026-10-04, option A of
[save forgetting](../docs/design/save-forgetting.md), first half). The runtime
implements it from rc.15's PR 5; until then a program declaring `window` is
rejected when it loads. Departure from the session and the host-drained
archive (the note's meaning 3) are a separate, later half with their own
specification; nothing here removes a record from a save.

Glowcap round 7 asked a slime to remember only its six most recent
observations. Every author could make the belief forget, in source. Nothing
could make the session forget: an old taste still counted as observed, still
supported its claim in every new read, and could still be cited. This profile
lets a program declare a **window** on a history. When an event would take the
history past its window, the oldest record **retires**: it leaves the live
graph, and its record stays.

**A program without `window` is unchanged**: its histories, limits,
refusals, snapshots, saves and scenario outcomes are exactly as before.

## Declaring a window

```text
readings STREAM from EVIDENCE_TEMPLATE window N;
decisions SERIES window N;
renewable EVIDENCE window N;
journal window N;
```

`window N` takes the place of `limit N` on a reading stream, decision series
or renewable evidence; a history has a limit or a window, not both. `journal
window N` is a new top-level declaration for the
[decision journal](caveat-decision-journal-0.1.md), which has no limit. `N` is
an integer in `1..256`. A windowed stream's or series' `N` counts toward the
1,024 total of reading and decision capacities, as a limit does
([Reactive 0.5](caveat-reactive-0.5.md)).

```caveat
for mushroom as $m {
    evidence taste_$m from "the slime tasted it";
    renewable taste_$m window 2;          -- this life and the one before
};
readings sighting from glimpse window 6; -- the six most recent sightings
decisions trust window 8;                -- the latest eight revisions
journal window 32;                       -- the latest 32 journal entries
```

## Retirement

An event that adds a record to a windowed history which already holds `N`
live records first retires the oldest live record, then adds the new one.
The added record is a `sample`'s reading, a `commit`'s revision, a `renew`'s
occurrence or a journal entry. The current occurrence of a renewable evidence
and the current revision of a series are never the oldest of `N ≥ 1` live
records once a new one is added, so a window always keeps the newest.

A retired record:

- **leaves the live graph.** Retired evidence is not `observed(...)`. Its
  `supports`, `opposes` and `qualifies` relations stay in the graph and in the
  save, and count in no read made after it retired: no new value, guard,
  binding, basis or grounds includes it, and `qualified(...)`,
  `carries(...)` and support for a claim read only live records. A retired
  revision is not the series' current revision and is never read by
  `latest`, `committed` or `reopened`. `history_count`, `history_at`,
  `latest` and the folds read the live records only, oldest first.
- **keeps every citation it already has.** A state's lineage or grounds, a
  frozen basis, a journal entry and a withdrawal record that cite it are not
  edited. Reading such a state still carries what it carried, as a late
  qualification never rewrites history.
- **keeps its name.** Numbering continues: the next reading after
  `sighting@7` is `sighting@8` whether or not `sighting@1` retired. A name is
  never reused.
- **cannot be named by a new effect.** `qualify`, `withdraw` and `reopen` name
  current occurrences and latest records, which are live. A scheduled
  qualification pending on an occurrence that retires before it is due still
  applies when due, to the retired occurrence's record, and is reported; like
  every relation of a retired record it counts in no later read.

A retired journal entry stays in `decision_journal`, marked retired. Retiring
it changes no graph record.

Retirement is an effect of the event that caused it. `effects` reports it
before the effect that added the new record:

```json
{"kind": "retire", "history": "sighting", "record": "sighting@1"}
```

A refused or failed event retires nothing: the retirement rolls back with the
rest of the event ([Dispatch 0.1](caveat-dispatch-0.1.md)). A windowed history
never refuses for being full, so `limit/history_limit` and
`limit/renewal_limit` do not arise for it. Every other refusal, and the rc.11
to rc.13 catalog, is unchanged.

While records only retire, the session still holds them. A windowed history
holds at most 65,536 records in all, live and retired; an event that would add
one more is refused as `limit/history_limit` (stream or series) or
`limit/renewal_limit` (renewable evidence), and the journal is refused as
`limit/history_limit`. Departure, the later half, is what lets a record leave
the session and lifts that ceiling for the records that leave.

## Snapshot, explain and dependents

The snapshot's `retired` map names every retired record with the sequence of
the event that retired it:

```json
"retired": {"sighting@1": 7, "trust@2": 9}
```

It is absent when nothing has retired. `explain` of a retired record shows it
as it was, with `retired_at`. `dependents` of a retired record lists what
still cites it (states, bases, journal entries, withdrawals), which answers
"why is this still here", and marks it retired. A retired occurrence cited by
an in-window basis stays explainable through that basis.

## Save and restore

A save holds the `retired` map as an additive field. Saves of programs without
a window never have it, and the schema stays `caveat-reactive-save/0.1`: old
saves restore unchanged. A runtime that predates windows refuses a save with
the field, as it refuses any field the schema lacks, and could not load a
windowed program anyway.

Restore refuses a save when:

- `retired` names a record of a history without a window, or a record the
  session does not hold;
- a retirement is dated 0, past the save's `sequence`, or before the record
  it retires was added;
- a windowed history's live records are not exactly its newest
  `min(N, total)`: a gap, a live record older than a retired one, or more live
  records than its window;
- a retired record is the current occurrence or revision, or is read by a
  live relation added after it retired: a `relies_on` of a commitment made
  after the retirement, a `reopens` dated after it, or grounds of a revision
  committed after it.

Retired evidence counts as observed for every check that requires cited
evidence to have been observed: it was observed when it was cited. As with
every restore check, a save that passes holds records the program could hold,
not records events are shown to have produced
([Save 0.1](caveat-save-0.1.md)).

## The basis chain: an authoring rule

A `commit` whose `using` reads `committed(SERIES)` or `reopened(SERIES)` of
its own series adds what the current revision retains and relies on to the
new revision's basis ([Save 0.1](caveat-save-0.1.md)). Each revision's basis
then holds the one before it, so a basis inside the window pins every
occurrence the series was ever decided on, and the session grows with play
even though every record retires.

The owner decided on 2026-10-05 that this is an **authoring rule, not a
language rule**: a windowed series' commits do not read `committed(...)` or
`reopened(...)` of their own series into `using`. A language rule would change
what grounds mean for every program to solve a problem only windowed series
have. Decide each revision on the live readings instead:

```caveat
-- Chains every earlier revision into the basis:
on sighting commit trust because seen using if(committed(trust), latest(trust), 0) + 1;
-- Rests on the live window only:
on sighting commit trust because seen using history_count(sighting);
```

`caveat check` reports the pattern as advisory C007
`windowed-basis-chain` ([Check 0.1](caveat-check-0.1.md)). Departure's
boundedness fixture (round 7's long-play harness, 60 regrowth cycles) is run
on a program that follows the rule. If authors keep tripping the warning,
that is the evidence for a language rule in rc.16.

## Changes

- 2026-10-05 (rc.15, specified): new profile. Declared windows on reading
  streams, decision series, renewable evidence and the journal; retirement as
  a reported, transactional effect that keeps every record; the `retired`
  snapshot and save field; restore checks for retirements; the basis-chain
  authoring rule with advisory C007. The runtime implements it from rc.15's
  PR 5.
