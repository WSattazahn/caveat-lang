# Caveat performance baseline: 2026-09-29T09-11-08-942-baseline

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=baseline --repeats=1 --build=never --engines=wasm --workloads=glowcap-replay --modes=published-method,adapter --target=main-local=tree:C:/Users/walte/AppData/Local/Temp/claude/C--Dev-GPT-SandBox-Web/3bc468a2-40f8-4895-bd0a-fb452fae7549/scratchpad/prPERF --monitor-interval=1 --out=C:/Users/walte/AppData/Local/Temp/claude/C--Dev-GPT-SandBox-Web/3bc468a2-40f8-4895-bd0a-fb452fae7549/scratchpad/audit/method/alone-1`

## Method

- Suite `baseline`: 1 repeat(s) of every mode, each in a fresh process; within a repeat every target runs a mode back to back, the first target rotating.
- Per-event modes: 1 untimed warm-up pass(es), then 3 timed pass(es), each episode on a freshly opened session. `published-method` runs 3 round(s) with no warm-up, as the published harness did.
- Load, save and restore: 3 untimed then 30 timed calls; published resume method: 3 run(s).
- Statistic: quantile q(p) = sorted[floor(p·(n−1))] over the pooled samples of a run (the published rule; the median is the lower median). A cell is the median over runs of each run's pooled median, with the lowest and highest run in brackets, then the median over runs of each run's p95.
- Units: microseconds per operation unless the operation says otherwise.

## Targets

| Target | Kind | Revision | Reactive WebAssembly sha256 | Glue sha256 | Kit session.mjs sha256 | Native benchmark |
| --- | --- | --- | --- | --- | --- | --- |
| main-local | tree | v0.1.0-rc.4-7-gd9c9358-dirty (d9c9358), tracked changes | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | - |

## Environment

- CPU: Intel(R) Core(TM) Ultra 9 275HX; 24 cores, 24 logical processors; performance cores 0, 1, 10, 11, 12, 13, 22, 23
- Memory: 63.4 GiB
- OS: Microsoft Windows 11 Home 10.0.26200 (build 26200)
- Power: Power Scheme GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c  (High performance); battery status 2 (2 = on mains power)
- Pinning: affinity 0x3C00 (logical processors 10, 11, 12, 13), priority high, via cmd /c start /b /wait /<priority> /affinity <mask>
- Node v24.11.1, V8 13.6.233.10-node.28, npm 11.12.1
- Rust: rustc 1.98.1 (48a229cea 2026-09-01) (LLVM version: 22.1.8); cargo 1.98.1 (797e8a9bc 2026-08-05); 1.98.1-x86_64-pc-windows-msvc (overridden by 'C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF\rust-toolchain.toml')
- wasm-bindgen 0.2.104; wasm-opt not installed (the build does not use it)
- Load before: total CPU 5/0/0%; busiest over the window: claude 1.359s, powershell 0.641s, claude 0.453s, ChatGPT 0.172s, ChatGPT 0.125s, IntelGraphicsSoftware 0.125s
- Load after: total CPU 2/5/4%; busiest over the window: claude 1.672s, powershell 1.188s, powershell 1s, claude 0.328s, ChatGPT 0.234s, IntelGraphicsSoftware 0.141s
- Load during: typeperf every 1 s for the whole run, 3 samples: total mean 9.5% p95 8.6% max 12.3%; processor 10 mean 17.6% p95 14.1% max 25.2%; processor 11 mean 15.5% p95 14.3% max 18.1%; processor 12 mean 16.1% p95 12.6% max 23.7%; processor 13 mean 98.4% p95 98.4% max 100%; processor 10 MHz mean 4187.2 MHz p95 4122.6 MHz max 4341.3 MHz; processor 11 MHz mean 3989 MHz p95 4003.5 MHz max 4122.1 MHz; processor 12 MHz mean 4193.2 MHz p95 4234 MHz max 4246.1 MHz; processor 13 MHz mean 3783.6 MHz p95 3760.1 MHz max 3847.7 MHz; total above 10% in 1 and above 25% in 0 samples; pinned processors summing above 150% in 1

## Published reference

- Dispatch + view, Glowcap replay: median 51.6 µs, p95 59.1 µs, max 646.7 µs (experiments/glowcap/evidence/round6-replay.json timing.dispatchAndShippedOutput); runtime 8d8596a, reactive WebAssembly `d897787f414a`, repository e6ace96.
- Resume ten minutes of play: median 3.2416 ms of runs 7.154, 3.2416, 3.1401 ms; save 2045 bytes (experiments/glowcap/evidence/round6-replay.json timing.resume (caveat5)).

## Final-state consistency

Within each target, every run of every mode and engine ended each workload in the same saved state (2 runs compared).

Between targets, by workload and kind of state (targets in one group ended in identical states):

- glowcap-replay adapter-save-json: main-local `bcc2ec60c6f5`

## One Glowcap event + view, piece by piece

Medians over runs of each run's median on the steady idle ticks (events 1300–9999), where the pooled median sits; each row is its own process, so parts need not sum exactly to the whole.

| Piece | main-local |
| --- | --- |
| Published method: adapter dispatch + view | 47.2 |
| Adapter dispatch (stringify, WebAssembly dispatch_view, JSON.parse) | 43.7 |
| Adapter view() (JavaScript reshaping only) | 2.2 |
| JSON.stringify(payload) | - |
| wasm-bindgen dispatch_view call, returning the view text | - |
|   arguments copied into WebAssembly memory | - |
|   WebAssembly execution (resolve, apply, view, serialize) | - |
|   result decoded to a JavaScript string | - |
|   result freed | - |
|   the four pieces, timed together in one process | - |
| JSON.parse(view text) | - |
| WebAssembly view() alone, execution only | - |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | - |
| Kit dispatch + view() | - |
| Native web::dispatch_view (the exported function, natively) | - |
| Native apply (numeric parameters: rules and bindings only) | - |
| Native dispatch_view_json (resolve + apply + view, not serialized) | - |
| Native view() build | - |
| Native view serialize | - |
| Native snapshot() build | - |
| Native session clone (upper bound of the transaction copy) | - |
| Native apply, the same program without bindings (glowcap-unbound) | - |
| *derived:* native resolve_payload ≈ dispatch_view_json − apply − view | - |
| *derived:* native binding evaluation ≈ apply − apply without bindings | - |
| *derived:* WebAssembly ÷ native, same exported function (ratio) | - |

## glowcap-replay

The Glowcap replay program and the exact event stream behind the published 51.6 us figure (experiments/glowcap/harness.mjs bench(), runtime/examples/profile_dispatch.rs): absorb cave glowcap, absorb pool duskcap, taste ruin glowcap, then 9,997 ticks of dt 0.05. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | main-local |
| --- | --- | --- | --- |
| wasm | published-method | adapter.dispatch+view | 47.4 · p95 60.5 |
| wasm | adapter | adapter.dispatch | 43.7 · p95 50.8 |
| wasm | adapter | adapter.view | 2.2 · p95 3.4 |
| wasm | adapter | adapter.dispatch+view | 46.0 · p95 53.8 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| wasm | published-method | adapter.dispatch+view | main-local | 177.9 | 53.8 | 48.3 | 47.2 |
| wasm | adapter | adapter.dispatch | main-local | 173.0 | 45.9 | 43.5 | 43.7 |
| wasm | adapter | adapter.view | main-local | 29.9 | 2.5 | 2.3 | 2.2 |
| wasm | adapter | adapter.dispatch+view | main-local | 207.8 | 48.5 | 45.8 | 46.0 |

