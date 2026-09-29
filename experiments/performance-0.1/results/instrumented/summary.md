# Caveat performance baseline: 2026-09-29T08-41-36-021-instrumented

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=instrumented --repeats=3 --build=never --keep-samples --target=main-local=tree:C:/Users/walte/AppData/Local/Temp/claude/C--Dev-GPT-SandBox-Web/3bc468a2-40f8-4895-bd0a-fb452fae7549/scratchpad/prPERF --target=i1=tree:C:/Users/walte/AppData/Local/Temp/claude/C--Dev-GPT-SandBox-Web/3bc468a2-40f8-4895-bd0a-fb452fae7549/scratchpad/perf-scratch/m/i1 --target=i2=tree:C:/Users/walte/AppData/Local/Temp/claude/C--Dev-GPT-SandBox-Web/3bc468a2-40f8-4895-bd0a-fb452fae7549/scratchpad/perf-scratch/m/i2 --out=C:/Users/walte/AppData/Local/Temp/claude/C--Dev-GPT-SandBox-Web/3bc468a2-40f8-4895-bd0a-fb452fae7549/scratchpad/audit/instrumented-rerun`

## Method

- Suite `instrumented`: 3 repeat(s) of every mode, each in a fresh process; within a repeat every target runs a mode back to back, the first target rotating.
- Per-event modes: 1 untimed warm-up pass(es), then 3 timed pass(es), each episode on a freshly opened session. `published-method` runs 3 round(s) with no warm-up, as the published harness did.
- Load, save and restore: 3 untimed then 30 timed calls; published resume method: 3 run(s).
- Statistic: quantile q(p) = sorted[floor(p·(n−1))] over the pooled samples of a run (the published rule; the median is the lower median). A cell is the median over runs of each run's pooled median, with the lowest and highest run in brackets, then the median over runs of each run's p95.
- Units: microseconds per operation unless the operation says otherwise.

## Targets

| Target | Kind | Revision | Reactive WebAssembly sha256 | Glue sha256 | Kit session.mjs sha256 | Native benchmark |
| --- | --- | --- | --- | --- | --- | --- |
| main-local | tree | v0.1.0-rc.4-7-gd9c9358 (d9c9358) | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |
| i1 | tree | n/a | `0368965f71b7` (2054637 B, reused) | `9a13a11a5dbc` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |
| i2 | tree | n/a | `2ee34de9ffab` (2057339 B, reused) | `337f81b667b4` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |

## Environment

- CPU: Intel(R) Core(TM) Ultra 9 275HX; 24 cores, 24 logical processors; performance cores 0, 1, 10, 11, 12, 13, 22, 23
- Memory: 63.4 GiB
- OS: Microsoft Windows 11 Home 10.0.26200 (build 26200)
- Power: Power Scheme GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c  (High performance); battery status 2 (2 = on mains power)
- Pinning: affinity 0x3C00 (logical processors 10, 11, 12, 13), priority high, via cmd /c start /b /wait /<priority> /affinity <mask>
- Node v24.11.1, V8 13.6.233.10-node.28, npm 11.12.1
- Rust: rustc 1.98.1 (48a229cea 2026-09-01) (LLVM version: 22.1.8); cargo 1.98.1 (797e8a9bc 2026-08-05); 1.98.1-x86_64-pc-windows-msvc (overridden by 'C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF\rust-toolchain.toml')
- wasm-bindgen 0.2.104; wasm-opt not installed (the build does not use it)
- Load before: total CPU 18/3/11%; busiest over the window: claude 1.172s, powershell 0.625s, claude 0.406s, claude 0.234s, ChatGPT 0.203s, ChatGPT 0.156s
- Load after: total CPU 14/17/0%; busiest over the window: claude 1.906s, claude 0.859s, powershell 0.688s, ChatGPT 0.234s, ChatGPT 0.172s, Reallusion Hub 0.172s
- Load during: typeperf every 5 s for the whole run, 259 samples: total mean 9.1% p95 11.1% max 12.8%; processor 10 mean 75.9% p95 99.1% max 99.7%; processor 11 mean 24.4% p95 52.2% max 76.6%; processor 12 mean 20.1% p95 44.5% max 68.2%; processor 13 mean 3.4% p95 12.8% max 29.8%; total above 10% in 44 and above 25% in 0 samples; pinned processors summing above 150% in 8

## Published reference

- Dispatch + view, Glowcap replay: median 51.6 µs, p95 59.1 µs, max 646.7 µs (experiments/glowcap/evidence/round6-replay.json timing.dispatchAndShippedOutput); runtime 8d8596a, reactive WebAssembly `d897787f414a`, repository e6ace96.
- Resume ten minutes of play: median 3.2416 ms of runs 7.154, 3.2416, 3.1401 ms; save 2045 bytes (experiments/glowcap/evidence/round6-replay.json timing.resume (caveat5)).

## Final-state consistency

Within each target, every run of every mode and engine ended each workload in the same saved state (390 runs compared).

Between targets, by workload and kind of state (targets in one group ended in identical states):

- glowcap-replay save-text: main-local, i1, i2 `fda3d6109816`
- glowcap-replay first-episode-save-text: main-local, i1, i2 `fda3d6109816`
- glowcap-unbound save-text: main-local, i1, i2 `9271bb69a75b`
- glowcap-resume first-episode-save-text: main-local, i1, i2 `bc6187eccca8`
- ledger-session save-text: main-local, i1, i2 `895698524f68`
- ledger-session first-episode-save-text: main-local, i1, i2 `895698524f68`
- trail-rescue-scenarios save-text: main-local, i1, i2 `bb72a4e76921`
- trail-rescue-scenarios first-episode-save-text: main-local, i1, i2 `5e76ed5e8ca7`

## One Glowcap event + view, piece by piece

Medians over runs of each run's median on the steady idle ticks (events 1300–9999), where the pooled median sits; each row is its own process, so parts need not sum exactly to the whole.

| Piece | main-local | i1 | i2 |
| --- | --- | --- | --- |
| Published method: adapter dispatch + view | - | - | - |
| Adapter dispatch (stringify, WebAssembly dispatch_view, JSON.parse) | - | - | - |
| Adapter view() (JavaScript reshaping only) | - | - | - |
| JSON.stringify(payload) | 0.3 | 0.3 | - |
| wasm-bindgen dispatch_view call, returning the view text | 26.0 | 25.3 | - |
|   arguments copied into WebAssembly memory | 0.1 | 0.1 | - |
|   WebAssembly execution (resolve, apply, view, serialize) | 21.3 | 21.4 | - |
|   result decoded to a JavaScript string | 2.4 | 2.5 | - |
|   result freed | 0.1 | 0.1 | - |
|   the four pieces, timed together in one process | 24.0 | 24.1 | - |
| JSON.parse(view text) | 17.0 | 16.8 | - |
| WebAssembly view() alone, execution only | 8.8 | 8.3 | - |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | - | - | - |
| Kit dispatch + view() | - | - | - |
| Native web::dispatch_view (the exported function, natively) | 20.1 | 20.1 | - |
| Native apply (numeric parameters: rules and bindings only) | 13.0 | 13.1 | - |
| Native dispatch_view_json (resolve + apply + view, not serialized) | - | - | - |
| Native view() build | 2.5 | 2.4 | - |
| Native view serialize | 4.6 | 4.6 | - |
| Native snapshot() build | 24.6 | 24.2 | - |
| Native session clone (upper bound of the transaction copy) | 7.0 | 7.2 | - |
| Native apply, the same program without bindings (glowcap-unbound) | 11.9 | 12.1 | - |
| *derived:* native resolve_payload ≈ dispatch_view_json − apply − view | - | - | - |
| *derived:* native binding evaluation ≈ apply − apply without bindings | 1.1 | 1.0 | - |
| *derived:* WebAssembly ÷ native, same exported function (ratio) | 1.1 | 1.1 | - |

## glowcap-replay

The Glowcap replay program and the exact event stream behind the published 51.6 us figure (experiments/glowcap/harness.mjs bench(), runtime/examples/profile_dispatch.rs): absorb cave glowcap, absorb pool duskcap, taste ruin glowcap, then 9,997 ticks of dt 0.05. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | main-local | i1 | i2 |
| --- | --- | --- | --- | --- | --- |
| native | apply | apply | 13.0 (13.0–14.6) · p95 16.6 | 13.1 (13.0–13.2) · p95 15.9 | - |
| native | web.dispatch_view | web.dispatch_view | 20.1 (19.9–20.6) · p95 22.7 | 20.2 (20.1–20.6) · p95 24.5 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | 63.3 (63.0–64.5) · p95 69.1 | 64.4 (62.5–69.7) · p95 72.5 | - |
| native | read | clone | 6.9 (6.9–7.2) · p95 7.5 | 7.1 (7.0–7.6) · p95 7.6 | - |
| native | read | clone.drop | 3.8 (3.8–3.9) · p95 4.1 | 4.0 (3.9–4.2) · p95 4.2 | - |
| native | read | save | 8.7 (8.7–8.8) · p95 9.5 | 8.8 (8.7–9.5) · p95 9.5 | - |
| native | read | save_json | 14.4 (14.4–14.6) · p95 15.6 | 14.6 (14.5–15.7) · p95 15.6 | - |
| native | read | snapshot | 24.5 (24.4–24.5) · p95 26.3 | 24.1 (24.0–26.2) · p95 25.8 | - |
| native | read | snapshot.drop | 10.6 (10.5–10.7) · p95 11.3 | 10.8 (10.5–11.4) · p95 11.5 | - |
| native | read | snapshot.serialize | 14.6 (14.6–14.7) · p95 15.9 | 14.6 (14.6–15.9) · p95 15.8 | - |
| native | read | snapshot.serialize_pretty | 31.0 (30.9–31.1) · p95 33.4 | 30.8 (30.7–33.5) · p95 32.9 | - |
| native | read | view | 2.4 (2.4–2.4) · p95 2.7 | 2.4 (2.4–2.6) · p95 2.6 | - |
| native | read | view.serialize | 4.6 (4.6–4.7) · p95 5.2 | 4.6 (4.6–4.9) · p95 5.0 | - |
| native | web.read | web.save | 15.6 (15.5–16.3) · p95 18.1 | 17.0 (15.6–17.9) · p95 18.4 | - |
| native | web.read | web.snapshot | 65.9 (65.4–69.5) · p95 76.7 | 71.2 (64.8–74.7) · p95 76.6 | - |
| native | web.read | web.view | 6.4 (6.4–6.6) · p95 7.3 | 6.8 (6.3–7.1) · p95 7.4 | - |
| native | lifecycle | from_source | 2599.6 (2468.5–2661.9) · p95 2736.1 | 2576.6 (2534.7–2584.2) · p95 2675.1 | - |
| native | lifecycle | restore | 2705.2 (2611.3–2825.4) · p95 2848.0 | 2728.4 (2671.1–2751.5) · p95 2853.1 | - |
| native | lifecycle | restore.parse | 28.7 (25.4–28.7) · p95 38.7 | 28.9 (27.5–30.4) · p95 38.2 | - |
| native | lifecycle | restore_json | 2751.4 (2665.6–2846.6) · p95 2850.8 | 2724.1 (2702.5–2782.3) · p95 2893.1 | - |
| native | lifecycle | save | 26.8 (22.9–28.0) · p95 33.6 | 26.3 (24.5–28.5) · p95 38.1 | - |
| native | lifecycle | save_json | 28.7 (23.6–30.5) · p95 33.5 | 30.8 (24.2–34.2) · p95 39.8 | - |
| native | lifecycle | web.new | 2586.7 (2475.2–2735.3) · p95 2722.7 | 2531.8 (2529.5–2540.6) · p95 2723.8 | - |
| native | lifecycle | web.restore | 2795.1 (2670.1–2849.1) · p95 2952.3 | 2746.9 (2694.5–2782.0) · p95 2879.0 | - |
| native | lifecycle | web.save | 20.8 (19.0–21.4) · p95 27.9 | 19.8 (19.1–20.9) · p95 24.9 | - |
| native | bench.dispatch_only | bench.dispatch_only | - | 13.4 (13.3–15.8) · p95 16.1 | - |
| native | bench.read | bench.snapshot_build | - | 34.5 (33.1–36.2) · p95 39.3 | - |
| native | bench.read | bench.view_build | - | 3.8 (3.6–4.0) · p95 4.4 | - |
| native | bench.read | web.snapshot | - | 65.4 (63.5–68.9) · p95 74.6 | - |
| native | bench.read | web.view | - | 8.3 (8.0–8.7) · p95 9.5 | - |
| native | probe.dispatch_view | apply.clone | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.dispatch_view | apply.swap | - | - | 1.1 (1.1–1.1) · p95 1.2 |
| native | probe.dispatch_view | apply.validate | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | bind.eval | - | - | 0.8 (0.8–0.8) · p95 0.9 |
| native | probe.dispatch_view | bind.explain | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_view | bind.none | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.dispatch_view | bind.sort | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | bind.stale | - | - | 1.1 (1.1–1.1) · p95 1.3 |
| native | probe.dispatch_view | bind.write | - | - | 0.8 (0.8–0.8) · p95 0.9 |
| native | probe.dispatch_view | exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.dispatch_view | payload.parse | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_view | payload.resolve | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.dispatch_view | run.changes | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.dispatch_view | run.clock | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | run.prologue | - | - | 0.1 (0.1–0.2) · p95 0.2 |
| native | probe.dispatch_view | run.rules | - | - | 10.5 (10.4–10.6) · p95 12.2 |
| native | probe.dispatch_view | total | - | - | 20.7 (20.5–20.8) · p95 23.8 |
| native | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.commitments | - | - | 0.4 (0.4–0.5) · p95 0.5 |
| native | probe.dispatch_view | view.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.dispatch_view | view.names | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| native | probe.dispatch_view | view.relations | - | - | 1.2 (1.2–1.2) · p95 1.4 |
| native | probe.dispatch_view | view.sort | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.dispatch_view | web.drop | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.dispatch_view | web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.dispatch_view | web.serialize | - | - | 3.4 (3.3–3.4) · p95 3.8 |
| native | probe.dispatch_outcome | apply.clone | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.dispatch_outcome | apply.swap | - | - | 1.2 (1.2–1.2) · p95 1.3 |
| native | probe.dispatch_outcome | apply.validate | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | bind.eval | - | - | 0.9 (0.9–1.0) · p95 1.1 |
| native | probe.dispatch_outcome | bind.explain | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| native | probe.dispatch_outcome | bind.none | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.dispatch_outcome | bind.sort | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | bind.stale | - | - | 1.1 (1.1–1.1) · p95 1.4 |
| native | probe.dispatch_outcome | bind.write | - | - | 1.3 (1.3–1.3) · p95 1.4 |
| native | probe.dispatch_outcome | exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.dispatch_outcome | outcome.built | - | - | 0.1 (0.1–0.1) · p95 0.3 |
| native | probe.dispatch_outcome | payload.parse | - | - | 0.2 (0.2–0.3) · p95 0.4 |
| native | probe.dispatch_outcome | payload.resolve | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_outcome | run.changes | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| native | probe.dispatch_outcome | run.clock | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | run.prologue | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_outcome | run.rules | - | - | 11.3 (11.3–11.3) · p95 13.1 |
| native | probe.dispatch_outcome | snap.build | - | - | 0.1 (0.1–0.2) · p95 0.2 |
| native | probe.dispatch_outcome | snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | snap.names | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| native | probe.dispatch_outcome | snap.records | - | - | 3.7 (3.6–3.8) · p95 4.1 |
| native | probe.dispatch_outcome | snap.relations | - | - | 1.4 (1.4–1.4) · p95 1.6 |
| native | probe.dispatch_outcome | snap.shown | - | - | 6.3 (6.3–6.4) · p95 6.8 |
| native | probe.dispatch_outcome | snap.sort | - | - | 0.8 (0.8–0.9) · p95 1.0 |
| native | probe.dispatch_outcome | snap.static | - | - | 1.9 (1.9–1.9) · p95 2.1 |
| native | probe.dispatch_outcome | snap.symbols | - | - | 3.6 (3.5–3.6) · p95 4.0 |
| native | probe.dispatch_outcome | snap.values | - | - | 5.6 (5.6–5.7) · p95 6.2 |
| native | probe.dispatch_outcome | total | - | - | 64.6 (64.2–64.8) · p95 70.3 |
| native | probe.dispatch_outcome | web.drop | - | - | 10.2 (10.2–10.4) · p95 11.5 |
| native | probe.dispatch_outcome | web.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | web.serialize | - | - | 14.4 (14.4–14.6) · p95 15.9 |
| native | probe.read | save:exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | save:save.build | - | - | 8.9 (8.8–8.9) · p95 9.8 |
| native | probe.read | save:save.drop | - | - | 2.9 (2.9–3.0) · p95 3.3 |
| native | probe.read | save:save.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | save:save.serialize | - | - | 4.2 (4.1–4.2) · p95 4.6 |
| native | probe.read | save:total | - | - | 16.1 (15.9–16.2) · p95 17.7 |
| native | probe.read | save:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:exit | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.read | snapshot:snap.build | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.read | snapshot:snap.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:snap.names | - | - | 0.3 (0.3–0.4) · p95 0.5 |
| native | probe.read | snapshot:snap.records | - | - | 3.9 (3.8–3.9) · p95 4.4 |
| native | probe.read | snapshot:snap.relations | - | - | 1.4 (1.4–1.5) · p95 1.6 |
| native | probe.read | snapshot:snap.shown | - | - | 6.5 (6.4–6.8) · p95 7.2 |
| native | probe.read | snapshot:snap.sort | - | - | 0.5 (0.5–0.6) · p95 0.6 |
| native | probe.read | snapshot:snap.static | - | - | 2.1 (2.0–2.1) · p95 2.2 |
| native | probe.read | snapshot:snap.symbols | - | - | 3.6 (3.6–3.7) · p95 4.1 |
| native | probe.read | snapshot:snap.values | - | - | 6.1 (6.1–6.2) · p95 6.8 |
| native | probe.read | snapshot:total | - | - | 67.4 (66.3–67.5) · p95 73.0 |
| native | probe.read | snapshot:web.drop | - | - | 10.6 (10.5–10.7) · p95 11.5 |
| native | probe.read | snapshot:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:web.serialize | - | - | 31.6 (31.2–31.8) · p95 34.8 |
| native | probe.read | view:exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:total | - | - | 6.6 (6.6–6.8) · p95 7.2 |
| native | probe.read | view:view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.read | view:view.commitments | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| native | probe.read | view:view.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:view.names | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.read | view:view.relations | - | - | 1.2 (1.2–1.2) · p95 1.4 |
| native | probe.read | view:view.sort | - | - | 0.6 (0.5–0.6) · p95 0.7 |
| native | probe.read | view:web.drop | - | - | 0.6 (0.6–0.6) · p95 0.6 |
| native | probe.read | view:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:web.serialize | - | - | 3.3 (3.3–3.4) · p95 3.7 |
| native | probe.lifecycle | new:bind.eval | - | - | 26.0 (25.6–26.3) · p95 26.6 |
| native | probe.lifecycle | new:bind.explain | - | - | 3.2 (3.2–3.3) · p95 3.5 |
| native | probe.lifecycle | new:bind.sort | - | - | 2.7 (2.7–2.7) · p95 2.9 |
| native | probe.lifecycle | new:bind.stale | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.lifecycle | new:bind.write | - | - | 9.9 (9.8–9.9) · p95 10.4 |
| native | probe.lifecycle | new:exit | - | - | 154.8 (145.3–154.8) · p95 172.4 |
| native | probe.lifecycle | new:load.bindings | - | - | 0.6 (0.6–0.6) · p95 0.6 |
| native | probe.lifecycle | new:load.declare | - | - | 278.7 (221.4–285.6) · p95 305.1 |
| native | probe.lifecycle | new:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.lifecycle | new:load.expand | - | - | 430.3 (412.5–431.2) · p95 462.6 |
| native | probe.lifecycle | new:load.map | - | - | 30.8 (24.2–30.8) · p95 34.5 |
| native | probe.lifecycle | new:load.parse | - | - | 1051.2 (1047.4–1097.4) · p95 1126.7 |
| native | probe.lifecycle | new:load.procedures | - | - | 232.5 (210.1–258.4) · p95 260.5 |
| native | probe.lifecycle | new:load.validate | - | - | 509.1 (505.8–512.4) · p95 529.6 |
| native | probe.lifecycle | new:total | - | - | 2718.9 (2698.4–2757.1) · p95 2853.2 |
| native | probe.lifecycle | restore:exit | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.lifecycle | restore:load.bindings | - | - | 49.0 (47.3–53.0) · p95 56.1 |
| native | probe.lifecycle | restore:load.declare | - | - | 221.1 (202.9–284.7) · p95 256.4 |
| native | probe.lifecycle | restore:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.lifecycle | restore:load.expand | - | - | 405.1 (394.9–415.4) · p95 451.1 |
| native | probe.lifecycle | restore:load.map | - | - | 20.6 (20.2–20.6) · p95 21.8 |
| native | probe.lifecycle | restore:load.parse | - | - | 1040.3 (1026.9–1071.0) · p95 1116.7 |
| native | probe.lifecycle | restore:load.procedures | - | - | 230.4 (214.4–237.3) · p95 260.0 |
| native | probe.lifecycle | restore:load.validate | - | - | 509.8 (507.0–553.6) · p95 569.9 |
| native | probe.lifecycle | restore:restore.bindings | - | - | 90.4 (90.1–96.0) · p95 95.7 |
| native | probe.lifecycle | restore:restore.done | - | - | 0.2 (0.2–0.3) · p95 0.3 |
| native | probe.lifecycle | restore:restore.drop_save | - | - | 5.4 (5.1–5.4) · p95 7.3 |
| native | probe.lifecycle | restore:restore.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.lifecycle | restore:restore.graph | - | - | 9.6 (9.5–9.8) · p95 10.2 |
| native | probe.lifecycle | restore:restore.header | - | - | 0.8 (0.8–0.8) · p95 0.9 |
| native | probe.lifecycle | restore:restore.loaded | - | - | 137.4 (132.5–148.1) · p95 160.4 |
| native | probe.lifecycle | restore:restore.parse | - | - | 31.9 (26.5–33.3) · p95 37.4 |
| native | probe.lifecycle | restore:restore.records | - | - | 26.3 (26.1–27.6) · p95 28.6 |
| native | probe.lifecycle | restore:restore.states | - | - | 7.8 (7.6–7.9) · p95 8.6 |
| native | probe.lifecycle | restore:total | - | - | 2801.7 (2760.2–2924.4) · p95 3038.1 |
| native | probe.lifecycle | restore:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.overhead | probe.mark | - | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | probe.overhead | probe.now | - | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.3–0.3) · p95 0.5 | - |
| wasm | raw.dispatch_view | raw.dispatch_view | 26.1 (24.3–27.2) · p95 30.5 | 25.3 (24.0–25.7) · p95 30.8 | - |
| wasm | raw.dispatch_view | js.parse_view | 17.0 (16.1–17.9) · p95 19.5 | 16.6 (16.0–17.1) · p95 19.5 | - |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 43.6 (40.8–45.5) · p95 49.9 | 42.5 (40.3–43.3) · p95 50.7 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 73.1 (68.8–77.5) · p95 89.6 | 73.1 (70.2–73.6) · p95 85.8 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 64.3 (61.2–68.0) · p95 74.9 | 64.1 (61.5–64.2) · p95 71.8 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 137.6 (130.3–146.1) · p95 162.6 | 137.7 (132.1–137.7) · p95 156.7 | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | - |
| wasm | abi.dispatch_view | abi.exec | 21.5 (21.4–21.7) · p95 26.7 | 21.7 (20.7–22.5) · p95 27.8 | - |
| wasm | abi.dispatch_view | abi.decode | 2.4 (2.4–2.5) · p95 3.2 | 2.5 (2.4–2.5) · p95 3.2 | - |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | - |
| wasm | abi.dispatch_view | abi.dispatch_view | 24.1 (24.0–24.5) · p95 30.1 | 24.5 (23.3–25.3) · p95 31.2 | - |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.1 (0.1–0.2) · p95 0.3 | 0.2 (0.2–0.2) · p95 0.4 | - |
| wasm | abi.dispatch_outcome | abi.exec | 57.5 (56.9–58.6) · p95 70.8 | 59.8 (57.9–60.4) · p95 69.9 | - |
| wasm | abi.dispatch_outcome | abi.decode | 7.3 (7.3–7.5) · p95 10.3 | 7.7 (7.4–7.7) · p95 10.4 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 65.3 (64.7–66.7) · p95 80.9 | 68.1 (66.0–68.6) · p95 81.0 | - |
| wasm | read | raw.view | 11.1 (10.9–11.4) · p95 13.1 | 11.0 (10.7–11.2) · p95 12.7 | - |
| wasm | read | js.parse_view | 18.4 (18.1–18.7) · p95 21.4 | 18.2 (17.8–18.6) · p95 20.6 | - |
| wasm | read | raw.snapshot | 92.0 (88.8–94.1) · p95 110.9 | 90.0 (87.4–93.3) · p95 104.6 | - |
| wasm | read | js.parse_snapshot | 69.7 (68.4–71.9) · p95 81.9 | 68.9 (67.2–70.8) · p95 77.7 | - |
| wasm | read | raw.save | 18.9 (18.3–19.4) · p95 23.8 | 18.7 (18.3–19.0) · p95 22.3 | - |
| wasm | abi.read | abi.view.exec | 8.7 (8.4–8.9) · p95 10.1 | 8.2 (8.2–8.4) · p95 9.5 | - |
| wasm | abi.read | abi.view.decode | 2.4 (2.2–2.4) · p95 2.8 | 2.2 (2.2–2.3) · p95 2.8 | - |
| wasm | abi.read | abi.snapshot.exec | 73.6 (70.5–75.2) · p95 86.9 | 70.2 (68.3–70.3) · p95 82.2 | - |
| wasm | abi.read | abi.snapshot.decode | 12.6 (11.9–12.6) · p95 16.7 | 12.0 (11.9–12.4) · p95 16.2 | - |
| wasm | abi.read | abi.save.exec | 15.8 (15.1–16.5) · p95 19.9 | 15.1 (14.6–15.2) · p95 18.6 | - |
| wasm | abi.read | abi.save.decode | 0.6 (0.6–0.6) · p95 0.8 | 0.6 (0.6–0.6) · p95 0.8 | - |
| wasm | lifecycle | raw.new | 2537.7 (2504.5–2570.9) · p95 2828.8 | 2500.6 (2426.7–2550.9) · p95 2902.6 | - |
| wasm | lifecycle | raw.save | 45.1 (44.7–46.6) · p95 62.7 | 48.7 (45.8–49.1) · p95 62.4 | - |
| wasm | lifecycle | raw.restore | 2566.7 (2547.4–2642.3) · p95 2911.1 | 2563.7 (2510.7–2666.6) · p95 3140.5 | - |
| wasm | lifecycle | kit.open | 2446.7 (2395.9–2455.4) · p95 2589.3 | 2409.5 (2323.6–2472.6) · p95 2827.5 | - |
| wasm | lifecycle | kit.save | 90.4 (85.8–91.8) · p95 130.0 | 91.3 (85.7–94.9) · p95 132.8 | - |
| wasm | lifecycle | kit.restore | 2549.6 (2538.2–2634.4) · p95 2764.8 | 2568.5 (2496.6–2592.0) · p95 2862.2 | - |
| wasm | abi.dispatch_only | abi.encode_args | - | 0.1 (0.1–0.1) · p95 0.3 | - |
| wasm | abi.dispatch_only | abi.dispatch_only.exec | - | 13.0 (12.9–13.1) · p95 18.5 | - |
| wasm | abi.dispatch_only | abi.dispatch_only | - | 13.1 (13.0–13.2) · p95 18.7 | - |
| wasm | abi.bench_read | abi.view_build.exec | - | 3.5 (3.3–3.5) · p95 4.1 | - |
| wasm | abi.bench_read | abi.snapshot_build.exec | - | 29.3 (27.7–30.4) · p95 35.4 | - |
| wasm | abi.bench_read | abi.view.exec | - | 9.2 (8.9–9.2) · p95 11.0 | - |
| wasm | abi.bench_read | abi.view.decode | - | 2.4 (2.3–2.4) · p95 3.0 | - |
| wasm | abi.bench_read | abi.snapshot.exec | - | 71.4 (70.0–71.6) · p95 83.7 | - |
| wasm | abi.bench_read | abi.snapshot.decode | - | 12.0 (11.8–12.4) · p95 16.2 | - |
| wasm | probe.dispatch_view | total | - | - | 43.2 (42.6–43.6) · p95 54.5 |
| wasm | probe.dispatch_view | web.enter | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.dispatch_view | payload.parse | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_view | payload.resolve | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_view | apply.validate | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | apply.clone | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | run.clock | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | run.prologue | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | run.rules | - | - | 11.0 (10.9–11.0) · p95 13.4 |
| wasm | probe.dispatch_view | run.changes | - | - | 0.6 (0.6–0.6) · p95 0.7 |
| wasm | probe.dispatch_view | bind.stale | - | - | 1.5 (1.5–1.5) · p95 1.8 |
| wasm | probe.dispatch_view | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | bind.eval | - | - | 0.9 (0.9–0.9) · p95 1.2 |
| wasm | probe.dispatch_view | bind.sort | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | bind.explain | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_view | bind.write | - | - | 0.7 (0.7–0.7) · p95 1.0 |
| wasm | probe.dispatch_view | apply.swap | - | - | 0.7 (0.7–0.8) · p95 1.0 |
| wasm | probe.dispatch_view | view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | view.names | - | - | 0.5 (0.5–0.6) · p95 0.7 |
| wasm | probe.dispatch_view | view.sort | - | - | 0.6 (0.6–0.6) · p95 0.7 |
| wasm | probe.dispatch_view | view.commitments | - | - | 0.6 (0.6–0.6) · p95 0.7 |
| wasm | probe.dispatch_view | view.relations | - | - | 1.0 (1.0–1.0) · p95 1.3 |
| wasm | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | web.serialize | - | - | 5.4 (5.3–5.5) · p95 6.4 |
| wasm | probe.dispatch_view | web.drop | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| wasm | probe.dispatch_view | exit | - | - | 2.5 (2.5–2.5) · p95 5.1 |
| wasm | probe.dispatch_view | js.parse | - | - | 16.7 (16.2–16.9) · p95 22.0 |
| wasm | probe.dispatch_outcome | total | - | - | 137.1 (134.8–137.8) · p95 163.7 |
| wasm | probe.dispatch_outcome | web.enter | - | - | 0.2 (0.2–0.2) · p95 0.5 |
| wasm | probe.dispatch_outcome | payload.parse | - | - | 0.3 (0.2–0.3) · p95 0.5 |
| wasm | probe.dispatch_outcome | payload.resolve | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | apply.validate | - | - | 0.1 (0.1–0.2) · p95 0.2 |
| wasm | probe.dispatch_outcome | apply.clone | - | - | 0.2 (0.1–0.2) · p95 0.2 |
| wasm | probe.dispatch_outcome | run.clock | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_outcome | run.prologue | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | run.rules | - | - | 11.5 (11.2–11.6) · p95 14.1 |
| wasm | probe.dispatch_outcome | run.changes | - | - | 0.6 (0.6–0.6) · p95 0.8 |
| wasm | probe.dispatch_outcome | bind.stale | - | - | 1.5 (1.5–1.6) · p95 1.9 |
| wasm | probe.dispatch_outcome | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_outcome | bind.eval | - | - | 1.0 (1.0–1.0) · p95 1.4 |
| wasm | probe.dispatch_outcome | bind.sort | - | - | 0.1 (0.1–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | bind.explain | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | bind.write | - | - | 0.8 (0.8–0.9) · p95 1.2 |
| wasm | probe.dispatch_outcome | apply.swap | - | - | 1.0 (1.0–1.0) · p95 1.4 |
| wasm | probe.dispatch_outcome | snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | snap.names | - | - | 0.5 (0.5–0.6) · p95 0.7 |
| wasm | probe.dispatch_outcome | snap.sort | - | - | 0.7 (0.7–0.7) · p95 1.0 |
| wasm | probe.dispatch_outcome | snap.symbols | - | - | 2.8 (2.8–2.8) · p95 3.5 |
| wasm | probe.dispatch_outcome | snap.shown | - | - | 4.9 (4.8–4.9) · p95 6.0 |
| wasm | probe.dispatch_outcome | snap.values | - | - | 4.3 (4.3–4.4) · p95 5.3 |
| wasm | probe.dispatch_outcome | snap.records | - | - | 2.4 (2.4–2.4) · p95 3.0 |
| wasm | probe.dispatch_outcome | snap.static | - | - | 1.3 (1.2–1.3) · p95 1.7 |
| wasm | probe.dispatch_outcome | snap.relations | - | - | 1.1 (1.1–1.1) · p95 1.4 |
| wasm | probe.dispatch_outcome | snap.build | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | outcome.built | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_outcome | web.serialize | - | - | 20.0 (19.5–20.0) · p95 23.6 |
| wasm | probe.dispatch_outcome | web.drop | - | - | 11.4 (11.1–11.5) · p95 14.7 |
| wasm | probe.dispatch_outcome | exit | - | - | 7.8 (7.8–7.9) · p95 11.2 |
| wasm | probe.dispatch_outcome | js.parse | - | - | 62.5 (61.6–63.1) · p95 74.3 |
| wasm | probe.read | view:total | - | - | 29.8 (29.7–31.1) · p95 35.0 |
| wasm | probe.read | view:web.enter | - | - | 0.1 (0.1–0.1) · p95 0.3 |
| wasm | probe.read | view:view.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| wasm | probe.read | view:view.names | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| wasm | probe.read | view:view.sort | - | - | 0.6 (0.6–0.7) · p95 0.8 |
| wasm | probe.read | view:view.commitments | - | - | 0.6 (0.6–0.7) · p95 0.8 |
| wasm | probe.read | view:view.relations | - | - | 1.2 (1.2–1.3) · p95 1.6 |
| wasm | probe.read | view:view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | view:web.serialize | - | - | 5.2 (5.1–5.3) · p95 6.0 |
| wasm | probe.read | view:web.drop | - | - | 0.6 (0.6–0.7) · p95 1.0 |
| wasm | probe.read | view:exit | - | - | 2.5 (2.4–2.6) · p95 3.4 |
| wasm | probe.read | view:js.parse | - | - | 18.2 (18.2–19.1) · p95 21.3 |
| wasm | probe.read | snapshot:total | - | - | 164.5 (162.4–170.8) · p95 191.5 |
| wasm | probe.read | snapshot:web.enter | - | - | 0.2 (0.1–0.2) · p95 0.3 |
| wasm | probe.read | snapshot:snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | snapshot:snap.names | - | - | 0.5 (0.5–0.6) · p95 0.7 |
| wasm | probe.read | snapshot:snap.sort | - | - | 0.8 (0.8–0.9) · p95 1.1 |
| wasm | probe.read | snapshot:snap.symbols | - | - | 3.9 (3.8–4.0) · p95 4.7 |
| wasm | probe.read | snapshot:snap.shown | - | - | 5.6 (5.6–5.8) · p95 6.5 |
| wasm | probe.read | snapshot:snap.values | - | - | 5.2 (5.2–5.5) · p95 6.6 |
| wasm | probe.read | snapshot:snap.records | - | - | 2.5 (2.5–2.6) · p95 3.0 |
| wasm | probe.read | snapshot:snap.static | - | - | 1.3 (1.3–1.3) · p95 1.6 |
| wasm | probe.read | snapshot:snap.relations | - | - | 1.1 (1.1–1.2) · p95 1.4 |
| wasm | probe.read | snapshot:snap.build | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.read | snapshot:web.serialize | - | - | 45.0 (43.7–46.7) · p95 53.2 |
| wasm | probe.read | snapshot:web.drop | - | - | 15.1 (15.1–15.6) · p95 18.2 |
| wasm | probe.read | snapshot:exit | - | - | 12.5 (12.3–12.8) · p95 16.0 |
| wasm | probe.read | snapshot:js.parse | - | - | 69.9 (69.6–72.9) · p95 81.4 |
| wasm | probe.read | save:total | - | - | 19.6 (19.6–20.2) · p95 24.4 |
| wasm | probe.read | save:web.enter | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.read | save:save.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | save:save.build | - | - | 8.6 (8.6–8.9) · p95 10.6 |
| wasm | probe.read | save:save.serialize | - | - | 5.6 (5.5–5.8) · p95 7.0 |
| wasm | probe.read | save:save.drop | - | - | 4.4 (4.3–4.4) · p95 5.3 |
| wasm | probe.read | save:exit | - | - | 0.9 (0.9–0.9) · p95 1.1 |
| wasm | probe.lifecycle | restore:total | - | - | 2891.4 (2708.8–2934.2) · p95 3440.2 |
| wasm | probe.lifecycle | restore:web.enter | - | - | 18.9 (17.1–19.5) · p95 24.2 |
| wasm | probe.lifecycle | restore:restore.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.lifecycle | restore:restore.parse | - | - | 36.3 (35.5–40.9) · p95 58.1 |
| wasm | probe.lifecycle | restore:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.lifecycle | restore:load.parse | - | - | 1118.6 (1083.7–1143.2) · p95 1302.0 |
| wasm | probe.lifecycle | restore:load.expand | - | - | 318.8 (292.1–319.5) · p95 375.7 |
| wasm | probe.lifecycle | restore:load.map | - | - | 31.1 (30.6–32.1) · p95 40.7 |
| wasm | probe.lifecycle | restore:load.declare | - | - | 139.6 (129.0–140.7) · p95 160.6 |
| wasm | probe.lifecycle | restore:load.procedures | - | - | 122.2 (119.5–128.0) · p95 136.7 |
| wasm | probe.lifecycle | restore:load.validate | - | - | 592.7 (567.4–593.9) · p95 703.2 |
| wasm | probe.lifecycle | restore:load.bindings | - | - | 52.4 (49.2–53.0) · p95 72.2 |
| wasm | probe.lifecycle | restore:restore.loaded | - | - | 198.9 (179.3–203.1) · p95 229.8 |
| wasm | probe.lifecycle | restore:restore.header | - | - | 1.0 (1.0–1.2) · p95 1.7 |
| wasm | probe.lifecycle | restore:restore.graph | - | - | 14.9 (14.0–16.0) · p95 18.3 |
| wasm | probe.lifecycle | restore:restore.states | - | - | 10.6 (9.9–10.6) · p95 12.7 |
| wasm | probe.lifecycle | restore:restore.records | - | - | 40.6 (36.8–43.2) · p95 48.7 |
| wasm | probe.lifecycle | restore:restore.bindings | - | - | 121.6 (114.9–124.5) · p95 140.0 |
| wasm | probe.lifecycle | restore:restore.done | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.lifecycle | restore:restore.drop_save | - | - | 8.2 (7.5–8.2) · p95 8.9 |
| wasm | probe.lifecycle | restore:exit | - | - | 4.3 (4.2–5.8) · p95 18.3 |
| wasm | probe.lifecycle | new:total | - | - | 2616.5 (2455.1–2640.0) · p95 2956.3 |
| wasm | probe.lifecycle | new:bind.stale | - | - | 0.3 (0.2–0.3) · p95 0.5 |
| wasm | probe.lifecycle | new:bind.eval | - | - | 32.1 (30.2–33.0) · p95 39.8 |
| wasm | probe.lifecycle | new:bind.sort | - | - | 3.8 (3.5–3.9) · p95 4.5 |
| wasm | probe.lifecycle | new:bind.explain | - | - | 3.9 (3.6–4.0) · p95 4.6 |
| wasm | probe.lifecycle | new:bind.write | - | - | 11.4 (10.7–11.7) · p95 14.5 |
| wasm | probe.lifecycle | new:load.enter | - | - | 14.8 (13.5–16.0) · p95 21.8 |
| wasm | probe.lifecycle | new:load.parse | - | - | 1126.2 (1059.9–1132.7) · p95 1296.9 |
| wasm | probe.lifecycle | new:load.expand | - | - | 316.9 (295.0–321.5) · p95 356.7 |
| wasm | probe.lifecycle | new:load.map | - | - | 32.0 (29.8–34.3) · p95 37.6 |
| wasm | probe.lifecycle | new:load.declare | - | - | 142.6 (131.1–144.5) · p95 164.7 |
| wasm | probe.lifecycle | new:load.procedures | - | - | 125.8 (114.8–127.5) · p95 139.8 |
| wasm | probe.lifecycle | new:load.validate | - | - | 598.8 (551.6–600.0) · p95 643.7 |
| wasm | probe.lifecycle | new:load.bindings | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| wasm | probe.lifecycle | new:exit | - | - | 202.5 (193.7–208.7) · p95 232.5 |
| wasm | probe.overhead | probe.mark | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| wasm | probe.overhead | js.performance_now | - | - | 0.0 (0.0–0.0) · p95 0.1 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | main-local | 83.1 (78.7–84.7) | 16.8 (16.1–18.3) | 13.5 (13.2–15.2) | 13.0 (13.0–14.6) |
| native | apply | apply | i1 | 78.6 (78.4–78.9) | 16.1 (16.1–16.3) | 13.4 (13.4–13.7) | 13.1 (13.0–13.2) |
| native | web.dispatch_view | web.dispatch_view | main-local | 110.9 (109.4–114.3) | 22.6 (22.3–23.6) | 20.1 (20.0–20.6) | 20.1 (19.9–20.5) |
| native | web.dispatch_view | web.dispatch_view | i1 | 113.5 (108.8–114.0) | 22.4 (22.3–22.8) | 20.1 (20.0–20.2) | 20.1 (20.0–20.6) |
| native | web.dispatch_outcome | web.dispatch_outcome | main-local | 149.1 (148.3–154.1) | 61.6 (61.4–62.8) | 61.9 (60.8–61.9) | 63.4 (63.2–64.7) |
| native | web.dispatch_outcome | web.dispatch_outcome | i1 | 158.0 (153.5–164.3) | 62.3 (60.9–67.6) | 62.0 (60.4–67.5) | 64.6 (62.7–69.9) |
| native | read | clone | main-local | 6.8 (6.6–6.9) | 6.0 (6.0–6.1) | 6.3 (6.2–6.3) | 7.0 (6.9–7.2) |
| native | read | clone | i1 | 7.3 (7.0–7.4) | 6.1 (6.0–6.6) | 6.3 (6.3–6.8) | 7.2 (7.1–7.6) |
| native | read | clone.drop | main-local | 3.7 (3.6–3.7) | 3.3 (3.3–3.5) | 3.4 (3.4–3.6) | 3.8 (3.8–4.0) |
| native | read | clone.drop | i1 | 3.7 (3.7–4.0) | 3.5 (3.4–3.8) | 3.6 (3.5–3.9) | 4.0 (3.9–4.2) |
| native | read | save | main-local | 5.7 (5.6–5.8) | 6.2 (6.2–6.3) | 8.3 (8.3–8.4) | 8.8 (8.8–8.8) |
| native | read | save | i1 | 5.7 (5.7–5.8) | 6.2 (6.2–6.8) | 8.4 (8.4–9.1) | 8.8 (8.8–9.6) |
| native | read | save_json | main-local | 9.4 (9.3–9.7) | 10.5 (10.5–10.5) | 13.9 (13.9–14.1) | 14.5 (14.4–14.6) |
| native | read | save_json | i1 | 9.0 (9.0–9.8) | 10.4 (10.4–11.3) | 14.2 (14.1–15.2) | 14.7 (14.5–15.8) |
| native | read | snapshot | main-local | 28.5 (28.3–32.5) | 21.5 (21.4–21.5) | 23.1 (23.0–23.4) | 24.6 (24.5–24.6) |
| native | read | snapshot | i1 | 30.8 (27.1–32.6) | 21.3 (21.2–23.3) | 23.1 (23.0–25.0) | 24.2 (24.1–26.4) |
| native | read | snapshot.drop | main-local | 10.3 (10.1–10.3) | 9.3 (9.1–9.4) | 9.9 (9.8–9.9) | 10.6 (10.5–10.7) |
| native | read | snapshot.drop | i1 | 10.5 (10.2–10.9) | 9.4 (9.2–10.1) | 10.1 (10.0–10.8) | 10.8 (10.5–11.5) |
| native | read | snapshot.serialize | main-local | 16.7 (16.4–17.4) | 13.3 (13.1–13.4) | 14.2 (14.2–14.3) | 14.7 (14.7–14.8) |
| native | read | snapshot.serialize | i1 | 17.8 (17.5–19.3) | 13.1 (13.1–14.3) | 14.3 (14.2–15.4) | 14.7 (14.6–15.9) |
| native | read | snapshot.serialize_pretty | main-local | 32.7 (32.6–32.8) | 28.7 (28.3–28.8) | 30.2 (30.1–30.4) | 31.0 (30.9–31.2) |
| native | read | snapshot.serialize_pretty | i1 | 33.3 (32.5–34.7) | 28.2 (28.2–30.8) | 30.2 (30.2–32.6) | 30.8 (30.8–33.7) |
| native | read | view | main-local | 2.2 (2.2–2.3) | 1.9 (1.9–1.9) | 2.2 (2.2–2.3) | 2.5 (2.4–2.5) |
| native | read | view | i1 | 2.2 (2.0–2.2) | 1.9 (1.8–2.0) | 2.3 (2.2–2.4) | 2.4 (2.4–2.6) |
| native | read | view.serialize | main-local | 6.2 (6.1–6.3) | 4.3 (4.2–4.3) | 4.4 (4.3–4.4) | 4.6 (4.6–4.7) |
| native | read | view.serialize | i1 | 6.7 (6.0–7.5) | 4.2 (4.2–4.5) | 4.4 (4.4–4.7) | 4.6 (4.6–5.0) |
| native | web.read | web.save | main-local | 11.3 (11.0–11.7) | 11.6 (11.5–12.3) | 15.1 (15.1–15.3) | 15.6 (15.6–16.4) |
| native | web.read | web.save | i1 | 12.6 (11.7–12.7) | 12.6 (11.6–13.5) | 16.4 (15.2–17.4) | 17.1 (15.6–18.0) |
| native | web.read | web.snapshot | main-local | 79.0 (77.5–83.6) | 58.6 (58.3–62.0) | 63.2 (62.8–64.4) | 66.2 (65.6–70.0) |
| native | web.read | web.snapshot | i1 | 81.4 (75.7–83.3) | 62.9 (57.6–67.6) | 67.7 (62.5–72.0) | 71.5 (64.9–75.2) |
| native | web.read | web.view | main-local | 6.2 (6.1–6.2) | 5.4 (5.3–5.6) | 6.0 (5.9–6.0) | 6.4 (6.4–6.7) |
| native | web.read | web.view | i1 | 6.7 (6.2–6.7) | 5.7 (5.3–6.1) | 6.4 (5.9–6.8) | 6.9 (6.3–7.2) |
| native | bench.dispatch_only | bench.dispatch_only | i1 | 88.3 (84.8–97.7) | 16.4 (16.2–19.1) | 13.7 (13.6–16.2) | 13.3 (13.3–15.7) |
| native | bench.read | bench.snapshot_build | i1 | 37.3 (36.4–39.3) | 29.6 (29.1–31.4) | 31.9 (30.9–32.9) | 34.8 (33.3–36.4) |
| native | bench.read | bench.view_build | i1 | 3.9 (3.7–4.0) | 3.0 (2.9–3.1) | 3.5 (3.4–3.5) | 3.9 (3.7–4.0) |
| native | bench.read | web.snapshot | i1 | 69.2 (67.1–69.8) | 57.1 (56.5–60.7) | 61.4 (60.5–63.4) | 66.0 (63.8–69.3) |
| native | bench.read | web.view | i1 | 10.1 (9.6–10.1) | 7.0 (6.8–7.3) | 7.6 (7.5–7.8) | 8.4 (8.0–8.7) |
| native | probe.dispatch_view | apply.clone | i2 | 0.5 (0.5–0.6) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| native | probe.dispatch_view | apply.swap | i2 | 2.9 (2.8–2.9) | 1.1 (1.1–1.2) | 1.0 (1.0–1.0) | 1.1 (1.1–1.1) |
| native | probe.dispatch_view | apply.validate | i2 | 0.3 (0.3–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | bind.eval | i2 | 31.0 (30.4–31.8) | 0.8 (0.8–0.8) | 54.6 (52.9–55.6) | - |
| native | probe.dispatch_view | bind.explain | i2 | 5.3 (5.1–5.5) | 0.2 (0.2–0.2) | 6.4 (6.3–7.2) | - |
| native | probe.dispatch_view | bind.none | i2 | - | - | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.dispatch_view | bind.sort | i2 | 1.0 (1.0–1.0) | 0.1 (0.1–0.1) | 0.9 (0.9–0.9) | - |
| native | probe.dispatch_view | bind.stale | i2 | 1.1 (1.1–1.2) | 1.1 (1.1–1.1) | 1.1 (1.1–1.1) | 1.1 (1.1–1.1) |
| native | probe.dispatch_view | bind.write | i2 | 5.5 (5.4–5.7) | 0.8 (0.8–0.8) | 6.0 (5.8–6.2) | - |
| native | probe.dispatch_view | exit | i2 | 0.1 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.dispatch_view | payload.parse | i2 | 1.3 (1.2–1.3) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_view | payload.resolve | i2 | 0.6 (0.5–0.6) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_view | run.changes | i2 | 0.6 (0.5–0.6) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| native | probe.dispatch_view | run.clock | i2 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | run.prologue | i2 | 0.3 (0.3–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.2) | 0.1 (0.1–0.2) |
| native | probe.dispatch_view | run.rules | i2 | 27.6 (26.8–27.9) | 12.0 (12.0–12.1) | 11.1 (11.0–11.1) | 10.4 (10.3–10.5) |
| native | probe.dispatch_view | total | i2 | 95.2 (92.8–95.7) | 22.8 (22.6–23.1) | 20.7 (20.5–20.7) | 20.6 (20.4–20.8) |
| native | probe.dispatch_view | view.build | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | view.commitments | i2 | 0.8 (0.7–0.8) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) | 0.4 (0.4–0.5) |
| native | probe.dispatch_view | view.enter | i2 | 0.1 (0.1–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.dispatch_view | view.names | i2 | 0.4 (0.4–0.5) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| native | probe.dispatch_view | view.relations | i2 | 1.1 (1.1–1.1) | 1.0 (1.0–1.0) | 1.1 (1.1–1.1) | 1.2 (1.2–1.2) |
| native | probe.dispatch_view | view.sort | i2 | 1.2 (1.0–1.2) | 0.4 (0.4–0.4) | 0.5 (0.5–0.5) | 0.5 (0.5–0.6) |
| native | probe.dispatch_view | web.drop | i2 | 0.6 (0.6–0.6) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) |
| native | probe.dispatch_view | web.enter | i2 | 0.0 (0.0–0.1) | 0.0 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.dispatch_view | web.serialize | i2 | 6.7 (6.6–6.8) | 3.1 (3.0–3.1) | 3.2 (3.2–3.3) | 3.4 (3.3–3.4) |
| native | probe.dispatch_outcome | apply.clone | i2 | 0.5 (0.5–0.6) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| native | probe.dispatch_outcome | apply.swap | i2 | 2.8 (2.8–2.9) | 1.3 (1.3–1.3) | 1.1 (1.1–1.2) | 1.2 (1.2–1.2) |
| native | probe.dispatch_outcome | apply.validate | i2 | 0.3 (0.3–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_outcome | bind.eval | i2 | 32.0 (31.2–32.5) | 0.9 (0.9–1.0) | 62.0 (60.1–65.6) | - |
| native | probe.dispatch_outcome | bind.explain | i2 | 5.8 (5.0–5.8) | 0.2 (0.2–0.2) | 6.0 (5.5–6.6) | - |
| native | probe.dispatch_outcome | bind.none | i2 | - | - | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.dispatch_outcome | bind.sort | i2 | 1.0 (1.0–1.0) | 0.1 (0.1–0.1) | 1.1 (1.0–1.1) | - |
| native | probe.dispatch_outcome | bind.stale | i2 | 1.2 (1.2–1.2) | 1.2 (1.2–1.2) | 1.1 (1.1–1.2) | 1.1 (1.1–1.1) |
| native | probe.dispatch_outcome | bind.write | i2 | 5.3 (5.2–5.5) | 1.3 (1.2–1.3) | 6.3 (6.2–6.3) | - |
| native | probe.dispatch_outcome | exit | i2 | 0.1 (0.1–0.1) | 0.0 (0.0–0.1) | 0.0 (0.0–0.1) | 0.0 (0.0–0.0) |
| native | probe.dispatch_outcome | outcome.built | i2 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_outcome | payload.parse | i2 | 1.6 (1.4–1.6) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.3) |
| native | probe.dispatch_outcome | payload.resolve | i2 | 0.6 (0.6–0.7) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_outcome | run.changes | i2 | 0.6 (0.5–0.6) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) |
| native | probe.dispatch_outcome | run.clock | i2 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_outcome | run.prologue | i2 | 0.4 (0.3–0.4) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_outcome | run.rules | i2 | 28.1 (27.4–28.2) | 12.9 (12.8–13.0) | 11.7 (11.7–11.9) | 11.3 (11.2–11.3) |
| native | probe.dispatch_outcome | snap.build | i2 | 0.2 (0.2–0.2) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.2) |
| native | probe.dispatch_outcome | snap.enter | i2 | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_outcome | snap.names | i2 | 0.4 (0.4–0.5) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.4 (0.4–0.4) |
| native | probe.dispatch_outcome | snap.records | i2 | 2.7 (2.6–2.8) | 2.7 (2.5–2.7) | 3.6 (3.6–3.7) | 3.8 (3.6–3.8) |
| native | probe.dispatch_outcome | snap.relations | i2 | 1.1 (1.1–1.2) | 1.1 (1.1–1.1) | 1.3 (1.2–1.3) | 1.4 (1.4–1.4) |
| native | probe.dispatch_outcome | snap.shown | i2 | 6.3 (6.2–6.4) | 5.8 (5.6–5.9) | 5.7 (5.6–5.7) | 6.3 (6.3–6.4) |
| native | probe.dispatch_outcome | snap.sort | i2 | 1.1 (1.1–1.2) | 0.7 (0.7–0.7) | 0.8 (0.8–0.8) | 0.8 (0.8–0.9) |
| native | probe.dispatch_outcome | snap.static | i2 | 2.3 (2.3–2.3) | 1.9 (1.9–1.9) | 1.9 (1.9–1.9) | 1.9 (1.9–1.9) |
| native | probe.dispatch_outcome | snap.symbols | i2 | 4.0 (3.8–4.0) | 3.1 (3.0–3.1) | 3.4 (3.3–3.4) | 3.6 (3.6–3.6) |
| native | probe.dispatch_outcome | snap.values | i2 | 7.2 (7.2–7.7) | 5.4 (5.4–5.4) | 5.4 (5.4–5.4) | 5.6 (5.6–5.7) |
| native | probe.dispatch_outcome | total | i2 | 141.5 (138.7–142.0) | 63.1 (62.1–63.2) | 61.6 (61.1–62.8) | 64.8 (64.5–65.0) |
| native | probe.dispatch_outcome | web.drop | i2 | 10.3 (10.1–10.3) | 9.1 (8.9–9.1) | 9.5 (9.4–9.8) | 10.3 (10.2–10.5) |
| native | probe.dispatch_outcome | web.enter | i2 | 0.1 (0.0–0.1) | 0.0 (0.0–0.1) | 0.1 (0.0–0.1) | 0.0 (0.0–0.1) |
| native | probe.dispatch_outcome | web.serialize | i2 | 17.6 (17.4–17.8) | 13.1 (13.0–13.2) | 14.0 (13.8–14.1) | 14.5 (14.4–14.7) |
| native | probe.read | save:exit | i2 | 0.1 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | save:save.build | i2 | 5.9 (5.7–6.0) | 6.2 (6.1–6.3) | 8.2 (8.1–8.3) | 8.9 (8.8–9.0) |
| native | probe.read | save:save.drop | i2 | 1.8 (1.8–1.9) | 2.1 (2.1–2.1) | 2.8 (2.8–2.8) | 2.9 (2.9–3.0) |
| native | probe.read | save:save.enter | i2 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | save:save.serialize | i2 | 3.8 (3.6–3.9) | 3.4 (3.4–3.5) | 4.0 (4.0–4.1) | 4.2 (4.1–4.2) |
| native | probe.read | save:total | i2 | 11.4 (11.2–11.5) | 11.9 (11.8–12.0) | 15.3 (15.3–15.5) | 16.2 (16.0–16.4) |
| native | probe.read | save:web.enter | i2 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | snapshot:exit | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.read | snapshot:snap.build | i2 | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.read | snapshot:snap.enter | i2 | 0.1 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | snapshot:snap.names | i2 | 0.4 (0.4–0.4) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.4 (0.3–0.4) |
| native | probe.read | snapshot:snap.records | i2 | 2.6 (2.6–2.7) | 2.7 (2.7–2.8) | 3.7 (3.7–3.8) | 3.9 (3.8–3.9) |
| native | probe.read | snapshot:snap.relations | i2 | 1.0 (1.0–1.1) | 1.1 (1.1–1.1) | 1.3 (1.3–1.3) | 1.5 (1.4–1.5) |
| native | probe.read | snapshot:snap.shown | i2 | 6.7 (6.5–6.7) | 5.8 (5.8–6.1) | 5.7 (5.7–6.1) | 6.6 (6.4–6.8) |
| native | probe.read | snapshot:snap.sort | i2 | 0.5 (0.4–0.5) | 0.4 (0.4–0.4) | 0.5 (0.5–0.5) | 0.5 (0.5–0.6) |
| native | probe.read | snapshot:snap.static | i2 | 2.3 (2.3–2.3) | 2.0 (2.0–2.0) | 2.0 (2.0–2.1) | 2.1 (2.0–2.1) |
| native | probe.read | snapshot:snap.symbols | i2 | 3.8 (3.8–5.8) | 3.1 (3.1–3.2) | 3.4 (3.4–3.5) | 3.7 (3.6–3.7) |
| native | probe.read | snapshot:snap.values | i2 | 7.2 (6.9–7.6) | 5.7 (5.7–5.8) | 5.8 (5.8–5.9) | 6.2 (6.1–6.3) |
| native | probe.read | snapshot:total | i2 | 68.9 (66.9–70.0) | 59.3 (59.2–60.5) | 63.3 (63.1–64.7) | 67.7 (66.6–68.2) |
| native | probe.read | snapshot:web.drop | i2 | 10.3 (10.1–10.3) | 9.2 (9.2–9.3) | 9.7 (9.7–9.9) | 10.6 (10.5–10.8) |
| native | probe.read | snapshot:web.enter | i2 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | snapshot:web.serialize | i2 | 32.6 (32.3–32.9) | 28.7 (28.7–29.1) | 30.4 (30.3–30.8) | 31.7 (31.4–32.1) |
| native | probe.read | view:exit | i2 | 0.0 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | view:total | i2 | 6.5 (6.4–6.7) | 5.7 (5.6–5.7) | 6.2 (6.2–6.2) | 6.6 (6.6–6.9) |
| native | probe.read | view:view.build | i2 | 0.1 (0.0–0.1) | 0.1 (0.1–0.1) | 0.1 (0.0–0.1) | 0.1 (0.1–0.1) |
| native | probe.read | view:view.commitments | i2 | 0.4 (0.4–0.5) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) | 0.4 (0.4–0.5) |
| native | probe.read | view:view.enter | i2 | 0.0 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | view:view.names | i2 | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| native | probe.read | view:view.relations | i2 | 0.8 (0.8–0.9) | 0.9 (0.9–1.0) | 1.1 (1.1–1.1) | 1.2 (1.2–1.3) |
| native | probe.read | view:view.sort | i2 | 0.6 (0.5–0.6) | 0.4 (0.4–0.4) | 0.5 (0.5–0.5) | 0.6 (0.5–0.6) |
| native | probe.read | view:web.drop | i2 | 0.4 (0.4–0.4) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) | 0.6 (0.6–0.6) |
| native | probe.read | view:web.enter | i2 | 0.0 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | view:web.serialize | i2 | 3.6 (3.6–3.7) | 3.0 (3.0–3.0) | 3.1 (3.1–3.1) | 3.3 (3.3–3.4) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local | 1.2 (0.9–1.2) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | i1 | 0.9 (0.9–1.2) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local | 119.4 (119.1–139.4) | 29.7 (29.6–32.2) | 26.0 (25.4–26.2) | 26.0 (24.2–27.1) |
| wasm | raw.dispatch_view | raw.dispatch_view | i1 | 133.3 (130.7–134.5) | 27.8 (27.3–28.3) | 25.3 (24.7–25.8) | 25.3 (23.9–25.6) |
| wasm | raw.dispatch_view | js.parse_view | main-local | 31.9 (31.7–33.9) | 16.7 (16.6–18.1) | 16.3 (16.0–16.3) | 17.0 (16.1–18.0) |
| wasm | raw.dispatch_view | js.parse_view | i1 | 32.5 (28.0–32.7) | 16.2 (15.6–16.3) | 16.3 (15.6–16.6) | 16.8 (16.0–17.1) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local | 171.9 (164.8–172.3) | 46.9 (46.6–50.7) | 42.7 (41.7–42.8) | 43.5 (40.6–45.5) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | i1 | 172.5 (159.9–177.5) | 44.4 (43.5–45.1) | 41.9 (40.8–42.8) | 42.5 (40.2–43.2) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | main-local | 186.5 (185.3–192.9) | 71.5 (67.8–73.6) | 72.5 (66.5–74.1) | 73.2 (68.9–77.9) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | i1 | 189.1 (171.3–195.9) | 71.1 (67.7–72.4) | 70.3 (68.5–74.2) | 73.4 (70.4–73.7) |
| wasm | raw.dispatch_outcome | js.parse_outcome | main-local | 76.4 (75.3–83.9) | 59.6 (56.5–61.2) | 63.2 (58.9–65.2) | 64.9 (61.4–68.4) |
| wasm | raw.dispatch_outcome | js.parse_outcome | i1 | 77.9 (74.3–89.5) | 59.2 (56.8–60.0) | 61.6 (59.9–63.8) | 64.4 (61.8–64.6) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | main-local | 265.3 (262.9–269.0) | 131.7 (124.6–135.5) | 136.3 (126.0–139.7) | 138.1 (130.7–146.8) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | i1 | 273.4 (246.6–277.7) | 130.3 (124.9–132.9) | 132.5 (128.9–138.3) | 138.0 (132.5–138.5) |
| wasm | abi.dispatch_view | abi.encode_args | main-local | 0.8 (0.7–0.9) | 0.2 (0.2–0.3) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | i1 | 0.8 (0.8–0.9) | 0.3 (0.2–0.3) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.exec | main-local | 116.7 (100.9–120.5) | 24.0 (23.2–25.6) | 22.0 (21.5–22.7) | 21.3 (21.2–21.4) |
| wasm | abi.dispatch_view | abi.exec | i1 | 105.9 (103.5–111.7) | 26.1 (24.8–26.4) | 23.1 (22.1–24.2) | 21.4 (20.6–22.1) |
| wasm | abi.dispatch_view | abi.decode | main-local | 3.5 (3.3–4.0) | 2.5 (2.3–2.6) | 2.4 (2.4–2.5) | 2.4 (2.4–2.5) |
| wasm | abi.dispatch_view | abi.decode | i1 | 3.7 (3.3–3.9) | 2.6 (2.5–2.6) | 2.5 (2.4–2.6) | 2.5 (2.3–2.5) |
| wasm | abi.dispatch_view | abi.free | main-local | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | i1 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local | 122.0 (104.1–128.9) | 26.9 (25.9–28.5) | 24.8 (24.1–25.4) | 24.0 (23.9–24.0) |
| wasm | abi.dispatch_view | abi.dispatch_view | i1 | 109.2 (107.6–116.6) | 29.2 (27.6–29.3) | 25.9 (24.8–27.1) | 24.1 (23.2–24.9) |
| wasm | abi.dispatch_outcome | abi.encode_args | main-local | 1.0 (0.8–1.0) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.encode_args | i1 | 0.9 (0.8–0.9) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) | 0.2 (0.1–0.2) |
| wasm | abi.dispatch_outcome | abi.exec | main-local | 159.8 (151.9–164.5) | 57.1 (56.3–59.8) | 57.4 (56.2–57.5) | 57.6 (56.9–58.6) |
| wasm | abi.dispatch_outcome | abi.exec | i1 | 162.0 (150.0–182.5) | 60.4 (55.3–67.8) | 58.0 (55.6–62.0) | 59.6 (58.2–60.3) |
| wasm | abi.dispatch_outcome | abi.decode | main-local | 7.0 (6.8–7.3) | 6.7 (6.5–6.9) | 7.2 (7.1–7.2) | 7.4 (7.3–7.6) |
| wasm | abi.dispatch_outcome | abi.decode | i1 | 7.3 (7.2–8.0) | 7.0 (6.6–7.7) | 7.4 (7.0–7.7) | 7.7 (7.5–7.7) |
| wasm | abi.dispatch_outcome | abi.free | main-local | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.free | i1 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | main-local | 167.2 (159.5–173.0) | 64.3 (63.4–67.4) | 65.1 (63.9–65.1) | 65.4 (64.7–66.8) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | i1 | 170.7 (157.2–190.8) | 68.0 (62.4–76.2) | 65.9 (63.2–70.4) | 68.0 (66.4–68.5) |
| wasm | read | raw.view | main-local | 11.2 (10.1–12.1) | 9.7 (9.5–9.9) | 10.5 (10.3–10.6) | 11.2 (10.9–11.6) |
| wasm | read | raw.view | i1 | 11.4 (10.9–11.4) | 9.9 (9.4–10.7) | 10.6 (9.9–11.6) | 11.1 (10.7–11.2) |
| wasm | read | js.parse_view | main-local | 29.8 (25.9–34.2) | 17.2 (17.2–17.6) | 17.4 (17.4–18.1) | 18.5 (18.1–19.0) |
| wasm | read | js.parse_view | i1 | 29.9 (26.8–30.5) | 17.4 (16.9–19.0) | 17.5 (17.1–19.3) | 18.3 (17.8–18.4) |
| wasm | read | raw.snapshot | main-local | 97.4 (92.9–99.5) | 82.3 (80.0–83.5) | 87.3 (82.8–88.7) | 92.8 (89.3–95.6) |
| wasm | read | raw.snapshot | i1 | 100.0 (99.3–106.3) | 82.0 (78.4–90.9) | 85.1 (81.1–95.1) | 90.6 (87.7–93.3) |
| wasm | read | js.parse_snapshot | main-local | 74.9 (71.6–80.7) | 62.1 (61.6–64.0) | 67.0 (65.8–68.0) | 70.1 (68.7–73.2) |
| wasm | read | js.parse_snapshot | i1 | 77.9 (74.9–96.1) | 62.7 (60.4–69.3) | 66.4 (64.5–73.4) | 69.3 (67.4–70.6) |
| wasm | read | raw.save | main-local | 15.6 (14.9–17.0) | 14.1 (13.7–14.1) | 16.7 (16.6–17.4) | 19.1 (18.4–19.7) |
| wasm | read | raw.save | i1 | 16.1 (15.9–17.0) | 13.9 (13.4–15.5) | 17.0 (16.6–18.5) | 18.8 (18.4–19.2) |
| wasm | abi.read | abi.view.exec | main-local | 8.9 (8.9–9.1) | 7.7 (7.5–7.8) | 8.2 (8.1–8.2) | 8.8 (8.5–9.0) |
| wasm | abi.read | abi.view.exec | i1 | 9.3 (9.0–9.6) | 7.4 (7.2–7.5) | 8.2 (7.8–8.3) | 8.3 (8.2–8.4) |
| wasm | abi.read | abi.view.decode | main-local | 2.0 (1.9–2.0) | 2.2 (2.2–2.3) | 2.3 (2.3–2.4) | 2.4 (2.2–2.4) |
| wasm | abi.read | abi.view.decode | i1 | 1.9 (1.8–2.0) | 2.2 (2.2–2.3) | 2.3 (2.2–2.4) | 2.3 (2.2–2.3) |
| wasm | abi.read | abi.snapshot.exec | main-local | 82.5 (81.8–86.1) | 66.1 (65.3–69.2) | 70.4 (69.7–71.2) | 74.7 (70.6–75.6) |
| wasm | abi.read | abi.snapshot.exec | i1 | 86.8 (85.0–88.8) | 64.8 (63.0–65.3) | 69.6 (67.1–71.1) | 70.4 (68.3–70.5) |
| wasm | abi.read | abi.snapshot.decode | main-local | 9.3 (9.0–9.6) | 10.9 (10.6–11.0) | 11.9 (11.4–12.0) | 12.6 (11.9–12.8) |
| wasm | abi.read | abi.snapshot.decode | i1 | 9.4 (9.0–9.4) | 10.6 (10.4–11.4) | 11.8 (11.2–12.2) | 12.1 (11.9–12.4) |
| wasm | abi.read | abi.save.exec | main-local | 14.1 (12.8–14.8) | 12.2 (11.9–12.3) | 14.8 (14.6–15.0) | 16.0 (15.2–16.6) |
| wasm | abi.read | abi.save.exec | i1 | 14.1 (13.9–14.2) | 11.7 (11.6–12.0) | 14.4 (14.2–15.0) | 15.2 (14.7–15.2) |
| wasm | abi.read | abi.save.decode | main-local | 0.5 (0.4–0.5) | 0.4 (0.4–0.5) | 0.6 (0.6–0.6) | 0.6 (0.6–0.6) |
| wasm | abi.read | abi.save.decode | i1 | 0.4 (0.4–0.4) | 0.4 (0.4–0.5) | 0.6 (0.5–0.6) | 0.6 (0.6–0.6) |
| wasm | abi.dispatch_only | abi.encode_args | i1 | 0.9 (0.8–1.0) | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_only | abi.dispatch_only.exec | i1 | 98.1 (94.8–109.6) | 18.4 (15.6–18.7) | 13.7 (13.6–14.8) | 12.9 (12.8–13.0) |
| wasm | abi.dispatch_only | abi.dispatch_only | i1 | 98.8 (95.6–111.7) | 18.6 (15.9–18.9) | 13.9 (13.8–14.9) | 13.0 (12.9–13.1) |
| wasm | abi.bench_read | abi.view_build.exec | i1 | 4.2 (4.2–6.1) | 2.7 (2.7–3.1) | 3.0 (3.0–3.4) | 3.5 (3.4–3.5) |
| wasm | abi.bench_read | abi.snapshot_build.exec | i1 | 35.8 (35.4–46.4) | 25.7 (24.8–29.3) | 24.8 (24.5–27.8) | 29.5 (28.0–30.9) |
| wasm | abi.bench_read | abi.view.exec | i1 | 12.8 (12.8–18.4) | 8.1 (7.9–9.3) | 8.6 (8.4–9.4) | 9.2 (9.0–9.3) |
| wasm | abi.bench_read | abi.view.decode | i1 | 3.1 (2.9–3.5) | 2.2 (2.2–2.6) | 2.3 (2.2–2.6) | 2.4 (2.3–2.4) |
| wasm | abi.bench_read | abi.snapshot.exec | i1 | 79.5 (74.5–87.0) | 63.3 (62.9–72.9) | 65.2 (64.5–72.2) | 71.2 (70.4–72.3) |
| wasm | abi.bench_read | abi.snapshot.decode | i1 | 9.3 (8.7–9.9) | 10.8 (10.6–12.3) | 11.3 (11.2–13.1) | 12.2 (11.8–12.3) |
| wasm | probe.dispatch_view | total | i2 | 157.8 (156.2–176.5) | 44.7 (43.8–44.9) | 42.5 (41.8–44.5) | 43.1 (42.5–43.5) |
| wasm | probe.dispatch_view | web.enter | i2 | 3.7 (2.0–6.9) | 0.3 (0.2–0.3) | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | payload.parse | i2 | 2.9 (2.4–3.3) | 0.3 (0.2–0.3) | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | payload.resolve | i2 | 0.8 (0.8–1.0) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | apply.validate | i2 | 0.4 (0.4–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | apply.clone | i2 | 0.6 (0.5–0.7) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | run.clock | i2 | 0.4 (0.2–0.5) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | run.prologue | i2 | 0.5 (0.4–0.7) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | run.rules | i2 | 34.2 (32.5–48.1) | 12.8 (12.6–13.0) | 11.7 (11.4–12.0) | 10.8 (10.8–10.9) |
| wasm | probe.dispatch_view | run.changes | i2 | 0.7 (0.7–0.9) | 0.5 (0.4–0.5) | 0.5 (0.5–0.5) | 0.6 (0.6–0.6) |
| wasm | probe.dispatch_view | bind.stale | i2 | 1.5 (1.3–1.6) | 1.5 (1.5–1.5) | 1.5 (1.5–1.5) | 1.5 (1.4–1.5) |
| wasm | probe.dispatch_view | bind.none | i2 | - | - | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | bind.eval | i2 | 35.3 (33.1–38.4) | 0.9 (0.9–0.9) | 78.0 (74.3–80.1) | - |
| wasm | probe.dispatch_view | bind.sort | i2 | 1.1 (1.0–1.2) | 0.1 (0.1–0.1) | 1.4 (1.3–1.4) | - |
| wasm | probe.dispatch_view | bind.explain | i2 | 6.1 (5.6–6.4) | 0.2 (0.2–0.2) | 8.0 (7.3–8.5) | - |
| wasm | probe.dispatch_view | bind.write | i2 | 5.3 (4.7–5.5) | 0.7 (0.7–0.7) | 7.3 (6.6–7.7) | - |
| wasm | probe.dispatch_view | apply.swap | i2 | 3.9 (3.3–4.0) | 0.9 (0.8–0.9) | 0.7 (0.7–0.7) | 0.7 (0.7–0.8) |
| wasm | probe.dispatch_view | view.enter | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | view.names | i2 | 0.5 (0.5–0.6) | 0.4 (0.4–0.4) | 0.5 (0.5–0.5) | 0.5 (0.5–0.6) |
| wasm | probe.dispatch_view | view.sort | i2 | 1.2 (1.1–1.2) | 0.4 (0.4–0.5) | 0.6 (0.5–0.6) | 0.6 (0.6–0.6) |
| wasm | probe.dispatch_view | view.commitments | i2 | 1.0 (0.8–1.0) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) | 0.6 (0.6–0.6) |
| wasm | probe.dispatch_view | view.relations | i2 | 1.0 (0.9–1.0) | 0.7 (0.7–0.8) | 0.9 (0.8–0.9) | 1.0 (1.0–1.0) |
| wasm | probe.dispatch_view | view.build | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | web.serialize | i2 | 9.5 (8.2–11.8) | 5.0 (5.0–5.1) | 5.2 (5.2–5.5) | 5.5 (5.4–5.5) |
| wasm | probe.dispatch_view | web.drop | i2 | 0.8 (0.8–0.8) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) | 0.5 (0.5–0.5) |
| wasm | probe.dispatch_view | exit | i2 | 3.5 (3.4–4.8) | 2.4 (2.4–2.5) | 2.4 (2.4–2.5) | 2.5 (2.5–2.5) |
| wasm | probe.dispatch_view | js.parse | i2 | 28.9 (26.1–34.5) | 15.7 (15.3–15.8) | 16.2 (15.7–16.9) | 16.8 (16.4–16.9) |
| wasm | probe.dispatch_outcome | total | i2 | 301.1 (282.3–374.8) | 131.2 (129.4–135.1) | 133.9 (132.3–137.4) | 137.4 (134.9–138.6) |
| wasm | probe.dispatch_outcome | web.enter | i2 | 3.7 (3.4–6.4) | 0.4 (0.4–0.4) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_outcome | payload.parse | i2 | 2.9 (2.9–3.8) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.2–0.3) |
| wasm | probe.dispatch_outcome | payload.resolve | i2 | 0.9 (0.9–1.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_outcome | apply.validate | i2 | 0.5 (0.4–0.7) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.2) |
| wasm | probe.dispatch_outcome | apply.clone | i2 | 0.5 (0.5–0.7) | 0.2 (0.1–0.2) | 0.2 (0.1–0.2) | 0.2 (0.1–0.2) |
| wasm | probe.dispatch_outcome | run.clock | i2 | 0.3 (0.3–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_outcome | run.prologue | i2 | 0.5 (0.4–0.6) | 0.1 (0.1–0.2) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_outcome | run.rules | i2 | 45.1 (35.9–49.1) | 13.4 (13.2–14.1) | 12.0 (11.9–12.5) | 11.4 (11.1–11.5) |
| wasm | probe.dispatch_outcome | run.changes | i2 | 0.8 (0.8–1.1) | 0.5 (0.5–0.6) | 0.6 (0.6–0.6) | 0.6 (0.6–0.6) |
| wasm | probe.dispatch_outcome | bind.stale | i2 | 1.7 (1.5–1.8) | 1.6 (1.6–1.6) | 1.5 (1.5–1.6) | 1.5 (1.5–1.6) |
| wasm | probe.dispatch_outcome | bind.none | i2 | - | - | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_outcome | bind.eval | i2 | 36.7 (35.8–41.5) | 1.0 (1.0–1.0) | 74.2 (72.7–82.9) | - |
| wasm | probe.dispatch_outcome | bind.sort | i2 | 1.2 (1.2–1.4) | 0.1 (0.1–0.2) | 1.5 (1.4–1.6) | - |
| wasm | probe.dispatch_outcome | bind.explain | i2 | 7.2 (6.6–7.6) | 0.2 (0.2–0.2) | 7.3 (6.9–7.5) | - |
| wasm | probe.dispatch_outcome | bind.write | i2 | 7.1 (5.4–7.2) | 0.8 (0.8–0.9) | 7.3 (7.2–7.6) | - |
| wasm | probe.dispatch_outcome | apply.swap | i2 | 3.6 (3.4–5.1) | 1.0 (1.0–1.1) | 0.9 (0.9–0.9) | 1.0 (1.0–1.0) |
| wasm | probe.dispatch_outcome | snap.enter | i2 | 0.3 (0.2–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_outcome | snap.names | i2 | 0.7 (0.6–0.7) | 0.4 (0.4–0.5) | 0.5 (0.5–0.5) | 0.5 (0.5–0.6) |
| wasm | probe.dispatch_outcome | snap.sort | i2 | 1.4 (1.4–1.8) | 0.5 (0.5–0.5) | 0.6 (0.6–0.7) | 0.7 (0.7–0.7) |
| wasm | probe.dispatch_outcome | snap.symbols | i2 | 4.3 (4.2–5.9) | 2.5 (2.5–2.6) | 2.7 (2.7–2.9) | 2.8 (2.8–2.8) |
| wasm | probe.dispatch_outcome | snap.shown | i2 | 6.2 (6.0–6.8) | 4.7 (4.6–4.8) | 4.6 (4.4–4.7) | 4.9 (4.9–5.0) |
| wasm | probe.dispatch_outcome | snap.values | i2 | 10.3 (7.6–10.4) | 4.3 (4.3–4.5) | 4.4 (4.1–4.5) | 4.3 (4.3–4.4) |
| wasm | probe.dispatch_outcome | snap.records | i2 | 2.8 (2.7–2.9) | 1.9 (1.9–2.0) | 2.3 (2.3–2.4) | 2.4 (2.4–2.4) |
| wasm | probe.dispatch_outcome | snap.static | i2 | 1.9 (1.8–2.1) | 1.3 (1.3–1.4) | 1.2 (1.2–1.3) | 1.3 (1.2–1.3) |
| wasm | probe.dispatch_outcome | snap.relations | i2 | 0.9 (0.9–1.0) | 0.9 (0.9–1.0) | 1.0 (1.0–1.0) | 1.1 (1.1–1.1) |
| wasm | probe.dispatch_outcome | snap.build | i2 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_outcome | outcome.built | i2 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_outcome | web.serialize | i2 | 33.1 (28.0–36.4) | 18.0 (18.0–19.0) | 19.3 (19.1–19.4) | 20.1 (19.6–20.2) |
| wasm | probe.dispatch_outcome | web.drop | i2 | 17.7 (15.5–22.2) | 10.4 (10.3–10.6) | 10.9 (10.8–11.0) | 11.4 (11.2–11.6) |
| wasm | probe.dispatch_outcome | exit | i2 | 7.4 (7.3–8.1) | 7.1 (7.1–7.1) | 7.4 (7.2–7.6) | 7.9 (7.8–8.0) |
| wasm | probe.dispatch_outcome | js.parse | i2 | 86.9 (77.7–103.4) | 57.0 (56.3–58.7) | 60.1 (59.9–60.9) | 62.8 (61.8–63.6) |
| wasm | probe.read | view:total | i2 | 43.9 (41.9–49.6) | 28.4 (27.3–30.6) | 29.7 (28.9–32.8) | 29.9 (29.7–31.0) |
| wasm | probe.read | view:web.enter | i2 | 0.5 (0.5–0.5) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) |
| wasm | probe.read | view:view.enter | i2 | 0.1 (0.0–0.1) | 0.1 (0.0–0.1) | 0.1 (0.0–0.1) | 0.1 (0.0–0.1) |
| wasm | probe.read | view:view.names | i2 | 0.4 (0.4–0.5) | 0.4 (0.4–0.4) | 0.5 (0.4–0.5) | 0.5 (0.5–0.5) |
| wasm | probe.read | view:view.sort | i2 | 0.7 (0.6–0.7) | 0.4 (0.4–0.5) | 0.6 (0.6–0.6) | 0.6 (0.6–0.7) |
| wasm | probe.read | view:view.commitments | i2 | 0.6 (0.5–0.6) | 0.5 (0.5–0.6) | 0.6 (0.6–0.7) | 0.6 (0.6–0.7) |
| wasm | probe.read | view:view.relations | i2 | 0.8 (0.8–0.9) | 1.0 (0.9–1.0) | 1.1 (1.1–1.2) | 1.2 (1.2–1.3) |
| wasm | probe.read | view:view.build | i2 | 0.1 (0.0–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.read | view:web.serialize | i2 | 5.4 (5.1–5.9) | 4.9 (4.7–5.3) | 5.1 (5.0–5.7) | 5.2 (5.1–5.3) |
| wasm | probe.read | view:web.drop | i2 | 0.6 (0.6–0.6) | 0.6 (0.6–0.6) | 0.7 (0.6–0.7) | 0.6 (0.6–0.7) |
| wasm | probe.read | view:exit | i2 | 2.4 (2.1–2.4) | 2.4 (2.3–2.5) | 2.5 (2.4–2.7) | 2.5 (2.4–2.6) |
| wasm | probe.read | view:js.parse | i2 | 30.5 (29.4–31.8) | 17.8 (17.1–19.2) | 18.2 (17.7–20.2) | 18.3 (18.2–19.0) |
| wasm | probe.read | snapshot:total | i2 | 180.6 (177.4–189.2) | 150.2 (144.6–162.8) | 159.2 (159.1–180.0) | 165.4 (163.0–171.2) |
| wasm | probe.read | snapshot:web.enter | i2 | 0.3 (0.3–0.6) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) |
| wasm | probe.read | snapshot:snap.enter | i2 | 0.1 (0.1–0.1) | 0.1 (0.0–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.read | snapshot:snap.names | i2 | 0.5 (0.5–0.6) | 0.5 (0.4–0.5) | 0.5 (0.5–0.6) | 0.5 (0.5–0.6) |
| wasm | probe.read | snapshot:snap.sort | i2 | 1.2 (1.1–1.3) | 0.6 (0.6–0.6) | 0.8 (0.8–0.8) | 0.8 (0.8–0.9) |
| wasm | probe.read | snapshot:snap.symbols | i2 | 4.2 (4.1–4.5) | 3.3 (3.1–3.4) | 3.5 (3.4–3.7) | 3.9 (3.8–4.0) |
| wasm | probe.read | snapshot:snap.shown | i2 | 5.4 (5.3–5.6) | 5.1 (5.0–5.6) | 5.3 (5.1–5.6) | 5.6 (5.6–5.8) |
| wasm | probe.read | snapshot:snap.values | i2 | 7.7 (7.5–7.8) | 5.1 (5.1–5.5) | 5.3 (5.2–5.9) | 5.2 (5.2–5.5) |
| wasm | probe.read | snapshot:snap.records | i2 | 2.4 (2.3–2.4) | 2.0 (2.0–2.2) | 2.5 (2.4–2.6) | 2.5 (2.5–2.6) |
| wasm | probe.read | snapshot:snap.static | i2 | 1.6 (1.6–1.7) | 1.3 (1.3–1.4) | 1.3 (1.3–1.4) | 1.3 (1.3–1.3) |
| wasm | probe.read | snapshot:snap.relations | i2 | 0.8 (0.7–0.8) | 0.9 (0.9–1.0) | 1.1 (1.0–1.1) | 1.2 (1.1–1.2) |
| wasm | probe.read | snapshot:snap.build | i2 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.read | snapshot:web.serialize | i2 | 47.3 (47.1–50.4) | 41.0 (39.5–44.8) | 43.6 (43.2–49.4) | 45.3 (43.8–46.7) |
| wasm | probe.read | snapshot:web.drop | i2 | 15.6 (15.5–17.3) | 13.3 (13.1–14.2) | 13.9 (13.8–15.2) | 15.2 (15.2–15.7) |
| wasm | probe.read | snapshot:exit | i2 | 9.8 (9.3–10.2) | 11.4 (11.3–12.1) | 12.2 (11.8–13.2) | 12.6 (12.4–12.9) |
| wasm | probe.read | snapshot:js.parse | i2 | 77.6 (74.8–81.2) | 64.4 (62.4–70.0) | 69.1 (68.0–77.5) | 70.3 (69.8–73.0) |
| wasm | probe.read | save:total | i2 | 20.3 (19.0–20.8) | 15.2 (15.0–16.3) | 19.1 (18.9–19.5) | 19.7 (19.7–20.4) |
| wasm | probe.read | save:web.enter | i2 | 0.5 (0.5–0.9) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) | 0.2 (0.1–0.2) |
| wasm | probe.read | save:save.enter | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.read | save:save.build | i2 | 6.6 (6.5–6.8) | 6.4 (6.3–6.9) | 8.2 (8.1–8.4) | 8.7 (8.6–9.0) |
| wasm | probe.read | save:save.serialize | i2 | 6.1 (5.6–6.2) | 4.8 (4.7–5.2) | 5.5 (5.4–5.9) | 5.6 (5.6–5.8) |
| wasm | probe.read | save:save.drop | i2 | 2.7 (2.7–3.0) | 3.0 (3.0–3.1) | 4.1 (4.1–4.3) | 4.4 (4.4–4.5) |
| wasm | probe.read | save:exit | i2 | 0.9 (0.9–0.9) | 0.8 (0.8–0.8) | 0.9 (0.9–0.9) | 0.9 (0.9–0.9) |

## glowcap-unbound

The Glowcap replay program with its 39 one-line bind statements removed, on the replay's full stream: the same rules, states and transaction with no bindings to evaluate, so apply here against apply on glowcap-replay isolates binding evaluation. Program `experiments/glowcap/caveat5/glowcap.cav` (`ffa078c4cb75`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | main-local | i1 | i2 |
| --- | --- | --- | --- | --- | --- |
| native | apply | apply | 12.0 (11.9–13.6) · p95 13.4 | 12.2 (11.9–12.5) · p95 14.0 | - |
| native | web.dispatch_view | web.dispatch_view | 16.9 (16.7–17.0) · p95 18.4 | 17.0 (16.8–17.2) · p95 18.9 | - |
| native | bench.dispatch_only | bench.dispatch_only | - | 12.3 (12.1–12.8) · p95 13.8 | - |
| native | probe.dispatch_view | apply.clone | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.dispatch_view | apply.swap | - | - | 1.1 (1.1–1.1) · p95 1.2 |
| native | probe.dispatch_view | apply.validate | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | bind.none | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.dispatch_view | bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.dispatch_view | payload.parse | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_view | payload.resolve | - | - | 0.2 (0.1–0.2) · p95 0.2 |
| native | probe.dispatch_view | run.changes | - | - | 0.3 (0.3–0.4) · p95 0.4 |
| native | probe.dispatch_view | run.clock | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | run.prologue | - | - | 0.2 (0.1–0.2) · p95 0.2 |
| native | probe.dispatch_view | run.rules | - | - | 11.1 (10.4–11.2) · p95 12.7 |
| native | probe.dispatch_view | total | - | - | 19.0 (17.8–19.2) · p95 20.1 |
| native | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.commitments | - | - | 0.5 (0.5–0.5) · p95 0.5 |
| native | probe.dispatch_view | view.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.dispatch_view | view.names | - | - | 0.3 (0.3–0.4) · p95 0.4 |
| native | probe.dispatch_view | view.relations | - | - | 1.3 (1.2–1.3) · p95 1.5 |
| native | probe.dispatch_view | view.sort | - | - | 0.5 (0.5–0.6) · p95 0.6 |
| native | probe.dispatch_view | web.drop | - | - | 0.6 (0.6–0.6) · p95 0.7 |
| native | probe.dispatch_view | web.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.dispatch_view | web.serialize | - | - | 1.8 (1.7–1.8) · p95 2.0 |
| wasm | abi.dispatch_view | abi.encode_args | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | - |
| wasm | abi.dispatch_view | abi.exec | 17.1 (16.9–17.7) · p95 22.1 | 17.5 (17.1–18.1) · p95 22.2 | - |
| wasm | abi.dispatch_view | abi.decode | 0.5 (0.5–0.5) · p95 0.9 | 0.5 (0.5–0.5) · p95 0.9 | - |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | abi.dispatch_view | abi.dispatch_view | 17.8 (17.7–18.5) · p95 23.3 | 18.3 (17.8–19.0) · p95 23.2 | - |
| wasm | abi.dispatch_only | abi.encode_args | - | 0.1 (0.1–0.1) · p95 0.3 | - |
| wasm | abi.dispatch_only | abi.dispatch_only.exec | - | 11.5 (11.5–12.0) · p95 15.2 | - |
| wasm | abi.dispatch_only | abi.dispatch_only | - | 11.6 (11.6–12.1) · p95 15.4 | - |
| wasm | probe.dispatch_view | total | - | - | 25.9 (25.8–26.5) · p95 32.3 |
| wasm | probe.dispatch_view | web.enter | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.dispatch_view | payload.parse | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_view | payload.resolve | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| wasm | probe.dispatch_view | apply.validate | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | apply.clone | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | run.clock | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | run.prologue | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | run.rules | - | - | 10.5 (10.4–10.8) · p95 14.3 |
| wasm | probe.dispatch_view | run.changes | - | - | 0.5 (0.5–0.6) · p95 0.7 |
| wasm | probe.dispatch_view | bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | bind.none | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | apply.swap | - | - | 0.8 (0.8–0.8) · p95 1.1 |
| wasm | probe.dispatch_view | view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | view.names | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| wasm | probe.dispatch_view | view.sort | - | - | 0.6 (0.6–0.6) · p95 0.7 |
| wasm | probe.dispatch_view | view.commitments | - | - | 0.6 (0.6–0.6) · p95 0.7 |
| wasm | probe.dispatch_view | view.relations | - | - | 1.0 (0.9–1.0) · p95 1.3 |
| wasm | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | web.serialize | - | - | 2.7 (2.7–2.7) · p95 3.3 |
| wasm | probe.dispatch_view | web.drop | - | - | 0.5 (0.5–0.5) · p95 0.7 |
| wasm | probe.dispatch_view | exit | - | - | 0.6 (0.6–0.6) · p95 1.6 |
| wasm | probe.dispatch_view | js.parse | - | - | 6.1 (6.1–6.3) · p95 8.0 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | main-local | 34.8 (32.8–37.5) | 13.2 (13.2–14.9) | 12.2 (12.2–14.1) | 11.9 (11.8–13.5) |
| native | apply | apply | i1 | 34.2 (34.1–39.4) | 13.5 (13.1–14.1) | 12.5 (12.2–13.0) | 12.1 (11.9–12.5) |
| native | web.dispatch_view | web.dispatch_view | main-local | 40.6 (37.5–40.9) | 17.3 (17.2–17.5) | 17.0 (16.7–17.1) | 16.9 (16.6–16.9) |
| native | web.dispatch_view | web.dispatch_view | i1 | 44.4 (43.6–45.6) | 17.4 (17.3–17.8) | 17.1 (16.7–17.2) | 16.9 (16.8–17.2) |
| native | bench.dispatch_only | bench.dispatch_only | i1 | 33.7 (32.3–34.9) | 13.5 (13.5–14.2) | 13.2 (12.6–13.4) | 12.3 (12.0–12.7) |
| native | probe.dispatch_view | apply.clone | i2 | 0.5 (0.5–0.5) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| native | probe.dispatch_view | apply.swap | i2 | 2.5 (2.3–2.5) | 0.9 (0.8–0.9) | 1.1 (1.0–1.1) | 1.1 (1.1–1.1) |
| native | probe.dispatch_view | apply.validate | i2 | 0.3 (0.3–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | bind.none | i2 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.dispatch_view | bind.stale | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | exit | i2 | 0.0 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.dispatch_view | payload.parse | i2 | 1.2 (1.2–1.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_view | payload.resolve | i2 | 0.6 (0.5–0.6) | 0.2 (0.1–0.2) | 0.2 (0.1–0.2) | 0.2 (0.1–0.2) |
| native | probe.dispatch_view | run.changes | i2 | 0.6 (0.6–0.6) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.4 (0.3–0.4) |
| native | probe.dispatch_view | run.clock | i2 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | run.prologue | i2 | 0.3 (0.3–0.3) | 0.2 (0.1–0.2) | 0.2 (0.1–0.2) | 0.2 (0.1–0.2) |
| native | probe.dispatch_view | run.rules | i2 | 28.4 (26.7–29.8) | 12.8 (11.8–13.1) | 11.6 (10.8–11.8) | 11.0 (10.4–11.2) |
| native | probe.dispatch_view | total | i2 | 40.1 (38.4–41.4) | 19.7 (18.1–20.1) | 19.1 (17.9–19.4) | 19.0 (17.8–19.1) |
| native | probe.dispatch_view | view.build | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | view.commitments | i2 | 0.7 (0.7–0.8) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) | 0.5 (0.5–0.5) |
| native | probe.dispatch_view | view.enter | i2 | 0.1 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.dispatch_view | view.names | i2 | 0.4 (0.4–0.4) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.4 (0.3–0.4) |
| native | probe.dispatch_view | view.relations | i2 | 1.0 (0.9–1.1) | 1.0 (1.0–1.1) | 1.2 (1.1–1.2) | 1.3 (1.3–1.3) |
| native | probe.dispatch_view | view.sort | i2 | 0.9 (0.8–0.9) | 0.4 (0.4–0.4) | 0.5 (0.5–0.5) | 0.6 (0.5–0.6) |
| native | probe.dispatch_view | web.drop | i2 | 0.5 (0.5–0.6) | 0.5 (0.5–0.5) | 0.5 (0.5–0.6) | 0.6 (0.6–0.6) |
| native | probe.dispatch_view | web.enter | i2 | 0.1 (0.0–0.1) | 0.1 (0.0–0.1) | 0.1 (0.0–0.1) | 0.1 (0.0–0.1) |
| native | probe.dispatch_view | web.serialize | i2 | 3.0 (2.9–3.0) | 1.6 (1.5–1.7) | 1.7 (1.6–1.8) | 1.8 (1.7–1.8) |
| wasm | abi.dispatch_view | abi.encode_args | main-local | 1.0 (1.0–1.2) | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | i1 | 0.8 (0.7–0.8) | 0.3 (0.3–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.exec | main-local | 54.4 (52.5–69.1) | 20.2 (19.3–20.7) | 18.8 (17.6–19.8) | 17.0 (16.8–17.5) |
| wasm | abi.dispatch_view | abi.exec | i1 | 52.0 (44.8–56.0) | 19.5 (17.4–21.1) | 18.5 (17.5–19.8) | 17.5 (17.0–17.8) |
| wasm | abi.dispatch_view | abi.decode | main-local | 1.8 (1.2–2.0) | 0.5 (0.5–0.5) | 0.5 (0.4–0.5) | 0.5 (0.4–0.5) |
| wasm | abi.dispatch_view | abi.decode | i1 | 1.3 (1.0–1.4) | 0.5 (0.5–0.6) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) |
| wasm | abi.dispatch_view | abi.free | main-local | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | i1 | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local | 57.8 (56.0–72.3) | 21.1 (20.1–21.6) | 19.6 (18.3–20.6) | 17.7 (17.6–18.2) |
| wasm | abi.dispatch_view | abi.dispatch_view | i1 | 54.4 (46.7–58.9) | 20.3 (18.1–22.0) | 19.2 (18.1–20.6) | 18.3 (17.7–18.7) |
| wasm | abi.dispatch_only | abi.encode_args | i1 | 0.7 (0.7–0.7) | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_only | abi.dispatch_only.exec | i1 | 44.7 (40.1–57.7) | 13.3 (12.9–14.6) | 13.4 (13.2–14.1) | 11.4 (11.3–11.8) |
| wasm | abi.dispatch_only | abi.dispatch_only | i1 | 45.0 (40.6–58.3) | 13.5 (13.1–14.8) | 13.6 (13.4–14.3) | 11.5 (11.4–11.9) |
| wasm | probe.dispatch_view | total | i2 | 89.0 (86.7–120.0) | 28.2 (25.9–28.8) | 26.1 (25.9–29.3) | 25.7 (25.7–26.5) |
| wasm | probe.dispatch_view | web.enter | i2 | 2.9 (2.8–4.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | payload.parse | i2 | 2.7 (2.6–2.8) | 0.3 (0.3–0.3) | 0.3 (0.2–0.3) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | payload.resolve | i2 | 0.8 (0.7–0.8) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | apply.validate | i2 | 0.4 (0.3–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | apply.clone | i2 | 0.6 (0.5–0.6) | 0.2 (0.1–0.2) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | run.clock | i2 | 0.4 (0.3–0.5) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | run.prologue | i2 | 0.5 (0.4–0.5) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | run.rules | i2 | 38.9 (37.6–42.9) | 13.7 (12.5–13.8) | 11.8 (11.6–13.0) | 10.4 (10.4–10.7) |
| wasm | probe.dispatch_view | run.changes | i2 | 0.7 (0.7–0.8) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) | 0.5 (0.5–0.6) |
| wasm | probe.dispatch_view | bind.stale | i2 | 0.1 (0.0–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.0–0.1) |
| wasm | probe.dispatch_view | bind.none | i2 | 0.1 (0.0–0.1) | 0.1 (0.0–0.1) | 0.1 (0.0–0.1) | 0.0 (0.0–0.1) |
| wasm | probe.dispatch_view | apply.swap | i2 | 3.4 (3.1–3.7) | 0.6 (0.6–0.6) | 0.8 (0.7–0.8) | 0.8 (0.8–0.8) |
| wasm | probe.dispatch_view | view.enter | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | view.names | i2 | 0.5 (0.5–0.6) | 0.4 (0.4–0.4) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) |
| wasm | probe.dispatch_view | view.sort | i2 | 1.4 (1.1–1.5) | 0.5 (0.4–0.5) | 0.6 (0.6–0.6) | 0.6 (0.6–0.6) |
| wasm | probe.dispatch_view | view.commitments | i2 | 0.9 (0.8–1.0) | 0.5 (0.5–0.5) | 0.6 (0.6–0.6) | 0.6 (0.6–0.6) |
| wasm | probe.dispatch_view | view.relations | i2 | 1.0 (0.9–1.0) | 0.8 (0.8–0.8) | 0.9 (0.9–1.0) | 1.0 (0.9–1.0) |
| wasm | probe.dispatch_view | view.build | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | web.serialize | i2 | 5.1 (4.8–5.4) | 2.7 (2.4–2.8) | 2.7 (2.6–3.0) | 2.7 (2.7–2.7) |
| wasm | probe.dispatch_view | web.drop | i2 | 0.9 (0.8–0.9) | 0.4 (0.4–0.4) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) |
| wasm | probe.dispatch_view | exit | i2 | 1.3 (1.3–1.6) | 0.6 (0.6–0.6) | 0.6 (0.6–0.6) | 0.6 (0.6–0.6) |
| wasm | probe.dispatch_view | js.parse | i2 | 13.2 (12.8–14.4) | 6.1 (5.7–6.2) | 6.0 (6.0–6.6) | 6.1 (6.1–6.3) |

## glowcap-resume

The published resume measurement's stream (experiments/glowcap/resume-bench.mjs): absorb cave glowcap, then 9,600 ticks of dt 0.0625 (ten minutes of play), then save and restore. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `0407c19120d4`, 1 episode(s) × 1 = 9601 events per pass.

| Engine | Mode | Operation | main-local | i1 | i2 |
| --- | --- | --- | --- | --- | --- |
| native | lifecycle | from_source | 2522.3 (2504.7–2723.0) · p95 2698.9 | 2552.3 (2519.3–2597.0) · p95 2694.7 | - |
| native | lifecycle | restore | 2629.8 (2568.5–2773.5) · p95 2846.4 | 2607.2 (2604.2–2657.8) · p95 2780.3 | - |
| native | lifecycle | restore.parse | 17.4 (17.2–21.1) · p95 27.5 | 19.1 (18.9–21.5) · p95 25.6 | - |
| native | lifecycle | restore_json | 2660.7 (2637.1–2833.9) · p95 2853.1 | 2669.4 (2642.1–2688.3) · p95 2839.8 | - |
| native | lifecycle | save | 13.4 (12.3–14.3) · p95 16.7 | 14.6 (12.6–15.0) · p95 18.9 | - |
| native | lifecycle | save_json | 24.1 (15.6–24.7) · p95 28.2 | 16.9 (15.7–18.4) · p95 22.2 | - |
| native | lifecycle | web.new | 2518.2 (2473.0–2688.2) · p95 2690.8 | 2553.3 (2531.7–2574.3) · p95 2678.3 | - |
| native | lifecycle | web.restore | 2634.6 (2614.0–2814.4) · p95 2801.1 | 2633.0 (2619.5–2719.7) · p95 2827.1 | - |
| native | lifecycle | web.save | 10.9 (10.7–12.8) · p95 14.3 | 11.2 (11.2–11.7) · p95 14.6 | - |
| native | probe.lifecycle | new:bind.eval | - | - | 26.4 (25.8–26.5) · p95 28.5 |
| native | probe.lifecycle | new:bind.explain | - | - | 3.4 (3.4–3.6) · p95 4.5 |
| native | probe.lifecycle | new:bind.sort | - | - | 2.7 (2.7–2.8) · p95 3.0 |
| native | probe.lifecycle | new:bind.stale | - | - | 0.2 (0.2–0.3) · p95 0.3 |
| native | probe.lifecycle | new:bind.write | - | - | 10.0 (9.9–10.5) · p95 10.8 |
| native | probe.lifecycle | new:exit | - | - | 142.7 (142.7–154.7) · p95 179.4 |
| native | probe.lifecycle | new:load.bindings | - | - | 0.6 (0.6–0.6) · p95 0.7 |
| native | probe.lifecycle | new:load.declare | - | - | 242.5 (233.9–256.6) · p95 289.8 |
| native | probe.lifecycle | new:load.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.lifecycle | new:load.expand | - | - | 451.8 (433.6–458.5) · p95 495.0 |
| native | probe.lifecycle | new:load.map | - | - | 24.1 (23.1–30.9) · p95 27.9 |
| native | probe.lifecycle | new:load.parse | - | - | 1066.9 (1058.7–1083.0) · p95 1146.1 |
| native | probe.lifecycle | new:load.procedures | - | - | 201.0 (201.0–203.8) · p95 269.8 |
| native | probe.lifecycle | new:load.validate | - | - | 524.9 (518.1–527.7) · p95 548.3 |
| native | probe.lifecycle | new:total | - | - | 2724.0 (2711.1–2724.5) · p95 2902.3 |
| native | probe.lifecycle | restore:exit | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.lifecycle | restore:load.bindings | - | - | 47.8 (47.2–48.7) · p95 54.0 |
| native | probe.lifecycle | restore:load.declare | - | - | 328.4 (326.9–335.1) · p95 366.2 |
| native | probe.lifecycle | restore:load.enter | - | - | 0.1 (0.1–0.2) · p95 0.1 |
| native | probe.lifecycle | restore:load.expand | - | - | 442.2 (440.7–462.0) · p95 507.8 |
| native | probe.lifecycle | restore:load.map | - | - | 33.9 (22.2–36.8) · p95 41.6 |
| native | probe.lifecycle | restore:load.parse | - | - | 1064.2 (1059.8–1131.0) · p95 1167.7 |
| native | probe.lifecycle | restore:load.procedures | - | - | 187.5 (186.2–232.3) · p95 232.9 |
| native | probe.lifecycle | restore:load.validate | - | - | 513.5 (503.7–519.1) · p95 539.8 |
| native | probe.lifecycle | restore:restore.bindings | - | - | 59.7 (59.4–59.9) · p95 62.8 |
| native | probe.lifecycle | restore:restore.done | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.lifecycle | restore:restore.drop_save | - | - | 3.7 (3.6–3.8) · p95 5.5 |
| native | probe.lifecycle | restore:restore.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.lifecycle | restore:restore.graph | - | - | 7.5 (7.2–8.0) · p95 8.0 |
| native | probe.lifecycle | restore:restore.header | - | - | 0.8 (0.8–0.9) · p95 1.0 |
| native | probe.lifecycle | restore:restore.loaded | - | - | 144.1 (141.8–145.4) · p95 163.6 |
| native | probe.lifecycle | restore:restore.parse | - | - | 18.7 (17.7–22.5) · p95 21.9 |
| native | probe.lifecycle | restore:restore.records | - | - | 17.5 (16.9–19.3) · p95 19.6 |
| native | probe.lifecycle | restore:restore.states | - | - | 4.6 (4.6–5.1) · p95 5.0 |
| native | probe.lifecycle | restore:total | - | - | 2884.4 (2883.4–3004.6) · p95 3075.1 |
| native | probe.lifecycle | restore:web.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| wasm | lifecycle | raw.new | 2566.8 (2426.8–2568.3) · p95 2809.4 | 2681.5 (2574.3–3023.4) · p95 2766.7 | - |
| wasm | lifecycle | raw.save | 33.4 (32.7–34.7) · p95 47.8 | 39.6 (33.5–41.2) · p95 50.6 | - |
| wasm | lifecycle | raw.restore | 2555.5 (2492.8–2559.0) · p95 2981.6 | 2701.6 (2552.2–3017.9) · p95 2848.8 | - |
| wasm | lifecycle | kit.open | 2509.5 (2377.9–2513.0) · p95 2782.1 | 2611.7 (2482.8–2943.0) · p95 2773.4 | - |
| wasm | lifecycle | kit.save | 72.6 (68.6–76.6) · p95 102.1 | 93.7 (75.2–97.2) · p95 123.5 | - |
| wasm | lifecycle | kit.restore | 2525.1 (2426.0–2553.9) · p95 2664.2 | 2631.6 (2526.9–2967.3) · p95 2768.2 | - |
| wasm | probe.lifecycle | restore:total | - | - | 2760.6 (2559.1–3058.5) · p95 3226.1 |
| wasm | probe.lifecycle | restore:web.enter | - | - | 16.2 (15.0–17.8) · p95 27.0 |
| wasm | probe.lifecycle | restore:restore.enter | - | - | 0.1 (0.1–0.1) · p95 0.3 |
| wasm | probe.lifecycle | restore:restore.parse | - | - | 26.6 (24.9–28.1) · p95 40.5 |
| wasm | probe.lifecycle | restore:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.3 |
| wasm | probe.lifecycle | restore:load.parse | - | - | 1132.9 (1047.5–1252.8) · p95 1393.8 |
| wasm | probe.lifecycle | restore:load.expand | - | - | 315.7 (291.9–342.6) · p95 375.9 |
| wasm | probe.lifecycle | restore:load.map | - | - | 31.2 (28.7–33.9) · p95 41.0 |
| wasm | probe.lifecycle | restore:load.declare | - | - | 136.7 (129.4–152.9) · p95 166.2 |
| wasm | probe.lifecycle | restore:load.procedures | - | - | 122.2 (112.2–137.9) · p95 145.9 |
| wasm | probe.lifecycle | restore:load.validate | - | - | 585.2 (546.7–654.3) · p95 700.8 |
| wasm | probe.lifecycle | restore:load.bindings | - | - | 52.4 (47.5–56.3) · p95 61.7 |
| wasm | probe.lifecycle | restore:restore.loaded | - | - | 198.8 (185.3–217.7) · p95 223.8 |
| wasm | probe.lifecycle | restore:restore.header | - | - | 1.0 (0.9–1.0) · p95 1.7 |
| wasm | probe.lifecycle | restore:restore.graph | - | - | 9.4 (8.5–10.4) · p95 13.1 |
| wasm | probe.lifecycle | restore:restore.states | - | - | 6.2 (5.6–6.9) · p95 8.1 |
| wasm | probe.lifecycle | restore:restore.records | - | - | 24.4 (21.7–26.2) · p95 31.4 |
| wasm | probe.lifecycle | restore:restore.bindings | - | - | 70.8 (66.4–78.4) · p95 82.1 |
| wasm | probe.lifecycle | restore:restore.done | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.lifecycle | restore:restore.drop_save | - | - | 4.9 (4.4–5.3) · p95 6.1 |
| wasm | probe.lifecycle | restore:exit | - | - | 4.1 (3.1–4.3) · p95 15.7 |
| wasm | probe.lifecycle | new:total | - | - | 2611.1 (2371.0–2815.6) · p95 3151.9 |
| wasm | probe.lifecycle | new:bind.stale | - | - | 0.2 (0.2–0.2) · p95 0.5 |
| wasm | probe.lifecycle | new:bind.eval | - | - | 32.9 (30.2–35.1) · p95 39.2 |
| wasm | probe.lifecycle | new:bind.sort | - | - | 3.9 (3.5–4.2) · p95 4.4 |
| wasm | probe.lifecycle | new:bind.explain | - | - | 4.0 (3.6–4.3) · p95 4.5 |
| wasm | probe.lifecycle | new:bind.write | - | - | 11.6 (10.1–12.2) · p95 15.7 |
| wasm | probe.lifecycle | new:load.enter | - | - | 14.8 (13.1–15.1) · p95 22.7 |
| wasm | probe.lifecycle | new:load.parse | - | - | 1136.5 (1029.2–1239.6) · p95 1329.8 |
| wasm | probe.lifecycle | new:load.expand | - | - | 315.4 (280.8–337.6) · p95 360.4 |
| wasm | probe.lifecycle | new:load.map | - | - | 30.9 (28.1–33.6) · p95 36.0 |
| wasm | probe.lifecycle | new:load.declare | - | - | 138.7 (128.9–149.8) · p95 150.3 |
| wasm | probe.lifecycle | new:load.procedures | - | - | 122.1 (114.5–133.1) · p95 143.1 |
| wasm | probe.lifecycle | new:load.validate | - | - | 592.0 (527.8–635.3) · p95 713.5 |
| wasm | probe.lifecycle | new:load.bindings | - | - | 0.5 (0.5–0.5) · p95 0.7 |
| wasm | probe.lifecycle | new:exit | - | - | 206.8 (183.3–222.2) · p95 254.3 |

## ledger-session

The agent ledger replaying the real 2026-09-24 session log: 20 events, five of them refused by policy. Played 100 times from a fresh session. Program `experiments/agent-ledger/ledger.cav` (`2206ac79f23c`), stream `ef2143311f86`, 1 episode(s) × 100 = 2000 events per pass.

| Engine | Mode | Operation | main-local | i1 | i2 |
| --- | --- | --- | --- | --- | --- |
| native | apply | apply | 14.7 (14.6–14.9) · p95 21.4 | 14.9 (14.7–17.6) · p95 21.5 | - |
| native | web.dispatch_view | web.dispatch_view | 23.1 (22.7–24.4) · p95 31.4 | 24.8 (23.2–25.3) · p95 33.3 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | 56.9 (56.5–57.0) · p95 71.8 | 57.3 (56.0–57.3) · p95 71.2 | - |
| native | read | clone | 1.7 (1.7–1.8) · p95 1.9 | 1.7 (1.7–1.7) · p95 2.0 | - |
| native | read | clone.drop | 1.0 (1.0–1.0) · p95 1.1 | 1.0 (1.0–1.0) · p95 1.1 | - |
| native | read | save | 6.5 (6.3–6.9) · p95 8.8 | 6.7 (6.4–6.7) · p95 9.0 | - |
| native | read | save_json | 11.6 (11.4–12.4) · p95 16.2 | 11.9 (11.3–12.2) · p95 16.3 | - |
| native | read | snapshot | 17.7 (17.2–18.7) · p95 21.0 | 17.9 (17.2–18.1) · p95 20.5 | - |
| native | read | snapshot.drop | 7.1 (6.9–7.6) · p95 8.0 | 7.2 (6.9–7.3) · p95 8.0 | - |
| native | read | snapshot.serialize | 13.4 (13.2–14.4) · p95 16.6 | 13.4 (13.2–14.2) · p95 15.9 | - |
| native | read | snapshot.serialize_pretty | 27.0 (26.6–28.8) · p95 32.6 | 26.9 (26.5–27.2) · p95 31.6 | - |
| native | read | view | 3.1 (3.1–3.4) · p95 4.5 | 3.2 (3.1–3.2) · p95 4.6 | - |
| native | read | view.serialize | 2.3 (2.3–2.5) · p95 2.9 | 2.3 (2.3–2.4) · p95 2.9 | - |
| native | web.read | web.save | 12.7 (12.6–14.1) · p95 17.0 | 12.6 (12.4–13.0) · p95 17.2 | - |
| native | web.read | web.snapshot | 51.4 (51.0–57.6) · p95 61.4 | 51.2 (50.2–52.7) · p95 62.3 | - |
| native | web.read | web.view | 5.1 (5.1–5.8) · p95 7.4 | 5.1 (5.1–5.3) · p95 7.3 | - |
| native | lifecycle | from_source | 1371.0 (1314.3–1476.4) · p95 1503.7 | 1313.4 (1296.4–1437.6) · p95 1371.2 | - |
| native | lifecycle | restore | 1382.1 (1361.7–1523.1) · p95 1543.6 | 1384.4 (1367.4–1487.7) · p95 1433.8 | - |
| native | lifecycle | restore.parse | 23.1 (22.6–25.0) · p95 25.7 | 23.2 (22.5–24.7) · p95 26.0 | - |
| native | lifecycle | restore_json | 1399.1 (1397.4–1560.0) · p95 1585.2 | 1410.8 (1385.7–1522.5) · p95 1479.7 | - |
| native | lifecycle | save | 14.9 (14.8–16.0) · p95 18.9 | 14.7 (13.9–15.3) · p95 16.8 | - |
| native | lifecycle | save_json | 21.1 (20.7–23.9) · p95 24.0 | 21.2 (21.0–22.9) · p95 26.5 | - |
| native | lifecycle | web.new | 1332.3 (1302.3–1485.2) · p95 1489.2 | 1319.6 (1309.0–1429.3) · p95 1416.8 | - |
| native | lifecycle | web.restore | 1388.3 (1379.3–1553.0) · p95 1576.6 | 1390.3 (1357.3–1518.1) · p95 1486.4 | - |
| native | lifecycle | web.save | 15.9 (15.9–17.5) · p95 17.7 | 15.8 (15.7–17.4) · p95 17.2 | - |
| native | bench.dispatch_only | bench.dispatch_only | - | 16.4 (16.2–17.2) · p95 22.7 | - |
| native | bench.read | bench.snapshot_build | - | 23.7 (23.3–25.8) · p95 26.9 | - |
| native | bench.read | bench.view_build | - | 4.3 (4.3–4.8) · p95 5.6 | - |
| native | bench.read | web.snapshot | - | 49.5 (49.0–53.9) · p95 57.0 | - |
| native | bench.read | web.view | - | 6.0 (5.9–6.5) · p95 7.3 | - |
| native | probe.dispatch_view | apply.clone | - | - | 0.4 (0.4–0.5) · p95 0.6 |
| native | probe.dispatch_view | apply.rollback | - | - | 3.5 (3.4–3.6) · p95 5.1 |
| native | probe.dispatch_view | apply.swap | - | - | 2.5 (2.5–2.6) · p95 3.4 |
| native | probe.dispatch_view | apply.validate | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_view | bind.eval | - | - | 2.8 (2.7–3.1) · p95 3.1 |
| native | probe.dispatch_view | bind.explain | - | - | 0.3 (0.3–0.3) · p95 0.3 |
| native | probe.dispatch_view | bind.sort | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_view | bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | bind.write | - | - | 1.1 (1.1–1.1) · p95 1.6 |
| native | probe.dispatch_view | exit | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_view | payload.parse | - | - | 0.6 (0.6–0.6) · p95 1.2 |
| native | probe.dispatch_view | payload.resolve | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| native | probe.dispatch_view | run.changes | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| native | probe.dispatch_view | run.clock | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.dispatch_view | run.prologue | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_view | run.rules | - | - | 11.8 (11.6–12.1) · p95 14.1 |
| native | probe.dispatch_view | total | - | - | 24.1 (23.8–26.3) · p95 32.4 |
| native | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.commitments | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.dispatch_view | view.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.dispatch_view | view.names | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.dispatch_view | view.relations | - | - | 0.9 (0.9–1.0) · p95 1.8 |
| native | probe.dispatch_view | view.sort | - | - | 1.9 (1.9–2.0) · p95 2.8 |
| native | probe.dispatch_view | web.drop | - | - | 0.4 (0.4–0.4) · p95 0.7 |
| native | probe.dispatch_view | web.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.dispatch_view | web.serialize | - | - | 2.1 (2.0–2.2) · p95 2.7 |
| native | probe.dispatch_outcome | apply.clone | - | - | 0.4 (0.4–0.5) · p95 0.5 |
| native | probe.dispatch_outcome | apply.rollback | - | - | 3.7 (3.7–4.1) · p95 5.3 |
| native | probe.dispatch_outcome | apply.swap | - | - | 2.7 (2.7–2.9) · p95 3.6 |
| native | probe.dispatch_outcome | apply.validate | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| native | probe.dispatch_outcome | bind.eval | - | - | 2.9 (2.9–3.1) · p95 3.2 |
| native | probe.dispatch_outcome | bind.explain | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.dispatch_outcome | bind.sort | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | bind.write | - | - | 1.3 (1.3–1.4) · p95 1.7 |
| native | probe.dispatch_outcome | exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.dispatch_outcome | outcome.built | - | - | 0.1 (0.1–0.1) · p95 0.3 |
| native | probe.dispatch_outcome | payload.parse | - | - | 0.8 (0.8–0.9) · p95 1.4 |
| native | probe.dispatch_outcome | payload.resolve | - | - | 0.4 (0.4–0.4) · p95 0.6 |
| native | probe.dispatch_outcome | run.changes | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| native | probe.dispatch_outcome | run.clock | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.dispatch_outcome | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.dispatch_outcome | run.rules | - | - | 12.5 (12.4–13.4) · p95 15.0 |
| native | probe.dispatch_outcome | snap.build | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_outcome | snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | snap.names | - | - | 0.5 (0.5–0.6) · p95 0.7 |
| native | probe.dispatch_outcome | snap.records | - | - | 2.6 (2.6–2.8) · p95 3.5 |
| native | probe.dispatch_outcome | snap.relations | - | - | 1.3 (1.2–1.4) · p95 1.9 |
| native | probe.dispatch_outcome | snap.shown | - | - | 1.2 (1.2–1.3) · p95 1.5 |
| native | probe.dispatch_outcome | snap.sort | - | - | 2.2 (2.1–2.3) · p95 2.8 |
| native | probe.dispatch_outcome | snap.static | - | - | 2.7 (2.7–3.0) · p95 3.3 |
| native | probe.dispatch_outcome | snap.symbols | - | - | 4.3 (4.3–4.6) · p95 6.1 |
| native | probe.dispatch_outcome | snap.values | - | - | 3.2 (3.2–3.4) · p95 4.3 |
| native | probe.dispatch_outcome | total | - | - | 57.5 (57.5–62.2) · p95 71.8 |
| native | probe.dispatch_outcome | web.drop | - | - | 6.5 (6.5–7.0) · p95 7.6 |
| native | probe.dispatch_outcome | web.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | web.serialize | - | - | 12.6 (12.5–13.6) · p95 16.4 |
| native | probe.read | save:exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | save:save.build | - | - | 6.3 (6.3–6.3) · p95 8.6 |
| native | probe.read | save:save.drop | - | - | 2.0 (2.0–2.0) · p95 2.7 |
| native | probe.read | save:save.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | save:save.serialize | - | - | 4.1 (4.1–4.1) · p95 5.4 |
| native | probe.read | save:total | - | - | 12.6 (12.5–12.7) · p95 16.7 |
| native | probe.read | save:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:exit | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.read | snapshot:snap.build | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.read | snapshot:snap.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:snap.names | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.read | snapshot:snap.records | - | - | 2.6 (2.6–2.6) · p95 3.3 |
| native | probe.read | snapshot:snap.relations | - | - | 1.2 (1.2–1.2) · p95 1.8 |
| native | probe.read | snapshot:snap.shown | - | - | 1.3 (1.3–1.3) · p95 1.6 |
| native | probe.read | snapshot:snap.sort | - | - | 1.2 (1.2–1.2) · p95 1.4 |
| native | probe.read | snapshot:snap.static | - | - | 2.8 (2.8–2.8) · p95 3.2 |
| native | probe.read | snapshot:snap.symbols | - | - | 4.0 (4.0–4.1) · p95 4.8 |
| native | probe.read | snapshot:snap.values | - | - | 3.2 (3.2–3.2) · p95 4.0 |
| native | probe.read | snapshot:total | - | - | 51.2 (50.8–51.5) · p95 59.6 |
| native | probe.read | snapshot:web.drop | - | - | 6.9 (6.8–6.9) · p95 7.7 |
| native | probe.read | snapshot:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:web.serialize | - | - | 26.8 (26.6–27.1) · p95 31.1 |
| native | probe.read | view:exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:total | - | - | 5.4 (5.3–5.4) · p95 7.6 |
| native | probe.read | view:view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.read | view:view.commitments | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| native | probe.read | view:view.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:view.names | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.read | view:view.relations | - | - | 1.0 (1.0–1.0) · p95 1.6 |
| native | probe.read | view:view.sort | - | - | 1.4 (1.4–1.5) · p95 1.9 |
| native | probe.read | view:web.drop | - | - | 0.4 (0.4–0.4) · p95 0.6 |
| native | probe.read | view:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:web.serialize | - | - | 1.6 (1.6–1.6) · p95 2.4 |
| native | probe.lifecycle | new:bind.eval | - | - | 3.5 (3.3–3.5) · p95 3.8 |
| native | probe.lifecycle | new:bind.explain | - | - | 0.5 (0.4–0.5) · p95 0.5 |
| native | probe.lifecycle | new:bind.sort | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.lifecycle | new:bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.lifecycle | new:bind.write | - | - | 2.3 (2.3–2.4) · p95 4.4 |
| native | probe.lifecycle | new:exit | - | - | 84.1 (81.8–92.4) · p95 109.9 |
| native | probe.lifecycle | new:load.bindings | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.lifecycle | new:load.declare | - | - | 152.4 (148.2–187.9) · p95 170.7 |
| native | probe.lifecycle | new:load.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.lifecycle | new:load.expand | - | - | 247.8 (247.1–251.9) · p95 281.5 |
| native | probe.lifecycle | new:load.map | - | - | 32.5 (29.4–37.5) · p95 40.7 |
| native | probe.lifecycle | new:load.parse | - | - | 645.5 (642.5–664.3) · p95 697.4 |
| native | probe.lifecycle | new:load.procedures | - | - | 0.9 (0.9–0.9) · p95 1.1 |
| native | probe.lifecycle | new:load.validate | - | - | 228.4 (217.8–233.3) · p95 243.3 |
| native | probe.lifecycle | new:total | - | - | 1403.3 (1391.9–1462.6) · p95 1512.2 |
| native | probe.lifecycle | restore:exit | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.lifecycle | restore:load.bindings | - | - | 7.0 (6.8–7.7) · p95 10.1 |
| native | probe.lifecycle | restore:load.declare | - | - | 169.7 (164.4–193.2) · p95 202.0 |
| native | probe.lifecycle | restore:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.lifecycle | restore:load.expand | - | - | 250.2 (247.7–253.2) · p95 279.3 |
| native | probe.lifecycle | restore:load.map | - | - | 36.2 (31.1–38.9) · p95 45.1 |
| native | probe.lifecycle | restore:load.parse | - | - | 693.4 (680.5–732.0) · p95 767.4 |
| native | probe.lifecycle | restore:load.procedures | - | - | 0.9 (0.9–1.0) · p95 1.2 |
| native | probe.lifecycle | restore:load.validate | - | - | 228.9 (221.1–250.5) · p95 245.7 |
| native | probe.lifecycle | restore:restore.bindings | - | - | 5.9 (5.8–5.9) · p95 6.2 |
| native | probe.lifecycle | restore:restore.done | - | - | 0.2 (0.2–0.3) · p95 0.3 |
| native | probe.lifecycle | restore:restore.drop_save | - | - | 5.8 (5.8–6.5) · p95 7.1 |
| native | probe.lifecycle | restore:restore.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.lifecycle | restore:restore.graph | - | - | 10.0 (9.9–10.2) · p95 11.0 |
| native | probe.lifecycle | restore:restore.header | - | - | 0.7 (0.7–0.7) · p95 0.8 |
| native | probe.lifecycle | restore:restore.loaded | - | - | 78.6 (76.4–83.1) · p95 96.3 |
| native | probe.lifecycle | restore:restore.parse | - | - | 22.9 (22.7–22.9) · p95 25.1 |
| native | probe.lifecycle | restore:restore.records | - | - | 16.9 (16.8–17.1) · p95 18.6 |
| native | probe.lifecycle | restore:restore.states | - | - | 4.0 (4.0–4.0) · p95 4.2 |
| native | probe.lifecycle | restore:total | - | - | 1559.1 (1524.9–1589.0) · p95 1656.6 |
| native | probe.lifecycle | restore:web.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.overhead | probe.mark | - | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | probe.overhead | probe.now | - | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.9 | 0.3 (0.3–0.3) · p95 0.9 | - |
| wasm | raw.dispatch_view | raw.dispatch_view | 28.4 (28.4–29.3) · p95 38.2 | 28.9 (28.3–29.2) · p95 39.6 | - |
| wasm | raw.dispatch_view | js.parse_view | 6.9 (6.8–7.1) · p95 9.9 | 7.1 (6.8–7.2) · p95 10.0 | - |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 38.5 (37.9–39.1) · p95 48.7 | 38.9 (37.7–39.4) · p95 50.3 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 63.9 (63.6–65.8) · p95 83.6 | 62.9 (62.7–66.4) · p95 79.5 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 45.1 (44.4–46.5) · p95 56.5 | 44.5 (44.3–46.9) · p95 54.7 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 108.7 (108.2–112.8) · p95 140.2 | 107.3 (107.2–114.0) · p95 133.7 | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.3 (0.3–0.3) · p95 0.6 | 0.3 (0.3–0.3) · p95 0.6 | - |
| wasm | abi.dispatch_view | abi.exec | 27.3 (27.2–27.3) · p95 37.1 | 27.4 (26.8–27.6) · p95 36.5 | - |
| wasm | abi.dispatch_view | abi.decode | 0.6 (0.6–0.6) · p95 1.8 | 0.6 (0.6–0.6) · p95 1.7 | - |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | - |
| wasm | abi.dispatch_view | abi.dispatch_view | 31.0 (30.9–31.8) · p95 39.6 | 30.8 (30.7–30.8) · p95 39.2 | - |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.3 (0.3–0.3) · p95 0.7 | 0.3 (0.3–0.3) · p95 0.7 | - |
| wasm | abi.dispatch_outcome | abi.exec | 59.2 (59.1–59.3) · p95 77.3 | 58.8 (58.5–59.2) · p95 75.4 | - |
| wasm | abi.dispatch_outcome | abi.decode | 2.3 (2.3–2.4) · p95 6.9 | 2.3 (2.3–2.3) · p95 6.2 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.4 (0.4–0.4) · p95 0.7 | 0.4 (0.4–0.4) · p95 0.8 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 63.0 (62.8–63.0) · p95 82.8 | 62.3 (61.9–62.7) · p95 80.0 | - |
| wasm | read | raw.view | 7.3 (7.1–7.5) · p95 10.2 | 7.3 (7.2–7.5) · p95 10.3 | - |
| wasm | read | js.parse_view | 8.1 (7.9–8.3) · p95 10.8 | 8.1 (8.0–8.4) · p95 10.8 | - |
| wasm | read | raw.snapshot | 69.6 (68.3–71.6) · p95 85.6 | 70.9 (68.5–71.9) · p95 87.1 | - |
| wasm | read | js.parse_snapshot | 51.9 (51.8–54.2) · p95 61.9 | 52.3 (52.2–53.8) · p95 63.7 | - |
| wasm | read | raw.save | 16.6 (16.3–17.1) · p95 23.1 | 16.5 (16.5–17.3) · p95 23.4 | - |
| wasm | abi.read | abi.view.exec | 6.7 (6.7–6.8) · p95 9.1 | 6.9 (6.7–7.3) · p95 9.5 | - |
| wasm | abi.read | abi.view.decode | 0.4 (0.4–0.4) · p95 0.5 | 0.4 (0.4–0.4) · p95 0.6 | - |
| wasm | abi.read | abi.snapshot.exec | 61.7 (60.9–62.7) · p95 73.9 | 63.4 (60.5–66.6) · p95 75.0 | - |
| wasm | abi.read | abi.snapshot.decode | 3.6 (3.4–3.7) · p95 6.3 | 3.6 (3.5–3.8) · p95 6.5 | - |
| wasm | abi.read | abi.save.exec | 14.9 (14.8–15.2) · p95 20.5 | 15.4 (14.7–16.1) · p95 21.3 | - |
| wasm | abi.read | abi.save.decode | 0.6 (0.6–0.6) · p95 0.9 | 0.6 (0.6–0.7) · p95 1.0 | - |
| wasm | lifecycle | raw.new | 1498.2 (1392.6–1517.4) · p95 1779.5 | 1529.7 (1508.7–1534.9) · p95 1668.6 | - |
| wasm | lifecycle | raw.save | 47.5 (46.0–51.7) · p95 70.1 | 48.1 (46.0–48.9) · p95 66.3 | - |
| wasm | lifecycle | raw.restore | 1581.5 (1494.7–1646.4) · p95 1909.6 | 1590.6 (1578.3–1648.1) · p95 1884.7 | - |
| wasm | lifecycle | kit.open | 1513.5 (1482.4–1570.4) · p95 1884.6 | 1509.4 (1498.8–1622.3) · p95 1785.8 | - |
| wasm | lifecycle | kit.save | 88.8 (85.6–92.3) · p95 119.3 | 88.5 (81.9–89.5) · p95 122.0 | - |
| wasm | lifecycle | kit.restore | 1520.1 (1483.4–1678.4) · p95 1891.7 | 1566.9 (1553.7–1646.4) · p95 1814.3 | - |
| wasm | abi.dispatch_only | abi.encode_args | - | 0.2 (0.2–0.2) · p95 0.5 | - |
| wasm | abi.dispatch_only | abi.dispatch_only.exec | - | 18.2 (18.2–18.9) · p95 26.1 | - |
| wasm | abi.dispatch_only | abi.dispatch_only | - | 18.7 (18.6–19.3) · p95 26.4 | - |
| wasm | abi.bench_read | abi.view_build.exec | - | 4.5 (4.4–4.6) · p95 5.8 | - |
| wasm | abi.bench_read | abi.snapshot_build.exec | - | 22.5 (22.3–22.9) · p95 27.8 | - |
| wasm | abi.bench_read | abi.view.exec | - | 7.3 (7.1–7.4) · p95 9.5 | - |
| wasm | abi.bench_read | abi.view.decode | - | 0.5 (0.5–0.5) · p95 1.3 | - |
| wasm | abi.bench_read | abi.snapshot.exec | - | 58.2 (57.5–59.3) · p95 69.3 | - |
| wasm | abi.bench_read | abi.snapshot.decode | - | 3.6 (3.5–3.7) · p95 6.6 | - |
| wasm | probe.dispatch_view | total | - | - | 38.3 (37.9–39.4) · p95 51.9 |
| wasm | probe.dispatch_view | web.enter | - | - | 0.3 (0.3–0.4) · p95 0.7 |
| wasm | probe.dispatch_view | payload.parse | - | - | 0.9 (0.9–1.0) · p95 2.1 |
| wasm | probe.dispatch_view | payload.resolve | - | - | 0.4 (0.4–0.5) · p95 0.8 |
| wasm | probe.dispatch_view | apply.validate | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.dispatch_view | apply.clone | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| wasm | probe.dispatch_view | run.clock | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_view | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| wasm | probe.dispatch_view | run.rules | - | - | 12.6 (12.3–12.7) · p95 16.4 |
| wasm | probe.dispatch_view | run.changes | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| wasm | probe.dispatch_view | bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | bind.eval | - | - | 3.0 (3.0–3.3) · p95 3.6 |
| wasm | probe.dispatch_view | bind.sort | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | bind.explain | - | - | 0.3 (0.3–0.4) · p95 0.4 |
| wasm | probe.dispatch_view | bind.write | - | - | 1.0 (1.0–1.1) · p95 1.5 |
| wasm | probe.dispatch_view | apply.swap | - | - | 3.7 (3.7–3.8) · p95 5.5 |
| wasm | probe.dispatch_view | apply.rollback | - | - | 3.8 (3.8–4.0) · p95 5.7 |
| wasm | probe.dispatch_view | view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | view.names | - | - | 0.8 (0.7–0.8) · p95 0.9 |
| wasm | probe.dispatch_view | view.sort | - | - | 2.0 (2.0–2.1) · p95 2.9 |
| wasm | probe.dispatch_view | view.commitments | - | - | 0.7 (0.7–0.7) · p95 0.8 |
| wasm | probe.dispatch_view | view.relations | - | - | 0.9 (0.9–0.9) · p95 1.6 |
| wasm | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | web.serialize | - | - | 3.4 (3.4–3.6) · p95 5.0 |
| wasm | probe.dispatch_view | web.drop | - | - | 0.6 (0.6–0.7) · p95 1.2 |
| wasm | probe.dispatch_view | exit | - | - | 0.9 (0.9–1.0) · p95 4.1 |
| wasm | probe.dispatch_view | js.parse | - | - | 7.3 (7.3–7.6) · p95 11.0 |
| wasm | probe.dispatch_outcome | total | - | - | 114.9 (114.3–116.1) · p95 145.5 |
| wasm | probe.dispatch_outcome | web.enter | - | - | 0.4 (0.4–0.4) · p95 0.9 |
| wasm | probe.dispatch_outcome | payload.parse | - | - | 1.1 (1.1–1.2) · p95 2.3 |
| wasm | probe.dispatch_outcome | payload.resolve | - | - | 0.5 (0.5–0.5) · p95 0.8 |
| wasm | probe.dispatch_outcome | apply.validate | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| wasm | probe.dispatch_outcome | apply.clone | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| wasm | probe.dispatch_outcome | run.clock | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| wasm | probe.dispatch_outcome | run.rules | - | - | 12.2 (12.2–12.3) · p95 16.2 |
| wasm | probe.dispatch_outcome | run.changes | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.dispatch_outcome | bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | bind.eval | - | - | 3.1 (3.1–3.1) · p95 3.7 |
| wasm | probe.dispatch_outcome | bind.sort | - | - | 0.1 (0.1–0.2) · p95 0.2 |
| wasm | probe.dispatch_outcome | bind.explain | - | - | 0.4 (0.4–0.4) · p95 0.4 |
| wasm | probe.dispatch_outcome | bind.write | - | - | 1.1 (1.1–1.1) · p95 1.6 |
| wasm | probe.dispatch_outcome | apply.swap | - | - | 4.3 (4.3–4.3) · p95 5.8 |
| wasm | probe.dispatch_outcome | apply.rollback | - | - | 4.1 (4.1–4.2) · p95 6.2 |
| wasm | probe.dispatch_outcome | snap.enter | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | snap.names | - | - | 0.8 (0.8–0.8) · p95 1.0 |
| wasm | probe.dispatch_outcome | snap.sort | - | - | 2.3 (2.3–2.3) · p95 3.1 |
| wasm | probe.dispatch_outcome | snap.symbols | - | - | 3.0 (2.9–3.1) · p95 4.2 |
| wasm | probe.dispatch_outcome | snap.shown | - | - | 1.1 (1.1–1.1) · p95 1.5 |
| wasm | probe.dispatch_outcome | snap.values | - | - | 2.8 (2.8–2.9) · p95 4.0 |
| wasm | probe.dispatch_outcome | snap.records | - | - | 1.8 (1.8–1.9) · p95 2.6 |
| wasm | probe.dispatch_outcome | snap.static | - | - | 1.6 (1.6–1.6) · p95 2.1 |
| wasm | probe.dispatch_outcome | snap.relations | - | - | 0.7 (0.6–0.7) · p95 1.2 |
| wasm | probe.dispatch_outcome | snap.build | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | outcome.built | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | web.serialize | - | - | 18.7 (18.5–18.8) · p95 24.3 |
| wasm | probe.dispatch_outcome | web.drop | - | - | 9.3 (9.2–9.3) · p95 11.5 |
| wasm | probe.dispatch_outcome | exit | - | - | 2.8 (2.8–2.9) · p95 5.9 |
| wasm | probe.dispatch_outcome | js.parse | - | - | 46.1 (46.0–46.8) · p95 58.6 |
| wasm | probe.read | view:total | - | - | 16.4 (16.1–16.6) · p95 21.8 |
| wasm | probe.read | view:web.enter | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.read | view:view.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| wasm | probe.read | view:view.names | - | - | 0.7 (0.7–0.7) · p95 0.9 |
| wasm | probe.read | view:view.sort | - | - | 1.5 (1.5–1.6) · p95 2.2 |
| wasm | probe.read | view:view.commitments | - | - | 0.7 (0.6–0.7) · p95 0.8 |
| wasm | probe.read | view:view.relations | - | - | 0.9 (0.9–0.9) · p95 1.6 |
| wasm | probe.read | view:view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | view:web.serialize | - | - | 2.6 (2.6–2.7) · p95 3.8 |
| wasm | probe.read | view:web.drop | - | - | 0.6 (0.6–0.6) · p95 1.0 |
| wasm | probe.read | view:exit | - | - | 0.6 (0.5–0.6) · p95 0.9 |
| wasm | probe.read | view:js.parse | - | - | 8.2 (8.1–8.5) · p95 11.0 |
| wasm | probe.read | snapshot:total | - | - | 124.0 (122.3–126.3) · p95 154.8 |
| wasm | probe.read | snapshot:web.enter | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.read | snapshot:snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | snapshot:snap.names | - | - | 0.8 (0.7–0.8) · p95 0.9 |
| wasm | probe.read | snapshot:snap.sort | - | - | 2.1 (2.0–2.1) · p95 2.7 |
| wasm | probe.read | snapshot:snap.symbols | - | - | 3.8 (3.7–3.8) · p95 4.7 |
| wasm | probe.read | snapshot:snap.shown | - | - | 1.3 (1.3–1.3) · p95 1.7 |
| wasm | probe.read | snapshot:snap.values | - | - | 3.2 (3.2–3.3) · p95 4.4 |
| wasm | probe.read | snapshot:snap.records | - | - | 2.0 (2.0–2.1) · p95 2.8 |
| wasm | probe.read | snapshot:snap.static | - | - | 1.7 (1.6–1.7) · p95 2.3 |
| wasm | probe.read | snapshot:snap.relations | - | - | 0.9 (0.8–0.9) · p95 1.4 |
| wasm | probe.read | snapshot:snap.build | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.read | snapshot:web.serialize | - | - | 39.3 (39.0–39.9) · p95 48.8 |
| wasm | probe.read | snapshot:web.drop | - | - | 10.3 (10.1–10.5) · p95 12.9 |
| wasm | probe.read | snapshot:exit | - | - | 4.3 (4.2–4.3) · p95 10.0 |
| wasm | probe.read | snapshot:js.parse | - | - | 52.7 (52.2–53.9) · p95 66.2 |
| wasm | probe.read | save:total | - | - | 17.2 (16.6–17.2) · p95 24.7 |
| wasm | probe.read | save:web.enter | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.read | save:save.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | save:save.build | - | - | 6.8 (6.6–6.8) · p95 9.8 |
| wasm | probe.read | save:save.serialize | - | - | 5.9 (5.7–5.9) · p95 8.3 |
| wasm | probe.read | save:save.drop | - | - | 3.2 (3.1–3.3) · p95 4.6 |
| wasm | probe.read | save:exit | - | - | 0.9 (0.8–0.9) · p95 1.8 |
| wasm | probe.lifecycle | restore:total | - | - | 1695.6 (1604.7–1925.0) · p95 2765.3 |
| wasm | probe.lifecycle | restore:web.enter | - | - | 13.0 (12.4–14.7) · p95 17.1 |
| wasm | probe.lifecycle | restore:restore.enter | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.lifecycle | restore:restore.parse | - | - | 47.7 (47.0–56.4) · p95 66.4 |
| wasm | probe.lifecycle | restore:load.enter | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.lifecycle | restore:load.parse | - | - | 733.6 (702.9–832.9) · p95 1056.9 |
| wasm | probe.lifecycle | restore:load.expand | - | - | 193.5 (190.6–232.2) · p95 301.2 |
| wasm | probe.lifecycle | restore:load.map | - | - | 40.4 (38.5–46.2) · p95 67.6 |
| wasm | probe.lifecycle | restore:load.declare | - | - | 137.7 (130.8–165.8) · p95 215.3 |
| wasm | probe.lifecycle | restore:load.procedures | - | - | 2.0 (1.9–2.3) · p95 4.0 |
| wasm | probe.lifecycle | restore:load.validate | - | - | 292.9 (280.9–355.6) · p95 609.0 |
| wasm | probe.lifecycle | restore:load.bindings | - | - | 10.4 (10.3–12.4) · p95 22.5 |
| wasm | probe.lifecycle | restore:restore.loaded | - | - | 105.2 (103.0–125.5) · p95 154.0 |
| wasm | probe.lifecycle | restore:restore.header | - | - | 1.0 (0.9–1.1) · p95 2.1 |
| wasm | probe.lifecycle | restore:restore.graph | - | - | 17.5 (17.4–21.9) · p95 33.7 |
| wasm | probe.lifecycle | restore:restore.states | - | - | 6.3 (6.1–7.9) · p95 10.3 |
| wasm | probe.lifecycle | restore:restore.records | - | - | 26.8 (26.6–32.9) · p95 44.2 |
| wasm | probe.lifecycle | restore:restore.bindings | - | - | 9.7 (9.5–11.7) · p95 17.5 |
| wasm | probe.lifecycle | restore:restore.done | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.lifecycle | restore:restore.drop_save | - | - | 6.7 (6.7–7.9) · p95 9.9 |
| wasm | probe.lifecycle | restore:exit | - | - | 4.7 (4.4–5.1) · p95 15.6 |
| wasm | probe.lifecycle | new:total | - | - | 1471.8 (1471.0–1745.9) · p95 2602.6 |
| wasm | probe.lifecycle | new:bind.stale | - | - | 0.4 (0.3–0.4) · p95 0.8 |
| wasm | probe.lifecycle | new:bind.eval | - | - | 6.8 (6.5–8.1) · p95 11.0 |
| wasm | probe.lifecycle | new:bind.sort | - | - | 0.4 (0.4–0.4) · p95 0.6 |
| wasm | probe.lifecycle | new:bind.explain | - | - | 0.6 (0.5–0.8) · p95 1.1 |
| wasm | probe.lifecycle | new:bind.write | - | - | 2.4 (2.2–2.8) · p95 3.9 |
| wasm | probe.lifecycle | new:load.enter | - | - | 8.2 (7.5–8.4) · p95 11.6 |
| wasm | probe.lifecycle | new:load.parse | - | - | 699.4 (694.3–822.9) · p95 1044.6 |
| wasm | probe.lifecycle | new:load.expand | - | - | 192.8 (190.6–223.4) · p95 316.2 |
| wasm | probe.lifecycle | new:load.map | - | - | 38.0 (37.7–46.3) · p95 68.4 |
| wasm | probe.lifecycle | new:load.declare | - | - | 132.2 (130.6–158.5) · p95 232.2 |
| wasm | probe.lifecycle | new:load.procedures | - | - | 2.0 (2.0–2.3) · p95 4.2 |
| wasm | probe.lifecycle | new:load.validate | - | - | 285.8 (274.2–346.2) · p95 546.0 |
| wasm | probe.lifecycle | new:load.bindings | - | - | 0.4 (0.4–0.5) · p95 0.7 |
| wasm | probe.lifecycle | new:exit | - | - | 113.1 (109.2–131.7) · p95 165.6 |
| wasm | probe.overhead | probe.mark | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| wasm | probe.overhead | js.performance_now | - | - | 0.0 (0.0–0.0) · p95 0.1 |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | main-local | i1 | i2 |
| --- | --- | --- | --- | --- | --- |
| native | apply | apply | 47.4 (47.0–47.9) · p95 121.6 | 47.4 (46.4–47.5) · p95 120.3 | - |
| native | web.dispatch_view | web.dispatch_view | 58.7 (58.4–60.4) · p95 132.6 | 59.3 (59.2–65.2) · p95 134.2 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | 118.3 (116.4–123.6) · p95 195.5 | 118.7 (118.0–119.8) · p95 196.7 | - |
| native | read | clone | 10.3 (10.2–10.5) · p95 11.9 | 10.4 (10.4–10.5) · p95 12.3 | - |
| native | read | clone.drop | 5.5 (5.5–5.6) · p95 6.4 | 5.6 (5.6–5.6) · p95 6.6 | - |
| native | read | save | 5.3 (5.3–5.4) · p95 9.1 | 5.4 (5.3–5.4) · p95 9.4 | - |
| native | read | save_json | 9.0 (8.9–9.1) · p95 15.5 | 9.1 (9.0–9.2) · p95 15.7 | - |
| native | read | snapshot | 29.8 (29.6–30.5) · p95 34.0 | 30.0 (29.9–30.2) · p95 34.7 | - |
| native | read | snapshot.drop | 11.9 (11.8–12.1) · p95 14.1 | 12.1 (12.1–12.2) · p95 14.4 | - |
| native | read | snapshot.serialize | 18.4 (18.3–18.8) · p95 21.1 | 18.9 (18.6–19.0) · p95 21.9 | - |
| native | read | snapshot.serialize_pretty | 37.4 (37.3–38.0) · p95 43.4 | 37.8 (37.2–38.0) · p95 44.3 | - |
| native | read | view | 1.3 (1.3–1.3) · p95 2.2 | 1.3 (1.3–1.3) · p95 2.3 | - |
| native | read | view.serialize | 6.5 (6.5–6.6) · p95 8.4 | 6.7 (6.6–6.7) · p95 8.3 | - |
| native | web.read | web.save | 10.7 (10.6–10.9) · p95 17.9 | 10.9 (10.8–10.9) · p95 18.2 | - |
| native | web.read | web.snapshot | 79.0 (78.9–80.1) · p95 91.4 | 79.6 (78.8–79.7) · p95 92.0 | - |
| native | web.read | web.view | 6.1 (6.0–6.1) · p95 8.5 | 6.1 (6.0–6.1) · p95 8.6 | - |
| native | lifecycle | from_source | 2922.3 (2821.6–3045.3) · p95 3033.7 | 2996.7 (2957.6–3119.6) · p95 3116.3 | - |
| native | lifecycle | restore | 3020.9 (2966.1–3135.0) · p95 3123.9 | 3101.9 (3055.0–3216.0) · p95 3205.5 | - |
| native | lifecycle | restore.parse | 20.4 (19.9–23.6) · p95 29.5 | 20.4 (19.1–20.5) · p95 25.2 | - |
| native | lifecycle | restore_json | 3029.1 (2976.5–3145.2) · p95 3133.4 | 3057.4 (3051.9–3194.4) · p95 3149.4 | - |
| native | lifecycle | save | 14.5 (14.3–16.6) · p95 20.5 | 13.3 (12.9–14.3) · p95 17.3 | - |
| native | lifecycle | save_json | 15.0 (14.8–16.0) · p95 18.3 | 15.9 (15.6–16.5) · p95 24.9 | - |
| native | lifecycle | web.new | 2912.5 (2857.0–3007.4) · p95 2997.7 | 2973.6 (2960.1–3092.8) · p95 3104.7 | - |
| native | lifecycle | web.restore | 3054.6 (2959.3–3142.3) · p95 3169.0 | 3079.1 (3043.0–3233.9) · p95 3183.2 | - |
| native | lifecycle | web.save | 10.8 (10.7–12.0) · p95 14.5 | 10.7 (10.5–10.8) · p95 13.3 | - |
| native | bench.dispatch_only | bench.dispatch_only | - | 50.7 (48.2–52.0) · p95 129.4 | - |
| native | bench.read | bench.snapshot_build | - | 40.1 (39.0–41.2) · p95 47.7 | - |
| native | bench.read | bench.view_build | - | 2.2 (2.1–2.2) · p95 3.8 | - |
| native | bench.read | web.snapshot | - | 74.6 (73.3–75.8) · p95 89.1 | - |
| native | bench.read | web.view | - | 9.0 (8.7–9.1) · p95 12.2 | - |
| native | probe.dispatch_view | apply.clone | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.dispatch_view | apply.rollback | - | - | 2.5 (2.5–2.7) · p95 5.8 |
| native | probe.dispatch_view | apply.swap | - | - | 2.3 (2.3–2.4) · p95 3.1 |
| native | probe.dispatch_view | apply.validate | - | - | 0.2 (0.2–0.2) · p95 0.6 |
| native | probe.dispatch_view | bind.eval | - | - | 12.9 (12.8–13.8) · p95 28.0 |
| native | probe.dispatch_view | bind.explain | - | - | 4.3 (4.2–4.4) · p95 7.4 |
| native | probe.dispatch_view | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | bind.sort | - | - | 1.7 (1.7–1.8) · p95 2.7 |
| native | probe.dispatch_view | bind.stale | - | - | 1.2 (1.2–1.3) · p95 1.9 |
| native | probe.dispatch_view | bind.write | - | - | 7.9 (7.9–8.2) · p95 10.4 |
| native | probe.dispatch_view | exit | - | - | 0.1 (0.0–0.1) · p95 0.3 |
| native | probe.dispatch_view | payload.parse | - | - | 0.8 (0.8–0.9) · p95 2.6 |
| native | probe.dispatch_view | payload.resolve | - | - | 0.4 (0.4–0.5) · p95 1.0 |
| native | probe.dispatch_view | run.changes | - | - | 0.8 (0.8–0.9) · p95 1.6 |
| native | probe.dispatch_view | run.clock | - | - | 0.2 (0.2–0.2) · p95 5.9 |
| native | probe.dispatch_view | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| native | probe.dispatch_view | run.rules | - | - | 35.6 (35.2–37.0) · p95 76.1 |
| native | probe.dispatch_view | total | - | - | 60.2 (59.8–63.4) · p95 135.8 |
| native | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.commitments | - | - | 0.3 (0.3–0.4) · p95 1.0 |
| native | probe.dispatch_view | view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.names | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_view | view.relations | - | - | 0.7 (0.7–0.7) · p95 1.3 |
| native | probe.dispatch_view | view.sort | - | - | 0.5 (0.5–0.6) · p95 0.8 |
| native | probe.dispatch_view | web.drop | - | - | 0.4 (0.4–0.4) · p95 0.8 |
| native | probe.dispatch_view | web.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_view | web.serialize | - | - | 6.6 (6.6–7.0) · p95 8.7 |
| native | probe.dispatch_outcome | apply.clone | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.dispatch_outcome | apply.rollback | - | - | 2.7 (2.6–2.8) · p95 6.5 |
| native | probe.dispatch_outcome | apply.swap | - | - | 2.4 (2.4–2.4) · p95 3.4 |
| native | probe.dispatch_outcome | apply.validate | - | - | 0.3 (0.3–0.3) · p95 0.7 |
| native | probe.dispatch_outcome | bind.eval | - | - | 12.9 (12.7–13.2) · p95 28.5 |
| native | probe.dispatch_outcome | bind.explain | - | - | 4.4 (4.4–4.4) · p95 7.6 |
| native | probe.dispatch_outcome | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | bind.sort | - | - | 1.8 (1.8–1.9) · p95 2.9 |
| native | probe.dispatch_outcome | bind.stale | - | - | 1.3 (1.3–1.3) · p95 1.9 |
| native | probe.dispatch_outcome | bind.write | - | - | 8.5 (8.5–8.7) · p95 11.0 |
| native | probe.dispatch_outcome | exit | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | outcome.built | - | - | 0.1 (0.1–0.1) · p95 0.3 |
| native | probe.dispatch_outcome | payload.parse | - | - | 1.0 (1.0–1.0) · p95 2.7 |
| native | probe.dispatch_outcome | payload.resolve | - | - | 0.5 (0.5–0.5) · p95 1.0 |
| native | probe.dispatch_outcome | run.changes | - | - | 0.9 (0.9–0.9) · p95 1.6 |
| native | probe.dispatch_outcome | run.clock | - | - | 0.2 (0.2–0.2) · p95 6.3 |
| native | probe.dispatch_outcome | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| native | probe.dispatch_outcome | run.rules | - | - | 36.8 (35.5–37.8) · p95 78.5 |
| native | probe.dispatch_outcome | snap.build | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.dispatch_outcome | snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.3 |
| native | probe.dispatch_outcome | snap.names | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.dispatch_outcome | snap.records | - | - | 1.2 (1.2–1.3) · p95 2.7 |
| native | probe.dispatch_outcome | snap.relations | - | - | 0.7 (0.7–0.8) · p95 1.4 |
| native | probe.dispatch_outcome | snap.shown | - | - | 10.0 (9.9–10.2) · p95 12.0 |
| native | probe.dispatch_outcome | snap.sort | - | - | 0.7 (0.6–0.7) · p95 0.9 |
| native | probe.dispatch_outcome | snap.static | - | - | 2.5 (2.5–2.6) · p95 3.2 |
| native | probe.dispatch_outcome | snap.symbols | - | - | 2.6 (2.6–2.7) · p95 3.2 |
| native | probe.dispatch_outcome | snap.values | - | - | 12.2 (12.0–12.5) · p95 14.7 |
| native | probe.dispatch_outcome | total | - | - | 120.2 (116.5–121.8) · p95 199.6 |
| native | probe.dispatch_outcome | web.drop | - | - | 11.9 (11.7–12.2) · p95 14.6 |
| native | probe.dispatch_outcome | web.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | web.serialize | - | - | 19.7 (19.5–20.1) · p95 24.5 |
| native | probe.read | save:exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | save:save.build | - | - | 5.6 (5.5–5.6) · p95 9.4 |
| native | probe.read | save:save.drop | - | - | 1.8 (1.7–1.8) · p95 3.0 |
| native | probe.read | save:save.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | save:save.serialize | - | - | 3.6 (3.5–3.6) · p95 5.5 |
| native | probe.read | save:total | - | - | 11.1 (10.9–11.1) · p95 18.1 |
| native | probe.read | save:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:exit | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.read | snapshot:snap.build | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.read | snapshot:snap.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:snap.names | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.read | snapshot:snap.records | - | - | 1.1 (1.1–1.1) · p95 2.5 |
| native | probe.read | snapshot:snap.relations | - | - | 0.6 (0.6–0.6) · p95 1.3 |
| native | probe.read | snapshot:snap.shown | - | - | 9.8 (9.7–9.9) · p95 11.6 |
| native | probe.read | snapshot:snap.sort | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.read | snapshot:snap.static | - | - | 2.5 (2.5–2.5) · p95 3.0 |
| native | probe.read | snapshot:snap.symbols | - | - | 2.4 (2.4–2.5) · p95 2.9 |
| native | probe.read | snapshot:snap.values | - | - | 11.9 (11.8–12.0) · p95 14.2 |
| native | probe.read | snapshot:total | - | - | 80.3 (79.4–80.6) · p95 91.8 |
| native | probe.read | snapshot:web.drop | - | - | 12.0 (11.8–12.0) · p95 14.3 |
| native | probe.read | snapshot:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:web.serialize | - | - | 38.3 (37.8–38.3) · p95 43.7 |
| native | probe.read | view:exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:total | - | - | 6.4 (6.3–6.4) · p95 8.9 |
| native | probe.read | view:view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.read | view:view.commitments | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| native | probe.read | view:view.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:view.names | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.read | view:view.relations | - | - | 0.5 (0.5–0.5) · p95 1.2 |
| native | probe.read | view:view.sort | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| native | probe.read | view:web.drop | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| native | probe.read | view:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:web.serialize | - | - | 4.5 (4.4–4.5) · p95 5.9 |
| native | probe.lifecycle | new:bind.eval | - | - | 14.6 (14.5–14.8) · p95 15.4 |
| native | probe.lifecycle | new:bind.explain | - | - | 8.4 (7.8–8.9) · p95 10.2 |
| native | probe.lifecycle | new:bind.sort | - | - | 8.2 (8.1–8.2) · p95 8.5 |
| native | probe.lifecycle | new:bind.stale | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.lifecycle | new:bind.write | - | - | 27.1 (27.0–27.3) · p95 29.5 |
| native | probe.lifecycle | new:exit | - | - | 159.0 (149.6–162.3) · p95 185.0 |
| native | probe.lifecycle | new:load.bindings | - | - | 0.8 (0.8–0.8) · p95 0.9 |
| native | probe.lifecycle | new:load.declare | - | - | 313.3 (268.5–317.4) · p95 354.6 |
| native | probe.lifecycle | new:load.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.lifecycle | new:load.expand | - | - | 445.1 (432.9–446.9) · p95 483.9 |
| native | probe.lifecycle | new:load.map | - | - | 23.5 (22.4–23.7) · p95 26.1 |
| native | probe.lifecycle | new:load.parse | - | - | 1146.6 (1119.6–1169.7) · p95 1250.4 |
| native | probe.lifecycle | new:load.procedures | - | - | 260.3 (225.9–270.8) · p95 295.8 |
| native | probe.lifecycle | new:load.validate | - | - | 744.7 (741.3–765.7) · p95 806.5 |
| native | probe.lifecycle | new:total | - | - | 3140.7 (3071.9–3192.4) · p95 3295.5 |
| native | probe.lifecycle | restore:exit | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.lifecycle | restore:load.bindings | - | - | 60.5 (57.2–60.7) · p95 67.7 |
| native | probe.lifecycle | restore:load.declare | - | - | 233.8 (221.2–288.3) · p95 274.2 |
| native | probe.lifecycle | restore:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.lifecycle | restore:load.expand | - | - | 439.2 (435.8–460.7) · p95 488.0 |
| native | probe.lifecycle | restore:load.map | - | - | 19.7 (19.3–20.2) · p95 22.9 |
| native | probe.lifecycle | restore:load.parse | - | - | 1177.7 (1113.4–1183.1) · p95 1261.6 |
| native | probe.lifecycle | restore:load.procedures | - | - | 237.2 (220.8–241.1) · p95 262.2 |
| native | probe.lifecycle | restore:load.validate | - | - | 745.0 (739.0–749.6) · p95 777.8 |
| native | probe.lifecycle | restore:restore.bindings | - | - | 72.2 (71.4–72.4) · p95 77.0 |
| native | probe.lifecycle | restore:restore.done | - | - | 0.3 (0.2–0.3) · p95 0.3 |
| native | probe.lifecycle | restore:restore.drop_save | - | - | 3.4 (3.3–4.0) · p95 4.3 |
| native | probe.lifecycle | restore:restore.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.lifecycle | restore:restore.graph | - | - | 3.6 (3.5–3.6) · p95 4.0 |
| native | probe.lifecycle | restore:restore.header | - | - | 0.9 (0.9–0.9) · p95 1.1 |
| native | probe.lifecycle | restore:restore.loaded | - | - | 142.0 (134.3–143.3) · p95 153.6 |
| native | probe.lifecycle | restore:restore.parse | - | - | 16.2 (15.8–16.6) · p95 18.7 |
| native | probe.lifecycle | restore:restore.records | - | - | 11.1 (10.9–11.7) · p95 13.0 |
| native | probe.lifecycle | restore:restore.states | - | - | 8.5 (8.4–8.5) · p95 8.9 |
| native | probe.lifecycle | restore:total | - | - | 3203.1 (3087.8–3240.9) · p95 3366.1 |
| native | probe.lifecycle | restore:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.overhead | probe.mark | - | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | probe.overhead | probe.now | - | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.6 (0.5–0.6) · p95 1.7 | 0.6 (0.6–0.6) · p95 1.7 | - |
| wasm | raw.dispatch_view | raw.dispatch_view | 72.6 (71.0–75.0) · p95 148.0 | 74.0 (71.9–74.0) · p95 151.6 | - |
| wasm | raw.dispatch_view | js.parse_view | 25.4 (25.2–26.0) · p95 31.7 | 26.0 (25.5–26.0) · p95 32.5 | - |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 120.7 (118.5–123.4) · p95 179.1 | 122.8 (121.2–122.8) · p95 183.7 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 143.7 (140.1–145.8) · p95 230.3 | 141.3 (139.8–143.1) · p95 225.2 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 76.3 (74.7–76.7) · p95 91.6 | 74.4 (74.2–75.5) · p95 90.1 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 223.6 (217.6–224.4) · p95 317.6 | 217.7 (217.2–221.2) · p95 310.0 | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.4 (0.4–0.4) · p95 1.0 | 0.4 (0.4–0.4) · p95 1.0 | - |
| wasm | abi.dispatch_view | abi.exec | 68.3 (68.2–70.0) · p95 143.5 | 67.2 (67.1–69.2) · p95 141.8 | - |
| wasm | abi.dispatch_view | abi.decode | 1.2 (1.2–1.2) · p95 3.8 | 1.2 (1.2–1.2) · p95 3.6 | - |
| wasm | abi.dispatch_view | abi.free | 0.2 (0.2–0.2) · p95 0.3 | 0.2 (0.2–0.2) · p95 0.3 | - |
| wasm | abi.dispatch_view | abi.dispatch_view | 93.9 (93.2–95.7) · p95 148.5 | 93.2 (92.4–94.4) · p95 146.6 | - |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.4 (0.4–0.4) · p95 1.1 | 0.4 (0.4–0.4) · p95 1.1 | - |
| wasm | abi.dispatch_outcome | abi.exec | 129.6 (129.4–131.1) · p95 208.6 | 130.1 (129.8–132.9) · p95 210.7 | - |
| wasm | abi.dispatch_outcome | abi.decode | 3.1 (3.1–3.2) · p95 6.4 | 3.2 (3.2–3.3) · p95 6.3 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.4 (0.4–0.4) · p95 0.8 | 0.4 (0.4–0.4) · p95 0.8 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 135.0 (134.7–136.5) · p95 215.8 | 135.5 (135.1–138.4) · p95 217.5 | - |
| wasm | read | raw.view | 10.4 (10.1–10.4) · p95 14.1 | 9.8 (9.7–9.9) · p95 13.6 | - |
| wasm | read | js.parse_view | 25.6 (25.1–25.9) · p95 32.4 | 25.4 (25.0–26.0) · p95 32.2 | - |
| wasm | read | raw.snapshot | 106.8 (105.5–108.3) · p95 132.4 | 105.0 (103.5–106.0) · p95 131.1 | - |
| wasm | read | js.parse_snapshot | 80.1 (79.0–80.3) · p95 95.8 | 79.6 (78.4–80.6) · p95 96.1 | - |
| wasm | read | raw.save | 14.4 (14.3–14.5) · p95 25.0 | 14.3 (13.9–14.4) · p95 24.9 | - |
| wasm | abi.read | abi.view.exec | 9.0 (8.8–9.5) · p95 12.1 | 9.0 (8.9–9.6) · p95 12.4 | - |
| wasm | abi.read | abi.view.decode | 0.8 (0.8–0.9) · p95 1.2 | 0.9 (0.8–0.9) · p95 1.3 | - |
| wasm | abi.read | abi.snapshot.exec | 97.3 (95.4–99.1) · p95 116.3 | 98.7 (96.5–100.6) · p95 120.9 | - |
| wasm | abi.read | abi.snapshot.decode | 4.7 (4.7–4.7) · p95 8.0 | 4.8 (4.7–4.9) · p95 8.1 | - |
| wasm | abi.read | abi.save.exec | 13.1 (13.0–13.4) · p95 22.8 | 13.4 (13.0–13.8) · p95 23.4 | - |
| wasm | abi.read | abi.save.decode | 0.4 (0.4–0.5) · p95 0.8 | 0.5 (0.4–0.5) · p95 0.8 | - |
| wasm | lifecycle | raw.new | 2874.3 (2723.3–3282.9) · p95 3504.1 | 3276.0 (2961.6–3283.4) · p95 3473.1 | - |
| wasm | lifecycle | raw.save | 36.5 (35.6–41.5) · p95 58.4 | 40.1 (37.2–41.6) · p95 59.9 | - |
| wasm | lifecycle | raw.restore | 2782.4 (2757.5–3290.9) · p95 3542.8 | 3261.5 (2970.7–3288.9) · p95 3505.0 | - |
| wasm | lifecycle | kit.open | 2715.9 (2666.0–3200.3) · p95 3385.5 | 3196.0 (2910.3–3220.2) · p95 3376.9 | - |
| wasm | lifecycle | kit.save | 74.0 (71.5–82.4) · p95 107.1 | 73.0 (72.3–78.6) · p95 101.8 | - |
| wasm | lifecycle | kit.restore | 2807.1 (2789.5–3278.5) · p95 3476.9 | 3222.9 (2986.5–3275.4) · p95 3437.3 | - |
| wasm | abi.dispatch_only | abi.encode_args | - | 0.3 (0.3–0.3) · p95 0.8 | - |
| wasm | abi.dispatch_only | abi.dispatch_only.exec | - | 56.8 (53.8–56.9) · p95 138.6 | - |
| wasm | abi.dispatch_only | abi.dispatch_only | - | 57.1 (54.2–57.2) · p95 138.9 | - |
| wasm | abi.bench_read | abi.view_build.exec | - | 2.4 (2.4–2.5) · p95 4.0 | - |
| wasm | abi.bench_read | abi.snapshot_build.exec | - | 41.2 (40.2–43.4) · p95 52.0 | - |
| wasm | abi.bench_read | abi.view.exec | - | 12.3 (12.1–12.9) · p95 16.3 | - |
| wasm | abi.bench_read | abi.view.decode | - | 1.0 (1.0–1.1) · p95 2.5 | - |
| wasm | abi.bench_read | abi.snapshot.exec | - | 90.3 (88.2–96.4) · p95 111.5 | - |
| wasm | abi.bench_read | abi.snapshot.decode | - | 4.8 (4.7–5.0) · p95 8.1 | - |
| wasm | probe.dispatch_view | total | - | - | 102.4 (102.1–103.2) · p95 176.5 |
| wasm | probe.dispatch_view | web.enter | - | - | 0.5 (0.5–0.5) · p95 1.1 |
| wasm | probe.dispatch_view | payload.parse | - | - | 1.1 (1.1–1.2) · p95 3.2 |
| wasm | probe.dispatch_view | payload.resolve | - | - | 0.5 (0.5–0.5) · p95 1.1 |
| wasm | probe.dispatch_view | apply.validate | - | - | 0.3 (0.3–0.3) · p95 0.7 |
| wasm | probe.dispatch_view | apply.clone | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.dispatch_view | run.clock | - | - | 0.2 (0.2–0.2) · p95 6.4 |
| wasm | probe.dispatch_view | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| wasm | probe.dispatch_view | run.rules | - | - | 38.9 (38.8–39.1) · p95 79.7 |
| wasm | probe.dispatch_view | run.changes | - | - | 1.1 (1.1–1.2) · p95 1.8 |
| wasm | probe.dispatch_view | bind.stale | - | - | 1.6 (1.6–1.6) · p95 2.2 |
| wasm | probe.dispatch_view | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | bind.eval | - | - | 14.0 (13.8–14.0) · p95 30.7 |
| wasm | probe.dispatch_view | bind.sort | - | - | 1.6 (1.6–1.6) · p95 2.6 |
| wasm | probe.dispatch_view | bind.explain | - | - | 4.4 (4.4–4.4) · p95 7.5 |
| wasm | probe.dispatch_view | bind.write | - | - | 7.2 (7.2–7.3) · p95 10.1 |
| wasm | probe.dispatch_view | apply.swap | - | - | 2.7 (2.6–2.7) · p95 4.4 |
| wasm | probe.dispatch_view | apply.rollback | - | - | 2.6 (2.5–2.6) · p95 6.6 |
| wasm | probe.dispatch_view | view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | view.names | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| wasm | probe.dispatch_view | view.sort | - | - | 0.8 (0.7–0.8) · p95 1.0 |
| wasm | probe.dispatch_view | view.commitments | - | - | 0.4 (0.4–0.4) · p95 1.0 |
| wasm | probe.dispatch_view | view.relations | - | - | 0.6 (0.6–0.6) · p95 1.3 |
| wasm | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | web.serialize | - | - | 9.8 (9.7–9.9) · p95 12.2 |
| wasm | probe.dispatch_view | web.drop | - | - | 0.5 (0.4–0.5) · p95 1.1 |
| wasm | probe.dispatch_view | exit | - | - | 1.6 (1.6–1.6) · p95 5.6 |
| wasm | probe.dispatch_view | js.parse | - | - | 25.6 (25.5–25.8) · p95 32.0 |
| wasm | probe.dispatch_outcome | total | - | - | 229.0 (226.7–229.1) · p95 316.1 |
| wasm | probe.dispatch_outcome | web.enter | - | - | 0.7 (0.7–0.7) · p95 1.4 |
| wasm | probe.dispatch_outcome | payload.parse | - | - | 1.5 (1.5–1.5) · p95 3.6 |
| wasm | probe.dispatch_outcome | payload.resolve | - | - | 0.5 (0.5–0.5) · p95 1.2 |
| wasm | probe.dispatch_outcome | apply.validate | - | - | 0.3 (0.3–0.3) · p95 0.7 |
| wasm | probe.dispatch_outcome | apply.clone | - | - | 0.4 (0.3–0.4) · p95 0.5 |
| wasm | probe.dispatch_outcome | run.clock | - | - | 0.2 (0.2–0.2) · p95 6.5 |
| wasm | probe.dispatch_outcome | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| wasm | probe.dispatch_outcome | run.rules | - | - | 39.8 (39.4–40.1) · p95 82.4 |
| wasm | probe.dispatch_outcome | run.changes | - | - | 1.2 (1.2–1.3) · p95 1.9 |
| wasm | probe.dispatch_outcome | bind.stale | - | - | 1.7 (1.7–1.7) · p95 2.3 |
| wasm | probe.dispatch_outcome | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | bind.eval | - | - | 14.3 (14.3–14.4) · p95 31.8 |
| wasm | probe.dispatch_outcome | bind.sort | - | - | 1.7 (1.6–1.7) · p95 2.8 |
| wasm | probe.dispatch_outcome | bind.explain | - | - | 4.5 (4.5–4.6) · p95 7.9 |
| wasm | probe.dispatch_outcome | bind.write | - | - | 7.5 (7.4–7.5) · p95 10.5 |
| wasm | probe.dispatch_outcome | apply.swap | - | - | 2.9 (2.9–3.0) · p95 4.6 |
| wasm | probe.dispatch_outcome | apply.rollback | - | - | 2.8 (2.7–2.8) · p95 7.3 |
| wasm | probe.dispatch_outcome | snap.enter | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | snap.names | - | - | 0.4 (0.4–0.4) · p95 0.6 |
| wasm | probe.dispatch_outcome | snap.sort | - | - | 0.8 (0.8–0.9) · p95 1.1 |
| wasm | probe.dispatch_outcome | snap.symbols | - | - | 2.7 (2.7–2.7) · p95 3.5 |
| wasm | probe.dispatch_outcome | snap.shown | - | - | 8.7 (8.6–8.8) · p95 10.7 |
| wasm | probe.dispatch_outcome | snap.values | - | - | 12.3 (12.1–12.3) · p95 15.8 |
| wasm | probe.dispatch_outcome | snap.records | - | - | 1.0 (1.0–1.1) · p95 1.8 |
| wasm | probe.dispatch_outcome | snap.static | - | - | 1.7 (1.6–1.7) · p95 2.3 |
| wasm | probe.dispatch_outcome | snap.relations | - | - | 0.5 (0.5–0.5) · p95 1.0 |
| wasm | probe.dispatch_outcome | snap.build | - | - | 0.1 (0.1–0.2) · p95 0.2 |
| wasm | probe.dispatch_outcome | outcome.built | - | - | 0.2 (0.1–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | web.serialize | - | - | 28.6 (28.5–28.7) · p95 35.5 |
| wasm | probe.dispatch_outcome | web.drop | - | - | 19.4 (19.3–19.5) · p95 24.5 |
| wasm | probe.dispatch_outcome | exit | - | - | 3.8 (3.7–3.8) · p95 7.1 |
| wasm | probe.dispatch_outcome | js.parse | - | - | 76.5 (75.5–76.8) · p95 92.5 |
| wasm | probe.read | view:total | - | - | 37.2 (36.9–37.9) · p95 47.6 |
| wasm | probe.read | view:web.enter | - | - | 0.3 (0.2–0.3) · p95 0.6 |
| wasm | probe.read | view:view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | view:view.names | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| wasm | probe.read | view:view.sort | - | - | 0.4 (0.4–0.4) · p95 0.7 |
| wasm | probe.read | view:view.commitments | - | - | 0.4 (0.3–0.4) · p95 0.7 |
| wasm | probe.read | view:view.relations | - | - | 0.5 (0.5–0.5) · p95 1.2 |
| wasm | probe.read | view:view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | view:web.serialize | - | - | 7.2 (7.1–7.4) · p95 9.4 |
| wasm | probe.read | view:web.drop | - | - | 0.4 (0.4–0.4) · p95 0.9 |
| wasm | probe.read | view:exit | - | - | 1.1 (1.1–1.1) · p95 2.3 |
| wasm | probe.read | view:js.parse | - | - | 26.2 (25.9–26.6) · p95 33.4 |
| wasm | probe.read | snapshot:total | - | - | 192.5 (190.5–195.6) · p95 234.0 |
| wasm | probe.read | snapshot:web.enter | - | - | 0.3 (0.2–0.3) · p95 0.4 |
| wasm | probe.read | snapshot:snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | snapshot:snap.names | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| wasm | probe.read | snapshot:snap.sort | - | - | 0.4 (0.4–0.4) · p95 0.7 |
| wasm | probe.read | snapshot:snap.symbols | - | - | 2.6 (2.6–2.7) · p95 3.5 |
| wasm | probe.read | snapshot:snap.shown | - | - | 8.3 (8.3–8.5) · p95 10.9 |
| wasm | probe.read | snapshot:snap.values | - | - | 12.8 (12.4–12.8) · p95 16.2 |
| wasm | probe.read | snapshot:snap.records | - | - | 1.0 (0.9–1.0) · p95 1.8 |
| wasm | probe.read | snapshot:snap.static | - | - | 1.8 (1.7–1.8) · p95 2.3 |
| wasm | probe.read | snapshot:snap.relations | - | - | 0.5 (0.5–0.5) · p95 1.0 |
| wasm | probe.read | snapshot:snap.build | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| wasm | probe.read | snapshot:web.serialize | - | - | 55.7 (54.8–56.4) · p95 67.6 |
| wasm | probe.read | snapshot:web.drop | - | - | 19.3 (19.0–19.6) · p95 24.7 |
| wasm | probe.read | snapshot:exit | - | - | 5.4 (5.4–5.5) · p95 18.3 |
| wasm | probe.read | snapshot:js.parse | - | - | 81.9 (81.1–83.5) · p95 100.1 |
| wasm | probe.read | save:total | - | - | 15.4 (15.1–15.6) · p95 25.6 |
| wasm | probe.read | save:web.enter | - | - | 0.3 (0.2–0.3) · p95 0.5 |
| wasm | probe.read | save:save.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | save:save.build | - | - | 6.2 (6.1–6.3) · p95 10.4 |
| wasm | probe.read | save:save.serialize | - | - | 5.3 (5.1–5.3) · p95 8.4 |
| wasm | probe.read | save:save.drop | - | - | 2.8 (2.7–2.8) · p95 5.2 |
| wasm | probe.read | save:exit | - | - | 0.8 (0.8–0.8) · p95 1.4 |
| wasm | probe.lifecycle | restore:total | - | - | 3128.9 (3017.7–3147.1) · p95 3478.0 |
| wasm | probe.lifecycle | restore:web.enter | - | - | 12.6 (12.0–12.6) · p95 16.2 |
| wasm | probe.lifecycle | restore:restore.enter | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.lifecycle | restore:restore.parse | - | - | 30.1 (28.9–30.8) · p95 41.1 |
| wasm | probe.lifecycle | restore:load.enter | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.lifecycle | restore:load.parse | - | - | 1171.8 (1122.3–1175.2) · p95 1276.9 |
| wasm | probe.lifecycle | restore:load.expand | - | - | 293.8 (282.1–296.7) · p95 345.6 |
| wasm | probe.lifecycle | restore:load.map | - | - | 23.8 (23.8–24.1) · p95 31.4 |
| wasm | probe.lifecycle | restore:load.declare | - | - | 177.8 (175.7–179.6) · p95 217.0 |
| wasm | probe.lifecycle | restore:load.procedures | - | - | 145.9 (137.8–148.4) · p95 163.4 |
| wasm | probe.lifecycle | restore:load.validate | - | - | 882.9 (865.0–892.8) · p95 980.2 |
| wasm | probe.lifecycle | restore:load.bindings | - | - | 64.0 (62.1–64.6) · p95 81.2 |
| wasm | probe.lifecycle | restore:restore.loaded | - | - | 196.0 (192.2–198.4) · p95 218.6 |
| wasm | probe.lifecycle | restore:restore.header | - | - | 1.1 (1.1–1.2) · p95 1.6 |
| wasm | probe.lifecycle | restore:restore.graph | - | - | 5.1 (5.1–5.2) · p95 6.2 |
| wasm | probe.lifecycle | restore:restore.states | - | - | 11.5 (11.3–11.8) · p95 13.4 |
| wasm | probe.lifecycle | restore:restore.records | - | - | 15.6 (15.3–15.8) · p95 20.9 |
| wasm | probe.lifecycle | restore:restore.bindings | - | - | 85.9 (82.9–86.9) · p95 127.4 |
| wasm | probe.lifecycle | restore:restore.done | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| wasm | probe.lifecycle | restore:restore.drop_save | - | - | 4.4 (4.4–4.5) · p95 4.9 |
| wasm | probe.lifecycle | restore:exit | - | - | 4.4 (4.2–4.4) · p95 10.3 |
| wasm | probe.lifecycle | new:total | - | - | 2973.1 (2860.5–3019.0) · p95 3292.2 |
| wasm | probe.lifecycle | new:bind.stale | - | - | 0.4 (0.4–0.4) · p95 0.7 |
| wasm | probe.lifecycle | new:bind.eval | - | - | 20.0 (18.8–20.2) · p95 28.6 |
| wasm | probe.lifecycle | new:bind.sort | - | - | 10.8 (10.2–10.8) · p95 12.1 |
| wasm | probe.lifecycle | new:bind.explain | - | - | 9.1 (8.5–9.2) · p95 15.3 |
| wasm | probe.lifecycle | new:bind.write | - | - | 22.8 (21.8–23.6) · p95 30.8 |
| wasm | probe.lifecycle | new:load.enter | - | - | 9.5 (9.3–9.5) · p95 11.7 |
| wasm | probe.lifecycle | new:load.parse | - | - | 1162.7 (1097.7–1177.7) · p95 1260.3 |
| wasm | probe.lifecycle | new:load.expand | - | - | 294.8 (288.1–295.5) · p95 328.0 |
| wasm | probe.lifecycle | new:load.map | - | - | 24.1 (23.2–24.5) · p95 29.3 |
| wasm | probe.lifecycle | new:load.declare | - | - | 175.0 (167.4–178.4) · p95 192.5 |
| wasm | probe.lifecycle | new:load.procedures | - | - | 144.1 (138.7–146.4) · p95 171.1 |
| wasm | probe.lifecycle | new:load.validate | - | - | 886.7 (826.8–928.2) · p95 1026.8 |
| wasm | probe.lifecycle | new:load.bindings | - | - | 0.8 (0.8–0.9) · p95 1.2 |
| wasm | probe.lifecycle | new:exit | - | - | 200.3 (185.4–201.2) · p95 232.4 |
| wasm | probe.overhead | probe.mark | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| wasm | probe.overhead | js.performance_now | - | - | 0.0 (0.0–0.0) · p95 0.1 |

