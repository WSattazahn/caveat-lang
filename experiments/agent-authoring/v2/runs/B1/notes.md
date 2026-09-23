# B1 author notes

## Scope and access

Implemented the assigned two-source access review task entirely in candidate.cav. No imports, host policy implementation, runtime edits, dependency changes, commits, agent communication, web access, or private verification were used. All writes were confined to experiments/agent-authoring/v2/runs/B1/. No access-rule deviation is known.

The only executable candidate path was the supplied packet/runner.mjs. One distinct source version was submitted, with five runtime checks: one validate and four replays. Preserved source versions and runner records/logs were not edited.

## References read

- experiments/agent-authoring/v2/prompts/B1.md (assigned entry instructions)
- experiments/agent-authoring/v2/tasks/access-review.md
- experiments/agent-authoring/v2/packet/README.md
- experiments/agent-authoring/v2/packet/runner.mjs
- Under packet/reference/docs/: AI_AUTHORING.md, CAVEAT_ESSENCE.md
- Under packet/reference/spec/: caveat-reactive-0.1.md, caveat-reactive-0.2.md, caveat-reactive-0.3.md, caveat-reactive-0.4.md, caveat-reactive-0.5.md, caveat-reactive-0.7.md, caveat-explanations-0.1.md, caveat-explanations-0.2.md, caveat-late-qualification-0.1.md, caveat-reject-0.1.md, caveat-decision-journal-0.1.md, caveat-elapsed-0.1.md, caveat-renewal-0.1.md, caveat-define-0.1.md, caveat-observation-order-0.1.md, caveat-save-0.1.md

The allowed reference directory was listed to find these documents. Links outside the packet were not followed. Some first combined documentation output was truncated; the qualified-computation specification was subsequently read in full.

## Design and contract reasoning

Declared numeric event signatures provide exact payload admission and numeric bounds. Explicit reject rules enforce integer codes, one-time submissions, observed/unrevoked revocation targets, active-vote approval, and an open or absent current decision. The runtime decision series has capacity three, and its own failure is transactional.

Each assertion is revealed exactly once, retaining its fixed identity and all contradictory relations. Assertions 1 through 4 have only the declared unverified_source caveat initially, then source-scheduled expired and explicit revoked qualifications. Scheduled qualification after 2 uses the source clock, captures the runtime observation-time value, and applies before advance rules. No bounded duplicate clock or rearranged deadline is authored. Challenge has no expiry, qualification or voting contribution.

refresh_votes resets each numeric vote unconditionally to zero, then assigns qualified(1, assertion) under that assertion's active guard. This uses the documented distinction between control lineage and content grounds: inactive assertions cannot persist in vote grounds. The approval value compares both sums, so its grounds include every active voter, including opposition, and only unverified_source. Prior decision and reopening dependencies can remain in lineage without contaminating the new commitment grounds. Runtime commitments and decision journals, rather than source-authored text history, provide the frozen decisions and chronology.

Only newly submitted opposition and challenge invoke reopen, without a not-reopened guard, so each distinct new witness records its own entry. Support, expiry and revocation do not commit or reopen. The seven hud properties use the live vote states, actual runtime decision queries/latest value, successful approval count and elapsed().

## Self-checks

1. validate succeeded with the exact seven initial HUD fields.
2. basic.jsonl: empty approval and absent revocation reject; nonlexical observation order is retained in journal because; support does not reopen; opposition and challenge add separate reopening entries even while already open; active-only grounds survive explicit revision; revocation changes a live verdict while the frozen approval stays unchanged; all assertions expire; expired evidence may be revoked; duplicate submissions reject; save/restore succeeds.
3. float-expiry.jsonl: observed at 0.3, an assertion remains active at binary64 elapsed 2.3 after dt 2, because the subtraction is below 2. It expires at 2.3000000000000003 on the next small advance. A staggered assertion at 2.3 expires at 4.3 while an assertion observed at 2.3000000000000003 remains active until 4.300000000000001. This distinguishes elapsed-minus-observed-time behavior from a rearranged deadline. Later approval excludes expired evidence. Save/restore preserves pending qualification and later behavior.
4. capacity.jsonl: three approvals succeed, a new challenge reopens revision three, subsequent approvals reject at the runtime history limit, and later support/revocations/expiry do not rewrite that decision. Restores succeed before and after expiry.
5. admission.jsonl: unknown event; missing/extra fields; null, array and scalar payloads; string/boolean/null values; out-of-range/fractional codes; invalid dt; challenge-only approval; duplicate submissions; repeated revocation; and approval with no remaining votes reject as expected. dt 0 and fractional positive dt are accepted. A rejected near-expiry advance leaves its timer intact.

Across all four replay logs there were 44 accepted events, 39 rejected events, and 8 successful restore points. A read-only comparison of the saved logs found every rejected/resumed complete snapshot equal to its predecessor, every previously frozen commitment-ground record unchanged, and every earlier journal prefix unchanged. This comparison inspected output only and did not execute or replace Caveat policy.

## Uncertainties and limitations

No known task mismatch remains from these checks. They are selected authored histories, not exhaustive verification of every ordering or binary64 value. No private verifier feedback was requested or supplied. Raw duplicate-key/nonfinite JSON spellings cannot be meaningfully exercised through this runner's parse/stringify transport; their runtime rejection is described by the public contract. The implementation avoids a source state duration cap by relying on the runtime clock and scheduled qualification.

The final source is version 1, SHA-256 571ab1bd9ce0b9a91efa8aef8cef1a0fb18050effe2fbfe8b3a7f19baa868c50. This note was written before finish; no edits are intended after freezing.
