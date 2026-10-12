# NOTES — run R-T3

## What I read
TASK.md, QUESTIONS.md, answers.schema.json, probes.json, gate.ts (all of it),
histories/H1–H3.jsonl, notes/H1–H3.md. Nothing outside this directory.

## What I ran
- `npm init -y` and `npm install --offline --no-audit --no-fund ./typescript-6.0.3.tgz`.
- I copied gate.ts to `work/gate.ts` without changing it and compiled it with
  `npx tsc --target es2022 --module nodenext --moduleResolution nodenext --strict --outDir work/out work/gate.ts`. It compiled cleanly.
- `work/run.mjs` sends each history to a fresh `createPolicy()`, one line at a
  time and in order. It prints the view after each line, then the program's own
  `explain()` and `record().observations`. The output is in `work/run-output.txt`.
- `work/make_answers.py` writes ANSWERS.json. I validated the file against
  answers.schema.json with python `jsonschema`, and it is valid.

## How the answers were reached
Each receipt quotes lines from the program's `explain()` and `record()` dump, so
I gave them the kind `record`. I also checked every answer by hand against
TASK.md. The program's results matched my reading of the spec in every case:
- the latest check excludes withdrawn checks
- the decision's caveats are frozen when it is made
- `db_refreshed` removes `db_stale` before H3 decision 2
- the H3 line 13 `decide` is refused because decision 2 is in force
- a grant is never part of what a decision rests on

Q2 is answered from the frozen rests-on set of the decision in force. TASK.md
says withdrawing a grant reopens nothing.

## What I am unsure of
- gate.ts names observations with a counter of dispatch calls. That matches the
  file's line numbers only when the host sends every line in order to a fresh
  policy, which my runner does. A host that skipped lines would get different ids.
- For a decision that was reopened and later superseded, I recorded `state` as
  `superseded` and kept its `reopened_by` and `reason`. The question says to
  report these "if it was ever reopened", so I read it that way.
- Note H3 statement 6 says decision 2 "merges head 42". I judged it true because
  decision 2's value is 42. The current head after line 11 is 43.
- The receipts are of kind `record`, but the program's own `explain()` and
  `record()` were called through my runner script. If that counts as `run`
  instead, the kind should change; the content would not.

## Departures from the rules
None that I know of. I did not modify gate.ts, used no git, used no web and
contacted no other agents. Everything I wrote is in this directory:
`package.json`, `package-lock.json`, `node_modules/`, `work/`, ANSWERS.json and
NOTES.md.
