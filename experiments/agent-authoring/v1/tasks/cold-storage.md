# Cold-storage dispatch

Implement one `.cav` reactive program. This is a synthetic policy for this authoring test; its temperatures and decisions are not physical food-safety guidance. The generic host only dispatches inputs, saves/restores the session, and reads runtime output. All policy belongs in the source. No application-specific JavaScript or fabricated history is permitted.

## Inputs and admission

Events have exactly these numeric payloads, with inclusive bounds:

| Event | Payload |
| --- | --- |
| `read` | `temperature` in `-10..20` |
| `decide` | empty |
| `advance` | `dt` in `0..2` |

`advance` is the source clock. Elapsed time starts at zero and increases by its accepted `dt`; all other events leave it unchanged. Clock arithmetic uses binary64 `elapsed = elapsed + dt`, without rounding or tolerance. Fractional temperatures and time increments are valid. Unknown events, extra/missing parameters, nonnumeric/nonfinite values, and values outside bounds reject atomically. Every additional domain rejection below also leaves the entire session unchanged, including sequence, identities, history, and pending timers. Error message wording is unrestricted.

## Observations and decisions

There are at most **four** accepted readings. Each `read` creates a new occurrence of renewable evidence `sensor_reading`: `sensor_reading`, `sensor_reading@2`, etc. Each occurrence supports claim `dispatch_safe` when its temperature is at most 4, and opposes it otherwise. Every occurrence carries the declared caveat `calibration_uncertain`. It additionally receives caveat `stale` when binary64 `elapsed - observedAt >= 2`, where `observedAt` is elapsed at that observation; use this subtraction, not an algebraically rearranged deadline comparison. A due qualification applies before that `advance` event's decision checks. An old occurrence ages on its own schedule after renewal.

The latest reading controls the live recommendation: no reading → `waiting`; stale latest reading → `stale`; otherwise temperature at most 4 → `release`, otherwise → `block`. Reading a new temperature alone never reopens or revises a decision.

`decide` requires a latest, nonstale reading and either no decision or an explicitly reopened current decision. It freezes 1 for release or 0 for block in decision series `dispatch`, with capacity **three**. Its grounds are exactly that latest reading occurrence and its `calibration_uncertain` caveat. Every decision's number and grounds remain frozen permanently.

On `advance`, if the current decision is closed and the specific reading occurrence on which it was based has become stale, reopen it because of that occurrence. This is required even if a newer reading is still fresh. Record the reopening with the witness's current caveats (`calibration_uncertain`, `stale`). Already open decisions get no additional reopening entry. An elapsed increment of zero is valid. Capacity exhaustion rejects atomically, without consuming an occurrence or revision.

## Observable output

Always bind all and only these properties on `hud` (other binding targets are permitted):

| Property | Value |
| --- | --- |
| `readings` | accepted reading count, initially 0 |
| `temperature` | latest temperature, initially 0 |
| `recommendation` | `waiting`, `stale`, `release`, or `block` as above |
| `decision` | `none` initially; `review` if reopened; otherwise frozen `release` or `block` |
| `frozen` | -1 initially; latest decision's frozen 0 or 1 |
| `revision` | number of committed revisions, initially 0 |
| `elapsed` | elapsed clock time |

Use real observed graph relations, real commitments, and runtime `decision_journal` and `commitment_grounds`. Do not encode a replacement journal in text. Relevant observed `supports`/`opposes` relations retain observation order. For each observed evidence occurrence its direct `qualifies` edges must include exactly the caveats specified above. Grounds evidence and caveat arrays are lexical; journal `because` is first-observation order and journal caveats are lexical. Journal entries use the actual accepted event sequence/name and elapsed time, with the frozen decision value also on reopen entries. Declaration-only evidence is not an observation. Save/restore must preserve the full session and future behavior.
