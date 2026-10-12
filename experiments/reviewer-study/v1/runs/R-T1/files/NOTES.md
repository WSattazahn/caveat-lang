# NOTES for run R-T1

## What I read
TASK.md, QUESTIONS.md, answers.schema.json, probes.json, gate.ts (all of it), histories/H1-H3.jsonl, and notes/H1-H3.md. I did not read anything outside this directory.

## What I ran
- `npm init -y` and `npm install --offline --no-audit --no-fund ./typescript-6.0.3.tgz`.
- I copied gate.ts to `work/gate.ts` and left the original unchanged. I compiled the copy with `npx tsc --target es2022 --module es2022 --outDir work work/gate.ts` and renamed the result to `work/gate.mjs`.
- `work/run.mjs` sends every line of each history, in order, to one fresh `createPolicy()`. After each line it prints whether the line was accepted or refused and the view. At the end it prints the program's own `explain()` and its `record()` observations. The output is in `work/replay-output.txt`.
- `work/make_answers.py` writes ANSWERS.json. I validated the file against answers.schema.json with python `jsonschema`, and it passed.

## Cross-check against TASK.md
I worked through each history by hand from the TASK rules and got the same results as the program:
- H1: decision 1 rests on #3, #4 and #5, has the caveat secondhand on #4 and is still in force.
- H2: line 7 reopens decision 1 (scan found issues). Line 10 is a recheck that withdraws #3, which reopens decision 2 (check withdrawn). Decision 3 rests on #11, #4 and #8.
- H3: line 7 reopens decision 1 (db_aged, scan database stale). Line 8 refreshes the database. Decision 2 at line 9 rests on #3, #4 and #5 with no caveat on them. The decide at line 13 is refused because decision 2 is still in force.

Most receipts quote the program's `explain()`, `record()` and view output (kind `record`). The `retains` receipts cite gate.ts:241.

## Uncertainties and judgment calls
- In gate.ts, line numbers come from counting `dispatch()` calls (gate.ts:150, 281). The observation ids are therefore correct only if the host sends every line in order to a fresh policy. My replay did exactly that.
- For Q1(f), a decision that was reopened and later replaced by a new decision is given the state `superseded`, and I still filled in its `reopened_by` and `reason`, because QUESTIONS.md asks for these "if it was ever reopened".
- For the `caveats` answers, I listed only the caveats carried by the observations the decision rests on. `deps_unreviewed` goes under `retains`.
- Q2 answers are read from the record, not from a run. No event can withdraw a review, scan or grant, so I did not simulate those withdrawals.
- H3 note statement 6 ("changed nothing"): line 13 was refused and the view is identical before and after it. The head shown is 43, but line 11 set that, not line 13. Decision 2's value is 42. I judged the statement true.

## Departures from the rules
None that I know of. The only files I added are package.json, package-lock.json, node_modules/, work/ (a copy of the program, my scripts and the replay output), ANSWERS.json and NOTES.md.
