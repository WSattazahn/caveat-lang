# Caveat performance baseline: 2026-09-29T03-53-23-652-baseline

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=baseline --repeats=3 --build=never --keep-samples --target=rc4-published=package:C:\Users\walte\caveat-rc4\package-files\package --target=rc4-local=tree:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF-rc4 --target=main-local=tree:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF --target=hist-e6ace96=runtime:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\perf-scratch\scout\hist-e6ace96\pkg-reactive --out=C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\perf-scratch\m\runs\baseline2`

## Method

- Suite `baseline`: 3 repeat(s) of every mode, each in a fresh process; within a repeat every target runs a mode back to back, the first target rotating.
- Per-event modes: 1 untimed warm-up pass(es), then 3 timed pass(es), each episode on a freshly opened session. `published-method` runs 3 round(s) with no warm-up, as the published harness did.
- Load, save and restore: 3 untimed then 30 timed calls; published resume method: 3 run(s).
- Statistic: quantile q(p) = sorted[floor(p·(n−1))] over the pooled samples of a run (the published rule; the median is the lower median). A cell is the median over runs of each run's pooled median, with the lowest and highest run in brackets, then the median over runs of each run's p95.
- Units: microseconds per operation unless the operation says otherwise.

## Targets

| Target | Kind | Revision | Reactive WebAssembly sha256 | Glue sha256 | Kit session.mjs sha256 | Native benchmark |
| --- | --- | --- | --- | --- | --- | --- |
| rc4-published | package | caveat-lang@0.1.0-rc.4; built 92b22ca on x86_64-unknown-linux-gnu | `698a0d0fe99b` (2053369 B) | `9066c9a19dcb` | `b171d128027a` | - |
| rc4-local | tree | v0.1.0-rc.4 (92b22ca) | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |
| main-local | tree | v0.1.0-rc.4-5-gdbfd2e3-dirty (dbfd2e3), tracked changes | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | release, lto, 1 codegen unit, reactive-only |
| hist-e6ace96 | runtime | n/a | `d897787f414a` (1682294 B) | `33a41153ec24` | - | - |

## Environment

- CPU: Intel(R) Core(TM) Ultra 9 275HX; 24 cores, 24 logical processors; performance cores 0, 1, 10, 11, 12, 13, 22, 23
- Memory: 63.4 GiB
- OS: Microsoft Windows 11 Home 10.0.26200 (build 26200)
- Power: Power Scheme GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c  (High performance); battery status 2 (2 = on mains power)
- Pinning: affinity 0x3C00 (logical processors 10, 11, 12, 13), priority high, via cmd /c start /b /wait /<priority> /affinity <mask>
- Node v24.11.1, V8 13.6.233.10-node.28, npm 11.12.1
- Rust: rustc 1.98.1 (48a229cea 2026-09-01) (LLVM version: 22.1.8); cargo 1.98.1 (797e8a9bc 2026-08-05); 1.98.1-x86_64-pc-windows-msvc (overridden by 'C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF-rc4\rust-toolchain.toml')
- wasm-bindgen 0.2.104; wasm-opt not installed (the build does not use it)
- Load before: total CPU 15/8/14%; busiest over the window: claude 1.672s, msedge 1.406s, powershell 0.766s, claude 0.469s, claude 0.25s, esrv 0.234s
- Load after: total CPU 23/18/21%; busiest over the window: node 8.188s, rustc 8.156s, rustc 8.156s, node 8.125s, indexed_names-1cc759968d216353 8s, claude 3.391s
- Load during: typeperf every 5 s for the whole run, 778 samples: total mean 57% p95 100% max 100%; processor 10 mean 79.3% p95 100% max 100%; processor 11 mean 65.1% p95 100% max 100%; processor 12 mean 64.2% p95 100% max 100%; processor 13 mean 69% p95 100% max 100%; total above 10% in 734 and above 25% in 586 samples; pinned processors summing above 150% in 618

## Published reference

- Dispatch + view, Glowcap replay: median 51.6 µs, p95 59.1 µs, max 646.7 µs (experiments/glowcap/evidence/round6-replay.json timing.dispatchAndShippedOutput); runtime 8d8596a, reactive WebAssembly `d897787f414a`, repository e6ace96.
- Resume ten minutes of play: median 3.2416 ms of runs 7.154, 3.2416, 3.1401 ms; save 2045 bytes (experiments/glowcap/evidence/round6-replay.json timing.resume (caveat5)).

## Final-state consistency

Within each target, every run of every mode and engine ended each workload in the same saved state (762 runs compared).

Between targets, by workload and kind of state (targets in one group ended in identical states):

- glowcap-replay save-text: rc4-local, main-local, rc4-published `fda3d6109816` | hist-e6ace96 `0e3d5c34f900`
- glowcap-replay first-episode-save-text: rc4-local, main-local, rc4-published `fda3d6109816` | hist-e6ace96 `0e3d5c34f900`
- glowcap-replay adapter-save-json: rc4-published, rc4-local, main-local `bcc2ec60c6f5` | hist-e6ace96 `6a38724483ec`
- glowcap-resume first-episode-save-text: rc4-local, main-local, rc4-published `bc6187eccca8` | hist-e6ace96 `764075f25768`
- glowcap-resume first-episode-adapter-save-json: rc4-published, rc4-local, main-local `65f36ab1b2e6` | hist-e6ace96 `478bfdcbd337`
- glowcap-unbound save-text: rc4-local, main-local, rc4-published `9271bb69a75b` | hist-e6ace96 `a8313824d79a`
- glowcap-scaled-16 save-text: rc4-local, main-local, rc4-published `e1e084a5061b` | hist-e6ace96 `b81c5e4e62e5`
- glowcap-scaled-16 first-episode-save-text: rc4-local, main-local, rc4-published `e1e084a5061b` | hist-e6ace96 `b81c5e4e62e5`
- glowcap-scaled-64 save-text: rc4-local, main-local, rc4-published `6c273b28e990` | hist-e6ace96 `f1d47d67af19`
- glowcap-scaled-64 first-episode-save-text: rc4-local, main-local, rc4-published `6c273b28e990` | hist-e6ace96 `f1d47d67af19`
- ledger-session save-text: rc4-local, main-local, rc4-published, hist-e6ace96 `895698524f68`
- ledger-session first-episode-save-text: rc4-local, main-local, rc4-published, hist-e6ace96 `895698524f68`
- trail-rescue-scenarios save-text: rc4-local, main-local, rc4-published `bb72a4e76921`
- trail-rescue-scenarios first-episode-save-text: rc4-local, main-local, rc4-published `5e76ed5e8ca7`

Skipped jobs (the target cannot run them):
- 0036-hist-e6ace96-glowcap-replay-wasm-raw.dispatch_outcome: this runtime has no dispatch_outcome
- 0048-hist-e6ace96-glowcap-replay-wasm-abi.dispatch_outcome: this runtime has no dispatch_outcome
- 0182-hist-e6ace96-ledger-session-wasm-raw.dispatch_outcome: this runtime has no dispatch_outcome
- 0194-hist-e6ace96-ledger-session-wasm-abi.dispatch_outcome: this runtime has no dispatch_outcome
- 0240-hist-e6ace96-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0244-hist-e6ace96-trail-rescue-scenarios-wasm-raw.dispatch_outcome: this runtime has no dispatch_outcome
- 0248-hist-e6ace96-trail-rescue-scenarios-wasm-raw.dispatch: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0252-hist-e6ace96-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0256-hist-e6ace96-trail-rescue-scenarios-wasm-abi.dispatch_outcome: this runtime has no dispatch_outcome
- 0260-hist-e6ace96-trail-rescue-scenarios-wasm-read: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0264-hist-e6ace96-trail-rescue-scenarios-wasm-abi.read: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0274-hist-e6ace96-trail-rescue-scenarios-wasm-lifecycle: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0278-hist-e6ace96-trail-rescue-scenarios-wasm-micro: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0313-hist-e6ace96-glowcap-replay-wasm-raw.dispatch_outcome: this runtime has no dispatch_outcome
- 0325-hist-e6ace96-glowcap-replay-wasm-abi.dispatch_outcome: this runtime has no dispatch_outcome
- 0459-hist-e6ace96-ledger-session-wasm-raw.dispatch_outcome: this runtime has no dispatch_outcome
- 0471-hist-e6ace96-ledger-session-wasm-abi.dispatch_outcome: this runtime has no dispatch_outcome
- 0517-hist-e6ace96-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0521-hist-e6ace96-trail-rescue-scenarios-wasm-raw.dispatch_outcome: this runtime has no dispatch_outcome
- 0525-hist-e6ace96-trail-rescue-scenarios-wasm-raw.dispatch: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0529-hist-e6ace96-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0533-hist-e6ace96-trail-rescue-scenarios-wasm-abi.dispatch_outcome: this runtime has no dispatch_outcome
- 0537-hist-e6ace96-trail-rescue-scenarios-wasm-read: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0541-hist-e6ace96-trail-rescue-scenarios-wasm-abi.read: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0551-hist-e6ace96-trail-rescue-scenarios-wasm-lifecycle: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0555-hist-e6ace96-trail-rescue-scenarios-wasm-micro: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0590-hist-e6ace96-glowcap-replay-wasm-raw.dispatch_outcome: this runtime has no dispatch_outcome
- 0602-hist-e6ace96-glowcap-replay-wasm-abi.dispatch_outcome: this runtime has no dispatch_outcome
- 0736-hist-e6ace96-ledger-session-wasm-raw.dispatch_outcome: this runtime has no dispatch_outcome
- 0748-hist-e6ace96-ledger-session-wasm-abi.dispatch_outcome: this runtime has no dispatch_outcome
- 0794-hist-e6ace96-trail-rescue-scenarios-wasm-raw.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0798-hist-e6ace96-trail-rescue-scenarios-wasm-raw.dispatch_outcome: this runtime has no dispatch_outcome
- 0802-hist-e6ace96-trail-rescue-scenarios-wasm-raw.dispatch: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0806-hist-e6ace96-trail-rescue-scenarios-wasm-abi.dispatch_view: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0810-hist-e6ace96-trail-rescue-scenarios-wasm-abi.dispatch_outcome: this runtime has no dispatch_outcome
- 0814-hist-e6ace96-trail-rescue-scenarios-wasm-read: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0818-hist-e6ace96-trail-rescue-scenarios-wasm-abi.read: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0828-hist-e6ace96-trail-rescue-scenarios-wasm-lifecycle: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)
- 0832-hist-e6ace96-trail-rescue-scenarios-wasm-micro: the program does not load on this runtime: line 389, column 1: invalid reactive identifier caveated(plan_basis, stale)

## One Glowcap event + view, piece by piece

Medians over runs of each run's median on the steady idle ticks (events 1300–9999), where the pooled median sits; each row is its own process, so parts need not sum exactly to the whole.

| Piece | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- |
| Published method: adapter dispatch + view | 46.3 | 51.2 | 51.0 | 48.1 |
| Adapter dispatch (stringify, WebAssembly dispatch_view, JSON.parse) | 53.9 | 45.8 | 49.6 | 50.4 |
| Adapter view() (JavaScript reshaping only) | 2.8 | 2.4 | 2.6 | 2.5 |
| JSON.stringify(payload) | 0.3 | 0.3 | 0.4 | 0.3 |
| wasm-bindgen dispatch_view call, returning the view text | 27.4 | 27.5 | 27.6 | 25.4 |
|   arguments copied into WebAssembly memory | 0.1 | 0.1 | 0.1 | 0.2 |
|   WebAssembly execution (resolve, apply, view, serialize) | 24.7 | 23.1 | 23.0 | 28.1 |
|   result decoded to a JavaScript string | 2.8 | 2.6 | 2.6 | 3.3 |
|   result freed | 0.1 | 0.1 | 0.1 | 0.1 |
|   the four pieces, timed together in one process | 27.7 | 26.0 | 25.8 | 31.7 |
| JSON.parse(view text) | 18.1 | 18.3 | 18.2 | 17.2 |
| WebAssembly view() alone, execution only | 9.2 | 9.8 | 9.5 | 9.6 |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | 151.1 | 150.1 | 154.0 | - |
| Kit dispatch + view() | 183.1 | 181.6 | 186.4 | - |
| Native web::dispatch_view (the exported function, natively) | - | 21.7 | 21.8 | - |
| Native apply (numeric parameters: rules and bindings only) | - | 14.6 | 13.7 | - |
| Native dispatch_view_json (resolve + apply + view, not serialized) | - | 17.0 | 21.2 | - |
| Native view() build | - | 3.0 | 2.7 | - |
| Native view serialize | - | 5.6 | 5.0 | - |
| Native snapshot() build | - | 30.5 | 26.3 | - |
| Native session clone (upper bound of the transaction copy) | - | 8.6 | 7.6 | - |
| Native apply, the same program without bindings (glowcap-unbound) | - | 14.8 | 16.1 | - |
| *derived:* native resolve_payload ≈ dispatch_view_json − apply − view | - | -0.6 | 4.8 | - |
| *derived:* native binding evaluation ≈ apply − apply without bindings | - | -0.2 | -2.4 | - |
| *derived:* WebAssembly ÷ native, same exported function (ratio) | - | 1.1 | 1.1 | - |

## glowcap-replay

The Glowcap replay program and the exact event stream behind the published 51.6 us figure (experiments/glowcap/harness.mjs bench(), runtime/examples/profile_dispatch.rs): absorb cave glowcap, absorb pool duskcap, taste ruin glowcap, then 9,997 ticks of dt 0.05. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 14.6 (13.8–17.5) · p95 17.6 | 13.9 (13.7–18.5) · p95 16.8 | - |
| native | dispatch_view_json | dispatch_view_json | - | 17.1 (17.0–23.8) · p95 20.6 | 21.4 (16.6–21.7) · p95 25.3 | - |
| native | dispatch_json | dispatch_json | - | 38.0 (37.5–51.0) · p95 42.3 | 39.0 (38.1–48.4) · p95 47.7 | - |
| native | dispatch_outcome_json | dispatch_outcome_json | - | 46.3 (37.6–51.2) · p95 51.9 | 50.7 (38.0–51.2) · p95 57.4 | - |
| native | web.dispatch_view | web.dispatch_view | - | 21.9 (21.6–28.7) · p95 26.0 | 21.8 (21.5–26.4) · p95 26.0 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 68.5 (67.2–81.8) · p95 82.1 | 77.8 (68.2–87.7) · p95 85.8 | - |
| native | web.dispatch | web.dispatch | - | 84.9 (84.1–98.6) · p95 94.6 | 86.0 (85.9–107.0) · p95 115.0 | - |
| native | read | clone | - | 8.2 (7.5–12.0) · p95 9.9 | 7.6 (7.5–8.0) · p95 8.9 | - |
| native | read | clone.drop | - | 4.5 (4.1–6.7) · p95 5.4 | 4.1 (4.1–4.4) · p95 4.8 | - |
| native | read | save | - | 10.6 (9.4–15.3) · p95 12.6 | 9.5 (9.5–10.4) · p95 11.4 | - |
| native | read | save_json | - | 17.3 (15.3–25.2) · p95 20.4 | 15.6 (15.5–16.9) · p95 18.6 | - |
| native | read | snapshot | - | 29.8 (26.2–42.5) · p95 34.5 | 26.1 (26.1–28.2) · p95 31.6 | - |
| native | read | snapshot.drop | - | 12.8 (11.3–18.6) · p95 14.9 | 11.3 (11.3–12.2) · p95 13.4 | - |
| native | read | snapshot.serialize | - | 18.0 (15.7–25.6) · p95 20.8 | 16.0 (15.9–17.4) · p95 19.4 | - |
| native | read | snapshot.serialize_pretty | - | 38.1 (33.0–54.2) · p95 43.4 | 33.5 (33.5–36.5) · p95 40.8 | - |
| native | read | view | - | 2.8 (2.6–4.2) · p95 3.5 | 2.6 (2.6–2.7) · p95 3.1 | - |
| native | read | view.serialize | - | 5.6 (4.9–7.9) · p95 6.5 | 5.0 (5.0–5.4) · p95 6.0 | - |
| native | web.read | web.save | - | 20.3 (16.7–24.2) · p95 22.7 | 20.9 (18.7–25.3) · p95 24.0 | - |
| native | web.read | web.snapshot | - | 86.2 (70.6–102.6) · p95 95.4 | 88.2 (79.0–106.3) · p95 100.8 | - |
| native | web.read | web.view | - | 8.3 (6.8–9.9) · p95 9.3 | 8.4 (7.6–10.3) · p95 9.7 | - |
| native | lifecycle | from_source | - | 2722.3 (2638.5–3994.6) · p95 2812.2 | 3468.2 (2740.8–4149.6) · p95 3892.9 | - |
| native | lifecycle | restore | - | 2826.5 (2772.2–4230.7) · p95 2994.6 | 3668.3 (2862.2–4280.6) · p95 4130.7 | - |
| native | lifecycle | restore.parse | - | 31.3 (25.2–51.3) · p95 34.7 | 42.8 (27.5–48.9) · p95 47.7 | - |
| native | lifecycle | restore_json | - | 2937.6 (2849.3–4347.7) · p95 3081.5 | 3755.8 (2911.7–4383.7) · p95 4216.5 | - |
| native | lifecycle | save | - | 28.2 (22.0–46.4) · p95 33.1 | 39.5 (22.6–47.4) · p95 47.1 | - |
| native | lifecycle | save_json | - | 28.1 (24.6–37.2) · p95 44.2 | 38.4 (33.4–40.9) · p95 58.4 | - |
| native | lifecycle | web.new | - | 2740.1 (2600.4–3978.8) · p95 2919.4 | 3524.8 (2737.7–4036.5) · p95 3923.5 | - |
| native | lifecycle | web.restore | - | 2906.7 (2856.3–4300.8) · p95 3112.5 | 3789.0 (2944.1–4378.2) · p95 4183.1 | - |
| native | lifecycle | web.save | - | 24.3 (18.9–33.7) · p95 32.1 | 31.3 (20.2–34.9) · p95 37.0 | - |
| wasm | published-method | adapter.dispatch+view | 46.5 (43.7–62.6) · p95 59.6 | 51.4 (43.7–62.1) · p95 67.9 | 51.4 (44.0–62.9) · p95 66.2 | 48.1 (42.9–61.8) · p95 60.4 |
| wasm | adapter | adapter.dispatch | 54.1 (41.6–59.7) · p95 70.4 | 46.0 (42.0–59.5) · p95 58.6 | 49.6 (41.3–59.3) · p95 69.5 | 50.4 (39.8–57.8) · p95 62.4 |
| wasm | adapter | adapter.view | 2.8 (2.3–2.8) · p95 3.8 | 2.4 (2.2–2.8) · p95 3.7 | 2.6 (2.2–2.8) · p95 3.7 | 2.5 (2.2–2.8) · p95 3.9 |
| wasm | adapter | adapter.dispatch+view | 57.1 (43.8–62.6) · p95 74.1 | 48.5 (44.2–62.4) · p95 61.6 | 52.3 (43.5–62.1) · p95 73.4 | 53.1 (42.0–60.7) · p95 66.0 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.4) · p95 0.6 | 0.3 (0.3–0.4) · p95 0.6 | 0.4 (0.3–0.4) · p95 0.6 | 0.3 (0.3–0.4) · p95 0.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 27.6 (24.7–33.9) · p95 34.3 | 27.7 (25.1–31.0) · p95 36.2 | 27.9 (25.0–35.5) · p95 34.3 | 25.6 (23.5–32.4) · p95 31.2 |
| wasm | raw.dispatch_view | js.parse_view | 18.1 (16.3–22.2) · p95 21.6 | 18.3 (16.3–20.2) · p95 23.0 | 18.1 (16.3–23.1) · p95 22.7 | 17.1 (15.9–21.3) · p95 20.5 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 46.2 (41.4–56.8) · p95 55.8 | 46.4 (41.8–51.7) · p95 59.2 | 46.4 (41.7–59.3) · p95 57.4 | 43.1 (39.9–54.1) · p95 51.6 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 80.3 (77.0–86.0) · p95 99.6 | 79.6 (73.3–91.5) · p95 101.1 | 76.2 (75.8–88.4) · p95 95.4 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 71.2 (68.1–73.9) · p95 84.2 | 69.6 (63.5–81.2) · p95 85.7 | 66.5 (66.3–77.0) · p95 81.0 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 151.9 (145.7–160.6) · p95 184.2 | 149.5 (137.2–173.0) · p95 186.9 | 143.1 (142.5–166.1) · p95 175.3 | - |
| wasm | raw.dispatch | raw.dispatch | 108.6 (102.5–144.4) · p95 141.7 | 104.3 (99.3–138.6) · p95 127.2 | 116.6 (110.0–123.7) · p95 154.6 | 107.7 (107.5–129.3) · p95 129.1 |
| wasm | abi.dispatch_view | abi.encode_args | 0.1 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.3 | 0.2 (0.1–0.2) · p95 0.4 |
| wasm | abi.dispatch_view | abi.exec | 24.8 (22.4–29.3) · p95 33.2 | 23.4 (22.4–29.6) · p95 30.6 | 23.3 (22.1–30.8) · p95 29.4 | 28.2 (21.5–28.3) · p95 35.1 |
| wasm | abi.dispatch_view | abi.decode | 2.8 (2.5–3.3) · p95 3.8 | 2.7 (2.4–3.4) · p95 3.5 | 2.6 (2.5–3.3) · p95 3.3 | 3.3 (2.5–3.3) · p95 4.0 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | abi.dispatch_view | abi.dispatch_view | 27.9 (25.2–32.9) · p95 37.2 | 26.3 (25.1–33.3) · p95 34.4 | 26.2 (24.8–34.4) · p95 33.0 | 31.8 (24.3–31.9) · p95 39.6 |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.3 | - |
| wasm | abi.dispatch_outcome | abi.exec | 72.1 (62.2–80.3) · p95 89.5 | 72.2 (61.0–78.1) · p95 89.3 | 59.4 (59.1–81.2) · p95 71.7 | - |
| wasm | abi.dispatch_outcome | abi.decode | 9.2 (8.1–10.2) · p95 12.4 | 9.1 (7.9–10.2) · p95 12.4 | 7.7 (7.7–10.1) · p95 10.5 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 81.9 (71.0–91.2) · p95 104.0 | 81.9 (69.6–89.1) · p95 104.1 | 67.6 (67.4–91.8) · p95 82.3 | - |
| wasm | read | raw.view | 13.8 (11.6–15.9) · p95 17.1 | 13.2 (11.8–15.2) · p95 17.1 | 11.8 (11.5–13.3) · p95 14.9 | 12.4 (10.7–13.1) · p95 18.0 |
| wasm | read | js.parse_view | 22.5 (18.7–26.3) · p95 27.9 | 22.1 (19.2–25.1) · p95 28.7 | 19.8 (19.2–21.9) · p95 25.3 | 21.8 (18.9–23.1) · p95 31.5 |
| wasm | read | raw.snapshot | 113.5 (96.0–130.4) · p95 142.2 | 109.2 (97.0–126.7) · p95 142.2 | 97.4 (94.8–112.0) · p95 126.0 | 112.6 (97.4–120.1) · p95 162.5 |
| wasm | read | js.parse_snapshot | 85.4 (71.4–99.9) · p95 105.1 | 85.0 (73.4–96.3) · p95 108.1 | 74.8 (72.6–84.3) · p95 94.2 | 84.5 (72.8–89.4) · p95 120.4 |
| wasm | read | raw.save | 23.5 (20.0–27.0) · p95 31.6 | 22.6 (19.8–26.1) · p95 31.3 | 20.2 (19.7–23.0) · p95 27.0 | 22.9 (19.7–24.3) · p95 33.4 |
| wasm | abi.read | abi.view.exec | 9.1 (8.6–12.0) · p95 10.4 | 9.8 (8.5–11.8) · p95 11.7 | 9.5 (8.8–11.7) · p95 12.3 | 9.4 (8.0–10.3) · p95 12.3 |
| wasm | abi.read | abi.view.decode | 2.5 (2.3–3.3) · p95 2.8 | 2.7 (2.3–3.2) · p95 3.1 | 2.5 (2.4–3.3) · p95 3.3 | 2.8 (2.4–3.1) · p95 3.6 |
| wasm | abi.read | abi.snapshot.exec | 77.1 (71.7–100.7) · p95 90.4 | 82.7 (72.3–100.0) · p95 99.2 | 79.9 (74.5–98.5) · p95 102.2 | 93.9 (75.5–98.7) · p95 118.3 |
| wasm | abi.read | abi.snapshot.decode | 13.1 (12.3–17.5) · p95 16.5 | 14.1 (12.5–17.5) · p95 17.1 | 13.5 (12.7–18.5) · p95 18.6 | 14.8 (12.6–17.4) · p95 20.4 |
| wasm | abi.read | abi.save.exec | 16.7 (15.6–21.5) · p95 20.3 | 17.8 (15.6–21.6) · p95 22.7 | 17.5 (15.8–21.1) · p95 23.0 | 19.6 (16.0–21.0) · p95 26.1 |
| wasm | abi.read | abi.save.decode | 0.7 (0.6–0.8) · p95 0.9 | 0.7 (0.6–0.8) · p95 1.0 | 0.7 (0.6–0.8) · p95 1.0 | 0.8 (0.7–0.9) · p95 1.2 |
| wasm | kit | kit.dispatch | 149.2 (142.3–195.1) · p95 187.1 | 149.9 (140.8–197.6) · p95 184.8 | 152.9 (140.7–195.1) · p95 177.6 | - |
| wasm | kit | kit.view | 31.2 (29.5–40.1) · p95 38.9 | 30.9 (29.0–40.8) · p95 38.5 | 31.8 (28.9–40.4) · p95 37.2 | - |
| wasm | kit | kit.dispatch+view | 180.6 (172.0–235.9) · p95 225.6 | 181.2 (170.0–239.3) · p95 222.9 | 185.0 (169.9–236.1) · p95 214.3 | - |
| wasm | kit.read | kit.view | 35.4 (31.7–47.9) · p95 46.3 | 36.8 (32.2–45.7) · p95 45.3 | 35.2 (32.9–41.7) · p95 44.4 | - |
| wasm | kit.read | kit.snapshot | 193.3 (174.5–266.9) · p95 267.3 | 202.3 (176.7–250.7) · p95 254.8 | 193.8 (179.9–231.1) · p95 256.0 | - |
| wasm | kit.read | kit.save | 46.3 (41.7–63.5) · p95 59.9 | 48.1 (42.4–60.3) · p95 60.5 | 46.4 (43.6–55.5) · p95 58.9 | - |
| wasm | lifecycle | raw.new | 2763.0 (2568.8–3686.5) · p95 2995.1 | 2722.7 (2463.5–3680.0) · p95 3034.4 | 2873.7 (2606.5–3667.6) · p95 3237.4 | 2486.5 (2351.6–3723.0) · p95 3147.0 |
| wasm | lifecycle | raw.save | 50.7 (49.2–82.4) · p95 65.4 | 49.6 (45.3–80.1) · p95 60.3 | 57.6 (47.2–81.8) · p95 72.3 | 43.9 (42.2–78.5) · p95 66.1 |
| wasm | lifecycle | raw.restore | 2845.0 (2616.8–3769.1) · p95 2974.4 | 2790.6 (2546.5–3761.5) · p95 3084.6 | 2973.5 (2706.7–3790.2) · p95 3434.4 | 2673.9 (2453.8–3859.5) · p95 3363.2 |
| wasm | lifecycle | kit.open | 2691.0 (2476.3–3574.9) · p95 2850.8 | 2674.7 (2380.5–3508.7) · p95 2860.6 | 2774.2 (2461.4–3519.8) · p95 2932.7 | - |
| wasm | lifecycle | kit.save | 116.7 (92.6–174.0) · p95 154.8 | 99.5 (86.8–170.6) · p95 132.8 | 125.9 (96.2–163.8) · p95 150.4 | - |
| wasm | lifecycle | kit.restore | 2787.7 (2596.6–3774.3) · p95 2922.5 | 2784.8 (2489.2–3719.4) · p95 3091.5 | 2942.8 (2618.8–3721.5) · p95 3174.9 | - |
| wasm | micro | timer.hrtime_pair | 0.2 (0.1–0.2) · p95 0.2 | 0.1 (0.1–0.2) · p95 0.2 | 0.1 (0.1–0.2) · p95 0.2 | 0.1 (0.1–0.2) · p95 0.2 |
| wasm | micro | js.stringify_payload.first | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | micro | js.stringify_payload.last | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | micro | bridge.pass_ascii.payload | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | micro | kit.payloadText.last | 0.2 (0.2–0.2) · p95 0.2 | 0.2 (0.2–0.2) · p95 0.3 | 0.2 (0.2–0.2) · p95 0.3 | - |
| wasm | micro | bridge.decode.view | 2.2 (2.1–3.0) · p95 2.4 | 3.0 (1.8–3.2) · p95 3.7 | 2.2 (2.1–3.1) · p95 2.5 | 2.4 (2.1–3.1) · p95 5.6 |
| wasm | micro | bridge.pass_ascii.view | 4.6 (4.3–5.7) · p95 6.5 | 5.6 (3.8–5.7) · p95 6.1 | 4.7 (3.8–5.9) · p95 5.0 | 4.2 (4.0–5.8) · p95 4.8 |
| wasm | micro | js.parse.view | 21.2 (16.9–22.6) · p95 23.5 | 19.2 (15.0–21.5) · p95 25.1 | 18.8 (15.5–22.2) · p95 21.2 | 15.7 (15.6–22.0) · p95 18.4 |
| wasm | micro | bridge.decode.outcome | 7.6 (6.4–9.5) · p95 9.4 | 6.4 (6.3–10.3) · p95 7.6 | 7.3 (6.4–9.8) · p95 7.9 | - |
| wasm | micro | bridge.pass_ascii.outcome | 23.5 (18.8–24.0) · p95 25.6 | 19.1 (17.6–29.8) · p95 20.6 | 21.8 (17.6–26.2) · p95 22.9 | - |
| wasm | micro | js.parse.outcome | 65.8 (62.8–81.2) · p95 89.9 | 70.6 (60.8–99.5) · p95 82.9 | 73.5 (60.7–84.0) · p95 78.9 | - |
| wasm | micro | bridge.decode.snapshot | 10.7 (10.5–15.7) · p95 11.5 | 10.6 (10.5–16.1) · p95 11.7 | 12.3 (11.2–16.6) · p95 13.2 | 10.4 (9.8–16.3) · p95 11.0 |
| wasm | micro | bridge.pass_ascii.snapshot | 30.4 (30.2–40.8) · p95 32.6 | 30.4 (30.0–46.1) · p95 33.3 | 35.8 (31.4–40.9) · p95 38.4 | 33.7 (32.2–43.2) · p95 35.7 |
| wasm | micro | js.parse.snapshot | 69.9 (68.8–87.0) · p95 78.5 | 69.8 (64.5–102.9) · p95 80.9 | 65.9 (65.4–90.7) · p95 80.3 | 73.1 (71.7–88.3) · p95 78.4 |
| wasm | micro | bridge.decode.save | 0.5 (0.5–0.5) · p95 0.6 | 0.4 (0.4–0.6) · p95 0.6 | 0.5 (0.4–0.6) · p95 0.6 | 0.5 (0.5–0.5) · p95 0.7 |
| wasm | micro | bridge.pass_ascii.save | 4.3 (4.1–5.3) · p95 4.6 | 3.9 (3.9–6.2) · p95 4.1 | 3.9 (3.8–5.6) · p95 4.1 | 4.4 (4.3–5.3) · p95 4.8 |
| wasm | micro | js.parse.save | 14.7 (13.7–17.4) · p95 15.7 | 14.6 (12.9–19.9) · p95 16.7 | 13.4 (12.8–18.0) · p95 15.7 | 14.3 (14.1–17.6) · p95 15.4 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | rc4-local | 86.1 (83.8–112.6) | 18.2 (16.8–22.3) | 14.9 (14.1–17.8) | 14.6 (13.7–17.5) |
| native | apply | apply | main-local | 88.0 (85.5–116.2) | 17.4 (16.9–22.8) | 14.4 (14.0–18.9) | 13.7 (13.7–18.5) |
| native | dispatch_view_json | dispatch_view_json | rc4-local | 95.9 (93.9–131.2) | 19.6 (19.5–27.4) | 17.3 (17.3–24.0) | 17.0 (16.9–23.7) |
| native | dispatch_view_json | dispatch_view_json | main-local | 114.3 (100.4–129.2) | 25.4 (19.9–25.9) | 21.9 (17.1–22.0) | 21.2 (16.5–21.6) |
| native | dispatch_json | dispatch_json | rc4-local | 128.7 (125.7–180.2) | 40.1 (39.1–51.1) | 37.6 (36.3–49.5) | 38.0 (37.5–51.1) |
| native | dispatch_json | dispatch_json | main-local | 129.4 (128.3–170.2) | 41.2 (39.0–51.0) | 38.7 (36.3–46.5) | 38.9 (38.1–48.4) |
| native | dispatch_outcome_json | dispatch_outcome_json | rc4-local | 143.2 (129.9–172.1) | 48.1 (38.3–53.5) | 45.2 (36.1–50.2) | 46.3 (37.6–51.1) |
| native | dispatch_outcome_json | dispatch_outcome_json | main-local | 162.8 (129.8–190.2) | 50.0 (38.7–53.6) | 47.1 (36.8–49.6) | 51.2 (38.0–51.2) |
| native | web.dispatch_view | web.dispatch_view | rc4-local | 113.8 (112.1–146.3) | 25.2 (24.2–32.1) | 22.6 (21.4–29.2) | 21.7 (21.5–28.5) |
| native | web.dispatch_view | web.dispatch_view | main-local | 116.3 (115.0–155.7) | 24.0 (23.8–29.9) | 21.3 (21.2–26.5) | 21.8 (21.4–26.4) |
| native | web.dispatch_outcome | web.dispatch_outcome | rc4-local | 167.5 (153.1–189.7) | 68.8 (64.7–79.5) | 66.7 (64.1–78.7) | 68.6 (67.5–82.0) |
| native | web.dispatch_outcome | web.dispatch_outcome | main-local | 171.2 (156.1–208.3) | 74.1 (66.1–85.2) | 73.2 (65.7–85.0) | 78.2 (68.3–88.1) |
| native | web.dispatch | web.dispatch | rc4-local | 192.4 (167.5–201.4) | 82.6 (81.5–93.8) | 82.4 (81.7–94.2) | 85.0 (84.4–98.8) |
| native | web.dispatch | web.dispatch | main-local | 208.0 (184.8–220.9) | 84.2 (83.2–102.2) | 82.8 (82.5–102.6) | 86.3 (86.0–107.4) |
| native | read | clone | rc4-local | 8.4 (7.2–12.0) | 7.7 (6.4–10.5) | 7.8 (6.8–11.1) | 8.6 (7.5–12.2) |
| native | read | clone | main-local | 7.4 (7.3–8.0) | 6.6 (6.4–7.1) | 6.9 (6.5–7.2) | 7.6 (7.5–8.1) |
| native | read | clone.drop | rc4-local | 4.6 (3.9–6.6) | 4.3 (3.6–5.9) | 4.3 (3.7–6.2) | 4.7 (4.1–6.7) |
| native | read | clone.drop | main-local | 4.1 (4.0–4.3) | 3.7 (3.6–3.9) | 3.8 (3.7–4.0) | 4.1 (4.1–4.4) |
| native | read | save | rc4-local | 7.3 (6.1–10.2) | 8.1 (6.7–10.9) | 9.1 (9.0–14.3) | 11.0 (9.5–15.5) |
| native | read | save | main-local | 6.2 (6.2–7.2) | 6.9 (6.6–7.4) | 8.9 (8.9–9.9) | 9.7 (9.6–10.4) |
| native | read | save_json | rc4-local | 12.0 (10.3–17.1) | 13.5 (11.3–18.6) | 14.9 (14.9–23.7) | 17.9 (15.4–25.4) |
| native | read | save_json | main-local | 10.9 (10.1–11.3) | 11.6 (11.1–12.3) | 15.0 (14.6–16.3) | 15.7 (15.6–17.0) |
| native | read | snapshot | rc4-local | 40.9 (36.4–51.0) | 28.0 (23.3–37.7) | 28.5 (25.1–40.8) | 30.5 (26.3–42.9) |
| native | read | snapshot | main-local | 40.1 (29.9–48.5) | 23.5 (22.5–25.1) | 25.2 (24.8–26.7) | 26.3 (26.2–28.4) |
| native | read | snapshot.drop | rc4-local | 12.8 (11.0–19.5) | 11.9 (9.9–16.3) | 12.2 (10.7–17.7) | 13.1 (11.3–18.8) |
| native | read | snapshot.drop | main-local | 11.2 (11.1–12.6) | 10.0 (9.7–10.8) | 10.9 (10.7–11.5) | 11.3 (11.3–12.2) |
| native | read | snapshot.serialize | rc4-local | 21.6 (17.6–30.5) | 17.2 (14.3–23.4) | 17.4 (15.5–25.2) | 18.2 (15.7–25.8) |
| native | read | snapshot.serialize | main-local | 20.0 (19.8–20.2) | 14.7 (14.2–15.9) | 15.8 (15.6–16.9) | 16.1 (15.9–17.5) |
| native | read | snapshot.serialize_pretty | rc4-local | 43.5 (35.0–58.4) | 36.5 (30.4–49.9) | 37.3 (32.6–53.4) | 38.5 (33.1–54.5) |
| native | read | snapshot.serialize_pretty | main-local | 41.0 (39.0–42.9) | 31.0 (30.1–33.6) | 33.2 (32.6–35.4) | 33.7 (33.5–36.7) |
| native | read | view | rc4-local | 2.7 (2.2–3.8) | 2.4 (2.0–3.4) | 2.5 (2.5–3.9) | 3.0 (2.6–4.3) |
| native | read | view | main-local | 2.4 (2.3–2.7) | 2.1 (2.0–2.2) | 2.4 (2.4–2.6) | 2.7 (2.6–2.8) |
| native | read | view.serialize | rc4-local | 7.9 (6.4–11.2) | 5.5 (4.6–7.6) | 5.5 (4.8–7.8) | 5.6 (4.9–7.9) |
| native | read | view.serialize | main-local | 7.8 (7.0–7.9) | 4.7 (4.6–5.1) | 4.9 (4.8–5.2) | 5.0 (5.0–5.5) |
| native | web.read | web.save | rc4-local | 14.4 (12.5–19.6) | 15.5 (12.6–18.0) | 18.2 (15.8–23.5) | 20.5 (16.8–24.3) |
| native | web.read | web.save | main-local | 14.9 (13.5–18.5) | 15.5 (13.6–18.6) | 19.7 (17.3–24.0) | 21.0 (19.0–25.4) |
| native | web.read | web.snapshot | rc4-local | 94.3 (86.9–120.0) | 79.0 (63.3–91.3) | 80.9 (67.2–98.5) | 87.2 (70.9–102.7) |
| native | web.read | web.snapshot | main-local | 112.7 (86.7–118.7) | 78.5 (68.6–94.4) | 82.5 (74.8–101.1) | 88.7 (80.4–106.8) |
| native | web.read | web.view | rc4-local | 8.7 (7.0–10.2) | 7.2 (5.8–8.3) | 7.6 (6.3–9.2) | 8.4 (6.9–9.9) |
| native | web.read | web.view | main-local | 8.5 (7.6–10.0) | 7.1 (6.2–8.6) | 7.7 (7.0–9.5) | 8.5 (7.7–10.3) |
| wasm | published-method | adapter.dispatch+view | rc4-published | 188.6 (169.8–314.5) | 55.6 (46.9–70.6) | 47.5 (43.6–63.9) | 46.3 (43.5–62.4) |
| wasm | published-method | adapter.dispatch+view | rc4-local | 192.0 (189.3–266.1) | 63.3 (46.7–68.4) | 48.7 (43.4–62.9) | 51.2 (43.6–62.0) |
| wasm | published-method | adapter.dispatch+view | main-local | 183.5 (179.7–301.3) | 58.7 (51.4–68.6) | 51.3 (43.7–64.2) | 51.0 (43.9–62.6) |
| wasm | published-method | adapter.dispatch+view | hist-e6ace96 | 210.0 (208.0–312.9) | 54.6 (48.5–68.5) | 45.7 (42.9–64.7) | 48.1 (42.7–61.6) |
| wasm | adapter | adapter.dispatch | rc4-published | 200.5 (167.8–261.0) | 59.3 (49.8–63.0) | 53.6 (40.7–59.5) | 53.9 (41.4–59.6) |
| wasm | adapter | adapter.dispatch | rc4-local | 170.7 (168.4–265.0) | 51.4 (42.1–63.4) | 48.4 (40.5–59.1) | 45.8 (42.1–59.4) |
| wasm | adapter | adapter.dispatch | main-local | 193.7 (162.3–253.7) | 52.1 (43.7–62.7) | 48.6 (41.6–58.4) | 49.6 (41.1–59.2) |
| wasm | adapter | adapter.dispatch | hist-e6ace96 | 173.9 (173.3–287.8) | 52.8 (42.7–64.6) | 47.5 (39.1–56.5) | 50.4 (39.7–57.7) |
| wasm | adapter | adapter.view | rc4-published | 39.8 (37.5–46.9) | 3.3 (2.7–3.3) | 2.9 (2.3–2.9) | 2.8 (2.3–2.8) |
| wasm | adapter | adapter.view | rc4-local | 29.1 (25.3–57.1) | 3.0 (2.4–3.2) | 2.5 (2.2–2.9) | 2.4 (2.2–2.8) |
| wasm | adapter | adapter.view | main-local | 54.8 (35.4–56.2) | 3.0 (2.6–3.5) | 2.6 (2.3–2.8) | 2.6 (2.1–2.8) |
| wasm | adapter | adapter.view | hist-e6ace96 | 31.2 (23.8–37.2) | 3.0 (2.5–3.4) | 2.5 (2.2–2.8) | 2.5 (2.1–2.8) |
| wasm | adapter | adapter.dispatch+view | rc4-published | 242.5 (223.2–300.5) | 63.3 (52.9–66.5) | 56.6 (43.0–62.5) | 56.9 (43.6–62.4) |
| wasm | adapter | adapter.dispatch+view | rc4-local | 207.6 (190.3–322.1) | 54.9 (44.7–66.8) | 50.9 (42.7–62.0) | 48.2 (44.3–62.3) |
| wasm | adapter | adapter.dispatch+view | main-local | 201.3 (195.4–287.4) | 55.6 (46.6–66.5) | 51.4 (43.8–61.3) | 52.2 (43.3–62.0) |
| wasm | adapter | adapter.dispatch+view | hist-e6ace96 | 201.6 (192.5–325.3) | 55.9 (45.7–68.0) | 50.2 (41.3–59.3) | 53.1 (41.9–60.5) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-published | 1.0 (0.8–1.5) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-local | 0.8 (0.8–1.3) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local | 1.7 (1.0–4.0) | 0.3 (0.3–0.5) | 0.3 (0.3–0.4) | 0.4 (0.3–0.4) |
| wasm | raw.dispatch_view | js.stringify_payload | hist-e6ace96 | 0.9 (0.9–2.7) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-published | 139.8 (120.7–208.7) | 33.2 (26.8–36.9) | 28.2 (24.4–32.9) | 27.4 (24.6–33.8) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-local | 139.9 (129.9–167.2) | 35.5 (28.4–36.9) | 27.6 (24.6–32.7) | 27.5 (25.0–30.9) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local | 137.3 (120.8–246.4) | 30.3 (27.2–40.6) | 28.1 (24.9–36.0) | 27.6 (24.8–35.4) |
| wasm | raw.dispatch_view | raw.dispatch_view | hist-e6ace96 | 145.3 (144.8–208.9) | 29.2 (26.8–36.2) | 25.7 (24.2–32.2) | 25.4 (23.4–32.3) |
| wasm | raw.dispatch_view | js.parse_view | rc4-published | 30.6 (28.8–42.8) | 18.8 (15.2–20.8) | 17.6 (15.4–20.8) | 18.1 (16.4–22.5) |
| wasm | raw.dispatch_view | js.parse_view | rc4-local | 34.5 (30.6–41.5) | 20.1 (15.5–20.3) | 17.3 (15.3–20.1) | 18.3 (16.4–20.2) |
| wasm | raw.dispatch_view | js.parse_view | main-local | 30.1 (27.6–49.1) | 17.1 (15.3–22.9) | 17.9 (15.5–22.7) | 18.2 (16.4–23.2) |
| wasm | raw.dispatch_view | js.parse_view | hist-e6ace96 | 32.8 (31.3–43.2) | 16.8 (15.6–20.3) | 16.7 (15.6–20.5) | 17.2 (15.9–21.4) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-published | 172.8 (148.4–265.1) | 52.6 (42.4–58.1) | 46.3 (40.1–54.1) | 46.0 (41.4–56.9) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-local | 176.8 (171.6–226.3) | 56.4 (44.5–57.5) | 45.4 (40.3–53.3) | 46.3 (41.7–51.4) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local | 171.0 (143.9–300.4) | 47.7 (42.7–64.1) | 46.5 (40.9–59.3) | 46.3 (41.7–59.1) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | hist-e6ace96 | 199.1 (177.7–273.0) | 46.4 (42.7–57.0) | 42.8 (40.1–53.1) | 43.0 (39.7–54.1) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | rc4-published | 210.6 (184.9–231.4) | 77.0 (71.2–81.5) | 77.2 (74.2–87.1) | 80.6 (77.4–86.2) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | rc4-local | 217.5 (182.7–262.9) | 77.8 (68.1–103.5) | 77.6 (67.9–93.8) | 79.8 (74.4–90.6) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | main-local | 206.4 (188.9–240.7) | 75.4 (70.3–87.9) | 75.8 (71.5–87.9) | 76.8 (76.3–88.4) |
| wasm | raw.dispatch_outcome | js.parse_outcome | rc4-published | 85.3 (80.2–93.7) | 65.6 (58.2–66.6) | 68.3 (63.5–73.0) | 71.7 (68.7–74.4) |
| wasm | raw.dispatch_outcome | js.parse_outcome | rc4-local | 82.1 (77.7–148.2) | 65.2 (56.3–87.8) | 67.2 (59.5–80.4) | 70.0 (64.1–81.0) |
| wasm | raw.dispatch_outcome | js.parse_outcome | main-local | 84.1 (80.1–103.7) | 61.8 (58.1–72.1) | 65.0 (61.7–75.9) | 67.9 (66.8–77.2) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | rc4-published | 320.4 (271.0–331.0) | 143.1 (130.2–148.6) | 146.1 (138.0–160.7) | 152.7 (146.6–161.2) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | rc4-local | 301.3 (289.6–441.2) | 143.3 (125.2–191.6) | 145.1 (127.9–175.3) | 150.2 (139.0–171.9) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | main-local | 311.9 (265.2–344.7) | 137.6 (129.2–161.0) | 141.2 (133.7–164.8) | 145.1 (143.4–166.4) |
| wasm | raw.dispatch | raw.dispatch | rc4-published | 239.4 (211.2–309.2) | 103.8 (98.3–131.6) | 108.1 (98.7–140.3) | 108.9 (102.9–145.2) |
| wasm | raw.dispatch | raw.dispatch | rc4-local | 238.7 (223.8–309.9) | 97.1 (91.3–126.7) | 103.7 (95.2–129.2) | 104.5 (99.8–139.6) |
| wasm | raw.dispatch | raw.dispatch | main-local | 230.1 (188.0–287.0) | 104.0 (96.7–115.9) | 113.2 (100.9–114.2) | 116.8 (110.8–124.8) |
| wasm | raw.dispatch | raw.dispatch | hist-e6ace96 | 238.0 (215.5–332.6) | 97.5 (96.6–118.1) | 99.6 (96.0–119.7) | 109.2 (108.7–129.7) |
| wasm | abi.dispatch_view | abi.encode_args | rc4-published | 1.0 (0.9–1.5) | 0.3 (0.2–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.encode_args | rc4-local | 1.0 (0.9–1.2) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.encode_args | main-local | 1.2 (0.9–1.3) | 0.3 (0.3–0.3) | 0.1 (0.1–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.encode_args | hist-e6ace96 | 1.5 (1.1–1.6) | 0.3 (0.3–0.4) | 0.2 (0.1–0.2) | 0.2 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.exec | rc4-published | 153.6 (121.0–172.1) | 25.6 (25.6–33.6) | 25.5 (22.1–30.3) | 24.7 (22.3–29.3) |
| wasm | abi.dispatch_view | abi.exec | rc4-local | 132.1 (119.6–186.1) | 28.3 (25.5–36.5) | 23.8 (23.4–32.8) | 23.1 (22.3–29.5) |
| wasm | abi.dispatch_view | abi.exec | main-local | 147.2 (139.4–172.9) | 28.5 (24.3–34.9) | 24.4 (22.6–31.2) | 23.0 (22.0–30.6) |
| wasm | abi.dispatch_view | abi.exec | hist-e6ace96 | 145.0 (131.0–181.5) | 31.9 (28.3–33.0) | 28.0 (23.0–29.3) | 28.1 (21.4–28.2) |
| wasm | abi.dispatch_view | abi.decode | rc4-published | 5.0 (4.7–5.8) | 2.6 (2.5–3.3) | 2.8 (2.4–3.3) | 2.8 (2.5–3.3) |
| wasm | abi.dispatch_view | abi.decode | rc4-local | 4.0 (3.9–6.3) | 2.8 (2.5–3.6) | 2.6 (2.5–3.5) | 2.6 (2.4–3.3) |
| wasm | abi.dispatch_view | abi.decode | main-local | 5.5 (4.1–5.9) | 2.8 (2.4–3.3) | 2.6 (2.4–3.2) | 2.6 (2.5–3.3) |
| wasm | abi.dispatch_view | abi.decode | hist-e6ace96 | 5.3 (3.9–5.9) | 3.2 (2.8–3.4) | 3.1 (2.6–3.2) | 3.3 (2.5–3.3) |
| wasm | abi.dispatch_view | abi.free | rc4-published | 0.3 (0.2–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | rc4-local | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | main-local | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | hist-e6ace96 | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.dispatch_view | rc4-published | 169.2 (127.4–179.8) | 28.6 (28.4–37.4) | 28.6 (24.8–34.0) | 27.7 (25.0–32.8) |
| wasm | abi.dispatch_view | abi.dispatch_view | rc4-local | 137.3 (123.7–194.2) | 31.6 (28.4–40.5) | 26.7 (26.2–36.7) | 26.0 (25.0–33.2) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local | 156.9 (148.5–180.3) | 31.6 (27.0–38.8) | 27.3 (25.3–34.9) | 25.8 (24.7–34.3) |
| wasm | abi.dispatch_view | abi.dispatch_view | hist-e6ace96 | 151.6 (136.2–186.7) | 35.8 (31.5–36.8) | 31.5 (25.9–32.9) | 31.7 (24.2–31.8) |
| wasm | abi.dispatch_outcome | abi.encode_args | rc4-published | 1.2 (0.8–1.6) | 0.3 (0.3–0.4) | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) |
| wasm | abi.dispatch_outcome | abi.encode_args | rc4-local | 1.3 (1.1–1.4) | 0.4 (0.3–0.4) | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) |
| wasm | abi.dispatch_outcome | abi.encode_args | main-local | 1.1 (1.0–1.5) | 0.3 (0.3–0.4) | 0.2 (0.2–0.2) | 0.2 (0.1–0.2) |
| wasm | abi.dispatch_outcome | abi.exec | rc4-published | 194.1 (172.3–233.0) | 63.0 (62.0–78.7) | 61.4 (61.2–76.9) | 72.6 (62.2–80.6) |
| wasm | abi.dispatch_outcome | abi.exec | rc4-local | 186.4 (174.1–229.9) | 70.5 (62.7–79.2) | 65.7 (58.1–79.0) | 73.0 (61.2–78.1) |
| wasm | abi.dispatch_outcome | abi.exec | main-local | 174.8 (171.0–239.7) | 60.5 (59.2–81.5) | 64.4 (59.1–79.0) | 59.3 (59.1–81.3) |
| wasm | abi.dispatch_outcome | abi.decode | rc4-published | 8.9 (8.6–12.0) | 7.6 (7.0–9.2) | 7.7 (7.6–9.6) | 9.3 (8.1–10.3) |
| wasm | abi.dispatch_outcome | abi.decode | rc4-local | 9.2 (8.5–10.5) | 8.1 (7.3–9.1) | 8.0 (7.3–9.7) | 9.3 (8.0–10.3) |
| wasm | abi.dispatch_outcome | abi.decode | main-local | 8.1 (7.8–10.7) | 7.1 (6.8–9.3) | 7.6 (7.4–9.6) | 7.7 (7.7–10.1) |
| wasm | abi.dispatch_outcome | abi.free | rc4-published | 0.4 (0.2–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.free | rc4-local | 0.3 (0.3–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.free | main-local | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | rc4-published | 203.9 (181.7–247.1) | 71.3 (69.3–88.8) | 69.7 (69.1–87.3) | 82.4 (71.0–91.6) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | rc4-local | 194.9 (188.9–242.0) | 79.4 (70.5–88.9) | 74.3 (66.1–89.7) | 82.8 (69.8–89.0) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | main-local | 184.0 (179.6–252.6) | 68.2 (66.6–91.4) | 72.7 (67.1–89.4) | 67.6 (67.4–92.0) |
| wasm | read | raw.view | rc4-published | 14.2 (11.5–17.6) | 12.2 (10.1–13.8) | 14.1 (10.9–14.7) | 13.9 (11.7–16.0) |
| wasm | read | raw.view | rc4-local | 14.5 (12.5–17.5) | 12.6 (11.8–13.8) | 13.4 (11.5–15.0) | 13.3 (11.8–15.2) |
| wasm | read | raw.view | main-local | 12.6 (12.5–16.0) | 10.9 (10.5–12.8) | 11.7 (11.3–13.1) | 11.8 (11.6–13.4) |
| wasm | read | raw.view | hist-e6ace96 | 12.4 (10.8–14.1) | 11.1 (9.7–11.3) | 12.3 (10.8–12.5) | 12.4 (10.7–13.3) |
| wasm | read | js.parse_view | rc4-published | 43.7 (32.4–48.2) | 21.9 (17.8–24.9) | 23.7 (18.0–25.0) | 22.4 (18.8–26.4) |
| wasm | read | js.parse_view | rc4-local | 39.2 (34.9–46.7) | 22.4 (20.3–24.8) | 22.1 (19.0–25.0) | 22.1 (19.2–25.2) |
| wasm | read | js.parse_view | main-local | 41.2 (32.0–47.5) | 19.9 (19.4–22.5) | 20.2 (19.6–21.9) | 19.7 (19.2–21.8) |
| wasm | read | js.parse_view | hist-e6ace96 | 38.7 (30.1–46.7) | 21.4 (18.4–21.5) | 22.2 (19.1–22.4) | 21.8 (18.9–23.3) |
| wasm | read | raw.snapshot | rc4-published | 146.5 (104.8–161.3) | 104.7 (86.6–116.5) | 117.2 (90.0–121.1) | 113.8 (96.7–131.1) |
| wasm | read | raw.snapshot | rc4-local | 132.0 (111.3–154.2) | 105.5 (96.7–119.8) | 109.9 (93.9–125.6) | 109.4 (97.2–126.8) |
| wasm | read | raw.snapshot | main-local | 112.5 (105.8–148.8) | 93.1 (90.3–108.3) | 97.7 (94.5–110.2) | 97.5 (95.1–112.4) |
| wasm | read | raw.snapshot | hist-e6ace96 | 138.2 (102.7–150.8) | 101.0 (88.1–103.1) | 112.2 (97.4–114.0) | 113.0 (97.7–122.1) |
| wasm | read | js.parse_snapshot | rc4-published | 97.1 (80.3–117.2) | 78.7 (64.4–89.8) | 88.8 (69.0–95.5) | 85.5 (71.9–100.5) |
| wasm | read | js.parse_snapshot | rc4-local | 96.1 (86.8–113.8) | 81.9 (73.1–90.3) | 84.9 (72.0–96.2) | 85.2 (73.5–96.4) |
| wasm | read | js.parse_snapshot | main-local | 88.8 (85.3–111.9) | 71.3 (68.6–81.9) | 75.2 (73.6–84.6) | 74.9 (72.7–84.5) |
| wasm | read | js.parse_snapshot | hist-e6ace96 | 121.8 (83.7–128.7) | 78.2 (67.0–79.6) | 85.5 (74.3–86.8) | 84.7 (72.9–90.8) |
| wasm | read | raw.save | rc4-published | 24.4 (17.5–28.2) | 17.9 (14.9–19.8) | 22.6 (17.7–24.8) | 23.7 (20.2–27.1) |
| wasm | read | raw.save | rc4-local | 24.7 (19.0–27.2) | 18.4 (16.7–20.4) | 21.2 (18.3–24.8) | 22.8 (20.0–26.2) |
| wasm | read | raw.save | main-local | 18.2 (17.6–26.5) | 16.2 (15.6–18.5) | 19.1 (18.8–22.0) | 20.4 (19.8–23.2) |
| wasm | read | raw.save | hist-e6ace96 | 24.3 (18.2–26.7) | 17.3 (15.5–17.7) | 21.4 (20.0–22.5) | 23.1 (19.8–24.7) |
| wasm | abi.read | abi.view.exec | rc4-published | 10.2 (8.7–14.2) | 7.9 (7.6–10.8) | 8.6 (8.3–11.7) | 9.2 (8.7–12.0) |
| wasm | abi.read | abi.view.exec | rc4-local | 11.9 (9.9–13.6) | 8.9 (7.9–10.4) | 9.4 (8.4–11.1) | 9.8 (8.5–11.9) |
| wasm | abi.read | abi.view.exec | main-local | 11.4 (9.3–16.3) | 8.8 (8.2–10.1) | 9.4 (8.9–11.1) | 9.5 (8.8–11.7) |
| wasm | abi.read | abi.view.exec | hist-e6ace96 | 12.3 (10.3–12.3) | 7.8 (7.1–9.0) | 9.0 (7.6–9.8) | 9.6 (8.0–10.4) |
| wasm | abi.read | abi.view.decode | rc4-published | 2.3 (1.8–3.1) | 2.4 (2.2–3.2) | 2.4 (2.3–3.2) | 2.5 (2.3–3.3) |
| wasm | abi.read | abi.view.decode | rc4-local | 2.5 (2.0–3.0) | 2.7 (2.3–3.1) | 2.7 (2.4–3.2) | 2.7 (2.3–3.3) |
| wasm | abi.read | abi.view.decode | main-local | 2.4 (2.0–2.9) | 2.5 (2.5–3.0) | 2.5 (2.5–3.3) | 2.5 (2.4–3.3) |
| wasm | abi.read | abi.view.decode | hist-e6ace96 | 2.5 (2.1–3.0) | 2.5 (2.3–2.9) | 2.7 (2.3–3.0) | 2.8 (2.4–3.1) |
| wasm | abi.read | abi.snapshot.exec | rc4-published | 96.0 (87.7–127.5) | 69.7 (65.1–94.3) | 73.3 (69.2–100.6) | 77.6 (71.9–100.8) |
| wasm | abi.read | abi.snapshot.exec | rc4-local | 111.1 (86.8–139.4) | 77.9 (69.1–91.5) | 80.2 (71.0–95.6) | 83.2 (72.4–100.4) |
| wasm | abi.read | abi.snapshot.exec | main-local | 109.7 (95.7–144.8) | 75.2 (71.9–89.0) | 81.6 (76.0–95.9) | 79.9 (74.5–99.1) |
| wasm | abi.read | abi.snapshot.exec | hist-e6ace96 | 124.2 (101.8–135.2) | 73.6 (67.6–84.7) | 86.4 (70.8–92.1) | 96.0 (76.0–99.3) |
| wasm | abi.read | abi.snapshot.decode | rc4-published | 10.9 (9.0–14.4) | 11.3 (10.8–15.8) | 12.5 (11.8–16.8) | 13.2 (12.4–17.6) |
| wasm | abi.read | abi.snapshot.decode | rc4-local | 11.8 (9.6–14.2) | 12.8 (11.6–15.3) | 13.9 (12.5–16.7) | 14.2 (12.5–17.6) |
| wasm | abi.read | abi.snapshot.decode | main-local | 11.6 (9.9–13.8) | 12.1 (11.8–15.8) | 13.3 (13.1–17.6) | 13.6 (12.7–18.7) |
| wasm | abi.read | abi.snapshot.decode | hist-e6ace96 | 11.3 (10.0–13.8) | 12.2 (11.5–15.3) | 14.2 (12.0–16.3) | 15.0 (12.7–17.7) |
| wasm | abi.read | abi.save.exec | rc4-published | 16.3 (14.9–23.1) | 12.8 (12.0–17.3) | 15.6 (14.8–20.6) | 16.8 (15.6–21.6) |
| wasm | abi.read | abi.save.exec | rc4-local | 19.4 (15.5–22.1) | 14.2 (12.7–16.4) | 16.9 (14.9–20.3) | 18.0 (15.7–21.7) |
| wasm | abi.read | abi.save.exec | main-local | 18.9 (15.4–25.7) | 13.8 (13.1–16.3) | 17.1 (15.5–19.8) | 17.5 (15.9–21.3) |
| wasm | abi.read | abi.save.exec | hist-e6ace96 | 19.4 (16.5–24.3) | 13.3 (12.1–15.1) | 17.6 (14.9–19.9) | 20.1 (16.2–21.3) |
| wasm | abi.read | abi.save.decode | rc4-published | 0.6 (0.5–0.7) | 0.5 (0.5–0.7) | 0.7 (0.6–0.8) | 0.7 (0.6–0.8) |
| wasm | abi.read | abi.save.decode | rc4-local | 0.6 (0.5–0.7) | 0.5 (0.5–0.6) | 0.7 (0.6–0.8) | 0.7 (0.6–0.8) |
| wasm | abi.read | abi.save.decode | main-local | 0.6 (0.5–0.8) | 0.5 (0.5–0.6) | 0.7 (0.6–0.8) | 0.7 (0.6–0.8) |
| wasm | abi.read | abi.save.decode | hist-e6ace96 | 0.6 (0.5–0.7) | 0.5 (0.5–0.6) | 0.7 (0.6–0.7) | 0.8 (0.7–0.9) |
| wasm | kit | kit.dispatch | rc4-published | 313.0 (288.4–440.5) | 135.3 (133.0–187.6) | 140.3 (138.1–187.7) | 151.1 (142.7–196.1) |
| wasm | kit | kit.dispatch | rc4-local | 306.1 (290.0–373.1) | 143.7 (140.2–164.8) | 147.3 (140.3–194.8) | 150.1 (140.8–201.0) |
| wasm | kit | kit.dispatch | main-local | 332.5 (261.3–483.0) | 138.1 (134.9–184.1) | 144.3 (136.6–182.2) | 154.0 (141.1–196.1) |
| wasm | kit | kit.view | rc4-published | 44.0 (42.3–64.7) | 27.7 (27.0–38.3) | 29.2 (29.1–38.9) | 31.7 (29.6–40.2) |
| wasm | kit | kit.view | rc4-local | 43.5 (39.2–51.5) | 28.8 (28.7–33.9) | 30.0 (29.2–40.3) | 31.0 (29.0–41.5) |
| wasm | kit | kit.view | main-local | 45.3 (44.5–63.5) | 28.3 (27.6–37.6) | 30.0 (28.4–38.1) | 32.1 (29.0–40.6) |
| wasm | kit | kit.dispatch+view | rc4-published | 359.2 (332.4–521.7) | 163.3 (160.5–226.6) | 169.7 (167.7–227.5) | 183.1 (172.5–236.9) |
| wasm | kit | kit.dispatch+view | rc4-local | 382.0 (339.5–426.3) | 172.9 (169.4–199.6) | 177.7 (169.5–236.5) | 181.6 (170.0–243.4) |
| wasm | kit | kit.dispatch+view | main-local | 378.6 (295.7–547.7) | 167.0 (162.9–222.6) | 174.7 (165.6–220.9) | 186.4 (170.5–237.2) |
| wasm | kit.read | kit.view | rc4-published | 46.7 (42.7–88.3) | 34.9 (30.4–45.9) | 35.1 (31.5–47.2) | 35.5 (31.8–48.1) |
| wasm | kit.read | kit.view | rc4-local | 44.8 (41.8–66.1) | 31.9 (28.7–42.7) | 31.2 (30.2–43.1) | 37.4 (32.3–45.9) |
| wasm | kit.read | kit.view | main-local | 53.2 (45.9–76.2) | 30.8 (30.4–36.4) | 33.1 (30.2–39.2) | 35.6 (33.1–42.5) |
| wasm | kit.read | kit.snapshot | rc4-published | 205.9 (178.5–256.6) | 187.0 (165.6–249.0) | 189.7 (172.2–255.7) | 193.9 (174.8–268.5) |
| wasm | kit.read | kit.snapshot | rc4-local | 181.4 (163.8–269.7) | 171.7 (154.4–228.9) | 168.9 (165.8–233.6) | 206.4 (177.6–252.0) |
| wasm | kit.read | kit.snapshot | main-local | 183.0 (177.9–243.2) | 165.1 (164.1–198.1) | 177.8 (164.8–212.9) | 195.6 (181.4–236.7) |
| wasm | kit.read | kit.save | rc4-published | 42.7 (39.9–64.3) | 38.5 (34.0–50.4) | 41.7 (38.6–58.1) | 46.8 (41.9–64.2) |
| wasm | kit.read | kit.save | rc4-local | 43.6 (39.6–63.5) | 35.1 (31.6–46.9) | 38.1 (37.8–55.5) | 49.2 (42.9–60.6) |
| wasm | kit.read | kit.save | main-local | 50.6 (46.1–64.5) | 33.8 (33.4–40.3) | 41.9 (37.9–49.3) | 47.0 (44.1–56.8) |

## glowcap-resume

The published resume measurement's stream (experiments/glowcap/resume-bench.mjs): absorb cave glowcap, then 9,600 ticks of dt 0.0625 (ten minutes of play), then save and restore. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `0407c19120d4`, 1 episode(s) × 1 = 9601 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | lifecycle | from_source | - | 2819.5 (2640.7–4050.9) · p95 2952.8 | 3693.5 (2650.1–4039.6) · p95 4127.9 | - |
| native | lifecycle | restore | - | 2897.0 (2683.9–4187.9) · p95 3194.7 | 3838.4 (2738.6–4184.9) · p95 4408.6 | - |
| native | lifecycle | restore.parse | - | 30.1 (19.0–40.9) · p95 32.9 | 33.6 (18.4–41.7) · p95 39.9 | - |
| native | lifecycle | restore_json | - | 2942.5 (2717.2–4181.0) · p95 3108.4 | 3879.7 (2744.3–4254.3) · p95 4322.3 | - |
| native | lifecycle | save | - | 19.3 (12.6–27.0) · p95 22.4 | 23.5 (12.7–26.7) · p95 29.9 | - |
| native | lifecycle | save_json | - | 21.0 (17.7–26.6) · p95 27.8 | 35.2 (24.1–43.4) · p95 43.1 | - |
| native | lifecycle | web.new | - | 2789.8 (2598.7–4013.8) · p95 3032.6 | 3654.4 (2626.3–4043.1) · p95 4125.9 | - |
| native | lifecycle | web.restore | - | 2923.5 (2735.3–4196.3) · p95 3157.5 | 3722.8 (2737.6–4301.2) · p95 4377.2 | - |
| native | lifecycle | web.save | - | 16.1 (10.7–20.4) · p95 18.2 | 18.2 (10.9–20.5) · p95 21.0 | - |
| wasm | lifecycle | raw.new | 3568.9 (2529.9–3916.7) · p95 4012.6 | 2860.8 (2717.9–3592.9) · p95 3165.3 | 3554.1 (2557.6–4580.1) · p95 3738.1 | 3319.9 (2512.0–4839.4) · p95 3566.3 |
| wasm | lifecycle | raw.save | 53.4 (33.9–61.6) · p95 77.0 | 42.4 (35.0–59.3) · p95 52.9 | 55.8 (32.5–65.0) · p95 70.0 | 53.7 (34.2–69.7) · p95 62.8 |
| wasm | lifecycle | raw.restore | 3519.7 (2557.4–3910.2) · p95 4069.5 | 2830.7 (2692.0–3514.6) · p95 3260.7 | 3550.7 (2544.4–4540.8) · p95 3766.6 | 3305.5 (2568.7–4844.3) · p95 3526.0 |
| wasm | lifecycle | kit.open | 3423.3 (2426.3–3842.4) · p95 3848.9 | 2792.4 (2637.2–3438.7) · p95 3139.4 | 3460.6 (2484.4–4464.3) · p95 3636.4 | - |
| wasm | lifecycle | kit.save | 115.1 (69.8–144.4) · p95 168.4 | 99.7 (68.4–133.7) · p95 126.4 | 133.9 (64.7–139.1) · p95 168.6 | - |
| wasm | lifecycle | kit.restore | 3482.7 (2511.1–3846.2) · p95 3969.0 | 2821.6 (2779.1–3525.2) · p95 3053.6 | 3476.0 (2533.2–4517.5) · p95 3652.8 | - |
| wasm | adapter-resume | adapter.resume.parse | 18.6 (14.4–19.5) · p95 18.6 | 18.6 (14.9–19.9) · p95 18.6 | 19.9 (14.4–21.7) · p95 19.9 | 16.9 (13.7–19.0) · p95 16.9 |
| wasm | adapter-resume | adapter.resume.createPolicy | 4139.5 (3605.8–4387.9) · p95 4139.5 | 4453.1 (3461.5–5763.3) · p95 4453.1 | 4616.2 (3586.0–5007.6) · p95 4616.2 | 4251.5 (3268.0–4287.1) · p95 4251.5 |
| wasm | adapter-resume | adapter.resume.view | 53.9 (46.1–57.2) · p95 53.9 | 55.0 (42.3–76.0) · p95 55.0 | 66.6 (43.6–67.1) · p95 66.6 | 50.7 (47.3–53.9) · p95 50.7 |
| wasm | adapter-resume | adapter.resume | 4212.9 (3655.3–4455.3) · p95 4212.9 | 4514.8 (3511.0–5827.6) · p95 4514.8 | 4675.3 (3633.3–5095.9) · p95 4675.3 | 4311.9 (3319.7–4343.5) · p95 4311.9 |

## glowcap-unbound

The Glowcap replay program with its 39 one-line bind statements removed, on the replay's full stream: the same rules, states and transaction with no bindings to evaluate, so apply here against apply on glowcap-replay isolates binding evaluation. Program `experiments/glowcap/caveat5/glowcap.cav` (`ffa078c4cb75`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 14.9 (12.5–16.2) · p95 16.5 | 16.1 (12.5–20.2) · p95 18.0 | - |
| native | dispatch_view_json | dispatch_view_json | - | 18.1 (16.0–19.7) · p95 19.7 | 19.3 (17.4–19.7) · p95 21.0 | - |
| native | web.dispatch_view | web.dispatch_view | - | 22.8 (17.6–22.9) · p95 24.4 | 20.6 (17.5–25.7) · p95 21.7 | - |
| native | read | clone | - | 0.4 (0.3–0.4) · p95 0.4 | 0.4 (0.3–0.4) · p95 0.5 | - |
| native | read | clone.drop | - | 0.3 (0.3–0.3) · p95 0.4 | 0.4 (0.3–0.4) · p95 0.4 | - |
| native | read | save | - | 10.2 (9.0–10.4) · p95 11.8 | 10.9 (9.1–11.0) · p95 12.8 | - |
| native | read | save_json | - | 16.4 (14.4–16.8) · p95 19.0 | 17.4 (14.6–17.7) · p95 20.6 | - |
| native | read | snapshot | - | 21.0 (18.2–21.1) · p95 23.9 | 21.8 (18.3–22.5) · p95 25.9 | - |
| native | read | snapshot.drop | - | 8.2 (7.2–8.3) · p95 9.3 | 8.4 (7.1–8.6) · p95 9.9 | - |
| native | read | snapshot.serialize | - | 13.5 (11.8–13.7) · p95 15.4 | 14.3 (12.0–14.7) · p95 16.9 | - |
| native | read | snapshot.serialize_pretty | - | 27.3 (23.9–27.7) · p95 31.4 | 28.6 (24.2–29.6) · p95 34.0 | - |
| native | read | view | - | 2.9 (2.5–2.9) · p95 3.3 | 2.9 (2.5–3.1) · p95 3.5 | - |
| native | read | view.serialize | - | 2.7 (2.4–2.8) · p95 3.1 | 2.9 (2.4–3.0) · p95 3.5 | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.2 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.3 |
| wasm | abi.dispatch_view | abi.exec | 20.3 (18.4–21.1) · p95 25.5 | 19.0 (19.0–25.8) · p95 23.8 | 18.3 (18.1–21.3) · p95 24.0 | 17.4 (17.0–19.6) · p95 23.3 |
| wasm | abi.dispatch_view | abi.decode | 0.6 (0.5–0.6) · p95 1.1 | 0.5 (0.5–0.7) · p95 1.0 | 0.5 (0.5–0.6) · p95 1.0 | 0.5 (0.5–0.5) · p95 0.9 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | abi.dispatch_view | abi.dispatch_view | 21.1 (19.2–22.0) · p95 27.0 | 19.8 (19.8–27.1) · p95 24.8 | 19.1 (18.9–22.3) · p95 25.1 | 18.3 (17.7–20.5) · p95 24.6 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.2–0.3) · p95 0.4 | 0.3 (0.3–0.3) · p95 0.5 | 0.2 (0.2–0.3) · p95 0.4 | 0.2 (0.2–0.3) · p95 0.4 |
| wasm | raw.dispatch_view | raw.dispatch_view | 19.9 (19.5–23.2) · p95 24.2 | 20.8 (19.9–24.1) · p95 29.4 | 19.5 (19.4–23.1) · p95 24.7 | 18.2 (17.7–22.5) · p95 21.7 |
| wasm | raw.dispatch_view | js.parse_view | 6.5 (6.4–7.6) · p95 7.8 | 6.6 (6.5–7.9) · p95 9.5 | 6.3 (6.2–7.5) · p95 7.5 | 5.9 (5.7–7.3) · p95 6.9 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 26.6 (26.2–31.1) · p95 32.2 | 27.8 (26.7–32.3) · p95 39.3 | 26.0 (25.9–31.0) · p95 32.5 | 24.4 (23.6–30.0) · p95 28.6 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | rc4-local | 38.6 (38.0–42.3) | 16.5 (13.7–18.1) | 15.5 (12.7–16.5) | 14.8 (12.4–16.2) |
| native | apply | apply | main-local | 48.7 (32.7–54.0) | 17.8 (13.7–22.6) | 16.6 (13.0–20.9) | 16.1 (12.4–20.1) |
| native | dispatch_view_json | dispatch_view_json | rc4-local | 44.4 (37.1–61.4) | 19.6 (16.8–21.0) | 18.6 (16.3–20.1) | 18.1 (16.0–19.6) |
| native | dispatch_view_json | dispatch_view_json | main-local | 43.4 (42.4–48.3) | 20.6 (18.4–21.1) | 19.8 (17.7–20.2) | 19.2 (17.3–19.7) |
| native | web.dispatch_view | web.dispatch_view | rc4-local | 52.1 (49.2–63.4) | 23.7 (18.3–23.8) | 22.9 (18.2–23.0) | 22.8 (17.6–22.9) |
| native | web.dispatch_view | web.dispatch_view | main-local | 49.3 (41.7–54.9) | 21.4 (18.1–26.6) | 20.8 (17.7–26.0) | 20.6 (17.5–25.6) |
| native | read | clone | rc4-local | 0.7 (0.6–0.7) | 0.4 (0.3–0.4) | 0.4 (0.3–0.4) | 0.4 (0.3–0.4) |
| native | read | clone | main-local | 0.7 (0.6–0.7) | 0.4 (0.3–0.4) | 0.4 (0.3–0.4) | 0.4 (0.3–0.4) |
| native | read | clone.drop | rc4-local | 0.5 (0.4–0.5) | 0.3 (0.3–0.3) | 0.3 (0.3–0.4) | 0.3 (0.3–0.3) |
| native | read | clone.drop | main-local | 0.5 (0.4–0.6) | 0.4 (0.3–0.4) | 0.4 (0.3–0.4) | 0.4 (0.3–0.4) |
| native | read | save | rc4-local | 6.2 (5.3–6.3) | 7.0 (6.2–7.4) | 9.5 (8.4–9.9) | 10.3 (9.0–10.5) |
| native | read | save | main-local | 6.6 (5.4–6.7) | 7.5 (6.3–7.7) | 9.9 (8.6–10.2) | 11.1 (9.1–11.1) |
| native | read | save_json | rc4-local | 10.4 (9.1–10.8) | 12.0 (10.6–12.6) | 15.9 (14.0–16.7) | 16.5 (14.5–16.9) |
| native | read | save_json | main-local | 11.1 (8.7–12.0) | 12.4 (10.5–12.8) | 16.5 (14.2–17.0) | 17.6 (14.6–17.9) |
| native | read | snapshot | rc4-local | 25.1 (24.1–28.6) | 17.9 (16.0–18.9) | 19.9 (17.6–20.9) | 21.1 (18.3–21.2) |
| native | read | snapshot | main-local | 23.7 (23.1–28.4) | 18.7 (15.9–19.7) | 20.6 (17.6–22.0) | 22.1 (18.4–22.6) |
| native | read | snapshot.drop | rc4-local | 7.8 (6.5–7.9) | 7.0 (6.2–7.3) | 7.7 (6.8–8.1) | 8.3 (7.2–8.3) |
| native | read | snapshot.drop | main-local | 7.9 (6.5–8.5) | 7.2 (6.2–7.5) | 7.9 (6.8–8.3) | 8.6 (7.2–8.7) |
| native | read | snapshot.serialize | rc4-local | 16.1 (14.8–17.3) | 12.3 (10.8–12.7) | 13.1 (11.5–13.7) | 13.6 (11.8–13.7) |
| native | read | snapshot.serialize | main-local | 17.5 (13.2–18.3) | 13.0 (11.1–13.4) | 13.7 (11.7–14.4) | 14.4 (12.0–14.7) |
| native | read | snapshot.serialize_pretty | rc4-local | 30.9 (28.6–35.4) | 24.5 (21.6–25.3) | 26.3 (23.4–27.8) | 27.5 (24.0–27.8) |
| native | read | snapshot.serialize_pretty | main-local | 30.3 (26.1–35.0) | 25.3 (21.8–26.3) | 27.4 (23.6–29.0) | 29.0 (24.3–29.7) |
| native | read | view | rc4-local | 2.6 (2.3–2.6) | 2.2 (1.9–2.3) | 2.7 (2.3–2.8) | 2.9 (2.5–2.9) |
| native | read | view | main-local | 2.5 (2.1–2.9) | 2.3 (1.9–2.5) | 2.6 (2.3–2.8) | 3.0 (2.5–3.1) |
| native | read | view.serialize | rc4-local | 3.6 (3.1–3.8) | 2.5 (2.2–2.6) | 2.6 (2.3–2.7) | 2.7 (2.4–2.8) |
| native | read | view.serialize | main-local | 3.7 (3.0–4.1) | 2.7 (2.2–2.7) | 2.7 (2.3–2.9) | 3.0 (2.5–3.0) |
| wasm | abi.dispatch_view | abi.encode_args | rc4-published | 0.9 (0.8–1.0) | 0.3 (0.2–0.3) | 0.2 (0.1–0.2) | 0.2 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.encode_args | rc4-local | 0.9 (0.8–1.4) | 0.3 (0.2–0.3) | 0.1 (0.1–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.encode_args | main-local | 0.9 (0.9–0.9) | 0.3 (0.3–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.encode_args | hist-e6ace96 | 1.0 (0.9–1.0) | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.exec | rc4-published | 57.7 (57.7–70.6) | 21.6 (19.9–22.7) | 21.7 (20.9–23.0) | 20.3 (18.1–20.8) |
| wasm | abi.dispatch_view | abi.exec | rc4-local | 58.5 (55.4–74.0) | 21.8 (20.2–26.4) | 19.8 (19.6–22.7) | 18.9 (18.9–26.2) |
| wasm | abi.dispatch_view | abi.exec | main-local | 60.4 (53.1–74.4) | 21.7 (19.6–27.8) | 19.9 (18.7–28.8) | 18.2 (17.9–21.1) |
| wasm | abi.dispatch_view | abi.exec | hist-e6ace96 | 67.4 (52.7–72.5) | 21.0 (18.6–21.3) | 19.6 (17.3–19.7) | 17.1 (16.9–19.4) |
| wasm | abi.dispatch_view | abi.decode | rc4-published | 1.4 (1.4–1.8) | 0.6 (0.5–0.6) | 0.6 (0.5–0.6) | 0.6 (0.5–0.6) |
| wasm | abi.dispatch_view | abi.decode | rc4-local | 1.4 (1.2–2.1) | 0.6 (0.5–0.7) | 0.5 (0.5–0.7) | 0.5 (0.5–0.8) |
| wasm | abi.dispatch_view | abi.decode | main-local | 1.5 (1.3–1.7) | 0.6 (0.6–0.7) | 0.5 (0.5–0.7) | 0.5 (0.5–0.6) |
| wasm | abi.dispatch_view | abi.decode | hist-e6ace96 | 1.5 (1.2–1.8) | 0.6 (0.5–0.6) | 0.5 (0.4–0.5) | 0.5 (0.5–0.5) |
| wasm | abi.dispatch_view | abi.free | rc4-published | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | rc4-local | 0.3 (0.2–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.free | main-local | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | hist-e6ace96 | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.dispatch_view | rc4-published | 60.5 (60.2–73.3) | 22.5 (20.9–23.7) | 22.5 (21.7–23.9) | 21.1 (19.0–21.7) |
| wasm | abi.dispatch_view | abi.dispatch_view | rc4-local | 61.4 (56.7–76.3) | 22.6 (21.2–27.5) | 20.6 (20.3–23.7) | 19.7 (19.7–27.6) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local | 63.0 (55.0–76.7) | 22.6 (20.6–29.0) | 20.7 (19.5–29.9) | 19.0 (18.7–22.0) |
| wasm | abi.dispatch_view | abi.dispatch_view | hist-e6ace96 | 73.3 (54.3–74.0) | 22.0 (19.4–22.4) | 20.4 (18.0–20.5) | 17.9 (17.6–20.4) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-published | 0.9 (0.7–1.5) | 0.3 (0.3–0.3) | 0.3 (0.2–0.3) | 0.3 (0.2–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-local | 1.1 (0.8–1.4) | 0.3 (0.3–0.4) | 0.3 (0.2–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local | 1.1 (1.1–1.1) | 0.3 (0.3–0.3) | 0.2 (0.2–0.3) | 0.2 (0.2–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | hist-e6ace96 | 0.8 (0.8–1.9) | 0.3 (0.3–0.3) | 0.2 (0.2–0.3) | 0.2 (0.2–0.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-published | 59.9 (55.7–113.0) | 20.7 (20.7–25.1) | 20.1 (19.9–24.2) | 19.8 (19.4–23.1) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-local | 81.2 (72.6–89.4) | 22.6 (20.9–25.9) | 20.8 (20.4–25.2) | 20.8 (19.8–24.0) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local | 92.9 (84.6–93.1) | 23.7 (20.8–24.0) | 20.1 (19.7–23.5) | 19.3 (19.3–23.1) |
| wasm | raw.dispatch_view | raw.dispatch_view | hist-e6ace96 | 74.2 (65.5–93.8) | 20.7 (19.5–23.1) | 18.7 (18.6–22.4) | 18.2 (17.6–22.4) |
| wasm | raw.dispatch_view | js.parse_view | rc4-published | 14.7 (12.7–20.0) | 6.1 (6.0–7.3) | 6.2 (6.0–7.5) | 6.5 (6.4–7.6) |
| wasm | raw.dispatch_view | js.parse_view | rc4-local | 14.4 (13.1–19.3) | 6.3 (6.0–7.6) | 6.4 (6.1–7.8) | 6.7 (6.5–7.9) |
| wasm | raw.dispatch_view | js.parse_view | main-local | 14.6 (13.6–18.2) | 6.6 (5.9–6.8) | 6.1 (6.1–7.1) | 6.4 (6.2–7.5) |
| wasm | raw.dispatch_view | js.parse_view | hist-e6ace96 | 13.1 (11.9–16.1) | 5.7 (5.5–6.5) | 5.7 (5.6–6.8) | 5.9 (5.7–7.3) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-published | 75.5 (67.5–133.3) | 27.1 (27.1–32.6) | 26.4 (26.2–32.0) | 26.6 (26.1–31.0) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-local | 96.6 (83.6–104.4) | 29.1 (27.3–33.8) | 27.5 (26.8–33.4) | 27.8 (26.6–32.3) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local | 108.1 (97.9–111.8) | 30.7 (27.0–31.1) | 26.3 (26.0–30.9) | 25.9 (25.8–30.9) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | hist-e6ace96 | 96.6 (79.0–111.2) | 26.7 (25.3–29.9) | 24.5 (24.5–29.5) | 24.4 (23.5–30.1) |

## glowcap-scaled-16

Glowcap with 12 more mushrooms declared after grove (16 in all): symbols, edges, tick rules and bindings grow with the mushroom count; the replay's first 2,000 events. Program `experiments/glowcap/caveat5/glowcap.cav` + 12 mushrooms (`468147924b4a`), stream `f93d60a5f44d`, 1 episode(s) × 1 = 2000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 31.3 (29.0–39.8) · p95 36.6 | 32.0 (28.8–39.4) · p95 37.3 | - |
| native | web.dispatch_view | web.dispatch_view | - | 53.4 (44.7–62.7) · p95 59.7 | 50.7 (44.9–63.5) · p95 56.8 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 161.2 (159.3–221.2) · p95 200.2 | 209.0 (146.1–215.9) · p95 245.4 | - |
| native | read | clone | - | 20.4 (18.0–26.7) · p95 23.5 | 21.7 (17.6–26.7) · p95 24.0 | - |
| native | read | clone.drop | - | 11.5 (10.2–15.1) · p95 12.8 | 12.1 (9.8–15.1) · p95 13.4 | - |
| native | read | save | - | 11.4 (9.7–15.1) · p95 13.0 | 11.8 (9.7–15.1) · p95 13.3 | - |
| native | read | save_json | - | 18.2 (15.4–23.5) · p95 19.9 | 18.9 (15.6–24.1) · p95 20.8 | - |
| native | read | snapshot | - | 68.4 (58.8–89.2) · p95 83.2 | 69.9 (57.9–89.7) · p95 79.1 | - |
| native | read | snapshot.drop | - | 26.9 (23.3–34.7) · p95 30.1 | 27.6 (23.0–34.7) · p95 30.2 | - |
| native | read | snapshot.serialize | - | 38.7 (33.5–50.1) · p95 56.4 | 40.2 (33.0–51.2) · p95 49.5 | - |
| native | read | snapshot.serialize_pretty | - | 83.4 (71.8–108.4) · p95 114.6 | 87.1 (71.5–110.6) · p95 109.6 | - |
| native | read | view | - | 7.0 (6.0–9.0) · p95 7.7 | 7.2 (5.8–9.0) · p95 8.0 | - |
| native | read | view.serialize | - | 10.3 (8.9–13.4) · p95 11.6 | 10.7 (8.8–13.6) · p95 12.0 | - |
| native | lifecycle | from_source | - | 15856.7 (11289.1–18262.9) · p95 17184.9 | 14808.1 (11267.3–18177.9) · p95 16259.4 | - |
| native | lifecycle | restore | - | 16217.1 (11583.7–18739.0) · p95 17568.3 | 15218.6 (11697.1–18617.6) · p95 16556.3 | - |
| native | lifecycle | restore.parse | - | 47.6 (37.5–53.1) · p95 56.3 | 44.5 (38.9–52.1) · p95 53.1 | - |
| native | lifecycle | restore_json | - | 16147.4 (11547.7–18839.5) · p95 17602.3 | 15057.5 (11688.5–18973.7) · p95 16004.0 | - |
| native | lifecycle | save | - | 51.4 (36.1–52.8) · p95 67.4 | 46.5 (39.5–50.3) · p95 62.4 | - |
| native | lifecycle | save_json | - | 37.2 (27.7–41.9) · p95 41.7 | 33.4 (28.5–41.5) · p95 37.6 | - |
| native | lifecycle | web.new | - | 15805.5 (11066.4–18067.5) · p95 16815.7 | 14606.3 (11661.3–18089.8) · p95 15835.7 | - |
| native | lifecycle | web.restore | - | 16393.4 (11999.4–18895.9) · p95 17438.8 | 15234.7 (11687.1–18677.8) · p95 16133.2 | - |
| native | lifecycle | web.save | - | 39.1 (28.7–43.1) · p95 52.4 | 36.5 (29.9–40.4) · p95 42.0 | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.5 (0.3–0.5) · p95 0.9 | 0.4 (0.3–0.5) · p95 0.8 | 0.4 (0.3–0.4) · p95 0.8 | 0.5 (0.3–0.5) · p95 0.9 |
| wasm | raw.dispatch_view | raw.dispatch_view | 80.1 (60.1–85.5) · p95 104.2 | 68.0 (58.6–88.0) · p95 85.5 | 69.2 (58.8–80.6) · p95 81.2 | 68.9 (57.9–85.6) · p95 103.8 |
| wasm | raw.dispatch_view | js.parse_view | 52.0 (38.6–55.3) · p95 64.4 | 44.2 (37.8–57.4) · p95 52.1 | 45.8 (38.1–51.8) · p95 51.4 | 45.0 (39.0–54.9) · p95 65.1 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 133.4 (99.3–142.5) · p95 169.2 | 113.0 (97.0–147.1) · p95 136.5 | 115.9 (97.5–134.0) · p95 131.6 | 115.4 (97.2–141.9) · p95 168.9 |
| wasm | abi.dispatch_view | abi.encode_args | 0.2 (0.2–0.3) · p95 0.6 | 0.2 (0.2–0.3) · p95 0.5 | 0.3 (0.2–0.3) · p95 0.5 | 0.2 (0.2–0.3) · p95 0.4 |
| wasm | abi.dispatch_view | abi.exec | 64.8 (52.4–75.7) · p95 87.7 | 60.7 (57.5–72.3) · p95 85.1 | 60.6 (53.2–75.7) · p95 86.4 | 58.6 (52.7–69.9) · p95 76.5 |
| wasm | abi.dispatch_view | abi.decode | 7.3 (5.6–8.1) · p95 9.5 | 6.8 (6.3–8.0) · p95 9.2 | 6.7 (5.9–8.2) · p95 9.8 | 6.8 (5.9–8.0) · p95 8.7 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | abi.dispatch_view | abi.dispatch_view | 72.7 (58.5–84.3) · p95 98.4 | 67.9 (64.3–80.9) · p95 95.7 | 67.7 (59.6–84.5) · p95 97.8 | 65.8 (59.0–78.5) · p95 86.1 |
| wasm | read | raw.view | 28.4 (24.9–36.9) · p95 44.6 | 26.1 (26.0–35.5) · p95 34.7 | 26.6 (24.9–37.4) · p95 34.0 | 29.2 (24.1–34.2) · p95 42.4 |
| wasm | read | js.parse_view | 47.3 (41.3–60.7) · p95 70.5 | 43.7 (43.1–59.2) · p95 55.7 | 44.7 (40.2–61.7) · p95 54.3 | 50.8 (42.8–59.3) · p95 70.3 |
| wasm | read | raw.snapshot | 312.7 (276.1–419.8) · p95 466.7 | 290.0 (288.9–411.7) · p95 362.6 | 297.1 (270.7–428.1) · p95 355.2 | 336.7 (279.5–416.2) · p95 481.1 |
| wasm | read | js.parse_snapshot | 193.8 (169.8–250.0) · p95 273.9 | 179.1 (177.6–243.4) · p95 225.6 | 183.2 (166.3–255.0) · p95 221.7 | 209.9 (176.8–246.6) · p95 285.7 |
| wasm | read | raw.save | 26.6 (23.6–36.0) · p95 44.4 | 25.5 (24.6–34.7) · p95 33.9 | 25.8 (23.3–36.5) · p95 33.3 | 28.0 (24.0–34.7) · p95 40.8 |
| wasm | kit | kit.dispatch | 419.0 (372.7–454.2) · p95 641.6 | 396.2 (374.0–439.6) · p95 512.0 | 451.4 (364.7–462.1) · p95 593.5 | - |
| wasm | kit | kit.view | 81.0 (72.0–87.4) · p95 120.6 | 75.5 (72.0–84.4) · p95 95.5 | 86.8 (70.3–88.5) · p95 102.1 | - |
| wasm | kit | kit.dispatch+view | 501.2 (445.1–543.8) · p95 750.7 | 475.2 (446.5–525.5) · p95 601.6 | 539.7 (435.4–552.2) · p95 686.6 | - |
| wasm | lifecycle | raw.new | 11820.8 (10581.7–13755.3) · p95 12437.8 | 13486.8 (10473.0–14227.9) · p95 14590.7 | 16430.7 (10236.6–16831.3) · p95 17364.8 | 10827.6 (10303.6–14561.3) · p95 11676.5 |
| wasm | lifecycle | raw.save | 81.3 (62.2–100.0) · p95 98.3 | 103.0 (64.4–133.9) · p95 155.5 | 123.3 (58.9–124.9) · p95 187.5 | 77.2 (57.5–141.8) · p95 99.4 |
| wasm | lifecycle | raw.restore | 11758.3 (10091.0–13638.1) · p95 12194.4 | 13431.9 (10441.2–14121.9) · p95 14403.5 | 16834.1 (10248.6–16981.9) · p95 17898.2 | 10677.9 (10031.2–14777.7) · p95 11972.3 |
| wasm | lifecycle | kit.open | 11927.5 (10812.7–13653.7) · p95 12447.7 | 13379.3 (10280.2–14384.0) · p95 14559.2 | 16940.0 (10146.9–17502.8) · p95 18509.5 | - |
| wasm | lifecycle | kit.save | 160.0 (134.4–199.2) · p95 196.1 | 200.2 (130.6–271.3) · p95 240.6 | 227.9 (122.5–243.8) · p95 326.0 | - |
| wasm | lifecycle | kit.restore | 11957.7 (10579.0–13885.4) · p95 12541.3 | 13501.0 (10517.5–14539.7) · p95 14836.5 | 16408.4 (10296.9–17116.9) · p95 18215.0 | - |

## glowcap-scaled-64

Glowcap with 60 more mushrooms declared after grove (64 in all); the replay's first 2,000 events. Program `experiments/glowcap/caveat5/glowcap.cav` + 60 mushrooms (`29f4f9803de2`), stream `f93d60a5f44d`, 1 episode(s) × 1 = 2000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 111.5 (100.1–132.9) · p95 130.5 | 121.3 (92.5–147.1) · p95 142.0 | - |
| native | web.dispatch_view | web.dispatch_view | - | 179.7 (150.2–231.1) · p95 201.0 | 171.8 (166.5–210.0) · p95 231.8 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 666.5 (597.4–686.2) · p95 774.2 | 612.0 (586.4–679.4) · p95 754.9 | - |
| native | read | clone | - | 79.4 (76.0–93.5) · p95 105.1 | 80.8 (73.8–91.2) · p95 104.2 | - |
| native | read | clone.drop | - | 43.7 (41.2–49.9) · p95 56.5 | 44.1 (40.9–49.0) · p95 55.5 | - |
| native | read | save | - | 17.7 (17.0–21.6) · p95 25.9 | 18.7 (17.1–21.5) · p95 26.4 | - |
| native | read | save_json | - | 25.0 (24.6–30.7) · p95 34.7 | 26.6 (23.8–30.2) · p95 34.8 | - |
| native | read | snapshot | - | 257.9 (245.2–302.8) · p95 338.8 | 265.3 (240.1–297.7) · p95 369.8 | - |
| native | read | snapshot.drop | - | 92.7 (88.4–107.5) · p95 119.3 | 94.0 (85.7–105.4) · p95 117.8 | - |
| native | read | snapshot.serialize | - | 139.6 (132.4–160.1) · p95 184.2 | 142.1 (130.9–160.9) · p95 242.6 | - |
| native | read | snapshot.serialize_pretty | - | 292.3 (277.6–335.5) · p95 373.5 | 293.9 (269.3–328.8) · p95 399.8 | - |
| native | read | view | - | 30.7 (28.9–35.1) · p95 39.9 | 30.8 (28.4–34.7) · p95 39.2 | - |
| native | read | view.serialize | - | 33.8 (31.8–39.1) · p95 46.0 | 34.0 (31.3–38.3) · p95 47.8 | - |
| native | lifecycle | from_source | - | 121834.6 (119690.3–127678.6) · p95 140423.4 | 135456.9 (111843.0–148032.0) · p95 139199.5 | - |
| native | lifecycle | restore | - | 122366.9 (120353.0–127118.3) · p95 139654.6 | 137370.0 (112528.8–147911.7) · p95 142252.7 | - |
| native | lifecycle | restore.parse | - | 50.6 (48.9–53.2) · p95 60.9 | 53.7 (47.2–57.9) · p95 60.0 | - |
| native | lifecycle | restore_json | - | 125972.2 (121417.6–128778.0) · p95 138501.7 | 137748.4 (111883.1–146258.1) · p95 141852.7 | - |
| native | lifecycle | save | - | 73.7 (71.7–73.9) · p95 88.9 | 72.7 (69.1–86.7) · p95 101.9 | - |
| native | lifecycle | save_json | - | 44.3 (39.7–47.9) · p95 59.3 | 47.2 (41.2–50.4) · p95 57.3 | - |
| native | lifecycle | web.new | - | 121262.9 (119947.9–127201.5) · p95 140001.3 | 136601.5 (110651.8–149932.9) · p95 141205.6 | - |
| native | lifecycle | web.restore | - | 128441.7 (121685.6–128671.2) · p95 139815.2 | 138346.9 (113156.8–152353.2) · p95 142995.5 | - |
| native | lifecycle | web.save | - | 54.5 (53.7–59.6) · p95 64.2 | 52.9 (52.7–70.2) · p95 61.0 | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.5 (0.5–0.6) · p95 1.5 | 0.6 (0.6–0.6) · p95 1.3 | 0.6 (0.5–0.7) · p95 1.4 | 0.6 (0.5–0.6) · p95 1.3 |
| wasm | raw.dispatch_view | raw.dispatch_view | 226.9 (220.4–241.5) · p95 389.0 | 232.6 (218.5–261.6) · p95 350.4 | 237.1 (225.0–307.6) · p95 308.4 | 218.2 (216.2–305.4) · p95 348.3 |
| wasm | raw.dispatch_view | js.parse_view | 155.0 (151.0–160.2) · p95 242.9 | 148.1 (141.7–173.4) · p95 217.2 | 157.3 (147.4–198.5) · p95 176.3 | 146.3 (142.2–204.4) · p95 217.0 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 393.3 (373.1–397.9) · p95 624.9 | 382.1 (363.0–441.5) · p95 558.4 | 396.2 (376.3–506.3) · p95 482.7 | 365.6 (361.8–516.2) · p95 561.7 |
| wasm | abi.dispatch_view | abi.encode_args | 0.3 (0.3–0.3) · p95 0.7 | 0.3 (0.3–0.5) · p95 0.7 | 0.3 (0.3–0.4) · p95 0.6 | 0.3 (0.3–0.3) · p95 0.6 |
| wasm | abi.dispatch_view | abi.exec | 204.0 (188.0–235.5) · p95 236.2 | 232.9 (184.8–243.3) · p95 274.4 | 214.8 (184.6–256.3) · p95 236.0 | 212.1 (200.3–217.1) · p95 272.9 |
| wasm | abi.dispatch_view | abi.decode | 25.1 (21.3–26.4) · p95 32.0 | 26.1 (21.7–29.7) · p95 32.1 | 24.2 (21.1–31.9) · p95 28.8 | 25.3 (24.4–27.3) · p95 38.0 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.4 | 0.1 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.4 |
| wasm | abi.dispatch_view | abi.dispatch_view | 231.8 (210.2–262.6) · p95 277.9 | 260.4 (207.3–275.5) · p95 317.1 | 240.2 (206.5–291.1) · p95 284.4 | 242.5 (226.3–243.8) · p95 313.9 |
| wasm | read | raw.view | 132.6 (115.8–137.9) · p95 155.2 | 118.5 (111.0–133.9) · p95 153.9 | 118.4 (112.6–132.6) · p95 153.1 | 113.5 (105.7–129.4) · p95 147.8 |
| wasm | read | js.parse_view | 187.3 (162.3–193.0) · p95 211.1 | 167.4 (158.3–184.5) · p95 208.2 | 163.2 (156.7–183.7) · p95 200.8 | 165.1 (158.1–183.3) · p95 201.4 |
| wasm | read | raw.snapshot | 1329.2 (1121.9–1373.2) · p95 1472.5 | 1136.7 (1068.5–1296.2) · p95 1441.9 | 1106.8 (1101.0–1318.3) · p95 1453.0 | 1137.6 (1076.4–1319.6) · p95 1431.0 |
| wasm | read | js.parse_snapshot | 943.8 (817.7–985.9) · p95 1028.9 | 835.4 (793.0–927.7) · p95 1065.0 | 800.3 (789.4–928.5) · p95 1009.3 | 817.1 (783.6–927.2) · p95 992.7 |
| wasm | read | raw.save | 57.5 (50.3–65.5) · p95 71.0 | 49.0 (46.5–56.5) · p95 77.6 | 48.2 (46.3–57.3) · p95 76.6 | 45.2 (44.6–56.2) · p95 70.3 |
| wasm | kit | kit.dispatch | 1542.6 (1542.5–1702.8) · p95 1985.2 | 1835.9 (1636.4–1949.0) · p95 2306.5 | 1509.4 (1503.4–1616.7) · p95 1868.6 | - |
| wasm | kit | kit.view | 276.7 (273.9–304.2) · p95 375.2 | 318.6 (287.3–326.9) · p95 479.0 | 267.2 (263.9–279.3) · p95 356.6 | - |
| wasm | kit | kit.dispatch+view | 1833.1 (1821.8–2024.5) · p95 2482.3 | 2185.8 (1931.4–2281.8) · p95 3066.5 | 1784.7 (1771.1–1903.6) · p95 2401.9 | - |
| wasm | lifecycle | raw.new | 109056.3 (101282.6–119907.5) · p95 121539.6 | 108181.7 (101706.9–120236.4) · p95 111882.3 | 102352.5 (96027.9–116226.5) · p95 118961.6 | 93211.6 (91083.4–106557.2) · p95 110277.8 |
| wasm | lifecycle | raw.save | 136.5 (116.6–160.5) · p95 197.0 | 120.5 (118.6–142.6) · p95 165.1 | 112.3 (107.5–149.8) · p95 147.6 | 116.5 (103.5–119.2) · p95 165.6 |
| wasm | lifecycle | raw.restore | 109230.8 (103531.7–121530.5) · p95 127540.0 | 106375.7 (101307.1–118276.6) · p95 109867.0 | 103096.4 (96502.7–115721.8) · p95 118942.6 | 93522.2 (90729.2–104740.2) · p95 111542.2 |
| wasm | lifecycle | kit.open | 105845.8 (100802.4–121897.3) · p95 120939.9 | 105755.9 (99214.8–124849.1) · p95 108482.1 | 98641.8 (95405.7–114882.5) · p95 114688.5 | - |
| wasm | lifecycle | kit.save | 203.8 (198.1–276.3) · p95 298.7 | 205.7 (203.6–237.7) · p95 254.9 | 193.1 (190.9–255.5) · p95 231.4 | - |
| wasm | lifecycle | kit.restore | 109199.5 (99214.9–116679.8) · p95 119666.0 | 105860.1 (103153.2–118806.9) · p95 113496.5 | 98092.0 (94779.8–116296.0) · p95 117683.5 | - |

## ledger-session

The agent ledger replaying the real 2026-09-24 session log: 20 events, five of them refused by policy. Played 100 times from a fresh session. Program `experiments/agent-ledger/ledger.cav` (`2206ac79f23c`), stream `ef2143311f86`, 1 episode(s) × 100 = 2000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 19.7 (18.1–24.0) · p95 29.4 | 20.0 (17.5–24.6) · p95 29.1 | - |
| native | dispatch_view_json | dispatch_view_json | - | 31.3 (23.1–32.1) · p95 43.8 | 25.3 (23.1–32.3) · p95 35.5 | - |
| native | dispatch_json | dispatch_json | - | 45.0 (40.4–57.3) · p95 59.1 | 41.9 (40.3–57.2) · p95 52.7 | - |
| native | dispatch_outcome_json | dispatch_outcome_json | - | 44.9 (40.8–59.2) · p95 57.8 | 42.4 (40.5–57.0) · p95 53.4 | - |
| native | web.dispatch_view | web.dispatch_view | - | 29.0 (24.8–37.3) · p95 44.6 | 26.4 (25.0–38.4) · p95 35.4 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 65.1 (61.8–88.7) · p95 87.2 | 62.3 (60.0–92.8) · p95 81.6 | - |
| native | web.dispatch | web.dispatch | - | 83.5 (80.1–111.7) · p95 107.3 | 79.8 (73.7–113.1) · p95 97.5 | - |
| native | read | clone | - | 2.1 (2.0–2.7) · p95 2.4 | 2.4 (2.1–2.7) · p95 3.2 | - |
| native | read | clone.drop | - | 1.2 (1.2–1.5) · p95 1.3 | 1.4 (1.2–1.6) · p95 1.8 | - |
| native | read | save | - | 7.8 (7.5–9.8) · p95 10.7 | 8.9 (7.8–10.4) · p95 13.4 | - |
| native | read | save_json | - | 14.1 (13.5–17.9) · p95 20.0 | 15.9 (14.0–18.7) · p95 24.3 | - |
| native | read | snapshot | - | 21.6 (20.6–27.1) · p95 25.1 | 24.2 (21.4–28.0) · p95 33.1 | - |
| native | read | snapshot.drop | - | 8.7 (8.4–10.9) · p95 10.0 | 9.7 (8.5–11.2) · p95 12.5 | - |
| native | read | snapshot.serialize | - | 16.4 (16.0–20.9) · p95 19.9 | 18.8 (16.4–21.6) · p95 27.1 | - |
| native | read | snapshot.serialize_pretty | - | 32.8 (31.9–41.8) · p95 39.2 | 37.4 (32.4–43.1) · p95 51.7 | - |
| native | read | view | - | 3.9 (3.8–4.9) · p95 5.5 | 4.4 (3.8–5.0) · p95 6.7 | - |
| native | read | view.serialize | - | 2.8 (2.7–3.6) · p95 3.6 | 3.2 (2.8–3.7) · p95 4.6 | - |
| native | web.read | web.save | - | 16.9 (15.5–19.6) · p95 23.0 | 16.0 (15.7–21.6) · p95 22.0 | - |
| native | web.read | web.snapshot | - | 69.0 (63.9–80.3) · p95 83.1 | 65.3 (64.1–86.7) · p95 78.1 | - |
| native | web.read | web.view | - | 7.0 (6.5–8.1) · p95 10.1 | 6.5 (6.5–8.7) · p95 9.4 | - |
| native | lifecycle | from_source | - | 1570.1 (1561.4–2056.8) · p95 1728.5 | 1604.8 (1490.6–2032.3) · p95 1672.3 | - |
| native | lifecycle | restore | - | 1717.6 (1639.7–2095.5) · p95 1785.1 | 1635.8 (1571.3–2109.0) · p95 1768.3 | - |
| native | lifecycle | restore.parse | - | 40.1 (37.0–49.0) · p95 48.1 | 38.7 (30.0–48.9) · p95 50.6 | - |
| native | lifecycle | restore_json | - | 1706.9 (1660.7–2143.3) · p95 1802.6 | 1665.5 (1536.6–2146.9) · p95 1807.2 | - |
| native | lifecycle | save | - | 27.1 (22.8–30.3) · p95 39.9 | 23.8 (19.7–28.9) · p95 32.3 | - |
| native | lifecycle | save_json | - | 30.5 (27.7–36.5) · p95 37.5 | 27.9 (25.3–35.8) · p95 37.1 | - |
| native | lifecycle | web.new | - | 1598.1 (1562.3–2041.9) · p95 1712.8 | 1568.1 (1474.8–2007.0) · p95 1704.1 | - |
| native | lifecycle | web.restore | - | 1703.8 (1681.2–2142.6) · p95 1835.0 | 1653.3 (1565.3–2121.9) · p95 1823.3 | - |
| native | lifecycle | web.save | - | 23.3 (21.4–28.1) · p95 27.6 | 22.8 (19.1–27.6) · p95 27.0 | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.4 (0.3–0.5) · p95 1.7 | 0.4 (0.3–0.5) · p95 1.6 | 0.5 (0.3–0.5) · p95 1.7 | 0.4 (0.3–0.5) · p95 1.6 |
| wasm | raw.dispatch_view | raw.dispatch_view | 38.9 (30.7–44.6) · p95 54.6 | 36.1 (30.6–44.2) · p95 57.0 | 39.9 (30.9–44.1) · p95 55.4 | 37.5 (30.7–42.0) · p95 53.8 |
| wasm | raw.dispatch_view | js.parse_view | 9.2 (7.5–10.6) · p95 14.8 | 9.1 (7.5–10.5) · p95 15.6 | 10.2 (7.8–10.8) · p95 15.4 | 9.8 (7.9–10.3) · p95 15.0 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 51.3 (41.6–59.1) · p95 71.8 | 49.5 (41.2–58.7) · p95 73.2 | 55.8 (42.1–57.3) · p95 72.7 | 51.6 (42.5–55.6) · p95 70.0 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 83.7 (72.5–93.6) · p95 105.5 | 86.1 (72.7–94.9) · p95 118.7 | 86.8 (73.3–94.1) · p95 116.6 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 59.4 (52.7–66.7) · p95 73.6 | 59.9 (51.5–69.0) · p95 83.3 | 60.9 (51.9–68.1) · p95 80.6 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 143.2 (125.8–161.0) · p95 178.1 | 145.5 (124.7–164.9) · p95 202.4 | 147.7 (126.0–162.7) · p95 199.6 | - |
| wasm | raw.dispatch | raw.dispatch | 105.2 (93.4–121.1) · p95 141.3 | 107.2 (94.9–122.3) · p95 129.4 | 107.3 (99.6–120.9) · p95 130.8 | 104.4 (103.3–120.2) · p95 127.5 |
| wasm | abi.dispatch_view | abi.encode_args | 0.3 (0.3–0.4) · p95 0.8 | 0.3 (0.3–0.4) · p95 0.8 | 0.3 (0.3–0.4) · p95 0.8 | 0.3 (0.3–0.4) · p95 0.8 |
| wasm | abi.dispatch_view | abi.exec | 35.2 (32.6–40.6) · p95 47.8 | 35.0 (34.9–40.8) · p95 49.2 | 35.2 (34.7–40.1) · p95 47.6 | 33.1 (30.9–38.4) · p95 45.4 |
| wasm | abi.dispatch_view | abi.decode | 0.8 (0.7–0.9) · p95 2.5 | 0.8 (0.8–0.9) · p95 2.5 | 0.8 (0.7–0.8) · p95 2.6 | 0.7 (0.7–0.8) · p95 2.4 |
| wasm | abi.dispatch_view | abi.free | 0.2 (0.2–0.2) · p95 0.3 | 0.2 (0.2–0.2) · p95 0.3 | 0.2 (0.1–0.2) · p95 0.3 | 0.2 (0.2–0.2) · p95 0.3 |
| wasm | abi.dispatch_view | abi.dispatch_view | 39.9 (36.3–45.5) · p95 51.2 | 39.6 (38.7–45.9) · p95 53.1 | 40.1 (38.7–44.9) · p95 50.7 | 39.1 (35.7–44.0) · p95 48.9 |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.4 (0.4–0.4) · p95 0.9 | 0.4 (0.3–0.4) · p95 0.9 | 0.4 (0.3–0.5) · p95 1.0 | - |
| wasm | abi.dispatch_outcome | abi.exec | 80.7 (73.5–88.2) · p95 106.1 | 73.8 (73.6–88.9) · p95 101.0 | 76.8 (71.9–89.0) · p95 110.6 | - |
| wasm | abi.dispatch_outcome | abi.decode | 2.8 (2.7–2.9) · p95 9.3 | 2.8 (2.7–2.9) · p95 9.3 | 2.8 (2.7–2.9) · p95 9.6 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.4 (0.4–0.5) · p95 0.8 | 0.4 (0.4–0.5) · p95 0.8 | 0.4 (0.4–0.4) · p95 0.8 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 85.4 (77.8–92.7) · p95 112.7 | 78.0 (77.9–93.6) · p95 108.1 | 81.4 (76.1–93.6) · p95 118.1 | - |
| wasm | read | raw.view | 9.1 (8.5–10.9) · p95 13.1 | 10.3 (8.5–10.6) · p95 15.0 | 9.6 (9.2–10.9) · p95 15.2 | 9.4 (8.2–10.1) · p95 14.0 |
| wasm | read | js.parse_view | 10.1 (9.3–12.0) · p95 14.2 | 11.4 (9.0–11.8) · p95 16.1 | 10.5 (10.0–12.0) · p95 16.9 | 11.6 (9.9–11.9) · p95 16.5 |
| wasm | read | raw.snapshot | 85.9 (79.7–103.0) · p95 114.3 | 95.6 (78.4–99.4) · p95 134.8 | 88.7 (85.2–102.3) · p95 123.3 | 95.2 (83.8–99.6) · p95 121.9 |
| wasm | read | js.parse_snapshot | 65.5 (60.3–79.6) · p95 84.6 | 72.7 (58.5–77.4) · p95 99.1 | 67.2 (64.0–78.6) · p95 93.3 | 74.0 (64.3–79.5) · p95 95.2 |
| wasm | read | raw.save | 21.0 (19.2–25.1) · p95 29.9 | 22.9 (19.0–24.7) · p95 33.6 | 21.5 (21.0–25.2) · p95 33.3 | 21.5 (19.7–23.7) · p95 32.2 |
| wasm | abi.read | abi.view.exec | 8.0 (7.7–10.3) · p95 12.6 | 8.2 (7.4–9.9) · p95 11.5 | 7.9 (7.2–10.3) · p95 10.5 | 7.5 (7.0–9.4) · p95 10.3 |
| wasm | abi.read | abi.view.decode | 0.4 (0.4–0.5) · p95 0.8 | 0.4 (0.4–0.5) · p95 0.7 | 0.4 (0.4–0.5) · p95 0.6 | 0.4 (0.4–0.5) · p95 0.7 |
| wasm | abi.read | abi.snapshot.exec | 72.9 (68.8–94.9) · p95 109.4 | 74.0 (66.8–92.0) · p95 99.3 | 72.5 (66.2–93.4) · p95 84.2 | 71.7 (67.5–91.0) · p95 92.1 |
| wasm | abi.read | abi.snapshot.decode | 4.2 (4.0–4.3) · p95 6.8 | 4.3 (4.0–4.3) · p95 7.0 | 4.2 (3.9–4.3) · p95 6.8 | 4.2 (4.1–4.3) · p95 7.2 |
| wasm | abi.read | abi.save.exec | 17.7 (16.9–22.9) · p95 28.3 | 18.0 (16.4–22.2) · p95 25.8 | 17.6 (16.0–23.0) · p95 23.3 | 17.1 (15.6–21.5) · p95 23.9 |
| wasm | abi.read | abi.save.decode | 0.7 (0.7–0.9) · p95 1.4 | 0.8 (0.7–0.8) · p95 1.2 | 0.7 (0.7–0.8) · p95 1.2 | 0.8 (0.7–0.8) · p95 1.2 |
| wasm | kit | kit.dispatch | 138.0 (130.8–168.3) · p95 169.1 | 137.8 (137.3–165.9) · p95 184.4 | 140.5 (135.4–168.6) · p95 211.0 | - |
| wasm | kit | kit.view | 17.9 (17.1–21.9) · p95 22.3 | 18.0 (17.9–21.7) · p95 24.4 | 18.3 (18.1–21.8) · p95 27.6 | - |
| wasm | kit | kit.dispatch+view | 155.6 (147.5–189.6) · p95 190.2 | 155.1 (154.7–187.2) · p95 208.1 | 158.4 (152.3–189.7) · p95 239.2 | - |
| wasm | kit.read | kit.view | 19.3 (16.7–24.2) · p95 23.8 | 19.3 (18.5–24.0) · p95 25.0 | 19.6 (18.4–23.5) · p95 27.1 | - |
| wasm | kit.read | kit.snapshot | 139.8 (122.4–175.1) · p95 163.2 | 139.4 (134.9–175.8) · p95 176.6 | 142.3 (134.1–169.4) · p95 198.6 | - |
| wasm | kit.read | kit.save | 37.0 (32.2–45.9) · p95 48.5 | 36.9 (34.4–46.1) · p95 50.7 | 37.4 (34.8–44.9) · p95 52.8 | - |
| wasm | lifecycle | raw.new | 1730.1 (1530.7–2074.3) · p95 1864.2 | 1696.1 (1692.5–2156.6) · p95 2285.8 | 1687.5 (1644.3–2121.0) · p95 2192.5 | 1812.2 (1725.0–2066.5) · p95 2605.6 |
| wasm | lifecycle | raw.save | 55.1 (49.9–79.4) · p95 84.9 | 63.1 (54.4–78.7) · p95 93.3 | 56.7 (56.0–84.0) · p95 82.8 | 58.4 (57.1–67.8) · p95 88.4 |
| wasm | lifecycle | raw.restore | 1833.7 (1658.9–2278.4) · p95 2225.9 | 1875.3 (1816.5–2298.7) · p95 2568.9 | 1864.4 (1770.9–2267.6) · p95 2328.2 | 1899.5 (1773.3–2079.8) · p95 2671.9 |
| wasm | lifecycle | kit.open | 1764.5 (1604.3–2196.2) · p95 2202.5 | 1776.5 (1729.0–2202.2) · p95 2272.8 | 1757.1 (1635.1–2185.2) · p95 2266.7 | - |
| wasm | lifecycle | kit.save | 108.6 (86.5–155.4) · p95 149.6 | 119.4 (100.2–168.7) · p95 179.2 | 114.4 (102.7–169.1) · p95 165.3 | - |
| wasm | lifecycle | kit.restore | 1792.5 (1643.2–2280.3) · p95 2037.7 | 1832.7 (1784.5–2286.3) · p95 2369.8 | 1830.8 (1759.9–2268.4) · p95 2314.5 | - |
| wasm | micro | timer.hrtime_pair | 0.1 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.3 | 0.2 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.2 |
| wasm | micro | js.stringify_payload.first | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | micro | js.stringify_payload.last | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | micro | bridge.pass_ascii.payload | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | micro | kit.payloadText.last | 0.2 (0.2–0.2) · p95 0.2 | 0.2 (0.2–0.2) · p95 0.3 | 0.2 (0.2–0.2) · p95 0.2 | - |
| wasm | micro | bridge.decode.view | 0.4 (0.4–0.5) · p95 0.6 | 0.4 (0.4–0.5) · p95 0.6 | 0.4 (0.3–0.5) · p95 0.6 | 0.4 (0.4–0.4) · p95 0.6 |
| wasm | micro | bridge.pass_ascii.view | 2.7 (2.6–3.3) · p95 2.9 | 3.0 (2.8–3.5) · p95 3.3 | 3.1 (2.4–3.3) · p95 3.4 | 2.5 (2.4–3.2) · p95 2.7 |
| wasm | micro | js.parse.view | 8.0 (7.9–9.7) · p95 9.7 | 8.6 (7.7–11.1) · p95 9.2 | 9.4 (6.3–9.8) · p95 10.0 | 9.1 (6.6–10.3) · p95 9.8 |
| wasm | micro | bridge.decode.outcome | 0.2 (0.2–0.2) · p95 0.3 | 0.2 (0.2–0.2) · p95 0.3 | 0.2 (0.2–0.2) · p95 0.3 | - |
| wasm | micro | bridge.pass_ascii.outcome | 0.2 (0.2–0.2) · p95 0.2 | 0.2 (0.2–0.3) · p95 0.3 | 0.2 (0.2–0.3) · p95 0.3 | - |
| wasm | micro | js.parse.outcome | 0.4 (0.3–0.4) · p95 0.4 | 0.4 (0.4–0.5) · p95 0.5 | 0.4 (0.3–0.5) · p95 0.5 | - |
| wasm | micro | bridge.decode.snapshot | 2.4 (2.1–3.3) · p95 14.4 | 2.5 (2.4–3.6) · p95 17.1 | 2.6 (2.1–3.2) · p95 14.6 | 2.7 (2.2–3.0) · p95 3.2 |
| wasm | micro | bridge.pass_ascii.snapshot | 34.4 (30.7–40.6) · p95 38.0 | 36.0 (34.5–45.8) · p95 39.5 | 39.6 (30.9–42.4) · p95 43.2 | 40.5 (33.0–43.0) · p95 44.2 |
| wasm | micro | js.parse.snapshot | 61.0 (55.7–71.8) · p95 70.7 | 65.8 (63.8–70.4) · p95 73.2 | 69.9 (56.8–73.1) · p95 75.9 | 70.4 (68.8–76.6) · p95 78.2 |
| wasm | micro | bridge.decode.save | 0.6 (0.5–0.7) · p95 0.8 | 0.8 (0.6–0.8) · p95 1.0 | 0.7 (0.5–0.8) · p95 0.9 | 0.7 (0.6–0.7) · p95 0.8 |
| wasm | micro | bridge.pass_ascii.save | 6.2 (5.1–6.9) · p95 6.7 | 6.0 (5.3–6.9) · p95 6.6 | 6.8 (5.5–7.1) · p95 7.2 | 6.9 (6.7–7.1) · p95 7.2 |
| wasm | micro | js.parse.save | 19.2 (18.6–20.1) · p95 21.7 | 18.6 (18.1–19.3) · p95 20.4 | 20.3 (15.6–22.3) · p95 25.7 | 20.1 (18.9–21.6) · p95 25.0 |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 60.4 (59.8–72.4) · p95 154.8 | 68.0 (64.3–74.6) · p95 189.1 | - |
| native | dispatch_view_json | dispatch_view_json | - | 74.7 (59.8–78.8) · p95 186.3 | 65.8 (59.5–80.4) · p95 159.1 | - |
| native | dispatch_json | dispatch_json | - | 102.4 (92.4–131.9) · p95 196.2 | 111.3 (94.8–133.9) · p95 220.5 | - |
| native | dispatch_outcome_json | dispatch_outcome_json | - | 105.1 (92.1–132.5) · p95 204.6 | 112.0 (99.0–136.4) · p95 215.8 | - |
| native | web.dispatch_view | web.dispatch_view | - | 80.3 (78.6–97.5) · p95 177.8 | 75.6 (75.1–92.8) · p95 175.4 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 151.7 (142.5–182.3) · p95 258.7 | 148.6 (144.3–186.8) · p95 257.6 | - |
| native | web.dispatch | web.dispatch | - | 166.9 (163.9–209.1) · p95 267.6 | 170.4 (167.3–209.5) · p95 273.8 | - |
| native | read | clone | - | 14.0 (13.8–14.3) · p95 19.8 | 13.2 (12.8–13.7) · p95 18.0 | - |
| native | read | clone.drop | - | 7.6 (7.4–7.7) · p95 10.6 | 7.1 (6.9–7.4) · p95 9.5 | - |
| native | read | save | - | 7.6 (7.4–7.6) · p95 13.8 | 7.1 (7.1–7.4) · p95 12.8 | - |
| native | read | save_json | - | 12.8 (12.5–13.1) · p95 23.2 | 12.1 (11.9–12.2) · p95 21.3 | - |
| native | read | snapshot | - | 40.9 (40.4–41.3) · p95 57.5 | 38.3 (37.7–39.9) · p95 52.1 | - |
| native | read | snapshot.drop | - | 16.3 (16.0–16.6) · p95 23.0 | 15.3 (14.9–15.9) · p95 20.4 | - |
| native | read | snapshot.serialize | - | 25.6 (25.3–25.8) · p95 36.6 | 24.2 (23.7–25.2) · p95 33.1 | - |
| native | read | snapshot.serialize_pretty | - | 51.2 (50.6–51.6) · p95 71.8 | 48.2 (47.3–50.3) · p95 64.4 | - |
| native | read | view | - | 1.8 (1.8–1.9) · p95 3.3 | 1.7 (1.7–1.7) · p95 3.0 | - |
| native | read | view.serialize | - | 9.0 (8.9–9.2) · p95 14.4 | 8.6 (8.4–8.8) · p95 13.1 | - |
| native | web.read | web.save | - | 13.2 (12.8–15.8) · p95 23.3 | 16.0 (13.2–16.3) · p95 26.5 | - |
| native | web.read | web.snapshot | - | 93.4 (93.2–115.6) · p95 132.2 | 112.9 (95.7–117.4) · p95 137.4 | - |
| native | web.read | web.view | - | 7.4 (7.2–8.8) · p95 11.4 | 9.0 (7.5–9.0) · p95 12.9 | - |
| native | lifecycle | from_source | - | 4285.3 (3544.5–4340.2) · p95 4413.7 | 4325.2 (3293.7–4386.4) · p95 4474.0 | - |
| native | lifecycle | restore | - | 4456.8 (3443.8–4537.8) · p95 4542.7 | 4488.2 (3415.9–4690.4) · p95 4629.1 | - |
| native | lifecycle | restore.parse | - | 37.6 (29.0–40.3) · p95 39.9 | 37.3 (29.6–42.1) · p95 44.5 | - |
| native | lifecycle | restore_json | - | 4548.0 (3453.5–4566.9) · p95 4671.1 | 4530.2 (3427.4–4669.7) · p95 4650.1 | - |
| native | lifecycle | save | - | 24.6 (19.2–29.9) · p95 32.7 | 26.0 (20.0–29.0) · p95 34.5 | - |
| native | lifecycle | save_json | - | 24.7 (24.7–25.8) · p95 30.5 | 23.5 (19.0–24.4) · p95 25.3 | - |
| native | lifecycle | web.new | - | 4304.8 (3508.1–4327.0) · p95 4392.2 | 4304.4 (3276.0–4466.6) · p95 4412.9 | - |
| native | lifecycle | web.restore | - | 4486.7 (3446.5–4539.3) · p95 4666.1 | 4502.3 (3464.0–4535.9) · p95 4596.5 | - |
| native | lifecycle | web.save | - | 18.9 (15.1–20.8) · p95 21.8 | 17.6 (15.1–20.0) · p95 19.4 | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.8 (0.6–0.8) · p95 3.3 | 0.7 (0.7–0.9) · p95 2.8 | 0.7 (0.6–0.8) · p95 2.8 | - |
| wasm | raw.dispatch_view | raw.dispatch_view | 95.2 (82.4–111.1) · p95 199.1 | 87.6 (86.5–111.9) · p95 181.1 | 92.2 (81.7–104.7) · p95 188.9 | - |
| wasm | raw.dispatch_view | js.parse_view | 33.2 (28.5–39.3) · p95 43.1 | 29.9 (29.4–39.3) · p95 42.5 | 32.4 (28.5–36.8) · p95 43.5 | - |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 158.4 (134.5–186.7) · p95 243.1 | 143.0 (138.8–185.2) · p95 222.8 | 153.3 (134.6–174.8) · p95 230.8 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 192.9 (169.4–213.4) · p95 326.1 | 185.4 (165.4–210.8) · p95 302.7 | 184.1 (163.7–211.6) · p95 314.5 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 99.7 (88.2–113.2) · p95 141.0 | 96.9 (87.4–112.8) · p95 126.3 | 96.4 (83.3–112.4) · p95 139.5 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 297.8 (260.9–331.5) · p95 465.3 | 285.8 (255.6–328.1) · p95 423.8 | 282.8 (250.1–328.3) · p95 445.1 | - |
| wasm | raw.dispatch | raw.dispatch | 210.2 (181.0–250.3) · p95 329.5 | 195.1 (192.5–226.0) · p95 301.9 | 200.4 (187.0–253.4) · p95 309.9 | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.5 (0.4–0.6) · p95 1.4 | 0.5 (0.4–0.6) · p95 1.4 | 0.6 (0.4–0.6) · p95 1.5 | - |
| wasm | abi.dispatch_view | abi.exec | 91.7 (74.5–108.5) · p95 191.2 | 96.7 (72.9–113.2) · p95 202.4 | 102.3 (72.7–102.6) · p95 208.5 | - |
| wasm | abi.dispatch_view | abi.decode | 1.6 (1.3–1.8) · p95 6.7 | 1.6 (1.2–1.9) · p95 6.0 | 1.7 (1.3–1.8) · p95 7.0 | - |
| wasm | abi.dispatch_view | abi.free | 0.2 (0.2–0.3) · p95 0.5 | 0.3 (0.2–0.3) · p95 0.5 | 0.2 (0.2–0.2) · p95 0.5 | - |
| wasm | abi.dispatch_view | abi.dispatch_view | 122.0 (99.9–145.0) · p95 199.9 | 128.8 (98.5–152.6) · p95 210.8 | 136.1 (98.7–136.3) · p95 215.8 | - |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.6 (0.5–0.7) · p95 1.6 | 0.6 (0.4–0.6) · p95 1.5 | 0.5 (0.4–0.7) · p95 1.5 | - |
| wasm | abi.dispatch_outcome | abi.exec | 174.8 (136.2–181.3) · p95 282.1 | 167.4 (135.4–182.8) · p95 279.7 | 167.2 (135.9–209.9) · p95 270.8 | - |
| wasm | abi.dispatch_outcome | abi.decode | 3.4 (3.2–4.1) · p95 10.3 | 4.0 (3.2–4.0) · p95 9.8 | 3.9 (3.2–4.2) · p95 10.1 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.4 (0.3–0.5) · p95 0.9 | 0.5 (0.4–0.5) · p95 0.8 | 0.4 (0.4–0.5) · p95 0.8 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 181.6 (141.9–188.9) · p95 293.2 | 174.0 (140.6–189.6) · p95 290.7 | 173.6 (141.8–217.2) · p95 281.7 | - |
| wasm | read | raw.view | 12.8 (10.4–13.6) · p95 18.3 | 12.6 (11.3–13.5) · p95 18.9 | 13.3 (10.5–14.4) · p95 19.6 | - |
| wasm | read | js.parse_view | 33.2 (26.5–35.6) · p95 46.8 | 33.5 (28.6–36.1) · p95 48.2 | 34.2 (27.9–36.5) · p95 48.1 | - |
| wasm | read | raw.snapshot | 134.2 (109.7–145.3) · p95 181.4 | 135.7 (116.8–140.1) · p95 195.2 | 137.4 (112.9–150.2) · p95 192.0 | - |
| wasm | read | js.parse_snapshot | 101.4 (83.1–110.5) · p95 135.6 | 102.5 (88.3–109.1) · p95 145.0 | 104.7 (86.0–113.7) · p95 139.3 | - |
| wasm | read | raw.save | 18.6 (15.1–20.0) · p95 33.1 | 18.6 (16.3–20.0) · p95 33.5 | 19.5 (15.7–20.7) · p95 35.4 | - |
| wasm | abi.read | abi.view.exec | 13.1 (10.5–14.3) · p95 17.9 | 12.1 (10.1–14.6) · p95 18.5 | 13.2 (10.5–14.2) · p95 18.1 | - |
| wasm | abi.read | abi.view.decode | 1.2 (1.0–1.2) · p95 2.1 | 1.1 (0.9–1.2) · p95 2.1 | 1.1 (1.0–1.2) · p95 2.1 | - |
| wasm | abi.read | abi.snapshot.exec | 141.3 (110.3–153.8) · p95 171.4 | 126.8 (107.5–156.0) · p95 183.7 | 142.0 (112.8–149.7) · p95 171.3 | - |
| wasm | abi.read | abi.snapshot.decode | 5.7 (5.1–5.8) · p95 10.1 | 5.1 (5.0–5.7) · p95 9.6 | 5.5 (5.1–5.6) · p95 9.6 | - |
| wasm | abi.read | abi.save.exec | 19.6 (15.5–21.4) · p95 33.4 | 17.6 (14.7–21.3) · p95 32.3 | 19.4 (15.5–20.6) · p95 33.8 | - |
| wasm | abi.read | abi.save.decode | 0.6 (0.5–0.7) · p95 1.3 | 0.6 (0.5–0.7) · p95 1.3 | 0.6 (0.5–0.7) · p95 1.3 | - |
| wasm | kit | kit.dispatch | 302.3 (269.2–327.6) · p95 454.0 | 302.0 (292.8–309.1) · p95 442.2 | 300.3 (251.7–331.4) · p95 452.5 | - |
| wasm | kit | kit.view | 46.3 (41.1–49.8) · p95 65.6 | 45.7 (44.1–47.2) · p95 63.1 | 45.6 (38.5–50.1) · p95 65.3 | - |
| wasm | kit | kit.dispatch+view | 349.7 (312.3–382.8) · p95 514.5 | 350.6 (339.4–361.1) · p95 500.5 | 348.6 (292.3–387.6) · p95 511.5 | - |
| wasm | kit.read | kit.view | 42.0 (38.3–43.2) · p95 59.7 | 43.6 (39.3–46.7) · p95 58.9 | 45.2 (39.2–49.5) · p95 64.1 | - |
| wasm | kit.read | kit.snapshot | 219.0 (199.8–224.6) · p95 312.7 | 226.0 (204.3–240.4) · p95 301.9 | 232.1 (204.0–258.4) · p95 322.7 | - |
| wasm | kit.read | kit.save | 33.7 (30.7–35.2) · p95 59.8 | 35.5 (31.1–37.8) · p95 60.0 | 36.6 (31.6–40.2) · p95 63.2 | - |
| wasm | lifecycle | raw.new | 3877.5 (2907.3–4311.2) · p95 4141.5 | 3412.6 (3370.5–4595.2) · p95 3937.3 | 3814.0 (3618.9–4446.4) · p95 4032.6 | - |
| wasm | lifecycle | raw.save | 66.0 (39.5–69.8) · p95 107.7 | 53.7 (44.6–71.7) · p95 83.5 | 66.4 (52.0–73.1) · p95 92.3 | - |
| wasm | lifecycle | raw.restore | 3886.4 (2905.1–4368.9) · p95 4064.1 | 3387.9 (3206.5–4605.6) · p95 3933.8 | 3835.5 (3717.1–4444.1) · p95 4101.4 | - |
| wasm | lifecycle | kit.open | 3752.2 (2873.4–4228.7) · p95 3909.2 | 3377.5 (3275.5–4400.8) · p95 3914.7 | 3780.6 (3543.2–4357.0) · p95 3994.3 | - |
| wasm | lifecycle | kit.save | 133.4 (80.1–143.3) · p95 152.6 | 118.3 (87.7–135.0) · p95 138.4 | 131.8 (104.1–150.0) · p95 172.0 | - |
| wasm | lifecycle | kit.restore | 3797.2 (2908.8–4270.0) · p95 3957.7 | 3460.6 (3406.0–4333.4) · p95 3992.4 | 3778.9 (3629.7–4449.8) · p95 4086.3 | - |
| wasm | micro | timer.hrtime_pair | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.2) · p95 0.2 | 0.1 (0.1–0.2) · p95 0.2 | - |
| wasm | micro | js.stringify_payload.first | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | - |
| wasm | micro | js.stringify_payload.last | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | - |
| wasm | micro | bridge.pass_ascii.payload | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | - |
| wasm | micro | kit.payloadText.last | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.0–0.1) · p95 0.1 | - |
| wasm | micro | bridge.decode.view | 0.6 (0.6–0.7) · p95 0.8 | 0.6 (0.5–0.7) · p95 0.8 | 0.6 (0.5–0.9) · p95 0.8 | - |
| wasm | micro | bridge.pass_ascii.view | 5.7 (5.4–6.7) · p95 5.9 | 5.9 (5.4–6.4) · p95 6.2 | 6.8 (5.0–8.3) · p95 7.3 | - |
| wasm | micro | js.parse.view | 21.8 (20.6–25.6) · p95 26.2 | 21.7 (20.3–26.7) · p95 26.6 | 21.5 (20.6–32.1) · p95 30.3 | - |
| wasm | micro | bridge.decode.outcome | 1.6 (1.5–2.2) · p95 2.8 | 1.5 (1.4–1.9) · p95 3.1 | 1.6 (1.5–2.2) · p95 3.4 | - |
| wasm | micro | bridge.pass_ascii.outcome | 22.4 (20.9–24.8) · p95 23.5 | 20.6 (18.4–24.2) · p95 21.6 | 22.6 (18.6–31.9) · p95 23.9 | - |
| wasm | micro | js.parse.outcome | 82.0 (71.2–82.4) · p95 90.0 | 70.6 (66.2–88.8) · p95 78.4 | 69.6 (66.9–82.1) · p95 76.9 | - |
| wasm | micro | bridge.decode.snapshot | 3.0 (2.6–3.4) · p95 4.4 | 2.7 (2.4–3.3) · p95 3.6 | 2.8 (2.5–3.4) · p95 3.2 | - |
| wasm | micro | bridge.pass_ascii.snapshot | 40.2 (31.4–40.5) · p95 43.9 | 35.3 (33.0–41.8) · p95 38.7 | 33.6 (32.5–42.2) · p95 37.3 | - |
| wasm | micro | js.parse.snapshot | 87.5 (70.0–94.1) · p95 98.6 | 75.4 (74.7–96.3) · p95 82.8 | 74.0 (69.9–90.4) · p95 83.2 | - |
| wasm | micro | bridge.decode.save | 0.3 (0.3–0.4) · p95 0.4 | 0.3 (0.3–0.4) · p95 0.4 | 0.3 (0.3–0.3) · p95 0.3 | - |
| wasm | micro | bridge.pass_ascii.save | 2.8 (2.2–2.8) · p95 2.8 | 2.4 (2.3–2.8) · p95 2.5 | 2.3 (2.2–2.8) · p95 2.4 | - |
| wasm | micro | js.parse.save | 9.0 (7.2–9.5) · p95 10.0 | 7.7 (7.6–9.7) · p95 8.3 | 7.6 (7.2–9.0) · p95 8.3 | - |

