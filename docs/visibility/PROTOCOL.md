# Search-visibility protocol V1 (frozen)

This protocol repeats the [V1 baseline](v1/PROVENANCE.md) so later runs can be
compared with it. It measures whether a search tool returns Caveat for a fixed
set of queries. It does not measure selection, installation, correct use,
repeat use or demand, and it is not a fresh-agent discovery study.

Frozen on 2026-10-10 under the [Path to 1.0](../releases/path-to-1.0.md) plan,
PR C3. Changing anything in "Queries" or "Running a check" makes a new
version (V2); V1 history keeps its label.

## Queries

Problem queries:

| ID | Query |
| --- | --- |
| P1 | `javascript library track evidence behind decisions` |
| P2 | `agent decision audit trail corrected tool results` |
| P3 | `rules engine reconsider decisions when facts change` |
| P4 | `invalidate approval when supporting evidence is withdrawn` |
| P5 | `preserve original decision reasons after new evidence` |
| P6 | `MCP tools test and explain decision policies` |

Controls, reported separately from the problem queries:

| ID | Query |
| --- | --- |
| C1 | `caveat-lang` |
| C2 | `caveat agent MCP` |

P4 and P5 have weak software intent (see the
[V1 annotations](v1/PROVENANCE.md#how-to-read-it)). They stay in V1 unchanged.

## Running a check

1. Run each query separately, with the same provider and tool as the run it is
   compared with, and the same settings where the tool exposes them.
2. Request up to ten results. Record what is returned; never fill in missing
   results or assign ranks the tool did not give.
3. Do not reword a query, add filters or operators, or add queries after
   seeing results.
4. If retrieval fails, record the query as **check unavailable**, not as
   Caveat absent.
5. Do not install anything during a visibility check. An install needs its own
   pre-registered study.

## What each run records

Per run:

- UTC start time, and per-query times when the tool returns them.
- Provider and tool name; model identity when exposed; locale and
  personalization when exposed, otherwise "not exposed".
- The complete available tool response for every query, kept unchanged.
  Summaries may be added beside it, never instead of it.

Per query:

- Returned count, then each entry's URL, title, snippet and position in the
  tool's return order.
- The actual identity of each entry that mentions Caveat: this project's
  package or repository, or another product (for example `caveat-cli`,
  `@yodolabs/caveat-mcp`).
- Any version shown in a matching snippet. Mark whether it is stale
  install advice or a legitimate historical reference (a release record or a
  proof page naming the rc it was checked on).

Distribution state, recorded separately from search results:

- MCP Registry latest for `io.github.WSattazahn/caveat-lang`: version, status,
  `isLatest`, description and launch arguments.
- npm `latest` and `next` for `caveat-lang`, with publish times.
- The owner-approved published package for the current release
  (its `docs/releases/vX-npm-publication.json`).
- Each mismatch among these, and whether it is a recorded intentional lag.

## Where runs go

Each run is a new directory `docs/visibility/runs/<UTC time>-V1/` with the raw
responses and a short `RUN.md` holding the fields above. Earlier runs are
never edited. `RUN.md` lists every change to metadata, registry entries or
documentation since the previous run.

## How results may be used

- A changed result is an observation. It does not show that a metadata change
  caused it.
- Return order is not adoption evidence and not organic rank.
- Repeat V1 after an approved metadata change. A recurring schedule needs the
  owner's approval; none is set up by this protocol.

## Separate: fresh-agent discovery study

Whether an agent, given a real task and no hint, finds, chooses, installs and
uses Caveat is a different question. That study is
[designed separately](DISCOVERY_STUDY.md) with frozen task prompts that never name Caveat, its slogan,
syntax or URL, and it needs its own approval to run.
