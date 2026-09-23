# Two-source access review

Implement one `.cav` reactive program. This is a synthetic policy for this authoring test, not a recommended real authorization policy. The generic host only dispatches inputs, saves/restores the session, and reads runtime output. All policy belongs in the source. No application-specific JavaScript or fabricated history is permitted.

## Inputs and admission

| Event | Exact numeric payload |
| --- | --- |
| `submit` | integer `code` in `1..5` |
| `revoke` | integer `code` in `1..4` |
| `approve` | empty |
| `advance` | `dt` in `0..2`, fractions allowed |

`advance` is the source clock; elapsed starts at zero. Clock arithmetic uses binary64 `elapsed = elapsed + dt`, without rounding or tolerance. Unknown events, extra/missing parameters, nonnumeric/nonfinite/out-of-range inputs, fractional codes, and domain violations below reject atomically. A rejection preserves all state, sequence, observations, pending timers, and decisions. Error message wording is unrestricted.

## Fixed assertions

Each submission code can be accepted **once** in the whole session. Do not renew or replace these evidence identities:

| Code | Evidence | Relation |
| --- | --- | --- |
| 1 | `alpha_allow` | supports `access_allowed` |
| 2 | `beta_allow` | supports `access_allowed` |
| 3 | `alpha_deny` | opposes `access_allowed` |
| 4 | `beta_deny` | opposes `access_allowed` |
| 5 | `challenge` | supports `review_required` |

Assertions 1–4 each carry declared caveat `unverified_source`. They expire when binary64 `elapsed - observedAt >= 2`, where `observedAt` is elapsed at observation; use this subtraction, not an algebraically rearranged deadline comparison. They gain caveat `expired` before that `advance` event's rules. `revoke` requires the named assertion to have been observed and not previously revoked; it adds caveat `revoked`. An expired assertion may still be revoked once. Assertions stay observed, and all supporting/opposing relations coexist permanently, including contradictory assertions from the same source.

An assertion votes only while observed, unexpired, and unrevoked. Each active allow assertion contributes one support vote; each active deny assertion contributes one opposition vote. Challenge never votes and never expires. The live verdict is `undecided` with no active votes, otherwise `allow` if support exceeds opposition, otherwise `deny` (ties deny).

`approve` explicitly freezes this verdict as numeric 1 for allow or 0 for deny in decision series `access`, capacity **three**. It requires at least one active vote and either no decision or a reopened current decision. Its grounds are **all currently active voting assertions**, including opposition, and their `unverified_source` caveat. Inactive assertions and challenge are excluded from its grounds.

A newly accepted opposition (code 3 or 4), or challenge (code 5), reopens the current decision if one exists, because of that exact new evidence. Each adds its own reopening journal entry even if the decision was already open. New support never reopens. Revocation and expiration change the live tally but **do not reopen, revise, or mutate a frozen approval**. This separation is deliberate. A later explicit approval uses only the votes active then. Capacity exhaustion rejects atomically.

## Observable output

Always bind all and only these properties on `hud` (other binding targets are permitted):

| Property | Value |
| --- | --- |
| `support` | active support votes |
| `opposition` | active opposition votes |
| `verdict` | live `undecided`, `allow`, or `deny` |
| `decision` | `none` initially; `review` if current decision reopened; otherwise frozen `allow` or `deny` |
| `frozen` | -1 initially; latest approval's 0 or 1 |
| `revision` | number of approvals, initially 0 |
| `elapsed` | elapsed clock time |

Use real observations and runtime commitments, `decision_journal`, and `commitment_grounds`, not a replacement text history. Relevant observed `supports`/`opposes` relations retain observation order. Direct `qualifies` edges on observed evidence must contain exactly the caveats specified above. Grounds evidence/caveat arrays are lexical; journal `because` uses first-observation order, and journal caveats are lexical. Journal entries carry actual accepted sequence/event, elapsed time, and frozen decision value (including reopen entries). Save/restore preserves the complete session and subsequent behavior.
