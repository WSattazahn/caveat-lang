# NOTES (run R-C2)

## What I read
- `TASK.md`, `QUESTIONS.md`, `answers.schema.json`, `probes.json`, `gate.cav`,
  `histories/H1-3.jsonl`, `notes/H1-3.md`.
- From the installed package (`node_modules/caveat-lang/`): `docs/AGENT_START.md`,
  `docs/HISTORY.md`, the parts of `docs/REFERENCE.md` covering qualify/unqualify, renew and
  predicates, `docs/reference/spec/caveat-take-back-0.1.md`, the `rests_on_withdrawn` section of
  `caveat-withdrawal-0.1.md`, and part of `caveat-explanations-0.2.md` (grounds compared with lineage).

## What I ran (all in this directory)
- `npm init -y` and `npm install --offline --no-audit --no-fund ./caveat-lang-0.1.0-rc.19.tgz`. These created
  `package.json`, `package-lock.json` and `node_modules/`.
- `caveat-lang validate gate.cav` (it loads) and `caveat-lang check gate.cav` (no warnings).
- `caveat-lang explain gate.cav histories/Hn.jsonl` for each history. The output is in `work/explain_Hn.txt`.
- `caveat-lang replay ...` for each history. The output is in `work/replay_Hn.jsonl`. I read the view after every
  line, and the final `decision_journal`, `commitment_grounds`, `commitment_bases`,
  `commitment_permissions`, `withdrawals` and `taken_back`.
- `caveat-lang dependents gate.cav <name> histories/Hn.jsonl` for every probe item, and for `db_stale` in H3.
  The output is in `work/dependents.txt`.
- I made an instrumented copy, `work/gate_instr.cav`. It is gate.cav plus a binding that shows
  `carries(scanner, db_stale)`. I replayed it on H3 (`work/replay_H3_instrumented.jsonl`).
- Diagnostic variants `work/varA/B/C.cav` drop the decide guards at lines 145-146. I used them only to look for
  the source of the "kept" db_stale (see below), and none of them supports an answer.
- `work/make_answers.py` generates `ANSWERS.json`, and I checked the file against `answers.schema.json`
  (it is valid).
- `gate.cav` was not modified.

## Main findings
- For all three histories, the program's decision record matches the task. Every decision rests on the latest
  check, review and scan, by line. The permitting grant is line 2 in every case.
- H2: the scan at line 7 reopens decision 1 with "scan found issues". The recheck at line 10 withdraws check #3
  and reopens decision 2 with "check withdrawn". Decision 3 rests on #11, #4 and #8.
- H3: line 13's `decide` is refused ("a merge decision is in force"), so decision 2 (head 42) stays in force
  even though the head is now 43.
- Note errors: H1 statements 3 and 5, H2 statements 2 and 4, and H3 statements 3 and 4 are false. The rest
  are true.

## What I am unsure of / caveats on my answers
- **H3, decision 2 and db_stale.** `explain` prints `merge@2 ... retaining: deps_unreviewed, db_stale` and
  `scanner supports ready  caveats: db_stale`. `commitment_bases.merge@2` has `kept: ["db_stale"]`, and
  `commitments.retained` lists db_stale. Other evidence says the scan did not carry db_stale at line 9:
  - `taken_back` records that db_stale was taken back from `scanner` at sequence 8;
  - the instrumented `carries(scanner, db_stale)` is 0 at lines 8 and 9;
  - the decision journal and `commitment_grounds` for merge@2 list only `deps_unreviewed`.

  I therefore answered caveats `[]` and retains `["deps_unreviewed"]` for H3 decision 2, and judged
  H3 statement 3 false. The take-back spec says a value "keeps" a caveat when it reached it through a frozen
  record such as a decision series. I believe the kept db_stale is lineage from merge@1's reopening record,
  not a caveat on the three observations. I could not pin down the exact read that brings it in: removing
  either or both decide guards at lines 145-146 did not remove it. If a reviewer counts kept lineage as "the
  caveats it was made with", H3 decision 2's caveats would be `["db_stale"]` instead.
- **Q2 rests on frozen grounds, not on lineage.** In the runtime graph, `relies_on` for a decision also covers
  lineage. For example, merge@3 in H2 `relies_on` scanner (#5), checks@1 and requeue. Following the task and
  `rests_on_withdrawn` (which "compares ... frozen grounds ... not its lineage"), I answered H2 probe 5 `no`.
  No event withdraws a scan, so this answer is from the record, not from a run.
- **H1 decision 1 caveats.** I listed only `secondhand`, the caveat the observations carried. `deps_unreviewed`
  is reported separately under retains, although the journal lists both together.

## Departures from the rules
- Before I started, the harness put the contents of an `AGENTS.md` file from another directory
  (`/home/claude/caveat-lang`) into my context. I did not open, list or use any file outside this working
  directory, and nothing in that text was used for the answers.
- I did not use the web, git or any other agent.
