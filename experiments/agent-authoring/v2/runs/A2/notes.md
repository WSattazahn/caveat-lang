# A2 author notes

## Submission

Implemented cold-storage dispatch entirely in candidate.cav. One source version was submitted; five public-runner replay commands were used. Every replay also validated construction. No commits were made. The submitted version is the original candidate, without repairs.

## References consulted

- experiments/agent-authoring/v2/prompts/A2.md (the assigned instructions).
- experiments/agent-authoring/v2/packet/README.md.
- experiments/agent-authoring/v2/tasks/cold-storage.md.
- experiments/agent-authoring/v2/packet/runner.mjs.
- Under packet/reference/docs/: AI_AUTHORING.md.
- Under packet/reference/spec/: caveat-reactive-0.1.md, caveat-reactive-0.2.md, caveat-reactive-0.3.md, caveat-reactive-0.4.md, caveat-reactive-0.5.md, caveat-renewal-0.1.md, caveat-late-qualification-0.1.md, caveat-state-caveats-0.1.md, caveat-decision-journal-0.1.md, caveat-reject-0.1.md, caveat-elapsed-0.1.md, caveat-explanations-0.2.md, caveat-save-0.1.md.
- Searched relevant excerpts within the allowed reference files caveat-procedure-symbols-0.1.md, caveat-reactive-0.6.md, caveat-typed-parameters-0.1.md and caveat-0.1.md. Listed filenames under packet/reference/. Did not follow links outside the packet.
- Own candidate, inputs, record and check logs only after authoring.

## Self-checks

All execution was through `node experiments/agent-authoring/v2/packet/runner.mjs A2 replay INPUT`.

1. float-boundary.jsonl: observed at 0.3, advanced by 2 to 2.3. The evidence remained fresh because binary64 subtraction is below 2. A zero increment did not age it; a 5e-16 increment caused qualification and reopening. Save/resume preserved the pending timer and frozen grounds.
2. lifecycle.jsonl: empty-state decision rejection, threshold temperature 4, conflicting fractional reading, fresh replacement without reopening, old decision evidence becoming stale while a newer reading remains fresh, three revisions, reading and decision capacity rejection, both temperature endpoints, stale rejection, no duplicate reopening, and several saves/restores.
3. admission.jsonl: unknown event, missing and extra parameters, null/array/scalar payloads, nonnumeric values, out-of-bound temperatures/time increments, malformed events while a timer is pending, fractional temperature just above 4, stale decision rejection, and renewal after staleness. All intended invalid events rejected.
4. repeated-readings.jsonl: equal readings produced four distinct evidence occurrences; a fifth rejected. Simultaneous expiry qualified all four occurrences, while the reopening cited only the second occurrence actually used by the decision. Zero time did not duplicate the journal. Save/resume succeeded before and after expiry.
5. unchosen-reading.jsonl: time without readings, initial save/resume, a previously unused reading ageing without reopening the current decision, fractional temperatures and time increments, reopening on the selected old reading despite a newer fresh reading, and three revisions using occurrences 2, 3 and 4.

Public runner logs contain ten successful save/resume snapshot-equality checks. I also read the five logs and compared each rejected event's complete snapshot with the immediately preceding snapshot using PowerShell JSON serialization: all 32 rejection snapshots were unchanged. This comparison only inspected recorded output; it did not execute or implement policy.

Manually inspected HUD values, graph observation order, direct calibration/stale qualification edges, commitment grounds, frozen values, and journal witness/event/sequence/time fields. The public runner's completed status alone does not assert policy correctness.

## Why this source meets the contract

The three exact numeric event declarations enforce payload admission. The source clock is advance and the HUD reads elapsed() directly, avoiding a separate bounded time accumulator. Renewable sensor_reading has capacity four, and every accepted read reveals exactly one support/opposition and schedules its occurrence's stale caveat after two seconds. The public floating-point edge check demonstrates subtraction-style timing in the supplied runtime.

The current temperature is qualified by only the latest occurrence. A successful decide copies its release/block assessment into a separate live decision_basis, commits the real dispatch series with that basis and increments the displayed revision count. The runtime freezes each commitment's grounds while late caveats update the separate live state. has_caveat and caveated select the exact retained occurrence on advance, so renewal cannot retarget reopening. A closed/open guard prevents duplicate reopening. Revision capacity is enforced transactionally by the actual decision series. The seven required HUD bindings are always present and are the only HUD properties.

## Uncertainties and limitations

No known policy discrepancy was observed in these self-checks. This is bounded author testing, not exhaustive proof or private-verifier feedback. The runner parses and stringifies JSON, so raw nonfinite/duplicate-key transport spellings were not tested through another execution path. Numeric finite admission relies on the documented/runtime event contract. Very long time histories were not enumerated; the source has no custom timestamp accumulator or duration cap.

## Access-rule deviations

None. No runtime implementation, private verification, other candidates, v1 material, repository examples/games, parent history, web or outside guide links were accessed. No other agents were contacted or spawned. Only the assigned prompt, public packet/task/reference content, and runs/A2 files were accessed. No preserved sources, runner records/logs, packet files or task files were edited.
