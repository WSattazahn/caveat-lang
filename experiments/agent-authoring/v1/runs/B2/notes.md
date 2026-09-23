# B2 author notes

Final candidate: version 2. Two distinct submitted source versions and six validation/replay commands were used. No commits were made.

## References read

- Frozen prompt: `experiments/agent-authoring/v1/prompts/B2.md`.
- Public task: `tasks/access-review.md`.
- Public packet: `packet/README.md`, `packet/runner.mjs`.
- `packet/reference/docs/AI_AUTHORING.md`.
- Under `packet/reference/spec/`: `caveat-0.1.md`, reactive profiles 0.1, 0.2, 0.3, 0.4, 0.5 and 0.7; `caveat-decision-journal-0.1.md`, `caveat-explanations-0.2.md`, `caveat-late-qualification-0.1.md`, `caveat-typed-parameters-0.1.md`, `caveat-reject-0.1.md`, `caveat-renewal-0.1.md`, `caveat-state-caveats-0.1.md`, `caveat-define-0.1.md`, `caveat-observation-order-0.1.md`, and `caveat-save-0.1.md`.
- File names and clock-related text were searched only under `packet/reference/`. This also returned snippets from `docs/CAVEAT_ESSENCE.md` and other packet references. Some batched initial reference output was truncated; relevant profiles were reread separately.
- Own candidate, own record and own runtime output logs were inspected.

## Implementation

All policy is in `candidate.cav`. Fixed evidence declarations receive real observations, scheduled expiry caveats, and explicit revocation caveats. Submission and revocation rules reject invalid domains and duplicate operations. No evidence is renewed or replaced. The generic runtime owns expiry scheduling, actual commitments, their grounds, the decision journal and save/restore.

The tally procedure resets both counters before adding one qualified vote for each currently active assertion. Each addition has evidence selection in its rule guard, so inactive assertions affect control lineage but never the new grounds. The approval expression reads both tallies, grounding each revision on all active support and opposition. The runtime separates these grounds from predecessor and reopening control lineage. Only newly submitted opposition and challenge can reopen, and their exact evidence identities are used even when the current decision is already open.

The first source failed validation because state bounds cannot exceed 1e12. Version 2 represents the display clock in high and low parts with a power-of-two radix of 1048576. It reconstructs the previous binary64 clock, adds dt once per evaluated expression, and splits the result without rounding to decimal time. Both state parts remain within the runtime's bounds throughout the clock's attainable binary64 range for dt in 0..2. The runtime clock independently supplies scheduled qualifications and journal elapsed values.

## Checks and observations

1. `validate`, version 1: failed at load because its elapsed state maximum exceeded the runtime's +/-1e12 state-bound limit. No runtime policy result was obtained from that version.
2. `replay main.jsonl`, version 2: 11 accepted events, 3 expected rejections, 2 resumes. Covered support-only approval, support without reopening, new opposition and challenge producing separate entries while already open, three revisions, revocation, expiry and preserved frozen decisions. Confirmed lexical grounds and first-observation journal order when beta was submitted before alpha.
3. `replay time.jsonl`: 12 accepted events, 2 resumes. Checked the binary64 subtraction boundary: an assertion observed at 0.3 remains active at 2.3 because 2.3 - 0.3 is below two, including an advance of zero. A tiny later advance expires it. Subsequent approval excludes expired evidence. Inspected scheduled_at and after fields in the real timer snapshot.
4. `replay admission-capacity.jsonl`: 11 accepted events, 23 expected rejections, 3 resumes. Covered unknown events; missing/extra payload fields; null/string values; fractional and out-of-range codes; negative/out-of-range dt; revocation before observation; duplicate submissions/revocations; three approvals followed by a fourth approval attempt while reopened. Every rejected event had a complete snapshot identical to its predecessor, including pending timers, sequence, decisions and grounds. The JSON 1e309 case was normalized by the public runner to null before runtime dispatch, so this checks rejection through the runner rather than a direct nonfinite-number runtime call.
5. `replay deny-first.jsonl`: 14 accepted events, 3 expected rejections, 2 resumes. Covered challenge alone, deny-only approval, later supports changing the live verdict without reopening, revocations, a later approval grounded only on beta's active allow and deny, expired assertions being revoked, and permanent single-use submission identities.
6. `replay staggered.jsonl`: 14 accepted events, 2 resumes. Covered staggered fractional observation times, selective expiry, immutable earlier grounds and values, reopening from later opposition/challenge, and explicit revisions excluding inactive evidence. Journal elapsed values and frozen reopening values were inspected.

Total successful-version replay results: 62 accepted events, 29 expected rejections, 11 successful save/restore operations. The public runner checks complete snapshot equality on every restore. Read-only inspection of all five logs found zero changed snapshots after rejection, zero mismatches between hud.elapsed and runtime elapsed, and exactly the seven required hud properties in every snapshot.

## Uncertainty and limitations

No private verifier feedback was requested or received, and these self-checks are not an exhaustive proof. The large elapsed-clock range and crossing the 1048576 split boundary were reasoned about but were not exercised by a long replay. Direct nonfinite payload calls cannot be faithfully represented by this public JSONL runner; the source declares numeric bounds and relies on the runtime's documented finite-number admission checks. No known remaining task-policy defect was found in the exercised histories.

## Access rules

No access-rule deviations. No runtime implementation, private verifier/tests/oracle, other candidates, study results, game/example source, web resources or linked out-of-packet examples were read. No other agents were contacted or spawned. Only the public runner executed or validated candidate source. No host policy implementation was supplied. Files were authored only under this run directory; preserved versions, logs and records were left to the runner.
