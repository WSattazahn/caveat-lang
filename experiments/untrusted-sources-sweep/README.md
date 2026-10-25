# Untrusted Sources sweep

Evidence for [Untrusted Sources 0.1](../../spec/caveat-untrusted-sources-0.1.md),
the 1.4 draft that answers items 2 to 5 of the
[prompt injection experiment](../prompt-injection/README.md#limits-found):
one action on several sources, one confirmation per action, the undo list and
withdrawing a whole source.

## The source sweep

`node experiments/untrusted-sources-sweep/sweep.mjs [--write]` reads every
tracked `.cav` file as text and writes [`RESULTS.json`](RESULTS.json). It
fails if any program writes `withdraw every NAME` or `because once E`, which
do not load before this profile. It also lists declarations named `every` or
`once`, whose meaning must not change, the streams a program withdraws from
several rules, nested routed blocks, events naming two members of one kind,
and questions about past revisions. The spec's "Corpus sweep" section quotes
its numbers.

## The behavior comparison

`node experiments/untrusted-sources-sweep/compare.mjs BASELINE [--write]`
runs `validate --json` on every tracked program and `test --json` on every
tracked scenario file, once with the CLI at BASELINE and once with this
checkout's `kit/bin/caveat.mjs` (after `npm run build`), and lists every file
whose status or output differs. Durations are zeroed first. With BASELINE the
published `caveat-lang@1.3.0` (`npm i caveat-lang@1.3.0` in an empty folder,
then `node_modules/caveat-lang/bin/caveat.mjs`), [`COMPARE.json`](COMPARE.json)
records the run on this profile's branch: 203 programs (172 load) and 70
scenario files (49 pass; the rest are deliberate failures, such as the
scenario format's fault fixtures and the authoring trials' mutants), with no
difference in status or output. The scenario report's `runtime` block, which
names the runtime that ran it, is left out of the comparison.

The comparison is of what a host sees. It does not cover saves made by
`caveat-lang@1.3.0` restored here; the runtime tests and the kit's save
fixtures do.
