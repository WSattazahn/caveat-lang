# Round 8 predictions, round level

Recorded by Claude for the owner on 2026-10-10, before the protocol is
registered, before any change request for the round is written and before
any author starts. The predictor has read round 7's protocol, results and
programs, and `docs/design/beat-typescript.md`; no round-8 implementation
exists. Per-request predictions for CR17–CR20 follow when those requests are
registered (protocol, row 8).

Every prediction is a number on a measure the protocol already defines,
read as the primary median per side. A prediction that fails is reported as
failed, beside the outcome, as round 7's were (RESULTS.md, round 7
"Predictions").

Two columns, kept apart on purpose. **Target** is the number
beat-typescript.md registered as what round 8 should show ("Round 8",
"Predictions pre-registered"); it is copied unchanged. **Expected** is the
predictor's own number today, with the reason. Where they differ, the record
scores both.

| Measure | Target | Expected, Caveat | Expected, TypeScript | Reason for the expectation |
| --- | ---: | ---: | ---: | --- |
| 1. Blind phases green on first run, of 4 | 4 | 3 | 4 | The save-bound and caveat-order causes are gone (departure; protocol row 5) and C006 reports round 7's `ungrounded_citation` cause before the run. New requests will still find something the packet does not teach; round 7's best Caveat author reached 3 |
| Blind runs to all-green | — | 5 | 4 | Follows the line above |
| 3. Regressions during the blind phases | 0 | 0 | 0 | Round 7's Caveat regressions came from the save bound and the journal glue (RESULTS.md failure table); both causes are removed |
| 3. Change cost, code lines, CR17–CR20 | lower than TypeScript | 150 | 220 | Round 7's Caveat diff was a third smaller with hand-written glue; new requests of similar size |
| 2. Size, code lines at the end | lower than TypeScript | 380 | 560 | Four more requests on both sides; the glue shrinks with the starter and ordered grounds, the policy grows |
| 4. Explanation failures | 0 | 0 | 0 | Round 7 was 0 against 0 |
| 6. Mutation score, overall | 100% | 98% | 100% | Round 7's only Caveat survivors were the M4 mutants that swap the order two observations enter the recovery commitment in the source. View 0.2 orders grounds by first observation, which the events decide, not the source, so such a mutant should still change nothing and stay "possibly equivalent". The scenario comparator also compares `decision.basis` as a set (`harness.mjs` L64). This expectation disagrees with beat-typescript.md's reasoning for its target; the round settles it. Both sides stay within the 10 percent margin |
| 7. Explanation drift, units, CR17–CR20 | at or below TypeScript | 35 | 30 | Windows remove the adapters' save code and ordered grounds remove the order code, but each `because` is still a unit (RESULTS.md, drift); this is the measure most likely to keep Rule B unmet |
| 5. Dispatch + view, median µs | under 50 | 100 | 20 | View 0.2 adds a delta view (`docs/design/view-0.2.md` L53), but the harness asks the adapter for the whole beat view after every event, so an author gains only by keeping that view up to date from deltas. Round 7 measured 148 against 17.9 |
| Gzipped bytes a host ships | under 150,000 | 760,000 | 7,000 | Measured on rc.16 today: the package already ships the reactive-only build (`runtime/build-info.json`, `pkg-reactive` hashes) and it is 751,039 bytes gzipped for the WASM plus 3,551 for its loader, larger than rc.11's 615 KB. Nothing planned for rc.17 shrinks it |
| Rule A, adoption | met | met | — | Size and change cost better with no regression, correctness within 2× |
| Rule B, the stated claim | — | not met | — | Its mutation clause needs Caveat higher (above, 98% against 100%), drift lower, and first-run correctness no worse; each is expected to miss |

The shipped-bytes target was met by no published build and is not a gate in
either rule (beat-typescript.md, the table's last row). The expectation is
on the record so that a miss there is not read as a surprise.

## The learnability probe

The probe has no target in beat-typescript.md. Expected, before its reply is
read: its program fails `caveat check` on the first run; of the constructs
it uses, fewer than half are **same** under the protocol's classes; and it
puts the explanation bookkeeping (`because` lists, caveat unions) in the
adapter rather than the source, which is the cause of most round 7 glue.

## Per request, CR17–CR20

Added at 2026-10-10 22:15 UTC, after the writer's revision and before any
author starts (protocol row 8). The predictor has read the requests, the
review and the scenarios, and no implementation, because none exists. "Diff"
is blind-phase change cost in code lines; "failed runs" counts runs that fail
during the request. Primary medians.

| CR | Predicted | Why |
| --- | --- | --- |
| CR17: the HUD shows countdowns | TypeScript smaller diff and fewer failed runs | three derived numbers from timers each side already keeps; TypeScript reads its fields, while Caveat needs the remaining time of a schedule exposed to the view and rounded up, likely partly in the adapter |
| CR18: doubt what you saw | Caveat smaller diff by more than 10%; Caveat at least one failed run | a caveat that every citation shows is late qualification, which Caveat carries to every citing view by itself; TypeScript must add `doubted` to each caveat union by hand. Excluding a doubted witness from the belief while keeping its memory slot and CR7's count is new bookkeeping on both sides |
| CR19: taste again | tie | a second taste per life is a renewable observation on the Caveat side and a copy of the taste path on the TypeScript side; the "replaces the first taste as what identifies the mushroom" rule touches the label table on both |
| CR20: take back a doubt | TypeScript smaller diff and fewer failed runs | removing a caveat everywhere it is cited is a flag flip in TypeScript; in Caveat a qualification is part of the evidence's record, so taking it back needs either a withdrawable qualification the packet documents or per-witness state the views consult |
| Drift | Caveat lower on CR18, TypeScript lower on CR17 and CR20, tie on CR19 | follows the diffs |
