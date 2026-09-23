# Trail Rescue: a new Caveat mechanic

Status: **requirements fixed before implementation** once this document and
`scenarios.json` are committed. These requirements were specified independently
of the implementation by a separate agent that read the project's public
overview and previous experiment protocol, but did not read runtime or game
sources. This is not a claim that the author was blind to Caveat's capabilities.
There is no competing implementation or benchmark winner in this exercise.

## The player's problem

A firefly is trapped beyond two tunnels, `stone` and `reed`. A scout has three
inspection tokens. A visitor may report on each tunnel once, for free, but a
visitor can be mistaken. Conditions can change without the scout seeing them.
The player must make a supported route plan, reconsider it when its grounds are
challenged, and actually attempt the rescue. A reasonable plan can still fail.

The complete beat is one rescue attempt. Initially stone is physically clear,
reed is physically blocked, neither has been observed, and no route is planned.
Physical conditions are simulation state, never disclosed by the knowledge view.

## Rules fixed before coding

1. A direct scout costs one token, including repeat inspections, and discovers
   the tunnel's actual current condition. A report costs nothing and says what
   its sender supplies, whether true or false. Each tunnel accepts one report.
2. Every accepted observation has its own identity. Earlier observations are
   retained. Reports carry `secondhand`; direct scouts do not.
3. At age **30 seconds or greater**, an observation also carries `stale` and
   stops contributing to the current assessment. Before that exact boundary it
   contributes. New information does not silently erase older disagreements.
4. A tunnel is `clear` with fresh clear observations only, `blocked` with fresh
   blocked observations only, `disputed` with both, and `unknown` with neither.
   A plan requires `clear`. A secondhand clear report is sufficient, openly
   qualified as secondhand. Stale observations are not sufficient.
5. Planning freezes all fresh clear observations for that tunnel, in the order
   acquired, and their caveats at that moment. Later supporting observations
   change the current assessment but never rewrite that decision.
6. A committed plan reopens once if a newly acquired observation says its tunnel
   is blocked, or any observation in its frozen basis becomes stale. The latter
   still reopens it when newer evidence keeps the tunnel's current status clear.
   Unrelated observations, hidden physical changes, and stale observations that
   were never in the basis cannot reopen it. Further challenges while already
   reopened do not add repeated reopening entries.
7. After reopening, the player can plan either currently clear tunnel, including
   the same tunnel on a refreshed basis. A still-committed plan cannot be replaced.
   A new commitment clears its `reopenedBy` and replaces its current frozen basis;
   it does not remove the earlier history.
8. Rescue requires a committed plan. It succeeds exactly when that tunnel is
   physically clear at the attempt; otherwise it fails. Either outcome ends the
   session. All later gameplay events are rejected without mutation.
9. A rejected event changes nothing: no token, evidence occurrence, time, world
   condition, decision, or history entry may be consumed or changed.
10. Save/resume preserves the complete view and every subsequent event's result,
    including hidden conditions, report availability, scout numbering, caveat
    deadlines, and decision order. A save is JSON, not an event replay recipe.

The finite inspection/report budget allows at most five observations. Replanning
requires a reopening caused by one of those observations, so this beat does not
impose an unbounded decision-history requirement.

## Host events

The policy exports `createPolicy(saved?)` returning `dispatch(event)`, `view()`,
and `save()`. Dispatch returns normally when accepted and throws when rejected.
The host may translate these event envelopes into source-declared events; it may
not implement the game's knowledge, resources, plan, aging, or rescue rules.

| `type` | Other fields | Meaning |
| --- | --- | --- |
| `observe` | `tunnel`, `method: "scout"` | Spend one token and read actual condition. `condition` must be absent. |
| `observe` | `tunnel`, `method: "report"`, `condition` | Record the tunnel's one free report. |
| `change` | `tunnel`, `condition` | Change the simulated physical condition without revealing it. |
| `tick` | `dt` | Advance time by a finite number from 0 through 30 inclusive. |
| `plan` | `tunnel` | Commit to a currently clear tunnel if there is no committed plan. |
| `rescue` | none | Make the one rescue attempt. |

There are five event types (`observe` has two methods). Tunnels are `stone` and
`reed`; conditions are `clear` and `blocked`. Unknown event types, unknown
names, missing required fields, wrong field types, and non-finite numbers are
rejected. No particular exception wording is required. Ordinary extra fields
may be ignored, except a scout's supplied `condition` must be rejected so the
host cannot invent a direct observation. Time uses ordinary JavaScript/Rust
binary64 addition: `now = now + dt`; an observation ages when
`now - observedAt >= 30`. Save/resume must preserve those numbers exactly.

Observation ids are `report_<tunnel>` or `scout_<tunnel>_<n>`, where `n` counts
accepted scouts of that tunnel, starting at 1. A rejected scout consumes no id.
Reports and scouts are ordered by acceptance, never by their ids. Simultaneously
aging basis observations cause one reopening citing all such observations in
acquisition order. Caveat lists use the stable order `secondhand`, then `stale`.

## Observable view

`scenarios.json` contains the exact initial view. Every subsequent view has the
same shape. The entire view is ordinary JSON; objects' key order is irrelevant,
but **every array is ordered** and is compared without sorting.

- `now`: elapsed time; `scoutsRemaining`: initially 3.
- `outcome`: `pending`, `rescued`, or `failed`; `canRescue` is true exactly when
  outcome is pending and the decision is committed.
- `evidence`: every observation in acquisition order, as
  `{ id, tunnel, method, condition, at, caveats }`. Aged evidence remains here.
- `tunnels.<tunnel>`: `{ status, clearBy, blockedBy, because, caveats,
  canScout, canReport, canPlan }`. `clearBy`/`blockedBy` are the fresh observations
  with that result. `because` combines both lists in acquisition order; caveats
  are their union in the stable caveat order. `canScout` requires a pending
  session and a token; `canReport` requires a pending session and an unused
  report for this tunnel. `canPlan` requires a pending session, status clear,
  and no currently committed plan.
- `decision`: `{ state, tunnel, basis, caveats, reopenedBy, history }`.
  Initially `state` is `none`, `tunnel` is null, and all arrays are empty.
  State is otherwise `committed` or `reopened`; reopening preserves tunnel,
  basis, and its frozen caveats. `reopenedBy` is the evidence cited by the one
  reopening of the current commitment. On rescue, the decision stays intact.
- `decision.history`: entries in decision-change order, each
  `{ change, tunnel, at, because, caveats }`. `change` is `committed` or
  `reopened`. A commitment entry freezes its basis and caveats; a reopening
  entry freezes its cause and caveats at reopening. Thus a stale secondhand
  cause has both caveats even though the original commitment retains only
  `secondhand`. Later aging cannot rewrite any history entry.

## Scenario format and verification

`scenarios.json` is data for a runtime-independent JavaScript harness, not a
second gameplay implementation. Every scenario begins from `initialView`.
Steps have one of these forms:

- `{ "op": "dispatch", "event": {...}, "expect": {"/json/pointer": value} }`
  requires acceptance, then exact deep equality at each JSON Pointer.
- The same with `"reject": true` requires rejection and an unchanged full
  view; assertions may follow. The harness should also compare a resumed copy
  against the untouched pre-rejection save to expose hidden partial writes.
- `{ "op": "resume" }` saves, round-trips through `JSON.stringify` and
  `JSON.parse`, restores, and compares complete views. Keep the pre-resume
  session as a shadow and compare acceptance/rejection and full views on the
  remaining events. This checks behavior, not merely a matching screenshot.

Every scenario also ends with a save/resume comparison. Runtime tests should
add non-JSON inputs (NaN/infinity) and corrupted-save safety separately, since
JSON fixtures cannot represent them. A deterministic invariant run should
exercise random event orders and resume points: at most three paid scouts,
one report per tunnel, no mutation on rejection, no hidden condition leakage,
frozen historical facts, and the plan/rescue admission rules above. It need
not recreate the complete game as an independent implementation.

Evidence to record after implementation: scenario pass/fail counts; any initial
failures; source/runtime changes the mechanic required; the checks actually run;
and any remaining limitation. Requirements changed after the registration commit
must be listed below with their reason. Do not call a known request a blind test.

## Amendments

None at registration.
