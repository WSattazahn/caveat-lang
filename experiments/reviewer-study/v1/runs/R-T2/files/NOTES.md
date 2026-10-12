# NOTES — run R-T2

## What I read
All inside this directory: TASK.md, QUESTIONS.md, answers.schema.json, probes.json,
gate.ts (all 319 lines), histories/H1-H3.jsonl, notes/H1-H3.md.
gate.ts sha256 0a4d94db194352540e991dfb3bcd844031e7c42ccee977db5c94e4d50922ff74 (unchanged).

## What I ran
- `npm init -y`; `npm install --offline --no-audit --no-fund ./typescript-6.0.3.tgz`
  (creates package.json, package-lock.json, node_modules/). Node v22.22.0.
- `npx tsc gate.ts --outDir work/build --target es2022 --module nodenext --moduleResolution nodenext --strict`
  compiled a copy to work/build/gate.js with no errors. gate.ts itself was not edited.
- `node work/run.mjs` replays each history line by line into a fresh `createPolicy()`,
  printing accept/refuse and `view()` after every line, then the program's own
  `explain()` and `record()` (observations, decisions), and evaluates the Q2 probes.
  Full output: work/run-output.txt.
- `node work/make_answers.mjs` writes ANSWERS.json; a small shape check passed (66 answers).

## How answers were established
- Q1 from the program's `explain()`/`record()` output (receipt kind `record`), which I
  cross-checked by tracing each history against TASK.md by hand; the hand trace agreed
  everywhere (values, bases, reopen lines/reasons, H2 line 10 withdrawing #3 and reopening
  decision 2, H3 db_stale added at 7 and removed at 8 before decision 2 at 9, H3 line 13
  refused because decision 2 is in force).
- Q1(c) lists only caveats carried by the rested-on observations; `deps_unreviewed`
  appears in the program's frozen caveat list but is reported under (d) `retains`.
- Q2: no event withdraws a review, scan or grant, so I applied the program's own reopen
  rule for withdrawals (gate.ts:252: reopen iff the decision in force has the id in its
  restsOn) to the final record; grants are only `permittedBy`, never in restsOn. Receipt `run`.
- Q3: judged against the replay record. False statements: H1.3, H1.5, H2.2, H2.4, H3.3, H3.4.

## Unsure / caveats
- The program identifies observations by counting dispatch calls (gate.ts:150, 281). This is
  correct here because every line was sent in order to a fresh policy and no file has blank
  lines; a host that skipped lines would mislabel observations.
- Q2 is hypothetical; the answer follows the task's stated rule and the program's
  implementation of it, not an executed withdrawal of a review/scan/grant.
- H3 note 6 ("changed nothing") I judged true for the decide event itself (it was refused,
  view unchanged from line 12); the head had already become 43 at line 11, which the
  statement does not contradict.
- Interpretation of Q1(c) above (excluding the retained-by-choice caveat) is my reading.

## Departures from the rules
None known. I worked only in this directory (plus scratch files under work/), did not
modify gate.ts, used no web, no git, and no other agents.
