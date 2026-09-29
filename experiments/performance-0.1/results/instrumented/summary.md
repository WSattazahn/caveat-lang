# Caveat performance baseline: 2026-09-29T03-28-17-772-instrumented

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=instrumented --repeats=3 --build=never --keep-samples --target=main-local=tree:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF --target=i1=tree:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\perf-scratch\m\i1 --target=i2=tree:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\perf-scratch\m\i2 --out=C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\perf-scratch\m\runs\instrumented`

## Method

- Suite `instrumented`: 3 repeat(s) of every mode, each in a fresh process; within a repeat every target runs a mode back to back, the first target rotating.
- Per-event modes: 1 untimed warm-up pass(es), then 3 timed pass(es), each episode on a freshly opened session. `published-method` runs 3 round(s) with no warm-up, as the published harness did.
- Load, save and restore: 3 untimed then 30 timed calls; published resume method: 3 run(s).
- Statistic: quantile q(p) = sorted[floor(p·(n−1))] over the pooled samples of a run (the published rule; the median is the lower median). A cell is the median over runs of each run's pooled median, with the lowest and highest run in brackets, then the median over runs of each run's p95.
- Units: microseconds per operation unless the operation says otherwise.

## Targets

| Target | Kind | Revision | Reactive WebAssembly sha256 | Glue sha256 | Kit session.mjs sha256 | Native benchmark |
| --- | --- | --- | --- | --- | --- | --- |
| main-local | tree | v0.1.0-rc.4-5-gdbfd2e3-dirty (dbfd2e3), tracked changes | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |
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
- Load before: total CPU 29/6/23%; busiest over the window: ChatGPT 3.422s, ChatGPT 1.156s, claude 0.938s, powershell 0.828s, claude 0.266s, IntelGraphicsSoftware 0.188s
- Load after: total CPU 16/0/0%; busiest over the window: claude 1.812s, claude 0.625s, powershell 0.562s, IntelGraphicsSoftware 0.188s, ChatGPT 0.172s, ChatGPT 0.125s
- Load during: typeperf every 5 s for the whole run, 289 samples: total mean 11.3% p95 14.9% max 21.1%; processor 10 mean 63.8% p95 99.4% max 100%; processor 11 mean 15.3% p95 48.9% max 100%; processor 12 mean 21.9% p95 51.9% max 71%; processor 13 mean 20.8% p95 99.1% max 100%; total above 10% in 233 and above 25% in 0 samples; pinned processors summing above 150% in 15

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
| JSON.stringify(payload) | 0.3 | 0.4 | - |
| wasm-bindgen dispatch_view call, returning the view text | 27.8 | 28.1 | - |
|   arguments copied into WebAssembly memory | 0.1 | 0.1 | - |
|   WebAssembly execution (resolve, apply, view, serialize) | 24.8 | 24.2 | - |
|   result decoded to a JavaScript string | 2.9 | 2.8 | - |
|   result freed | 0.1 | 0.1 | - |
|   the four pieces, timed together in one process | 28.0 | 27.3 | - |
| JSON.parse(view text) | 18.2 | 18.6 | - |
| WebAssembly view() alone, execution only | 9.6 | 9.4 | - |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | - | - | - |
| Kit dispatch + view() | - | - | - |
| Native web::dispatch_view (the exported function, natively) | 22.7 | 22.8 | - |
| Native apply (numeric parameters: rules and bindings only) | 14.9 | 14.9 | - |
| Native dispatch_view_json (resolve + apply + view, not serialized) | - | - | - |
| Native view() build | 2.8 | 2.8 | - |
| Native view serialize | 5.4 | 5.2 | - |
| Native snapshot() build | 28.5 | 28.2 | - |
| Native session clone (upper bound of the transaction copy) | 8.1 | 8.1 | - |
| Native apply, the same program without bindings (glowcap-unbound) | 15.3 | 13.9 | - |
| *derived:* native resolve_payload ≈ dispatch_view_json − apply − view | - | - | - |
| *derived:* native binding evaluation ≈ apply − apply without bindings | -0.4 | 1.0 | - |
| *derived:* WebAssembly ÷ native, same exported function (ratio) | 1.1 | 1.1 | - |

## glowcap-replay

The Glowcap replay program and the exact event stream behind the published 51.6 us figure (experiments/glowcap/harness.mjs bench(), runtime/examples/profile_dispatch.rs): absorb cave glowcap, absorb pool duskcap, taste ruin glowcap, then 9,997 ticks of dt 0.05. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | main-local | i1 | i2 |
| --- | --- | --- | --- | --- | --- |
| native | apply | apply | 15.0 (14.3–15.4) · p95 18.6 | 15.0 (14.4–15.6) · p95 17.9 | - |
| native | web.dispatch_view | web.dispatch_view | 22.7 (21.9–23.5) · p95 26.1 | 22.8 (22.0–24.6) · p95 26.0 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | 75.1 (69.6–77.6) · p95 83.8 | 72.2 (69.0–73.3) · p95 79.3 | - |
| native | read | clone | 8.0 (7.8–8.2) · p95 9.3 | 8.0 (7.8–8.4) · p95 9.3 | - |
| native | read | clone.drop | 4.4 (4.2–4.5) · p95 4.9 | 4.3 (4.2–4.6) · p95 5.0 | - |
| native | read | save | 10.2 (10.0–10.7) · p95 11.9 | 10.2 (10.0–10.8) · p95 12.1 | - |
| native | read | save_json | 16.7 (16.1–17.2) · p95 19.1 | 16.6 (16.1–17.4) · p95 19.6 | - |
| native | read | snapshot | 28.3 (27.6–29.2) · p95 32.7 | 28.0 (27.2–29.3) · p95 33.1 | - |
| native | read | snapshot.drop | 12.1 (11.7–12.5) · p95 13.9 | 12.1 (11.7–12.6) · p95 14.1 | - |
| native | read | snapshot.serialize | 17.0 (16.7–17.6) · p95 19.6 | 16.8 (16.2–17.6) · p95 19.8 | - |
| native | read | snapshot.serialize_pretty | 36.1 (35.1–37.2) · p95 41.2 | 35.5 (34.2–37.2) · p95 41.9 | - |
| native | read | view | 2.7 (2.7–2.8) · p95 3.2 | 2.7 (2.7–2.9) · p95 3.2 | - |
| native | read | view.serialize | 5.4 (5.3–5.5) · p95 6.2 | 5.2 (5.0–5.4) · p95 6.0 | - |
| native | web.read | web.save | 18.5 (18.1–18.8) · p95 21.4 | 18.1 (17.0–18.2) · p95 19.6 | - |
| native | web.read | web.snapshot | 77.6 (76.4–79.9) · p95 89.1 | 75.1 (71.0–76.0) · p95 82.4 | - |
| native | web.read | web.view | 7.5 (7.3–7.6) · p95 8.7 | 7.2 (6.9–7.3) · p95 7.8 | - |
| native | lifecycle | from_source | 3073.4 (2847.2–3225.1) · p95 3212.1 | 2886.9 (2809.6–2895.5) · p95 3056.7 | - |
| native | lifecycle | restore | 3237.4 (3012.2–3405.9) · p95 3335.5 | 3033.8 (2972.9–3037.4) · p95 3312.2 | - |
| native | lifecycle | restore.parse | 39.2 (38.6–41.5) · p95 48.5 | 37.6 (36.6–38.1) · p95 46.0 | - |
| native | lifecycle | restore_json | 3285.1 (3066.2–3411.2) · p95 3361.4 | 3095.7 (3076.1–3097.7) · p95 3243.6 | - |
| native | lifecycle | save | 37.6 (32.8–40.2) · p95 50.2 | 33.2 (31.3–39.5) · p95 45.6 | - |
| native | lifecycle | save_json | 30.6 (29.8–35.9) · p95 39.1 | 32.7 (26.6–33.8) · p95 40.1 | - |
| native | lifecycle | web.new | 3033.8 (2839.9–3187.8) · p95 3169.8 | 2862.8 (2792.7–2876.1) · p95 3017.3 | - |
| native | lifecycle | web.restore | 3313.7 (3099.3–3394.6) · p95 3419.4 | 3080.8 (3064.3–3127.1) · p95 3306.5 | - |
| native | lifecycle | web.save | 29.3 (26.8–29.5) · p95 34.7 | 27.3 (26.5–28.2) · p95 34.0 | - |
| native | bench.dispatch_only | bench.dispatch_only | - | 15.2 (14.6–15.9) · p95 18.2 | - |
| native | bench.read | bench.snapshot_build | - | 38.7 (36.6–40.2) · p95 41.4 | - |
| native | bench.read | bench.view_build | - | 4.2 (4.0–4.5) · p95 4.6 | - |
| native | bench.read | web.snapshot | - | 73.6 (69.3–76.3) · p95 78.6 | - |
| native | bench.read | web.view | - | 9.3 (8.8–9.7) · p95 10.0 | - |
| native | probe.dispatch_view | apply.clone | - | - | 0.4 (0.4–0.4) · p95 0.4 |
| native | probe.dispatch_view | apply.swap | - | - | 1.2 (1.2–1.3) · p95 1.4 |
| native | probe.dispatch_view | apply.validate | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_view | bind.eval | - | - | 0.9 (0.9–0.9) · p95 1.0 |
| native | probe.dispatch_view | bind.explain | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_view | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | bind.sort | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | bind.stale | - | - | 1.3 (1.2–1.3) · p95 1.4 |
| native | probe.dispatch_view | bind.write | - | - | 0.9 (0.9–0.9) · p95 1.1 |
| native | probe.dispatch_view | exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.dispatch_view | payload.parse | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_view | payload.resolve | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.dispatch_view | run.changes | - | - | 0.4 (0.4–0.4) · p95 0.4 |
| native | probe.dispatch_view | run.clock | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | run.prologue | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.dispatch_view | run.rules | - | - | 11.9 (11.5–12.2) · p95 14.0 |
| native | probe.dispatch_view | total | - | - | 23.6 (22.8–24.1) · p95 27.1 |
| native | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.commitments | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.dispatch_view | view.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.names | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| native | probe.dispatch_view | view.relations | - | - | 1.4 (1.3–1.4) · p95 1.5 |
| native | probe.dispatch_view | view.sort | - | - | 0.6 (0.6–0.6) · p95 0.7 |
| native | probe.dispatch_view | web.drop | - | - | 0.6 (0.6–0.6) · p95 0.7 |
| native | probe.dispatch_view | web.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | web.serialize | - | - | 3.9 (3.7–3.9) · p95 4.3 |
| native | probe.dispatch_outcome | apply.clone | - | - | 0.4 (0.3–0.4) · p95 0.4 |
| native | probe.dispatch_outcome | apply.swap | - | - | 1.3 (1.3–1.4) · p95 1.5 |
| native | probe.dispatch_outcome | apply.validate | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | bind.eval | - | - | 1.1 (1.0–1.1) · p95 1.2 |
| native | probe.dispatch_outcome | bind.explain | - | - | 0.2 (0.2–0.3) · p95 0.4 |
| native | probe.dispatch_outcome | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | bind.sort | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | bind.stale | - | - | 1.3 (1.2–1.4) · p95 1.5 |
| native | probe.dispatch_outcome | bind.write | - | - | 1.4 (1.4–1.5) · p95 1.6 |
| native | probe.dispatch_outcome | exit | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | outcome.built | - | - | 0.1 (0.1–0.1) · p95 0.3 |
| native | probe.dispatch_outcome | payload.parse | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.dispatch_outcome | payload.resolve | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_outcome | run.changes | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.dispatch_outcome | run.clock | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | run.prologue | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_outcome | run.rules | - | - | 12.9 (12.2–13.5) · p95 14.6 |
| native | probe.dispatch_outcome | snap.build | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_outcome | snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | snap.names | - | - | 0.4 (0.4–0.4) · p95 0.6 |
| native | probe.dispatch_outcome | snap.records | - | - | 4.3 (4.0–4.4) · p95 4.7 |
| native | probe.dispatch_outcome | snap.relations | - | - | 1.6 (1.5–1.7) · p95 1.9 |
| native | probe.dispatch_outcome | snap.shown | - | - | 7.6 (6.9–7.7) · p95 8.0 |
| native | probe.dispatch_outcome | snap.sort | - | - | 1.0 (0.9–1.0) · p95 1.2 |
| native | probe.dispatch_outcome | snap.static | - | - | 2.3 (2.2–2.5) · p95 2.5 |
| native | probe.dispatch_outcome | snap.symbols | - | - | 4.1 (3.9–4.2) · p95 4.5 |
| native | probe.dispatch_outcome | snap.values | - | - | 6.5 (6.1–6.7) · p95 6.9 |
| native | probe.dispatch_outcome | total | - | - | 74.4 (69.9–76.8) · p95 80.0 |
| native | probe.dispatch_outcome | web.drop | - | - | 11.8 (11.4–12.2) · p95 12.7 |
| native | probe.dispatch_outcome | web.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | web.serialize | - | - | 16.5 (15.6–17.3) · p95 18.0 |
| native | probe.read | save:exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | save:save.build | - | - | 9.8 (9.5–9.9) · p95 10.7 |
| native | probe.read | save:save.drop | - | - | 3.3 (3.1–3.3) · p95 3.6 |
| native | probe.read | save:save.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | save:save.serialize | - | - | 4.6 (4.4–4.6) · p95 5.1 |
| native | probe.read | save:total | - | - | 17.8 (17.2–18.0) · p95 19.2 |
| native | probe.read | save:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:exit | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.read | snapshot:snap.build | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.read | snapshot:snap.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:snap.names | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| native | probe.read | snapshot:snap.records | - | - | 4.1 (3.8–4.2) · p95 4.6 |
| native | probe.read | snapshot:snap.relations | - | - | 1.6 (1.5–1.6) · p95 1.8 |
| native | probe.read | snapshot:snap.shown | - | - | 7.3 (7.2–7.3) · p95 7.9 |
| native | probe.read | snapshot:snap.sort | - | - | 0.6 (0.6–0.6) · p95 0.7 |
| native | probe.read | snapshot:snap.static | - | - | 2.4 (2.3–2.4) · p95 2.6 |
| native | probe.read | snapshot:snap.symbols | - | - | 4.0 (3.9–4.0) · p95 4.5 |
| native | probe.read | snapshot:snap.values | - | - | 6.8 (6.5–6.9) · p95 7.4 |
| native | probe.read | snapshot:total | - | - | 74.6 (72.0–75.5) · p95 80.6 |
| native | probe.read | snapshot:web.drop | - | - | 11.9 (11.4–12.0) · p95 12.7 |
| native | probe.read | snapshot:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:web.serialize | - | - | 35.1 (33.7–35.5) · p95 37.8 |
| native | probe.read | view:exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:total | - | - | 7.5 (7.2–7.6) · p95 8.2 |
| native | probe.read | view:view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.read | view:view.commitments | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.read | view:view.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:view.names | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| native | probe.read | view:view.relations | - | - | 1.4 (1.3–1.4) · p95 1.6 |
| native | probe.read | view:view.sort | - | - | 0.7 (0.6–0.7) · p95 0.8 |
| native | probe.read | view:web.drop | - | - | 0.6 (0.6–0.7) · p95 0.7 |
| native | probe.read | view:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:web.serialize | - | - | 3.7 (3.6–3.8) · p95 4.1 |
| native | probe.lifecycle | new:bind.eval | - | - | 28.0 (27.7–29.9) · p95 29.7 |
| native | probe.lifecycle | new:bind.explain | - | - | 3.7 (3.5–3.7) · p95 5.1 |
| native | probe.lifecycle | new:bind.sort | - | - | 3.0 (3.0–3.1) · p95 3.3 |
| native | probe.lifecycle | new:bind.stale | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.lifecycle | new:bind.write | - | - | 10.8 (10.6–11.4) · p95 11.8 |
| native | probe.lifecycle | new:exit | - | - | 174.6 (165.8–178.2) · p95 216.4 |
| native | probe.lifecycle | new:load.bindings | - | - | 0.6 (0.6–0.7) · p95 0.7 |
| native | probe.lifecycle | new:load.declare | - | - | 242.6 (209.7–271.9) · p95 345.2 |
| native | probe.lifecycle | new:load.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.lifecycle | new:load.expand | - | - | 454.6 (439.5–488.9) · p95 506.7 |
| native | probe.lifecycle | new:load.map | - | - | 32.0 (24.6–36.0) · p95 37.8 |
| native | probe.lifecycle | new:load.parse | - | - | 1169.0 (1126.1–1207.1) · p95 1259.3 |
| native | probe.lifecycle | new:load.procedures | - | - | 252.9 (238.7–258.7) · p95 297.0 |
| native | probe.lifecycle | new:load.validate | - | - | 579.6 (551.4–586.7) · p95 644.3 |
| native | probe.lifecycle | new:total | - | - | 2968.8 (2866.9–3099.7) · p95 3117.8 |
| native | probe.lifecycle | restore:exit | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.lifecycle | restore:load.bindings | - | - | 53.2 (52.9–54.7) · p95 61.0 |
| native | probe.lifecycle | restore:load.declare | - | - | 292.0 (217.6–323.7) · p95 379.6 |
| native | probe.lifecycle | restore:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.lifecycle | restore:load.expand | - | - | 451.9 (435.2–478.1) · p95 498.3 |
| native | probe.lifecycle | restore:load.map | - | - | 23.1 (22.1–26.7) · p95 26.0 |
| native | probe.lifecycle | restore:load.parse | - | - | 1142.6 (1136.3–1195.9) · p95 1285.7 |
| native | probe.lifecycle | restore:load.procedures | - | - | 242.9 (230.1–285.5) · p95 305.6 |
| native | probe.lifecycle | restore:load.validate | - | - | 564.6 (563.1–586.7) · p95 617.3 |
| native | probe.lifecycle | restore:restore.bindings | - | - | 102.0 (99.9–104.0) · p95 109.2 |
| native | probe.lifecycle | restore:restore.done | - | - | 0.2 (0.2–0.3) · p95 0.3 |
| native | probe.lifecycle | restore:restore.drop_save | - | - | 5.9 (5.9–8.0) · p95 27.7 |
| native | probe.lifecycle | restore:restore.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.lifecycle | restore:restore.graph | - | - | 10.9 (10.8–11.5) · p95 11.6 |
| native | probe.lifecycle | restore:restore.header | - | - | 0.9 (0.9–1.0) · p95 1.0 |
| native | probe.lifecycle | restore:restore.loaded | - | - | 155.1 (154.5–166.8) · p95 182.5 |
| native | probe.lifecycle | restore:restore.parse | - | - | 36.1 (33.6–39.9) · p95 40.2 |
| native | probe.lifecycle | restore:restore.records | - | - | 30.6 (30.4–32.2) · p95 37.9 |
| native | probe.lifecycle | restore:restore.states | - | - | 9.2 (8.8–9.3) · p95 10.5 |
| native | probe.lifecycle | restore:total | - | - | 3157.7 (3056.4–3295.2) · p95 3397.8 |
| native | probe.lifecycle | restore:web.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.overhead | probe.mark | - | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | probe.overhead | probe.now | - | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.4) · p95 0.6 | 0.4 (0.3–0.4) · p95 0.6 | - |
| wasm | raw.dispatch_view | raw.dispatch_view | 28.0 (27.7–29.7) · p95 36.2 | 28.3 (28.1–28.7) · p95 34.9 | - |
| wasm | raw.dispatch_view | js.parse_view | 18.2 (18.2–18.4) · p95 23.1 | 18.7 (18.2–18.8) · p95 22.7 | - |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 46.8 (46.5–48.4) · p95 59.6 | 47.5 (46.8–48.1) · p95 57.7 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 80.7 (78.7–84.4) · p95 96.6 | 79.0 (78.1–87.1) · p95 98.4 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 70.6 (69.4–74.1) · p95 81.2 | 70.0 (68.9–76.5) · p95 82.8 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 151.7 (148.6–159.3) · p95 176.6 | 149.7 (147.7–164.3) · p95 182.1 | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.1 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.3 | - |
| wasm | abi.dispatch_view | abi.exec | 24.9 (24.7–25.8) · p95 32.1 | 24.3 (22.9–25.0) · p95 30.9 | - |
| wasm | abi.dispatch_view | abi.decode | 2.9 (2.7–2.9) · p95 3.8 | 2.8 (2.6–2.9) · p95 3.7 | - |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | - |
| wasm | abi.dispatch_view | abi.dispatch_view | 28.0 (27.7–29.0) · p95 36.2 | 27.4 (25.7–28.3) · p95 34.9 | - |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | - |
| wasm | abi.dispatch_outcome | abi.exec | 65.7 (65.1–67.3) · p95 78.2 | 64.8 (63.9–65.8) · p95 77.3 | - |
| wasm | abi.dispatch_outcome | abi.decode | 8.7 (8.6–8.9) · p95 12.4 | 8.6 (8.6–8.7) · p95 12.3 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 75.2 (74.5–76.8) · p95 90.1 | 74.2 (73.3–75.2) · p95 88.8 | - |
| wasm | read | raw.view | 12.5 (12.2–13.6) · p95 14.6 | 12.6 (12.2–12.6) · p95 14.7 | - |
| wasm | read | js.parse_view | 20.7 (20.3–22.4) · p95 24.1 | 20.4 (20.4–20.6) · p95 23.6 | - |
| wasm | read | raw.snapshot | 102.4 (101.5–112.9) · p95 123.6 | 103.0 (101.4–103.2) · p95 122.8 | - |
| wasm | read | js.parse_snapshot | 78.6 (78.1–85.3) · p95 92.3 | 78.4 (77.4–78.7) · p95 90.7 | - |
| wasm | read | raw.save | 21.1 (20.7–23.5) · p95 26.7 | 21.1 (20.5–21.1) · p95 26.5 | - |
| wasm | abi.read | abi.view.exec | 9.6 (9.2–10.1) · p95 11.1 | 9.3 (9.2–9.7) · p95 11.1 | - |
| wasm | abi.read | abi.view.decode | 2.6 (2.6–2.8) · p95 3.3 | 2.6 (2.6–2.8) · p95 3.3 | - |
| wasm | abi.read | abi.snapshot.exec | 80.1 (77.2–84.6) · p95 93.5 | 79.6 (78.1–82.7) · p95 96.4 | - |
| wasm | abi.read | abi.snapshot.decode | 14.1 (14.1–15.1) · p95 20.4 | 14.1 (14.1–14.8) · p95 20.4 | - |
| wasm | abi.read | abi.save.exec | 17.1 (16.7–18.2) · p95 21.0 | 17.0 (16.7–17.6) · p95 21.7 | - |
| wasm | abi.read | abi.save.decode | 0.7 (0.7–0.7) · p95 0.9 | 0.7 (0.7–0.7) · p95 1.0 | - |
| wasm | lifecycle | raw.new | 2768.3 (2706.0–3179.9) · p95 3296.0 | 3253.8 (3188.8–3255.6) · p95 3507.9 | - |
| wasm | lifecycle | raw.save | 55.3 (55.1–72.9) · p95 84.3 | 64.7 (64.3–65.7) · p95 88.3 | - |
| wasm | lifecycle | raw.restore | 2794.6 (2769.6–3315.0) · p95 3608.8 | 3325.6 (3283.0–3354.6) · p95 3609.3 | - |
| wasm | lifecycle | kit.open | 2691.9 (2617.3–3126.1) · p95 3281.1 | 3130.5 (3044.0–3138.2) · p95 3379.1 | - |
| wasm | lifecycle | kit.save | 123.7 (121.4–158.9) · p95 155.2 | 136.5 (130.3–144.7) · p95 176.6 | - |
| wasm | lifecycle | kit.restore | 2834.7 (2823.9–3264.3) · p95 3225.2 | 3289.5 (3157.2–3296.5) · p95 3579.3 | - |
| wasm | abi.dispatch_only | abi.encode_args | - | 0.1 (0.1–0.1) · p95 0.3 | - |
| wasm | abi.dispatch_only | abi.dispatch_only.exec | - | 14.7 (14.7–14.8) · p95 20.2 | - |
| wasm | abi.dispatch_only | abi.dispatch_only | - | 14.8 (14.8–14.9) · p95 20.5 | - |
| wasm | abi.bench_read | abi.view_build.exec | - | 3.9 (3.5–3.9) · p95 4.6 | - |
| wasm | abi.bench_read | abi.snapshot_build.exec | - | 32.3 (29.5–32.3) · p95 38.7 | - |
| wasm | abi.bench_read | abi.view.exec | - | 10.1 (9.4–10.4) · p95 11.8 | - |
| wasm | abi.bench_read | abi.view.decode | - | 2.7 (2.5–2.8) · p95 3.5 | - |
| wasm | abi.bench_read | abi.snapshot.exec | - | 79.5 (73.1–79.7) · p95 91.1 | - |
| wasm | abi.bench_read | abi.snapshot.decode | - | 14.1 (13.4–14.6) · p95 20.2 | - |
| wasm | probe.dispatch_view | total | - | - | 49.7 (49.2–49.7) · p95 62.7 |
| wasm | probe.dispatch_view | web.enter | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.dispatch_view | payload.parse | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| wasm | probe.dispatch_view | payload.resolve | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_view | apply.validate | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | apply.clone | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| wasm | probe.dispatch_view | run.clock | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | run.prologue | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| wasm | probe.dispatch_view | run.rules | - | - | 12.5 (12.3–12.6) · p95 15.9 |
| wasm | probe.dispatch_view | run.changes | - | - | 0.7 (0.6–0.7) · p95 0.8 |
| wasm | probe.dispatch_view | bind.stale | - | - | 1.7 (1.7–1.7) · p95 2.0 |
| wasm | probe.dispatch_view | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | bind.eval | - | - | 1.0 (1.0–1.0) · p95 1.5 |
| wasm | probe.dispatch_view | bind.sort | - | - | 0.1 (0.1–0.2) · p95 0.2 |
| wasm | probe.dispatch_view | bind.explain | - | - | 0.3 (0.2–0.3) · p95 0.4 |
| wasm | probe.dispatch_view | bind.write | - | - | 0.8 (0.8–0.8) · p95 1.3 |
| wasm | probe.dispatch_view | apply.swap | - | - | 0.8 (0.8–0.9) · p95 1.1 |
| wasm | probe.dispatch_view | view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | view.names | - | - | 0.6 (0.6–0.6) · p95 0.8 |
| wasm | probe.dispatch_view | view.sort | - | - | 0.7 (0.7–0.7) · p95 0.8 |
| wasm | probe.dispatch_view | view.commitments | - | - | 0.6 (0.6–0.7) · p95 0.8 |
| wasm | probe.dispatch_view | view.relations | - | - | 1.1 (1.1–1.1) · p95 1.5 |
| wasm | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | web.serialize | - | - | 6.2 (6.1–6.3) · p95 7.5 |
| wasm | probe.dispatch_view | web.drop | - | - | 0.5 (0.5–0.5) · p95 0.7 |
| wasm | probe.dispatch_view | exit | - | - | 3.0 (2.9–3.0) · p95 5.7 |
| wasm | probe.dispatch_view | js.parse | - | - | 19.2 (18.9–19.3) · p95 25.3 |
| wasm | probe.dispatch_outcome | total | - | - | 159.9 (157.0–161.5) · p95 232.2 |
| wasm | probe.dispatch_outcome | web.enter | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| wasm | probe.dispatch_outcome | payload.parse | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| wasm | probe.dispatch_outcome | payload.resolve | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.dispatch_outcome | apply.validate | - | - | 0.2 (0.1–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | apply.clone | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | run.clock | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | run.prologue | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | run.rules | - | - | 13.5 (13.1–13.9) · p95 20.6 |
| wasm | probe.dispatch_outcome | run.changes | - | - | 0.7 (0.7–0.7) · p95 1.2 |
| wasm | probe.dispatch_outcome | bind.stale | - | - | 1.8 (1.7–1.8) · p95 2.6 |
| wasm | probe.dispatch_outcome | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_outcome | bind.eval | - | - | 1.2 (1.1–1.2) · p95 1.6 |
| wasm | probe.dispatch_outcome | bind.sort | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | bind.explain | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| wasm | probe.dispatch_outcome | bind.write | - | - | 1.0 (0.9–1.0) · p95 1.5 |
| wasm | probe.dispatch_outcome | apply.swap | - | - | 1.2 (1.2–1.2) · p95 1.9 |
| wasm | probe.dispatch_outcome | snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | snap.names | - | - | 0.6 (0.6–0.7) · p95 0.9 |
| wasm | probe.dispatch_outcome | snap.sort | - | - | 0.8 (0.8–0.9) · p95 1.3 |
| wasm | probe.dispatch_outcome | snap.symbols | - | - | 3.3 (3.3–3.3) · p95 4.8 |
| wasm | probe.dispatch_outcome | snap.shown | - | - | 5.8 (5.7–5.8) · p95 8.0 |
| wasm | probe.dispatch_outcome | snap.values | - | - | 5.1 (5.1–5.2) · p95 7.7 |
| wasm | probe.dispatch_outcome | snap.records | - | - | 2.8 (2.7–2.8) · p95 3.8 |
| wasm | probe.dispatch_outcome | snap.static | - | - | 1.5 (1.4–1.5) · p95 2.1 |
| wasm | probe.dispatch_outcome | snap.relations | - | - | 1.3 (1.2–1.3) · p95 1.9 |
| wasm | probe.dispatch_outcome | snap.build | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | outcome.built | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | web.serialize | - | - | 23.2 (22.7–23.3) · p95 32.9 |
| wasm | probe.dispatch_outcome | web.drop | - | - | 13.4 (13.1–13.4) · p95 19.0 |
| wasm | probe.dispatch_outcome | exit | - | - | 9.3 (9.2–9.5) · p95 14.6 |
| wasm | probe.dispatch_outcome | js.parse | - | - | 72.7 (71.6–73.8) · p95 103.7 |
| wasm | probe.read | view:total | - | - | 35.5 (33.8–36.5) · p95 41.3 |
| wasm | probe.read | view:web.enter | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.read | view:view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | view:view.names | - | - | 0.6 (0.5–0.6) · p95 0.7 |
| wasm | probe.read | view:view.sort | - | - | 0.7 (0.7–0.8) · p95 1.0 |
| wasm | probe.read | view:view.commitments | - | - | 0.7 (0.7–0.8) · p95 0.9 |
| wasm | probe.read | view:view.relations | - | - | 1.4 (1.4–1.5) · p95 1.8 |
| wasm | probe.read | view:view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | view:web.serialize | - | - | 6.1 (5.7–6.3) · p95 7.1 |
| wasm | probe.read | view:web.drop | - | - | 0.7 (0.7–0.8) · p95 1.2 |
| wasm | probe.read | view:exit | - | - | 3.0 (2.9–3.1) · p95 4.1 |
| wasm | probe.read | view:js.parse | - | - | 21.7 (20.6–22.3) · p95 25.1 |
| wasm | probe.read | snapshot:total | - | - | 193.4 (185.1–202.1) · p95 223.9 |
| wasm | probe.read | snapshot:web.enter | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.read | snapshot:snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | snapshot:snap.names | - | - | 0.6 (0.6–0.7) · p95 0.8 |
| wasm | probe.read | snapshot:snap.sort | - | - | 0.9 (0.9–1.0) · p95 1.3 |
| wasm | probe.read | snapshot:snap.symbols | - | - | 4.5 (4.4–4.9) · p95 5.5 |
| wasm | probe.read | snapshot:snap.shown | - | - | 6.6 (6.4–6.7) · p95 7.7 |
| wasm | probe.read | snapshot:snap.values | - | - | 6.2 (5.9–6.5) · p95 7.8 |
| wasm | probe.read | snapshot:snap.records | - | - | 2.9 (2.8–3.0) · p95 3.6 |
| wasm | probe.read | snapshot:snap.static | - | - | 1.5 (1.5–1.5) · p95 1.9 |
| wasm | probe.read | snapshot:snap.relations | - | - | 1.3 (1.3–1.3) · p95 1.6 |
| wasm | probe.read | snapshot:snap.build | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.read | snapshot:web.serialize | - | - | 52.7 (49.9–54.5) · p95 62.3 |
| wasm | probe.read | snapshot:web.drop | - | - | 17.6 (17.0–18.9) · p95 21.2 |
| wasm | probe.read | snapshot:exit | - | - | 15.0 (14.5–15.7) · p95 19.9 |
| wasm | probe.read | snapshot:js.parse | - | - | 82.1 (78.7–85.6) · p95 95.0 |
| wasm | probe.read | save:total | - | - | 22.9 (22.0–24.4) · p95 28.8 |
| wasm | probe.read | save:web.enter | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.read | save:save.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | save:save.build | - | - | 10.0 (9.6–10.6) · p95 12.4 |
| wasm | probe.read | save:save.serialize | - | - | 6.6 (6.3–6.9) · p95 8.3 |
| wasm | probe.read | save:save.drop | - | - | 5.0 (4.9–5.5) · p95 6.2 |
| wasm | probe.read | save:exit | - | - | 1.0 (1.0–1.0) · p95 1.4 |
| wasm | probe.lifecycle | restore:total | - | - | 3485.3 (3071.7–3537.0) · p95 3918.6 |
| wasm | probe.lifecycle | restore:web.enter | - | - | 21.8 (18.9–22.3) · p95 31.1 |
| wasm | probe.lifecycle | restore:restore.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.lifecycle | restore:restore.parse | - | - | 46.5 (41.1–50.4) · p95 65.4 |
| wasm | probe.lifecycle | restore:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.lifecycle | restore:load.parse | - | - | 1381.1 (1221.3–1432.7) · p95 1487.4 |
| wasm | probe.lifecycle | restore:load.expand | - | - | 376.6 (339.5–390.2) · p95 452.5 |
| wasm | probe.lifecycle | restore:load.map | - | - | 37.3 (33.9–39.1) · p95 46.2 |
| wasm | probe.lifecycle | restore:load.declare | - | - | 167.2 (151.6–172.9) · p95 196.9 |
| wasm | probe.lifecycle | restore:load.procedures | - | - | 149.7 (136.0–154.6) · p95 173.5 |
| wasm | probe.lifecycle | restore:load.validate | - | - | 728.7 (634.1–790.8) · p95 887.3 |
| wasm | probe.lifecycle | restore:load.bindings | - | - | 63.2 (55.7–63.6) · p95 83.9 |
| wasm | probe.lifecycle | restore:restore.loaded | - | - | 237.4 (213.5–248.4) · p95 266.5 |
| wasm | probe.lifecycle | restore:restore.header | - | - | 1.3 (1.2–1.4) · p95 2.1 |
| wasm | probe.lifecycle | restore:restore.graph | - | - | 18.4 (16.4–18.6) · p95 23.1 |
| wasm | probe.lifecycle | restore:restore.states | - | - | 12.9 (11.7–13.4) · p95 14.9 |
| wasm | probe.lifecycle | restore:restore.records | - | - | 47.8 (42.9–49.2) · p95 58.4 |
| wasm | probe.lifecycle | restore:restore.bindings | - | - | 143.4 (129.5–151.3) · p95 174.5 |
| wasm | probe.lifecycle | restore:restore.done | - | - | 0.2 (0.2–0.3) · p95 0.4 |
| wasm | probe.lifecycle | restore:restore.drop_save | - | - | 9.5 (8.5–9.7) · p95 11.0 |
| wasm | probe.lifecycle | restore:exit | - | - | 5.8 (5.3–6.4) · p95 17.2 |
| wasm | probe.lifecycle | new:total | - | - | 3122.0 (2779.8–3194.3) · p95 3452.7 |
| wasm | probe.lifecycle | new:bind.stale | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.lifecycle | new:bind.eval | - | - | 38.6 (35.7–40.5) · p95 46.5 |
| wasm | probe.lifecycle | new:bind.sort | - | - | 4.5 (4.1–4.7) · p95 6.2 |
| wasm | probe.lifecycle | new:bind.explain | - | - | 4.7 (4.3–4.9) · p95 5.7 |
| wasm | probe.lifecycle | new:bind.write | - | - | 13.7 (12.4–14.3) · p95 20.7 |
| wasm | probe.lifecycle | new:load.enter | - | - | 16.8 (15.6–16.8) · p95 25.7 |
| wasm | probe.lifecycle | new:load.parse | - | - | 1343.4 (1203.7–1368.2) · p95 1520.6 |
| wasm | probe.lifecycle | new:load.expand | - | - | 381.2 (338.4–390.2) · p95 414.8 |
| wasm | probe.lifecycle | new:load.map | - | - | 37.7 (33.2–38.5) · p95 42.4 |
| wasm | probe.lifecycle | new:load.declare | - | - | 163.7 (148.7–171.5) · p95 181.9 |
| wasm | probe.lifecycle | new:load.procedures | - | - | 148.0 (132.0–154.4) · p95 163.3 |
| wasm | probe.lifecycle | new:load.validate | - | - | 714.1 (642.9–735.4) · p95 811.7 |
| wasm | probe.lifecycle | new:load.bindings | - | - | 0.6 (0.5–0.6) · p95 0.7 |
| wasm | probe.lifecycle | new:exit | - | - | 244.9 (220.5–255.6) · p95 281.5 |
| wasm | probe.overhead | probe.mark | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| wasm | probe.overhead | js.performance_now | - | - | 0.0 (0.0–0.1) · p95 0.1 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | main-local | 91.3 (88.4–94.5) | 19.0 (17.5–19.1) | 15.6 (14.8–15.9) | 14.9 (14.2–15.4) |
| native | apply | apply | i1 | 88.3 (87.5–99.4) | 18.2 (17.6–19.2) | 15.2 (14.8–16.1) | 14.9 (14.3–15.5) |
| native | web.dispatch_view | web.dispatch_view | main-local | 122.1 (114.1–125.5) | 25.2 (24.6–26.1) | 22.6 (22.1–23.4) | 22.7 (21.8–23.5) |
| native | web.dispatch_view | web.dispatch_view | i1 | 124.8 (123.5–133.4) | 25.4 (24.2–27.4) | 22.6 (22.5–24.1) | 22.8 (21.9–24.5) |
| native | web.dispatch_outcome | web.dispatch_outcome | main-local | 171.2 (161.5–173.5) | 73.1 (66.9–73.8) | 72.0 (66.6–72.6) | 75.3 (70.1–78.0) |
| native | web.dispatch_outcome | web.dispatch_outcome | i1 | 168.5 (162.4–174.4) | 71.0 (66.9–71.3) | 70.0 (66.0–70.1) | 72.4 (69.2–73.5) |
| native | read | clone | main-local | 7.7 (7.6–7.8) | 6.9 (6.8–7.1) | 7.1 (6.8–7.2) | 8.1 (7.9–8.3) |
| native | read | clone | i1 | 7.7 (7.6–7.7) | 6.9 (6.6–7.4) | 7.0 (6.8–7.5) | 8.1 (7.8–8.5) |
| native | read | clone.drop | main-local | 4.2 (4.2–4.2) | 3.8 (3.7–3.9) | 3.9 (3.8–4.0) | 4.4 (4.3–4.5) |
| native | read | clone.drop | i1 | 4.2 (4.1–4.2) | 3.8 (3.7–4.1) | 3.9 (3.8–4.2) | 4.3 (4.2–4.6) |
| native | read | save | main-local | 6.5 (6.3–6.9) | 7.1 (7.0–7.3) | 9.4 (9.3–9.9) | 10.3 (10.2–10.7) |
| native | read | save | i1 | 6.4 (6.2–6.7) | 7.1 (6.8–7.7) | 9.5 (9.3–9.9) | 10.3 (10.1–10.9) |
| native | read | save_json | main-local | 10.8 (10.5–10.9) | 11.9 (11.8–12.3) | 15.4 (15.4–16.3) | 16.8 (16.3–17.3) |
| native | read | save_json | i1 | 10.6 (10.2–10.8) | 11.8 (11.4–12.8) | 15.6 (15.3–16.0) | 16.7 (16.2–17.5) |
| native | read | snapshot | main-local | 36.9 (32.9–38.6) | 24.4 (24.1–25.1) | 26.2 (25.6–27.2) | 28.5 (27.8–29.4) |
| native | read | snapshot | i1 | 33.1 (31.4–38.9) | 24.4 (23.4–26.2) | 26.0 (25.6–27.7) | 28.2 (27.3–29.5) |
| native | read | snapshot.drop | main-local | 11.6 (11.3–11.6) | 10.4 (10.4–10.8) | 11.2 (11.0–11.6) | 12.2 (11.8–12.6) |
| native | read | snapshot.drop | i1 | 11.4 (11.2–11.8) | 10.6 (10.2–11.3) | 11.3 (11.0–11.9) | 12.2 (11.8–12.7) |
| native | read | snapshot.serialize | main-local | 18.3 (18.2–20.2) | 15.4 (15.1–15.8) | 16.4 (16.0–17.0) | 17.1 (16.9–17.7) |
| native | read | snapshot.serialize | i1 | 19.7 (18.7–20.4) | 15.1 (14.4–16.2) | 16.1 (15.8–17.0) | 16.8 (16.3–17.7) |
| native | read | snapshot.serialize_pretty | main-local | 37.2 (37.2–42.2) | 32.6 (31.9–33.5) | 34.4 (33.7–35.6) | 36.3 (35.3–37.4) |
| native | read | snapshot.serialize_pretty | i1 | 37.0 (37.0–38.0) | 32.4 (31.0–34.7) | 34.0 (33.4–36.1) | 35.7 (34.4–37.4) |
| native | read | view | main-local | 2.4 (2.4–2.7) | 2.1 (2.1–2.1) | 2.5 (2.4–2.6) | 2.8 (2.7–2.9) |
| native | read | view | i1 | 2.4 (2.3–2.5) | 2.1 (2.1–2.3) | 2.5 (2.4–2.6) | 2.8 (2.7–2.9) |
| native | read | view.serialize | main-local | 6.7 (6.5–7.2) | 4.9 (4.9–5.0) | 5.1 (4.9–5.2) | 5.4 (5.3–5.6) |
| native | read | view.serialize | i1 | 6.8 (6.6–7.0) | 4.8 (4.6–5.1) | 4.9 (4.8–5.3) | 5.2 (5.0–5.4) |
| native | web.read | web.save | main-local | 13.3 (13.2–13.4) | 13.8 (13.3–14.7) | 17.0 (16.8–17.2) | 18.7 (18.2–19.2) |
| native | web.read | web.save | i1 | 12.6 (12.5–13.4) | 13.6 (12.7–13.6) | 17.3 (16.5–17.4) | 18.1 (17.0–18.3) |
| native | web.read | web.snapshot | main-local | 96.4 (89.5–104.4) | 69.5 (68.0–73.8) | 71.8 (71.4–77.1) | 78.3 (76.8–81.2) |
| native | web.read | web.snapshot | i1 | 81.9 (81.7–88.5) | 67.6 (63.5–67.8) | 71.0 (68.2–72.0) | 75.6 (71.2–76.5) |
| native | web.read | web.view | main-local | 7.5 (7.2–7.9) | 6.4 (6.2–6.7) | 6.8 (6.7–7.1) | 7.6 (7.4–7.7) |
| native | web.read | web.view | i1 | 7.1 (7.1–7.1) | 6.1 (5.8–6.1) | 6.7 (6.4–6.8) | 7.2 (6.9–7.4) |
| native | bench.dispatch_only | bench.dispatch_only | i1 | 95.0 (93.8–98.2) | 18.8 (17.7–20.1) | 15.6 (15.2–16.7) | 15.2 (14.6–15.9) |
| native | bench.read | bench.snapshot_build | i1 | 46.9 (45.2–48.6) | 33.9 (32.3–35.9) | 35.7 (34.5–37.7) | 39.0 (36.7–40.4) |
| native | bench.read | bench.view_build | i1 | 4.4 (4.2–4.5) | 3.4 (3.2–3.6) | 3.9 (3.8–4.1) | 4.3 (4.1–4.5) |
| native | bench.read | web.snapshot | i1 | 87.2 (77.9–90.0) | 65.6 (62.0–69.2) | 68.9 (66.1–72.7) | 74.0 (69.5–76.7) |
| native | bench.read | web.view | i1 | 11.6 (11.2–12.1) | 8.0 (7.6–8.4) | 8.6 (8.2–9.1) | 9.3 (8.8–9.8) |
| native | probe.dispatch_view | apply.clone | i2 | 0.6 (0.6–0.6) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) |
| native | probe.dispatch_view | apply.swap | i2 | 3.1 (3.1–3.4) | 1.3 (1.3–1.3) | 1.2 (1.1–1.2) | 1.2 (1.2–1.3) |
| native | probe.dispatch_view | apply.validate | i2 | 0.3 (0.3–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | bind.eval | i2 | 35.0 (34.1–35.6) | 0.9 (0.9–0.9) | 62.3 (60.8–63.4) | - |
| native | probe.dispatch_view | bind.explain | i2 | 6.4 (6.3–7.1) | 0.2 (0.2–0.2) | 7.4 (7.4–8.6) | - |
| native | probe.dispatch_view | bind.none | i2 | - | - | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | bind.sort | i2 | 1.1 (1.1–1.2) | 0.1 (0.1–0.1) | 1.0 (1.0–1.1) | - |
| native | probe.dispatch_view | bind.stale | i2 | 1.3 (1.2–1.4) | 1.3 (1.2–1.3) | 1.2 (1.2–1.3) | 1.3 (1.2–1.3) |
| native | probe.dispatch_view | bind.write | i2 | 6.1 (5.9–6.3) | 0.9 (0.9–0.9) | 7.1 (7.1–7.2) | - |
| native | probe.dispatch_view | exit | i2 | 0.1 (0.1–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.dispatch_view | payload.parse | i2 | 1.4 (1.4–1.4) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_view | payload.resolve | i2 | 0.7 (0.6–0.7) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_view | run.changes | i2 | 0.6 (0.6–0.6) | 0.3 (0.3–0.3) | 0.3 (0.3–0.4) | 0.4 (0.4–0.4) |
| native | probe.dispatch_view | run.clock | i2 | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | run.prologue | i2 | 0.4 (0.3–0.4) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_view | run.rules | i2 | 30.5 (29.6–30.6) | 13.8 (13.0–13.9) | 12.3 (12.0–12.8) | 11.9 (11.4–12.1) |
| native | probe.dispatch_view | total | i2 | 108.4 (105.1–109.0) | 26.2 (25.1–26.3) | 23.5 (22.7–24.2) | 23.6 (22.7–24.0) |
| native | probe.dispatch_view | view.build | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | view.commitments | i2 | 0.8 (0.8–0.8) | 0.4 (0.4–0.5) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) |
| native | probe.dispatch_view | view.enter | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.0–0.1) |
| native | probe.dispatch_view | view.names | i2 | 0.4 (0.4–0.5) | 0.3 (0.3–0.3) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) |
| native | probe.dispatch_view | view.relations | i2 | 1.2 (1.2–1.2) | 1.1 (1.1–1.1) | 1.3 (1.2–1.3) | 1.4 (1.3–1.4) |
| native | probe.dispatch_view | view.sort | i2 | 1.2 (1.2–1.3) | 0.4 (0.4–0.5) | 0.6 (0.5–0.6) | 0.6 (0.6–0.6) |
| native | probe.dispatch_view | web.drop | i2 | 0.6 (0.6–0.6) | 0.5 (0.5–0.5) | 0.6 (0.6–0.6) | 0.6 (0.6–0.6) |
| native | probe.dispatch_view | web.enter | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | web.serialize | i2 | 7.9 (7.7–8.2) | 3.5 (3.4–3.5) | 3.7 (3.6–3.8) | 3.9 (3.7–3.9) |
| native | probe.dispatch_outcome | apply.clone | i2 | 0.6 (0.6–0.6) | 0.4 (0.3–0.4) | 0.4 (0.3–0.4) | 0.4 (0.3–0.4) |
| native | probe.dispatch_outcome | apply.swap | i2 | 3.3 (3.0–3.4) | 1.5 (1.4–1.5) | 1.3 (1.2–1.3) | 1.3 (1.3–1.4) |
| native | probe.dispatch_outcome | apply.validate | i2 | 0.4 (0.3–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_outcome | bind.eval | i2 | 33.9 (33.7–35.8) | 1.1 (1.0–1.1) | 76.9 (74.5–84.1) | - |
| native | probe.dispatch_outcome | bind.explain | i2 | 6.2 (6.1–6.6) | 0.2 (0.2–0.3) | 6.9 (6.6–7.4) | - |
| native | probe.dispatch_outcome | bind.none | i2 | - | - | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_outcome | bind.sort | i2 | 1.2 (1.1–1.2) | 0.1 (0.1–0.1) | 1.3 (1.3–1.3) | - |
| native | probe.dispatch_outcome | bind.stale | i2 | 1.3 (1.2–1.3) | 1.3 (1.3–1.4) | 1.3 (1.2–1.3) | 1.3 (1.2–1.4) |
| native | probe.dispatch_outcome | bind.write | i2 | 6.2 (5.7–6.2) | 1.4 (1.4–1.5) | 7.2 (6.8–7.6) | - |
| native | probe.dispatch_outcome | exit | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_outcome | outcome.built | i2 | 0.3 (0.2–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_outcome | payload.parse | i2 | 1.7 (1.6–1.7) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| native | probe.dispatch_outcome | payload.resolve | i2 | 0.7 (0.7–0.7) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_outcome | run.changes | i2 | 0.6 (0.6–0.7) | 0.4 (0.4–0.5) | 0.4 (0.4–0.5) | 0.5 (0.5–0.5) |
| native | probe.dispatch_outcome | run.clock | i2 | 0.2 (0.2–0.3) | 0.1 (0.1–0.2) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) |
| native | probe.dispatch_outcome | run.prologue | i2 | 0.4 (0.4–0.5) | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_outcome | run.rules | i2 | 30.3 (30.1–31.7) | 14.7 (14.0–15.2) | 13.3 (12.8–14.0) | 12.9 (12.1–13.4) |
| native | probe.dispatch_outcome | snap.build | i2 | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_outcome | snap.enter | i2 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_outcome | snap.names | i2 | 0.5 (0.5–0.6) | 0.3 (0.3–0.4) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) |
| native | probe.dispatch_outcome | snap.records | i2 | 3.1 (3.1–3.2) | 3.1 (2.8–3.1) | 4.2 (4.0–4.2) | 4.4 (4.0–4.4) |
| native | probe.dispatch_outcome | snap.relations | i2 | 1.2 (1.2–1.3) | 1.3 (1.2–1.3) | 1.4 (1.4–1.5) | 1.6 (1.6–1.7) |
| native | probe.dispatch_outcome | snap.shown | i2 | 7.3 (6.9–7.3) | 6.7 (6.3–6.9) | 6.7 (6.2–6.8) | 7.6 (6.9–7.8) |
| native | probe.dispatch_outcome | snap.sort | i2 | 1.3 (1.1–1.3) | 0.8 (0.7–0.8) | 0.9 (0.9–0.9) | 1.0 (0.9–1.0) |
| native | probe.dispatch_outcome | snap.static | i2 | 2.6 (2.3–2.8) | 2.2 (2.1–2.3) | 2.2 (2.1–2.3) | 2.3 (2.2–2.5) |
| native | probe.dispatch_outcome | snap.symbols | i2 | 4.3 (4.0–4.9) | 3.5 (3.4–3.7) | 3.9 (3.7–4.0) | 4.1 (3.9–4.3) |
| native | probe.dispatch_outcome | snap.values | i2 | 8.6 (8.2–9.6) | 6.5 (6.1–6.6) | 6.3 (5.9–6.4) | 6.5 (6.1–6.7) |
| native | probe.dispatch_outcome | total | i2 | 156.9 (151.1–162.3) | 72.1 (68.9–74.4) | 70.6 (67.9–72.7) | 74.6 (70.0–77.2) |
| native | probe.dispatch_outcome | web.drop | i2 | 11.7 (11.5–11.8) | 10.3 (10.0–10.7) | 10.9 (10.7–11.3) | 11.9 (11.4–12.3) |
| native | probe.dispatch_outcome | web.enter | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_outcome | web.serialize | i2 | 20.0 (19.3–21.5) | 14.9 (14.2–15.4) | 15.7 (15.3–16.3) | 16.6 (15.7–17.4) |
| native | probe.read | save:exit | i2 | 0.0 (0.0–0.1) | 0.0 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | save:save.build | i2 | 6.4 (6.4–6.5) | 6.9 (6.6–7.0) | 9.3 (9.1–9.3) | 9.9 (9.6–9.9) |
| native | probe.read | save:save.drop | i2 | 2.2 (2.0–2.2) | 2.4 (2.3–2.4) | 3.2 (3.0–3.2) | 3.3 (3.1–3.4) |
| native | probe.read | save:save.enter | i2 | 0.0 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | save:save.serialize | i2 | 4.0 (3.9–4.2) | 3.9 (3.7–3.9) | 4.5 (4.3–4.5) | 4.6 (4.4–4.6) |
| native | probe.read | save:total | i2 | 12.6 (12.4–13.0) | 13.4 (12.7–13.5) | 17.2 (16.8–17.3) | 17.9 (17.3–18.1) |
| native | probe.read | save:web.enter | i2 | 0.1 (0.0–0.1) | 0.0 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | snapshot:exit | i2 | 0.0 (0.0–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.read | snapshot:snap.build | i2 | 0.2 (0.1–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.read | snapshot:snap.enter | i2 | 0.1 (0.1–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | snapshot:snap.names | i2 | 0.4 (0.3–0.4) | 0.3 (0.3–0.3) | 0.4 (0.3–0.4) | 0.4 (0.4–0.4) |
| native | probe.read | snapshot:snap.records | i2 | 3.0 (2.9–3.1) | 2.9 (2.7–3.0) | 4.1 (3.9–4.2) | 4.1 (3.8–4.2) |
| native | probe.read | snapshot:snap.relations | i2 | 1.1 (1.1–1.2) | 1.3 (1.2–1.3) | 1.4 (1.4–1.4) | 1.6 (1.5–1.6) |
| native | probe.read | snapshot:snap.shown | i2 | 7.2 (7.1–7.4) | 6.5 (6.1–6.6) | 6.5 (6.3–6.5) | 7.3 (7.2–7.3) |
| native | probe.read | snapshot:snap.sort | i2 | 0.5 (0.5–0.6) | 0.4 (0.4–0.4) | 0.5 (0.5–0.6) | 0.6 (0.6–0.6) |
| native | probe.read | snapshot:snap.static | i2 | 2.6 (2.4–2.7) | 2.2 (2.2–2.3) | 2.3 (2.2–2.3) | 2.4 (2.3–2.4) |
| native | probe.read | snapshot:snap.symbols | i2 | 3.9 (3.9–4.2) | 3.5 (3.4–3.5) | 3.9 (3.7–3.9) | 4.1 (3.9–4.1) |
| native | probe.read | snapshot:snap.values | i2 | 8.4 (8.4–8.7) | 6.5 (6.3–6.6) | 6.6 (6.3–6.6) | 6.8 (6.5–6.9) |
| native | probe.read | snapshot:total | i2 | 74.9 (73.4–77.5) | 66.9 (64.0–67.7) | 71.2 (69.0–71.4) | 75.0 (72.2–76.1) |
| native | probe.read | snapshot:web.drop | i2 | 11.2 (11.2–11.7) | 10.5 (9.9–10.5) | 11.1 (10.7–11.1) | 12.0 (11.5–12.1) |
| native | probe.read | snapshot:web.enter | i2 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | snapshot:web.serialize | i2 | 35.5 (34.5–37.3) | 32.3 (31.0–32.7) | 33.8 (32.8–34.2) | 35.2 (33.8–35.8) |
| native | probe.read | view:exit | i2 | 0.1 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | view:total | i2 | 7.2 (7.0–7.7) | 6.4 (6.1–6.5) | 7.0 (6.7–7.0) | 7.6 (7.2–7.7) |
| native | probe.read | view:view.build | i2 | 0.0 (0.0–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.read | view:view.commitments | i2 | 0.5 (0.4–0.5) | 0.4 (0.4–0.4) | 0.5 (0.4–0.5) | 0.5 (0.5–0.5) |
| native | probe.read | view:view.enter | i2 | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | view:view.names | i2 | 0.4 (0.4–0.4) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.4 (0.4–0.4) |
| native | probe.read | view:view.relations | i2 | 0.8 (0.8–1.0) | 1.1 (1.0–1.1) | 1.2 (1.2–1.3) | 1.4 (1.3–1.4) |
| native | probe.read | view:view.sort | i2 | 0.5 (0.5–0.6) | 0.5 (0.5–0.5) | 0.6 (0.6–0.6) | 0.7 (0.7–0.7) |
| native | probe.read | view:web.drop | i2 | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) | 0.6 (0.6–0.6) | 0.6 (0.6–0.7) |
| native | probe.read | view:web.enter | i2 | 0.1 (0.0–0.1) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) | 0.0 (0.0–0.0) |
| native | probe.read | view:web.serialize | i2 | 4.0 (3.9–4.2) | 3.4 (3.2–3.4) | 3.5 (3.3–3.5) | 3.8 (3.6–3.8) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local | 1.3 (1.0–1.5) | 0.4 (0.3–0.4) | 0.3 (0.3–0.3) | 0.3 (0.3–0.4) |
| wasm | raw.dispatch_view | js.stringify_payload | i1 | 1.1 (1.1–2.4) | 0.3 (0.3–0.4) | 0.3 (0.3–0.3) | 0.4 (0.3–0.4) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local | 141.6 (130.1–158.5) | 33.6 (32.4–33.8) | 28.4 (27.8–30.6) | 27.8 (27.5–29.5) |
| wasm | raw.dispatch_view | raw.dispatch_view | i1 | 151.2 (129.6–159.7) | 31.3 (31.2–32.2) | 29.3 (28.2–30.4) | 28.1 (28.0–28.5) |
| wasm | raw.dispatch_view | js.parse_view | main-local | 38.0 (32.8–47.1) | 18.3 (17.9–19.0) | 17.6 (17.3–17.8) | 18.2 (18.2–18.5) |
| wasm | raw.dispatch_view | js.parse_view | i1 | 34.5 (32.2–34.5) | 17.7 (17.6–18.4) | 18.4 (17.7–19.1) | 18.6 (18.2–19.0) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local | 172.3 (170.1–197.8) | 52.0 (51.2–53.3) | 46.8 (45.5–48.6) | 46.7 (46.2–48.2) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | i1 | 182.7 (172.0–190.8) | 49.4 (49.3–51.0) | 48.4 (46.4–50.1) | 47.2 (46.7–48.0) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | main-local | 220.1 (206.4–227.2) | 77.1 (73.8–80.2) | 76.9 (76.3–78.7) | 81.2 (78.8–85.1) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | i1 | 197.9 (183.5–203.3) | 80.5 (76.8–83.6) | 81.7 (76.9–83.9) | 79.2 (77.5–87.5) |
| wasm | raw.dispatch_outcome | js.parse_outcome | main-local | 86.6 (78.3–93.8) | 64.0 (62.1–66.2) | 67.1 (66.7–68.4) | 71.2 (69.7–74.7) |
| wasm | raw.dispatch_outcome | js.parse_outcome | i1 | 85.6 (83.0–109.6) | 67.7 (64.1–69.5) | 71.3 (67.6–73.3) | 70.3 (68.7–77.0) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | main-local | 296.4 (293.6–318.1) | 141.4 (136.2–147.1) | 145.1 (143.7–147.9) | 152.7 (149.1–160.8) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | i1 | 303.5 (297.4–318.9) | 148.5 (141.5–153.8) | 153.5 (145.3–157.6) | 150.2 (146.8–165.3) |
| wasm | abi.dispatch_view | abi.encode_args | main-local | 1.0 (0.9–1.2) | 0.3 (0.3–0.3) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | i1 | 0.8 (0.8–0.8) | 0.3 (0.2–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.exec | main-local | 129.2 (126.9–137.8) | 28.2 (27.0–28.7) | 24.6 (24.4–24.6) | 24.8 (24.6–25.7) |
| wasm | abi.dispatch_view | abi.exec | i1 | 127.4 (111.1–136.2) | 27.7 (27.4–28.1) | 24.3 (24.0–24.9) | 24.2 (22.7–25.0) |
| wasm | abi.dispatch_view | abi.decode | main-local | 4.1 (4.0–4.8) | 2.8 (2.8–2.9) | 2.7 (2.7–2.7) | 2.9 (2.7–2.9) |
| wasm | abi.dispatch_view | abi.decode | i1 | 4.3 (3.1–4.6) | 2.8 (2.7–2.8) | 2.7 (2.6–2.8) | 2.8 (2.6–2.9) |
| wasm | abi.dispatch_view | abi.free | main-local | 0.3 (0.3–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | i1 | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local | 134.8 (132.3–145.6) | 31.4 (30.3–31.9) | 27.5 (27.4–27.5) | 28.0 (27.7–29.0) |
| wasm | abi.dispatch_view | abi.dispatch_view | i1 | 132.4 (113.9–140.7) | 30.8 (30.6–31.3) | 27.3 (26.9–28.1) | 27.3 (25.5–28.2) |
| wasm | abi.dispatch_outcome | abi.encode_args | main-local | 1.0 (1.0–1.7) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | abi.dispatch_outcome | abi.encode_args | i1 | 1.0 (0.8–1.0) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | abi.dispatch_outcome | abi.exec | main-local | 213.0 (180.6–221.6) | 64.7 (64.6–68.9) | 64.6 (64.0–66.7) | 65.7 (65.1–67.4) |
| wasm | abi.dispatch_outcome | abi.exec | i1 | 181.0 (154.7–195.5) | 66.0 (64.3–70.7) | 63.1 (61.5–64.4) | 64.9 (64.0–65.9) |
| wasm | abi.dispatch_outcome | abi.decode | main-local | 9.3 (8.4–10.1) | 8.0 (7.7–8.3) | 8.4 (8.3–8.5) | 8.8 (8.7–9.0) |
| wasm | abi.dispatch_outcome | abi.decode | i1 | 9.3 (8.6–9.8) | 7.9 (7.7–8.1) | 8.0 (7.9–8.3) | 8.7 (8.6–8.7) |
| wasm | abi.dispatch_outcome | abi.free | main-local | 0.3 (0.3–0.5) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.free | i1 | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | main-local | 226.9 (189.8–234.6) | 73.5 (72.9–78.0) | 73.6 (73.2–75.8) | 75.3 (74.6–77.0) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | i1 | 191.5 (170.4–204.0) | 74.6 (72.8–79.6) | 72.0 (70.1–73.5) | 74.3 (73.4–75.4) |
| wasm | read | raw.view | main-local | 12.2 (10.7–12.6) | 11.6 (11.4–11.7) | 11.8 (11.7–12.4) | 12.6 (12.3–13.8) |
| wasm | read | raw.view | i1 | 12.3 (12.0–13.0) | 11.3 (11.2–12.0) | 12.1 (11.7–12.4) | 12.6 (12.2–12.7) |
| wasm | read | js.parse_view | main-local | 34.7 (29.3–38.7) | 20.5 (19.9–20.9) | 19.8 (19.8–21.0) | 20.8 (20.3–22.8) |
| wasm | read | js.parse_view | i1 | 34.5 (33.9–46.2) | 19.9 (19.7–21.4) | 20.5 (19.7–21.5) | 20.4 (20.3–20.7) |
| wasm | read | raw.snapshot | main-local | 113.9 (95.0–118.8) | 98.6 (92.6–99.2) | 97.2 (95.3–103.2) | 102.9 (101.8–114.9) |
| wasm | read | raw.snapshot | i1 | 121.8 (118.1–123.2) | 95.5 (94.4–100.8) | 99.5 (96.4–103.0) | 103.6 (101.3–103.7) |
| wasm | read | js.parse_snapshot | main-local | 83.2 (77.9–85.8) | 74.3 (71.8–76.3) | 75.8 (75.3–78.2) | 79.0 (78.3–86.8) |
| wasm | read | js.parse_snapshot | i1 | 91.1 (84.0–93.9) | 72.4 (71.3–77.5) | 76.5 (74.8–79.4) | 78.8 (77.2–79.1) |
| wasm | read | raw.save | main-local | 17.6 (16.1–18.9) | 16.7 (15.9–17.0) | 19.0 (18.9–19.5) | 21.2 (20.9–24.0) |
| wasm | read | raw.save | i1 | 18.4 (17.9–20.7) | 16.1 (15.9–16.8) | 19.4 (19.2–19.6) | 21.2 (20.6–21.2) |
| wasm | abi.read | abi.view.exec | main-local | 10.1 (9.9–10.6) | 8.4 (8.1–8.7) | 8.9 (8.7–9.0) | 9.6 (9.3–10.2) |
| wasm | abi.read | abi.view.exec | i1 | 10.1 (9.8–11.2) | 8.3 (8.3–8.9) | 9.2 (8.8–9.5) | 9.4 (9.2–9.7) |
| wasm | abi.read | abi.view.decode | main-local | 1.9 (1.9–2.3) | 2.5 (2.5–2.7) | 2.6 (2.5–2.7) | 2.6 (2.6–2.8) |
| wasm | abi.read | abi.view.decode | i1 | 2.4 (2.1–2.5) | 2.6 (2.5–2.7) | 2.7 (2.5–2.8) | 2.6 (2.6–2.8) |
| wasm | abi.read | abi.snapshot.exec | main-local | 90.7 (87.6–94.2) | 73.0 (70.4–74.6) | 76.5 (73.8–76.7) | 80.5 (77.7–85.5) |
| wasm | abi.read | abi.snapshot.exec | i1 | 91.7 (89.1–102.8) | 74.6 (73.4–77.7) | 79.6 (76.3–81.1) | 79.9 (78.1–83.2) |
| wasm | abi.read | abi.snapshot.decode | main-local | 10.4 (9.5–10.8) | 12.7 (12.5–13.0) | 13.7 (13.3–13.9) | 14.2 (14.2–15.2) |
| wasm | abi.read | abi.snapshot.decode | i1 | 10.5 (10.5–11.5) | 13.2 (12.6–13.6) | 14.4 (13.5–14.6) | 14.2 (14.2–14.9) |
| wasm | abi.read | abi.save.exec | main-local | 14.6 (14.0–15.4) | 13.2 (12.9–13.7) | 15.5 (15.5–16.2) | 17.2 (16.8–18.5) |
| wasm | abi.read | abi.save.exec | i1 | 16.8 (15.6–18.1) | 13.7 (13.1–14.0) | 16.1 (16.0–17.0) | 17.1 (16.8–17.7) |
| wasm | abi.read | abi.save.decode | main-local | 0.5 (0.4–0.5) | 0.5 (0.5–0.5) | 0.7 (0.7–0.7) | 0.7 (0.7–0.7) |
| wasm | abi.read | abi.save.decode | i1 | 0.5 (0.5–0.6) | 0.5 (0.5–0.6) | 0.7 (0.7–0.7) | 0.7 (0.7–0.7) |
| wasm | abi.dispatch_only | abi.encode_args | i1 | 1.0 (1.0–2.4) | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_only | abi.dispatch_only.exec | i1 | 117.0 (114.1–136.2) | 18.8 (17.3–20.3) | 15.9 (15.5–17.2) | 14.6 (14.4–14.6) |
| wasm | abi.dispatch_only | abi.dispatch_only | i1 | 118.0 (115.7–139.4) | 19.0 (17.6–20.5) | 16.1 (15.7–17.3) | 14.7 (14.5–14.7) |
| wasm | abi.bench_read | abi.view_build.exec | i1 | 4.7 (4.4–4.8) | 3.0 (2.9–3.2) | 3.4 (3.2–3.5) | 3.9 (3.6–4.0) |
| wasm | abi.bench_read | abi.snapshot_build.exec | i1 | 40.0 (37.7–41.5) | 28.2 (27.1–29.0) | 28.2 (26.2–28.3) | 32.7 (29.7–33.0) |
| wasm | abi.bench_read | abi.view.exec | i1 | 13.9 (13.5–14.9) | 9.0 (8.5–9.4) | 9.6 (8.9–9.8) | 10.2 (9.4–10.5) |
| wasm | abi.bench_read | abi.view.decode | i1 | 3.7 (3.4–4.3) | 2.6 (2.5–2.7) | 2.6 (2.5–2.7) | 2.7 (2.5–2.8) |
| wasm | abi.bench_read | abi.snapshot.exec | i1 | 87.4 (81.4–90.9) | 71.6 (67.3–73.6) | 74.0 (68.0–74.6) | 80.1 (73.5–81.2) |
| wasm | abi.bench_read | abi.snapshot.decode | i1 | 11.3 (10.1–12.1) | 12.5 (12.1–12.8) | 13.6 (13.0–13.7) | 14.2 (13.5–14.7) |
| wasm | probe.dispatch_view | total | i2 | 219.2 (180.4–220.5) | 53.0 (50.2–53.7) | 48.7 (48.0–49.1) | 49.6 (49.1–49.6) |
| wasm | probe.dispatch_view | web.enter | i2 | 6.7 (1.3–9.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | payload.parse | i2 | 3.3 (2.9–4.0) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | probe.dispatch_view | payload.resolve | i2 | 1.0 (1.0–1.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | apply.validate | i2 | 0.5 (0.4–0.5) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | apply.clone | i2 | 0.8 (0.7–0.8) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | run.clock | i2 | 0.4 (0.4–0.5) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | run.prologue | i2 | 0.5 (0.5–0.6) | 0.1 (0.1–0.2) | 0.2 (0.1–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | run.rules | i2 | 43.9 (39.1–53.6) | 15.1 (14.3–15.4) | 13.2 (13.1–14.0) | 12.4 (12.2–12.4) |
| wasm | probe.dispatch_view | run.changes | i2 | 0.9 (0.8–1.1) | 0.6 (0.5–0.6) | 0.6 (0.6–0.6) | 0.7 (0.6–0.7) |
| wasm | probe.dispatch_view | bind.stale | i2 | 1.8 (1.5–1.8) | 1.8 (1.7–1.8) | 1.7 (1.7–1.7) | 1.7 (1.6–1.7) |
| wasm | probe.dispatch_view | bind.none | i2 | - | - | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | bind.eval | i2 | 41.1 (37.5–41.3) | 1.0 (1.0–1.0) | 90.4 (83.9–90.4) | - |
| wasm | probe.dispatch_view | bind.sort | i2 | 1.5 (1.3–1.5) | 0.1 (0.1–0.2) | 1.7 (1.7–1.9) | - |
| wasm | probe.dispatch_view | bind.explain | i2 | 7.8 (7.0–8.5) | 0.3 (0.2–0.3) | 8.9 (8.8–9.1) | - |
| wasm | probe.dispatch_view | bind.write | i2 | 6.5 (5.8–6.7) | 0.8 (0.8–0.8) | 8.1 (8.1–8.2) | - |
| wasm | probe.dispatch_view | apply.swap | i2 | 4.5 (4.1–4.8) | 0.9 (0.9–1.0) | 0.8 (0.8–0.8) | 0.8 (0.8–0.9) |
| wasm | probe.dispatch_view | view.enter | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | view.names | i2 | 0.7 (0.6–0.7) | 0.5 (0.5–0.5) | 0.6 (0.5–0.6) | 0.6 (0.6–0.6) |
| wasm | probe.dispatch_view | view.sort | i2 | 1.6 (1.4–1.8) | 0.5 (0.5–0.5) | 0.6 (0.6–0.6) | 0.7 (0.7–0.7) |
| wasm | probe.dispatch_view | view.commitments | i2 | 1.1 (1.1–1.1) | 0.6 (0.5–0.6) | 0.6 (0.6–0.6) | 0.6 (0.6–0.7) |
| wasm | probe.dispatch_view | view.relations | i2 | 1.3 (1.3–1.3) | 0.9 (0.9–0.9) | 1.0 (1.0–1.0) | 1.1 (1.1–1.2) |
| wasm | probe.dispatch_view | view.build | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | web.serialize | i2 | 11.4 (10.1–12.2) | 6.0 (5.7–6.0) | 6.0 (5.9–6.1) | 6.2 (6.1–6.3) |
| wasm | probe.dispatch_view | web.drop | i2 | 1.0 (0.9–1.1) | 0.5 (0.4–0.5) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) |
| wasm | probe.dispatch_view | exit | i2 | 4.2 (4.1–4.4) | 3.0 (2.8–3.0) | 2.8 (2.8–2.9) | 3.0 (3.0–3.0) |
| wasm | probe.dispatch_view | js.parse | i2 | 38.1 (37.8–41.6) | 18.8 (17.8–18.9) | 18.5 (18.3–18.8) | 19.3 (19.0–19.3) |
| wasm | probe.dispatch_outcome | total | i2 | 337.2 (293.1–347.7) | 154.8 (149.7–163.7) | 155.2 (153.2–159.7) | 160.6 (156.7–163.1) |
| wasm | probe.dispatch_outcome | web.enter | i2 | 3.6 (3.1–3.6) | 0.4 (0.4–0.5) | 0.4 (0.3–0.4) | 0.3 (0.3–0.3) |
| wasm | probe.dispatch_outcome | payload.parse | i2 | 3.3 (3.0–3.5) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) | 0.3 (0.3–0.3) |
| wasm | probe.dispatch_outcome | payload.resolve | i2 | 1.0 (0.9–1.1) | 0.2 (0.2–0.3) | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_outcome | apply.validate | i2 | 0.5 (0.5–0.5) | 0.2 (0.2–0.2) | 0.2 (0.1–0.2) | 0.2 (0.1–0.2) |
| wasm | probe.dispatch_outcome | apply.clone | i2 | 0.6 (0.5–0.7) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_outcome | run.clock | i2 | 0.4 (0.4–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_outcome | run.prologue | i2 | 0.5 (0.4–0.5) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_outcome | run.rules | i2 | 42.8 (40.6–52.6) | 16.2 (15.7–16.7) | 14.4 (14.1–14.8) | 13.4 (13.0–13.7) |
| wasm | probe.dispatch_outcome | run.changes | i2 | 1.0 (0.9–1.0) | 0.6 (0.6–0.7) | 0.7 (0.7–0.7) | 0.7 (0.7–0.7) |
| wasm | probe.dispatch_outcome | bind.stale | i2 | 1.8 (1.6–1.8) | 1.9 (1.8–2.0) | 1.7 (1.7–1.8) | 1.8 (1.7–1.8) |
| wasm | probe.dispatch_outcome | bind.none | i2 | - | - | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_outcome | bind.eval | i2 | 39.1 (37.0–42.8) | 1.1 (1.1–1.2) | 90.3 (86.1–91.9) | - |
| wasm | probe.dispatch_outcome | bind.sort | i2 | 1.5 (1.4–1.6) | 0.2 (0.2–0.2) | 1.8 (1.7–2.3) | - |
| wasm | probe.dispatch_outcome | bind.explain | i2 | 7.1 (6.5–7.2) | 0.3 (0.3–0.3) | 8.7 (8.3–9.1) | - |
| wasm | probe.dispatch_outcome | bind.write | i2 | 6.4 (6.2–6.6) | 1.0 (0.9–1.0) | 7.8 (7.6–9.3) | - |
| wasm | probe.dispatch_outcome | apply.swap | i2 | 4.1 (3.9–4.8) | 1.2 (1.2–1.3) | 1.1 (1.0–1.1) | 1.2 (1.2–1.2) |
| wasm | probe.dispatch_outcome | snap.enter | i2 | 0.3 (0.3–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_outcome | snap.names | i2 | 0.7 (0.6–0.7) | 0.5 (0.5–0.6) | 0.6 (0.6–0.6) | 0.6 (0.6–0.7) |
| wasm | probe.dispatch_outcome | snap.sort | i2 | 1.7 (1.7–1.8) | 0.6 (0.6–0.7) | 0.8 (0.7–0.8) | 0.8 (0.8–0.9) |
| wasm | probe.dispatch_outcome | snap.symbols | i2 | 5.1 (4.9–5.4) | 2.9 (2.8–3.2) | 3.2 (3.1–3.5) | 3.3 (3.3–3.3) |
| wasm | probe.dispatch_outcome | snap.shown | i2 | 7.0 (6.3–7.2) | 5.4 (5.3–6.1) | 5.4 (5.3–5.8) | 5.8 (5.7–5.8) |
| wasm | probe.dispatch_outcome | snap.values | i2 | 9.5 (8.0–9.7) | 5.1 (5.0–5.6) | 5.1 (5.0–5.2) | 5.1 (5.0–5.2) |
| wasm | probe.dispatch_outcome | snap.records | i2 | 3.1 (2.7–3.2) | 2.2 (2.2–2.6) | 2.8 (2.6–2.9) | 2.8 (2.7–2.8) |
| wasm | probe.dispatch_outcome | snap.static | i2 | 2.2 (1.8–2.2) | 1.5 (1.5–1.8) | 1.4 (1.4–1.6) | 1.5 (1.4–1.5) |
| wasm | probe.dispatch_outcome | snap.relations | i2 | 1.1 (1.0–1.2) | 1.1 (1.1–1.3) | 1.1 (1.1–1.3) | 1.3 (1.3–1.3) |
| wasm | probe.dispatch_outcome | snap.build | i2 | 0.2 (0.1–0.3) | 0.1 (0.1–0.2) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_outcome | outcome.built | i2 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_outcome | web.serialize | i2 | 33.8 (28.7–40.7) | 21.8 (20.9–22.9) | 22.2 (21.8–23.2) | 23.3 (22.6–23.6) |
| wasm | probe.dispatch_outcome | web.drop | i2 | 18.7 (17.2–19.6) | 12.1 (11.6–14.5) | 12.9 (12.7–14.5) | 13.4 (13.0–13.5) |
| wasm | probe.dispatch_outcome | exit | i2 | 10.0 (9.3–10.1) | 8.4 (8.2–9.5) | 8.8 (8.6–9.6) | 9.4 (9.2–9.6) |
| wasm | probe.dispatch_outcome | js.parse | i2 | 97.3 (92.6–101.0) | 68.3 (65.9–70.1) | 69.4 (69.3–72.5) | 73.2 (71.6–74.7) |
| wasm | probe.read | view:total | i2 | 47.0 (46.8–55.9) | 32.8 (32.2–36.2) | 33.5 (32.3–33.6) | 35.6 (33.9–37.1) |
| wasm | probe.read | view:web.enter | i2 | 0.5 (0.4–0.5) | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.read | view:view.enter | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.read | view:view.names | i2 | 0.5 (0.4–0.6) | 0.4 (0.4–0.5) | 0.5 (0.5–0.5) | 0.6 (0.5–0.6) |
| wasm | probe.read | view:view.sort | i2 | 0.8 (0.6–0.8) | 0.5 (0.5–0.6) | 0.7 (0.6–0.7) | 0.8 (0.7–0.8) |
| wasm | probe.read | view:view.commitments | i2 | 0.7 (0.7–0.7) | 0.6 (0.6–0.7) | 0.7 (0.7–0.7) | 0.7 (0.7–0.8) |
| wasm | probe.read | view:view.relations | i2 | 1.0 (0.9–1.1) | 1.1 (1.1–1.2) | 1.2 (1.2–1.3) | 1.4 (1.4–1.5) |
| wasm | probe.read | view:view.build | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.read | view:web.serialize | i2 | 6.1 (6.1–6.5) | 5.7 (5.6–6.3) | 5.8 (5.6–5.8) | 6.1 (5.8–6.4) |
| wasm | probe.read | view:web.drop | i2 | 0.8 (0.7–0.8) | 0.7 (0.7–0.8) | 0.7 (0.7–0.7) | 0.7 (0.7–0.8) |
| wasm | probe.read | view:exit | i2 | 2.8 (2.5–2.9) | 2.7 (2.7–3.0) | 2.8 (2.7–2.8) | 3.0 (2.9–3.1) |
| wasm | probe.read | view:js.parse | i2 | 34.2 (33.1–40.7) | 20.5 (20.2–22.7) | 20.4 (19.8–20.7) | 21.6 (20.7–22.7) |
| wasm | probe.read | snapshot:total | i2 | 228.1 (192.2–261.0) | 174.3 (171.2–192.1) | 183.2 (176.9–184.1) | 194.9 (186.4–205.6) |
| wasm | probe.read | snapshot:web.enter | i2 | 0.6 (0.4–0.7) | 0.3 (0.2–0.3) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.read | snapshot:snap.enter | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.read | snapshot:snap.names | i2 | 0.7 (0.5–0.7) | 0.5 (0.5–0.6) | 0.6 (0.6–0.6) | 0.6 (0.6–0.7) |
| wasm | probe.read | snapshot:snap.sort | i2 | 1.3 (1.2–1.5) | 0.7 (0.6–0.7) | 0.9 (0.8–0.9) | 0.9 (0.9–1.1) |
| wasm | probe.read | snapshot:snap.symbols | i2 | 5.4 (4.6–5.7) | 3.8 (3.6–4.1) | 4.1 (3.9–4.1) | 4.6 (4.4–5.0) |
| wasm | probe.read | snapshot:snap.shown | i2 | 6.3 (5.9–6.6) | 5.9 (5.8–6.6) | 5.7 (5.7–6.0) | 6.6 (6.4–6.8) |
| wasm | probe.read | snapshot:snap.values | i2 | 9.9 (8.3–10.1) | 5.9 (5.9–6.7) | 6.0 (5.8–6.0) | 6.2 (5.9–6.6) |
| wasm | probe.read | snapshot:snap.records | i2 | 2.7 (2.3–3.2) | 2.3 (2.3–2.6) | 2.8 (2.7–2.9) | 2.9 (2.8–3.1) |
| wasm | probe.read | snapshot:snap.static | i2 | 2.1 (1.9–2.2) | 1.5 (1.5–1.7) | 1.5 (1.4–1.5) | 1.5 (1.5–1.6) |
| wasm | probe.read | snapshot:snap.relations | i2 | 0.9 (0.8–0.9) | 1.1 (1.0–1.2) | 1.2 (1.2–1.2) | 1.3 (1.3–1.4) |
| wasm | probe.read | snapshot:snap.build | i2 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.read | snapshot:web.serialize | i2 | 58.4 (52.9–62.2) | 47.9 (47.0–53.1) | 49.9 (47.8–50.1) | 52.9 (50.3–55.4) |
| wasm | probe.read | snapshot:web.drop | i2 | 18.5 (17.4–21.0) | 15.4 (14.9–16.9) | 16.6 (15.8–16.6) | 17.8 (17.2–19.2) |
| wasm | probe.read | snapshot:exit | i2 | 10.8 (10.5–11.8) | 13.4 (13.4–14.5) | 14.1 (13.7–14.2) | 15.0 (14.6–15.9) |
| wasm | probe.read | snapshot:js.parse | i2 | 98.9 (84.4–104.0) | 74.1 (73.3–81.7) | 77.9 (75.5–78.1) | 82.6 (79.3–87.1) |
| wasm | probe.read | save:total | i2 | 21.0 (18.9–22.8) | 17.9 (17.2–19.4) | 21.5 (21.2–21.8) | 23.2 (22.2–24.8) |
| wasm | probe.read | save:web.enter | i2 | 0.8 (0.8–0.8) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.read | save:save.enter | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.read | save:save.build | i2 | 8.2 (7.1–8.9) | 7.5 (7.1–8.1) | 9.2 (9.1–9.2) | 10.1 (9.7–10.8) |
| wasm | probe.read | save:save.serialize | i2 | 6.7 (6.3–7.5) | 5.6 (5.4–6.1) | 6.2 (6.1–6.3) | 6.6 (6.3–7.0) |
| wasm | probe.read | save:save.drop | i2 | 3.2 (3.1–3.4) | 3.5 (3.3–3.8) | 4.6 (4.6–4.8) | 5.1 (5.0–5.6) |
| wasm | probe.read | save:exit | i2 | 1.0 (1.0–1.1) | 0.9 (0.9–1.0) | 1.0 (0.9–1.0) | 1.0 (1.0–1.0) |

## glowcap-unbound

The Glowcap replay program with its 39 one-line bind statements removed, on the replay's full stream: the same rules, states and transaction with no bindings to evaluate, so apply here against apply on glowcap-replay isolates binding evaluation. Program `experiments/glowcap/caveat5/glowcap.cav` (`ffa078c4cb75`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | main-local | i1 | i2 |
| --- | --- | --- | --- | --- | --- |
| native | apply | apply | 15.4 (13.7–16.8) · p95 17.1 | 14.0 (13.8–14.1) · p95 16.2 | - |
| native | web.dispatch_view | web.dispatch_view | 20.0 (19.7–22.4) · p95 22.6 | 19.6 (19.3–19.9) · p95 22.4 | - |
| native | bench.dispatch_only | bench.dispatch_only | - | 13.8 (13.8–14.1) · p95 16.1 | - |
| native | probe.dispatch_view | apply.clone | - | - | 0.4 (0.4–0.4) · p95 0.4 |
| native | probe.dispatch_view | apply.swap | - | - | 1.3 (1.2–1.4) · p95 1.4 |
| native | probe.dispatch_view | apply.validate | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_view | bind.none | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.dispatch_view | bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | exit | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.dispatch_view | payload.parse | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_view | payload.resolve | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.dispatch_view | run.changes | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| native | probe.dispatch_view | run.clock | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | run.prologue | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.dispatch_view | run.rules | - | - | 12.3 (11.8–13.6) · p95 14.2 |
| native | probe.dispatch_view | total | - | - | 20.8 (20.0–23.3) · p95 23.8 |
| native | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.commitments | - | - | 0.5 (0.5–0.6) · p95 0.6 |
| native | probe.dispatch_view | view.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.names | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| native | probe.dispatch_view | view.relations | - | - | 1.5 (1.3–1.6) · p95 1.7 |
| native | probe.dispatch_view | view.sort | - | - | 0.6 (0.6–0.7) · p95 0.7 |
| native | probe.dispatch_view | web.drop | - | - | 0.6 (0.6–0.7) · p95 0.7 |
| native | probe.dispatch_view | web.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | web.serialize | - | - | 2.0 (1.9–2.2) · p95 2.3 |
| wasm | abi.dispatch_view | abi.encode_args | 0.1 (0.1–0.2) · p95 0.4 | 0.1 (0.1–0.2) · p95 0.4 | - |
| wasm | abi.dispatch_view | abi.exec | 21.1 (20.0–22.0) · p95 27.0 | 20.5 (19.9–22.1) · p95 25.5 | - |
| wasm | abi.dispatch_view | abi.decode | 0.5 (0.5–0.6) · p95 1.1 | 0.5 (0.5–0.6) · p95 1.1 | - |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | abi.dispatch_view | abi.dispatch_view | 22.0 (20.9–23.0) · p95 28.3 | 21.4 (20.8–23.1) · p95 26.9 | - |
| wasm | abi.dispatch_only | abi.encode_args | - | 0.1 (0.1–0.2) · p95 0.3 | - |
| wasm | abi.dispatch_only | abi.dispatch_only.exec | - | 14.8 (14.7–17.6) · p95 19.6 | - |
| wasm | abi.dispatch_only | abi.dispatch_only | - | 15.0 (14.8–17.8) · p95 19.8 | - |
| wasm | probe.dispatch_view | total | - | - | 31.4 (30.3–32.5) · p95 42.6 |
| wasm | probe.dispatch_view | web.enter | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.dispatch_view | payload.parse | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| wasm | probe.dispatch_view | payload.resolve | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_view | apply.validate | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| wasm | probe.dispatch_view | apply.clone | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| wasm | probe.dispatch_view | run.clock | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | run.prologue | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| wasm | probe.dispatch_view | run.rules | - | - | 12.7 (12.3–13.2) · p95 16.9 |
| wasm | probe.dispatch_view | run.changes | - | - | 0.7 (0.6–0.7) · p95 0.9 |
| wasm | probe.dispatch_view | bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | apply.swap | - | - | 1.0 (0.9–1.0) · p95 1.4 |
| wasm | probe.dispatch_view | view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | view.names | - | - | 0.6 (0.6–0.6) · p95 0.8 |
| wasm | probe.dispatch_view | view.sort | - | - | 0.8 (0.7–0.8) · p95 1.0 |
| wasm | probe.dispatch_view | view.commitments | - | - | 0.7 (0.7–0.8) · p95 0.9 |
| wasm | probe.dispatch_view | view.relations | - | - | 1.1 (1.1–1.2) · p95 1.8 |
| wasm | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | web.serialize | - | - | 3.3 (3.2–3.5) · p95 4.3 |
| wasm | probe.dispatch_view | web.drop | - | - | 0.6 (0.6–0.6) · p95 0.9 |
| wasm | probe.dispatch_view | exit | - | - | 0.7 (0.7–0.7) · p95 2.0 |
| wasm | probe.dispatch_view | js.parse | - | - | 7.5 (7.1–7.7) · p95 10.8 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | main-local | 36.5 (33.9–39.9) | 17.2 (15.6–18.8) | 15.9 (14.4–17.4) | 15.3 (13.6–16.7) |
| native | apply | apply | i1 | 37.4 (36.3–37.4) | 14.8 (14.4–15.4) | 15.0 (14.3–15.1) | 13.9 (13.6–14.0) |
| native | web.dispatch_view | web.dispatch_view | main-local | 53.2 (50.0–54.6) | 20.8 (20.0–23.1) | 19.8 (19.7–22.4) | 20.0 (19.6–22.3) |
| native | web.dispatch_view | web.dispatch_view | i1 | 48.6 (47.8–52.5) | 20.4 (20.2–20.5) | 20.0 (19.1–20.2) | 19.5 (19.2–19.8) |
| native | bench.dispatch_only | bench.dispatch_only | i1 | 41.2 (38.1–46.8) | 15.5 (14.7–16.0) | 14.2 (14.1–14.5) | 13.7 (13.7–14.0) |
| native | probe.dispatch_view | apply.clone | i2 | 0.6 (0.6–0.7) | 0.4 (0.3–0.4) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) |
| native | probe.dispatch_view | apply.swap | i2 | 2.5 (2.5–3.0) | 1.0 (0.9–1.1) | 1.2 (1.1–1.4) | 1.3 (1.2–1.4) |
| native | probe.dispatch_view | apply.validate | i2 | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | bind.none | i2 | 0.1 (0.0–0.1) | 0.0 (0.0–0.1) | 0.0 (0.0–0.1) | 0.0 (0.0–0.1) |
| native | probe.dispatch_view | bind.stale | i2 | 0.1 (0.0–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | exit | i2 | 0.0 (0.0–0.1) | 0.0 (0.0–0.1) | 0.0 (0.0–0.1) | 0.0 (0.0–0.1) |
| native | probe.dispatch_view | payload.parse | i2 | 1.4 (1.2–1.4) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_view | payload.resolve | i2 | 0.7 (0.6–0.7) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_view | run.changes | i2 | 0.6 (0.6–0.7) | 0.3 (0.3–0.4) | 0.4 (0.3–0.4) | 0.4 (0.4–0.4) |
| native | probe.dispatch_view | run.clock | i2 | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | run.prologue | i2 | 0.3 (0.3–0.4) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| native | probe.dispatch_view | run.rules | i2 | 30.2 (29.5–36.7) | 13.7 (13.1–15.9) | 13.0 (12.4–14.4) | 12.2 (11.7–13.5) |
| native | probe.dispatch_view | total | i2 | 44.0 (42.1–52.6) | 20.9 (20.1–24.5) | 20.9 (20.2–23.5) | 20.8 (20.0–23.2) |
| native | probe.dispatch_view | view.build | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | view.commitments | i2 | 0.8 (0.8–1.1) | 0.4 (0.4–0.5) | 0.5 (0.5–0.6) | 0.5 (0.5–0.6) |
| native | probe.dispatch_view | view.enter | i2 | 0.1 (0.1–0.1) | 0.0 (0.0–0.1) | 0.0 (0.0–0.1) | 0.1 (0.0–0.1) |
| native | probe.dispatch_view | view.names | i2 | 0.4 (0.3–0.4) | 0.3 (0.3–0.4) | 0.4 (0.3–0.4) | 0.4 (0.4–0.4) |
| native | probe.dispatch_view | view.relations | i2 | 1.2 (1.1–1.4) | 1.2 (1.1–1.3) | 1.3 (1.2–1.4) | 1.5 (1.4–1.6) |
| native | probe.dispatch_view | view.sort | i2 | 1.1 (1.0–1.1) | 0.4 (0.4–0.5) | 0.5 (0.5–0.6) | 0.6 (0.6–0.7) |
| native | probe.dispatch_view | web.drop | i2 | 0.6 (0.5–0.7) | 0.5 (0.5–0.6) | 0.6 (0.6–0.7) | 0.6 (0.6–0.7) |
| native | probe.dispatch_view | web.enter | i2 | 0.1 (0.0–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| native | probe.dispatch_view | web.serialize | i2 | 3.5 (3.5–3.9) | 1.7 (1.7–2.0) | 1.8 (1.8–2.1) | 2.0 (1.9–2.2) |
| wasm | abi.dispatch_view | abi.encode_args | main-local | 0.9 (0.9–1.1) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.encode_args | i1 | 1.0 (0.9–1.0) | 0.3 (0.3–0.3) | 0.1 (0.1–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.exec | main-local | 58.0 (56.3–61.1) | 23.8 (22.2–24.0) | 23.7 (22.4–24.0) | 20.8 (19.8–21.8) |
| wasm | abi.dispatch_view | abi.exec | i1 | 60.3 (55.7–77.4) | 21.3 (21.0–25.2) | 20.8 (20.7–23.4) | 20.4 (19.7–22.0) |
| wasm | abi.dispatch_view | abi.decode | main-local | 1.5 (1.3–1.5) | 0.6 (0.6–0.6) | 0.6 (0.6–0.6) | 0.5 (0.5–0.6) |
| wasm | abi.dispatch_view | abi.decode | i1 | 1.7 (1.5–2.3) | 0.6 (0.6–0.6) | 0.5 (0.5–0.6) | 0.5 (0.5–0.6) |
| wasm | abi.dispatch_view | abi.free | main-local | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.free | i1 | 0.3 (0.2–0.5) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local | 60.8 (58.1–64.3) | 24.9 (23.2–25.0) | 24.6 (23.2–24.9) | 21.8 (20.7–22.8) |
| wasm | abi.dispatch_view | abi.dispatch_view | i1 | 63.2 (58.6–82.8) | 22.3 (22.0–26.4) | 21.7 (21.5–24.3) | 21.3 (20.6–22.9) |
| wasm | abi.dispatch_only | abi.encode_args | i1 | 0.8 (0.7–0.9) | 0.2 (0.2–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_only | abi.dispatch_only.exec | i1 | 44.9 (42.5–68.7) | 15.5 (15.3–20.3) | 16.0 (15.3–21.2) | 14.8 (14.6–17.4) |
| wasm | abi.dispatch_only | abi.dispatch_only | i1 | 45.8 (42.8–69.3) | 15.6 (15.5–20.7) | 16.1 (15.4–21.4) | 14.9 (14.8–17.5) |
| wasm | probe.dispatch_view | total | i2 | 73.3 (71.0–120.6) | 31.4 (30.5–32.1) | 31.9 (31.8–32.5) | 31.4 (30.1–32.6) |
| wasm | probe.dispatch_view | web.enter | i2 | 1.1 (0.9–7.2) | 0.2 (0.2–0.3) | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | payload.parse | i2 | 2.7 (2.6–3.2) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | probe.dispatch_view | payload.resolve | i2 | 0.8 (0.8–0.9) | 0.2 (0.2–0.2) | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | apply.validate | i2 | 0.4 (0.4–0.5) | 0.2 (0.2–0.2) | 0.2 (0.1–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | apply.clone | i2 | 0.7 (0.7–0.8) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | run.clock | i2 | 0.4 (0.4–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | run.prologue | i2 | 0.5 (0.5–0.5) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | probe.dispatch_view | run.rules | i2 | 39.2 (38.1–46.4) | 15.2 (14.4–15.4) | 14.3 (13.9–14.5) | 12.6 (12.1–13.1) |
| wasm | probe.dispatch_view | run.changes | i2 | 0.8 (0.8–0.9) | 0.5 (0.5–0.6) | 0.6 (0.6–0.7) | 0.7 (0.6–0.7) |
| wasm | probe.dispatch_view | bind.stale | i2 | 0.1 (0.0–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | bind.none | i2 | 0.1 (0.0–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | apply.swap | i2 | 3.6 (3.5–4.4) | 0.7 (0.7–0.7) | 0.9 (0.9–1.0) | 1.0 (0.9–1.0) |
| wasm | probe.dispatch_view | view.enter | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | view.names | i2 | 0.6 (0.6–0.7) | 0.5 (0.5–0.5) | 0.6 (0.6–0.6) | 0.6 (0.6–0.6) |
| wasm | probe.dispatch_view | view.sort | i2 | 1.3 (1.3–1.4) | 0.5 (0.5–0.6) | 0.7 (0.7–0.7) | 0.8 (0.7–0.8) |
| wasm | probe.dispatch_view | view.commitments | i2 | 1.0 (0.9–1.1) | 0.6 (0.6–0.6) | 0.7 (0.7–0.7) | 0.7 (0.7–0.8) |
| wasm | probe.dispatch_view | view.relations | i2 | 1.1 (1.0–1.2) | 0.9 (0.9–0.9) | 1.1 (1.0–1.2) | 1.2 (1.1–1.2) |
| wasm | probe.dispatch_view | view.build | i2 | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | probe.dispatch_view | web.serialize | i2 | 5.5 (5.2–5.8) | 2.9 (2.9–3.1) | 3.2 (3.2–3.3) | 3.3 (3.2–3.5) |
| wasm | probe.dispatch_view | web.drop | i2 | 0.9 (0.8–0.9) | 0.5 (0.5–0.5) | 0.6 (0.6–0.7) | 0.6 (0.6–0.6) |
| wasm | probe.dispatch_view | exit | i2 | 1.3 (1.2–1.7) | 0.6 (0.6–0.6) | 0.7 (0.7–0.7) | 0.7 (0.7–0.7) |
| wasm | probe.dispatch_view | js.parse | i2 | 14.3 (14.0–14.3) | 6.8 (6.7–7.0) | 7.3 (7.2–7.4) | 7.5 (7.1–7.8) |

## glowcap-resume

The published resume measurement's stream (experiments/glowcap/resume-bench.mjs): absorb cave glowcap, then 9,600 ticks of dt 0.0625 (ten minutes of play), then save and restore. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `0407c19120d4`, 1 episode(s) × 1 = 9601 events per pass.

| Engine | Mode | Operation | main-local | i1 | i2 |
| --- | --- | --- | --- | --- | --- |
| native | lifecycle | from_source | 3156.1 (2821.3–3738.0) · p95 3479.1 | 3189.7 (2943.9–3365.6) · p95 3353.5 | - |
| native | lifecycle | restore | 3247.1 (2864.8–3780.6) · p95 3738.1 | 3253.7 (2973.1–3463.0) · p95 3432.3 | - |
| native | lifecycle | restore.parse | 34.0 (28.7–36.8) · p95 43.7 | 30.3 (28.9–33.3) · p95 37.4 | - |
| native | lifecycle | restore_json | 3272.4 (2924.3–3955.5) · p95 3859.8 | 3313.0 (3006.8–3563.2) · p95 3516.3 | - |
| native | lifecycle | save | 22.8 (19.2–26.1) · p95 33.9 | 21.1 (19.1–23.6) · p95 27.0 | - |
| native | lifecycle | save_json | 24.4 (21.5–28.1) · p95 43.5 | 21.8 (20.3–23.8) · p95 40.0 | - |
| native | lifecycle | web.new | 3131.5 (2806.5–3659.0) · p95 3480.3 | 3174.4 (2880.6–3344.4) · p95 3327.2 | - |
| native | lifecycle | web.restore | 3309.1 (2905.0–3862.4) · p95 3592.7 | 3317.8 (3092.3–3514.1) · p95 3510.4 | - |
| native | lifecycle | web.save | 18.2 (15.0–20.2) · p95 23.1 | 17.6 (15.9–19.1) · p95 20.4 | - |
| native | probe.lifecycle | new:bind.eval | - | - | 30.6 (28.3–30.7) · p95 33.0 |
| native | probe.lifecycle | new:bind.explain | - | - | 3.9 (3.7–4.0) · p95 5.3 |
| native | probe.lifecycle | new:bind.sort | - | - | 3.2 (3.1–3.2) · p95 3.6 |
| native | probe.lifecycle | new:bind.stale | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.lifecycle | new:bind.write | - | - | 11.9 (11.2–12.1) · p95 12.7 |
| native | probe.lifecycle | new:exit | - | - | 181.2 (168.6–188.0) · p95 221.5 |
| native | probe.lifecycle | new:load.bindings | - | - | 0.7 (0.6–0.7) · p95 0.8 |
| native | probe.lifecycle | new:load.declare | - | - | 276.3 (240.5–348.7) · p95 323.1 |
| native | probe.lifecycle | new:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.lifecycle | new:load.expand | - | - | 511.4 (479.3–514.1) · p95 557.4 |
| native | probe.lifecycle | new:load.map | - | - | 27.7 (24.6–31.6) · p95 35.2 |
| native | probe.lifecycle | new:load.parse | - | - | 1241.1 (1139.4–1263.1) · p95 1343.5 |
| native | probe.lifecycle | new:load.procedures | - | - | 254.4 (233.3–281.1) · p95 310.9 |
| native | probe.lifecycle | new:load.validate | - | - | 598.7 (590.3–619.7) · p95 662.7 |
| native | probe.lifecycle | new:total | - | - | 3216.7 (2951.0–3267.2) · p95 3381.5 |
| native | probe.lifecycle | restore:exit | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.lifecycle | restore:load.bindings | - | - | 56.3 (54.5–61.6) · p95 66.3 |
| native | probe.lifecycle | restore:load.declare | - | - | 337.6 (289.0–363.2) · p95 411.8 |
| native | probe.lifecycle | restore:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.lifecycle | restore:load.expand | - | - | 538.5 (522.5–557.2) · p95 619.8 |
| native | probe.lifecycle | restore:load.map | - | - | 36.2 (25.1–41.1) · p95 49.3 |
| native | probe.lifecycle | restore:load.parse | - | - | 1293.3 (1173.2–1388.9) · p95 1414.5 |
| native | probe.lifecycle | restore:load.procedures | - | - | 250.0 (237.0–256.7) · p95 283.2 |
| native | probe.lifecycle | restore:load.validate | - | - | 597.8 (549.6–602.1) · p95 642.2 |
| native | probe.lifecycle | restore:restore.bindings | - | - | 66.2 (64.4–68.8) · p95 72.2 |
| native | probe.lifecycle | restore:restore.done | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.lifecycle | restore:restore.drop_save | - | - | 4.5 (4.4–5.0) · p95 6.4 |
| native | probe.lifecycle | restore:restore.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.lifecycle | restore:restore.graph | - | - | 8.8 (8.6–9.7) · p95 10.2 |
| native | probe.lifecycle | restore:restore.header | - | - | 1.0 (0.9–1.0) · p95 1.2 |
| native | probe.lifecycle | restore:restore.loaded | - | - | 166.8 (159.7–171.4) · p95 203.3 |
| native | probe.lifecycle | restore:restore.parse | - | - | 24.0 (20.7–26.1) · p95 28.9 |
| native | probe.lifecycle | restore:restore.records | - | - | 21.4 (21.3–23.5) · p95 25.3 |
| native | probe.lifecycle | restore:restore.states | - | - | 5.3 (5.1–5.6) · p95 6.1 |
| native | probe.lifecycle | restore:total | - | - | 3452.2 (3217.2–3492.1) · p95 3717.5 |
| native | probe.lifecycle | restore:web.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| wasm | lifecycle | raw.new | 2872.6 (2825.4–3336.8) · p95 3512.5 | 3462.1 (2787.4–4551.3) · p95 4135.4 | - |
| wasm | lifecycle | raw.save | 41.7 (39.6–47.0) · p95 57.6 | 50.6 (41.5–64.7) · p95 62.1 | - |
| wasm | lifecycle | raw.restore | 2804.9 (2780.5–3294.5) · p95 3333.5 | 3289.3 (2810.0–4465.1) · p95 3947.4 | - |
| wasm | lifecycle | kit.open | 2843.0 (2765.0–3188.5) · p95 3159.9 | 3109.7 (2706.3–4383.4) · p95 4031.5 | - |
| wasm | lifecycle | kit.save | 102.1 (97.3–108.5) · p95 153.6 | 111.7 (95.0–151.5) · p95 159.3 | - |
| wasm | lifecycle | kit.restore | 2856.6 (2799.7–3266.9) · p95 3250.6 | 3158.6 (2770.2–4469.8) · p95 4268.7 | - |
| wasm | probe.lifecycle | restore:total | - | - | 3001.0 (2910.6–3427.3) · p95 3868.9 |
| wasm | probe.lifecycle | restore:web.enter | - | - | 17.7 (17.4–19.6) · p95 27.7 |
| wasm | probe.lifecycle | restore:restore.enter | - | - | 0.1 (0.1–0.1) · p95 0.3 |
| wasm | probe.lifecycle | restore:restore.parse | - | - | 33.2 (29.9–35.5) · p95 49.7 |
| wasm | probe.lifecycle | restore:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.lifecycle | restore:load.parse | - | - | 1222.9 (1193.0–1394.4) · p95 1602.9 |
| wasm | probe.lifecycle | restore:load.expand | - | - | 355.7 (331.4–386.9) · p95 412.4 |
| wasm | probe.lifecycle | restore:load.map | - | - | 34.6 (32.0–38.4) · p95 42.7 |
| wasm | probe.lifecycle | restore:load.declare | - | - | 151.5 (145.8–172.8) · p95 193.1 |
| wasm | probe.lifecycle | restore:load.procedures | - | - | 132.7 (130.2–155.8) · p95 178.0 |
| wasm | probe.lifecycle | restore:load.validate | - | - | 655.4 (624.9–743.9) · p95 812.3 |
| wasm | probe.lifecycle | restore:load.bindings | - | - | 59.1 (52.7–64.6) · p95 74.7 |
| wasm | probe.lifecycle | restore:restore.loaded | - | - | 216.5 (207.1–241.6) · p95 271.2 |
| wasm | probe.lifecycle | restore:restore.header | - | - | 1.3 (1.2–1.3) · p95 2.1 |
| wasm | probe.lifecycle | restore:restore.graph | - | - | 11.8 (10.6–11.9) · p95 15.2 |
| wasm | probe.lifecycle | restore:restore.states | - | - | 7.7 (6.6–8.3) · p95 9.6 |
| wasm | probe.lifecycle | restore:restore.records | - | - | 28.6 (27.0–30.7) · p95 40.9 |
| wasm | probe.lifecycle | restore:restore.bindings | - | - | 80.1 (76.6–89.0) · p95 97.5 |
| wasm | probe.lifecycle | restore:restore.done | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.lifecycle | restore:restore.drop_save | - | - | 5.6 (5.2–5.9) · p95 6.6 |
| wasm | probe.lifecycle | restore:exit | - | - | 5.7 (5.4–6.6) · p95 26.6 |
| wasm | probe.lifecycle | new:total | - | - | 2862.7 (2708.0–3167.2) · p95 3521.2 |
| wasm | probe.lifecycle | new:bind.stale | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.lifecycle | new:bind.eval | - | - | 35.1 (32.5–39.2) · p95 43.7 |
| wasm | probe.lifecycle | new:bind.sort | - | - | 4.1 (3.9–4.6) · p95 5.2 |
| wasm | probe.lifecycle | new:bind.explain | - | - | 4.4 (4.1–4.9) · p95 5.1 |
| wasm | probe.lifecycle | new:bind.write | - | - | 12.7 (11.5–13.7) · p95 15.1 |
| wasm | probe.lifecycle | new:load.enter | - | - | 15.9 (15.2–16.8) · p95 27.0 |
| wasm | probe.lifecycle | new:load.parse | - | - | 1244.1 (1167.0–1379.0) · p95 1537.1 |
| wasm | probe.lifecycle | new:load.expand | - | - | 344.0 (332.8–381.8) · p95 441.0 |
| wasm | probe.lifecycle | new:load.map | - | - | 34.6 (32.5–37.9) · p95 41.2 |
| wasm | probe.lifecycle | new:load.declare | - | - | 155.1 (147.5–177.0) · p95 177.7 |
| wasm | probe.lifecycle | new:load.procedures | - | - | 137.4 (131.1–150.8) · p95 176.4 |
| wasm | probe.lifecycle | new:load.validate | - | - | 642.6 (616.2–715.1) · p95 829.7 |
| wasm | probe.lifecycle | new:load.bindings | - | - | 0.5 (0.5–0.6) · p95 0.7 |
| wasm | probe.lifecycle | new:exit | - | - | 220.3 (210.1–247.9) · p95 259.5 |

## ledger-session

The agent ledger replaying the real 2026-09-24 session log: 20 events, five of them refused by policy. Played 100 times from a fresh session. Program `experiments/agent-ledger/ledger.cav` (`2206ac79f23c`), stream `ef2143311f86`, 1 episode(s) × 100 = 2000 events per pass.

| Engine | Mode | Operation | main-local | i1 | i2 |
| --- | --- | --- | --- | --- | --- |
| native | apply | apply | 16.9 (16.8–21.0) · p95 24.6 | 17.3 (16.7–17.5) · p95 25.1 | - |
| native | web.dispatch_view | web.dispatch_view | 26.1 (25.6–27.7) · p95 35.2 | 26.2 (25.6–27.8) · p95 35.6 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | 63.2 (61.9–64.7) · p95 80.8 | 63.5 (62.3–65.4) · p95 84.4 | - |
| native | read | clone | 1.9 (1.9–1.9) · p95 2.2 | 1.9 (1.8–1.9) · p95 2.2 | - |
| native | read | clone.drop | 1.1 (1.1–1.1) · p95 1.3 | 1.1 (1.0–1.1) · p95 1.2 | - |
| native | read | save | 7.2 (7.1–7.3) · p95 9.8 | 7.1 (7.1–7.4) · p95 9.8 | - |
| native | read | save_json | 12.9 (12.8–13.0) · p95 17.9 | 12.6 (12.6–13.1) · p95 17.9 | - |
| native | read | snapshot | 19.5 (19.4–19.8) · p95 23.5 | 19.4 (19.2–20.4) · p95 22.3 | - |
| native | read | snapshot.drop | 7.8 (7.7–7.9) · p95 8.9 | 7.7 (7.6–8.1) · p95 8.7 | - |
| native | read | snapshot.serialize | 14.8 (14.7–15.2) · p95 18.0 | 14.8 (14.6–15.4) · p95 17.9 | - |
| native | read | snapshot.serialize_pretty | 29.6 (29.3–30.0) · p95 35.7 | 29.7 (29.3–30.9) · p95 36.2 | - |
| native | read | view | 3.5 (3.5–3.5) · p95 5.0 | 3.5 (3.4–3.6) · p95 5.0 | - |
| native | read | view.serialize | 2.6 (2.5–2.6) · p95 3.2 | 2.6 (2.5–2.6) · p95 3.2 | - |
| native | web.read | web.save | 14.0 (13.9–14.4) · p95 18.7 | 13.9 (13.9–14.0) · p95 18.8 | - |
| native | web.read | web.snapshot | 57.2 (56.4–58.6) · p95 69.1 | 56.2 (56.2–56.5) · p95 67.1 | - |
| native | web.read | web.view | 5.7 (5.6–5.9) · p95 8.2 | 5.7 (5.6–5.7) · p95 8.1 | - |
| native | lifecycle | from_source | 1519.2 (1443.5–1590.0) · p95 1691.7 | 1550.9 (1479.7–1570.9) · p95 1697.8 | - |
| native | lifecycle | restore | 1565.4 (1485.1–1631.1) · p95 1655.4 | 1571.1 (1480.6–1645.4) · p95 1651.3 | - |
| native | lifecycle | restore.parse | 29.5 (28.4–36.3) · p95 33.0 | 29.9 (28.8–37.3) · p95 38.2 | - |
| native | lifecycle | restore_json | 1568.9 (1518.3–1639.8) · p95 1664.9 | 1599.5 (1536.8–1667.5) · p95 1725.6 | - |
| native | lifecycle | save | 18.6 (17.9–22.4) · p95 26.6 | 19.1 (17.5–23.0) · p95 23.8 | - |
| native | lifecycle | save_json | 24.8 (24.0–27.5) · p95 40.6 | 26.7 (24.2–27.9) · p95 34.2 | - |
| native | lifecycle | web.new | 1541.5 (1460.1–1589.1) · p95 1692.8 | 1514.8 (1469.0–1598.8) · p95 1606.3 | - |
| native | lifecycle | web.restore | 1592.8 (1513.6–1659.2) · p95 1686.7 | 1593.0 (1564.1–1657.2) · p95 1694.3 | - |
| native | lifecycle | web.save | 19.2 (18.5–22.1) · p95 23.0 | 20.0 (18.3–22.0) · p95 22.0 | - |
| native | bench.dispatch_only | bench.dispatch_only | - | 18.2 (18.2–19.6) · p95 26.0 | - |
| native | bench.read | bench.snapshot_build | - | 25.9 (25.8–27.1) · p95 30.6 | - |
| native | bench.read | bench.view_build | - | 4.8 (4.8–5.0) · p95 6.2 | - |
| native | bench.read | web.snapshot | - | 54.2 (54.1–56.7) · p95 64.8 | - |
| native | bench.read | web.view | - | 6.6 (6.5–6.9) · p95 8.3 | - |
| native | probe.dispatch_view | apply.clone | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.dispatch_view | apply.rollback | - | - | 3.8 (3.7–3.8) · p95 5.5 |
| native | probe.dispatch_view | apply.swap | - | - | 2.8 (2.7–2.8) · p95 3.8 |
| native | probe.dispatch_view | apply.validate | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| native | probe.dispatch_view | bind.eval | - | - | 3.1 (3.0–3.1) · p95 3.4 |
| native | probe.dispatch_view | bind.explain | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.dispatch_view | bind.sort | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_view | bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | bind.write | - | - | 1.2 (1.2–1.2) · p95 1.7 |
| native | probe.dispatch_view | exit | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_view | payload.parse | - | - | 0.6 (0.6–0.6) · p95 1.4 |
| native | probe.dispatch_view | payload.resolve | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| native | probe.dispatch_view | run.changes | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| native | probe.dispatch_view | run.clock | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_view | run.prologue | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| native | probe.dispatch_view | run.rules | - | - | 12.9 (12.8–12.9) · p95 15.9 |
| native | probe.dispatch_view | total | - | - | 27.1 (27.1–27.6) · p95 36.0 |
| native | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.commitments | - | - | 0.5 (0.5–0.6) · p95 0.7 |
| native | probe.dispatch_view | view.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.names | - | - | 0.5 (0.5–0.5) · p95 0.7 |
| native | probe.dispatch_view | view.relations | - | - | 1.0 (1.0–1.0) · p95 2.0 |
| native | probe.dispatch_view | view.sort | - | - | 2.1 (2.1–2.1) · p95 3.1 |
| native | probe.dispatch_view | web.drop | - | - | 0.4 (0.4–0.4) · p95 0.7 |
| native | probe.dispatch_view | web.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | web.serialize | - | - | 2.2 (2.2–2.3) · p95 3.0 |
| native | probe.dispatch_outcome | apply.clone | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.dispatch_outcome | apply.rollback | - | - | 4.1 (4.1–4.2) · p95 5.8 |
| native | probe.dispatch_outcome | apply.swap | - | - | 3.0 (3.0–3.1) · p95 4.0 |
| native | probe.dispatch_outcome | apply.validate | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| native | probe.dispatch_outcome | bind.eval | - | - | 3.2 (3.1–3.2) · p95 3.6 |
| native | probe.dispatch_outcome | bind.explain | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.dispatch_outcome | bind.sort | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | bind.write | - | - | 1.4 (1.4–1.5) · p95 1.8 |
| native | probe.dispatch_outcome | exit | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | outcome.built | - | - | 0.1 (0.1–0.1) · p95 0.3 |
| native | probe.dispatch_outcome | payload.parse | - | - | 0.9 (0.9–0.9) · p95 1.6 |
| native | probe.dispatch_outcome | payload.resolve | - | - | 0.4 (0.4–0.4) · p95 0.7 |
| native | probe.dispatch_outcome | run.changes | - | - | 0.2 (0.2–0.2) · p95 0.5 |
| native | probe.dispatch_outcome | run.clock | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.dispatch_outcome | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.dispatch_outcome | run.rules | - | - | 13.6 (13.5–13.9) · p95 16.6 |
| native | probe.dispatch_outcome | snap.build | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_outcome | snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | snap.names | - | - | 0.6 (0.6–0.6) · p95 0.8 |
| native | probe.dispatch_outcome | snap.records | - | - | 2.8 (2.6–2.9) · p95 3.8 |
| native | probe.dispatch_outcome | snap.relations | - | - | 1.3 (1.3–1.4) · p95 2.1 |
| native | probe.dispatch_outcome | snap.shown | - | - | 1.3 (1.3–1.3) · p95 1.8 |
| native | probe.dispatch_outcome | snap.sort | - | - | 2.4 (2.3–2.4) · p95 3.2 |
| native | probe.dispatch_outcome | snap.static | - | - | 3.0 (3.0–3.1) · p95 3.6 |
| native | probe.dispatch_outcome | snap.symbols | - | - | 4.7 (4.7–4.9) · p95 8.6 |
| native | probe.dispatch_outcome | snap.values | - | - | 3.7 (3.4–3.8) · p95 4.9 |
| native | probe.dispatch_outcome | total | - | - | 63.9 (63.0–65.2) · p95 82.7 |
| native | probe.dispatch_outcome | web.drop | - | - | 7.2 (7.0–7.4) · p95 8.4 |
| native | probe.dispatch_outcome | web.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | web.serialize | - | - | 13.8 (13.5–14.1) · p95 20.5 |
| native | probe.read | save:exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | save:save.build | - | - | 6.9 (6.8–7.0) · p95 9.6 |
| native | probe.read | save:save.drop | - | - | 2.2 (2.2–2.2) · p95 3.0 |
| native | probe.read | save:save.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | save:save.serialize | - | - | 4.5 (4.5–4.6) · p95 5.9 |
| native | probe.read | save:total | - | - | 13.8 (13.7–14.1) · p95 18.6 |
| native | probe.read | save:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:exit | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.read | snapshot:snap.build | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.read | snapshot:snap.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:snap.names | - | - | 0.5 (0.5–0.5) · p95 0.7 |
| native | probe.read | snapshot:snap.records | - | - | 3.0 (2.9–3.0) · p95 3.8 |
| native | probe.read | snapshot:snap.relations | - | - | 1.3 (1.3–1.3) · p95 1.9 |
| native | probe.read | snapshot:snap.shown | - | - | 1.4 (1.4–1.4) · p95 1.7 |
| native | probe.read | snapshot:snap.sort | - | - | 1.3 (1.3–1.4) · p95 1.5 |
| native | probe.read | snapshot:snap.static | - | - | 3.1 (3.0–3.1) · p95 3.6 |
| native | probe.read | snapshot:snap.symbols | - | - | 4.4 (4.4–4.5) · p95 5.5 |
| native | probe.read | snapshot:snap.values | - | - | 3.6 (3.6–3.6) · p95 4.5 |
| native | probe.read | snapshot:total | - | - | 56.5 (56.4–57.8) · p95 69.6 |
| native | probe.read | snapshot:web.drop | - | - | 7.6 (7.6–7.8) · p95 8.7 |
| native | probe.read | snapshot:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:web.serialize | - | - | 29.5 (29.4–30.3) · p95 37.3 |
| native | probe.read | view:exit | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:total | - | - | 5.9 (5.9–6.1) · p95 8.3 |
| native | probe.read | view:view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.read | view:view.commitments | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.read | view:view.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:view.names | - | - | 0.5 (0.5–0.5) · p95 0.6 |
| native | probe.read | view:view.relations | - | - | 1.1 (1.1–1.2) · p95 1.7 |
| native | probe.read | view:view.sort | - | - | 1.6 (1.6–1.6) · p95 2.0 |
| native | probe.read | view:web.drop | - | - | 0.4 (0.4–0.4) · p95 0.7 |
| native | probe.read | view:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | view:web.serialize | - | - | 1.8 (1.8–1.8) · p95 2.6 |
| native | probe.lifecycle | new:bind.eval | - | - | 3.7 (3.7–3.8) · p95 4.1 |
| native | probe.lifecycle | new:bind.explain | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| native | probe.lifecycle | new:bind.sort | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.lifecycle | new:bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.lifecycle | new:bind.write | - | - | 2.5 (2.3–2.7) · p95 3.4 |
| native | probe.lifecycle | new:exit | - | - | 100.8 (100.0–106.5) · p95 124.0 |
| native | probe.lifecycle | new:load.bindings | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.lifecycle | new:load.declare | - | - | 166.8 (157.6–173.2) · p95 204.8 |
| native | probe.lifecycle | new:load.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.lifecycle | new:load.expand | - | - | 277.3 (272.6–285.6) · p95 311.6 |
| native | probe.lifecycle | new:load.map | - | - | 34.1 (33.2–40.0) · p95 40.2 |
| native | probe.lifecycle | new:load.parse | - | - | 730.3 (690.4–750.8) · p95 781.9 |
| native | probe.lifecycle | new:load.procedures | - | - | 1.0 (1.0–1.0) · p95 1.2 |
| native | probe.lifecycle | new:load.validate | - | - | 245.9 (244.4–253.9) · p95 280.7 |
| native | probe.lifecycle | new:total | - | - | 1580.7 (1541.8–1612.3) · p95 1693.9 |
| native | probe.lifecycle | restore:exit | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.lifecycle | restore:load.bindings | - | - | 8.9 (7.2–11.1) · p95 14.2 |
| native | probe.lifecycle | restore:load.declare | - | - | 213.0 (186.2–219.8) · p95 275.8 |
| native | probe.lifecycle | restore:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.lifecycle | restore:load.expand | - | - | 283.5 (268.2–289.9) · p95 317.1 |
| native | probe.lifecycle | restore:load.map | - | - | 35.8 (34.8–40.0) · p95 46.0 |
| native | probe.lifecycle | restore:load.parse | - | - | 820.3 (747.6–825.1) · p95 869.4 |
| native | probe.lifecycle | restore:load.procedures | - | - | 1.0 (1.0–1.0) · p95 1.3 |
| native | probe.lifecycle | restore:load.validate | - | - | 261.5 (260.0–269.9) · p95 284.4 |
| native | probe.lifecycle | restore:restore.bindings | - | - | 6.5 (6.2–6.6) · p95 6.9 |
| native | probe.lifecycle | restore:restore.done | - | - | 0.3 (0.3–0.3) · p95 0.3 |
| native | probe.lifecycle | restore:restore.drop_save | - | - | 5.9 (5.9–6.8) · p95 7.6 |
| native | probe.lifecycle | restore:restore.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.lifecycle | restore:restore.graph | - | - | 11.3 (10.8–11.5) · p95 12.5 |
| native | probe.lifecycle | restore:restore.header | - | - | 0.8 (0.7–0.8) · p95 1.0 |
| native | probe.lifecycle | restore:restore.loaded | - | - | 90.5 (89.1–91.0) · p95 110.3 |
| native | probe.lifecycle | restore:restore.parse | - | - | 26.3 (24.6–26.4) · p95 28.8 |
| native | probe.lifecycle | restore:restore.records | - | - | 18.9 (18.1–19.5) · p95 21.5 |
| native | probe.lifecycle | restore:restore.states | - | - | 4.4 (4.1–4.4) · p95 4.7 |
| native | probe.lifecycle | restore:total | - | - | 1782.2 (1675.3–1821.7) · p95 1923.7 |
| native | probe.lifecycle | restore:web.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.overhead | probe.mark | - | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | probe.overhead | probe.now | - | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.4) · p95 1.1 | 0.3 (0.3–0.4) · p95 1.1 | - |
| wasm | raw.dispatch_view | raw.dispatch_view | 31.8 (31.2–32.4) · p95 43.4 | 31.8 (31.2–33.2) · p95 42.7 | - |
| wasm | raw.dispatch_view | js.parse_view | 7.8 (7.5–8.0) · p95 11.2 | 7.8 (7.6–8.2) · p95 11.4 | - |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 42.8 (41.8–43.5) · p95 55.6 | 42.8 (42.3–44.9) · p95 54.7 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 72.3 (70.9–74.6) · p95 96.8 | 70.0 (68.5–74.7) · p95 93.1 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 50.7 (50.1–52.9) · p95 65.9 | 49.9 (48.9–53.9) · p95 63.6 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 123.4 (121.6–127.9) · p95 162.8 | 120.6 (118.0–128.9) · p95 156.3 | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.3 (0.3–0.3) · p95 0.7 | 0.3 (0.3–0.3) · p95 0.7 | - |
| wasm | abi.dispatch_view | abi.exec | 30.6 (30.0–30.7) · p95 42.6 | 30.7 (29.6–31.1) · p95 43.1 | - |
| wasm | abi.dispatch_view | abi.decode | 0.7 (0.6–0.7) · p95 2.1 | 0.7 (0.7–0.7) · p95 2.1 | - |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.3 | - |
| wasm | abi.dispatch_view | abi.dispatch_view | 34.9 (34.1–35.2) · p95 45.5 | 35.1 (34.1–35.2) · p95 46.3 | - |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.3 (0.3–0.3) · p95 0.8 | 0.3 (0.3–0.3) · p95 0.8 | - |
| wasm | abi.dispatch_outcome | abi.exec | 66.1 (64.7–68.7) · p95 86.3 | 65.5 (64.6–65.9) · p95 83.9 | - |
| wasm | abi.dispatch_outcome | abi.decode | 2.6 (2.6–2.6) · p95 8.1 | 2.6 (2.5–2.6) · p95 7.6 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.4 (0.4–0.4) · p95 1.1 | 0.4 (0.4–0.5) · p95 1.0 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 70.3 (69.0–73.1) · p95 93.2 | 69.7 (68.8–70.0) · p95 89.6 | - |
| wasm | read | raw.view | 8.1 (8.0–8.6) · p95 11.8 | 8.1 (8.1–8.3) · p95 11.6 | - |
| wasm | read | js.parse_view | 9.1 (8.8–9.3) · p95 12.3 | 8.9 (8.8–9.2) · p95 12.0 | - |
| wasm | read | raw.snapshot | 77.4 (76.5–80.2) · p95 98.8 | 77.8 (77.5–80.6) · p95 97.4 | - |
| wasm | read | js.parse_snapshot | 58.9 (57.3–60.7) · p95 73.2 | 58.5 (58.2–60.6) · p95 70.7 | - |
| wasm | read | raw.save | 18.7 (18.3–19.4) · p95 26.6 | 18.8 (18.7–19.1) · p95 26.6 | - |
| wasm | abi.read | abi.view.exec | 7.8 (7.4–7.9) · p95 10.9 | 7.4 (7.2–7.5) · p95 10.1 | - |
| wasm | abi.read | abi.view.decode | 0.4 (0.4–0.4) · p95 0.7 | 0.4 (0.4–0.4) · p95 0.6 | - |
| wasm | abi.read | abi.snapshot.exec | 71.6 (66.8–73.9) · p95 88.0 | 67.8 (66.2–69.3) · p95 81.8 | - |
| wasm | abi.read | abi.snapshot.decode | 4.3 (3.6–4.4) · p95 7.7 | 4.2 (3.5–4.2) · p95 7.6 | - |
| wasm | abi.read | abi.save.exec | 16.9 (16.2–17.4) · p95 24.3 | 16.4 (16.1–16.7) · p95 22.5 | - |
| wasm | abi.read | abi.save.decode | 0.7 (0.7–0.7) · p95 1.2 | 0.7 (0.6–0.7) · p95 1.1 | - |
| wasm | lifecycle | raw.new | 1644.8 (1586.7–1777.6) · p95 1881.0 | 1799.7 (1795.4–1826.0) · p95 2108.6 | - |
| wasm | lifecycle | raw.save | 54.1 (53.3–55.1) · p95 75.4 | 58.7 (56.7–59.3) · p95 82.2 | - |
| wasm | lifecycle | raw.restore | 1780.3 (1736.2–1850.6) · p95 2105.7 | 1964.1 (1936.9–1969.6) · p95 2380.1 | - |
| wasm | lifecycle | kit.open | 1785.5 (1625.2–1840.1) · p95 2148.1 | 1860.6 (1850.0–1873.7) · p95 2129.3 | - |
| wasm | lifecycle | kit.save | 101.8 (97.8–103.5) · p95 143.8 | 111.5 (99.1–123.1) · p95 163.7 | - |
| wasm | lifecycle | kit.restore | 1729.8 (1700.2–1868.9) · p95 2124.3 | 1903.0 (1855.7–1903.7) · p95 2259.8 | - |
| wasm | abi.dispatch_only | abi.encode_args | - | 0.2 (0.2–0.3) · p95 0.5 | - |
| wasm | abi.dispatch_only | abi.dispatch_only.exec | - | 20.5 (20.2–22.2) · p95 30.0 | - |
| wasm | abi.dispatch_only | abi.dispatch_only | - | 20.9 (20.6–22.5) · p95 30.4 | - |
| wasm | abi.bench_read | abi.view_build.exec | - | 4.9 (4.8–5.2) · p95 6.5 | - |
| wasm | abi.bench_read | abi.snapshot_build.exec | - | 24.9 (24.2–27.0) · p95 31.0 | - |
| wasm | abi.bench_read | abi.view.exec | - | 8.0 (7.8–8.6) · p95 10.5 | - |
| wasm | abi.bench_read | abi.view.decode | - | 0.5 (0.5–0.6) · p95 1.5 | - |
| wasm | abi.bench_read | abi.snapshot.exec | - | 63.3 (62.4–69.4) · p95 77.4 | - |
| wasm | abi.bench_read | abi.snapshot.decode | - | 4.2 (3.6–4.3) · p95 7.9 | - |
| wasm | probe.dispatch_view | total | - | - | 42.0 (40.7–42.9) · p95 57.3 |
| wasm | probe.dispatch_view | web.enter | - | - | 0.4 (0.4–0.4) · p95 0.7 |
| wasm | probe.dispatch_view | payload.parse | - | - | 1.0 (0.9–1.0) · p95 2.3 |
| wasm | probe.dispatch_view | payload.resolve | - | - | 0.5 (0.5–0.5) · p95 0.8 |
| wasm | probe.dispatch_view | apply.validate | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| wasm | probe.dispatch_view | apply.clone | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.dispatch_view | run.clock | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_view | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.dispatch_view | run.rules | - | - | 13.6 (13.3–13.7) · p95 18.0 |
| wasm | probe.dispatch_view | run.changes | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.dispatch_view | bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | bind.eval | - | - | 3.3 (3.3–3.7) · p95 4.0 |
| wasm | probe.dispatch_view | bind.sort | - | - | 0.1 (0.1–0.2) · p95 0.2 |
| wasm | probe.dispatch_view | bind.explain | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| wasm | probe.dispatch_view | bind.write | - | - | 1.1 (1.1–1.2) · p95 1.6 |
| wasm | probe.dispatch_view | apply.swap | - | - | 4.0 (4.0–4.1) · p95 6.0 |
| wasm | probe.dispatch_view | apply.rollback | - | - | 4.2 (4.1–4.4) · p95 6.4 |
| wasm | probe.dispatch_view | view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | view.names | - | - | 0.8 (0.8–0.8) · p95 1.1 |
| wasm | probe.dispatch_view | view.sort | - | - | 2.3 (2.1–2.3) · p95 3.2 |
| wasm | probe.dispatch_view | view.commitments | - | - | 0.7 (0.7–0.8) · p95 0.9 |
| wasm | probe.dispatch_view | view.relations | - | - | 1.0 (0.9–1.0) · p95 1.7 |
| wasm | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.dispatch_view | web.serialize | - | - | 3.7 (3.6–3.8) · p95 5.4 |
| wasm | probe.dispatch_view | web.drop | - | - | 0.7 (0.7–0.7) · p95 1.3 |
| wasm | probe.dispatch_view | exit | - | - | 1.0 (1.0–1.1) · p95 4.7 |
| wasm | probe.dispatch_view | js.parse | - | - | 8.0 (7.8–8.6) · p95 12.2 |
| wasm | probe.dispatch_outcome | total | - | - | 127.4 (122.1–132.1) · p95 165.0 |
| wasm | probe.dispatch_outcome | web.enter | - | - | 0.5 (0.4–0.5) · p95 1.0 |
| wasm | probe.dispatch_outcome | payload.parse | - | - | 1.3 (1.2–1.4) · p95 2.7 |
| wasm | probe.dispatch_outcome | payload.resolve | - | - | 0.5 (0.5–0.5) · p95 0.9 |
| wasm | probe.dispatch_outcome | apply.validate | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.dispatch_outcome | apply.clone | - | - | 0.3 (0.3–0.4) · p95 0.5 |
| wasm | probe.dispatch_outcome | run.clock | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.dispatch_outcome | run.rules | - | - | 13.4 (12.9–13.8) · p95 18.5 |
| wasm | probe.dispatch_outcome | run.changes | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.dispatch_outcome | bind.stale | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | bind.eval | - | - | 3.5 (3.3–3.6) · p95 4.2 |
| wasm | probe.dispatch_outcome | bind.sort | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | bind.explain | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| wasm | probe.dispatch_outcome | bind.write | - | - | 1.2 (1.1–1.3) · p95 1.8 |
| wasm | probe.dispatch_outcome | apply.swap | - | - | 4.6 (4.5–4.9) · p95 6.6 |
| wasm | probe.dispatch_outcome | apply.rollback | - | - | 4.6 (4.4–4.7) · p95 7.1 |
| wasm | probe.dispatch_outcome | snap.enter | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | snap.names | - | - | 0.9 (0.8–1.0) · p95 1.2 |
| wasm | probe.dispatch_outcome | snap.sort | - | - | 2.5 (2.4–2.6) · p95 3.5 |
| wasm | probe.dispatch_outcome | snap.symbols | - | - | 3.5 (3.2–3.5) · p95 4.7 |
| wasm | probe.dispatch_outcome | snap.shown | - | - | 1.2 (1.1–1.2) · p95 1.7 |
| wasm | probe.dispatch_outcome | snap.values | - | - | 3.2 (3.0–3.3) · p95 4.6 |
| wasm | probe.dispatch_outcome | snap.records | - | - | 2.0 (1.9–2.1) · p95 2.8 |
| wasm | probe.dispatch_outcome | snap.static | - | - | 1.7 (1.6–1.8) · p95 2.3 |
| wasm | probe.dispatch_outcome | snap.relations | - | - | 0.8 (0.7–0.8) · p95 1.4 |
| wasm | probe.dispatch_outcome | snap.build | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_outcome | outcome.built | - | - | 0.1 (0.1–0.1) · p95 0.3 |
| wasm | probe.dispatch_outcome | web.serialize | - | - | 20.6 (20.0–21.4) · p95 27.6 |
| wasm | probe.dispatch_outcome | web.drop | - | - | 10.2 (9.9–10.6) · p95 13.3 |
| wasm | probe.dispatch_outcome | exit | - | - | 3.2 (2.9–3.2) · p95 6.7 |
| wasm | probe.dispatch_outcome | js.parse | - | - | 51.0 (49.2–52.7) · p95 65.7 |
| wasm | probe.read | view:total | - | - | 17.6 (16.7–18.4) · p95 23.1 |
| wasm | probe.read | view:web.enter | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.read | view:view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | view:view.names | - | - | 0.7 (0.7–0.8) · p95 0.9 |
| wasm | probe.read | view:view.sort | - | - | 1.7 (1.6–1.8) · p95 2.3 |
| wasm | probe.read | view:view.commitments | - | - | 0.7 (0.7–0.7) · p95 0.9 |
| wasm | probe.read | view:view.relations | - | - | 1.0 (0.9–1.0) · p95 1.7 |
| wasm | probe.read | view:view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | view:web.serialize | - | - | 2.8 (2.7–3.0) · p95 4.0 |
| wasm | probe.read | view:web.drop | - | - | 0.6 (0.6–0.7) · p95 1.1 |
| wasm | probe.read | view:exit | - | - | 0.6 (0.6–0.6) · p95 1.0 |
| wasm | probe.read | view:js.parse | - | - | 8.9 (8.3–9.3) · p95 11.9 |
| wasm | probe.read | snapshot:total | - | - | 134.5 (127.8–140.9) · p95 164.7 |
| wasm | probe.read | snapshot:web.enter | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.read | snapshot:snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | snapshot:snap.names | - | - | 0.8 (0.8–0.8) · p95 1.0 |
| wasm | probe.read | snapshot:snap.sort | - | - | 2.2 (2.1–2.3) · p95 2.9 |
| wasm | probe.read | snapshot:snap.symbols | - | - | 4.1 (3.9–4.2) · p95 5.1 |
| wasm | probe.read | snapshot:snap.shown | - | - | 1.3 (1.3–1.4) · p95 1.8 |
| wasm | probe.read | snapshot:snap.values | - | - | 3.5 (3.3–3.7) · p95 4.8 |
| wasm | probe.read | snapshot:snap.records | - | - | 2.2 (2.1–2.3) · p95 3.1 |
| wasm | probe.read | snapshot:snap.static | - | - | 1.8 (1.7–1.9) · p95 2.5 |
| wasm | probe.read | snapshot:snap.relations | - | - | 0.9 (0.9–0.9) · p95 1.4 |
| wasm | probe.read | snapshot:snap.build | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.read | snapshot:web.serialize | - | - | 43.0 (41.0–44.4) · p95 53.2 |
| wasm | probe.read | snapshot:web.drop | - | - | 11.0 (10.5–11.6) · p95 13.7 |
| wasm | probe.read | snapshot:exit | - | - | 4.9 (4.1–4.9) · p95 11.1 |
| wasm | probe.read | snapshot:js.parse | - | - | 57.0 (54.3–60.2) · p95 70.3 |
| wasm | probe.read | save:total | - | - | 18.3 (17.5–19.0) · p95 26.0 |
| wasm | probe.read | save:web.enter | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.read | save:save.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | save:save.build | - | - | 7.2 (7.0–7.4) · p95 10.5 |
| wasm | probe.read | save:save.serialize | - | - | 6.3 (5.9–6.5) · p95 8.7 |
| wasm | probe.read | save:save.drop | - | - | 3.4 (3.3–3.5) · p95 4.9 |
| wasm | probe.read | save:exit | - | - | 0.9 (0.9–1.0) · p95 1.9 |
| wasm | probe.lifecycle | restore:total | - | - | 1944.2 (1673.6–2025.9) · p95 3093.5 |
| wasm | probe.lifecycle | restore:web.enter | - | - | 15.1 (13.4–16.2) · p95 20.5 |
| wasm | probe.lifecycle | restore:restore.enter | - | - | 0.2 (0.2–0.2) · p95 0.5 |
| wasm | probe.lifecycle | restore:restore.parse | - | - | 57.4 (51.8–61.3) · p95 81.1 |
| wasm | probe.lifecycle | restore:load.enter | - | - | 0.2 (0.2–0.3) · p95 0.4 |
| wasm | probe.lifecycle | restore:load.parse | - | - | 851.2 (731.2–889.3) · p95 1256.4 |
| wasm | probe.lifecycle | restore:load.expand | - | - | 238.1 (200.1–249.2) · p95 352.2 |
| wasm | probe.lifecycle | restore:load.map | - | - | 46.5 (39.1–46.9) · p95 83.0 |
| wasm | probe.lifecycle | restore:load.declare | - | - | 161.0 (134.7–167.1) · p95 251.7 |
| wasm | probe.lifecycle | restore:load.procedures | - | - | 2.3 (1.9–2.5) · p95 3.9 |
| wasm | probe.lifecycle | restore:load.validate | - | - | 334.8 (306.7–356.1) · p95 672.6 |
| wasm | probe.lifecycle | restore:load.bindings | - | - | 11.9 (10.6–13.6) · p95 22.1 |
| wasm | probe.lifecycle | restore:restore.loaded | - | - | 126.1 (107.0–134.8) · p95 174.3 |
| wasm | probe.lifecycle | restore:restore.header | - | - | 1.2 (0.9–1.3) · p95 2.6 |
| wasm | probe.lifecycle | restore:restore.graph | - | - | 20.4 (17.8–22.1) · p95 41.4 |
| wasm | probe.lifecycle | restore:restore.states | - | - | 7.4 (6.1–7.8) · p95 13.0 |
| wasm | probe.lifecycle | restore:restore.records | - | - | 32.0 (26.3–33.1) · p95 55.6 |
| wasm | probe.lifecycle | restore:restore.bindings | - | - | 10.6 (9.8–12.4) · p95 18.5 |
| wasm | probe.lifecycle | restore:restore.done | - | - | 0.4 (0.3–0.4) · p95 0.5 |
| wasm | probe.lifecycle | restore:restore.drop_save | - | - | 7.6 (6.6–8.5) · p95 11.9 |
| wasm | probe.lifecycle | restore:exit | - | - | 5.2 (4.1–5.3) · p95 17.2 |
| wasm | probe.lifecycle | new:total | - | - | 1793.4 (1532.8–1893.2) · p95 2909.6 |
| wasm | probe.lifecycle | new:bind.stale | - | - | 0.4 (0.3–0.4) · p95 1.2 |
| wasm | probe.lifecycle | new:bind.eval | - | - | 7.5 (6.9–8.9) · p95 15.9 |
| wasm | probe.lifecycle | new:bind.sort | - | - | 0.5 (0.4–0.5) · p95 0.7 |
| wasm | probe.lifecycle | new:bind.explain | - | - | 0.7 (0.7–0.8) · p95 1.2 |
| wasm | probe.lifecycle | new:bind.write | - | - | 2.8 (2.3–3.0) · p95 5.7 |
| wasm | probe.lifecycle | new:load.enter | - | - | 8.7 (7.3–8.8) · p95 12.1 |
| wasm | probe.lifecycle | new:load.parse | - | - | 849.6 (707.8–894.8) · p95 1251.9 |
| wasm | probe.lifecycle | new:load.expand | - | - | 229.0 (198.0–251.4) · p95 433.8 |
| wasm | probe.lifecycle | new:load.map | - | - | 46.1 (41.2–48.9) · p95 83.1 |
| wasm | probe.lifecycle | new:load.declare | - | - | 157.8 (133.2–167.8) · p95 266.5 |
| wasm | probe.lifecycle | new:load.procedures | - | - | 2.2 (2.0–2.5) · p95 4.5 |
| wasm | probe.lifecycle | new:load.validate | - | - | 337.8 (290.5–355.4) · p95 706.9 |
| wasm | probe.lifecycle | new:load.bindings | - | - | 0.4 (0.4–0.5) · p95 0.7 |
| wasm | probe.lifecycle | new:exit | - | - | 134.3 (113.8–136.7) · p95 217.1 |
| wasm | probe.overhead | probe.mark | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| wasm | probe.overhead | js.performance_now | - | - | 0.0 (0.0–0.0) · p95 0.1 |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | main-local | i1 | i2 |
| --- | --- | --- | --- | --- | --- |
| native | apply | apply | 52.7 (48.0–54.7) · p95 135.9 | 51.1 (49.1–53.3) · p95 131.7 | - |
| native | web.dispatch_view | web.dispatch_view | 65.5 (61.1–68.6) · p95 146.8 | 64.6 (60.5–66.9) · p95 144.3 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | 128.0 (123.2–128.6) · p95 215.0 | 128.4 (122.4–130.8) · p95 215.2 | - |
| native | read | clone | 11.1 (10.8–11.4) · p95 13.4 | 11.1 (10.8–11.6) · p95 13.2 | - |
| native | read | clone.drop | 6.1 (5.9–6.2) · p95 7.2 | 6.1 (5.8–6.2) · p95 7.2 | - |
| native | read | save | 5.8 (5.6–5.9) · p95 10.2 | 5.9 (5.4–5.9) · p95 10.3 | - |
| native | read | save_json | 9.9 (9.5–10.0) · p95 17.1 | 9.8 (9.3–10.0) · p95 17.2 | - |
| native | read | snapshot | 32.5 (31.2–33.1) · p95 37.9 | 32.2 (31.2–33.3) · p95 37.3 | - |
| native | read | snapshot.drop | 12.9 (12.4–13.2) · p95 15.6 | 13.0 (12.5–13.4) · p95 15.6 | - |
| native | read | snapshot.serialize | 20.5 (19.7–20.9) · p95 23.9 | 20.3 (19.4–20.8) · p95 23.8 | - |
| native | read | snapshot.serialize_pretty | 41.1 (39.4–41.6) · p95 48.9 | 40.6 (38.7–41.5) · p95 48.2 | - |
| native | read | view | 1.4 (1.3–1.4) · p95 2.4 | 1.4 (1.3–1.4) · p95 2.4 | - |
| native | read | view.serialize | 7.2 (6.9–7.4) · p95 9.5 | 7.2 (6.9–7.4) · p95 9.3 | - |
| native | web.read | web.save | 11.9 (11.8–12.0) · p95 20.2 | 12.1 (11.9–12.5) · p95 20.3 | - |
| native | web.read | web.snapshot | 86.8 (86.7–88.1) · p95 103.0 | 87.7 (86.1–89.7) · p95 103.5 | - |
| native | web.read | web.view | 6.8 (6.7–6.8) · p95 9.6 | 6.7 (6.6–7.0) · p95 9.6 | - |
| native | lifecycle | from_source | 3171.9 (3139.3–3185.0) · p95 3416.6 | 3123.4 (3107.9–3184.0) · p95 3352.5 | - |
| native | lifecycle | restore | 3272.3 (3255.6–3284.6) · p95 3566.1 | 3262.1 (3222.3–3264.5) · p95 3483.4 | - |
| native | lifecycle | restore.parse | 28.2 (21.9–29.5) · p95 35.7 | 27.7 (21.4–28.1) · p95 37.1 | - |
| native | lifecycle | restore_json | 3300.8 (3239.3–3341.7) · p95 3523.2 | 3245.7 (3231.8–3313.2) · p95 3451.9 | - |
| native | lifecycle | save | 17.8 (14.4–18.3) · p95 22.1 | 18.5 (14.9–19.6) · p95 25.7 | - |
| native | lifecycle | save_json | 17.8 (15.9–18.3) · p95 26.8 | 20.4 (16.6–20.5) · p95 31.7 | - |
| native | lifecycle | web.new | 3164.2 (3150.3–3231.7) · p95 3360.8 | 3090.6 (3072.0–3157.8) · p95 3381.0 | - |
| native | lifecycle | web.restore | 3296.7 (3284.2–3318.1) · p95 3524.0 | 3267.5 (3219.4–3307.8) · p95 3465.5 | - |
| native | lifecycle | web.save | 14.0 (11.7–14.6) · p95 16.6 | 13.9 (11.5–14.8) · p95 18.1 | - |
| native | bench.dispatch_only | bench.dispatch_only | - | 52.6 (52.4–53.7) · p95 133.8 | - |
| native | bench.read | bench.snapshot_build | - | 43.6 (42.2–43.6) · p95 51.3 | - |
| native | bench.read | bench.view_build | - | 2.4 (2.3–2.4) · p95 4.1 | - |
| native | bench.read | web.snapshot | - | 81.0 (78.6–81.2) · p95 96.9 | - |
| native | bench.read | web.view | - | 9.7 (9.4–9.8) · p95 13.2 | - |
| native | probe.dispatch_view | apply.clone | - | - | 0.5 (0.5–0.5) · p95 0.7 |
| native | probe.dispatch_view | apply.rollback | - | - | 2.8 (2.7–2.9) · p95 6.5 |
| native | probe.dispatch_view | apply.swap | - | - | 2.5 (2.5–2.6) · p95 3.6 |
| native | probe.dispatch_view | apply.validate | - | - | 0.2 (0.2–0.2) · p95 0.7 |
| native | probe.dispatch_view | bind.eval | - | - | 14.1 (13.9–14.5) · p95 31.0 |
| native | probe.dispatch_view | bind.explain | - | - | 4.6 (4.6–4.8) · p95 8.2 |
| native | probe.dispatch_view | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | bind.sort | - | - | 1.9 (1.9–1.9) · p95 3.1 |
| native | probe.dispatch_view | bind.stale | - | - | 1.3 (1.3–1.4) · p95 2.1 |
| native | probe.dispatch_view | bind.write | - | - | 8.6 (8.5–8.8) · p95 11.7 |
| native | probe.dispatch_view | exit | - | - | 0.1 (0.1–0.1) · p95 0.3 |
| native | probe.dispatch_view | payload.parse | - | - | 0.9 (0.9–0.9) · p95 2.9 |
| native | probe.dispatch_view | payload.resolve | - | - | 0.5 (0.5–0.5) · p95 1.1 |
| native | probe.dispatch_view | run.changes | - | - | 0.9 (0.9–0.9) · p95 1.8 |
| native | probe.dispatch_view | run.clock | - | - | 0.2 (0.2–0.2) · p95 6.4 |
| native | probe.dispatch_view | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| native | probe.dispatch_view | run.rules | - | - | 39.6 (38.0–40.5) · p95 83.6 |
| native | probe.dispatch_view | total | - | - | 66.9 (66.1–69.5) · p95 150.7 |
| native | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.commitments | - | - | 0.4 (0.4–0.4) · p95 1.1 |
| native | probe.dispatch_view | view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_view | view.names | - | - | 0.3 (0.2–0.3) · p95 0.4 |
| native | probe.dispatch_view | view.relations | - | - | 0.7 (0.7–0.8) · p95 1.5 |
| native | probe.dispatch_view | view.sort | - | - | 0.6 (0.6–0.6) · p95 0.9 |
| native | probe.dispatch_view | web.drop | - | - | 0.4 (0.4–0.4) · p95 0.9 |
| native | probe.dispatch_view | web.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_view | web.serialize | - | - | 7.3 (7.2–7.6) · p95 9.9 |
| native | probe.dispatch_outcome | apply.clone | - | - | 0.5 (0.5–0.5) · p95 0.7 |
| native | probe.dispatch_outcome | apply.rollback | - | - | 3.0 (2.9–3.1) · p95 7.4 |
| native | probe.dispatch_outcome | apply.swap | - | - | 2.7 (2.6–2.7) · p95 3.9 |
| native | probe.dispatch_outcome | apply.validate | - | - | 0.3 (0.3–0.3) · p95 0.8 |
| native | probe.dispatch_outcome | bind.eval | - | - | 14.4 (14.0–14.5) · p95 32.2 |
| native | probe.dispatch_outcome | bind.explain | - | - | 4.8 (4.7–4.9) · p95 8.6 |
| native | probe.dispatch_outcome | bind.none | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | bind.sort | - | - | 2.0 (2.0–2.0) · p95 3.3 |
| native | probe.dispatch_outcome | bind.stale | - | - | 1.5 (1.4–1.5) · p95 2.2 |
| native | probe.dispatch_outcome | bind.write | - | - | 9.3 (9.3–9.5) · p95 12.6 |
| native | probe.dispatch_outcome | exit | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.dispatch_outcome | outcome.built | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| native | probe.dispatch_outcome | payload.parse | - | - | 1.1 (1.0–1.1) · p95 3.1 |
| native | probe.dispatch_outcome | payload.resolve | - | - | 0.5 (0.5–0.5) · p95 1.1 |
| native | probe.dispatch_outcome | run.changes | - | - | 1.0 (1.0–1.0) · p95 1.8 |
| native | probe.dispatch_outcome | run.clock | - | - | 0.2 (0.2–0.2) · p95 7.0 |
| native | probe.dispatch_outcome | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.7 |
| native | probe.dispatch_outcome | run.rules | - | - | 41.0 (39.8–41.3) · p95 88.7 |
| native | probe.dispatch_outcome | snap.build | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_outcome | snap.enter | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.dispatch_outcome | snap.names | - | - | 0.3 (0.3–0.3) · p95 0.4 |
| native | probe.dispatch_outcome | snap.records | - | - | 1.4 (1.4–1.4) · p95 3.0 |
| native | probe.dispatch_outcome | snap.relations | - | - | 0.8 (0.8–0.8) · p95 1.6 |
| native | probe.dispatch_outcome | snap.shown | - | - | 11.0 (10.8–11.0) · p95 13.7 |
| native | probe.dispatch_outcome | snap.sort | - | - | 0.7 (0.7–0.7) · p95 1.0 |
| native | probe.dispatch_outcome | snap.static | - | - | 2.8 (2.7–2.8) · p95 3.6 |
| native | probe.dispatch_outcome | snap.symbols | - | - | 2.9 (2.8–2.9) · p95 3.6 |
| native | probe.dispatch_outcome | snap.values | - | - | 13.6 (13.2–13.6) · p95 16.7 |
| native | probe.dispatch_outcome | total | - | - | 133.6 (129.6–134.9) · p95 225.1 |
| native | probe.dispatch_outcome | web.drop | - | - | 13.1 (12.9–13.2) · p95 16.6 |
| native | probe.dispatch_outcome | web.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.dispatch_outcome | web.serialize | - | - | 21.8 (21.8–21.8) · p95 28.1 |
| native | probe.read | save:exit | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.read | save:save.build | - | - | 6.6 (5.8–7.3) · p95 11.5 |
| native | probe.read | save:save.drop | - | - | 2.1 (1.9–2.3) · p95 3.7 |
| native | probe.read | save:save.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.read | save:save.serialize | - | - | 4.3 (3.8–4.6) · p95 6.8 |
| native | probe.read | save:total | - | - | 13.2 (11.7–14.4) · p95 22.1 |
| native | probe.read | save:web.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.read | snapshot:exit | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.read | snapshot:snap.build | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| native | probe.read | snapshot:snap.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.read | snapshot:snap.names | - | - | 0.3 (0.2–0.3) · p95 0.4 |
| native | probe.read | snapshot:snap.records | - | - | 1.4 (1.2–1.5) · p95 3.1 |
| native | probe.read | snapshot:snap.relations | - | - | 0.7 (0.7–0.8) · p95 1.6 |
| native | probe.read | snapshot:snap.shown | - | - | 11.9 (10.5–13.0) · p95 14.3 |
| native | probe.read | snapshot:snap.sort | - | - | 0.4 (0.3–0.4) · p95 0.5 |
| native | probe.read | snapshot:snap.static | - | - | 3.1 (2.7–3.3) · p95 3.8 |
| native | probe.read | snapshot:snap.symbols | - | - | 2.9 (2.6–3.2) · p95 3.7 |
| native | probe.read | snapshot:snap.values | - | - | 15.0 (12.8–15.7) · p95 18.2 |
| native | probe.read | snapshot:total | - | - | 97.6 (85.5–105.5) · p95 115.0 |
| native | probe.read | snapshot:web.drop | - | - | 14.7 (12.9–15.8) · p95 18.0 |
| native | probe.read | snapshot:web.enter | - | - | 0.0 (0.0–0.0) · p95 0.1 |
| native | probe.read | snapshot:web.serialize | - | - | 46.1 (40.5–50.0) · p95 54.3 |
| native | probe.read | view:exit | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.read | view:total | - | - | 7.7 (6.8–8.3) · p95 10.8 |
| native | probe.read | view:view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| native | probe.read | view:view.commitments | - | - | 0.3 (0.3–0.4) · p95 0.7 |
| native | probe.read | view:view.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.read | view:view.names | - | - | 0.2 (0.2–0.3) · p95 0.4 |
| native | probe.read | view:view.relations | - | - | 0.6 (0.6–0.7) · p95 1.4 |
| native | probe.read | view:view.sort | - | - | 0.4 (0.4–0.4) · p95 0.6 |
| native | probe.read | view:web.drop | - | - | 0.4 (0.3–0.4) · p95 0.7 |
| native | probe.read | view:web.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.read | view:web.serialize | - | - | 5.5 (4.8–5.9) · p95 7.2 |
| native | probe.lifecycle | new:bind.eval | - | - | 15.7 (15.6–16.2) · p95 19.2 |
| native | probe.lifecycle | new:bind.explain | - | - | 8.0 (7.7–8.4) · p95 10.8 |
| native | probe.lifecycle | new:bind.sort | - | - | 8.8 (8.7–9.0) · p95 9.8 |
| native | probe.lifecycle | new:bind.stale | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.lifecycle | new:bind.write | - | - | 29.8 (27.4–32.6) · p95 38.5 |
| native | probe.lifecycle | new:exit | - | - | 166.5 (164.5–169.8) · p95 186.4 |
| native | probe.lifecycle | new:load.bindings | - | - | 0.9 (0.9–0.9) · p95 1.0 |
| native | probe.lifecycle | new:load.declare | - | - | 288.8 (281.2–303.9) · p95 349.0 |
| native | probe.lifecycle | new:load.enter | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| native | probe.lifecycle | new:load.expand | - | - | 474.8 (466.6–475.9) · p95 529.5 |
| native | probe.lifecycle | new:load.map | - | - | 19.6 (18.6–20.5) · p95 28.2 |
| native | probe.lifecycle | new:load.parse | - | - | 1236.8 (1219.9–1271.2) · p95 1367.1 |
| native | probe.lifecycle | new:load.procedures | - | - | 245.9 (245.7–276.2) · p95 317.1 |
| native | probe.lifecycle | new:load.validate | - | - | 823.5 (821.6–831.4) · p95 884.1 |
| native | probe.lifecycle | new:total | - | - | 3297.9 (3279.8–3413.5) · p95 3732.2 |
| native | probe.lifecycle | restore:exit | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| native | probe.lifecycle | restore:load.bindings | - | - | 64.4 (60.3–68.1) · p95 73.7 |
| native | probe.lifecycle | restore:load.declare | - | - | 301.7 (293.6–338.9) · p95 368.1 |
| native | probe.lifecycle | restore:load.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.lifecycle | restore:load.expand | - | - | 467.9 (462.4–476.7) · p95 515.3 |
| native | probe.lifecycle | restore:load.map | - | - | 21.3 (20.2–21.6) · p95 24.6 |
| native | probe.lifecycle | restore:load.parse | - | - | 1229.2 (1197.0–1268.2) · p95 1390.5 |
| native | probe.lifecycle | restore:load.procedures | - | - | 274.9 (264.0–286.0) · p95 314.6 |
| native | probe.lifecycle | restore:load.validate | - | - | 792.4 (789.7–796.1) · p95 868.5 |
| native | probe.lifecycle | restore:restore.bindings | - | - | 77.5 (76.9–78.5) · p95 86.0 |
| native | probe.lifecycle | restore:restore.done | - | - | 0.3 (0.2–0.3) · p95 0.3 |
| native | probe.lifecycle | restore:restore.drop_save | - | - | 3.7 (3.6–3.7) · p95 4.4 |
| native | probe.lifecycle | restore:restore.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| native | probe.lifecycle | restore:restore.graph | - | - | 4.4 (4.2–4.7) · p95 6.0 |
| native | probe.lifecycle | restore:restore.header | - | - | 1.0 (1.0–1.1) · p95 1.2 |
| native | probe.lifecycle | restore:restore.loaded | - | - | 163.3 (162.0–171.2) · p95 189.7 |
| native | probe.lifecycle | restore:restore.parse | - | - | 20.1 (18.9–20.2) · p95 25.3 |
| native | probe.lifecycle | restore:restore.records | - | - | 14.5 (12.3–14.9) · p95 18.2 |
| native | probe.lifecycle | restore:restore.states | - | - | 9.5 (9.4–9.5) · p95 11.1 |
| native | probe.lifecycle | restore:total | - | - | 3483.8 (3391.9–3552.5) · p95 3821.0 |
| native | probe.lifecycle | restore:web.enter | - | - | 0.0 (0.0–0.1) · p95 0.1 |
| native | probe.overhead | probe.mark | - | - | 0.0 (0.0–0.0) · p95 0.0 |
| native | probe.overhead | probe.now | - | - | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.6 (0.6–0.6) · p95 2.0 | 0.6 (0.6–0.6) · p95 2.1 | - |
| wasm | raw.dispatch_view | raw.dispatch_view | 80.9 (75.8–82.0) · p95 167.0 | 81.0 (76.2–81.3) · p95 166.6 | - |
| wasm | raw.dispatch_view | js.parse_view | 27.9 (26.3–28.5) · p95 36.4 | 28.2 (26.8–28.4) · p95 36.4 | - |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 134.3 (125.7–134.9) · p95 203.5 | 133.9 (128.0–134.9) · p95 203.3 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 155.2 (149.6–158.1) · p95 248.4 | 158.5 (151.3–177.5) · p95 256.1 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 82.4 (80.0–82.8) · p95 99.4 | 84.0 (81.1–90.0) · p95 102.7 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 241.4 (233.7–243.7) · p95 342.8 | 245.4 (235.1–269.7) · p95 352.4 | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.4 (0.4–0.5) · p95 1.2 | 0.4 (0.4–0.5) · p95 1.2 | - |
| wasm | abi.dispatch_view | abi.exec | 75.8 (73.2–82.2) · p95 157.9 | 78.0 (74.8–80.7) · p95 163.8 | - |
| wasm | abi.dispatch_view | abi.decode | 1.3 (1.3–1.5) · p95 4.3 | 1.4 (1.3–1.5) · p95 4.6 | - |
| wasm | abi.dispatch_view | abi.free | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | - |
| wasm | abi.dispatch_view | abi.dispatch_view | 102.9 (98.8–109.4) · p95 164.1 | 105.3 (100.5–106.9) · p95 170.4 | - |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.5 (0.5–0.5) · p95 1.3 | 0.5 (0.5–0.5) · p95 1.3 | - |
| wasm | abi.dispatch_outcome | abi.exec | 154.5 (145.0–157.2) · p95 256.2 | 149.5 (148.2–154.8) · p95 245.6 | - |
| wasm | abi.dispatch_outcome | abi.decode | 3.6 (3.6–3.8) · p95 8.6 | 3.7 (3.6–3.7) · p95 8.6 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.5 (0.5–0.5) · p95 1.1 | 0.5 (0.5–0.5) · p95 1.1 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 160.3 (151.8–163.6) · p95 265.7 | 155.5 (154.4–161.4) · p95 253.7 | - |
| wasm | read | raw.view | 11.4 (11.3–11.6) · p95 15.7 | 11.0 (10.5–11.9) · p95 15.2 | - |
| wasm | read | js.parse_view | 29.1 (29.0–29.6) · p95 37.4 | 28.5 (27.1–30.3) · p95 36.5 | - |
| wasm | read | raw.snapshot | 119.9 (119.0–124.0) · p95 150.3 | 116.4 (111.6–127.4) · p95 146.7 | - |
| wasm | read | js.parse_snapshot | 91.3 (90.8–94.4) · p95 110.1 | 89.0 (85.3–96.6) · p95 107.1 | - |
| wasm | read | raw.save | 16.5 (16.3–16.9) · p95 28.3 | 16.0 (15.2–17.3) · p95 27.6 | - |
| wasm | abi.read | abi.view.exec | 10.1 (9.6–10.4) · p95 13.9 | 10.4 (9.7–11.2) · p95 14.2 | - |
| wasm | abi.read | abi.view.decode | 0.9 (0.9–0.9) · p95 1.5 | 1.0 (0.9–1.0) · p95 1.5 | - |
| wasm | abi.read | abi.snapshot.exec | 109.8 (102.6–110.3) · p95 131.8 | 111.2 (104.4–119.7) · p95 138.0 | - |
| wasm | abi.read | abi.snapshot.decode | 5.3 (4.9–5.5) · p95 9.7 | 5.5 (4.9–5.9) · p95 10.1 | - |
| wasm | abi.read | abi.save.exec | 14.9 (14.0–15.0) · p95 25.9 | 15.3 (14.1–16.7) · p95 26.7 | - |
| wasm | abi.read | abi.save.decode | 0.5 (0.5–0.5) · p95 0.9 | 0.5 (0.5–0.6) · p95 0.9 | - |
| wasm | lifecycle | raw.new | 3240.6 (3220.5–3583.4) · p95 3886.4 | 3742.4 (3224.2–3771.9) · p95 4238.1 | - |
| wasm | lifecycle | raw.save | 49.1 (46.1–51.6) · p95 82.3 | 51.2 (47.5–54.3) · p95 76.7 | - |
| wasm | lifecycle | raw.restore | 3270.5 (3212.9–3538.7) · p95 3878.7 | 3768.0 (3197.1–3799.0) · p95 4165.1 | - |
| wasm | lifecycle | kit.open | 3146.9 (3099.0–3382.3) · p95 3627.6 | 3641.5 (3160.7–3659.7) · p95 4016.3 | - |
| wasm | lifecycle | kit.save | 98.0 (96.7–103.5) · p95 143.8 | 107.5 (101.4–122.8) · p95 136.6 | - |
| wasm | lifecycle | kit.restore | 3218.9 (3211.0–3533.3) · p95 3871.1 | 3682.9 (3176.1–3775.3) · p95 4167.0 | - |
| wasm | abi.dispatch_only | abi.encode_args | - | 0.4 (0.3–0.4) · p95 0.9 | - |
| wasm | abi.dispatch_only | abi.dispatch_only.exec | - | 60.9 (60.7–61.6) · p95 146.3 | - |
| wasm | abi.dispatch_only | abi.dispatch_only | - | 61.4 (61.1–62.0) · p95 146.6 | - |
| wasm | abi.bench_read | abi.view_build.exec | - | 2.7 (2.6–2.7) · p95 4.6 | - |
| wasm | abi.bench_read | abi.snapshot_build.exec | - | 45.9 (45.4–46.2) · p95 57.8 | - |
| wasm | abi.bench_read | abi.view.exec | - | 13.8 (13.6–13.9) · p95 18.3 | - |
| wasm | abi.bench_read | abi.view.decode | - | 1.2 (1.1–1.2) · p95 3.3 | - |
| wasm | abi.bench_read | abi.snapshot.exec | - | 100.2 (99.4–101.5) · p95 123.9 | - |
| wasm | abi.bench_read | abi.snapshot.decode | - | 5.5 (5.2–5.5) · p95 10.0 | - |
| wasm | probe.dispatch_view | total | - | - | 119.1 (115.2–119.3) · p95 208.1 |
| wasm | probe.dispatch_view | web.enter | - | - | 0.6 (0.6–0.6) · p95 1.3 |
| wasm | probe.dispatch_view | payload.parse | - | - | 1.3 (1.3–1.3) · p95 3.8 |
| wasm | probe.dispatch_view | payload.resolve | - | - | 0.5 (0.5–0.6) · p95 1.2 |
| wasm | probe.dispatch_view | apply.validate | - | - | 0.3 (0.3–0.3) · p95 0.8 |
| wasm | probe.dispatch_view | apply.clone | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| wasm | probe.dispatch_view | run.clock | - | - | 0.2 (0.2–0.2) · p95 7.5 |
| wasm | probe.dispatch_view | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| wasm | probe.dispatch_view | run.rules | - | - | 43.9 (43.8–45.0) · p95 91.4 |
| wasm | probe.dispatch_view | run.changes | - | - | 1.3 (1.3–1.3) · p95 2.1 |
| wasm | probe.dispatch_view | bind.stale | - | - | 1.8 (1.7–1.8) · p95 2.6 |
| wasm | probe.dispatch_view | bind.none | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| wasm | probe.dispatch_view | bind.eval | - | - | 16.1 (15.4–16.3) · p95 35.0 |
| wasm | probe.dispatch_view | bind.sort | - | - | 1.8 (1.8–1.9) · p95 3.0 |
| wasm | probe.dispatch_view | bind.explain | - | - | 4.9 (4.9–5.0) · p95 8.8 |
| wasm | probe.dispatch_view | bind.write | - | - | 8.2 (8.0–8.2) · p95 11.8 |
| wasm | probe.dispatch_view | apply.swap | - | - | 3.0 (3.0–3.1) · p95 5.1 |
| wasm | probe.dispatch_view | apply.rollback | - | - | 3.0 (2.9–3.1) · p95 7.8 |
| wasm | probe.dispatch_view | view.enter | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | view.names | - | - | 0.5 (0.4–0.5) · p95 0.6 |
| wasm | probe.dispatch_view | view.sort | - | - | 0.8 (0.8–0.9) · p95 1.2 |
| wasm | probe.dispatch_view | view.commitments | - | - | 0.5 (0.5–0.5) · p95 1.2 |
| wasm | probe.dispatch_view | view.relations | - | - | 0.7 (0.7–0.7) · p95 1.5 |
| wasm | probe.dispatch_view | view.build | - | - | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | probe.dispatch_view | web.serialize | - | - | 11.3 (11.0–11.3) · p95 14.5 |
| wasm | probe.dispatch_view | web.drop | - | - | 0.5 (0.5–0.5) · p95 1.3 |
| wasm | probe.dispatch_view | exit | - | - | 1.8 (1.8–1.8) · p95 6.8 |
| wasm | probe.dispatch_view | js.parse | - | - | 29.6 (28.7–29.8) · p95 38.4 |
| wasm | probe.dispatch_outcome | total | - | - | 254.7 (251.1–262.7) · p95 357.3 |
| wasm | probe.dispatch_outcome | web.enter | - | - | 0.7 (0.7–0.8) · p95 1.6 |
| wasm | probe.dispatch_outcome | payload.parse | - | - | 1.6 (1.6–1.6) · p95 4.1 |
| wasm | probe.dispatch_outcome | payload.resolve | - | - | 0.6 (0.6–0.6) · p95 1.3 |
| wasm | probe.dispatch_outcome | apply.validate | - | - | 0.3 (0.3–0.3) · p95 0.8 |
| wasm | probe.dispatch_outcome | apply.clone | - | - | 0.4 (0.4–0.4) · p95 0.6 |
| wasm | probe.dispatch_outcome | run.clock | - | - | 0.2 (0.2–0.2) · p95 7.4 |
| wasm | probe.dispatch_outcome | run.prologue | - | - | 0.3 (0.3–0.3) · p95 0.6 |
| wasm | probe.dispatch_outcome | run.rules | - | - | 44.4 (43.3–45.4) · p95 91.2 |
| wasm | probe.dispatch_outcome | run.changes | - | - | 1.4 (1.3–1.4) · p95 2.1 |
| wasm | probe.dispatch_outcome | bind.stale | - | - | 1.9 (1.8–1.9) · p95 2.5 |
| wasm | probe.dispatch_outcome | bind.none | - | - | 0.2 (0.1–0.2) · p95 0.2 |
| wasm | probe.dispatch_outcome | bind.eval | - | - | 16.3 (15.6–17.2) · p95 35.5 |
| wasm | probe.dispatch_outcome | bind.sort | - | - | 1.9 (1.8–1.9) · p95 3.0 |
| wasm | probe.dispatch_outcome | bind.explain | - | - | 4.9 (4.8–5.0) · p95 8.8 |
| wasm | probe.dispatch_outcome | bind.write | - | - | 8.3 (8.0–8.3) · p95 11.7 |
| wasm | probe.dispatch_outcome | apply.swap | - | - | 3.2 (3.2–3.3) · p95 5.1 |
| wasm | probe.dispatch_outcome | apply.rollback | - | - | 3.2 (3.0–3.3) · p95 7.9 |
| wasm | probe.dispatch_outcome | snap.enter | - | - | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | probe.dispatch_outcome | snap.names | - | - | 0.5 (0.5–0.5) · p95 0.7 |
| wasm | probe.dispatch_outcome | snap.sort | - | - | 0.9 (0.9–0.9) · p95 1.2 |
| wasm | probe.dispatch_outcome | snap.symbols | - | - | 3.0 (2.9–3.1) · p95 3.8 |
| wasm | probe.dispatch_outcome | snap.shown | - | - | 9.5 (9.5–9.8) · p95 12.0 |
| wasm | probe.dispatch_outcome | snap.values | - | - | 13.5 (13.4–14.2) · p95 17.8 |
| wasm | probe.dispatch_outcome | snap.records | - | - | 1.2 (1.2–1.2) · p95 2.0 |
| wasm | probe.dispatch_outcome | snap.static | - | - | 1.9 (1.9–1.9) · p95 2.6 |
| wasm | probe.dispatch_outcome | snap.relations | - | - | 0.6 (0.6–0.6) · p95 1.1 |
| wasm | probe.dispatch_outcome | snap.build | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| wasm | probe.dispatch_outcome | outcome.built | - | - | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | probe.dispatch_outcome | web.serialize | - | - | 32.0 (31.8–32.2) · p95 41.0 |
| wasm | probe.dispatch_outcome | web.drop | - | - | 21.6 (21.1–22.0) · p95 27.5 |
| wasm | probe.dispatch_outcome | exit | - | - | 4.3 (3.9–4.4) · p95 8.6 |
| wasm | probe.dispatch_outcome | js.parse | - | - | 85.5 (82.3–87.2) · p95 105.8 |
| wasm | probe.read | view:total | - | - | 41.5 (40.3–41.9) · p95 51.3 |
| wasm | probe.read | view:web.enter | - | - | 0.3 (0.3–0.3) · p95 0.7 |
| wasm | probe.read | view:view.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | view:view.names | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| wasm | probe.read | view:view.sort | - | - | 0.4 (0.4–0.5) · p95 0.8 |
| wasm | probe.read | view:view.commitments | - | - | 0.4 (0.4–0.4) · p95 0.8 |
| wasm | probe.read | view:view.relations | - | - | 0.6 (0.6–0.6) · p95 1.3 |
| wasm | probe.read | view:view.build | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | view:web.serialize | - | - | 8.0 (8.0–8.2) · p95 10.5 |
| wasm | probe.read | view:web.drop | - | - | 0.4 (0.4–0.4) · p95 1.0 |
| wasm | probe.read | view:exit | - | - | 1.2 (1.2–1.2) · p95 2.6 |
| wasm | probe.read | view:js.parse | - | - | 29.0 (28.0–29.5) · p95 36.0 |
| wasm | probe.read | snapshot:total | - | - | 214.6 (207.1–216.2) · p95 253.7 |
| wasm | probe.read | snapshot:web.enter | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.read | snapshot:snap.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | snapshot:snap.names | - | - | 0.4 (0.4–0.4) · p95 0.5 |
| wasm | probe.read | snapshot:snap.sort | - | - | 0.5 (0.4–0.5) · p95 0.7 |
| wasm | probe.read | snapshot:snap.symbols | - | - | 2.9 (2.9–2.9) · p95 3.8 |
| wasm | probe.read | snapshot:snap.shown | - | - | 9.1 (9.0–9.3) · p95 11.8 |
| wasm | probe.read | snapshot:snap.values | - | - | 13.9 (13.6–13.9) · p95 17.2 |
| wasm | probe.read | snapshot:snap.records | - | - | 1.0 (1.0–1.1) · p95 2.0 |
| wasm | probe.read | snapshot:snap.static | - | - | 1.9 (1.9–2.0) · p95 2.6 |
| wasm | probe.read | snapshot:snap.relations | - | - | 0.5 (0.5–0.6) · p95 1.1 |
| wasm | probe.read | snapshot:snap.build | - | - | 0.2 (0.2–0.2) · p95 0.2 |
| wasm | probe.read | snapshot:web.serialize | - | - | 61.3 (60.2–62.3) · p95 72.8 |
| wasm | probe.read | snapshot:web.drop | - | - | 21.2 (20.6–21.3) · p95 26.6 |
| wasm | probe.read | snapshot:exit | - | - | 6.4 (5.6–6.4) · p95 19.8 |
| wasm | probe.read | snapshot:js.parse | - | - | 90.7 (87.9–92.9) · p95 108.4 |
| wasm | probe.read | save:total | - | - | 16.9 (16.5–17.1) · p95 27.7 |
| wasm | probe.read | save:web.enter | - | - | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | probe.read | save:save.enter | - | - | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | probe.read | save:save.build | - | - | 6.9 (6.6–6.9) · p95 11.3 |
| wasm | probe.read | save:save.serialize | - | - | 5.8 (5.7–5.8) · p95 9.2 |
| wasm | probe.read | save:save.drop | - | - | 3.1 (3.0–3.1) · p95 5.6 |
| wasm | probe.read | save:exit | - | - | 0.8 (0.8–0.8) · p95 1.5 |
| wasm | probe.lifecycle | restore:total | - | - | 3522.8 (3288.8–3526.5) · p95 4157.1 |
| wasm | probe.lifecycle | restore:web.enter | - | - | 14.0 (13.3–14.2) · p95 17.0 |
| wasm | probe.lifecycle | restore:restore.enter | - | - | 0.2 (0.2–0.3) · p95 0.4 |
| wasm | probe.lifecycle | restore:restore.parse | - | - | 35.8 (32.0–36.2) · p95 52.0 |
| wasm | probe.lifecycle | restore:load.enter | - | - | 0.2 (0.2–0.3) · p95 0.4 |
| wasm | probe.lifecycle | restore:load.parse | - | - | 1289.3 (1251.8–1291.7) · p95 1487.8 |
| wasm | probe.lifecycle | restore:load.expand | - | - | 330.8 (312.0–340.6) · p95 374.5 |
| wasm | probe.lifecycle | restore:load.map | - | - | 27.3 (25.5–27.6) · p95 42.0 |
| wasm | probe.lifecycle | restore:load.declare | - | - | 199.7 (188.7–207.9) · p95 230.6 |
| wasm | probe.lifecycle | restore:load.procedures | - | - | 162.5 (156.1–164.9) · p95 180.4 |
| wasm | probe.lifecycle | restore:load.validate | - | - | 979.5 (918.7–984.5) · p95 1135.2 |
| wasm | probe.lifecycle | restore:load.bindings | - | - | 72.2 (67.3–73.0) · p95 94.3 |
| wasm | probe.lifecycle | restore:restore.loaded | - | - | 213.9 (208.1–215.2) · p95 239.0 |
| wasm | probe.lifecycle | restore:restore.header | - | - | 1.3 (1.1–1.8) · p95 2.4 |
| wasm | probe.lifecycle | restore:restore.graph | - | - | 6.6 (5.6–6.7) · p95 9.1 |
| wasm | probe.lifecycle | restore:restore.states | - | - | 12.6 (12.4–13.4) · p95 16.4 |
| wasm | probe.lifecycle | restore:restore.records | - | - | 19.6 (16.6–20.5) · p95 28.6 |
| wasm | probe.lifecycle | restore:restore.bindings | - | - | 94.4 (90.9–94.8) · p95 126.9 |
| wasm | probe.lifecycle | restore:restore.done | - | - | 0.4 (0.3–0.4) · p95 0.5 |
| wasm | probe.lifecycle | restore:restore.drop_save | - | - | 5.1 (5.0–5.1) · p95 6.2 |
| wasm | probe.lifecycle | restore:exit | - | - | 5.8 (4.6–6.1) · p95 13.9 |
| wasm | probe.lifecycle | new:total | - | - | 3296.4 (3130.1–3310.9) · p95 3557.2 |
| wasm | probe.lifecycle | new:bind.stale | - | - | 0.5 (0.4–0.5) · p95 1.1 |
| wasm | probe.lifecycle | new:bind.eval | - | - | 22.1 (21.0–22.3) · p95 33.0 |
| wasm | probe.lifecycle | new:bind.sort | - | - | 11.8 (11.5–11.9) · p95 13.5 |
| wasm | probe.lifecycle | new:bind.explain | - | - | 10.0 (9.4–10.3) · p95 14.8 |
| wasm | probe.lifecycle | new:bind.write | - | - | 24.9 (24.5–26.1) · p95 30.3 |
| wasm | probe.lifecycle | new:load.enter | - | - | 10.4 (10.0–10.6) · p95 13.8 |
| wasm | probe.lifecycle | new:load.parse | - | - | 1272.4 (1243.6–1290.5) · p95 1407.1 |
| wasm | probe.lifecycle | new:load.expand | - | - | 320.8 (311.5–323.8) · p95 376.8 |
| wasm | probe.lifecycle | new:load.map | - | - | 26.4 (25.2–26.6) · p95 30.4 |
| wasm | probe.lifecycle | new:load.declare | - | - | 197.2 (184.3–198.0) · p95 220.9 |
| wasm | probe.lifecycle | new:load.procedures | - | - | 158.9 (155.8–159.2) · p95 179.9 |
| wasm | probe.lifecycle | new:load.validate | - | - | 968.0 (932.2–979.6) · p95 1100.2 |
| wasm | probe.lifecycle | new:load.bindings | - | - | 0.9 (0.9–0.9) · p95 1.3 |
| wasm | probe.lifecycle | new:exit | - | - | 222.5 (209.9–223.3) · p95 248.9 |
| wasm | probe.overhead | probe.mark | - | - | 0.1 (0.0–0.1) · p95 0.1 |
| wasm | probe.overhead | js.performance_now | - | - | 0.0 (0.0–0.0) · p95 0.1 |

