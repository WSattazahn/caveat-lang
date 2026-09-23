# B1 author notes

Final candidate: version 2. Two distinct submitted source versions and seven runtime checks were used. All candidate execution was through the supplied packet/runner.mjs B1 validate|replay commands. The final step is the supplied finish command. No commits.

## References read

- Assigned prompts/B1.md, packet/README.md, tasks/access-review.md, and the public packet/runner.mjs.
- packet/reference/docs/AI_AUTHORING.md and CAVEAT_ESSENCE.md.
- packet/reference/spec/caveat-reactive-0.1.md, caveat-reactive-0.2.md, caveat-reactive-0.3.md, caveat-reactive-0.4.md, and caveat-reactive-0.5.md.
- packet/reference/spec/caveat-late-qualification-0.1.md, caveat-decision-journal-0.1.md, caveat-reject-0.1.md, caveat-state-caveats-0.1.md, caveat-renewal-0.1.md, caveat-explanations-0.2.md, caveat-define-0.1.md, caveat-typed-parameters-0.1.md, caveat-observation-order-0.1.md, and caveat-save-0.1.md.
- Listed filenames under packet/reference/ to locate these references. Read my own candidate, record, and check logs. No linked examples, games, runtime implementation, private verification, other candidates, study results, or web content were read. No agents were contacted or spawned. No access-rule deviations.

## Implementation rationale

The source declares the five fixed evidence identities and retains every reached support/opposition relation. The four voting assertions inherit unverified_source. A submission observes exactly the named assertion and schedules its expired caveat after two source-clock seconds; there is no renewal. Explicit domain guards reject fractional codes, duplicates, unobserved revocations, and repeat revocations. The exact numeric input signatures enforce all remaining payload bounds and fields.

Live voting reads observed/carries predicates. Each approval builds a fresh numeric basis using qualified(0, evidence) only under active-assertion rule guards. A final guarded addition selects the numeric verdict. This keeps all active support and opposition in commitment grounds, while excluding inactive assertions and challenge. Control dependencies remain available as lineage, as documented; they are not falsely presented as approval grounds. Native access decisions have capacity three, and their reopening/commitment rules own the real runtime journal.

Only newly submitted opposition or challenge reopens an existing decision. No closed/open restriction prevents a second distinct reopening witness. Support, revocation, and expiration never reopen a decision. HUD has precisely the seven required properties. Frozen values come from the runtime series, and revision increments only after a successful native commit. Runtime transactions handle late capacity failures atomically.

## Self-checks

1. Version 1 validate failed: I attempted a state bound of 1e300, but runtime state bounds must be within +/-1e12. No candidate events ran.
2. Version 2 validate passed after changing only the elapsed state bound to 1e12.
3. mixed.jsonl: initial rejection without votes, unobserved revocation rejection, support additions without reopening, revoked-support exclusion, two distinct reopening entries while already open, approval grounds with both vote directions, revocations without reopening, challenge reopening without voting, third approval, simultaneous expiration, late revocation, repeat submissions/revocations, and direct restore.
4. clock.jsonl: submission at elapsed 0.3, advance by 2 to 2.3 retaining the vote because binary64 subtraction is below 2, zero advance retaining it, next representable increment expiring it, restoration with the timer pending, frozen history unchanged by expiration, fresh support and challenge producing a revision grounded only in that support, and later expiry/restore.
5. capacity.jsonl: three approvals with distinct reopening causes, challenge reopening at capacity, three fourth-approval attempts rejected, subsequent support/revocation inputs still functioning, elapsed progression, expiration, and repeated restoration.
6. admission.jsonl: unknown event, extra/missing fields, out-of-range codes and times, fractional codes, numeric strings, booleans/null, a nonfinite numeric input attempt, duplicate submission/revocation, timer progression after rejected advances, and restore. The runner JSON parse/stringify path converts the 1e309 attempt to null before dispatch, so this is not a direct nonfinite-native-payload test.
7. staggered.jsonl: challenge before any decision, opposition-only denial, new support changing live verdict without reopening, staggered assertion expiry, later opposition reopening, approval grounds excluding expired opposition and challenge, observation-order journal output differing from lexical grounds, late revocation, and restore.

The five replays contain 54 accepted events, 38 rejected events, and 11 successful save/restore points. After reading their output, I also inspected their saved snapshots with generic log comparisons: every rejected snapshot was identical to its predecessor; every HUD had exactly the required property names; HUD elapsed equaled runtime elapsed; all existing journal entries were unchanged prefixes; and all previously stored commitment grounds remained unchanged. This inspection read logs only and did not execute candidate policy outside the runner.

## Uncertainties and limits

No hidden-test feedback was received, and the five authored histories are not exhaustive. The observed floating-point boundary establishes the required subtraction behavior for that adversarial case, not a proof over every binary64 input.

The elapsed display state is limited to 1e12 by the language's state-bound restriction. A session whose next advance would exceed that total would reject, even though the task does not explicitly impose a total-clock bound. Reaching it from zero with dt <= 2 would require at least 500 billion accepted advances. This is a known formal limitation of this candidate, not claimed as task-compliant behavior at unlimited duration.

Within the exercised duration and histories, final version 2 matched the public contract and my explicit expectations. Preserved runner logs contain the full evidence, timers, sequence, grounds, and journal snapshots for independent review.
