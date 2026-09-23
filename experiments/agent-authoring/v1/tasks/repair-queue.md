# Repair queue

Implement one `.cav` reactive program. This is a synthetic policy for this authoring test. The generic host only dispatches inputs, saves/restores the session, and reads runtime output. All policy belongs in the source. No application-specific JavaScript or fabricated history is permitted.

## Inputs and admission

| Event | Exact numeric payload |
| --- | --- |
| `inspect` | integer `cause` in `1..2`, integer `score` in `0..2` |
| `choose` | integer `repair` in `1..2` |
| `outcome` | integer `success` in `0..1` |
| `next` | empty |

There is no clock; runtime elapsed time remains zero. Unknown events, extra/missing parameters, nonnumeric/nonfinite/out-of-range inputs, fractional codes/scores, and domain violations below reject atomically. Rejections preserve the entire session, including diagnostic tokens, identity counters, graph, history, and sequence. Error message wording is unrestricted.

## Investigation and repair

Start in investigation 1, phase `investigate`, with **three diagnostic tokens total for the whole session**. There can be at most two investigations, and tokens are not replenished.

`inspect` is permitted only during `investigate` and spends one token. Cause 1 is bearing: each accepted inspection observes a fresh renewable occurrence of `bearing_check` supporting `bearing_fault`. Cause 2 is motor: similarly use `motor_check` supporting `motor_fault`. For each base evidence the first occurrence has the base name, then `@2`, then `@3`; each renewable's capacity is three. Every diagnostic occurrence carries declared caveat `alternative_cause`. Repeating the same cause is permitted and must create a distinct observation, even for an identical score. It replaces only that cause's latest score and identity for the current investigation. The other cause's current diagnosis remains. Neither cause defeats or erases the other's graph evidence.

`choose` is permitted only during `investigate`, and requires a latest score **greater than zero for the selected cause in this investigation**. Its numeric frozen value is the selected repair code, not the score. Commit to decision series `repair`, capacity **two**, grounded exactly on the selected cause's latest diagnostic occurrence. Retain caveats `alternative_cause` and `repair_unverified`. The second is an unresolved limitation of the selected repair, not a qualification on the diagnostic evidence. Choosing changes phase to `awaiting`. The frozen grounds must not include the alternative cause's observation, an older same-cause observation, a prior investigation, or an earlier failure merely because those influenced control flow.

`outcome` requires phase `awaiting`. Success 1 changes phase to `complete`, a terminal state; it creates no new evidence and does not reopen. Success 0 observes a fresh occurrence of renewable `failure` (capacity two), opposing `repair_effective`, and reopens the current repair because of that failure. Failure has no caveats. Then phase becomes `failed`; the original repair value and grounds remain frozen.

`next` is allowed only in phase `failed` after investigation 1. It starts investigation 2 in phase `investigate`, clears both current diagnosis scores and current selected repair, and preserves remaining tokens and the whole graph and decision history. Evidence occurrence numbering continues across investigations. The already reopened first decision permits a second deliberate choice. `next` after success, before failure, or after investigation 2 rejects. After the second failure there is no further investigation.

## Observable output

Always bind all and only these properties on `hud` (other binding targets are permitted):

| Property | Value |
| --- | --- |
| `investigation` | 1 or 2 |
| `tokens` | remaining diagnostic tokens, initially 3 |
| `bearing` | latest current-investigation bearing score; -1 when absent |
| `motor` | latest current-investigation motor score; -1 when absent |
| `phase` | `investigate`, `awaiting`, `failed`, or `complete` |
| `selected` | selected repair in this investigation, initially/reset to 0 |
| `frozen` | latest decision's frozen repair code, initially 0; remains previous value across `next` |
| `revision` | number of committed repair decisions, initially 0 |

Use actual observations, qualification edges, commitments, `decision_journal`, and `commitment_grounds`; no replacement text history. Relevant observed `supports`/`opposes` relations retain observation order. Direct qualification edges on observed evidence contain exactly `alternative_cause` for diagnoses and no caveats for failure. Grounds evidence/caveat arrays are lexical; journal `because` is first-observation order and journal caveats are lexical. Each journal entry uses the accepted sequence/event, elapsed zero, and the frozen repair code, including on reopening. Save/restore preserves the complete session and future behavior. You may share source behavior across causes, but no particular syntax or source layout is required.
