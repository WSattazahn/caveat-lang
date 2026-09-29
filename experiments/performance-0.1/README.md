# Performance baseline 0.1

A measurement-only harness for the reactive runtime. It times where an
event's cost goes, on the runtime as it is. It changes nothing: no file under
`runtime/`, `kit/` or anywhere else is modified, no crate is added, and the
runtime's `Cargo.lock` stays as it is. The published Glowcap figure it starts
from is "dispatch + view, 51.6 / 59.1 µs median / p95"
([RESULTS.md](../glowcap/RESULTS.md), round 6 replay). That figure came from a
development runtime from before rc.3, 8d8596a, so this harness measures each
runtime separately rather than assuming it.

## Running it

It needs Node 20 or later, and for a `tree` target also the toolchain
`scripts/build-web.mjs` needs: rustup with `rust-toolchain.toml`'s compiler and
wasm32 target, and wasm-bindgen 0.2.104. Run it from a checkout of this branch:

```sh
# Prove it runs, end to end, on every target (short streams, one round; not measurements)
node experiments/performance-0.1/run.mjs --suite=smoke \
  --target=rc4-published=package:PATH/TO/caveat-lang-0.1.0-rc.4/package \
  --target=rc4-local=tree:PATH/TO/checkout-at-v0.1.0-rc.4 \
  --target=main-local=tree:.

# The baseline: every mode, 3 repeats, 3 timed passes each after a warm-up pass
node experiments/performance-0.1/run.mjs \
  --target=rc4-published=package:PATH/TO/package \
  --target=rc4-local=tree:PATH/TO/checkout-at-v0.1.0-rc.4 \
  --target=main-local=tree:. \
  --out=experiments/performance-0.1/results/baseline

# Re-render summary.md from a results directory
node experiments/performance-0.1/summarize.mjs experiments/performance-0.1/results/baseline

# Label every event of every workload with its class (before timing; untimed)
node experiments/performance-0.1/classify.mjs --runtime=PATH/TO/package/runtime --runtime=PATH/TO/pkg-reactive

# Per-class statistics of a run made with --keep-samples (writes classes.json)
node experiments/performance-0.1/analyze.mjs RESULTS_DIR

# The published command itself, in trees built with `npm run build`, interleaved
node experiments/performance-0.1/verbatim.mjs --tree=main=. --tree=rc4=PATH/TO/rc4 --repeats=3 --out=verbatim.json

# The same, and any run.mjs job, under each of several processor masks in turn
# (the masks rotate with the targets; a run.mjs target becomes LABEL@MASK)
node experiments/performance-0.1/verbatim.mjs --tree=main=. --affinity-set=0x3,0xC00,0x3000,0xC00000,0x3C00 --priority=high --out=cores.json
node experiments/performance-0.1/run.mjs --target=main-local=tree:. --build=never --workloads=glowcap-replay \
  --modes=published-method,web.dispatch_view --affinity-set=0x3,0xC00,0x3000,0xC00000,0x3C00 --monitor-interval=1 --out=DIR

# RESULTS.md's tables and derived numbers from the raw results (reads .json or .json.gz);
# --replicate (repeatable) adds other sessions, --method a paired in-process/alone test;
# repeated FILE options are read in the order given (it sets their row order): RESULTS.md
# section 16 has the exact argument list and order that reproduce the committed tables
node experiments/performance-0.1/attribute.mjs --baseline=DIR --instrumented=DIR --verbatim=FILE \
  [--replicate=DIR ...] [--method=DIR] [--cores=DIR --cores-verbatim=FILE ...] [--copies=DIR] \
  [--allocbench=FILE] --out=DIR

# The clock each performance core reaches under a pinned single-threaded busy loop
node experiments/performance-0.1/clocks.mjs --out=clocks.json

# The harness's own tests (statistics, inputs and hashes, the adapter rewrite, event classes)
node --test experiments/performance-0.1/harness.test.mjs
```

The measured baseline, its method and its findings are in
[RESULTS.md](RESULTS.md); the raw JSON behind it is in `results/`.

A target is `LABEL=KIND:DIR`:

| Kind | DIR | Native | WebAssembly | Kit |
| --- | --- | --- | --- | --- |
| `tree` | a caveat-lang checkout, such as a worktree at `v0.1.0-rc.4` | built against `DIR/runtime` | built from `DIR` | `DIR/kit/lib` |
| `package` | an unpacked `caveat-lang` npm package | — | `DIR/runtime`: the published bytes | `DIR/lib` |
| `runtime` | a directory with `caveat_runtime.js` and `caveat_runtime_bg.wasm`, for example an older build | — | `DIR` | — |

For a `tree` everything built goes under `DIR/runtime/target/`, which git
ignores. Nothing tracked changes, so a read-only checkout of a tag works.

- **WebAssembly.** This is the reactive-only build with exactly the cargo
  arguments, target directory and `--remap-path-prefix` flags of
  `scripts/build-web.mjs`, bound into `runtime/target/perf-baseline/pkg-reactive/`.
  On this machine a local build of `v0.1.0-rc.4` and of main b2f424c are
  byte-identical (`e35944503272…`). **A Windows build is not byte-identical to the
  published Linux build:** the published `698a0d0f…` is 513 bytes larger,
  because it holds Linux panic-location paths. Measure the `package` target
  for the published bytes, and the `tree` targets to compare like for like.
- **Native.** This is a generated crate in `runtime/target/perf-baseline/native/`.
  It depends on `DIR/runtime` by path, with default features off, as the
  WebAssembly is built. It compiles [native/perf_baseline.rs](native/perf_baseline.rs)
  with the runtime's own `[profile.release]` (`lto = true`,
  `codegen-units = 1`). The build starts from the runtime's `Cargo.lock` and
  runs `--offline`. The harness then checks that every locked package kept its
  version and checksum.
- `--build=never` reuses what was built. `--build=always` rebuilds the
  WebAssembly as well. The native benchmark is rebuilt unless `--build=never`,
  because its source lives here, not in the target.

The **inputs always come from this checkout**, never from the target, so every
target runs the same bytes. For a `tree`, the target's own
`experiments/glowcap/caveat5/adapter.mjs` is used. For the other kinds this
checkout's adapter is used. Its hash is recorded in both cases.

## Protocol

- **One process per (target, workload, mode).** Nothing a mode leaves behind,
  such as a grown WebAssembly heap or warm JIT state, reaches another mode.
- **Interleaved.** Within a repeat, every target runs one mode back to back
  before the next mode starts. The target that goes first rotates each
  repeat, so drift over a run affects every target alike.
- **Pinned.** Each child starts with `start /b /wait /high /affinity MASK` on
  Windows, or `taskset` on Linux. By default the mask is four
  performance cores: the highest `EfficiencyClass` from
  `GetSystemCpuSetInformation`, leaving logical processors 0 and 1 alone. On
  the Core Ultra 9 275HX that is processors 10–13, `0x3C00`. Override it with
  `--affinity=0xMASK|none` and `--priority=high|abovenormal|normal`.
  **The performance cores are not interchangeable.** On this laptop the same
  bytes take about a quarter less time on processors 0–1 than on 10–13, and
  6–12% more on 22–23 (RESULTS.md, section 7), so an absolute figure
  holds only for the mask it was measured on, and two runs compare only if
  they ran on the same mask. `--affinity-set=0xA,0xB,...` runs every job
  under each mask in turn, as the target `LABEL@MASK`, the masks rotating
  with the targets. Each child records what it actually got. Node records
  `os.availableParallelism()`, which honours the mask. The native benchmark
  records `GetProcessAffinityMask`, its priority class and, once per timed
  event outside the timed region, the processor it is on
  (`GetCurrentProcessorNumber`, `extra.processorsSeen`). `results.json`
  records every logical processor's CPU set (core, efficiency class,
  scheduling class, cache) in `environment.machine.cpuSetList`, and each run
  the mask it ran under and, from the monitor, which of the monitored
  processors it kept busy and at what clock (`cores`).
- **Warm-up.** Each per-event mode plays every episode once untimed, then
  `rounds` times timed, each episode on a freshly opened session. Opening and
  closing sessions are not timed. Node runs with `--expose-gc` and collects
  garbage before each pass. `published-method` is the exception: it runs the
  published loop verbatim, with 3 rounds, no warm-up and no forced collection.
- **Quiet machine.** Before and after the run, `run.log` and `summary.md`
  record total CPU use and the busiest processes, and for the whole run
  `typeperf` samples total CPU use and each pinned processor's use and actual
  clock (MHz) every 5 s (`--monitor-interval=N` changes it)
  (`load.csv`, summarized as `environment.loadDuring`; `--no-monitor` turns
  it off). The harness does not stop anything: close heavy programs first.
  A session whose recorded load is above the quiet level, or which overlapped
  other jobs, is discarded, not kept as a replicate. A session with no load
  record during the run is rerun (RESULTS.md, section 2;
  [results/excluded/](results/excluded/README.md)).
- **Power.** On Windows a measurement (any suite but `smoke`) stops before
  timing unless the machine is on mains power with the High performance plan
  (`--allow-any-power` overrides).
- **Timers.** Native uses `std::time::Instant`. Node uses
  `process.hrtime.bigint()`, as the published harness did, and
  `performance.now()` in the published resume method. On Windows both read
  QueryPerformanceCounter, which ticks every 100 ns. Each result records the
  timer's own overhead.

## Statistics

For each operation, a run pools every timed sample: every event of every
timed pass. From that pool it takes the quantile q(p) = sorted[floor(p·(n−1))].
This is the rule `experiments/glowcap/harness.mjs` published with, so the
median of an even count is the lower median. `results.json` holds, per run,
the pooled n, median, p95, p99, p25, p75, IQR, min, max and mean. It also
holds each pass's own summary, and the median of every named segment of the
stream. Across runs (repeats), it gives the median, lowest and highest of each
statistic, and the range as a percentage of the median. `summary.md` shows the
across-run median of the pooled medians, with the run range in brackets, and
then the across-run median p95.

The Glowcap replay has four segments, because the pooled median is really the
steady segment:

| Segment | Events | Why |
| --- | --- | --- |
| observations | 0–2 | the absorb, absorb and taste events |
| decay | 3–602 | glow (30 s) and heavy (20 s) count down |
| transition | 603–1299 | idle ticks, regrowth at 45 s and the taste's fade at 60 s |
| steady | 1300–9999 | idle ticks: 87% of the stream |

Every per-event mode of every engine must end each workload in the same saved
state within a target. The harness hashes each episode's final `save()` text
and compares the hashes. A difference fails the run as a correctness finding.
Between targets the states are compared and reported; runtimes of different
versions may differ.

## Workloads

[`inputs/workloads.json`](inputs/workloads.json) lists every program and
stream with its sha256. The harness refuses inputs whose hash has changed.

| Workload | What | Events per pass |
| --- | --- | --- |
| `glowcap-replay` | `experiments/glowcap/caveat5/glowcap.cav` and the published benchmark's stream: 3 observations, then 9,997 ticks of dt 0.05 | 10,000 |
| `glowcap-resume` | the published resume stream: an absorb, then 9,600 ticks of dt 0.0625; used for save and restore | 9,601 |
| `glowcap-unbound` | synthetic: the replay program with its 39 one-line `bind` statements removed, on the replay's stream. The rules, states and transaction are the same, with no bindings to evaluate. | 10,000 |
| `glowcap-scaled-16`, `-64` | synthetic: Glowcap with 12 or 60 more mushrooms (so more symbols, edges, rules and bindings); the replay's first 2,000 events | 2,000 |
| `ledger-session` | `experiments/agent-ledger/ledger.cav` on the real 2026-09-24 session log (5 of 20 events refused), 100 times | 2,000 |
| `trail-rescue-scenarios` | `game/trail_rescue.cav` on the runtime events of the 24 registered scenarios, 20 times | 2,800 |

[`inputs/trail-rescue-episodes.json`](inputs/trail-rescue-episodes.json) is
generated by [`lib/record-trail-rescue.mjs`](lib/record-trail-rescue.mjs).

The ledger and Trail Rescue are the decision workloads: they run every mode
the Glowcap replay runs except the three that go through the Glowcap adapter.

## Event classes

[`inputs/event-classes.json`](inputs/event-classes.json) labels every event of
every workload, written by [`classify.mjs`](classify.mjs) before any timing
from what each event did, never from how long it took: its dispatch outcome,
the effects the runtime reported, and which parts of the save it changed.
[`lib/classes.mjs`](lib/classes.mjs) holds the rules, tried in order:
`refused`, `commit+reopen`, `commit`, `reopen`, `evidence` (reveal, sample,
withdraw, examine), `qualify`, `renew`, `state-changing` (no effect, but the
save changed outside the sequence, clock and effects) and `idle`. Every label
also has a signature (event, class and effect kinds, such as
`advance reopen [qualify,reopen]`). The published runtime and both local
builds label every event identically. For Glowcap streams
[`analyze.mjs`](analyze.mjs) also groups events into the owner's categories:
idle tick, state-changing tick, and observation (with its class). Per pass:

| Workload | Classes per pass |
| --- | --- |
| `glowcap-replay`, `-unbound` | idle 9,394; state-changing 601; renew 1; qualify 1; commit 1; reopen 1; evidence 1 |
| `glowcap-resume` | idle 9,118; state-changing 481; renew 1; commit 1 |
| `glowcap-scaled-16`, `-64` | idle 1,394; state-changing 601; renew 1; qualify 1; commit 1; reopen 1; evidence 1 |
| `ledger-session` | evidence 1,500 (400 of them renew + reveal); refused 500 |
| `trail-rescue-scenarios` | evidence 880; commit 480; state-changing 440; refused 440; reopen 200; qualify 120; idle 240 |

A class with fewer than about 100 samples a run is reported with its count and
marked unreliable; the workloads are not changed to fill it.
That script sends the registered scenarios through the page's own policy,
`web/trail-rescue-policy.js`, and records what reaches the runtime.

## What each mode times

Every mode runs each workload's events in order. The operations below are the
names in `results.json` and `summary.md`, in microseconds per call.

Native: `perf_baseline`, with the runtime's public API only.

| Mode | Operations | What |
| --- | --- | --- |
| `apply` | `apply` | `ReactiveSession::apply` with numeric parameters: the transaction (parameter checks, session copy, rules, change detection, binding evaluation). No payload JSON, no name resolution, no view or snapshot. Names become positions untimed, as the runtime resolves them; the final-state check proves the mapping. |
| `dispatch_view_json` | `dispatch_view_json` | payload parse + name resolution + `apply` + building the view, not serialized |
| `dispatch_json` | `dispatch_json` | … + building the full snapshot (the legacy dispatch) |
| `dispatch_outcome_json` | `dispatch_outcome_json` | … + the snapshot inside a dispatch outcome (the kit's call) |
| `web.dispatch_view`, `web.dispatch_outcome`, `web.dispatch` | same | the exact functions the WebAssembly exports (`web::WebReactiveSession`), natively, serialization and drops included |
| `read` | `view`, `view.serialize`, `snapshot`, `snapshot.serialize`, `snapshot.serialize_pretty`, `snapshot.drop`, `save`, `save_json`, `clone`, `clone.drop` | read-only calls on each event's resulting state, after an untimed dispatch. Builds are timed without their drop. `clone` is a whole-session clone: it includes the bindings that `apply` moves rather than copies, so it is an upper bound on the transaction's copy. |
| `web.read` | `web.view`, `web.snapshot`, `web.save` | the exported read calls natively |
| `lifecycle` | `from_source`, `web.new`, `save`, `save_json`, `web.save`, `restore.parse`, `restore`, `restore_json`, `web.restore` | at the first episode's final state; `restore` is `from_source` plus applying the save, and `restore.parse` is the JSON parse alone |

WebAssembly: `wasm-bench.mjs` under Node.

| Mode | Operations | What |
| --- | --- | --- |
| `published-method` | `adapter.dispatch+view` | `experiments/glowcap/harness.mjs` `bench()` for caveat5 alone, verbatim: `policy.dispatch(event); policy.view();` through the real adapter. The adapter is rewritten only in its three file locations ([lib/adapter.mjs](lib/adapter.mjs)). |
| `adapter` | `adapter.dispatch`, `adapter.view`, `adapter.dispatch+view` | the same loop, warmed up, split at the call boundary |
| `raw.dispatch_view` | `js.stringify_payload`, `raw.dispatch_view`, `js.parse_view`, `raw.dispatch_view+parse` | the wasm-bindgen class as the web pages use it |
| `raw.dispatch_outcome`, `raw.dispatch` | `raw.dispatch_outcome`, `js.parse_outcome`, …; `raw.dispatch` | the outcome with its compact snapshot; the legacy pretty snapshot |
| `abi.dispatch_view`, `abi.dispatch_outcome` | `abi.encode_args`, `abi.exec`, `abi.decode`, `abi.free`, and the four together | the same exports called without the glue. `abi.encode_args` mirrors `passStringToWasm0`. `abi.exec` is the WebAssembly call alone. `abi.decode` is the glue's fatal `TextDecoder`. `abi.free` is `__wbindgen_free`. |
| `read`, `abi.read` | `raw.view`, `js.parse_view`, `raw.snapshot`, `js.parse_snapshot`, `raw.save`; `abi.{view,snapshot,save}.{exec,decode}` | read-only calls after an untimed dispatch |
| `kit`, `kit.read` | `kit.dispatch`, `kit.view`, `kit.dispatch+view`; `kit.view`, `kit.snapshot`, `kit.save` | the developer kit's `CaveatSession`. Dispatch is `dispatch_outcome`, with its full snapshot parsed. A host that wants the view also calls `view()`. |
| `lifecycle` | `raw.new`, `raw.save`, `raw.restore`, `kit.open`, `kit.save`, `kit.restore` | at the first episode's final state. Each restored session must save to the text it came from. |
| `adapter-resume` | `adapter.resume.parse`, `.createPolicy`, `.view`, `adapter.resume` | `experiments/glowcap/resume-bench.mjs` for caveat5. Views and continued play are checked outside the timed region. |
| `micro` | `js.stringify_payload.*`, `kit.payloadText.last`, `bridge.pass_ascii.*`, `bridge.decode.*`, `js.parse.*`, `timer.hrtime_pair` | synthetic isolation of single costs, on the texts the final state produces: batches of calls on fixed inputs |

Where the task's questions land:

| Question | Where to read it |
| --- | --- |
| (a) core event dispatch, no view or snapshot | native `apply` on `glowcap-unbound`: the transaction and rules, with no bindings; `dispatch_view_json` minus `apply` minus `view` adds payload parsing and name resolution |
| (b) event + reactive binding evaluation | native `apply` on `glowcap-replay`; binding evaluation is the difference from `glowcap-unbound` (the summary derives it). Its inner parts (change detection, stale groups, citations) need throwaway instrumentation (below). |
| (c) `view()` alone | native `read` `view` + `view.serialize`, `web.read` `web.view`; WebAssembly `abi.read` `abi.view.exec`, `read` `raw.view` |
| (d) `snapshot()` alone | native `read` `snapshot`, `snapshot.serialize(_pretty)`; WebAssembly `abi.read` `abi.snapshot.exec`, `read` `raw.snapshot` |
| (e) event + view, the published 51.6 µs | `published-method`, decomposed by `adapter`, `raw.dispatch_view`, `abi.dispatch_view`; see the summary's "piece by piece" table |
| (f) save, (g) restore | `lifecycle` on `glowcap-resume` (and on every workload), `adapter-resume` |
| (h) native against WebAssembly | `web.*` natively against `abi.*` (execution) and `raw.*` (with the bridge), same exported function. The runtime sets no `#[global_allocator]`: natively that is the system allocator (on Windows the process heap), in WebAssembly Rust's bundled dlmalloc, so a ratio compares that pairing, not code generation alone |
| snapshot after an accepted dispatch | `dispatch_outcome_json` / `web.dispatch_outcome` / `kit.dispatch` against `dispatch_view_json` / `web.dispatch_view` |
| JSON and bridge overhead | `abi.encode_args`, `abi.decode`, `abi.free`, `js.parse_*`, `micro` |
| NodeId→name map, symbol sort, commitment and relation records | grow with symbols and edges: compare `view` across `glowcap-replay`, `-scaled-16`, `-scaled-64`; exact attribution needs instrumentation |
| provenance and state cloning per event | `clone` (upper bound) against `apply`; the copies themselves need instrumentation (RESULTS.md, i3) |
| core placement | `--affinity-set` for `run.mjs` and `verbatim.mjs`; `attribute.mjs --cores` |

## Throwaway instrumentation

The public API gives binding evaluation only as a whole, by the
`glowcap-unbound` difference. It cannot split `apply` into the session copy,
the rules, change detection and binding evaluation. Nor can it split `view()`
into its name map, its symbol sort and its record building. Timing those pieces needs timers inside
`runtime/src/reactive.rs`. That is allowed only in a throwaway copy of a tree,
outside this repository. It is never committed, and every report that uses it
says exactly what was instrumented. Such a copy is an ordinary `tree` target,
so this harness measures it unchanged; the modes that call the copy's added
functions live only in the copy's own harness. RESULTS.md describes the three
copies used for the 0.1 baseline (i1: benchmark-only entry points; i2:
timestamp marks; i3: timers around the copy-on-write copies and a
provenance-copy counter) and compares each against the ordinary build.
i1 directly times the isolated operations in the instrumented build, but
adding its entry points changes ordinary-path timing by up to about 7%, so
its absolute timings are attribution evidence (proportions and ordering),
not production-path costs: `attribute.mjs` labels every row timed in or
computed from i1 "i1: attribution only". i3's copy timers change
ordinary-path timing too (−2% to +8% in WebAssembly and 3–11% natively on
accepted events), so its copy timings are likewise attribution evidence and
the report ranks copying by i3's shares. i2 is quoted only as shares, or
converted to µs as INFERRED against the ordinary build. Production costs come
from the ordinary build's DIRECT paths.

## Output

`--out` (default `results/<timestamp>-<suite>/`) receives:

- `results.json`: the environment, targets with hashes, inputs with hashes,
  the method, every run's summaries and the across-run spread;
- `summary.md`: the same as tables;
- `run.log`: what ran, in order, with each process's pinning;
- `load.csv`: the typeperf samples taken during the run;
- `samples/*.json.gz`, only with `--keep-samples`: every raw sample.

`analyze.mjs` adds `classes.json` (per event class) to a run kept with
`--keep-samples`. The committed baseline in [`results/`](results/) stores
`results.json` and `classes.json` gzipped; every script here reads either
form.
