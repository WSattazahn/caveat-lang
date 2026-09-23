# B2 independent author notes

## Submission and budget

Final candidate is source version 2, unchanged after its successful validation and six replays. Two distinct source versions and eight validation/replay commands were used, within the limits of three and twelve. No commits were made. All program execution used only `node experiments/agent-authoring/v2/packet/runner.mjs B2 validate|replay|finish`.

Version 1 failed immediately because I used `--` line comments, which Caveat does not recognize. After reading the packet source-text reference, I removed those comments. Version 2 has no policy change from version 1; it validates and is the final submission.

## References read

- Assigned prompt: `experiments/agent-authoring/v2/prompts/B2.md`.
- Assigned task: `experiments/agent-authoring/v2/tasks/access-review.md`.
- `experiments/agent-authoring/v2/packet/README.md` and its public `runner.mjs`.
- Under `experiments/agent-authoring/v2/packet/reference/`:
  - `docs/AI_AUTHORING.md`
  - `docs/CAVEAT_ESSENCE.md`
  - `spec/caveat-reactive-0.1.md`
  - `spec/caveat-reactive-0.2.md`
  - `spec/caveat-reactive-0.3.md`
  - `spec/caveat-reactive-0.4.md`
  - `spec/caveat-reactive-0.5.md`
  - `spec/caveat-reactive-0.7.md`
  - `spec/caveat-late-qualification-0.1.md`
  - `spec/caveat-renewal-0.1.md`
  - `spec/caveat-reject-0.1.md`
  - `spec/caveat-explanations-0.1.md`
  - `spec/caveat-explanations-0.2.md`
  - `spec/caveat-decision-journal-0.1.md`
  - `spec/caveat-define-0.1.md`
  - `spec/caveat-elapsed-0.1.md`
  - `spec/caveat-observation-order-0.1.md`
  - `spec/caveat-save-0.1.md`
  - `spec/caveat-text-0.1.md`

I listed filenames under packet/reference/ to find these references, but did not follow links outside the packet. The remaining inspected files were my own candidate, event histories, runner record, and runner snapshots under runs/B2/.

## Self-checks

1. Check 01: version 1 validation rejected unsupported comment syntax.
2. Check 02: version 2 validation succeeded; all seven initial hud properties were present with the required values.
3. Check 03, `lifecycle.jsonl`: both support assertions, contradiction from the same source, repeated distinct reopen reasons while already open, tie denial, revocation, staggered expiry, three revisions, and restore. Frozen grounds and journal entries remained unchanged after later revocation/expiry.
4. Check 04, `subtraction-boundary.jsonl`: observed alpha_allow at elapsed 0.3, then advanced by 2. At elapsed 2.3 the assertion correctly stayed active because binary64 subtraction is below 2. A zero advance preserved it and its pending qualification; after restore, the next representable increment expired it. The snapshot showed the pending timer removed, the expired qualification applied, and support zero before final publication. Revoking the expired assertion remained legal once.
5. Check 05, `capacity-order.jsonl`: observed beta_allow before alpha_allow, and beta_deny before alpha_deny. Grounds evidence arrays were lexical, whereas journal because arrays followed that observation order. Three approvals succeeded; fourth and repeated approvals rejected atomically while the decision remained review. All active opposition was included in an allow approval's grounds.
6. Check 06, `admission.jsonl`: missing/extra parameters, null/array/scalar payloads, nonnumeric values, out-of-range values, fractional codes, duplicate submission, unobserved/repeated revocation, unknown events, invalid clock inputs, and no-vote approvals all rejected. Valid continuation after rejection and restore retained the original expiry schedule. The fresh approval excluded expired/revoked alpha_allow.
7. Check 07, `fresh-revisions.jsonl`: challenge reopened an old approval even with zero votes; approval still rejected until new support arrived. Later approvals' grounds contained only beta_allow, excluding expired alpha_allow, challenge, and revoked alpha_deny. Journal elapsed and frozen value fields reflected the actual approval/reopening events.
8. Check 08, `challenge-first.jsonl`: challenge before any approval neither voted nor expired; opposition before any approval created no reopening entry. An opposition-only first approval froze deny. Revoking both opposition assertions and adding support changed live verdict to allow while the frozen decision remained deny and closed. All five submission identities remained one-use.

Across the six replays: 61 accepted events, 50 rejected events, and 13 successful save/restore records. I inspected the full runner snapshots and compared each rejected snapshot with its immediately preceding snapshot; all 50 were identical, including state, sequence, effects, pending timers, relations, and decisions. The runner itself asserts full snapshot equality on each resume. I also inspected final observed relation order and direct qualifies edges: contradictory supports/opposes persisted, only the four voting assertions had declared unverified_source, and challenge gained no caveat.

## Why the source meets the contract

The source declares exact numeric event signatures, rejects fractional codes and domain violations, and uses fixed real evidence revealed once. Each voting assertion schedules its own expired caveat after 2 on the advance clock; revoke adds only revoked. Runtime scheduled qualification supplies subtraction-based aging, including the tested non-algebraic boundary. There is no bounded shadow clock or observedAt accumulator.

A procedure resets the four live vote values and conditionally fills each with qualified(1, its_assertion). Selection is in a rule guard, so an inactive assertion's control lineage cannot become vote grounds. Support and opposition sum those votes. The approval expression compares both totals, thereby grounding its numeric result in every currently active voting assertion, including opposing votes. The decision series has capacity 3. Real commit/reopen effects produce commitment grounds and decision_journal; no source or host replacement history exists. Only new opposition or challenge has a reopening rule, and those rules do not exclude an already-open decision.

The hud binds exactly support, opposition, verdict, decision, frozen, revision, and elapsed. Frozen value and openness are read from the actual runtime decision series, and revision increments only after a successful commit in the same transaction.

## Uncertainties and limits of self-checking

No known functional mismatch remained in the tested histories. These are author self-checks, not private-verifier results or exhaustive verification of all possible histories. The public runner parses/stringifies JSON; raw duplicate keys and raw nonfinite transport spellings were therefore not tested. Admission of numeric inputs otherwise relies on the documented runtime event contract. Conservative lineage can include historical control dependencies; the requested commitment grounds and journal citations were inspected separately and contain only the required content.

## Access-rule deviations

None. I did not inspect runtime implementation, private verification, other candidates, prior study files/results, repository examples/games, external web content, or parent-task history. I did not contact or spawn agents. Only my run directory was written, apart from the provided runner's own writes into that same directory. Runner snapshots, preserved versions, and records were not edited manually.
