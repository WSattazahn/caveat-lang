# A1 author notes

## References read

- Assigned prompt: experiments/agent-authoring/v1/prompts/A1.md.
- Public task: experiments/agent-authoring/v1/tasks/cold-storage.md.
- Packet README.md and runner.mjs.
- Packet reference/docs/AI_AUTHORING.md.
- Packet reference/spec/caveat-reactive-0.1.md through caveat-reactive-0.5.md (the first batch output was partly truncated; relevant 0.3-0.5 documents were read again separately).
- Packet reference/spec/caveat-renewal-0.1.md, caveat-late-qualification-0.1.md, caveat-state-caveats-0.1.md, caveat-decision-journal-0.1.md, caveat-reject-0.1.md, caveat-explanations-0.2.md, and caveat-save-0.1.md.
- A text search restricted to packet/reference/ additionally returned short matching snippets from docs/CAVEAT_ESSENCE.md, spec/caveat-reactive-0.7.md, and spec/caveat-text-0.1.md. No links outside the packet were followed.
- Own candidate, record, and runtime check logs.

## Submissions and self-checks

Three distinct sources were submitted, using eight runner validation/replay checks in total. No alternate candidate execution path was used.

1. Version 1 failed initial validation because a numeric state's declared upper bound exceeded the runtime's maximum of 1e12. No gameplay replay ran for that version.
2. Version 2 used an ordinary bounded clock and passed main, fractional, and admission histories. Its clock would eventually have rejected advances above the state's maximum, so it was superseded.
3. Version 3 represents elapsed time using a mantissa and a power-of-two scale. It passed four histories:
   - main.jsonl: missing reading, closed decision, old decision basis aging while a newer reading is fresh, three revisions, stale decisions, four-reading and three-decision capacities, zero advances, and save/restore with future events.
   - fractional.jsonl: reading at 0.3 remains fresh at 2.3 because binary64 subtraction is below 2; a following 4.440892098500626e-16 increment qualifies and reopens it. Equal renewed temperatures receive distinct identities. Save/restore preserves pending aging.
   - admission.jsonl: unknown events, extra/missing parameters, strings, booleans, null, attempted nonfinite input, out-of-range temperature/dt, valid temperature endpoints, and the representable boundary immediately below 2 seconds.
   - repeated.jsonl: four observations including repeated equal readings, simultaneous qualification of several occurrences, reopening only on the specific committed occurrence, unchanged closed decisions after reads, and no duplicate reopening on zero advances.

The version 3 logs are checks/05.jsonl through checks/08.jsonl. Post-replay comparisons of those logs found full-snapshot equality for all 24 rejected events and all nine restore events. Every snapshot had exactly the seven required hud properties, and hud.elapsed equaled runtime elapsed. The repeated-reading history's final observed relations were supports, supports, opposes, supports in occurrence order. Every occurrence had exactly calibration_uncertain and, once due, stale direct qualification edges. Its decision grounds and reopening journal witnesses selected sensor_reading@2 and then sensor_reading@4, without importing other observations into grounds.

## Why the final candidate follows the contract

Input signatures own numeric admission. Explicit reject rules cover unavailable/stale readings and closed decisions. Renewable sensor_reading identities and per-occurrence delayed qualifications preserve observation and aging history. The separately retained decision_basis gains late caveats without following a renewed alias; has_caveat and caveated therefore reopen on the exact basis occurrence. Commitments and the runtime journal freeze each decision's numeric value and grounds. Decision-series and renewable-evidence capacities remain runtime-enforced; a rejected fourth revision rolls back its preceding basis assignment.

The source clock mirrors the runtime's binary64 elapsed + dt operation. At a large clock value it changes representation by exact multiplication/division by 32768. With nonnegative dt <= 2, binary64 accumulation starting at zero cannot progress past 2^54: at that point even +2 rounds back to 2^54. Dividing by 32768 keeps the state within 2^39, below the runtime's 1e12 state limit. The rescaling transition is intended to preserve the exact represented elapsed value and leaves the ordinary low-time path unchanged.

## Uncertainties and limitations

- These are authored public self-checks, not hidden-verifier results; no private feedback was supplied or consulted.
- The large-time rescaling branch has been reasoned about but not reached by replay. Reaching it through admissible events requires hundreds of billions of advances, so it is not covered by the eight runtime checks.
- The runner parses and stringifies event payloads. The attempted 1e309 input reaches the runtime as null and is rejected; this does not directly test the runtime's nonfinite numeric parser. Duplicate JSON keys likewise cannot be meaningfully tested through that parse/stringify path.
- No exhaustive exploration of every finite input history is claimed.

## Access rules

No access-rule deviations. No runtime implementation, private verifier/oracle/tests, other candidates, existing repository examples/games, study results, or web sources were read. No other agents were contacted or spawned. Only own runs/A1 files were written, aside from files the authorized runner itself records. No preserved versions, check logs, or runner records were edited. No host gameplay policy implementation or commits were created.
