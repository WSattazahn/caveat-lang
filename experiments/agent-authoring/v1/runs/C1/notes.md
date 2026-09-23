# C1 author notes

## Submission

Task: Repair queue. Candidate: candidate.cav. One source version, six runtime checks (one validation and five replays). No source repairs were needed. All candidate execution used the provided packet/runner.mjs with run ID C1. No commits.

## References read

- Assigned instructions: prompts/C1.md.
- packet/README.md and public packet/runner.mjs.
- tasks/repair-queue.md.
- packet/reference/docs/AI_AUTHORING.md.
- packet/reference/docs/CAVEAT_ESSENCE.md.
- packet/reference/spec/caveat-reactive-0.1.md.
- packet/reference/spec/caveat-reactive-0.2.md (tool output partially truncated).
- packet/reference/spec/caveat-reactive-0.3.md.
- packet/reference/spec/caveat-reactive-0.4.md (tool output partially truncated).
- packet/reference/spec/caveat-reactive-0.5.md (tool output partially truncated).
- packet/reference/spec/caveat-renewal-0.1.md.
- packet/reference/spec/caveat-typed-parameters-0.1.md.
- packet/reference/spec/caveat-reject-0.1.md.
- packet/reference/spec/caveat-decision-journal-0.1.md.
- packet/reference/spec/caveat-explanations-0.2.md.

I listed the reference directory and read only the permitted materials and my own run outputs. I followed no guide links to external examples, games, tests, runtime implementation, or the web. I neither contacted nor spawned another agent. Access-rule deviations: none.

## Implementation rationale

The program declares exactly the four numeric event contracts. Bounds and exact payload membership are handled by those declarations; source reject rules enforce integer values, phases, tokens, latest positive diagnoses, and the single permitted investigation transition.

Each inspection renews an already observed occurrence of its selected cause before revealing it. The first observation uses the base name. Only the selected score changes; tokens decrement once. Each diagnostic renewable has capacity three and inherits the sole declared evidence caveat alternative_cause. The two causes never oppose each other.

Choice commits a numeric repair code directly qualified by the selected cause's current occurrence. Eligibility checks use reject rules; the selected diagnostic score is not used as the frozen numeric value. The commitment retains repair_unverified without qualifying the diagnostic evidence. Commitment grounds therefore contain only the latest selected observation and the two specified caveats. Full runtime lineage can still include older observations and the previous reopened decision; that is distinct from commitment_grounds.

Failure renews failure if necessary, observes opposition to repair_effective, and reopens the current real decision. Success only changes the phase. The next event resets current scores and selected repair without resetting tokens, renewals, graph, or decision history. The frozen HUD value reads the actual latest decision. No clock is declared.

## Runtime self-checks

1. validate: loaded successfully with the exact eight initial HUD properties.
2. two-failures.jsonl: bearing repair fails, next clears diagnoses, motor zero score blocks choice, renewed positive motor supports revision two, failure@2 reopens it, and no third investigation is admitted. Includes rejected wrong-phase operations, exhausted-token inspection, and five restores.
3. repeated-bearing-success.jsonl: three identical readings create bearing_check, bearing_check@2, and bearing_check@3; a fourth inspection rejects; grounds use only @3; success creates no failure evidence; all subsequent events reject. Includes two restores.
4. latest-zero-other-preserved.jsonl: motor diagnosis remains while bearing is inspected twice; latest bearing zero blocks that repair; motor repair freezes code 2 despite score 1 and excludes both bearing observations from grounds; next preserves zero remaining tokens and clears both diagnoses. Includes two restores.
5. same-cause-next-success.jsonl: bearing occurrence numbering continues across next; the second decision uses only bearing_check@3 while repair@1 retains its original grounds; second success is terminal. Includes three restores.
6. admission-rollback.jsonl: unknown events, undeclared tick, wrong phases, missing/extra parameters, strings, boolean/null/array/object inputs, out-of-range values, and fractional cause/score/repair/success reject; valid operations afterward retain correct occurrence numbering. Includes three restores.

Across five replays: 34 accepted events, 57 rejected events, 15 successful save/restore checkpoints. I inspected acceptance/rejection, HUD transitions, commitment grounds, journal entries, frozen codes, and relevant graph relations. A post-hoc read of the complete stored public snapshots found every rejected event identical to its preceding snapshot. Every recorded elapsed value was zero. Final snapshots have exactly the required eight HUD properties, observation relations in acquisition order, only alternative_cause qualification edges on diagnosis occurrences, no qualification edges on failures, and the declared renewal capacities.

## Uncertainty and limits

No private verifier feedback was requested or received. These are author-created scenario checks, not an exhaustive proof over all histories. The public runner parses and reserializes JSON payloads: the attempted 1e999 input becomes null before dispatch and rejects, so it does not independently test the runtime's direct nonfinite-number path. Duplicate JSON keys likewise cannot be tested faithfully through this runner. I rely on the supplied documented exact numeric-event contract for those runtime-level admission guarantees. No known candidate policy defect remains from the inspected checks.
