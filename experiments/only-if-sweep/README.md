# Only If sweep

Evidence for [Only If 0.1](../../spec/caveat-only-if-0.1.md), the 1.4 draft
that adds `only_if(CONDITION, VALUE)`.

## The source sweep

`node experiments/only-if-sweep/sweep.mjs [--write]` reads every tracked
`.cav` file as text and writes [`RESULTS.json`](RESULTS.json). It fails if any
program written before the profile mentions `only_if`. Such a program is the
only kind whose meaning the profile could touch: before it, `only_if(...)`
loaded only as a call to a program's own `fn only_if`, which keeps its
meaning. The one program that mentions it,
[`experiments/model-votes/votes.cav`](../model-votes/votes.cav), was written
for the profile.

## The behavior comparison

[`COMPARE.json`](COMPARE.json) is the output of
`node experiments/untrusted-sources-sweep/compare.mjs BASELINE` with BASELINE
the published `caveat-lang@1.3.0`. It runs `validate --json` on every tracked
program and `test --json` on every tracked scenario file, under 1.3.0 and under
this checkout (after `npm run build`), and lists each file whose status or
output differs. The only differences are the model-votes program and its
scenario file, which use the 1.4 drafts and do not load on 1.3.0.
