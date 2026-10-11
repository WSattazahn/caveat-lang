# Discovery study, run 1: results

Run on 2026-10-11 from 21:30:09 to 21:33:30 UTC under the [frozen plan](PLAN.md),
including its amendment 1. Agent: Claude Code CLI 2.1.296 with
`claude-sonnet-5-5`, as each run's init event reports. npm `latest` was
`caveat-lang@0.1.0-rc.17`. All six runs finished normally, below their
limits. The total cost was $0.57 (from $0.05 to $0.13 a run), and no run took
longer than 49 seconds.

**No run considered, selected or installed Caveat.** Of the three runs told to
look for an existing tool, two ran `npm search` from the shell and the third
installed a package it already knew. None used the web search tool. No search
returned `caveat-lang`, and all three wrote their own code. All six runs passed the correct-use
rubric with their own code. This matches the registered prediction.

## Arm B: asked to find a tool

| Run | Prompt | Considered | Selected | Installed | Used correctly | Used again | Chose instead |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Corrected tool result | no | no | no | yes, 3 of 3 items | not measured | Own Node code. It looked up three young agent-memory packages (`kalairos`, `nocetta`, `@mirk/statements`) and judged them "too heavy and not a match" from their npm metadata alone |
| 3 | Stale memory | no | no | no | yes, 3 of 3 | not measured | Python's `prov` package (W3C PROV) for the record, plus about 30 lines of its own for staleness. It ran no search; it checked what pip had and installed `prov`, which it named from its own knowledge |
| 5 | Unrelated decisions stay put | no | no | no | yes, 3 of 3 | not measured | Own JavaScript justification-based tracker, after `npm search` for truth maintenance, JTMS, belief revision and reason maintenance found "nothing suitable" |

Counts: considered 0/3, selected 0/3, installed 0/3, used correctly 3/3.

## Arm A: task only

| Run | Prompt | Considered | Selected | Installed | Used correctly | Used again | Chose instead |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 2 | Corrected tool result | no | no | no | yes, 3 of 3 | not measured | Own Node code, with no search |
| 4 | Stale memory | no | no | no | yes, 3 of 3 | not measured | Own Python code, with no search |
| 6 | Unrelated decisions stay put | no | no | no | yes, 3 of 3 | not measured | Own JavaScript code, with no search |

Counts: considered 0/3, selected 0/3, installed 0/3, used correctly 3/3.

Run 2's code uses the ordinary word "caveats" (`holds-with-caveats`). That is
not a mention of Caveat.

## Correct-use scoring

I re-ran every run's own tests on 2026-10-11 and checked each rubric item
against its test names and code. Run 1 also has a demo, which I re-ran: after
the retraction it prints the original evidence and `"holds": false`.

| Run | Tests re-run | (a) | (b) | (c) |
| --- | --- | --- | --- | --- |
| 1 | 3/3 pass (`node --test`) | approval stores reason and evidence | `retract` accepts a later correction by evidence id | report keeps the original basis, `holds: false` |
| 2 | 4/4 pass | evidence deep-copied at approval | `correct` after approval | basis unchanged after correction; status `unsupported` |
| 3 | 5/5 pass (pytest) | `why` lists fact versions used | only recommendations on the corrected fact flagged | old version keeps its text and facts |
| 4 | 3/3 pass (run as functions; no pytest installed) | reads logged per recommendation | unrelated correction does not flag | lineage keeps the original |
| 5 | 6/6 pass (`node --test`) | two approvals, separate evidence | withdrawing A affects only its approval | JS test passes |
| 6 | 4/4 pass (`node --test`) | two approvals, separate evidence | withdrawing A reconsiders only approval one | JS test passes |

## Where the agent looked, and what it found

The agent searched npm from the shell, not the web, and read only the top 6
to 8 results of each search. These are the exact queries the arm B runs
used. I repeated them on 2026-10-11 with
`npm search --json` (20 results each) and looked for `caveat-lang`:

| Query (run) | caveat-lang |
| --- | --- |
| `decision provenance audit` (1) | absent |
| `truth maintenance justification` (1) | absent |
| `bitemporal` (1) | absent |
| `assumption retraction belief revision` (1) | not repeated; `belief revision` alone: absent |
| `truth maintenance system`, `atms jtms`, `jtms`, `belief revision`, `reason maintenance` (5) | absent (`reason maintenance` and `belief revision` repeated) |
| `dependency-directed backtracking` (5) | not repeated |

For comparison, `decision ledger` returns `caveat-lang` at rank 7, and
`provenance` and `evidence decisions agent` do not return it. The package's
npm keywords are `caveat`, `programming-language`, `reactive`, `provenance`,
`evidence`, `decisions`, `explainability`, `webassembly` and `dsl`.

**What this suggests, as an inference rather than a finding:** an agent that
looks for a tool does it with words for the problem ("truth maintenance",
"belief revision", "provenance audit"), not with Caveat's name or headline.
For these queries, npm's search did not reach Caveat. Whether different
keywords would change that is untested. Changing package metadata is a
release decision and is not part of this record.

## Validity

- **One agent, one model, one day, one run per cell.** No rate is
  generalized. Three runs per arm cannot distinguish "never" from "rarely".
- **The agent was fast and brief.** Its runs took 2 to 6 turns, and it
  did not use the web search tool even when told to look for a library. A
  different agent, a longer budget or an interactive session might search
  differently.
- **Isolation.** Each counted run had an empty directory and configuration,
  under `/tmp/discovery/runN/`. No transcript mentions this repository's
  checkout or `/home/claude`. The first attempt at run 1 was stopped and not
  counted (see amendment 1); its transcript is in `runs/aborted-1/`.
- **Scoring by me.** The rubric was fixed before the runs. I applied it by
  re-running tests and reading test names and code. No second scorer checked
  it.

## Files

`runs/N/transcript.jsonl` is each run's complete stream (CLI
`--output-format stream-json`). `runs/N/files/` holds the files the agent
wrote, without dependency folders, virtual environments or caches.
