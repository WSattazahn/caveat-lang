# A1 author notes

## Scope and access

Implemented the assigned cold-storage task independently in `candidate.cav`.
All application policy is Caveat source. No host policy implementation, runtime changes, imports, dependencies, or commits were made.
Access-rule deviations: none. I did not inspect other candidates, v1 artifacts, study results/protocol, private verification, repository examples/games, runtime implementation, or the web. Links outside the packet were not followed. I did not contact or spawn other agents.

## References read

- `v2/prompts/A1.md`
- `v2/tasks/cold-storage.md`
- `v2/packet/README.md`
- `v2/packet/runner.mjs`
- `v2/packet/reference/docs/AI_AUTHORING.md`
- `v2/packet/reference/spec/caveat-0.1.md`
- `v2/packet/reference/spec/caveat-reactive-0.1.md`
- `v2/packet/reference/spec/caveat-reactive-0.2.md`
- `v2/packet/reference/spec/caveat-reactive-0.3.md`
- `v2/packet/reference/spec/caveat-reactive-0.4.md`
- `v2/packet/reference/spec/caveat-reactive-0.5.md`
- `v2/packet/reference/spec/caveat-renewal-0.1.md`
- `v2/packet/reference/spec/caveat-late-qualification-0.1.md`
- `v2/packet/reference/spec/caveat-state-caveats-0.1.md`
- `v2/packet/reference/spec/caveat-reject-0.1.md`
- `v2/packet/reference/spec/caveat-elapsed-0.1.md`
- `v2/packet/reference/spec/caveat-explanations-0.2.md`
- `v2/packet/reference/spec/caveat-decision-journal-0.1.md`
- `v2/packet/reference/spec/caveat-observation-order-0.1.md`

I also listed the filenames under packet/reference to locate relevant specifications and inspected only my own source, event histories, and runner snapshots. One large batch of specification output was truncated; the relevant qualified-computation and text profiles were subsequently read in full in smaller batches.

## Source design

The event declarations enforce the exact numeric payload contract. A bounded renewable `sensor_reading` supplies the required first identity and subsequent `@N` occurrences. Every read reveals one supports/opposes relation, constructs fresh qualified temperature and assessment values, and schedules `stale after 2` against that occurrence. The declared calibration caveat is inherited independently by each renewal. The clock is read directly with `elapsed()` and has no duplicate bounded state accumulator.

The three-entry `dispatch` decision series owns the real frozen commitments and journal. A separate live `decision_basis` preserves the exact occurrence used by the current commitment so late qualification can reopen it via `caveated(decision_basis, stale)` after renewal. Reopening is guarded against already open decisions. New readings do not commit or reopen anything. Decision grounds contain only the current assessment's reading occurrence and calibration caveat; predecessor lineage remains available separately in the runtime. Bindings provide exactly the seven required hud properties.

## Runtime checks (one source version; five checks)

1. `validate`: valid source; seven initial hud values match the task.
2. `replay lifecycle.jsonl`: release at the inclusive threshold, fractional blocking input, missing/closed decision rejection, an older committed reading aging while the latest stays fresh, three revisions, fourth-decision and fifth-reading rejection, zero increment, save/restore, final stale state.
3. `replay fractional.jsonl`: read at elapsed 0.3; advancing by 2 leaves elapsed 2.3 with a fresh reading because binary64 subtraction is below 2. A zero increment preserves freshness. One representable increment then qualifies and reopens it. This distinguishes subtraction-based aging from a rearranged deadline comparison. Save/restore was exercised on both sides of that boundary.
4. `replay admission.jsonl`: missing/extra fields, strings, booleans, nulls, objects, arrays, scalar/null payloads, unknown event, out-of-range values, numeric endpoints, fractional temperatures, simultaneous aging of all four observations, reading capacity, atomic rejection with pending timers, zero increment, and save/restore.
5. `replay repeated.jsonl`: four equal readings remain distinct occurrences; stale reading before any decision rejects; a fresh renewal supports the first decision; exact older occurrence witnesses reopen each subsequent decision with unchanged frozen block values.

The actual task behavior was inspected in the supplied runner's output and retained JSONL snapshots. A read-only PowerShell comparison over the four replay logs checked 87 snapshots: all 33 rejected snapshots and 10 restored snapshots equaled their immediate predecessor in full. It also checked that every previously recorded commitment basis and grounds entry, and every earlier journal entry, remained unchanged. These comparisons do not execute candidate source or implement application policy.

## Uncertainties and limits

No mismatch was found in these self-authored cases. Testing is finite, not an exhaustive proof, and no private verifier feedback was received. The runner parses/stringifies JSON: the `1e309` admission input was rejected after this transport normalization, and duplicate raw JSON keys / raw nonfinite spellings were not directly tested. Scheduled qualification arithmetic was checked through the supplied runner's observable fractional-boundary behavior; runtime implementation was not inspected.

The final candidate is the original validated source version, without repairs or post-submission source edits.
