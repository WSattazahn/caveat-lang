# Samples 001 and 002 re-run on the published rc.19

Re-run on 2026-10-12, during rc.19's soak, under the owner's "Field test"
card (15:28 UTC). The samples' own files are unchanged: their `explain.txt`
files remain the rc.15 records they were written against. This directory pins
the published `caveat-lang@0.1.0-rc.19` (doctor: wasm sha256 `3c894023…`) and
holds its outputs beside them.

```sh
cd experiments/agent-decisions/retest-rc19 && npm ci
npx --no-install caveat-lang test ../001-departure-reason-mechanism/decision.scenarios.json ../002-departure-continuation/decision.scenarios.json
```

## Results

- **Scenarios:** 001 5/5 and 002 4/4 pass on rc.19 (`test.txt`). Nothing a
  sample accepted or refused on rc.15 changed.
- **Explanations:** the only differences from the rc.15 records are the
  rendering changes below and a new `made under source sha256:…` line
  (`001-explain.txt`, `002-explain.txt`).

| Finding | rc.15 | rc.19 | Receipt |
| --- | --- | --- | --- |
| D001 | an undeclared `commit` shows "none declared" | shown as `close_170: plain commitment` with grounds, grant and retained caveat; `dependents` lists it under Decisions | `d001.txt` |
| D002 | `dependents` on a reason omits its withdrawal | `Withdrawals resting on this reason: mechanism_reopen withdrawn at #3 … because thread_correction` | `001-dependents.txt` |
| D003 | "based on withdrawal_L93 (caveats: ci_pending)", "based on evidence with ci_pending" | "based on withdrawal_L93" then "retaining: ci_pending"; dependents "retaining: ci_pending" | `001-explain.txt`, `001-dependents.txt` |
| D004 | practice gap in skill step 2 | step 2 now says runtime lineage is what the rules read and that a withdrawal guard is used only when the correction gated the decision (`skills/caveatist/SKILL.md` L21, identical in all three copies, sha256 `4b562d8e…`) | the skill text |
| D005 | `dependents` reports no claims | a `Claims` section: `mechanism_reopen supports reopen_strands` | `001-dependents.txt` |

All five are fixed in the published package. The skill is not part of the npm
package, so D004's receipt is the repository text, not rc.19.
