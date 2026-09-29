# Performance Optimization 0.1: the view path

`session.dispatchView(event, payload)` in the kit, and the runtime's
`dispatch_view_outcome` it calls, give a host the dispatch outcome with the
view after the event, and never build the snapshot
([Dispatch 0.1](../../spec/caveat-dispatch-0.1.md#the-view-path)). The
existing `session.dispatch()`, `dispatch_outcome`, `dispatch_view` and
`dispatch` are unchanged.

- [correctness/](correctness/README.md): the evidence that the new path
  returns what the existing ones do and that they are unchanged.
- The timings and the size table are **not recorded yet**. They are to be
  taken in a quiet window, as the baseline's were, and will go in
  `RESULTS.md`, with the raw results in `results/`, both in this directory.
  The tools that produce them are here and in the baseline's harness.

[Performance Baseline 0.1](../performance-0.1/RESULTS.md) is frozen evidence:
nothing under `experiments/performance-0.1/results/` or its RESULTS.md
changed. Its harness gained two modes and two suites, and nothing it measured
before changed:

| Mode | Times, per event |
| --- | --- |
| `kit.dispatchView` | `session.dispatchView(event, payload)`: the kit's payload check and encoding, the call, and `JSON.parse` of the outcome |
| `raw.dispatch_view_outcome` | as `raw.dispatch_view`: `JSON.stringify` of the payload, `WebReactiveSession.dispatch_view_outcome`, and `JSON.parse` of its text |

The `view-path` suite runs those two and the baseline's `raw.dispatch_view`
and `kit` modes, with the baseline's method (one process per target, workload
and mode; interleaved; one warm-up pass, then 3 timed passes), on the Glowcap
replay, the agent ledger and Trail Rescue. A target without the new entry
points, such as main, skips the new modes. `view-path-smoke` runs the same on
short streams to show it runs; like `smoke`, it runs one repeat by default and
on any power plan, and its figures are not measurements. A suite is a smoke
suite when it says so (`smoke: true` in `lib/suites.mjs`), no longer by the
name `smoke`; the baseline's suites behave as before.

- `size.mjs` measures, for the base and the branch, the WebAssembly and glue
  of both builds (raw and gzipped), the kit's session library and the kit
  tarball `npm pack` makes.
- `report.mjs` renders the tables from a `view-path` run and a size report:
  each path's DIRECT per-event figures for the owner's event classes (Glowcap
  idle and state-changing ticks; Trail Rescue evidence, commit, reopen and
  qualify; ledger evidence; refusals), the saving paired by repeat, the
  existing paths on main against the branch, and the sizes.

## Running it

```sh
# The measurement: this branch and main, interleaved, 3 repeats, samples kept
node experiments/performance-0.1/run.mjs --suite=view-path --engines=wasm --keep-samples \
  --target=opt=tree:. --target=main=tree:PATH/TO/checkout-at-768275b \
  --out=experiments/performance-opt-0.1/results/view-path
# Per event class (writes classes.json next to results.json)
node experiments/performance-0.1/analyze.mjs experiments/performance-opt-0.1/results/view-path
# Size: each checkout with its `npm run build` output
node experiments/performance-opt-0.1/size.mjs --base=PATH/TO/checkout-at-768275b,PATH/TO/its/dist \
  --branch=.,dist --out=experiments/performance-opt-0.1/results/size.json
# RESULTS.md's tables
node experiments/performance-opt-0.1/report.mjs --run=experiments/performance-opt-0.1/results/view-path \
  --size=experiments/performance-opt-0.1/results/size.json
```
