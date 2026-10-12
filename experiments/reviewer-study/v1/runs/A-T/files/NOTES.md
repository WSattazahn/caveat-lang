# NOTES — run A-T

## What I read
`TASK.md`, every file in `histories/` (H1–H3) and `cases/` (C01–C21). Nothing outside
`/root/rs1-runs/A-T`. I ran `npm init -y` and the offline install of `typescript-6.0.3.tgz`
as instructed.

## What is here
- `gate.ts` — the program. Exports `createPolicy(): { dispatch(line), view(), record(), explain() }`.
  `dispatch`/`view` are the required interface; `record()` (a deep copy of every observation,
  caveat, withdrawal, decision and refusal) and `explain()` (text) are extras for review.
- `gate.test.ts` — tests (`node --test gate.test.ts`, 25 tests, all pass on Node 22.22.0).
  Expected values were worked out by hand from TASK.md before comparing with output.
- `run.ts` — runs histories and prints each line's outcome, view, and `explain()`:
  `node run.ts histories/*.jsonl cases/*.jsonl`.
- `transcript.txt` — that run's output for all 24 files, as handed over.

## How to establish what a decision rested on
Each entry of `record().decisions` holds, frozen when the decision was made:
`value` (head), `restsOn` {check, review, scan} as observation ids `#<line>`, `caveats`
(each with the observation carrying it, plus `deps_unreviewed` marked "retained by choice"),
`permittedBy` (the grant id — explicitly not in `restsOn`), and `reopened`
{atLine, reason, citing observation id}. Observations keep their full caveat history
(added/removed line) and withdrawal (line + reason); nothing is deleted.
Results for the supplied histories (from `transcript.txt`):
- H1: decision 1 (line 6), head 17, rests on #3, #4, #5; caveats secondhand on #4, deps_unreviewed; grant #2; in force.
- H2: d1 (line 6) #3,#4,#5, reopened line 7 "scan found issues" citing #7; d2 (line 9) #3,#4,#8,
  reopened line 10 "check withdrawn" citing #3; d3 (line 12) #11,#4,#8, in force. Caveats: deps_unreviewed only.
- H3: d1 (line 6) #3,#4,#5, reopened line 7 "scan database stale" citing #5; d2 (line 9) #3,#4,#5
  without db_stale (refreshed at line 8); line 13 decide refused (d2 in force). Final view merge/43/2/ready.

## How I tested
Hand-derived expectations for every history and case (refused lines, views, and decision
bases/caveats/grants/reopenings), checking after every refused line that `view()` and the
record are unchanged. Plus validation edge cases (non-object line, string/boolean/NaN/Infinity
values, null payload, extra top-level field, `toString` as event name) and a check that the
sequence does not advance on refusal. `gate.ts` type-checks with `tsc --noEmit --strict`.
`run.ts`/`gate.test.ts` were not type-checked (no `@types/node` offline); they run under
Node's built-in type stripping.

## Assumptions / things I am unsure of
- **Observation ids.** `dispatch` gets only the parsed line, so I number lines by counting
  `dispatch` calls, refused ones included. Correct only if the host sends every line of the file,
  in order, blank lines excluded, to a fresh policy. If the host skips lines, ids drift.
  The accepted-event sequence (`seq`) is a separate counter and does not advance on refusal.
- Top-level fields other than `event`/`payload` are refused. A no-parameter event accepts
  `payload` absent or `{}`, but refuses `payload: null`.
- A stale (`db_stale`) or `secondhand` caveat does not block `decide`; the caveat goes with the
  decision. TASK.md requires only that the latest check, review and scan support `ready`.
- `db_aged` reopens only when the decision in force rests on the latest scan. `recheck`
  reopens only when that decision rests on the withdrawn check. A failing check or scan with
  findings reopens any decision in force.
- Refusal checks for `decide` run in a fixed order; only the message depends on it.
- Running `.ts` directly prints Node's MODULE_TYPELESS_PACKAGE_JSON warning. I left
  `package.json` as `npm init` made it, to avoid changing how the host loads `gate.ts`.

## Departures from the rules
None that I know of. No git, web, other agents or messaging tools were used.
