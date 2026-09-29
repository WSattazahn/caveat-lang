# Performance Optimization 0.1: the view path

`session.dispatchView(event, payload)` in the kit, and the runtime's
`dispatch_view_outcome` it calls, give a host the dispatch outcome with the
view after the event, and never build the snapshot
([Dispatch 0.1](../../spec/caveat-dispatch-0.1.md#the-view-path)). The
existing `session.dispatch()`, `dispatch_outcome`, `dispatch_view` and
`dispatch` are unchanged.

- [RESULTS.md](RESULTS.md): the measurement. The paths side by side per
  event class, the measured saving, what remains in the kit's wrapper, the
  size impact, the correctness evidence and the load of every session; the
  raw results are in [results/](results/).
- [correctness/](correctness/README.md): the evidence that the new path
  returns what the existing ones do and that they are unchanged.

[Performance Baseline 0.1](../performance-0.1/RESULTS.md) is frozen evidence:
nothing under `experiments/performance-0.1/results/` or its RESULTS.md
changed. Its harness gained three modes, two suites and the job guard
(`--monitor-lead`, `--job-guard`, off unless given), and nothing it measured
before changed:

| Mode | Times, per event |
| --- | --- |
| `kit.dispatchView` | `session.dispatchView(event, payload)`: the kit's payload check and encoding, the call, and `JSON.parse` of the outcome |
| `raw.dispatch_view_outcome` | as `raw.dispatch_view`: `JSON.stringify` of the payload, `WebReactiveSession.dispatch_view_outcome`, and `JSON.parse` of its text |
| `kit.dispatchView.pieces` | `dispatchView`'s pieces one by one, in its order, on a raw session of the same runtime: the kit's own `payloadText`, the `dispatch_view_outcome` call, `JSON.parse` of the envelope and the kit's own `validOutcome(outcome, 'view')` (imported from a copy of the target's `lib/session.mjs` with one added export line; both hashes recorded), their sum from the first timer to the last, and, for an accepted event, `JSON.parse` of the view text alone, cut from the envelope untimed |

The `view-path` suite runs those three and the baseline's `raw.dispatch_view`
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
- `load.mjs` gives the load of every timed job and every repeat of a run,
  from its `load.csv` and the job times in `results.json`, and applies the
  keep rules: every timed job keeps the job guard's rule
  (`../performance-0.1/lib/guard.mjs`, which `run.mjs --job-guard` applied as
  each job ended, running again any job that broke it), and a repeat is kept
  when, over the samples covering its timed jobs, the mean total CPU is at
  most 12% and no one-second sample is above 20%. A session is used only when
  every job and every repeat is kept.
- `report.mjs` renders the tables from a `view-path` run and a size report:
  each path's DIRECT per-event figures for the owner's event classes (Glowcap
  idle and state-changing ticks; Trail Rescue evidence, commit, reopen and
  qualify; ledger evidence) with the saving paired by repeat; the existing
  paths on main against the branch, the wrapper's pieces, the load per
  repeat and per timed job with every attempt the job guard discarded, and
  the sessions given with `--replicate` side by side (accepted events);
  then, after every accepted-event result and out of the headline, the
  refusals in their own tables (this run's, then every session's); and the
  sizes.

## Running it

```sh
# A session: this branch and main, interleaved, samples kept (RESULTS.md ran
# 5 repeats on processors 10-13 twice, and 3 on 0-1 with --affinity=0x3)
node experiments/performance-0.1/run.mjs --suite=view-path --engines=wasm --keep-samples \
  --repeats=5 --monitor-interval=1 --monitor-lead=5 --job-guard --affinity=0x3C00 --priority=high \
  --target=opt=tree:. --target=main=tree:PATH/TO/checkout-at-768275b --out=DIR
# Per event class (writes classes.json next to results.json), then the keep rule
node experiments/performance-0.1/analyze.mjs DIR
node experiments/performance-opt-0.1/load.mjs DIR
# Size: each checkout with its `npm run build` output (RESULTS.md: main 334d1b7,
# and this branch merged into it)
node experiments/performance-opt-0.1/size.mjs --base=PATH/TO/base-checkout,PATH/TO/its/dist \
  --branch=.,dist --out=experiments/performance-opt-0.1/results/size.json
# RESULTS.md's tables
node experiments/performance-opt-0.1/report.mjs --run=DIR \
  --size=experiments/performance-opt-0.1/results/size.json [--replicate=NAME=DIR ...] [--out=tables.md]
```
