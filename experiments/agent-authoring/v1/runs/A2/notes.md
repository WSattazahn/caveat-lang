# A2 author notes

## Scope and references

Assigned task: `tasks/cold-storage.md`. Read the frozen `prompts/A2.md`, packet README, and the public `packet/runner.mjs`.

Read these packet/reference files:
- docs/AI_AUTHORING.md
- docs/CAVEAT_ESSENCE.md
- spec/caveat-reactive-0.1.md through spec/caveat-reactive-0.5.md
- spec/caveat-renewal-0.1.md
- spec/caveat-late-qualification-0.1.md
- spec/caveat-state-caveats-0.1.md
- spec/caveat-explanations-0.2.md
- spec/caveat-decision-journal-0.1.md
- spec/caveat-reject-0.1.md

Enumerated filenames under packet/reference and searched (without matching output) for clock/elapsed access in spec/caveat-define-0.1.md, spec/caveat-reactive-0.6.md, spec/caveat-reactive-0.7.md, spec/caveat-view-0.1.md, and runtime/prelude.cav. Did not follow references outside the packet.

## Implementation

All policy is in candidate.cav. Renewable sensor_reading evidence has four occurrences and a declared calibration caveat. Each accepted reading reveals exactly one directional claim edge and schedules stale after two units using the source clock. A separate plan_basis state retains the concrete decision evidence, allowing late qualification of an old occurrence to reopen the current decision even when the latest reading is fresh. The three-capacity real dispatch decision series supplies frozen values, commitment grounds, and journal records. Reopening uses caveated(plan_basis, stale) and is guarded against an already open decision. Bindings expose exactly the seven required hud properties.

## Submissions and checks

Two distinct source versions, five runner replay commands:
1. Version 1 failed validation: I attempted an explicit elapsed state bound above the runtime maximum. The runtime reported that bounds must lie within +/-1e12. No events ran.
2. Version 2 lifecycle.jsonl: missing-reading decision rejection; zero time; inclusive threshold; closed-decision rejection; newer contradictory reading without automatic revision; old-reading expiry while latest remains fresh; one reopening per closed revision; three revisions; fourth-revision and fifth-reading rejections; save/restore before and after expiry.
3. Version 2 fractional.jsonl: temperatures immediately above and below four; observation at 0.3; advancing by two yields 2.3 and leaves the reading fresh because binary64 2.3 - 0.3 is below two; zero increment preserves it; a subsequent 5e-16 increment makes it stale and reopens using the old occurrence while the newly renewed reading stays fresh. Further revision and restore checked.
4. Version 2 admission.jsonl: unknown event; missing/extra payload fields; string/null/boolean numeric inputs; out-of-range temperatures and dt; inclusive -10/20; failed advance does not age pending evidence; non-basis evidence aging does not reopen the decision; actual basis expiry does; stale decision rejected.
5. Version 2 repeated.jsonl: four equal readings are distinct; fifth rejected; simultaneous scheduled aging; immediately below two remains fresh; reaching two qualifies all four occurrences but reopens only because of the actual first-reading decision basis; zero increment creates no duplicate reopening; stale decision rejected; restore checked.

Inspected 74 complete snapshots from checks 02 through 05. Compared every rejected or resumed snapshot with the preceding complete snapshot: all 25 rejections were unchanged and all eight restores were identical. Compared every existing commitment's stored grounds and basis across later snapshots: all remained frozen. All inspected hud objects had exactly the seven required properties. These comparisons only read runner logs; candidate execution occurred solely through the supplied runner.

## Uncertainties and limitations

The first submission exposed the runtime's numeric-state bound. Version 2 mirrors elapsed in a state bounded to 0..1e12. An extraordinarily long accepted history that would take elapsed above 1e12 would therefore be rejected, although the task does not explicitly set a total-time horizon. I did not attempt an alternate large-time representation with different rounding behavior. This is a known limitation of the final candidate.

The runner's JSON serialization limits direct tests of nonfinite numbers; malformed/nonnumeric payloads were tested, and numeric admission relies on declared event bounds. No exhaustive binary64 search or private verification was performed. The tested fractional boundary distinguishes subtraction-based aging from a rearranged deadline comparison.

No access-rule deviations. No runtime, reference, task, private-verifier, other-author, existing-game, or web access. No agent contact, no subagents, no host gameplay implementation, and no commits. Preserved versions and runner records/logs were not edited.