# Caveat performance baseline: 2026-09-29T07-42-49-636-baseline

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=baseline --build=never --keep-samples --target=main-local=tree:. --target=hist-e6ace96=runtime:C:/Users/walte/AppData/Local/Temp/claude/C--Dev-GPT-SandBox-Web/3bc468a2-40f8-4895-bd0a-fb452fae7549/scratchpad/perf-scratch/scout/hist-e6ace96/pkg-reactive --workloads=glowcap-replay,trail-rescue-scenarios --modes=published-method,web.dispatch_view,abi.dispatch_view,raw.dispatch_view,kit --affinity-set=0x3,0xC00,0x3000,0xC00000,0x3C00 --monitor-interval=1 --out=C:/Users/walte/AppData/Local/Temp/claude/C--Dev-GPT-SandBox-Web/3bc468a2-40f8-4895-bd0a-fb452fae7549/scratchpad/perf-scratch/m2/cores/run`

## Method

- Suite `baseline`: 3 repeat(s) of every mode, each in a fresh process; within a repeat every target runs a mode back to back, the first target rotating.
- Per-event modes: 1 untimed warm-up pass(es), then 3 timed pass(es), each episode on a freshly opened session. `published-method` runs 3 round(s) with no warm-up, as the published harness did.
- Load, save and restore: 3 untimed then 30 timed calls; published resume method: 3 run(s).
- Statistic: quantile q(p) = sorted[floor(p·(n−1))] over the pooled samples of a run (the published rule; the median is the lower median). A cell is the median over runs of each run's pooled median, with the lowest and highest run in brackets, then the median over runs of each run's p95.
- Units: microseconds per operation unless the operation says otherwise.

## Targets

| Target | Kind | Revision | Reactive WebAssembly sha256 | Glue sha256 | Kit session.mjs sha256 | Native benchmark |
| --- | --- | --- | --- | --- | --- | --- |
| main-local@0x3 | tree | v0.1.0-rc.4-6-gd132610-dirty (d132610), tracked changes | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |
| main-local@0xC00 | tree | v0.1.0-rc.4-6-gd132610-dirty (d132610), tracked changes | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |
| main-local@0x3000 | tree | v0.1.0-rc.4-6-gd132610-dirty (d132610), tracked changes | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |
| main-local@0xC00000 | tree | v0.1.0-rc.4-6-gd132610-dirty (d132610), tracked changes | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |
| main-local@0x3C00 | tree | v0.1.0-rc.4-6-gd132610-dirty (d132610), tracked changes | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |
| hist-e6ace96@0x3 | runtime | n/a | `d897787f414a` (1682294 B) | `33a41153ec24` | - | - |
| hist-e6ace96@0xC00 | runtime | n/a | `d897787f414a` (1682294 B) | `33a41153ec24` | - | - |
| hist-e6ace96@0x3000 | runtime | n/a | `d897787f414a` (1682294 B) | `33a41153ec24` | - | - |
| hist-e6ace96@0xC00000 | runtime | n/a | `d897787f414a` (1682294 B) | `33a41153ec24` | - | - |
| hist-e6ace96@0x3C00 | runtime | n/a | `d897787f414a` (1682294 B) | `33a41153ec24` | - | - |

## Environment

- CPU: Intel(R) Core(TM) Ultra 9 275HX; 24 cores, 24 logical processors; performance cores 0, 1, 10, 11, 12, 13, 22, 23
- Memory: 63.4 GiB
- OS: Microsoft Windows 11 Home 10.0.26200 (build 26200)
- Power: Power Scheme GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c  (High performance); battery status 2 (2 = on mains power)
- Pinning: every target under each of the masks 0x3 (0, 1), 0xC00 (10, 11), 0x3000 (12, 13), 0xC00000 (22, 23), 0x3C00 (10, 11, 12, 13) in turn, as LABEL@MASK; priority high, via cmd /c start /b /wait /<priority> /affinity <mask>
- Node v24.11.1, V8 13.6.233.10-node.28, npm 11.12.1
- Rust: rustc 1.98.1 (48a229cea 2026-09-01) (LLVM version: 22.1.8); cargo 1.98.1 (797e8a9bc 2026-08-05); 1.98.1-x86_64-pc-windows-msvc (overridden by 'C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF\rust-toolchain.toml')
- wasm-bindgen 0.2.104; wasm-opt not installed (the build does not use it)
- Load before: total CPU 6/0/3%; busiest over the window: claude 1.438s, powershell 0.625s, claude 0.547s, ChatGPT 0.203s, claude 0.188s, ChatGPT 0.125s
- Load after: total CPU 5/0/1%; busiest over the window: claude 2.297s, powershell 0.766s, claude 0.719s, ChatGPT 0.328s, IntelGraphicsSoftware 0.219s, claude 0.219s
- Load during: typeperf every 1 s for the whole run, 694 samples: total mean 9.3% p95 13.6% max 20.3%; processor 0 mean 15.7% p95 59.6% max 100%; processor 1 mean 23.9% p95 70.6% max 100%; processor 10 mean 25% p95 89.1% max 100%; processor 11 mean 19.1% p95 62.6% max 100%; processor 12 mean 18.6% p95 100% max 100%; processor 13 mean 39.8% p95 100% max 100%; processor 22 mean 10.8% p95 41.2% max 100%; processor 23 mean 45.2% p95 100% max 100%; processor 0 MHz mean 4146.2 MHz p95 5064.8 MHz max 5154.1 MHz; processor 1 MHz mean 4539 MHz p95 5092.9 MHz max 5155.2 MHz; processor 10 MHz mean 4025.7 MHz p95 4430.6 MHz max 4748.5 MHz; processor 11 MHz mean 3927 MHz p95 4272.9 MHz max 4579.7 MHz; processor 12 MHz mean 3950.8 MHz p95 4347.8 MHz max 4721.6 MHz; processor 13 MHz mean 3914.2 MHz p95 4728.2 MHz max 5074.8 MHz; processor 22 MHz mean 3338.9 MHz p95 4454.1 MHz max 4711.4 MHz; processor 23 MHz mean 4377.6 MHz p95 4820.2 MHz max 5009.1 MHz; total above 10% in 233 and above 25% in 0 samples; pinned processors summing above 150% in 689

## Published reference

- Dispatch + view, Glowcap replay: median 51.6 µs, p95 59.1 µs, max 646.7 µs (experiments/glowcap/evidence/round6-replay.json timing.dispatchAndShippedOutput); runtime 8d8596a, reactive WebAssembly `d897787f414a`, repository e6ace96.
- Resume ten minutes of play: median 3.2416 ms of runs 7.154, 3.2416, 3.1401 ms; save 2045 bytes (experiments/glowcap/evidence/round6-replay.json timing.resume (caveat5)).

## Final-state consistency

Within each target, every run of every mode and engine ended each workload in the same saved state (180 runs compared).

Between targets, by workload and kind of state (targets in one group ended in identical states):

- glowcap-replay save-text: main-local@0x3, main-local@0xC00, main-local@0x3000, main-local@0xC00000, main-local@0x3C00 `fda3d6109816` | hist-e6ace96@0x3, hist-e6ace96@0xC00, hist-e6ace96@0x3000, hist-e6ace96@0xC00000, hist-e6ace96@0x3C00 `0e3d5c34f900`
- glowcap-replay adapter-save-json: main-local@0x3, main-local@0xC00, main-local@0x3000, main-local@0xC00000, main-local@0x3C00 `bcc2ec60c6f5` | hist-e6ace96@0x3, hist-e6ace96@0xC00, hist-e6ace96@0x3000, hist-e6ace96@0xC00000, hist-e6ace96@0x3C00 `6a38724483ec`
- trail-rescue-scenarios save-text: main-local@0x3, main-local@0xC00, main-local@0x3000, main-local@0xC00000, main-local@0x3C00 `bb72a4e76921`

Skipped jobs (the target cannot run them):
- 0051-hist-e6ace96@0x3-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0052-hist-e6ace96@0xC00-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0053-hist-e6ace96@0x3000-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0054-hist-e6ace96@0xC00000-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0055-hist-e6ace96@0x3C00-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0061-hist-e6ace96@0x3-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0062-hist-e6ace96@0xC00-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0063-hist-e6ace96@0x3000-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0064-hist-e6ace96@0xC00000-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0065-hist-e6ace96@0x3C00-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0120-hist-e6ace96@0x3-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0121-hist-e6ace96@0xC00-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0122-hist-e6ace96@0x3000-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0123-hist-e6ace96@0xC00000-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0124-hist-e6ace96@0x3C00-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0130-hist-e6ace96@0x3-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0131-hist-e6ace96@0xC00-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0132-hist-e6ace96@0x3000-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0133-hist-e6ace96@0xC00000-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0134-hist-e6ace96@0x3C00-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0189-hist-e6ace96@0x3-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0190-hist-e6ace96@0xC00-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0191-hist-e6ace96@0x3000-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0192-hist-e6ace96@0xC00000-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0193-hist-e6ace96@0x3C00-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0199-hist-e6ace96@0x3-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0200-hist-e6ace96@0xC00-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0201-hist-e6ace96@0x3000-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0202-hist-e6ace96@0xC00000-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0203-hist-e6ace96@0x3C00-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)

## One Glowcap event + view, piece by piece

Medians over runs of each run's median on the steady idle ticks (events 1300–9999), where the pooled median sits; each row is its own process, so parts need not sum exactly to the whole.

| Piece | main-local@0x3 | main-local@0xC00 | main-local@0x3000 | main-local@0xC00000 | main-local@0x3C00 | hist-e6ace96@0x3 | hist-e6ace96@0xC00 | hist-e6ace96@0x3000 | hist-e6ace96@0xC00000 | hist-e6ace96@0x3C00 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Published method: adapter dispatch + view | 34.1 | 45.3 | 47.3 | 51.1 | 46.3 | 33.2 | 42.1 | 45.9 | 48.8 | 45.7 |
| Adapter dispatch (stringify, WebAssembly dispatch_view, JSON.parse) | - | - | - | - | - | - | - | - | - | - |
| Adapter view() (JavaScript reshaping only) | - | - | - | - | - | - | - | - | - | - |
| JSON.stringify(payload) | 0.2 | 0.3 | 0.3 | 0.3 | 0.3 | 0.2 | 0.3 | 0.3 | 0.3 | 0.3 |
| wasm-bindgen dispatch_view call, returning the view text | 19.0 | 25.1 | 28.3 | 28.2 | 26.7 | 18.2 | 23.5 | 26.9 | 27.7 | 25.0 |
|   arguments copied into WebAssembly memory | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 |
|   WebAssembly execution (resolve, apply, view, serialize) | 16.3 | 21.3 | 22.5 | 24.7 | 22.8 | 15.6 | 20.7 | 21.5 | 23.5 | 21.4 |
|   result decoded to a JavaScript string | 1.9 | 2.4 | 2.5 | 2.8 | 2.5 | 1.9 | 2.5 | 2.5 | 2.8 | 2.5 |
|   result freed | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 |
|   the four pieces, timed together in one process | 18.4 | 24.0 | 25.4 | 27.7 | 25.6 | 17.7 | 23.4 | 24.3 | 26.5 | 24.2 |
| JSON.parse(view text) | 12.8 | 16.7 | 18.9 | 19.3 | 17.6 | 12.5 | 16.1 | 18.3 | 18.7 | 17.1 |
| WebAssembly view() alone, execution only | - | - | - | - | - | - | - | - | - | - |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | 107.6 | 139.0 | 145.6 | 159.3 | 144.2 | - | - | - | - | - |
| Kit dispatch + view() | 130.6 | 168.3 | 176.5 | 192.8 | 174.3 | - | - | - | - | - |
| Native web::dispatch_view (the exported function, natively) | 16.0 | 20.2 | 22.4 | 23.5 | 21.1 | - | - | - | - | - |
| Native apply (numeric parameters: rules and bindings only) | - | - | - | - | - | - | - | - | - | - |
| Native dispatch_view_json (resolve + apply + view, not serialized) | - | - | - | - | - | - | - | - | - | - |
| Native view() build | - | - | - | - | - | - | - | - | - | - |
| Native view serialize | - | - | - | - | - | - | - | - | - | - |
| Native snapshot() build | - | - | - | - | - | - | - | - | - | - |
| Native session clone (upper bound of the transaction copy) | - | - | - | - | - | - | - | - | - | - |
| Native apply, the same program without bindings (glowcap-unbound) | - | - | - | - | - | - | - | - | - | - |
| *derived:* native resolve_payload ≈ dispatch_view_json − apply − view | - | - | - | - | - | - | - | - | - | - |
| *derived:* native binding evaluation ≈ apply − apply without bindings | - | - | - | - | - | - | - | - | - | - |
| *derived:* WebAssembly ÷ native, same exported function (ratio) | 1.0 | 1.1 | 1.0 | 1.1 | 1.1 | - | - | - | - | - |

## glowcap-replay

The Glowcap replay program and the exact event stream behind the published 51.6 us figure (experiments/glowcap/harness.mjs bench(), runtime/examples/profile_dispatch.rs): absorb cave glowcap, absorb pool duskcap, taste ruin glowcap, then 9,997 ticks of dt 0.05. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | main-local@0x3 | main-local@0xC00 | main-local@0x3000 | main-local@0xC00000 | main-local@0x3C00 | hist-e6ace96@0x3 | hist-e6ace96@0xC00 | hist-e6ace96@0x3000 | hist-e6ace96@0xC00000 | hist-e6ace96@0x3C00 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| native | web.dispatch_view | web.dispatch_view | 16.0 (15.9–16.2) · p95 18.3 | 20.3 (20.2–22.9) · p95 25.6 | 22.5 (21.4–23.2) · p95 25.9 | 23.6 (23.4–23.7) · p95 27.0 | 21.2 (20.3–21.6) · p95 23.5 | - | - | - | - | - |
| wasm | published-method | adapter.dispatch+view | 34.2 (33.7–34.3) · p95 49.2 | 45.3 (44.4–47.9) · p95 57.0 | 47.4 (47.2–50.6) · p95 60.8 | 51.2 (50.7–53.6) · p95 68.3 | 46.4 (46.0–48.0) · p95 58.1 | 33.3 (33.2–33.3) · p95 43.8 | 42.3 (42.2–47.1) · p95 53.0 | 46.0 (44.8–48.4) · p95 57.3 | 48.9 (48.4–50.5) · p95 64.3 | 45.9 (44.0–45.9) · p95 56.2 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.2 (0.2–0.2) · p95 0.4 | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.3–0.3) · p95 0.6 | 0.3 (0.3–0.4) · p95 0.6 | 0.3 (0.3–0.3) · p95 0.6 | 0.2 (0.2–0.2) · p95 0.5 | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.3–0.3) · p95 0.6 | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 19.1 (19.0–19.1) · p95 21.8 | 25.2 (24.7–27.2) · p95 30.8 | 28.3 (27.4–29.1) · p95 32.5 | 28.3 (28.2–30.0) · p95 34.4 | 26.7 (26.0–27.5) · p95 31.0 | 18.3 (18.0–18.4) · p95 21.9 | 23.6 (23.6–24.2) · p95 28.6 | 27.0 (25.2–28.0) · p95 30.6 | 27.8 (26.8–28.2) · p95 33.4 | 25.1 (24.0–25.6) · p95 30.0 |
| wasm | raw.dispatch_view | js.parse_view | 12.8 (12.6–12.8) · p95 14.7 | 16.6 (16.3–17.7) · p95 20.2 | 18.8 (18.2–19.0) · p95 21.1 | 19.3 (18.5–19.6) · p95 22.6 | 17.5 (17.1–18.0) · p95 20.0 | 12.4 (12.3–12.5) · p95 15.0 | 16.1 (16.0–16.4) · p95 18.8 | 18.3 (17.0–18.8) · p95 20.3 | 18.7 (18.4–18.8) · p95 21.9 | 17.0 (16.2–17.2) · p95 19.5 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 32.1 (32.1–32.2) · p95 36.5 | 42.2 (41.4–45.3) · p95 51.2 | 47.6 (46.1–48.5) · p95 53.7 | 48.1 (47.3–50.1) · p95 57.1 | 44.7 (43.6–46.1) · p95 50.9 | 31.1 (30.6–31.2) · p95 37.1 | 40.1 (40.0–41.0) · p95 47.0 | 45.7 (42.7–47.3) · p95 50.4 | 47.0 (45.6–47.6) · p95 55.2 | 42.6 (40.6–43.2) · p95 49.0 |
| wasm | abi.dispatch_view | abi.encode_args | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 |
| wasm | abi.dispatch_view | abi.exec | 16.4 (16.2–16.4) · p95 19.0 | 21.4 (21.2–23.0) · p95 26.3 | 22.6 (22.4–24.7) · p95 26.3 | 24.7 (24.5–25.2) · p95 30.4 | 22.9 (21.3–23.0) · p95 27.0 | 15.6 (15.6–15.7) · p95 18.3 | 20.9 (20.6–22.2) · p95 24.8 | 21.6 (21.3–23.7) · p95 24.7 | 23.5 (23.4–23.9) · p95 29.1 | 21.5 (20.9–21.7) · p95 24.8 |
| wasm | abi.dispatch_view | abi.decode | 1.9 (1.9–1.9) · p95 2.5 | 2.4 (2.4–2.5) · p95 3.2 | 2.5 (2.5–2.7) · p95 3.3 | 2.8 (2.8–2.8) · p95 3.4 | 2.5 (2.4–2.5) · p95 3.3 | 1.9 (1.9–1.9) · p95 2.6 | 2.5 (2.4–2.6) · p95 3.2 | 2.5 (2.5–2.7) · p95 3.2 | 2.7 (2.7–2.7) · p95 3.5 | 2.5 (2.4–2.5) · p95 3.1 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | abi.dispatch_view | abi.dispatch_view | 18.5 (18.4–18.6) · p95 21.4 | 24.2 (23.9–25.8) · p95 29.7 | 25.5 (25.2–27.7) · p95 29.6 | 27.8 (27.6–28.2) · p95 34.2 | 25.7 (23.9–25.8) · p95 30.4 | 17.8 (17.8–17.9) · p95 20.7 | 23.7 (23.3–25.0) · p95 28.2 | 24.4 (24.1–26.8) · p95 27.8 | 26.6 (26.4–27.0) · p95 32.8 | 24.3 (23.6–24.5) · p95 28.0 |
| wasm | kit | kit.dispatch | 107.3 (106.0–109.8) · p95 145.5 | 137.8 (136.1–139.0) · p95 165.7 | 145.1 (142.4–154.5) · p95 167.1 | 158.7 (156.0–158.9) · p95 188.2 | 143.7 (135.7–144.3) · p95 162.0 | - | - | - | - | - |
| wasm | kit | kit.view | 22.5 (22.2–22.8) · p95 26.0 | 28.7 (28.5–28.9) · p95 34.7 | 30.4 (29.5–32.2) · p95 34.6 | 33.0 (32.4–33.4) · p95 38.9 | 29.6 (28.1–30.0) · p95 33.7 | - | - | - | - | - |
| wasm | kit | kit.dispatch+view | 130.2 (128.5–133.0) · p95 173.6 | 166.9 (165.0–167.9) · p95 200.2 | 175.9 (172.1–186.9) · p95 202.5 | 192.1 (188.8–192.5) · p95 227.9 | 173.7 (163.9–174.5) · p95 195.3 | - | - | - | - | - |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | web.dispatch_view | web.dispatch_view | main-local@0x3 | 97.6 (93.6–99.8) | 18.1 (18.1–18.3) | 15.9 (15.8–16.1) | 16.0 (15.8–16.2) |
| native | web.dispatch_view | web.dispatch_view | main-local@0xC00 | 106.8 (101.1–119.4) | 22.6 (22.3–26.2) | 20.3 (19.9–23.1) | 20.2 (20.1–22.8) |
| native | web.dispatch_view | web.dispatch_view | main-local@0x3000 | 116.6 (111.4–118.7) | 25.1 (23.9–26.0) | 22.3 (21.4–23.2) | 22.4 (21.3–23.1) |
| native | web.dispatch_view | web.dispatch_view | main-local@0xC00000 | 133.8 (131.1–134.4) | 26.2 (25.8–26.6) | 23.2 (22.8–23.6) | 23.5 (23.3–23.6) |
| native | web.dispatch_view | web.dispatch_view | main-local@0x3C00 | 110.6 (109.3–119.5) | 23.6 (22.6–24.1) | 21.1 (20.2–21.3) | 21.1 (20.2–21.5) |
| wasm | published-method | adapter.dispatch+view | main-local@0x3 | 170.6 (144.5–193.8) | 37.6 (36.9–38.2) | 35.0 (34.6–36.7) | 34.1 (33.6–34.1) |
| wasm | published-method | adapter.dispatch+view | main-local@0xC00 | 165.1 (155.9–195.4) | 46.3 (46.0–53.3) | 44.4 (43.3–49.0) | 45.3 (44.4–47.2) |
| wasm | published-method | adapter.dispatch+view | main-local@0x3000 | 176.3 (156.5–342.0) | 50.7 (50.6–52.9) | 47.1 (46.1–51.1) | 47.3 (47.1–50.5) |
| wasm | published-method | adapter.dispatch+view | main-local@0xC00000 | 178.4 (175.6–213.4) | 54.3 (52.3–58.3) | 51.8 (47.8–55.3) | 51.1 (50.8–53.4) |
| wasm | published-method | adapter.dispatch+view | main-local@0x3C00 | 170.7 (159.2–265.3) | 52.7 (51.2–52.8) | 48.2 (47.6–48.4) | 46.3 (45.6–47.8) |
| wasm | published-method | adapter.dispatch+view | hist-e6ace96@0x3 | 167.0 (164.5–279.1) | 37.3 (36.5–38.3) | 34.4 (34.0–34.4) | 33.2 (33.0–33.2) |
| wasm | published-method | adapter.dispatch+view | hist-e6ace96@0xC00 | 176.3 (165.3–224.0) | 48.5 (44.5–50.0) | 44.4 (44.3–48.5) | 42.1 (41.8–46.8) |
| wasm | published-method | adapter.dispatch+view | hist-e6ace96@0x3000 | 179.4 (168.4–200.8) | 50.0 (48.4–50.4) | 47.3 (45.5–49.3) | 45.9 (44.6–48.3) |
| wasm | published-method | adapter.dispatch+view | hist-e6ace96@0xC00000 | 191.5 (165.3–221.5) | 50.9 (48.9–52.4) | 51.0 (49.8–53.1) | 48.8 (48.3–50.4) |
| wasm | published-method | adapter.dispatch+view | hist-e6ace96@0x3C00 | 178.2 (170.3–180.6) | 52.5 (50.1–53.5) | 46.7 (42.9–47.2) | 45.7 (43.9–45.7) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local@0x3 | 0.8 (0.7–0.8) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local@0xC00 | 0.8 (0.8–0.8) | 0.3 (0.2–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local@0x3000 | 1.2 (0.9–1.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local@0xC00000 | 0.7 (0.7–1.0) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.4) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local@0x3C00 | 0.7 (0.7–0.9) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | hist-e6ace96@0x3 | 0.8 (0.7–0.9) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | raw.dispatch_view | js.stringify_payload | hist-e6ace96@0xC00 | 0.8 (0.8–1.0) | 0.3 (0.2–0.3) | 0.3 (0.2–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | hist-e6ace96@0x3000 | 0.9 (0.9–1.0) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | hist-e6ace96@0xC00000 | 0.8 (0.8–0.9) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | hist-e6ace96@0x3C00 | 0.8 (0.8–1.0) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local@0x3 | 99.2 (96.8–106.0) | 21.2 (21.2–21.3) | 19.2 (19.2–19.3) | 19.0 (19.0–19.1) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local@0xC00 | 112.9 (108.4–120.6) | 26.7 (25.5–27.0) | 25.2 (24.3–26.1) | 25.1 (24.6–27.4) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local@0x3000 | 138.7 (136.8–143.8) | 30.9 (30.4–32.1) | 28.3 (27.5–28.7) | 28.3 (27.3–29.1) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local@0xC00000 | 116.6 (105.5–129.0) | 31.2 (27.5–33.2) | 28.6 (28.3–29.8) | 28.2 (28.1–29.9) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local@0x3C00 | 126.5 (120.6–132.6) | 30.1 (29.4–31.0) | 27.0 (25.3–28.3) | 26.7 (26.0–27.4) |
| wasm | raw.dispatch_view | raw.dispatch_view | hist-e6ace96@0x3 | 97.1 (96.2–116.1) | 20.5 (20.4–20.6) | 18.2 (18.1–18.4) | 18.2 (17.9–18.4) |
| wasm | raw.dispatch_view | raw.dispatch_view | hist-e6ace96@0xC00 | 134.8 (112.3–135.8) | 26.1 (25.5–26.9) | 23.9 (22.8–25.4) | 23.5 (23.5–24.1) |
| wasm | raw.dispatch_view | raw.dispatch_view | hist-e6ace96@0x3000 | 136.7 (131.4–137.6) | 29.7 (28.0–31.0) | 27.1 (25.4–27.8) | 26.9 (25.1–27.9) |
| wasm | raw.dispatch_view | raw.dispatch_view | hist-e6ace96@0xC00000 | 122.5 (121.4–123.0) | 31.0 (28.6–31.2) | 28.1 (26.4–28.4) | 27.7 (26.8–28.1) |
| wasm | raw.dispatch_view | raw.dispatch_view | hist-e6ace96@0x3C00 | 133.5 (127.3–142.6) | 28.3 (27.9–28.9) | 25.7 (25.5–25.8) | 25.0 (23.8–25.5) |
| wasm | raw.dispatch_view | js.parse_view | main-local@0x3 | 25.3 (23.8–25.3) | 12.2 (12.1–12.4) | 12.4 (12.2–12.5) | 12.8 (12.7–12.8) |
| wasm | raw.dispatch_view | js.parse_view | main-local@0xC00 | 27.3 (25.6–32.4) | 15.3 (14.5–15.7) | 15.9 (15.2–16.3) | 16.7 (16.3–17.9) |
| wasm | raw.dispatch_view | js.parse_view | main-local@0x3000 | 30.7 (29.4–36.2) | 17.8 (17.5–18.5) | 18.1 (17.6–18.2) | 18.9 (18.3–19.1) |
| wasm | raw.dispatch_view | js.parse_view | main-local@0xC00000 | 27.5 (25.8–32.9) | 18.6 (15.7–18.7) | 18.7 (17.7–18.8) | 19.3 (18.7–19.7) |
| wasm | raw.dispatch_view | js.parse_view | main-local@0x3C00 | 29.5 (26.5–30.6) | 17.3 (16.7–17.7) | 17.0 (16.0–17.9) | 17.6 (17.2–18.1) |
| wasm | raw.dispatch_view | js.parse_view | hist-e6ace96@0x3 | 24.1 (23.8–25.6) | 11.9 (11.8–12.1) | 12.0 (11.9–12.1) | 12.5 (12.3–12.5) |
| wasm | raw.dispatch_view | js.parse_view | hist-e6ace96@0xC00 | 28.0 (26.4–32.3) | 15.3 (14.8–15.6) | 15.7 (14.9–16.6) | 16.1 (16.1–16.5) |
| wasm | raw.dispatch_view | js.parse_view | hist-e6ace96@0x3000 | 29.7 (24.9–31.6) | 17.3 (16.1–18.1) | 17.8 (16.5–18.1) | 18.3 (17.0–18.9) |
| wasm | raw.dispatch_view | js.parse_view | hist-e6ace96@0xC00000 | 28.2 (28.2–30.1) | 18.0 (17.1–18.1) | 18.1 (17.4–18.3) | 18.7 (18.5–18.9) |
| wasm | raw.dispatch_view | js.parse_view | hist-e6ace96@0x3C00 | 27.8 (27.7–29.5) | 16.4 (16.2–16.7) | 16.8 (16.7–16.8) | 17.1 (16.2–17.2) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local@0x3 | 126.7 (123.4–129.4) | 33.6 (33.6–33.8) | 32.0 (31.8–32.1) | 32.1 (32.0–32.1) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local@0xC00 | 147.1 (136.1–154.3) | 42.6 (40.4–42.9) | 41.5 (39.9–42.9) | 42.2 (41.3–45.8) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local@0x3000 | 184.0 (164.7–214.7) | 49.2 (48.4–51.3) | 46.9 (45.6–47.4) | 47.6 (46.1–48.6) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local@0xC00000 | 147.1 (132.1–156.8) | 50.4 (43.7–52.6) | 47.8 (46.5–49.1) | 48.0 (47.4–50.1) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local@0x3C00 | 165.9 (143.1–191.6) | 48.0 (46.4–49.3) | 44.5 (41.7–46.7) | 44.6 (43.6–46.0) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | hist-e6ace96@0x3 | 125.3 (120.7–139.1) | 32.7 (32.5–32.9) | 30.5 (30.3–30.8) | 31.0 (30.5–31.1) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | hist-e6ace96@0xC00 | 159.4 (144.6–189.0) | 41.6 (40.7–42.9) | 40.0 (38.1–42.5) | 40.0 (39.9–41.2) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | hist-e6ace96@0x3000 | 165.2 (163.4–188.2) | 47.3 (44.5–49.7) | 45.3 (42.3–46.4) | 45.7 (42.6–47.3) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | hist-e6ace96@0xC00000 | 152.6 (150.4–154.1) | 49.5 (46.2–49.7) | 46.6 (44.1–47.2) | 46.9 (45.7–47.5) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | hist-e6ace96@0x3C00 | 156.3 (154.6–164.4) | 45.1 (44.5–46.0) | 42.8 (42.7–42.9) | 42.5 (40.4–43.1) |
| wasm | abi.dispatch_view | abi.encode_args | main-local@0x3 | 0.7 (0.7–0.8) | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | main-local@0xC00 | 0.8 (0.8–0.9) | 0.3 (0.2–0.3) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | main-local@0x3000 | 0.8 (0.8–1.1) | 0.3 (0.3–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | main-local@0xC00000 | 0.8 (0.8–0.8) | 0.3 (0.3–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | main-local@0x3C00 | 0.8 (0.8–0.9) | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | hist-e6ace96@0x3 | 0.7 (0.7–0.7) | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | hist-e6ace96@0xC00 | 0.8 (0.8–1.0) | 0.2 (0.2–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | hist-e6ace96@0x3000 | 0.8 (0.7–0.9) | 0.2 (0.2–0.3) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | hist-e6ace96@0xC00000 | 0.8 (0.7–1.1) | 0.3 (0.3–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | hist-e6ace96@0x3C00 | 0.8 (0.8–0.9) | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.exec | main-local@0x3 | 94.4 (93.4–96.1) | 18.8 (18.7–18.8) | 16.6 (16.6–16.6) | 16.3 (16.2–16.4) |
| wasm | abi.dispatch_view | abi.exec | main-local@0xC00 | 124.6 (115.3–133.9) | 24.1 (23.6–27.0) | 21.9 (21.8–24.5) | 21.3 (21.1–22.7) |
| wasm | abi.dispatch_view | abi.exec | main-local@0x3000 | 123.7 (107.6–128.7) | 25.3 (25.2–28.1) | 23.0 (22.7–24.9) | 22.5 (22.3–24.6) |
| wasm | abi.dispatch_view | abi.exec | main-local@0xC00000 | 117.0 (107.9–117.3) | 27.5 (24.6–27.9) | 24.5 (23.5–25.0) | 24.7 (24.5–25.1) |
| wasm | abi.dispatch_view | abi.exec | main-local@0x3C00 | 114.8 (109.9–125.3) | 25.7 (24.1–25.9) | 23.2 (22.6–23.3) | 22.8 (21.1–22.9) |
| wasm | abi.dispatch_view | abi.exec | hist-e6ace96@0x3 | 104.6 (91.2–112.6) | 18.1 (17.9–18.3) | 15.8 (15.7–16.0) | 15.6 (15.5–15.7) |
| wasm | abi.dispatch_view | abi.exec | hist-e6ace96@0xC00 | 96.4 (94.8–113.4) | 21.3 (20.7–24.3) | 20.3 (19.6–22.0) | 20.7 (20.6–22.3) |
| wasm | abi.dispatch_view | abi.exec | hist-e6ace96@0x3000 | 114.4 (101.6–126.2) | 24.3 (23.7–26.8) | 21.7 (21.3–23.7) | 21.5 (21.2–23.6) |
| wasm | abi.dispatch_view | abi.exec | hist-e6ace96@0xC00000 | 112.2 (102.9–115.5) | 26.5 (23.5–26.5) | 23.1 (22.5–24.5) | 23.5 (23.4–23.8) |
| wasm | abi.dispatch_view | abi.exec | hist-e6ace96@0x3C00 | 116.6 (115.9–118.3) | 24.4 (21.3–25.2) | 21.9 (20.0–22.2) | 21.4 (20.9–21.6) |
| wasm | abi.dispatch_view | abi.decode | main-local@0x3 | 3.6 (3.3–3.8) | 1.9 (1.9–1.9) | 1.9 (1.8–1.9) | 1.9 (1.9–1.9) |
| wasm | abi.dispatch_view | abi.decode | main-local@0xC00 | 3.6 (3.0–4.2) | 2.4 (2.4–2.6) | 2.4 (2.4–2.5) | 2.4 (2.4–2.5) |
| wasm | abi.dispatch_view | abi.decode | main-local@0x3000 | 3.6 (3.5–4.1) | 2.5 (2.5–2.7) | 2.5 (2.5–2.7) | 2.5 (2.5–2.7) |
| wasm | abi.dispatch_view | abi.decode | main-local@0xC00000 | 3.7 (3.7–3.8) | 2.8 (2.5–2.8) | 2.6 (2.6–2.7) | 2.8 (2.8–2.8) |
| wasm | abi.dispatch_view | abi.decode | main-local@0x3C00 | 4.3 (3.3–7.8) | 2.5 (2.4–2.6) | 2.5 (2.5–2.5) | 2.5 (2.4–2.5) |
| wasm | abi.dispatch_view | abi.decode | hist-e6ace96@0x3 | 3.6 (3.6–4.8) | 1.9 (1.8–1.9) | 1.9 (1.9–1.9) | 1.9 (1.9–1.9) |
| wasm | abi.dispatch_view | abi.decode | hist-e6ace96@0xC00 | 3.4 (3.2–4.6) | 2.2 (2.1–2.5) | 2.3 (2.3–2.5) | 2.5 (2.4–2.6) |
| wasm | abi.dispatch_view | abi.decode | hist-e6ace96@0x3000 | 3.5 (3.2–4.6) | 2.5 (2.4–2.7) | 2.4 (2.4–2.7) | 2.5 (2.5–2.7) |
| wasm | abi.dispatch_view | abi.decode | hist-e6ace96@0xC00000 | 3.9 (3.4–3.9) | 2.7 (2.5–2.7) | 2.6 (2.6–2.7) | 2.8 (2.7–2.8) |
| wasm | abi.dispatch_view | abi.decode | hist-e6ace96@0x3C00 | 3.7 (3.4–4.7) | 2.5 (2.2–2.6) | 2.5 (2.3–2.5) | 2.5 (2.5–2.5) |
| wasm | abi.dispatch_view | abi.free | main-local@0x3 | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | main-local@0xC00 | 0.2 (0.1–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | main-local@0x3000 | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | main-local@0xC00000 | 0.2 (0.1–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | main-local@0x3C00 | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | hist-e6ace96@0x3 | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.0–0.1) |
| wasm | abi.dispatch_view | abi.free | hist-e6ace96@0xC00 | 0.2 (0.1–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | hist-e6ace96@0x3000 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | hist-e6ace96@0xC00000 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | hist-e6ace96@0x3C00 | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local@0x3 | 98.0 (97.8–101.2) | 21.0 (20.9–21.0) | 18.7 (18.7–18.8) | 18.4 (18.3–18.5) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local@0xC00 | 128.8 (119.6–138.1) | 26.9 (26.4–30.0) | 24.6 (24.5–27.3) | 24.0 (23.8–25.4) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local@0x3000 | 131.9 (112.2–132.7) | 28.3 (28.1–31.2) | 25.8 (25.5–27.9) | 25.4 (25.1–27.6) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local@0xC00000 | 120.8 (111.1–121.6) | 30.8 (27.6–31.1) | 27.6 (26.3–28.0) | 27.7 (27.6–28.1) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local@0x3C00 | 118.1 (115.1–130.4) | 28.7 (26.9–28.9) | 26.0 (25.3–26.1) | 25.6 (23.8–25.7) |
| wasm | abi.dispatch_view | abi.dispatch_view | hist-e6ace96@0x3 | 108.0 (95.1–123.4) | 20.3 (20.2–20.4) | 18.1 (17.9–18.2) | 17.7 (17.7–17.8) |
| wasm | abi.dispatch_view | abi.dispatch_view | hist-e6ace96@0xC00 | 101.0 (99.1–119.2) | 23.8 (23.2–27.3) | 23.0 (22.1–24.8) | 23.4 (23.4–25.2) |
| wasm | abi.dispatch_view | abi.dispatch_view | hist-e6ace96@0x3000 | 117.7 (106.1–131.5) | 27.1 (26.4–29.9) | 24.4 (24.0–26.7) | 24.3 (24.0–26.7) |
| wasm | abi.dispatch_view | abi.dispatch_view | hist-e6ace96@0xC00000 | 115.9 (107.5–120.6) | 29.6 (26.3–29.7) | 26.0 (25.4–27.6) | 26.5 (26.5–26.8) |
| wasm | abi.dispatch_view | abi.dispatch_view | hist-e6ace96@0x3C00 | 121.4 (119.7–126.7) | 27.3 (23.8–28.1) | 24.7 (22.5–25.0) | 24.2 (23.6–24.4) |
| wasm | kit | kit.dispatch | main-local@0x3 | 223.7 (216.7–232.8) | 102.5 (101.9–102.8) | 103.3 (102.4–104.8) | 107.6 (106.5–110.3) |
| wasm | kit | kit.dispatch | main-local@0xC00 | 258.6 (233.1–275.1) | 131.2 (124.4–135.2) | 133.0 (128.8–139.3) | 139.0 (136.5–139.2) |
| wasm | kit | kit.dispatch | main-local@0x3000 | 275.3 (245.2–302.2) | 137.8 (136.5–146.5) | 139.5 (136.9–148.8) | 145.6 (142.8–155.1) |
| wasm | kit | kit.dispatch | main-local@0xC00000 | 252.5 (247.6–279.7) | 148.9 (147.6–150.9) | 153.3 (152.1–155.0) | 159.3 (156.6–159.7) |
| wasm | kit | kit.dispatch | main-local@0x3C00 | 265.6 (255.1–300.8) | 137.9 (127.9–138.4) | 138.2 (127.8–140.1) | 144.2 (136.4–144.7) |
| wasm | kit | kit.view | main-local@0x3 | 35.0 (31.8–40.4) | 21.1 (20.8–21.2) | 21.7 (21.4–21.9) | 22.6 (22.2–22.9) |
| wasm | kit | kit.view | main-local@0xC00 | 36.2 (33.9–39.3) | 27.0 (25.8–27.6) | 27.9 (27.2–29.1) | 28.9 (28.6–29.0) |
| wasm | kit | kit.view | main-local@0x3000 | 36.7 (36.3–40.7) | 28.4 (27.9–30.0) | 29.3 (28.4–30.9) | 30.5 (29.6–32.4) |
| wasm | kit | kit.view | main-local@0xC00000 | 45.6 (35.1–51.7) | 30.9 (30.0–31.0) | 32.1 (31.7–32.3) | 33.2 (32.6–33.6) |
| wasm | kit | kit.view | main-local@0x3C00 | 37.6 (35.9–44.4) | 28.3 (26.3–28.4) | 28.4 (26.7–29.2) | 29.7 (28.2–30.1) |
| wasm | kit | kit.dispatch+view | main-local@0x3 | 264.1 (251.2–271.1) | 124.1 (123.1–124.4) | 125.5 (124.3–127.3) | 130.6 (129.0–133.6) |
| wasm | kit | kit.dispatch+view | main-local@0xC00 | 314.6 (269.3–324.2) | 158.7 (150.4–163.2) | 161.4 (156.4–168.6) | 168.3 (165.4–168.3) |
| wasm | kit | kit.dispatch+view | main-local@0x3000 | 324.9 (277.8–357.4) | 166.6 (164.8–176.8) | 169.3 (165.8–180.0) | 176.5 (172.7–187.7) |
| wasm | kit | kit.dispatch+view | main-local@0xC00000 | 292.2 (291.0–324.4) | 180.2 (178.1–182.5) | 186.2 (184.1–188.0) | 192.8 (189.5–193.5) |
| wasm | kit | kit.dispatch+view | main-local@0x3C00 | 308.8 (296.3–354.8) | 166.7 (154.5–167.1) | 167.0 (154.7–169.5) | 174.3 (164.7–175.0) |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | main-local@0x3 | main-local@0xC00 | main-local@0x3000 | main-local@0xC00000 | main-local@0x3C00 | hist-e6ace96@0x3 | hist-e6ace96@0xC00 | hist-e6ace96@0x3000 | hist-e6ace96@0xC00000 | hist-e6ace96@0x3C00 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| native | web.dispatch_view | web.dispatch_view | 48.2 (47.4–50.4) · p95 108.6 | 60.8 (59.0–61.0) · p95 137.0 | 64.1 (63.7–71.2) · p95 144.9 | 68.0 (67.0–68.8) · p95 153.1 | 63.1 (59.4–63.7) · p95 141.4 | - | - | - | - | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.5 (0.5–0.5) · p95 1.6 | 0.6 (0.5–0.6) · p95 1.8 | 0.6 (0.6–0.6) · p95 1.9 | 0.6 (0.6–0.7) · p95 2.1 | 0.6 (0.6–0.6) · p95 1.8 | - | - | - | - | - |
| wasm | raw.dispatch_view | raw.dispatch_view | 59.9 (59.4–60.1) · p95 122.6 | 74.0 (72.0–75.0) · p95 152.8 | 77.9 (77.8–81.7) · p95 159.5 | 82.8 (80.9–84.9) · p95 170.9 | 76.2 (73.2–77.5) · p95 155.8 | - | - | - | - | - |
| wasm | raw.dispatch_view | js.parse_view | 20.7 (20.6–21.0) · p95 26.0 | 25.8 (25.2–26.4) · p95 32.7 | 27.5 (27.1–28.5) · p95 34.4 | 29.3 (28.3–29.7) · p95 36.9 | 26.6 (25.4–27.0) · p95 32.6 | - | - | - | - | - |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 98.4 (97.8–98.7) · p95 148.3 | 122.1 (119.5–124.4) · p95 186.2 | 130.0 (130.0–136.5) · p95 193.0 | 138.1 (135.2–141.2) · p95 207.8 | 127.7 (121.0–129.8) · p95 188.1 | - | - | - | - | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.3 (0.3–0.4) · p95 0.9 | 0.4 (0.4–0.4) · p95 1.1 | 0.4 (0.4–0.4) · p95 1.1 | 0.4 (0.4–0.5) · p95 1.2 | 0.4 (0.4–0.4) · p95 1.1 | - | - | - | - | - |
| wasm | abi.dispatch_view | abi.exec | 57.4 (57.3–58.1) · p95 118.4 | 71.1 (71.0–71.2) · p95 150.5 | 72.9 (72.7–81.9) · p95 154.0 | 78.1 (77.8–79.9) · p95 164.5 | 74.1 (70.9–75.0) · p95 156.3 | - | - | - | - | - |
| wasm | abi.dispatch_view | abi.decode | 1.0 (1.0–1.0) · p95 3.6 | 1.2 (1.2–1.3) · p95 3.8 | 1.3 (1.3–1.4) · p95 4.1 | 1.4 (1.4–1.4) · p95 4.7 | 1.3 (1.2–1.3) · p95 4.4 | - | - | - | - | - |
| wasm | abi.dispatch_view | abi.free | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | - | - | - | - | - |
| wasm | abi.dispatch_view | abi.dispatch_view | 77.5 (76.4–77.7) · p95 123.9 | 96.4 (95.9–96.7) · p95 156.1 | 100.5 (100.4–111.7) · p95 158.9 | 106.1 (105.9–107.1) · p95 170.8 | 101.4 (96.2–102.3) · p95 161.7 | - | - | - | - | - |
| wasm | kit | kit.dispatch | 182.4 (182.0–182.7) · p95 263.0 | 222.4 (221.7–230.6) · p95 317.5 | 238.8 (231.7–254.5) · p95 339.9 | 249.4 (245.7–250.2) · p95 355.8 | 231.1 (230.9–233.3) · p95 324.8 | - | - | - | - | - |
| wasm | kit | kit.view | 27.6 (27.5–27.7) · p95 35.6 | 34.4 (34.3–35.5) · p95 44.7 | 36.4 (35.5–39.1) · p95 48.5 | 38.8 (38.0–38.9) · p95 50.4 | 36.1 (35.4–36.5) · p95 46.4 | - | - | - | - | - |
| wasm | kit | kit.dispatch+view | 213.4 (213.1–213.6) · p95 297.9 | 258.7 (258.6–267.7) · p95 360.2 | 278.7 (271.4–297.3) · p95 384.5 | 290.9 (287.7–292.3) · p95 404.4 | 270.2 (269.4–273.4) · p95 369.2 | - | - | - | - | - |

