# Scenario coverage inventory

Before the developer kit replaces anything, every existing program-level check
needs a destination. This inventory lists them and records what each one
requires from the scenario format. Nothing here is deleted: frozen experiments
stay exactly as recorded, and the standard runner runs alongside them.

Scope is the reactive profile. The inventory was taken at runtime revision
`c4b25e1` and updated for the dispatch outcomes of `541e09d`.

The [kit runner](../kit/README.md) makes the checks below. Trail Rescue's 24
scenarios and Glowcap's 38 `cr12` scenarios are
[converted](../experiments/scenario-conversion/README.md) and pass it; the
original harnesses are unchanged.

## Destinations

- **S — standard scenario.** Caveat events sent to a `.cav` source, with
  expectations on the Caveat view or snapshot, checked by the kit's runner.
- **H — host conformance.** A host adapter's own input validation or view
  projection. Evidence about the host, never about a Caveat rule.
- **R — retained.** Stays as custom code: seeded properties and fuzzing,
  oracle scoring, measurement, browser flows, build checks and runtime tests.

## Checks the runner makes on every step

The harnesses repeat these by hand. The standard runner makes them without
being asked, so a scenario author writes only expectations.
[Scenarios 0.1](../spec/caveat-scenarios-0.1.md) specifies them.

1. Admission matches the expected outcome, including its origin under the
   [dispatch outcome contract](../spec/caveat-dispatch-0.1.md).
2. A rejection leaves both the view and the save unchanged. Trail Rescue and
   the authoring studies check both; Glowcap checks the view only.
3. At each `resume`, the restored session has the same full view. The
   sessions from before the resume keep running, and all of them must agree
   on every later admission and view (Trail Rescue, authoring studies).
4. A final save and restore keeps the full view (Trail Rescue).
5. Grounds are a subset of lineage for every state value and commitment
   (`assertGrounds` in the authoring studies). This is a language invariant
   and cheap to check everywhere.
6. A `fatal` outcome fails the scenario and ends that session.

## Trail Rescue — `experiments/trail-rescue/harness.mjs`

| Check | Size | Destination | Notes |
| --- | --- | --- | --- |
| Scenarios TR01–TR24 | 24 scenarios; 151 events, 11 resumes, 33 expected rejections | S (parallel copy) | Sent through `web/trail-rescue-policy.js`, which translates host events and projects the view. Converting means Caveat events and paths into the Caveat view; the projection gets H tests. |
| Non-JSON event rejection is atomic | 17 inputs | H for 15, S for 2 | Non-numeric, non-finite and missing `dt`, and non-object events, stop in the adapter. `dt` −1 and 30.001 reach the core as payload-bound refusals. |
| Source alone controls budget, aging, caveats and outcomes | 1 | S and H | S against a committed variant source. One H test runs the same variant through the adapter, which is what shows the adapter holds no policy. |
| A hidden physical change does not leak | 1 | S | Needs "view unchanged by an accepted event". |
| A negative-time source clock restores its journal and timers | 1 | S | Variant source, a resume, and a journal entry at elapsed −1. |
| Seeded event and restore invariants | 200 × 80 events, seed `0x7a11beef` | R | Domain invariants: scout count, evidence limits, history prefix. The generic ones duplicate runner checks 2–4. |
| `scripts/test-trail-rescue-page.mjs` | browser | R | Real buttons in the shipped page. |

## Glowcap — `experiments/glowcap/`

Glowcap is the benchmark. Its harness, TypeScript side and `runs.jsonl`
scoreboard stay unchanged so earlier rounds remain comparable.

| Check | Size | Destination | Notes |
| --- | --- | --- | --- |
| Scenarios | 42 total, 38 at `cr12`; 13 phases | S (parallel copy of `cr12`, `caveat5` only) | See the conversion notes below. |
| `--measure`, `--bench`, `resume-bench.mjs` | — | R | Measurement. |
| `differential.mjs` | seeded; `--round=6`, `--ordered-decisions` | R | Compares TypeScript with Caveat. |
| `replay-divergences.mjs` and `fixtures/round6-float-divergences.json` | the five pre-fix failures | R | Compares two implementations, not expected values. |
| `compare-views.test.mjs` | — | R | Tests the harness's own comparison. |

Conversion notes. The harness compares arrays as sets (sorted), but
`compare-views.mjs` records that `basis`, `reopenedBy`, `history` and each
entry's `because` keep order; a sorted comparison hid an ordering bug fixed in
`e6ace96`. Expectations are partial. `reject` checks the view but not the save.
`resume` enforces a 4,096-byte save limit. `tick` takes a repeat count.
Scenarios apply to phases through `since` and `until`.

## `scripts/`

| Script | Destination | Notes |
| --- | --- | --- |
| `test-glowcap.mjs` | S for program facts; H for `web/glowcap-explain.js` text | Labels, grounds, journal, late caveats, and two rejections: `already absorbed` (policy) and `does not accept meadow` (input). It already parses rejection text with a helper. |
| `test-slime-glow.mjs` | S for program assertions; H for the wrapper's `reset()` and `free()`; R for build freshness | Needs comparison with an earlier step (relations and commitment bases unchanged since acquisition) and a snapshot growth bound (+128 bytes). `reset()` replaces the session inside `SlimeGlowPolicy`, so "reset equals initial" tests the wrapper. Vessel consumes this program; its side is not covered here. |
| `test-elapsed-clock.mjs` | R (runtime conformance) | Clock-only invalidation, rollback, a manufactured save above `1e12`, signed zero. The manufactured and signed-zero saves are the first candidates for save-compatibility fixtures. |
| `test-beacon.mjs`, `test-rescue.mjs`, `test-launch.mjs`, `test-glowcap-page.mjs` | R | Browser flows; the beacon and rescue games use sessions outside the reactive profile. `test-rescue.mjs`'s `historyComputation` is a browser smoke test of `thermostat_history.cav`. |

## Authoring studies — `experiments/agent-authoring/v1`, `v2`

Both studies are registered and frozen. Their files are not edited.

| Check | Destination | Notes |
| --- | --- | --- |
| `verify.mjs` oracle scoring | R | Independent domain oracle. |
| `checks.mjs` `assertGrounds` | Runner check 5 | Promoted to every step of every scenario. |
| `checks.mjs` `assertSessionMetadata` | S | Assertions on snapshot `elapsed` and `sequence`. |
| `checks.mjs` `assertAdmission` | Runner check 1 | |
| `packet/runner.mjs` | R | The prototype of the kit's CLI. Version and check budgets are study infrastructure. |

## Rust runtime tests — `runtime/tests/`

There are 404 tests in 45 files. They stay in `cargo` (R) and also cover the
native interface.

- Malformed JSON text and duplicate payload keys stay here and in the WASM
  dispatch checks (`scripts/test-dispatch-outcomes.mjs`). A parsed scenario
  file cannot express them, and signed zero stays in `test-elapsed-clock.mjs`.
- 86 `unwrap_err()` sites assert failures by substring, such as
  `contains("rejected: already absorbed")`. Existing APIs are preserved, so they
  keep passing. Moving them to typed outcomes belongs to the outcome
  implementation.
- `thermostat_history`, `slime_glow_ability`, `squeeze_affordance` and
  `ventilation_controller` are scenario-shaped. `squeeze_affordance` already has
  its own fixture schema (`caveat-squeeze-fixtures/1`) and replay file. Keep
  them in `cargo` and add standard-scenario copies as kit examples.
  `thermostat_history` is the authoring guide's walkthrough, so it should be
  the first standard scenario file.

## Rejection-origin audit

Existing results establish acceptance, rejection and atomicity at the tested
boundary. They do not establish which layer rejected an event, because these
sites count any exception as a rejection:

- `experiments/trail-rescue/harness.mjs:18`
- `experiments/glowcap/harness.mjs:103`, `replay-divergences.mjs:16`,
  `differential.mjs:82`
- `experiments/agent-authoring/v1/verify.mjs:46`,
  `v1/session-metadata.mjs:57`, `v2/verify.mjs:53`
- `experiments/agent-authoring/v1`, `v2` `packet/runner.mjs`, which log
  `String(error)` with each rejection

The original results stand as recorded. The
[dispatch audit](../experiments/dispatch-audit/README.md) qualifies Trail
Rescue and the authoring-study logs. Glowcap is measured here with the same
structured API, against the clean compiled build of `541e09d`. Its histories
were replayed on `caveat5/glowcap.cav` with the adapter's payload mapping; that
adapter does no validation.

| Suite | Expected rejections | `policy/reject` | `input` | Host adapter | Fatal |
| --- | ---: | ---: | ---: | ---: | ---: |
| Trail Rescue TR01–TR24 ([audit](../experiments/dispatch-audit/TRAIL_RESCUE.md)) | 33 | 20 | 2 | 11 | 0 |
| Glowcap `cr12`, `caveat5` | 21 | 15 | 6 | 0 | 0 |

- All 11 Trail Rescue adapter rejections are in TR12, with its two `input`
  refusals (`dt` −1 and 30.1). TR12 is evidence about the adapter and a payload
  bound, not about Caveat rules.
- Glowcap's six `input` refusals are two `bound_exceeded` (`dt` 0.2 and −0.1)
  and four `payload_invalid` (an unknown target or sort). All 21 rejections
  left the save and view unchanged. The same replay accepted 24,315 events
  without a fatal outcome.
- Legacy diagnostics report an authored rejection as
  `event <name>, rule <n>: rejected: <message>`, not with a prefix. The dispatch
  audit's author-log classification is a retrospective inference from that
  text, as it states.
- Not yet audited: `verify.mjs` in both studies (needs each frozen packet
  runtime), `differential.mjs` and `replay-divergences.mjs`.

## What the scenario format must support

1. Caveat events, and JSON Pointer paths into the Caveat view and snapshot.
2. A source per scenario file, so variant sources are committed files.
3. An expected outcome per event: accepted, or rejected with origin, code and
   message. A bare `rejected` means a policy rejection.
4. Partial matching by default, and full-view equality on request.
5. Arrays compared in order by default; set comparison is opted into per path.
6. "Unchanged" relative to the previous step, and named checkpoints to compare
   with a later step.
7. Snapshot metadata: `elapsed` and `sequence`.
8. `resume` steps and event repeat counts.
9. Optional byte bounds on the save and snapshot.

Phase filtering is not required, because Glowcap keeps its own harness.
