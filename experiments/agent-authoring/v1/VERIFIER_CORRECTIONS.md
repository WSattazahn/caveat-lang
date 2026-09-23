# Verifier corrections

## Explicit null payloads, before scoring

After registration and launch of wave one, root review found that the scoring
transport used `event.payload ?? {}`. This changed a deliberately invalid
`null` payload into an empty object before the runtime could reject it.
The scorer now forwards `JSON.stringify(event.payload)` unchanged. The task
contracts, oracle, scenario inputs, seeds and scoring criteria are unchanged.
No candidate source or verifier outcome was inspected to motivate this repair;
review found it while reading the independent oracle's malformed-input cases.
The original scorer remains preserved in registration commit `fba9c78`.

All six candidates will be scored with this correction. Authors receive no
private result or policy hint. The public self-check wrapper retains its
registered behavior, including defaulting a null/missing payload to `{}`;
author runtime check logs are preserved as produced. This is a tool limitation
to consider if an author's self-check used an explicit null envelope.

## Runtime metadata coverage, after scoring

Independent review after all six candidates were frozen and scored found that
the registered domain projection omits session elapsed and sequence. HUD time
and journal fields constrain these indirectly, but task C explicitly requires
elapsed zero even before any decision. Mutating a saved C1 snapshot in memory
to elapsed 123 and sequence 999 still passed the original projection/grounds
assertions. This demonstrates a missing check, not a candidate defect.

The original scorer and `results/scored.json` remain unchanged. A separately
labelled `session-metadata.mjs` repeats the same corpus for all first and final
sources, checking snapshot elapsed and sequence against the independent oracle
initially, after each step, and after final restore. It includes two deliberate
corruption checks. Results are in `results/session-metadata.json`.

The first draft of this supplement also checked the compact runtime view,
which does not expose elapsed. Every loadable candidate consequently failed
before any event with `undefined !== 0`. That checker error is preserved in
`results/session-metadata-draft-error.json`; removing the compact-view check
leaves the intended full-snapshot checks. No candidate, task, oracle, seed or
registered score was changed in response.
