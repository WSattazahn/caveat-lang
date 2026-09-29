# Caveat performance baseline: 2026-09-29T08-08-32-439-copies

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=copies --build=never --keep-samples --target=main-local=tree:C:/Users/walte/AppData/Local/Temp/claude/C--Dev-GPT-SandBox-Web/3bc468a2-40f8-4895-bd0a-fb452fae7549/scratchpad/prPERF --target=i3=tree:C:/Users/walte/AppData/Local/Temp/claude/C--Dev-GPT-SandBox-Web/3bc468a2-40f8-4895-bd0a-fb452fae7549/scratchpad/perf-scratch/m2/i3 --out=C:/Users/walte/AppData/Local/Temp/claude/C--Dev-GPT-SandBox-Web/3bc468a2-40f8-4895-bd0a-fb452fae7549/scratchpad/perf-scratch/m2/copies`

## Method

- Suite `copies`: 3 repeat(s) of every mode, each in a fresh process; within a repeat every target runs a mode back to back, the first target rotating.
- Per-event modes: 1 untimed warm-up pass(es), then 3 timed pass(es), each episode on a freshly opened session. `published-method` runs 3 round(s) with no warm-up, as the published harness did.
- Load, save and restore: 3 untimed then 30 timed calls; published resume method: 3 run(s).
- Statistic: quantile q(p) = sorted[floor(p·(n−1))] over the pooled samples of a run (the published rule; the median is the lower median). A cell is the median over runs of each run's pooled median, with the lowest and highest run in brackets, then the median over runs of each run's p95.
- Units: microseconds per operation unless the operation says otherwise.

## Targets

| Target | Kind | Revision | Reactive WebAssembly sha256 | Glue sha256 | Kit session.mjs sha256 | Native benchmark |
| --- | --- | --- | --- | --- | --- | --- |
| main-local | tree | v0.1.0-rc.4-6-gd132610-dirty (d132610), tracked changes | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |
| i3 | tree | n/a | `d704e1f57e1c` (2054401 B, reused) | `93f93e47fa4b` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |

## Environment

- CPU: Intel(R) Core(TM) Ultra 9 275HX; 24 cores, 24 logical processors; performance cores 0, 1, 10, 11, 12, 13, 22, 23
- Memory: 63.4 GiB
- OS: Microsoft Windows 11 Home 10.0.26200 (build 26200)
- Power: Power Scheme GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c  (High performance); battery status 2 (2 = on mains power)
- Pinning: affinity 0x3C00 (logical processors 10, 11, 12, 13), priority high, via cmd /c start /b /wait /<priority> /affinity <mask>
- Node v24.11.1, V8 13.6.233.10-node.28, npm 11.12.1
- Rust: rustc 1.98.1 (48a229cea 2026-09-01) (LLVM version: 22.1.8); cargo 1.98.1 (797e8a9bc 2026-08-05); 1.98.1-x86_64-pc-windows-msvc (overridden by 'C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF\rust-toolchain.toml')
- wasm-bindgen 0.2.104; wasm-opt not installed (the build does not use it)
- Load before: total CPU 9/5/13%; busiest over the window: claude 1.312s, powershell 0.688s, claude 0.484s, claude 0.281s, ChatGPT 0.203s, OneDrive.Sync.Service 0.078s
- Load after: total CPU 9/0/0%; busiest over the window: claude 1.969s, claude 0.75s, powershell 0.594s, ChatGPT 0.312s, Reallusion Hub 0.219s, IntelGraphicsSoftware 0.203s
- Load during: typeperf every 5 s for the whole run, 79 samples: total mean 8.9% p95 11% max 18%; processor 10 mean 82% p95 99.1% max 99.4%; processor 11 mean 18.3% p95 38.8% max 53.9%; processor 12 mean 19.3% p95 39.7% max 78.1%; processor 13 mean 2.8% p95 8.2% max 14.5%; processor 10 MHz mean 4168.2 MHz p95 4298 MHz max 4326.8 MHz; processor 11 MHz mean 3962.6 MHz p95 4155.7 MHz max 4243.5 MHz; processor 12 MHz mean 3880.9 MHz p95 4042.9 MHz max 4096.7 MHz; processor 13 MHz mean 3275.1 MHz p95 4323.3 MHz max 4501 MHz; total above 10% in 8 and above 25% in 0 samples; pinned processors summing above 150% in 3

## Published reference

- Dispatch + view, Glowcap replay: median 51.6 µs, p95 59.1 µs, max 646.7 µs (experiments/glowcap/evidence/round6-replay.json timing.dispatchAndShippedOutput); runtime 8d8596a, reactive WebAssembly `d897787f414a`, repository e6ace96.
- Resume ten minutes of play: median 3.2416 ms of runs 7.154, 3.2416, 3.1401 ms; save 2045 bytes (experiments/glowcap/evidence/round6-replay.json timing.resume (caveat5)).

## Final-state consistency

Within each target, every run of every mode and engine ended each workload in the same saved state (108 runs compared).

Between targets, by workload and kind of state (targets in one group ended in identical states):

- glowcap-replay save-text: main-local, i3 `fda3d6109816`
- ledger-session save-text: main-local, i3 `895698524f68`
- trail-rescue-scenarios save-text: main-local, i3 `bb72a4e76921`

## One Glowcap event + view, piece by piece

Medians over runs of each run's median on the steady idle ticks (events 1300–9999), where the pooled median sits; each row is its own process, so parts need not sum exactly to the whole.

| Piece | main-local | i3 |
| --- | --- | --- |
| Published method: adapter dispatch + view | - | - |
| Adapter dispatch (stringify, WebAssembly dispatch_view, JSON.parse) | - | - |
| Adapter view() (JavaScript reshaping only) | - | - |
| JSON.stringify(payload) | 0.3 | 0.3 |
| wasm-bindgen dispatch_view call, returning the view text | 24.4 | 24.7 |
|   arguments copied into WebAssembly memory | - | - |
|   WebAssembly execution (resolve, apply, view, serialize) | - | - |
|   result decoded to a JavaScript string | - | - |
|   result freed | - | - |
|   the four pieces, timed together in one process | - | - |
| JSON.parse(view text) | 16.1 | 16.1 |
| WebAssembly view() alone, execution only | - | - |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | - | - |
| Kit dispatch + view() | - | - |
| Native web::dispatch_view (the exported function, natively) | 20.1 | 22.1 |
| Native apply (numeric parameters: rules and bindings only) | - | - |
| Native dispatch_view_json (resolve + apply + view, not serialized) | - | - |
| Native view() build | - | - |
| Native view serialize | - | - |
| Native snapshot() build | - | - |
| Native session clone (upper bound of the transaction copy) | - | - |
| Native apply, the same program without bindings (glowcap-unbound) | - | - |
| *derived:* native resolve_payload ≈ dispatch_view_json − apply − view | - | - |
| *derived:* native binding evaluation ≈ apply − apply without bindings | - | - |
| *derived:* WebAssembly ÷ native, same exported function (ratio) | - | - |

## glowcap-replay

The Glowcap replay program and the exact event stream behind the published 51.6 us figure (experiments/glowcap/harness.mjs bench(), runtime/examples/profile_dispatch.rs): absorb cave glowcap, absorb pool duskcap, taste ruin glowcap, then 9,997 ticks of dt 0.05. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | main-local | i3 |
| --- | --- | --- | --- | --- |
| native | web.dispatch_view | web.dispatch_view | 20.2 (20.0–22.7) · p95 23.9 | 22.2 (21.6–24.7) · p95 26.2 |
| native | web.dispatch_outcome | web.dispatch_outcome | 63.9 (63.5–64.7) · p95 69.4 | 67.7 (67.5–68.5) · p95 72.1 |
| native | copy.dispatch_view | copy.cow | - | 1.1 (1.1–1.1) · p95 1.2 |
| native | copy.dispatch_view | copy.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | copy.cow.graph | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | copy.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | copy.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | copy.cow.qualifications | - | 0.9 (0.9–0.9) · p95 1.0 |
| native | copy.dispatch_view | copy.cow.states | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | copy.dispatch_view | copy.cow.symbols | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | copy.provenance | - | 2.1 (2.1–2.1) · p95 2.9 |
| native | copy.dispatch_view | count.cow | - | 2.0 (2.0–2.0) · p95 2.0 |
| native | copy.dispatch_view | count.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | count.cow.graph | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | count.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | count.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | count.cow.qualifications | - | 1.0 (1.0–1.0) · p95 1.0 |
| native | copy.dispatch_view | count.cow.states | - | 1.0 (1.0–1.0) · p95 1.0 |
| native | copy.dispatch_view | count.cow.symbols | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | count.provenance | - | 74.0 (74.0–74.0) · p95 134.0 |
| native | copy.dispatch_view | count.provenance_in_cow | - | 12.0 (12.0–12.0) · p95 12.0 |
| native | copy.dispatch_view | count.provenance_names | - | 12.0 (12.0–12.0) · p95 12.0 |
| native | copy.dispatch_view | total | - | 21.8 (21.4–21.9) · p95 25.4 |
| native | copy.dispatch_outcome | copy.cow | - | 1.2 (1.1–1.3) · p95 1.3 |
| native | copy.dispatch_outcome | copy.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | copy.cow.graph | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | copy.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | copy.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | copy.cow.qualifications | - | 1.0 (0.9–1.1) · p95 1.1 |
| native | copy.dispatch_outcome | copy.cow.states | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | copy.dispatch_outcome | copy.cow.symbols | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | copy.provenance | - | 8.3 (8.3–9.6) · p95 8.8 |
| native | copy.dispatch_outcome | count.cow | - | 2.0 (2.0–2.0) · p95 2.0 |
| native | copy.dispatch_outcome | count.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | count.cow.graph | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | count.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | count.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | count.cow.qualifications | - | 1.0 (1.0–1.0) · p95 1.0 |
| native | copy.dispatch_outcome | count.cow.states | - | 1.0 (1.0–1.0) · p95 1.0 |
| native | copy.dispatch_outcome | count.cow.symbols | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | count.provenance | - | 205.0 (205.0–205.0) · p95 254.0 |
| native | copy.dispatch_outcome | count.provenance_in_cow | - | 12.0 (12.0–12.0) · p95 12.0 |
| native | copy.dispatch_outcome | count.provenance_names | - | 144.0 (144.0–144.0) · p95 144.0 |
| native | copy.dispatch_outcome | total | - | 67.5 (66.8–77.7) · p95 72.7 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 24.5 (24.3–25.6) · p95 30.1 | 24.8 (24.7–26.0) · p95 30.8 |
| wasm | raw.dispatch_view | js.parse_view | 16.1 (16.0–16.8) · p95 19.5 | 16.1 (16.0–16.6) · p95 19.6 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 40.9 (40.6–42.8) · p95 49.8 | 41.3 (41.2–43.0) · p95 50.1 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 71.8 (70.5–75.6) · p95 89.9 | 73.7 (73.4–76.4) · p95 88.4 |
| wasm | raw.dispatch_outcome | js.parse_outcome | 62.6 (62.3–65.7) · p95 74.7 | 63.8 (61.9–65.1) · p95 74.5 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 134.3 (133.6–141.6) · p95 163.9 | 137.5 (135.3–141.6) · p95 161.8 |
| wasm | copy.dispatch_view | total | - | 41.6 (41.3–42.0) · p95 51.3 |
| wasm | copy.dispatch_view | call | - | 25.3 (25.0–25.5) · p95 31.7 |
| wasm | copy.dispatch_view | copy.cow | - | 0.8 (0.8–0.9) · p95 1.0 |
| wasm | copy.dispatch_view | count.cow | - | 2.0 (2.0–2.0) · p95 2.0 |
| wasm | copy.dispatch_view | count.provenance | - | 74.0 (74.0–74.0) · p95 134.0 |
| wasm | copy.dispatch_view | count.provenance_names | - | 12.0 (12.0–12.0) · p95 12.0 |
| wasm | copy.dispatch_view | count.provenance_in_cow | - | 12.0 (12.0–12.0) · p95 12.0 |
| wasm | copy.dispatch_view | copy.cow.states | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | copy.dispatch_view | count.cow.states | - | 1.0 (1.0–1.0) · p95 1.0 |
| wasm | copy.dispatch_view | copy.cow.graph | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | count.cow.graph | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | copy.cow.symbols | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | count.cow.symbols | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | copy.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | count.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | copy.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | count.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | copy.cow.qualifications | - | 0.7 (0.7–0.7) · p95 0.9 |
| wasm | copy.dispatch_view | count.cow.qualifications | - | 1.0 (1.0–1.0) · p95 1.0 |
| wasm | copy.dispatch_view | copy.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | count.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | total | - | 137.1 (134.4–137.4) · p95 158.4 |
| wasm | copy.dispatch_outcome | call | - | 74.2 (71.8–74.3) · p95 88.8 |
| wasm | copy.dispatch_outcome | copy.cow | - | 1.0 (0.9–1.0) · p95 1.3 |
| wasm | copy.dispatch_outcome | count.cow | - | 2.0 (2.0–2.0) · p95 2.0 |
| wasm | copy.dispatch_outcome | count.provenance | - | 205.0 (205.0–205.0) · p95 254.0 |
| wasm | copy.dispatch_outcome | count.provenance_names | - | 144.0 (144.0–144.0) · p95 144.0 |
| wasm | copy.dispatch_outcome | count.provenance_in_cow | - | 12.0 (12.0–12.0) · p95 12.0 |
| wasm | copy.dispatch_outcome | copy.cow.states | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | copy.dispatch_outcome | count.cow.states | - | 1.0 (1.0–1.0) · p95 1.0 |
| wasm | copy.dispatch_outcome | copy.cow.graph | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | count.cow.graph | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | copy.cow.symbols | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | count.cow.symbols | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | copy.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | count.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | copy.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | count.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | copy.cow.qualifications | - | 0.8 (0.8–0.8) · p95 1.2 |
| wasm | copy.dispatch_outcome | count.cow.qualifications | - | 1.0 (1.0–1.0) · p95 1.0 |
| wasm | copy.dispatch_outcome | copy.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | count.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | web.dispatch_view | web.dispatch_view | main-local | 110.4 (105.3–113.9) | 22.9 (22.6–25.1) | 20.3 (19.7–22.7) | 20.1 (19.9–22.6) |
| native | web.dispatch_view | web.dispatch_view | i3 | 119.8 (117.3–121.6) | 26.3 (25.7–29.3) | 22.6 (21.9–24.9) | 22.1 (21.6–24.6) |
| native | web.dispatch_outcome | web.dispatch_outcome | main-local | 159.4 (151.0–165.7) | 61.7 (61.7–64.4) | 61.5 (61.4–63.8) | 64.1 (63.7–64.7) |
| native | web.dispatch_outcome | web.dispatch_outcome | i3 | 164.7 (161.8–170.9) | 67.6 (67.4–67.7) | 65.6 (65.6–66.8) | 67.8 (67.5–68.6) |
| native | copy.dispatch_view | copy.cow | i3 | 2.4 (2.4–2.6) | 0.6 (0.6–0.7) | 1.1 (1.0–1.1) | 1.1 (1.1–1.1) |
| native | copy.dispatch_view | copy.cow.commitments | i3 | 0.6 (0.6–0.6) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_view | copy.cow.graph | i3 | 1.4 (1.4–1.5) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_view | copy.cow.journal | i3 | 0.4 (0.3–0.4) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_view | copy.cow.other | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_view | copy.cow.qualifications | i3 | 0.3 (0.3–0.3) | 0.5 (0.5–0.5) | 0.9 (0.9–0.9) | 0.9 (0.9–0.9) |
| native | copy.dispatch_view | copy.cow.states | i3 | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | copy.dispatch_view | copy.cow.symbols | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_view | copy.provenance | i3 | 8.4 (7.9–9.1) | 3.0 (3.0–3.1) | 2.1 (2.1–2.2) | 2.1 (2.0–2.1) |
| native | copy.dispatch_view | count.cow | i3 | 5.0 (5.0–5.0) | 2.0 (2.0–2.0) | 2.0 (2.0–2.0) | 2.0 (2.0–2.0) |
| native | copy.dispatch_view | count.cow.commitments | i3 | 1.0 (1.0–1.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_view | count.cow.graph | i3 | 1.0 (1.0–1.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_view | count.cow.journal | i3 | 1.0 (1.0–1.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_view | count.cow.other | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_view | count.cow.qualifications | i3 | 2.0 (2.0–2.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) |
| native | copy.dispatch_view | count.cow.states | i3 | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) |
| native | copy.dispatch_view | count.cow.symbols | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_view | count.provenance | i3 | 278.0 (278.0–278.0) | 142.0 (142.0–142.0) | 74.0 (74.0–74.0) | 74.0 (74.0–74.0) |
| native | copy.dispatch_view | count.provenance_in_cow | i3 | 1.0 (1.0–1.0) | 6.0 (6.0–6.0) | 12.0 (12.0–12.0) | 12.0 (12.0–12.0) |
| native | copy.dispatch_view | count.provenance_names | i3 | 89.0 (89.0–89.0) | 12.0 (12.0–12.0) | 12.0 (12.0–12.0) | 12.0 (12.0–12.0) |
| native | copy.dispatch_view | total | i3 | 113.6 (110.0–121.7) | 25.6 (25.4–25.7) | 21.8 (21.6–21.9) | 21.7 (21.3–21.9) |
| native | copy.dispatch_outcome | copy.cow | i3 | 2.4 (2.3–2.7) | 0.7 (0.7–0.8) | 1.1 (1.1–1.2) | 1.2 (1.1–1.3) |
| native | copy.dispatch_outcome | copy.cow.commitments | i3 | 0.4 (0.4–0.5) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_outcome | copy.cow.graph | i3 | 1.4 (1.4–1.5) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_outcome | copy.cow.journal | i3 | 0.2 (0.1–0.3) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_outcome | copy.cow.other | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_outcome | copy.cow.qualifications | i3 | 0.3 (0.3–0.4) | 0.5 (0.5–0.6) | 0.9 (0.9–1.0) | 1.0 (0.9–1.1) |
| native | copy.dispatch_outcome | copy.cow.states | i3 | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | copy.dispatch_outcome | copy.cow.symbols | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_outcome | copy.provenance | i3 | 13.3 (12.4–13.5) | 7.7 (7.7–8.7) | 7.3 (7.3–8.4) | 8.3 (8.3–9.7) |
| native | copy.dispatch_outcome | count.cow | i3 | 5.0 (5.0–5.0) | 2.0 (2.0–2.0) | 2.0 (2.0–2.0) | 2.0 (2.0–2.0) |
| native | copy.dispatch_outcome | count.cow.commitments | i3 | 1.0 (1.0–1.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_outcome | count.cow.graph | i3 | 1.0 (1.0–1.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_outcome | count.cow.journal | i3 | 1.0 (1.0–1.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_outcome | count.cow.other | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_outcome | count.cow.qualifications | i3 | 2.0 (2.0–2.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) |
| native | copy.dispatch_outcome | count.cow.states | i3 | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) |
| native | copy.dispatch_outcome | count.cow.symbols | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | copy.dispatch_outcome | count.provenance | i3 | 390.0 (390.0–390.0) | 262.0 (262.0–262.0) | 206.0 (206.0–206.0) | 205.0 (205.0–205.0) |
| native | copy.dispatch_outcome | count.provenance_in_cow | i3 | 1.0 (1.0–1.0) | 6.0 (6.0–6.0) | 12.0 (12.0–12.0) | 12.0 (12.0–12.0) |
| native | copy.dispatch_outcome | count.provenance_names | i3 | 152.0 (152.0–152.0) | 105.0 (105.0–105.0) | 120.0 (120.0–120.0) | 144.0 (144.0–144.0) |
| native | copy.dispatch_outcome | total | i3 | 172.8 (156.5–175.3) | 67.6 (66.9–76.2) | 65.4 (64.5–72.8) | 67.6 (66.8–78.1) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local | 0.8 (0.7–1.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | i3 | 0.9 (0.7–1.0) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local | 128.6 (117.0–132.3) | 27.6 (27.2–27.7) | 24.6 (24.3–25.8) | 24.4 (24.2–25.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | i3 | 118.6 (114.8–133.7) | 28.9 (28.1–30.0) | 25.9 (25.3–26.0) | 24.7 (24.6–25.8) |
| wasm | raw.dispatch_view | js.parse_view | main-local | 28.2 (25.9–29.3) | 15.8 (15.4–15.9) | 15.6 (15.3–16.3) | 16.1 (16.0–17.0) |
| wasm | raw.dispatch_view | js.parse_view | i3 | 29.0 (28.1–29.4) | 15.7 (15.3–16.3) | 15.8 (15.8–15.9) | 16.1 (16.1–16.6) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local | 149.5 (145.3–162.8) | 43.9 (42.8–44.0) | 40.6 (39.9–42.5) | 40.8 (40.6–42.8) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | i3 | 163.0 (143.9–164.0) | 45.0 (43.8–46.7) | 42.0 (41.4–42.2) | 41.2 (41.0–42.8) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | main-local | 169.6 (169.0–218.7) | 67.9 (67.5–71.8) | 71.5 (68.4–74.2) | 72.0 (70.8–76.0) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | i3 | 176.4 (168.5–183.9) | 70.4 (68.8–71.1) | 69.0 (67.2–75.3) | 74.0 (73.8–76.7) |
| wasm | raw.dispatch_outcome | js.parse_outcome | main-local | 73.8 (73.6–80.7) | 57.1 (55.2–58.5) | 61.7 (60.7–63.7) | 62.8 (62.6–66.3) |
| wasm | raw.dispatch_outcome | js.parse_outcome | i3 | 77.7 (75.8–87.2) | 57.1 (56.8–58.3) | 59.8 (58.5–64.4) | 64.6 (62.2–65.7) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | main-local | 244.1 (243.2–334.8) | 125.4 (123.1–131.2) | 134.0 (129.9–138.6) | 135.0 (134.1–142.8) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | i3 | 252.3 (244.2–272.2) | 128.0 (125.9–129.3) | 128.9 (125.6–139.6) | 138.5 (135.9–142.4) |
| wasm | copy.dispatch_view | total | i3 | 165.9 (165.0–169.6) | 45.6 (45.3–46.1) | 41.5 (40.6–41.5) | 41.5 (41.1–41.9) |
| wasm | copy.dispatch_view | call | i3 | 136.4 (136.2–140.3) | 29.1 (29.1–29.8) | 25.4 (25.0–25.6) | 25.2 (24.9–25.4) |
| wasm | copy.dispatch_view | copy.cow | i3 | 3.2 (3.1–3.3) | 0.6 (0.6–0.6) | 0.8 (0.8–0.8) | 0.8 (0.8–0.9) |
| wasm | copy.dispatch_view | count.cow | i3 | 5.0 (5.0–5.0) | 2.0 (2.0–2.0) | 2.0 (2.0–2.0) | 2.0 (2.0–2.0) |
| wasm | copy.dispatch_view | count.provenance | i3 | 278.0 (278.0–278.0) | 142.0 (142.0–142.0) | 74.0 (74.0–74.0) | 74.0 (74.0–74.0) |
| wasm | copy.dispatch_view | count.provenance_names | i3 | 89.0 (89.0–89.0) | 12.0 (12.0–12.0) | 12.0 (12.0–12.0) | 12.0 (12.0–12.0) |
| wasm | copy.dispatch_view | count.provenance_in_cow | i3 | 1.0 (1.0–1.0) | 6.0 (6.0–6.0) | 12.0 (12.0–12.0) | 12.0 (12.0–12.0) |
| wasm | copy.dispatch_view | copy.cow.states | i3 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | copy.dispatch_view | count.cow.states | i3 | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) |
| wasm | copy.dispatch_view | copy.cow.graph | i3 | 1.7 (1.6–1.7) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_view | count.cow.graph | i3 | 1.0 (1.0–1.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_view | copy.cow.symbols | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_view | count.cow.symbols | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_view | copy.cow.journal | i3 | 0.4 (0.3–0.4) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_view | count.cow.journal | i3 | 1.0 (1.0–1.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_view | copy.cow.commitments | i3 | 0.6 (0.6–0.7) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_view | count.cow.commitments | i3 | 1.0 (1.0–1.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_view | copy.cow.qualifications | i3 | 0.5 (0.5–0.5) | 0.5 (0.4–0.5) | 0.7 (0.7–0.7) | 0.7 (0.7–0.7) |
| wasm | copy.dispatch_view | count.cow.qualifications | i3 | 2.0 (2.0–2.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) |
| wasm | copy.dispatch_view | copy.cow.other | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_view | count.cow.other | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_outcome | total | i3 | 273.7 (269.7–274.0) | 124.6 (123.2–124.8) | 127.6 (127.5–127.8) | 138.1 (135.4–138.4) |
| wasm | copy.dispatch_outcome | call | i3 | 192.8 (177.4–195.6) | 67.9 (67.4–68.1) | 67.8 (67.6–67.8) | 74.8 (72.5–75.0) |
| wasm | copy.dispatch_outcome | copy.cow | i3 | 2.6 (2.6–2.8) | 0.6 (0.6–0.6) | 0.9 (0.8–0.9) | 1.0 (1.0–1.0) |
| wasm | copy.dispatch_outcome | count.cow | i3 | 5.0 (5.0–5.0) | 2.0 (2.0–2.0) | 2.0 (2.0–2.0) | 2.0 (2.0–2.0) |
| wasm | copy.dispatch_outcome | count.provenance | i3 | 390.0 (390.0–390.0) | 262.0 (262.0–262.0) | 206.0 (206.0–206.0) | 205.0 (205.0–205.0) |
| wasm | copy.dispatch_outcome | count.provenance_names | i3 | 152.0 (152.0–152.0) | 105.0 (105.0–105.0) | 120.0 (120.0–120.0) | 144.0 (144.0–144.0) |
| wasm | copy.dispatch_outcome | count.provenance_in_cow | i3 | 1.0 (1.0–1.0) | 6.0 (6.0–6.0) | 12.0 (12.0–12.0) | 12.0 (12.0–12.0) |
| wasm | copy.dispatch_outcome | copy.cow.states | i3 | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | copy.dispatch_outcome | count.cow.states | i3 | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) |
| wasm | copy.dispatch_outcome | copy.cow.graph | i3 | 1.6 (1.5–1.6) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_outcome | count.cow.graph | i3 | 1.0 (1.0–1.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_outcome | copy.cow.symbols | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_outcome | count.cow.symbols | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_outcome | copy.cow.journal | i3 | 0.3 (0.2–0.3) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_outcome | count.cow.journal | i3 | 1.0 (1.0–1.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_outcome | copy.cow.commitments | i3 | 0.4 (0.4–0.4) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_outcome | count.cow.commitments | i3 | 1.0 (1.0–1.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_outcome | copy.cow.qualifications | i3 | 0.5 (0.4–0.5) | 0.5 (0.5–0.5) | 0.8 (0.7–0.8) | 0.9 (0.8–0.9) |
| wasm | copy.dispatch_outcome | count.cow.qualifications | i3 | 2.0 (2.0–2.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) | 1.0 (1.0–1.0) |
| wasm | copy.dispatch_outcome | copy.cow.other | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| wasm | copy.dispatch_outcome | count.cow.other | i3 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |

## ledger-session

The agent ledger replaying the real 2026-09-24 session log: 20 events, five of them refused by policy. Played 100 times from a fresh session. Program `experiments/agent-ledger/ledger.cav` (`2206ac79f23c`), stream `ef2143311f86`, 1 episode(s) × 100 = 2000 events per pass.

| Engine | Mode | Operation | main-local | i3 |
| --- | --- | --- | --- | --- |
| native | web.dispatch_view | web.dispatch_view | 22.7 (22.5–23.3) · p95 31.3 | 24.7 (24.6–26.5) · p95 33.6 |
| native | web.dispatch_outcome | web.dispatch_outcome | 57.5 (56.9–62.6) · p95 72.4 | 59.9 (59.5–60.0) · p95 72.9 |
| native | copy.dispatch_view | copy.cow | - | 2.8 (2.7–2.8) · p95 5.4 |
| native | copy.dispatch_view | copy.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | copy.cow.graph | - | 1.8 (1.8–1.8) · p95 2.1 |
| native | copy.dispatch_view | copy.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | copy.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | copy.cow.qualifications | - | 0.7 (0.7–0.7) · p95 2.1 |
| native | copy.dispatch_view | copy.cow.states | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | copy.dispatch_view | copy.cow.symbols | - | 0.0 (0.0–0.0) · p95 1.3 |
| native | copy.dispatch_view | copy.provenance | - | 1.1 (1.1–1.1) · p95 1.3 |
| native | copy.dispatch_view | count.cow | - | 3.0 (3.0–3.0) · p95 5.0 |
| native | copy.dispatch_view | count.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | count.cow.graph | - | 1.0 (1.0–1.0) · p95 1.0 |
| native | copy.dispatch_view | count.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | count.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | count.cow.qualifications | - | 1.0 (1.0–1.0) · p95 2.0 |
| native | copy.dispatch_view | count.cow.states | - | 1.0 (1.0–1.0) · p95 1.0 |
| native | copy.dispatch_view | count.cow.symbols | - | 0.0 (0.0–0.0) · p95 1.0 |
| native | copy.dispatch_view | count.provenance | - | 59.0 (59.0–59.0) · p95 65.0 |
| native | copy.dispatch_view | count.provenance_in_cow | - | 5.0 (5.0–5.0) · p95 24.0 |
| native | copy.dispatch_view | count.provenance_names | - | 0.0 (0.0–0.0) · p95 2.0 |
| native | copy.dispatch_view | total | - | 24.7 (24.6–25.4) · p95 33.2 |
| native | copy.dispatch_outcome | copy.cow | - | 3.2 (3.2–3.5) · p95 6.0 |
| native | copy.dispatch_outcome | copy.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | copy.cow.graph | - | 1.9 (1.9–2.1) · p95 2.4 |
| native | copy.dispatch_outcome | copy.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | copy.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | copy.cow.qualifications | - | 0.8 (0.8–0.9) · p95 2.3 |
| native | copy.dispatch_outcome | copy.cow.states | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | copy.dispatch_outcome | copy.cow.symbols | - | 0.0 (0.0–0.0) · p95 1.4 |
| native | copy.dispatch_outcome | copy.provenance | - | 2.0 (2.0–2.2) · p95 2.5 |
| native | copy.dispatch_outcome | count.cow | - | 3.0 (3.0–3.0) · p95 5.0 |
| native | copy.dispatch_outcome | count.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | count.cow.graph | - | 1.0 (1.0–1.0) · p95 1.0 |
| native | copy.dispatch_outcome | count.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | count.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | count.cow.qualifications | - | 1.0 (1.0–1.0) · p95 2.0 |
| native | copy.dispatch_outcome | count.cow.states | - | 1.0 (1.0–1.0) · p95 1.0 |
| native | copy.dispatch_outcome | count.cow.symbols | - | 0.0 (0.0–0.0) · p95 1.0 |
| native | copy.dispatch_outcome | count.provenance | - | 123.0 (123.0–123.0) · p95 143.0 |
| native | copy.dispatch_outcome | count.provenance_in_cow | - | 5.0 (5.0–5.0) · p95 24.0 |
| native | copy.dispatch_outcome | count.provenance_names | - | 5.0 (5.0–5.0) · p95 10.0 |
| native | copy.dispatch_outcome | total | - | 59.9 (59.9–65.2) · p95 75.5 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.4) · p95 1.0 | 0.3 (0.3–0.3) · p95 1.0 |
| wasm | raw.dispatch_view | raw.dispatch_view | 28.9 (28.5–31.9) · p95 38.2 | 30.0 (29.1–30.1) · p95 41.0 |
| wasm | raw.dispatch_view | js.parse_view | 6.9 (6.8–8.2) · p95 10.2 | 7.2 (6.8–7.2) · p95 10.1 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 38.4 (37.8–44.6) · p95 49.7 | 40.2 (38.5–40.7) · p95 52.0 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 64.0 (62.9–65.8) · p95 85.4 | 67.4 (66.1–68.1) · p95 86.4 |
| wasm | raw.dispatch_outcome | js.parse_outcome | 45.2 (44.3–46.1) · p95 58.4 | 45.8 (44.8–46.1) · p95 57.6 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 108.9 (107.6–112.1) · p95 143.0 | 113.3 (111.0–114.4) · p95 143.1 |
| wasm | copy.dispatch_view | total | - | 40.7 (39.7–46.0) · p95 52.5 |
| wasm | copy.dispatch_view | call | - | 30.3 (30.1–34.1) · p95 42.1 |
| wasm | copy.dispatch_view | copy.cow | - | 3.2 (3.1–3.6) · p95 5.4 |
| wasm | copy.dispatch_view | count.cow | - | 3.0 (3.0–3.0) · p95 5.0 |
| wasm | copy.dispatch_view | count.provenance | - | 59.0 (59.0–59.0) · p95 65.0 |
| wasm | copy.dispatch_view | count.provenance_names | - | 0.0 (0.0–0.0) · p95 2.0 |
| wasm | copy.dispatch_view | count.provenance_in_cow | - | 5.0 (5.0–5.0) · p95 24.0 |
| wasm | copy.dispatch_view | copy.cow.states | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | copy.dispatch_view | count.cow.states | - | 1.0 (1.0–1.0) · p95 1.0 |
| wasm | copy.dispatch_view | copy.cow.graph | - | 1.8 (1.8–2.0) · p95 2.4 |
| wasm | copy.dispatch_view | count.cow.graph | - | 1.0 (1.0–1.0) · p95 1.0 |
| wasm | copy.dispatch_view | copy.cow.symbols | - | 0.0 (0.0–0.0) · p95 1.2 |
| wasm | copy.dispatch_view | count.cow.symbols | - | 0.0 (0.0–0.0) · p95 1.0 |
| wasm | copy.dispatch_view | copy.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | count.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | copy.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | count.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | copy.cow.qualifications | - | 0.9 (0.9–1.0) · p95 2.1 |
| wasm | copy.dispatch_view | count.cow.qualifications | - | 1.0 (1.0–1.0) · p95 2.0 |
| wasm | copy.dispatch_view | copy.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | count.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | total | - | 113.6 (112.4–116.8) · p95 146.6 |
| wasm | copy.dispatch_outcome | call | - | 67.5 (66.7–68.1) · p95 87.7 |
| wasm | copy.dispatch_outcome | copy.cow | - | 2.6 (2.6–2.7) · p95 4.7 |
| wasm | copy.dispatch_outcome | count.cow | - | 3.0 (3.0–3.0) · p95 5.0 |
| wasm | copy.dispatch_outcome | count.provenance | - | 123.0 (123.0–123.0) · p95 143.0 |
| wasm | copy.dispatch_outcome | count.provenance_names | - | 5.0 (5.0–5.0) · p95 10.0 |
| wasm | copy.dispatch_outcome | count.provenance_in_cow | - | 5.0 (5.0–5.0) · p95 24.0 |
| wasm | copy.dispatch_outcome | copy.cow.states | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | copy.dispatch_outcome | count.cow.states | - | 1.0 (1.0–1.0) · p95 1.0 |
| wasm | copy.dispatch_outcome | copy.cow.graph | - | 1.5 (1.4–1.5) · p95 2.0 |
| wasm | copy.dispatch_outcome | count.cow.graph | - | 1.0 (1.0–1.0) · p95 1.0 |
| wasm | copy.dispatch_outcome | copy.cow.symbols | - | 0.0 (0.0–0.0) · p95 1.1 |
| wasm | copy.dispatch_outcome | count.cow.symbols | - | 0.0 (0.0–0.0) · p95 1.0 |
| wasm | copy.dispatch_outcome | copy.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | count.cow.journal | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | copy.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | count.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | copy.cow.qualifications | - | 0.7 (0.7–0.8) · p95 1.8 |
| wasm | copy.dispatch_outcome | count.cow.qualifications | - | 1.0 (1.0–1.0) · p95 2.0 |
| wasm | copy.dispatch_outcome | copy.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | count.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | main-local | i3 |
| --- | --- | --- | --- | --- |
| native | web.dispatch_view | web.dispatch_view | 59.4 (58.8–60.7) · p95 133.4 | 62.0 (61.4–63.2) · p95 140.3 |
| native | web.dispatch_outcome | web.dispatch_outcome | 118.0 (116.0–128.7) · p95 195.5 | 126.2 (125.7–126.5) · p95 207.4 |
| native | copy.dispatch_view | copy.cow | - | 1.5 (1.5–1.5) · p95 3.0 |
| native | copy.dispatch_view | copy.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.5 |
| native | copy.dispatch_view | copy.cow.graph | - | 0.7 (0.7–0.7) · p95 1.0 |
| native | copy.dispatch_view | copy.cow.journal | - | 0.0 (0.0–0.0) · p95 0.4 |
| native | copy.dispatch_view | copy.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | copy.cow.qualifications | - | 0.0 (0.0–0.0) · p95 0.6 |
| native | copy.dispatch_view | copy.cow.states | - | 0.3 (0.3–0.3) · p95 1.6 |
| native | copy.dispatch_view | copy.cow.symbols | - | 0.0 (0.0–0.0) · p95 0.5 |
| native | copy.dispatch_view | copy.provenance | - | 3.9 (3.9–3.9) · p95 10.4 |
| native | copy.dispatch_view | count.cow | - | 4.0 (4.0–4.0) · p95 10.0 |
| native | copy.dispatch_view | count.cow.commitments | - | 0.0 (0.0–0.0) · p95 3.0 |
| native | copy.dispatch_view | count.cow.graph | - | 1.0 (1.0–1.0) · p95 1.0 |
| native | copy.dispatch_view | count.cow.journal | - | 0.0 (0.0–0.0) · p95 1.0 |
| native | copy.dispatch_view | count.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_view | count.cow.qualifications | - | 0.0 (0.0–0.0) · p95 3.0 |
| native | copy.dispatch_view | count.cow.states | - | 1.0 (1.0–1.0) · p95 7.0 |
| native | copy.dispatch_view | count.cow.symbols | - | 0.0 (0.0–0.0) · p95 1.0 |
| native | copy.dispatch_view | count.provenance | - | 129.0 (129.0–129.0) · p95 333.0 |
| native | copy.dispatch_view | count.provenance_in_cow | - | 0.0 (0.0–0.0) · p95 12.0 |
| native | copy.dispatch_view | count.provenance_names | - | 42.0 (42.0–42.0) · p95 123.0 |
| native | copy.dispatch_view | total | - | 61.4 (60.8–61.7) · p95 139.5 |
| native | copy.dispatch_outcome | copy.cow | - | 1.6 (1.6–1.6) · p95 3.0 |
| native | copy.dispatch_outcome | copy.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.4 |
| native | copy.dispatch_outcome | copy.cow.graph | - | 0.8 (0.8–0.8) · p95 1.0 |
| native | copy.dispatch_outcome | copy.cow.journal | - | 0.0 (0.0–0.0) · p95 0.4 |
| native | copy.dispatch_outcome | copy.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | copy.cow.qualifications | - | 0.0 (0.0–0.0) · p95 0.6 |
| native | copy.dispatch_outcome | copy.cow.states | - | 0.3 (0.3–0.3) · p95 1.5 |
| native | copy.dispatch_outcome | copy.cow.symbols | - | 0.0 (0.0–0.0) · p95 0.5 |
| native | copy.dispatch_outcome | copy.provenance | - | 10.2 (10.2–10.5) · p95 16.8 |
| native | copy.dispatch_outcome | count.cow | - | 4.0 (4.0–4.0) · p95 10.0 |
| native | copy.dispatch_outcome | count.cow.commitments | - | 0.0 (0.0–0.0) · p95 3.0 |
| native | copy.dispatch_outcome | count.cow.graph | - | 1.0 (1.0–1.0) · p95 1.0 |
| native | copy.dispatch_outcome | count.cow.journal | - | 0.0 (0.0–0.0) · p95 1.0 |
| native | copy.dispatch_outcome | count.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | copy.dispatch_outcome | count.cow.qualifications | - | 0.0 (0.0–0.0) · p95 3.0 |
| native | copy.dispatch_outcome | count.cow.states | - | 1.0 (1.0–1.0) · p95 7.0 |
| native | copy.dispatch_outcome | count.cow.symbols | - | 0.0 (0.0–0.0) · p95 1.0 |
| native | copy.dispatch_outcome | count.provenance | - | 349.0 (349.0–349.0) · p95 554.0 |
| native | copy.dispatch_outcome | count.provenance_in_cow | - | 0.0 (0.0–0.0) · p95 12.0 |
| native | copy.dispatch_outcome | count.provenance_names | - | 104.0 (104.0–104.0) · p95 261.0 |
| native | copy.dispatch_outcome | total | - | 124.6 (124.6–131.0) · p95 204.8 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.6 (0.5–0.6) · p95 1.7 | 0.6 (0.6–0.6) · p95 1.7 |
| wasm | raw.dispatch_view | raw.dispatch_view | 72.3 (71.6–72.6) · p95 147.4 | 72.8 (72.4–79.6) · p95 148.6 |
| wasm | raw.dispatch_view | js.parse_view | 25.3 (25.0–25.9) · p95 32.0 | 25.8 (25.6–27.1) · p95 33.1 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 120.1 (119.0–121.2) · p95 178.7 | 120.5 (120.3–127.0) · p95 181.8 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 145.2 (140.4–145.3) · p95 231.6 | 143.8 (138.3–144.0) · p95 224.1 |
| wasm | raw.dispatch_outcome | js.parse_outcome | 77.6 (75.4–77.9) · p95 94.4 | 75.1 (75.1–77.3) · p95 91.7 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 225.6 (218.3–225.9) · p95 321.3 | 220.5 (216.1–222.8) · p95 310.6 |
| wasm | copy.dispatch_view | total | - | 121.5 (121.1–123.9) · p95 177.9 |
| wasm | copy.dispatch_view | call | - | 74.4 (74.1–76.0) · p95 148.7 |
| wasm | copy.dispatch_view | copy.cow | - | 1.6 (1.6–1.7) · p95 3.4 |
| wasm | copy.dispatch_view | count.cow | - | 4.0 (4.0–4.0) · p95 10.0 |
| wasm | copy.dispatch_view | count.provenance | - | 129.0 (129.0–129.0) · p95 333.0 |
| wasm | copy.dispatch_view | count.provenance_names | - | 42.0 (42.0–42.0) · p95 123.0 |
| wasm | copy.dispatch_view | count.provenance_in_cow | - | 0.0 (0.0–0.0) · p95 12.0 |
| wasm | copy.dispatch_view | copy.cow.states | - | 0.2 (0.2–0.2) · p95 1.5 |
| wasm | copy.dispatch_view | count.cow.states | - | 1.0 (1.0–1.0) · p95 7.0 |
| wasm | copy.dispatch_view | copy.cow.graph | - | 0.8 (0.8–0.9) · p95 1.2 |
| wasm | copy.dispatch_view | count.cow.graph | - | 1.0 (1.0–1.0) · p95 1.0 |
| wasm | copy.dispatch_view | copy.cow.symbols | - | 0.0 (0.0–0.0) · p95 0.7 |
| wasm | copy.dispatch_view | count.cow.symbols | - | 0.0 (0.0–0.0) · p95 1.0 |
| wasm | copy.dispatch_view | copy.cow.journal | - | 0.0 (0.0–0.0) · p95 0.5 |
| wasm | copy.dispatch_view | count.cow.journal | - | 0.0 (0.0–0.0) · p95 1.0 |
| wasm | copy.dispatch_view | copy.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.8 |
| wasm | copy.dispatch_view | count.cow.commitments | - | 0.0 (0.0–0.0) · p95 3.0 |
| wasm | copy.dispatch_view | copy.cow.qualifications | - | 0.0 (0.0–0.0) · p95 0.8 |
| wasm | copy.dispatch_view | count.cow.qualifications | - | 0.0 (0.0–0.0) · p95 3.0 |
| wasm | copy.dispatch_view | copy.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_view | count.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | total | - | 222.1 (220.7–227.5) · p95 310.8 |
| wasm | copy.dispatch_outcome | call | - | 144.6 (143.3–149.0) · p95 226.6 |
| wasm | copy.dispatch_outcome | copy.cow | - | 1.5 (1.5–1.6) · p95 3.1 |
| wasm | copy.dispatch_outcome | count.cow | - | 4.0 (4.0–4.0) · p95 10.0 |
| wasm | copy.dispatch_outcome | count.provenance | - | 349.0 (349.0–349.0) · p95 554.0 |
| wasm | copy.dispatch_outcome | count.provenance_names | - | 104.0 (104.0–104.0) · p95 261.0 |
| wasm | copy.dispatch_outcome | count.provenance_in_cow | - | 0.0 (0.0–0.0) · p95 12.0 |
| wasm | copy.dispatch_outcome | copy.cow.states | - | 0.2 (0.2–0.2) · p95 1.3 |
| wasm | copy.dispatch_outcome | count.cow.states | - | 1.0 (1.0–1.0) · p95 7.0 |
| wasm | copy.dispatch_outcome | copy.cow.graph | - | 0.8 (0.8–0.8) · p95 1.2 |
| wasm | copy.dispatch_outcome | count.cow.graph | - | 1.0 (1.0–1.0) · p95 1.0 |
| wasm | copy.dispatch_outcome | copy.cow.symbols | - | 0.0 (0.0–0.0) · p95 0.7 |
| wasm | copy.dispatch_outcome | count.cow.symbols | - | 0.0 (0.0–0.0) · p95 1.0 |
| wasm | copy.dispatch_outcome | copy.cow.journal | - | 0.0 (0.0–0.0) · p95 0.3 |
| wasm | copy.dispatch_outcome | count.cow.journal | - | 0.0 (0.0–0.0) · p95 1.0 |
| wasm | copy.dispatch_outcome | copy.cow.commitments | - | 0.0 (0.0–0.0) · p95 0.7 |
| wasm | copy.dispatch_outcome | count.cow.commitments | - | 0.0 (0.0–0.0) · p95 3.0 |
| wasm | copy.dispatch_outcome | copy.cow.qualifications | - | 0.0 (0.0–0.0) · p95 0.7 |
| wasm | copy.dispatch_outcome | count.cow.qualifications | - | 0.0 (0.0–0.0) · p95 3.0 |
| wasm | copy.dispatch_outcome | copy.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | copy.dispatch_outcome | count.cow.other | - | 0.0 (0.0–0.0) · p95 0.0 |

