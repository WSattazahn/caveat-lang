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
