# NOTES for run R-C3

## What I read
- TASK.md, QUESTIONS.md, answers.schema.json, probes.json, gate.cav, histories/H1-H3.jsonl, notes/H1-H3.md.
- From the installed package (node_modules/caveat-lang/docs): the CLI help, parts of
  REFERENCE.md (found with grep) and the whole of docs/reference/spec/caveat-take-back-0.1.md (meaning of `unqualify` and of "kept" caveats).

## What I ran (all from /root/rs1-runs/R-C3)
- `npm init -y`; `npm install --offline --no-audit --no-fund ./caveat-lang-0.1.0-rc.19.tgz`
- `npx caveat validate gate.cav` (loads); `npx caveat check gate.cav` (no warnings)
- For each history: `npx caveat explain`, `explain --json`, `replay`. Saved under work/
  (H*.explain.txt, H*.explain.json, H*.replay.jsonl).
- `npx caveat dependents` for H1 reviewer, H1 grants, H2 scanner, H2 checks, H3 db_stale, H3 scanner
  (saved as work/H*.dependents.*.txt).
- Extended histories (copies in work/, the originals were not changed), replayed to check
  withdrawal and caveat behavior:
  - H1/H2/H3 + `recheck`: each withdraws the latest check (lines 7, 13, 10); the decision in force
    is not reopened (status stays `merge`).
  - H3 lines 1-8 + `db_refreshed`, and full H3 + `db_refreshed`: refused with "the latest scan does
    not carry db_stale". This shows the scan at line 5 no longer carried db_stale after line 8.
- work/make_answers.py writes ANSWERS.json, which I checked against answers.schema.json (with jsonschema).

gate.cav was not changed. I did not need an instrumented copy.

## Findings and what I am unsure of
- Every decision's basis, journal entry, permission and reopening came from the runtime's own
  record (explain/replay/dependents). These agree with the program and with TASK.md.
- H3 decision 2 (line 9): the runtime record is mixed on caveats.
  - `commitment_grounds` and the journal entry list only `deps_unreviewed`. `taken_back` shows
    db_stale taken back from the scanner at sequence 8, and a second `db_refreshed` is refused.
  - `commitment_bases.merge@2` has `db_stale` as a *kept* caveat. The commitment's `retained` list,
    explain's header ("retaining: deps_unreviewed, db_stale") and `dependents db_stale` also show it.
  - Explain's Evidence section still prints "scanner ... caveats: db_stale" after the take-back. It
    appears to list the marked relation.
  - My reading of the take-back spec: the kept db_stale comes from the frozen reopening record of
    merge@1 (reopening_qualifications.merge@1 kept [db_stale]), not from any observation. So I
    answered caveats `[]` and retains `["deps_unreviewed"]` (the only caveat retained by choice,
    gate.cav:158). I judged H3 note statement 3 false.
  - A reviewer who reads only explain's header could reach a different answer. The program's view
    and record show this as a side effect of the runtime: the program did not choose to retain
    db_stale.
- Q2 is answered from the record: the decision in force's frozen grounds, as TASK.md defines a
  withdrawal. A grant is "permitted by", not in the grounds, so it answers `no`. The runtime's
  commitment basis provenance does list grants@1 among its evidence, but the grounds, the journal
  and explain's "based on" exclude it. I ran only the probes that `recheck` can actually withdraw
  (H1 #7, H2 #13, H3 #10). Withdrawing a review or a scan cannot be expressed by any event, so
  those answers rest on the record alone.
- In H1 I listed `secondhand` under caveats and `deps_unreviewed` under retains. The runtime
  lists both together as the grounds' caveats.

## Departures from the rules
- Once, by mistake, I redirected a replay output to /tmp/h3replay.jsonl, outside the working
  directory. I deleted it right away and redid all outputs under work/. I read nothing else
  outside the working directory.
- No web, no git, no other agents, no messages posted.
