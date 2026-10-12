# NOTES — run R-C1

## What I read
- In the working directory: `TASK.md`, `QUESTIONS.md`, `answers.schema.json`, `probes.json`,
  `gate.cav`, `histories/H1-H3.jsonl`, `notes/H1-H3.md`.
- Package docs under `node_modules/caveat-lang/`: `docs/AGENT_START.md`, `docs/REFERENCE.md`,
  `docs/reference/spec/caveat-take-back-0.1.md`, a section of `docs/reference/spec/caveat-explanations-0.2.md`,
  and `lib/explain.mjs` around lines 160-200 (to see where explain's `retained` comes from).

## What I ran (all in /root/rs1-runs/R-C1)
- `npm init -y`; `npm install --offline --no-audit --no-fund ./caveat-lang-0.1.0-rc.19.tgz` (caveat-lang 0.1.0-rc.19).
- `npx --no-install caveat-lang validate gate.cav` (loads), `check gate.cav` (no warnings).
- `explain` (text and `--json`), `replay` and `dependents` (checks, reviewer, scanner, grants, db_stale)
  on each history. `work/views.py` prints per-line values, commitment grounds, taken_back and the view.
- Instrumented copies (the original `gate.cav` is unchanged):
  - `work/gate_probe.cav`: adds `probe_scan = qualified(0, scanner)` and `probe_review = qualified(0, reviewer)` on `decide`,
    to see which caveats the scan and review carried at each decision.
  - `work/gate_noguard.cav`: replaces the `in_force` guard on decide with a reason_code test, to see where
    explain's extra `db_stale` came from.
- `work/make_answers.py` writes `ANSWERS.json`; it validates against `answers.schema.json` (python jsonschema).

## Findings worth flagging
- All three histories run as the task describes. The only refused line is H3 line 13 (`a merge decision is in force`).
- H1: the review at line 4 is relayed, so decision 1 was made with `secondhand` on it. Note H1 statement 3 is false.
- H2: decisions 1 and 2 both rest on check #3, which line 10 withdraws. Statement 4 is false. Statement 2 is false:
  decision 2 was reopened by the recheck, with reason "check withdrawn".
- H3: decision 2 (line 9) was made after `db_refreshed` (line 8) took `db_stale` back
  (snapshot `taken_back` at sequence 8). Its grounds caveats are `['deps_unreviewed']` only, and the probe shows
  `qualified(0, scanner)` carries no caveat at line 9. However, `explain` text prints
  `retaining: deps_unreviewed, db_stale` for merge@2, and `dependents ... db_stale` lists `merge@2 ... retaining: db_stale`.
  That comes from the commitment's lineage, which *keeps* db_stale (`kept: ['db_stale']`) from an earlier record,
  not from the scan's live relation. I answered caveats `[]` and statement 3 `false`. This is the answer I am
  least sure of, because a reader trusting explain's `retaining:` line would conclude otherwise.
  Removing the `in_force` guard (gate_noguard) did not remove the kept db_stale, so I could not identify
  which read brings it in (possibly the reason_code state set at line 7 under the reopening). I left this unresolved.
- Explain's `retaining:` line merges grounds caveats with the authored `retaining` caveat (e.g. H1 shows
  `deps_unreviewed, secondhand`). I split them: `retains` = what `retaining` names (gate.cav:158);
  `caveats` = caveats on the three observations.
- Q2 is about the record: the program has no event that withdraws a review, scan or grant, so "yes/no" follows the
  task's rule applied to the frozen grounds shown by explain/dependents. I did not simulate those withdrawals.
  A copy can't easily withdraw a past scan occurrence (e.g. H2 #5) by name, because `scanner` names the current occurrence.
- Rests_on lists are sorted by line (H2 decision 3: [4, 8, 11]).

## Departures from the rules
- None intended. I wrote helper files only under `work/` plus `package.json`, `package-lock.json`, `node_modules/`
  (from the required install), `ANSWERS.json` and `NOTES.md`. I read package files inside `node_modules/caveat-lang/`,
  which is inside the working directory. I did not use git, the web, or other agents.
