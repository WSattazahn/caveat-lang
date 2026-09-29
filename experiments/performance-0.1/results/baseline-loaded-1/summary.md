# Caveat performance baseline: 2026-09-29T02-18-48-081-baseline

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=baseline --repeats=3 --build=never --keep-samples --target=rc4-published=package:C:\Users\walte\caveat-rc4\package-files\package --target=rc4-local=tree:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF-rc4 --target=main-local=tree:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF --target=hist-e6ace96=runtime:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\perf-scratch\scout\hist-e6ace96\pkg-reactive --out=C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\perf-scratch\m\runs\baseline`

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
- Load before: total CPU 8/38/45%; busiest over the window: chrome 4.922s, claude 1.016s, powershell 0.797s, chrome 0.781s, ChatGPT 0.547s, codex 0.391s
- Load after: total CPU 7/7/5%; busiest over the window: ChatGPT 3.141s, ChatGPT 1.422s, claude 1.156s, powershell 0.703s, claude 0.406s, ChatGPT 0.281s
- Load during: typeperf every 5 s for the whole run, 742 samples: total mean 27.3% p95 70.2% max 91.2%; processor 10 mean 58.3% p95 99.4% max 100%; processor 11 mean 28.3% p95 80.7% max 100%; processor 12 mean 26.1% p95 81.9% max 100%; processor 13 mean 48.4% p95 99.4% max 100%; total above 10% in 707 and above 25% in 267 samples; pinned processors summing above 150% in 312

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
| Published method: adapter dispatch + view | 50.9 | 52.8 | 53.4 | 54.0 |
| Adapter dispatch (stringify, WebAssembly dispatch_view, JSON.parse) | 51.8 | 54.1 | 48.9 | 50.5 |
| Adapter view() (JavaScript reshaping only) | 2.6 | 2.7 | 2.4 | 2.7 |
| JSON.stringify(payload) | 0.4 | 0.4 | 0.4 | 0.5 |
| wasm-bindgen dispatch_view call, returning the view text | 36.4 | 29.7 | 28.9 | 34.2 |
|   arguments copied into WebAssembly memory | 0.1 | 0.1 | 0.1 | 0.1 |
|   WebAssembly execution (resolve, apply, view, serialize) | 25.1 | 24.8 | 24.7 | 23.9 |
|   result decoded to a JavaScript string | 2.8 | 2.8 | 2.9 | 2.8 |
|   result freed | 0.1 | 0.1 | 0.1 | 0.1 |
|   the four pieces, timed together in one process | 28.2 | 27.9 | 27.9 | 27.0 |
| JSON.parse(view text) | 24.1 | 19.5 | 19.1 | 23.5 |
| WebAssembly view() alone, execution only | 9.9 | 9.8 | 9.7 | 9.3 |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | 146.4 | 171.4 | 157.3 | - |
| Kit dispatch + view() | 177.1 | 207.7 | 189.8 | - |
| Native web::dispatch_view (the exported function, natively) | - | 24.4 | 27.2 | - |
| Native apply (numeric parameters: rules and bindings only) | - | 19.8 | 18.4 | - |
| Native dispatch_view_json (resolve + apply + view, not serialized) | - | 20.4 | 19.3 | - |
| Native view() build | - | 2.9 | 2.9 | - |
| Native view serialize | - | 5.3 | 5.6 | - |
| Native snapshot() build | - | 28.7 | 29.5 | - |
| Native session clone (upper bound of the transaction copy) | - | 8.2 | 8.4 | - |
| Native apply, the same program without bindings (glowcap-unbound) | - | 13.3 | 12.8 | - |
| *derived:* native resolve_payload ≈ dispatch_view_json − apply − view | - | -2.3 | -2.0 | - |
| *derived:* native binding evaluation ≈ apply − apply without bindings | - | 6.5 | 5.6 | - |
| *derived:* WebAssembly ÷ native, same exported function (ratio) | - | 1.0 | 0.9 | - |

## glowcap-replay

The Glowcap replay program and the exact event stream behind the published 51.6 us figure (experiments/glowcap/harness.mjs bench(), runtime/examples/profile_dispatch.rs): absorb cave glowcap, absorb pool duskcap, taste ruin glowcap, then 9,997 ticks of dt 0.05. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 20.0 (13.9–21.3) · p95 27.7 | 18.6 (15.2–18.7) · p95 24.0 | - |
| native | dispatch_view_json | dispatch_view_json | - | 20.5 (20.4–23.9) · p95 25.1 | 19.4 (16.3–22.0) · p95 22.6 | - |
| native | dispatch_json | dispatch_json | - | 42.7 (41.2–54.5) · p95 48.1 | 49.9 (44.1–50.7) · p95 55.0 | - |
| native | dispatch_outcome_json | dispatch_outcome_json | - | 46.9 (39.0–48.5) · p95 59.6 | 44.7 (37.0–53.8) · p95 51.2 | - |
| native | web.dispatch_view | web.dispatch_view | - | 24.5 (23.3–29.9) · p95 28.4 | 27.6 (25.7–29.8) · p95 36.1 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 88.7 (66.8–98.2) · p95 104.1 | 84.2 (74.3–98.2) · p95 98.0 | - |
| native | web.dispatch | web.dispatch | - | 93.4 (82.8–107.2) · p95 99.8 | 92.8 (87.9–121.9) · p95 174.6 | - |
| native | read | clone | - | 8.1 (7.8–10.1) · p95 9.5 | 8.2 (7.5–10.2) · p95 10.1 | - |
| native | read | clone.drop | - | 4.4 (4.3–5.6) · p95 5.2 | 4.5 (4.0–5.6) · p95 5.4 | - |
| native | read | save | - | 10.3 (9.9–12.7) · p95 12.1 | 10.7 (9.4–13.3) · p95 13.0 | - |
| native | read | save_json | - | 16.7 (16.2–20.9) · p95 19.6 | 17.1 (15.4–21.7) · p95 20.9 | - |
| native | read | snapshot | - | 28.6 (27.6–35.6) · p95 33.8 | 29.2 (25.8–36.6) · p95 35.5 | - |
| native | read | snapshot.drop | - | 12.3 (11.9–15.3) · p95 14.6 | 12.4 (11.1–15.7) · p95 15.1 | - |
| native | read | snapshot.serialize | - | 17.1 (16.6–21.6) · p95 20.4 | 17.6 (15.7–22.6) · p95 21.4 | - |
| native | read | snapshot.serialize_pretty | - | 36.0 (35.1–45.2) · p95 42.8 | 37.2 (33.2–47.4) · p95 45.0 | - |
| native | read | view | - | 2.9 (2.7–3.6) · p95 3.3 | 2.8 (2.5–3.6) · p95 3.5 | - |
| native | read | view.serialize | - | 5.3 (5.1–6.7) · p95 6.3 | 5.6 (4.9–7.0) · p95 6.8 | - |
| native | web.read | web.save | - | 18.3 (16.5–21.8) · p95 22.5 | 20.2 (16.8–23.9) · p95 28.5 | - |
| native | web.read | web.snapshot | - | 78.2 (69.7–92.9) · p95 94.5 | 84.8 (70.4–100.8) · p95 117.7 | - |
| native | web.read | web.view | - | 7.5 (6.7–8.9) · p95 9.1 | 8.2 (6.8–9.6) · p95 11.5 | - |
| native | lifecycle | from_source | - | 3391.5 (2699.8–3543.9) · p95 4271.7 | 3083.8 (2637.2–3361.0) · p95 3345.6 | - |
| native | lifecycle | restore | - | 3672.4 (2860.2–3810.9) · p95 4792.6 | 3182.1 (2760.2–3588.5) · p95 3538.3 | - |
| native | lifecycle | restore.parse | - | 43.9 (32.1–46.3) · p95 63.4 | 38.3 (32.2–49.4) · p95 47.3 | - |
| native | lifecycle | restore_json | - | 3657.5 (2894.5–3851.4) · p95 4491.5 | 3175.9 (2827.2–3631.9) · p95 3642.0 | - |
| native | lifecycle | save | - | 42.4 (28.8–47.8) · p95 55.3 | 37.0 (26.0–40.0) · p95 44.5 | - |
| native | lifecycle | save_json | - | 37.7 (25.1–52.3) · p95 46.8 | 36.3 (33.2–48.4) · p95 58.4 | - |
| native | lifecycle | web.new | - | 3466.1 (2652.5–3534.4) · p95 4085.7 | 2984.0 (2660.7–3337.0) · p95 3335.5 | - |
| native | lifecycle | web.restore | - | 3606.7 (2902.9–3890.8) · p95 4338.3 | 3250.0 (2839.3–3656.6) · p95 3638.4 | - |
| native | lifecycle | web.save | - | 35.0 (24.7–35.7) · p95 47.3 | 29.2 (22.1–34.0) · p95 34.1 | - |
| wasm | published-method | adapter.dispatch+view | 51.2 (46.4–68.8) · p95 74.1 | 53.1 (45.4–67.1) · p95 70.5 | 53.5 (46.4–66.0) · p95 69.5 | 54.6 (44.1–64.6) · p95 79.2 |
| wasm | adapter | adapter.dispatch | 51.6 (45.0–65.3) · p95 82.3 | 54.3 (42.3–61.6) · p95 84.8 | 48.3 (43.4–60.2) · p95 62.2 | 50.7 (42.1–61.8) · p95 72.6 |
| wasm | adapter | adapter.view | 2.6 (2.4–3.5) · p95 4.0 | 2.7 (2.3–3.2) · p95 4.2 | 2.4 (2.3–3.2) · p95 3.2 | 2.8 (2.3–3.4) · p95 4.2 |
| wasm | adapter | adapter.dispatch+view | 54.2 (47.5–68.9) · p95 86.5 | 57.1 (44.7–65.0) · p95 89.1 | 50.7 (45.8–63.5) · p95 65.6 | 53.7 (44.5–65.4) · p95 76.4 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.5 (0.4–0.5) · p95 0.7 | 0.4 (0.3–0.5) · p95 0.6 | 0.4 (0.3–0.5) · p95 0.6 | 0.5 (0.3–0.5) · p95 0.8 |
| wasm | raw.dispatch_view | raw.dispatch_view | 37.1 (25.2–38.3) · p95 52.3 | 29.8 (24.8–36.5) · p95 34.3 | 29.0 (26.7–37.7) · p95 38.5 | 34.7 (25.6–37.6) · p95 58.0 |
| wasm | raw.dispatch_view | js.parse_view | 24.5 (16.5–25.4) · p95 32.4 | 19.4 (16.4–24.1) · p95 21.8 | 19.1 (17.0–24.5) · p95 25.2 | 23.5 (17.0–25.1) · p95 36.2 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 62.3 (42.2–64.4) · p95 85.0 | 49.7 (41.6–61.3) · p95 56.0 | 48.5 (44.0–63.0) · p95 64.0 | 58.9 (43.1–63.4) · p95 94.8 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 84.5 (71.8–106.4) · p95 117.0 | 99.5 (76.7–110.0) · p95 135.9 | 104.0 (74.2–108.8) · p95 183.4 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 75.2 (63.9–92.5) · p95 100.5 | 87.7 (68.0–97.6) · p95 114.6 | 92.6 (64.7–95.5) · p95 153.8 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 160.1 (136.2–199.6) · p95 217.2 | 187.4 (144.7–208.7) · p95 250.4 | 197.2 (139.3–204.6) · p95 338.4 | - |
| wasm | raw.dispatch | raw.dispatch | 111.2 (98.5–128.6) · p95 129.4 | 112.6 (97.3–128.7) · p95 144.7 | 125.4 (103.9–132.4) · p95 178.0 | 117.2 (103.9–133.1) · p95 145.0 |
| wasm | abi.dispatch_view | abi.encode_args | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.3 | 0.2 (0.1–0.2) · p95 0.3 | 0.2 (0.1–0.2) · p95 0.3 |
| wasm | abi.dispatch_view | abi.exec | 25.3 (21.8–25.4) · p95 28.8 | 25.0 (21.9–26.8) · p95 32.1 | 24.8 (21.3–25.8) · p95 30.0 | 24.0 (20.6–24.6) · p95 28.8 |
| wasm | abi.dispatch_view | abi.decode | 2.8 (2.4–2.9) · p95 3.4 | 2.8 (2.4–3.0) · p95 3.7 | 2.9 (2.4–2.9) · p95 3.6 | 2.8 (2.4–2.8) · p95 3.5 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | abi.dispatch_view | abi.dispatch_view | 28.4 (24.5–28.4) · p95 32.4 | 28.1 (24.6–30.1) · p95 35.9 | 28.1 (23.9–29.0) · p95 33.7 | 27.1 (23.2–27.7) · p95 32.5 |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | - |
| wasm | abi.dispatch_outcome | abi.exec | 69.0 (59.5–70.1) · p95 77.5 | 65.5 (59.5–71.4) · p95 75.9 | 68.3 (62.8–69.2) · p95 79.3 | - |
| wasm | abi.dispatch_outcome | abi.decode | 8.9 (7.5–8.9) · p95 10.9 | 8.3 (7.6–9.2) · p95 10.0 | 8.7 (7.5–9.0) · p95 10.7 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 78.7 (67.5–79.6) · p95 89.8 | 74.3 (67.6–81.5) · p95 86.5 | 78.1 (70.8–78.4) · p95 91.4 | - |
| wasm | read | raw.view | 14.2 (11.0–14.6) · p95 21.2 | 12.0 (11.3–13.5) · p95 15.0 | 13.3 (11.0–13.6) · p95 14.7 | 12.2 (10.2–12.8) · p95 13.5 |
| wasm | read | js.parse_view | 23.7 (18.0–24.0) · p95 35.3 | 19.9 (18.6–22.9) · p95 25.2 | 22.1 (18.0–22.3) · p95 24.4 | 21.2 (17.9–22.1) · p95 23.5 |
| wasm | read | raw.snapshot | 119.0 (90.3–120.1) · p95 200.6 | 100.1 (92.1–112.5) · p95 130.8 | 110.0 (89.8–112.4) · p95 126.4 | 111.4 (93.5–116.0) · p95 126.8 |
| wasm | read | js.parse_snapshot | 89.1 (68.7–92.1) · p95 132.4 | 75.7 (70.3–86.5) · p95 94.9 | 83.9 (67.9–85.4) · p95 92.2 | 82.6 (68.9–85.8) · p95 90.5 |
| wasm | read | raw.save | 24.4 (18.4–24.6) · p95 38.3 | 20.7 (19.0–23.3) · p95 27.9 | 22.6 (18.9–23.2) · p95 28.2 | 21.8 (18.8–22.9) · p95 26.9 |
| wasm | abi.read | abi.view.exec | 9.9 (8.6–10.8) · p95 11.7 | 9.7 (9.2–11.3) · p95 13.4 | 9.7 (8.7–10.7) · p95 12.1 | 9.2 (7.8–9.5) · p95 11.4 |
| wasm | abi.read | abi.view.decode | 2.7 (2.3–2.9) · p95 3.4 | 2.6 (2.5–3.2) · p95 3.5 | 2.7 (2.4–2.8) · p95 3.4 | 2.7 (2.3–2.8) · p95 3.4 |
| wasm | abi.read | abi.snapshot.exec | 83.6 (72.6–91.6) · p95 102.0 | 81.1 (78.2–96.9) · p95 111.4 | 81.6 (73.1–88.7) · p95 103.5 | 86.4 (75.1–88.6) · p95 109.4 |
| wasm | abi.read | abi.snapshot.decode | 14.9 (12.6–15.7) · p95 21.3 | 14.0 (14.0–17.3) · p95 19.3 | 14.7 (12.7–15.0) · p95 20.8 | 14.9 (12.6–15.1) · p95 19.5 |
| wasm | abi.read | abi.save.exec | 17.8 (15.6–19.6) · p95 22.5 | 17.6 (16.7–21.0) · p95 24.6 | 17.5 (16.0–19.2) · p95 23.1 | 18.4 (15.9–18.5) · p95 24.2 |
| wasm | abi.read | abi.save.decode | 0.7 (0.6–0.8) · p95 1.0 | 0.7 (0.7–0.8) · p95 1.0 | 0.7 (0.6–0.7) · p95 0.9 | 0.7 (0.7–0.8) · p95 1.1 |
| wasm | kit | kit.dispatch | 146.1 (142.3–170.3) · p95 188.5 | 169.7 (149.5–175.3) · p95 208.2 | 156.4 (148.8–182.1) · p95 225.2 | - |
| wasm | kit | kit.view | 30.3 (29.8–35.1) · p95 38.8 | 35.4 (31.2–36.6) · p95 43.1 | 32.2 (30.7–37.9) · p95 45.4 | - |
| wasm | kit | kit.dispatch+view | 176.7 (172.4–205.8) · p95 226.9 | 205.6 (180.9–212.7) · p95 252.5 | 188.9 (179.9–220.5) · p95 271.0 | - |
| wasm | kit.read | kit.view | 34.2 (33.7–39.6) · p95 45.3 | 38.7 (34.6–43.9) · p95 54.9 | 33.6 (32.4–37.1) · p95 46.7 | - |
| wasm | kit.read | kit.snapshot | 186.9 (185.0–216.2) · p95 264.1 | 211.0 (187.7–243.2) · p95 317.3 | 185.1 (175.9–201.4) · p95 262.2 | - |
| wasm | kit.read | kit.save | 44.8 (44.5–52.0) · p95 59.5 | 50.8 (45.3–58.7) · p95 72.8 | 44.0 (42.5–48.7) · p95 62.1 | - |
| wasm | lifecycle | raw.new | 2833.5 (2609.0–4150.0) · p95 3211.2 | 2800.3 (2557.3–3075.8) · p95 3051.5 | 2806.3 (2579.0–3436.4) · p95 3106.1 | 2668.0 (2554.3–3089.8) · p95 2923.8 |
| wasm | lifecycle | raw.save | 58.7 (55.7–81.8) · p95 82.8 | 56.8 (53.4–65.8) · p95 82.0 | 56.5 (54.1–70.9) · p95 70.7 | 52.4 (51.7–71.4) · p95 74.5 |
| wasm | lifecycle | raw.restore | 2888.5 (2688.1–4335.9) · p95 3261.8 | 2870.6 (2588.8–3171.0) · p95 3119.0 | 2871.3 (2633.8–3171.7) · p95 3267.7 | 2748.3 (2671.8–3381.3) · p95 3204.2 |
| wasm | lifecycle | kit.open | 2757.0 (2521.7–4096.4) · p95 3091.7 | 2752.7 (2454.8–2965.9) · p95 2936.0 | 2730.3 (2488.4–2975.1) · p95 2970.7 | - |
| wasm | lifecycle | kit.save | 135.2 (121.2–164.1) · p95 225.6 | 123.2 (118.1–139.8) · p95 178.0 | 121.1 (119.3–149.4) · p95 159.0 | - |
| wasm | lifecycle | kit.restore | 2919.6 (2651.5–4273.5) · p95 3258.9 | 2865.2 (2563.5–3076.9) · p95 3062.8 | 2830.6 (2570.2–3069.0) · p95 3083.7 | - |
| wasm | micro | timer.hrtime_pair | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.2) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | micro | js.stringify_payload.first | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | micro | js.stringify_payload.last | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | micro | bridge.pass_ascii.payload | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | micro | kit.payloadText.last | 0.2 (0.2–0.2) · p95 0.3 | 0.2 (0.2–0.2) · p95 0.2 | 0.2 (0.2–0.2) · p95 0.2 | - |
| wasm | micro | bridge.decode.view | 2.2 (2.2–2.7) · p95 3.3 | 2.3 (2.2–2.6) · p95 2.8 | 2.5 (2.2–2.8) · p95 3.4 | 2.2 (2.0–2.6) · p95 7.1 |
| wasm | micro | bridge.pass_ascii.view | 4.7 (4.1–5.3) · p95 5.3 | 4.4 (4.2–5.0) · p95 4.7 | 4.4 (4.3–5.9) · p95 4.7 | 4.2 (4.1–4.4) · p95 4.9 |
| wasm | micro | js.parse.view | 18.3 (17.4–20.2) · p95 23.2 | 16.6 (15.7–19.0) · p95 18.8 | 17.6 (16.3–20.5) · p95 20.5 | 17.5 (15.4–18.4) · p95 21.4 |
| wasm | micro | bridge.decode.outcome | 7.1 (6.9–7.9) · p95 9.2 | 6.7 (6.5–7.4) · p95 9.6 | 6.9 (6.6–7.7) · p95 9.2 | - |
| wasm | micro | bridge.pass_ascii.outcome | 19.6 (19.6–21.3) · p95 22.6 | 18.6 (18.5–20.3) · p95 19.8 | 19.0 (18.5–22.5) · p95 20.0 | - |
| wasm | micro | js.parse.outcome | 69.4 (67.7–75.5) · p95 79.4 | 63.1 (62.4–75.3) · p95 70.0 | 63.3 (63.1–75.8) · p95 69.7 | - |
| wasm | micro | bridge.decode.snapshot | 11.2 (11.1–12.6) · p95 14.7 | 11.0 (10.9–12.0) · p95 13.5 | 10.9 (10.9–13.4) · p95 13.0 | 11.4 (11.0–12.7) · p95 15.6 |
| wasm | micro | bridge.pass_ascii.snapshot | 33.8 (32.4–35.9) · p95 38.5 | 31.2 (30.6–35.3) · p95 33.1 | 32.2 (31.5–37.5) · p95 34.6 | 32.1 (30.8–36.3) · p95 34.9 |
| wasm | micro | js.parse.snapshot | 72.3 (70.7–82.6) · p95 91.8 | 66.9 (66.6–80.5) · p95 73.8 | 69.6 (66.8–83.0) · p95 76.3 | 71.8 (70.8–80.4) · p95 92.1 |
| wasm | micro | bridge.decode.save | 0.5 (0.5–0.5) · p95 0.7 | 0.5 (0.5–0.5) · p95 0.7 | 0.5 (0.5–0.5) · p95 0.7 | 0.5 (0.5–0.6) · p95 0.8 |
| wasm | micro | bridge.pass_ascii.save | 4.3 (4.2–4.6) · p95 5.0 | 4.0 (4.0–4.6) · p95 4.2 | 4.1 (3.9–4.9) · p95 4.3 | 4.1 (4.0–4.9) · p95 4.3 |
| wasm | micro | js.parse.save | 15.5 (13.8–15.8) · p95 18.0 | 13.5 (13.4–16.4) · p95 15.3 | 14.5 (13.2–16.5) · p95 17.0 | 14.1 (14.0–15.3) · p95 16.7 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | rc4-local | 119.6 (100.5–123.9) | 22.5 (21.7–27.3) | 21.0 (18.4–22.1) | 19.8 (13.8–21.0) |
| native | apply | apply | main-local | 116.0 (92.8–120.5) | 21.9 (18.7–23.5) | 18.2 (15.6–19.4) | 18.4 (15.1–18.6) |
| native | dispatch_view_json | dispatch_view_json | rc4-local | 112.6 (108.7–136.5) | 24.1 (23.4–26.9) | 21.1 (20.9–24.8) | 20.4 (20.3–23.6) |
| native | dispatch_view_json | dispatch_view_json | main-local | 104.4 (94.7–131.5) | 22.3 (18.9–25.4) | 20.4 (16.4–23.2) | 19.3 (16.3–21.8) |
| native | dispatch_json | dispatch_json | rc4-local | 137.9 (127.5–363.8) | 44.4 (41.7–57.4) | 41.4 (39.0–51.1) | 42.8 (41.2–54.6) |
| native | dispatch_json | dispatch_json | main-local | 159.8 (132.1–198.1) | 51.5 (44.3–52.3) | 47.4 (42.3–50.5) | 49.9 (44.1–50.6) |
| native | dispatch_outcome_json | dispatch_outcome_json | rc4-local | 152.5 (148.7–162.7) | 47.5 (39.1–53.5) | 47.4 (36.8–48.5) | 46.8 (39.0–48.4) |
| native | dispatch_outcome_json | dispatch_outcome_json | main-local | 130.5 (128.7–375.0) | 46.3 (38.4–56.1) | 43.0 (36.0–52.1) | 44.7 (37.0–53.7) |
| native | web.dispatch_view | web.dispatch_view | rc4-local | 127.9 (116.6–149.8) | 27.1 (25.7–32.4) | 24.3 (23.3–30.8) | 24.4 (23.2–29.6) |
| native | web.dispatch_view | web.dispatch_view | main-local | 152.9 (131.7–185.6) | 33.3 (29.1–34.5) | 31.6 (25.9–31.7) | 27.2 (25.6–29.6) |
| native | web.dispatch_outcome | web.dispatch_outcome | rc4-local | 191.1 (149.3–249.0) | 72.9 (63.8–97.4) | 72.7 (63.2–95.6) | 89.5 (67.2–98.5) |
| native | web.dispatch_outcome | web.dispatch_outcome | main-local | 182.4 (180.9–201.4) | 80.3 (72.5–92.6) | 78.9 (71.6–96.6) | 84.7 (74.5–98.7) |
| native | web.dispatch | web.dispatch | rc4-local | 179.5 (166.7–201.6) | 90.0 (80.4–104.0) | 90.6 (80.5–104.8) | 93.7 (82.9–107.6) |
| native | web.dispatch | web.dispatch | main-local | 218.4 (180.0–252.5) | 94.1 (82.3–113.5) | 92.8 (83.1–114.1) | 92.8 (88.6–122.9) |
| native | read | clone | rc4-local | 8.3 (7.5–8.9) | 7.0 (6.8–9.1) | 7.3 (7.0–9.2) | 8.2 (7.9–10.2) |
| native | read | clone | main-local | 7.8 (7.6–12.4) | 7.2 (6.5–8.9) | 7.4 (6.6–9.0) | 8.4 (7.5–10.4) |
| native | read | clone.drop | rc4-local | 4.5 (4.1–4.8) | 3.9 (3.7–5.1) | 3.9 (3.8–5.1) | 4.4 (4.3–5.6) |
| native | read | clone.drop | main-local | 4.1 (4.1–5.0) | 4.0 (3.6–4.9) | 4.1 (3.6–5.0) | 4.5 (4.0–5.7) |
| native | read | save | rc4-local | 6.9 (6.3–7.7) | 7.4 (7.1–9.6) | 9.9 (9.0–11.8) | 10.4 (10.0–12.8) |
| native | read | save | main-local | 6.3 (6.0–8.3) | 7.5 (6.8–9.3) | 9.4 (9.0–11.6) | 10.8 (9.5–13.6) |
| native | read | save_json | rc4-local | 11.9 (10.8–12.7) | 12.1 (12.1–15.9) | 16.3 (14.9–19.6) | 16.8 (16.4–21.2) |
| native | read | save_json | main-local | 10.3 (10.3–13.7) | 12.3 (11.3–15.4) | 15.5 (15.0–19.1) | 17.3 (15.4–22.1) |
| native | read | snapshot | rc4-local | 38.3 (37.1–41.9) | 25.2 (24.7–32.6) | 27.3 (25.7–34.2) | 28.7 (27.8–35.9) |
| native | read | snapshot | main-local | 32.3 (29.2–46.8) | 25.8 (22.9–31.6) | 27.2 (24.5–33.2) | 29.5 (25.8–37.2) |
| native | read | snapshot.drop | rc4-local | 12.7 (11.0–13.5) | 10.7 (10.5–13.8) | 11.6 (11.0–14.7) | 12.3 (12.0–15.5) |
| native | read | snapshot.drop | main-local | 11.3 (11.2–13.6) | 11.1 (9.8–13.5) | 11.7 (10.5–14.2) | 12.5 (11.1–16.0) |
| native | read | snapshot.serialize | rc4-local | 22.9 (20.8–25.7) | 15.5 (15.1–20.1) | 16.8 (15.9–21.1) | 17.1 (16.7–21.7) |
| native | read | snapshot.serialize | main-local | 19.8 (18.8–24.5) | 16.2 (14.4–19.9) | 17.0 (15.4–20.8) | 17.7 (15.7–22.9) |
| native | read | snapshot.serialize_pretty | rc4-local | 44.1 (42.0–44.8) | 33.0 (32.2–42.7) | 35.4 (33.4–44.8) | 36.1 (35.2–45.4) |
| native | read | snapshot.serialize_pretty | main-local | 37.1 (36.6–48.4) | 34.4 (30.4–42.1) | 36.0 (32.6–43.9) | 37.5 (33.2–48.2) |
| native | read | view | rc4-local | 2.5 (2.5–2.6) | 2.3 (2.2–2.8) | 2.7 (2.5–3.3) | 2.9 (2.8–3.6) |
| native | read | view | main-local | 2.3 (2.3–2.9) | 2.2 (2.0–2.7) | 2.5 (2.4–3.1) | 2.9 (2.6–3.7) |
| native | read | view.serialize | rc4-local | 8.8 (7.6–9.4) | 5.0 (4.9–6.4) | 5.1 (4.9–6.6) | 5.3 (5.2–6.7) |
| native | read | view.serialize | main-local | 7.1 (6.8–9.6) | 5.2 (4.6–6.3) | 5.3 (4.7–6.4) | 5.6 (4.9–7.1) |
| native | web.read | web.save | rc4-local | 13.5 (12.6–15.9) | 13.3 (12.5–17.4) | 16.1 (16.0–20.4) | 18.7 (16.5–22.0) |
| native | web.read | web.save | main-local | 15.1 (12.4–17.0) | 14.5 (12.5–17.8) | 18.6 (16.3–20.7) | 20.6 (16.8–24.3) |
| native | web.read | web.snapshot | rc4-local | 98.8 (86.9–102.4) | 66.3 (62.6–87.9) | 71.2 (68.2–90.9) | 79.4 (69.8–93.4) |
| native | web.read | web.snapshot | main-local | 101.7 (77.4–108.8) | 73.0 (63.0–89.2) | 81.3 (67.9–95.8) | 85.9 (70.5–102.1) |
| native | web.read | web.view | rc4-local | 7.5 (7.4–8.9) | 6.1 (5.8–8.0) | 6.7 (6.4–8.5) | 7.6 (6.7–9.0) |
| native | web.read | web.view | main-local | 8.4 (7.1–9.3) | 6.7 (5.7–8.1) | 7.6 (6.4–8.8) | 8.3 (6.8–9.7) |
| wasm | published-method | adapter.dispatch+view | rc4-published | 211.6 (185.8–849.5) | 63.7 (49.6–90.1) | 51.4 (44.9–64.7) | 50.9 (46.4–68.6) |
| wasm | published-method | adapter.dispatch+view | rc4-local | 177.1 (175.0–264.9) | 57.2 (49.6–79.1) | 55.8 (45.7–65.8) | 52.8 (45.3–66.9) |
| wasm | published-method | adapter.dispatch+view | main-local | 180.8 (173.0–194.0) | 60.3 (53.1–79.6) | 52.6 (45.9–71.4) | 53.4 (46.2–65.1) |
| wasm | published-method | adapter.dispatch+view | hist-e6ace96 | 338.9 (217.9–403.8) | 70.6 (50.7–77.6) | 55.6 (44.8–67.8) | 54.0 (44.0–63.8) |
| wasm | adapter | adapter.dispatch | rc4-published | 195.2 (159.4–274.9) | 53.5 (43.2–69.0) | 48.5 (41.0–60.7) | 51.8 (45.5–65.2) |
| wasm | adapter | adapter.dispatch | rc4-local | 177.2 (170.4–185.0) | 61.5 (45.4–62.8) | 51.6 (42.2–58.6) | 54.1 (42.2–61.6) |
| wasm | adapter | adapter.dispatch | main-local | 186.0 (147.9–212.3) | 45.6 (43.9–65.2) | 43.6 (43.5–54.5) | 48.9 (43.3–60.3) |
| wasm | adapter | adapter.dispatch | hist-e6ace96 | 195.2 (180.3–241.4) | 54.0 (43.9–63.3) | 48.8 (41.2–59.7) | 50.5 (42.0–62.0) |
| wasm | adapter | adapter.view | rc4-published | 35.1 (32.1–38.2) | 3.1 (2.5–4.3) | 2.5 (2.3–3.5) | 2.6 (2.4–3.5) |
| wasm | adapter | adapter.view | rc4-local | 37.4 (31.1–42.5) | 3.4 (2.7–4.2) | 2.8 (2.3–3.4) | 2.7 (2.3–3.2) |
| wasm | adapter | adapter.view | main-local | 30.4 (29.0–38.6) | 2.7 (2.6–4.2) | 2.5 (2.2–3.1) | 2.4 (2.3–3.1) |
| wasm | adapter | adapter.view | hist-e6ace96 | 30.1 (29.9–44.2) | 3.0 (2.6–4.3) | 2.7 (2.4–3.4) | 2.7 (2.3–3.4) |
| wasm | adapter | adapter.dispatch+view | rc4-published | 246.3 (199.7–309.8) | 56.8 (45.9–73.3) | 51.2 (43.2–64.1) | 54.5 (48.0–68.8) |
| wasm | adapter | adapter.dispatch+view | rc4-local | 230.4 (215.2–231.1) | 65.4 (48.2–67.4) | 54.3 (44.5–62.1) | 56.9 (44.5–65.0) |
| wasm | adapter | adapter.dispatch+view | main-local | 216.4 (166.2–239.4) | 48.2 (46.7–69.5) | 46.1 (45.8–57.8) | 51.3 (45.7–63.5) |
| wasm | adapter | adapter.dispatch+view | hist-e6ace96 | 225.4 (204.8–250.2) | 57.4 (46.6–67.9) | 51.7 (43.6–63.3) | 53.4 (44.3–65.5) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-published | 1.3 (1.2–1.9) | 0.5 (0.3–0.6) | 0.5 (0.3–0.5) | 0.4 (0.4–0.5) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-local | 1.2 (1.0–1.3) | 0.3 (0.3–0.5) | 0.3 (0.3–0.5) | 0.4 (0.3–0.5) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local | 1.7 (0.9–1.9) | 0.3 (0.3–0.6) | 0.3 (0.3–0.5) | 0.4 (0.3–0.5) |
| wasm | raw.dispatch_view | js.stringify_payload | hist-e6ace96 | 1.8 (1.0–3.0) | 0.5 (0.3–0.6) | 0.5 (0.3–0.5) | 0.5 (0.3–0.5) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-published | 168.5 (136.0–189.4) | 41.6 (27.9–51.4) | 42.2 (25.0–42.9) | 36.4 (25.1–38.0) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-local | 138.8 (123.2–198.0) | 33.0 (28.4–40.4) | 30.0 (26.4–37.1) | 29.7 (24.7–36.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local | 135.3 (129.4–222.6) | 32.8 (27.7–50.2) | 29.2 (24.8–38.9) | 28.9 (27.2–37.2) |
| wasm | raw.dispatch_view | raw.dispatch_view | hist-e6ace96 | 184.5 (154.6–185.0) | 39.7 (28.1–54.0) | 36.2 (24.1–43.6) | 34.2 (25.6–37.0) |
| wasm | raw.dispatch_view | js.parse_view | rc4-published | 41.0 (32.9–57.0) | 23.6 (15.9–29.6) | 26.8 (15.8–26.9) | 24.1 (16.6–25.4) |
| wasm | raw.dispatch_view | js.parse_view | rc4-local | 39.5 (29.0–67.0) | 18.6 (16.0–23.5) | 18.8 (16.5–23.6) | 19.5 (16.4–24.1) |
| wasm | raw.dispatch_view | js.parse_view | main-local | 32.3 (30.3–69.8) | 18.4 (15.5–28.0) | 18.4 (15.5–24.4) | 19.1 (17.9–24.4) |
| wasm | raw.dispatch_view | js.parse_view | hist-e6ace96 | 46.2 (38.3–65.1) | 23.2 (15.8–30.8) | 23.5 (15.6–28.6) | 23.5 (17.2–24.9) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-published | 199.5 (159.6–279.5) | 66.0 (44.1–81.8) | 69.8 (41.2–70.5) | 61.1 (42.1–64.1) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-local | 184.2 (151.5–252.3) | 52.1 (44.7–64.6) | 49.2 (43.2–61.4) | 49.6 (41.5–61.2) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local | 189.4 (156.0–344.9) | 51.6 (43.5–79.1) | 48.0 (40.6–64.0) | 48.4 (45.5–62.3) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | hist-e6ace96 | 244.1 (193.1–275.7) | 63.7 (44.3–85.8) | 60.2 (40.0–72.9) | 58.4 (43.2–62.6) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | rc4-published | 223.9 (193.5–318.0) | 74.7 (69.9–104.6) | 76.4 (69.2–103.2) | 86.3 (72.0–106.6) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | rc4-local | 235.2 (202.8–278.1) | 80.0 (75.1–101.6) | 81.5 (73.9–101.8) | 102.6 (76.9–111.3) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | main-local | 309.0 (211.0–316.8) | 101.3 (76.9–110.4) | 96.8 (73.0–98.4) | 104.1 (74.2–110.2) |
| wasm | raw.dispatch_outcome | js.parse_outcome | rc4-published | 86.5 (85.0–144.6) | 63.1 (58.5–86.1) | 67.1 (61.9–89.0) | 77.5 (64.2–93.0) |
| wasm | raw.dispatch_outcome | js.parse_outcome | rc4-local | 100.5 (85.5–216.5) | 68.0 (61.0–84.6) | 71.4 (63.8–88.8) | 90.4 (68.4–99.1) |
| wasm | raw.dispatch_outcome | js.parse_outcome | main-local | 139.7 (85.7–194.4) | 84.7 (61.9–90.0) | 85.4 (63.9–86.1) | 93.1 (64.8–96.4) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | rc4-published | 312.1 (277.8–462.6) | 139.2 (128.8–191.6) | 143.8 (131.4–192.5) | 164.2 (136.8–200.2) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | rc4-local | 335.7 (288.3–494.6) | 149.2 (136.2–187.0) | 153.6 (138.6–191.6) | 193.4 (145.3–211.4) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | main-local | 448.7 (296.7–478.5) | 186.9 (139.2–201.7) | 183.5 (137.4–184.8) | 197.8 (139.4–207.2) |
| wasm | raw.dispatch | raw.dispatch | rc4-published | 243.0 (218.6–258.9) | 102.0 (94.0–118.8) | 102.8 (95.0–119.5) | 112.1 (98.8–129.7) |
| wasm | raw.dispatch | raw.dispatch | rc4-local | 221.2 (220.3–250.9) | 107.4 (91.9–115.5) | 109.1 (92.2–123.8) | 113.0 (97.7–130.3) |
| wasm | raw.dispatch | raw.dispatch | main-local | 233.4 (225.8–252.0) | 118.2 (97.1–127.2) | 119.4 (99.2–126.1) | 126.1 (104.6–133.0) |
| wasm | raw.dispatch | raw.dispatch | hist-e6ace96 | 287.4 (237.8–328.9) | 112.4 (94.3–122.2) | 106.6 (98.5–133.8) | 117.6 (104.8–133.7) |
| wasm | abi.dispatch_view | abi.encode_args | rc4-published | 1.1 (1.0–1.6) | 0.3 (0.2–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | rc4-local | 0.9 (0.9–1.0) | 0.3 (0.2–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.encode_args | main-local | 1.0 (0.9–1.2) | 0.3 (0.3–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | hist-e6ace96 | 1.1 (1.0–1.2) | 0.3 (0.2–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.exec | rc4-published | 139.1 (130.7–141.7) | 27.6 (23.6–33.0) | 24.8 (21.8–26.8) | 25.1 (21.7–25.3) |
| wasm | abi.dispatch_view | abi.exec | rc4-local | 131.7 (109.4–141.5) | 30.5 (24.3–31.3) | 26.0 (22.4–28.0) | 24.8 (21.8–26.6) |
| wasm | abi.dispatch_view | abi.exec | main-local | 139.5 (109.3–144.0) | 28.6 (24.1–30.0) | 25.9 (22.0–26.6) | 24.7 (21.1–25.7) |
| wasm | abi.dispatch_view | abi.exec | hist-e6ace96 | 148.4 (120.1–164.6) | 28.0 (23.4–34.1) | 25.1 (20.8–25.2) | 23.9 (20.5–24.5) |
| wasm | abi.dispatch_view | abi.decode | rc4-published | 4.5 (4.3–5.2) | 2.7 (2.3–3.3) | 2.6 (2.4–2.9) | 2.8 (2.4–2.9) |
| wasm | abi.dispatch_view | abi.decode | rc4-local | 4.0 (3.9–4.0) | 3.0 (2.4–3.0) | 2.9 (2.4–2.9) | 2.8 (2.4–3.0) |
| wasm | abi.dispatch_view | abi.decode | main-local | 4.3 (3.7–4.9) | 2.9 (2.4–3.0) | 2.8 (2.4–3.0) | 2.9 (2.4–2.9) |
| wasm | abi.dispatch_view | abi.decode | hist-e6ace96 | 4.3 (4.0–4.5) | 2.8 (2.4–3.3) | 2.7 (2.3–2.8) | 2.8 (2.4–2.8) |
| wasm | abi.dispatch_view | abi.free | rc4-published | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | rc4-local | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | main-local | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | hist-e6ace96 | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.dispatch_view | rc4-published | 150.4 (135.0–151.0) | 30.7 (26.3–36.8) | 27.8 (24.4–30.0) | 28.2 (24.4–28.3) |
| wasm | abi.dispatch_view | abi.dispatch_view | rc4-local | 135.4 (114.7–146.1) | 34.1 (27.1–34.7) | 29.3 (25.0–31.3) | 27.9 (24.5–29.9) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local | 145.7 (114.7–148.9) | 31.9 (26.8–33.6) | 29.0 (24.6–29.9) | 27.9 (23.7–28.9) |
| wasm | abi.dispatch_view | abi.dispatch_view | hist-e6ace96 | 153.9 (125.3–175.0) | 31.1 (26.1–37.9) | 28.1 (23.3–28.3) | 27.0 (23.1–27.6) |
| wasm | abi.dispatch_outcome | abi.encode_args | rc4-published | 1.0 (0.9–1.1) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | abi.dispatch_outcome | abi.encode_args | rc4-local | 1.0 (0.9–1.1) | 0.3 (0.3–0.4) | 0.2 (0.2–0.2) | 0.2 (0.1–0.2) |
| wasm | abi.dispatch_outcome | abi.encode_args | main-local | 1.0 (0.9–1.2) | 0.3 (0.3–0.4) | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) |
| wasm | abi.dispatch_outcome | abi.exec | rc4-published | 182.8 (160.8–189.5) | 68.9 (59.0–70.0) | 67.5 (59.0–71.1) | 69.1 (59.5–70.1) |
| wasm | abi.dispatch_outcome | abi.exec | rc4-local | 170.4 (162.1–185.5) | 65.8 (57.9–71.2) | 66.9 (57.3–70.2) | 65.5 (59.7–71.5) |
| wasm | abi.dispatch_outcome | abi.exec | main-local | 178.1 (165.6–212.2) | 69.0 (59.1–72.3) | 66.7 (61.0–67.2) | 68.4 (63.0–69.3) |
| wasm | abi.dispatch_outcome | abi.decode | rc4-published | 8.3 (7.9–9.6) | 8.0 (6.6–8.4) | 8.6 (7.2–8.8) | 8.9 (7.6–9.0) |
| wasm | abi.dispatch_outcome | abi.decode | rc4-local | 8.0 (7.8–10.7) | 7.6 (6.8–8.4) | 8.2 (7.2–8.8) | 8.3 (7.7–9.3) |
| wasm | abi.dispatch_outcome | abi.decode | main-local | 8.2 (7.8–8.8) | 7.8 (6.5–8.4) | 8.2 (7.2–8.5) | 8.8 (7.6–9.0) |
| wasm | abi.dispatch_outcome | abi.free | rc4-published | 0.3 (0.3–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.free | rc4-local | 0.2 (0.2–0.6) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.free | main-local | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | rc4-published | 192.1 (170.2–201.1) | 78.0 (66.1–78.6) | 76.8 (66.8–80.3) | 78.8 (67.5–79.6) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | rc4-local | 179.5 (171.0–197.2) | 74.0 (65.3–80.4) | 75.7 (65.0–79.8) | 74.3 (67.8–81.7) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | main-local | 187.4 (175.0–222.9) | 77.5 (66.2–81.3) | 76.1 (68.7–76.1) | 78.2 (71.0–78.5) |
| wasm | read | raw.view | rc4-published | 13.7 (11.2–14.3) | 11.7 (9.7–12.7) | 13.1 (10.3–14.1) | 14.2 (11.0–14.8) |
| wasm | read | raw.view | rc4-local | 10.9 (10.7–13.1) | 9.9 (9.9–12.2) | 10.9 (10.7–12.9) | 12.3 (11.4–13.6) |
| wasm | read | raw.view | main-local | 11.6 (11.6–12.4) | 11.4 (10.0–11.7) | 12.2 (10.3–12.4) | 13.4 (11.1–13.8) |
| wasm | read | raw.view | hist-e6ace96 | 10.9 (10.3–11.7) | 9.5 (8.9–10.8) | 11.4 (9.5–11.8) | 12.2 (10.3–12.9) |
| wasm | read | js.parse_view | rc4-published | 39.2 (28.0–54.7) | 21.2 (17.1–23.2) | 22.3 (17.2–23.6) | 23.7 (18.0–24.4) |
| wasm | read | js.parse_view | rc4-local | 29.4 (28.3–33.6) | 17.9 (17.6–22.5) | 18.5 (17.9–22.5) | 20.4 (18.7–23.0) |
| wasm | read | js.parse_view | main-local | 32.9 (31.9–51.7) | 20.6 (17.0–21.1) | 20.9 (17.1–21.1) | 22.2 (18.1–22.4) |
| wasm | read | js.parse_view | hist-e6ace96 | 31.9 (29.1–31.9) | 18.2 (16.7–20.4) | 20.4 (16.9–20.9) | 21.3 (18.0–22.3) |
| wasm | read | raw.snapshot | rc4-published | 119.6 (100.3–122.6) | 100.8 (81.2–109.8) | 111.3 (84.0–118.6) | 119.3 (90.7–122.5) |
| wasm | read | raw.snapshot | rc4-local | 97.8 (97.7–117.2) | 84.8 (83.8–103.8) | 90.9 (87.6–107.9) | 103.6 (92.9–113.1) |
| wasm | read | raw.snapshot | main-local | 103.1 (102.5–121.5) | 97.1 (80.9–99.1) | 101.5 (83.6–102.6) | 110.6 (90.2–113.4) |
| wasm | read | raw.snapshot | hist-e6ace96 | 101.0 (98.0–108.8) | 88.1 (80.0–97.9) | 104.9 (86.3–107.4) | 112.0 (94.3–117.0) |
| wasm | read | js.parse_snapshot | rc4-published | 98.1 (75.9–113.3) | 77.0 (61.7–82.4) | 85.7 (66.0–88.8) | 89.3 (69.0–93.8) |
| wasm | read | js.parse_snapshot | rc4-local | 90.6 (73.5–101.9) | 64.2 (63.1–80.6) | 70.3 (67.7–83.7) | 78.1 (70.9–87.0) |
| wasm | read | js.parse_snapshot | main-local | 83.1 (76.4–95.2) | 74.0 (61.1–75.9) | 78.5 (65.3–79.8) | 84.3 (68.2–86.0) |
| wasm | read | js.parse_snapshot | hist-e6ace96 | 75.8 (74.3–82.4) | 67.0 (61.1–75.3) | 79.2 (65.4–81.2) | 82.9 (69.3–86.6) |
| wasm | read | raw.save | rc4-published | 20.0 (16.4–20.5) | 17.2 (13.7–18.7) | 21.8 (16.8–22.9) | 24.7 (18.5–25.3) |
| wasm | read | raw.save | rc4-local | 18.9 (16.2–20.4) | 14.4 (14.4–17.5) | 18.2 (17.2–21.1) | 21.3 (19.2–23.5) |
| wasm | read | raw.save | main-local | 18.0 (16.5–18.1) | 16.6 (13.9–16.9) | 19.5 (17.1–20.5) | 22.7 (19.0–23.4) |
| wasm | read | raw.save | hist-e6ace96 | 18.4 (16.0–18.4) | 14.9 (13.8–16.4) | 21.1 (17.7–21.6) | 22.0 (19.0–23.1) |
| wasm | abi.read | abi.view.exec | rc4-published | 10.8 (9.5–12.4) | 9.2 (7.6–10.3) | 9.6 (8.0–11.1) | 9.9 (8.6–10.9) |
| wasm | abi.read | abi.view.exec | rc4-local | 10.5 (9.5–10.9) | 8.4 (8.2–9.0) | 9.1 (8.6–9.9) | 9.8 (9.2–11.6) |
| wasm | abi.read | abi.view.exec | main-local | 9.9 (9.7–10.6) | 8.6 (7.7–9.2) | 9.1 (8.3–9.9) | 9.7 (8.8–10.8) |
| wasm | abi.read | abi.view.exec | hist-e6ace96 | 9.4 (9.0–9.7) | 7.0 (6.8–7.7) | 7.9 (7.5–8.5) | 9.3 (7.9–9.6) |
| wasm | abi.read | abi.view.decode | rc4-published | 2.1 (2.0–2.2) | 2.8 (2.3–2.9) | 2.8 (2.3–3.1) | 2.7 (2.3–2.9) |
| wasm | abi.read | abi.view.decode | rc4-local | 2.1 (2.0–2.6) | 2.4 (2.4–2.7) | 2.5 (2.4–2.8) | 2.6 (2.5–3.2) |
| wasm | abi.read | abi.view.decode | main-local | 2.2 (2.0–2.2) | 2.6 (2.3–2.6) | 2.6 (2.3–2.8) | 2.7 (2.4–2.8) |
| wasm | abi.read | abi.view.decode | hist-e6ace96 | 2.2 (2.1–2.2) | 2.3 (2.2–2.6) | 2.4 (2.2–2.6) | 2.8 (2.3–2.8) |
| wasm | abi.read | abi.snapshot.exec | rc4-published | 98.6 (89.5–117.3) | 80.7 (66.4–86.8) | 83.9 (68.1–93.9) | 83.8 (72.9–92.1) |
| wasm | abi.read | abi.snapshot.exec | rc4-local | 97.4 (95.6–101.7) | 72.8 (72.4–80.4) | 77.8 (74.4–85.8) | 82.0 (78.6–99.6) |
| wasm | abi.read | abi.snapshot.exec | main-local | 90.5 (90.0–93.3) | 75.5 (67.3–81.3) | 77.6 (69.8–84.6) | 82.1 (73.3–89.1) |
| wasm | abi.read | abi.snapshot.exec | hist-e6ace96 | 97.7 (91.6–102.5) | 66.6 (65.1–73.6) | 75.7 (70.4–79.2) | 87.5 (75.5–89.7) |
| wasm | abi.read | abi.snapshot.decode | rc4-published | 9.8 (9.4–10.2) | 13.7 (11.2–14.3) | 14.7 (11.9–15.9) | 14.9 (12.6–15.8) |
| wasm | abi.read | abi.snapshot.decode | rc4-local | 10.3 (10.1–11.6) | 12.3 (11.6–13.4) | 13.3 (12.7–15.2) | 14.2 (14.1–17.6) |
| wasm | abi.read | abi.snapshot.decode | main-local | 10.2 (9.5–10.4) | 12.7 (11.4–13.3) | 13.8 (12.1–15.2) | 14.9 (12.8–15.1) |
| wasm | abi.read | abi.snapshot.decode | hist-e6ace96 | 10.7 (9.7–10.7) | 11.5 (11.5–13.0) | 12.9 (12.0–14.1) | 15.0 (12.6–15.3) |
| wasm | abi.read | abi.save.exec | rc4-published | 15.2 (15.0–19.8) | 14.6 (12.1–16.1) | 17.0 (14.5–19.3) | 17.9 (15.7–19.9) |
| wasm | abi.read | abi.save.exec | rc4-local | 16.0 (16.0–16.4) | 13.3 (13.2–14.6) | 16.0 (15.3–18.1) | 17.9 (16.9–21.6) |
| wasm | abi.read | abi.save.exec | main-local | 15.6 (15.4–16.9) | 13.6 (12.3–15.0) | 16.3 (15.0–18.3) | 17.6 (16.0–19.4) |
| wasm | abi.read | abi.save.exec | hist-e6ace96 | 16.5 (15.5–16.5) | 11.9 (11.7–13.1) | 15.3 (15.1–16.7) | 18.7 (16.1–18.8) |
| wasm | abi.read | abi.save.decode | rc4-published | 0.5 (0.5–0.5) | 0.5 (0.5–0.6) | 0.7 (0.6–0.8) | 0.7 (0.6–0.8) |
| wasm | abi.read | abi.save.decode | rc4-local | 0.5 (0.4–0.5) | 0.5 (0.5–0.6) | 0.7 (0.6–0.7) | 0.7 (0.7–0.8) |
| wasm | abi.read | abi.save.decode | main-local | 0.5 (0.5–0.5) | 0.5 (0.5–0.6) | 0.7 (0.6–0.7) | 0.7 (0.7–0.7) |
| wasm | abi.read | abi.save.decode | hist-e6ace96 | 0.6 (0.5–0.6) | 0.5 (0.5–0.5) | 0.6 (0.6–0.7) | 0.8 (0.7–0.8) |
| wasm | kit | kit.dispatch | rc4-published | 335.0 (331.0–389.2) | 141.7 (134.5–161.5) | 143.3 (135.9–161.1) | 146.4 (143.0–171.3) |
| wasm | kit | kit.dispatch | rc4-local | 316.5 (316.0–328.5) | 150.3 (149.4–150.6) | 155.7 (144.8–156.2) | 171.4 (149.7–177.2) |
| wasm | kit | kit.dispatch | main-local | 355.0 (341.2–452.5) | 147.4 (146.6–166.0) | 148.5 (147.1–166.8) | 157.3 (148.9–184.8) |
| wasm | kit | kit.view | rc4-published | 43.2 (42.7–53.5) | 29.2 (27.9–33.0) | 29.9 (28.4–33.6) | 30.3 (30.0–35.3) |
| wasm | kit | kit.view | rc4-local | 44.8 (43.0–47.6) | 31.1 (30.2–31.8) | 32.8 (30.1–32.8) | 35.7 (31.3–37.0) |
| wasm | kit | kit.view | main-local | 48.4 (43.3–100.8) | 30.6 (30.3–34.3) | 30.8 (30.5–34.9) | 32.4 (30.7–38.4) |
| wasm | kit | kit.dispatch+view | rc4-published | 395.7 (373.7–432.4) | 170.9 (162.9–194.4) | 173.5 (164.5–195.3) | 177.1 (173.2–207.1) |
| wasm | kit | kit.dispatch+view | rc4-local | 362.9 (357.2–382.6) | 181.8 (180.0–182.5) | 188.9 (175.6–189.6) | 207.7 (181.1–215.0) |
| wasm | kit | kit.dispatch+view | main-local | 400.5 (393.8–609.9) | 178.1 (178.1–201.5) | 179.3 (177.8–202.8) | 189.8 (179.9–223.6) |
| wasm | kit.read | kit.view | rc4-published | 49.1 (44.7–58.4) | 32.7 (31.0–38.6) | 32.1 (31.6–37.8) | 34.4 (33.9–39.8) |
| wasm | kit.read | kit.view | rc4-local | 51.8 (49.7–131.1) | 32.8 (31.7–41.3) | 33.3 (32.7–42.3) | 39.2 (34.9–44.2) |
| wasm | kit.read | kit.view | main-local | 44.0 (43.7–55.5) | 31.3 (30.2–34.9) | 32.9 (31.1–35.8) | 33.7 (32.5–37.3) |
| wasm | kit.read | kit.snapshot | rc4-published | 186.1 (178.3–227.4) | 173.9 (166.5–206.8) | 172.0 (171.0–205.0) | 188.5 (186.3–217.3) |
| wasm | kit.read | kit.snapshot | rc4-local | 187.7 (178.2–556.6) | 174.6 (170.1–222.7) | 180.7 (173.3–229.1) | 214.1 (189.1–245.8) |
| wasm | kit.read | kit.snapshot | main-local | 179.3 (167.9–187.5) | 167.9 (160.4–186.5) | 177.6 (165.8–192.9) | 185.7 (176.8–202.8) |
| wasm | kit.read | kit.save | rc4-published | 51.0 (40.8–67.6) | 35.6 (34.1–43.2) | 40.3 (39.5–47.8) | 45.2 (44.8–52.6) |
| wasm | kit.read | kit.save | rc4-local | 42.9 (41.9–136.7) | 35.6 (35.3–46.3) | 41.8 (40.4–54.4) | 51.8 (45.7–59.5) |
| wasm | kit.read | kit.save | main-local | 42.6 (41.9–57.1) | 34.5 (32.9–38.2) | 42.2 (39.5–44.8) | 44.3 (42.8–49.2) |

## glowcap-resume

The published resume measurement's stream (experiments/glowcap/resume-bench.mjs): absorb cave glowcap, then 9,600 ticks of dt 0.0625 (ten minutes of play), then save and restore. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `0407c19120d4`, 1 episode(s) × 1 = 9601 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | lifecycle | from_source | - | 2773.2 (2738.4–3009.1) · p95 3090.1 | 2691.6 (2637.1–3008.4) · p95 2819.2 | - |
| native | lifecycle | restore | - | 2897.3 (2875.1–3123.3) · p95 3244.5 | 2790.1 (2712.0–3082.5) · p95 2960.4 | - |
| native | lifecycle | restore.parse | - | 32.9 (27.1–35.8) · p95 40.6 | 27.1 (20.5–28.6) · p95 31.2 | - |
| native | lifecycle | restore_json | - | 2895.8 (2870.8–3191.6) · p95 3386.1 | 2847.0 (2739.6–3130.7) · p95 3075.5 | - |
| native | lifecycle | save | - | 21.2 (18.6–21.5) · p95 27.6 | 17.1 (14.2–18.8) · p95 22.3 | - |
| native | lifecycle | save_json | - | 23.0 (22.8–27.5) · p95 33.0 | 18.0 (15.6–19.0) · p95 23.8 | - |
| native | lifecycle | web.new | - | 2769.9 (2746.3–3026.8) · p95 3137.6 | 2689.7 (2659.0–2995.9) · p95 2786.7 | - |
| native | lifecycle | web.restore | - | 2881.6 (2862.0–3166.9) · p95 3200.9 | 2829.8 (2776.0–3123.3) · p95 3180.6 | - |
| native | lifecycle | web.save | - | 17.6 (15.7–18.2) · p95 21.6 | 13.6 (11.4–14.6) · p95 16.1 | - |
| wasm | lifecycle | raw.new | 2618.7 (2447.3–2870.9) · p95 3089.7 | 2679.1 (2489.9–3136.0) · p95 3103.1 | 2678.7 (2468.1–3162.9) · p95 3177.8 | 2680.8 (2342.0–3276.9) · p95 3192.5 |
| wasm | lifecycle | raw.save | 41.8 (31.6–45.8) · p95 51.3 | 39.5 (36.5–49.4) · p95 53.1 | 35.8 (32.0–49.0) · p95 45.8 | 41.5 (30.5–52.5) · p95 52.2 |
| wasm | lifecycle | raw.restore | 2595.9 (2474.2–2875.7) · p95 2970.6 | 2775.1 (2495.0–3118.4) · p95 3290.0 | 2610.4 (2473.7–3116.9) · p95 3148.3 | 2688.2 (2342.3–3349.5) · p95 3143.5 |
| wasm | lifecycle | kit.open | 2565.6 (2388.7–2842.5) · p95 3040.7 | 2833.7 (2414.6–3092.1) · p95 3267.4 | 2553.3 (2423.6–3128.0) · p95 3109.9 | - |
| wasm | lifecycle | kit.save | 99.6 (71.5–114.0) · p95 147.5 | 93.6 (78.9–122.2) · p95 137.7 | 84.0 (68.6–130.7) · p95 112.5 | - |
| wasm | lifecycle | kit.restore | 2574.3 (2422.4–2839.2) · p95 3100.0 | 2694.5 (2457.4–3096.9) · p95 3273.4 | 2593.0 (2460.7–3091.5) · p95 2862.4 | - |
| wasm | adapter-resume | adapter.resume.parse | 15.1 (14.7–17.9) · p95 15.1 | 14.5 (13.6–20.3) · p95 14.5 | 14.7 (14.7–19.0) · p95 14.7 | 16.0 (15.0–18.2) · p95 16.0 |
| wasm | adapter-resume | adapter.resume.createPolicy | 3431.7 (3335.8–4584.6) · p95 3431.7 | 3504.9 (3335.7–4863.3) · p95 3504.9 | 3468.6 (3409.8–5163.4) · p95 3468.6 | 3079.7 (3066.8–3501.9) · p95 3079.7 |
| wasm | adapter-resume | adapter.resume.view | 58.0 (46.0–105.6) · p95 58.0 | 58.5 (44.3–70.5) · p95 58.5 | 49.8 (46.1–56.2) · p95 49.8 | 47.7 (46.4–48.7) · p95 47.7 |
| wasm | adapter-resume | adapter.resume | 3532.5 (3396.5–4655.0) · p95 3532.5 | 3551.2 (3418.1–4938.8) · p95 3551.2 | 3537.6 (3458.4–5244.0) · p95 3537.6 | 3127.5 (3113.4–3568.8) · p95 3127.5 |

## glowcap-unbound

The Glowcap replay program with its 39 one-line bind statements removed, on the replay's full stream: the same rules, states and transaction with no bindings to evaluate, so apply here against apply on glowcap-replay isolates binding evaluation. Program `experiments/glowcap/caveat5/glowcap.cav` (`ffa078c4cb75`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 13.5 (13.1–15.7) · p95 17.2 | 12.9 (12.4–16.1) · p95 16.0 | - |
| native | dispatch_view_json | dispatch_view_json | - | 15.7 (15.1–25.0) · p95 16.9 | 16.2 (15.1–19.3) · p95 19.8 | - |
| native | web.dispatch_view | web.dispatch_view | - | 20.0 (18.8–23.2) · p95 23.2 | 19.0 (18.2–22.1) · p95 23.6 | - |
| native | read | clone | - | 0.4 (0.3–0.4) · p95 0.4 | 0.3 (0.3–0.4) · p95 0.4 | - |
| native | read | clone.drop | - | 0.3 (0.3–0.3) · p95 0.4 | 0.3 (0.3–0.4) · p95 0.4 | - |
| native | read | save | - | 10.1 (9.3–10.5) · p95 11.5 | 9.3 (9.2–11.7) · p95 9.9 | - |
| native | read | save_json | - | 16.2 (14.8–16.7) · p95 18.4 | 14.9 (14.7–18.3) · p95 15.9 | - |
| native | read | snapshot | - | 20.4 (18.8–21.3) · p95 23.2 | 18.5 (18.3–23.3) · p95 19.7 | - |
| native | read | snapshot.drop | - | 8.0 (7.4–8.4) · p95 9.1 | 7.3 (7.1–9.0) · p95 7.7 | - |
| native | read | snapshot.serialize | - | 13.2 (12.1–14.1) · p95 14.9 | 12.2 (12.1–15.2) · p95 13.1 | - |
| native | read | snapshot.serialize_pretty | - | 26.8 (24.5–27.9) · p95 30.2 | 24.7 (24.4–30.5) · p95 26.4 | - |
| native | read | view | - | 2.8 (2.6–2.9) · p95 3.2 | 2.5 (2.5–3.2) · p95 2.8 | - |
| native | read | view.serialize | - | 2.7 (2.4–2.8) · p95 3.1 | 2.5 (2.5–3.1) · p95 2.7 | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.1 (0.1–0.2) · p95 0.4 | 0.1 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.4 | 0.2 (0.1–0.2) · p95 0.3 |
| wasm | abi.dispatch_view | abi.exec | 18.7 (18.0–22.0) · p95 25.5 | 18.3 (18.0–21.4) · p95 24.3 | 18.7 (18.2–21.7) · p95 25.2 | 18.8 (17.1–20.9) · p95 27.1 |
| wasm | abi.dispatch_view | abi.decode | 0.5 (0.5–0.6) · p95 1.1 | 0.5 (0.5–0.6) · p95 1.0 | 0.5 (0.5–0.6) · p95 1.1 | 0.5 (0.5–0.6) · p95 1.1 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | abi.dispatch_view | abi.dispatch_view | 19.5 (18.7–23.0) · p95 26.9 | 19.1 (18.8–22.3) · p95 25.5 | 19.5 (19.1–22.7) · p95 26.5 | 19.8 (17.9–21.8) · p95 28.9 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.2–0.4) · p95 0.5 | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.2–0.3) · p95 0.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 22.0 (19.0–23.1) · p95 29.5 | 22.2 (19.1–23.9) · p95 28.8 | 20.3 (19.6–23.4) · p95 28.9 | 19.0 (18.4–23.0) · p95 25.4 |
| wasm | raw.dispatch_view | js.parse_view | 7.1 (6.1–7.4) · p95 9.3 | 7.1 (6.2–8.1) · p95 9.0 | 6.4 (6.3–7.5) · p95 9.2 | 6.1 (5.9–7.4) · p95 8.1 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 29.5 (25.4–30.8) · p95 39.2 | 29.7 (25.6–32.4) · p95 38.4 | 27.1 (26.2–31.3) · p95 38.5 | 25.5 (24.6–30.8) · p95 33.9 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | rc4-local | 36.9 (34.0–40.0) | 15.3 (14.5–17.6) | 14.0 (13.5–16.3) | 13.3 (13.1–15.7) |
| native | apply | apply | main-local | 37.9 (35.1–42.9) | 15.1 (13.6–17.6) | 13.8 (12.7–16.1) | 12.8 (12.3–16.0) |
| native | dispatch_view_json | dispatch_view_json | rc4-local | 37.7 (36.7–59.3) | 16.6 (15.9–26.0) | 16.0 (15.3–24.8) | 15.7 (15.1–24.9) |
| native | dispatch_view_json | dispatch_view_json | main-local | 43.5 (38.3–45.1) | 16.8 (16.1–19.9) | 16.7 (15.3–19.4) | 16.1 (15.0–19.2) |
| native | web.dispatch_view | web.dispatch_view | rc4-local | 54.9 (46.1–74.4) | 20.5 (20.4–25.1) | 19.9 (19.8–24.5) | 20.0 (18.6–23.0) |
| native | web.dispatch_view | web.dispatch_view | main-local | 47.4 (43.2–53.1) | 19.5 (19.0–22.4) | 19.0 (18.4–22.1) | 19.0 (18.2–22.1) |
| native | read | clone | rc4-local | 0.6 (0.6–0.7) | 0.4 (0.3–0.4) | 0.4 (0.4–0.4) | 0.4 (0.3–0.4) |
| native | read | clone | main-local | 0.6 (0.6–0.8) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) |
| native | read | clone.drop | rc4-local | 0.4 (0.4–0.4) | 0.3 (0.3–0.3) | 0.3 (0.3–0.4) | 0.3 (0.3–0.3) |
| native | read | clone.drop | main-local | 0.4 (0.4–0.6) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) |
| native | read | save | rc4-local | 5.7 (5.5–5.8) | 7.2 (6.4–7.3) | 9.6 (8.8–9.7) | 10.2 (9.3–10.6) |
| native | read | save | main-local | 5.6 (5.1–6.9) | 6.5 (6.4–7.8) | 8.8 (8.7–9.8) | 9.3 (9.3–11.8) |
| native | read | save_json | rc4-local | 10.3 (9.6–10.3) | 12.1 (10.8–12.3) | 15.9 (14.6–16.0) | 16.3 (14.8–16.8) |
| native | read | save_json | main-local | 9.3 (9.0–11.3) | 10.7 (10.6–12.9) | 14.6 (14.4–16.0) | 14.9 (14.8–18.6) |
| native | read | snapshot | rc4-local | 21.6 (19.4–30.3) | 18.4 (16.3–18.5) | 19.9 (18.4–20.9) | 20.5 (18.8–21.5) |
| native | read | snapshot | main-local | 22.9 (21.9–23.5) | 16.3 (16.1–19.6) | 18.0 (17.8–21.6) | 18.5 (18.4–23.6) |
| native | read | snapshot.drop | rc4-local | 7.4 (6.7–7.5) | 7.1 (6.3–7.2) | 7.7 (7.1–8.1) | 8.1 (7.4–8.5) |
| native | read | snapshot.drop | main-local | 6.8 (6.7–7.6) | 6.4 (6.2–7.5) | 7.0 (6.9–8.3) | 7.3 (7.2–9.1) |
| native | read | snapshot.serialize | rc4-local | 16.1 (13.9–17.5) | 12.3 (11.1–12.9) | 13.0 (12.0–14.0) | 13.3 (12.1–14.2) |
| native | read | snapshot.serialize | main-local | 15.0 (14.0–18.8) | 11.3 (11.1–13.6) | 12.0 (11.9–14.3) | 12.2 (12.1–15.4) |
| native | read | snapshot.serialize_pretty | rc4-local | 28.8 (25.5–31.1) | 24.7 (22.2–25.2) | 26.3 (24.3–27.8) | 26.9 (24.6–28.0) |
| native | read | snapshot.serialize_pretty | main-local | 26.8 (25.2–31.9) | 22.4 (21.9–26.8) | 24.3 (23.9–28.8) | 24.7 (24.6–30.9) |
| native | read | view | rc4-local | 2.3 (2.2–2.5) | 2.2 (2.0–2.2) | 2.6 (2.4–2.7) | 2.8 (2.6–3.0) |
| native | read | view | main-local | 2.3 (2.1–2.6) | 2.0 (1.9–2.4) | 2.3 (2.3–2.8) | 2.6 (2.5–3.2) |
| native | read | view.serialize | rc4-local | 3.3 (3.3–3.5) | 2.5 (2.2–2.5) | 2.6 (2.4–2.7) | 2.7 (2.4–2.8) |
| native | read | view.serialize | main-local | 3.4 (3.1–3.9) | 2.3 (2.2–2.7) | 2.4 (2.3–2.9) | 2.5 (2.5–3.1) |
| wasm | abi.dispatch_view | abi.encode_args | rc4-published | 1.0 (0.8–1.1) | 0.3 (0.2–0.4) | 0.1 (0.1–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.encode_args | rc4-local | 0.9 (0.9–1.3) | 0.3 (0.2–0.3) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | main-local | 0.9 (0.9–1.1) | 0.3 (0.3–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.encode_args | hist-e6ace96 | 1.1 (1.0–1.2) | 0.3 (0.3–0.3) | 0.2 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.exec | rc4-published | 58.6 (57.1–110.2) | 22.7 (18.9–24.6) | 20.3 (18.4–23.4) | 18.5 (17.9–21.9) |
| wasm | abi.dispatch_view | abi.exec | rc4-local | 62.8 (58.4–82.1) | 21.1 (18.6–22.6) | 18.5 (18.2–22.3) | 18.2 (17.9–21.3) |
| wasm | abi.dispatch_view | abi.exec | main-local | 65.7 (60.8–80.2) | 21.4 (20.0–26.9) | 22.6 (19.7–23.4) | 18.5 (18.1–21.4) |
| wasm | abi.dispatch_view | abi.exec | hist-e6ace96 | 71.6 (53.0–75.1) | 20.6 (18.9–25.0) | 18.0 (17.8–22.3) | 19.0 (17.0–20.7) |
| wasm | abi.dispatch_view | abi.decode | rc4-published | 1.9 (1.6–2.6) | 0.6 (0.5–0.7) | 0.5 (0.5–0.6) | 0.5 (0.5–0.6) |
| wasm | abi.dispatch_view | abi.decode | rc4-local | 1.4 (1.3–1.4) | 0.6 (0.5–0.6) | 0.5 (0.5–0.6) | 0.5 (0.5–0.6) |
| wasm | abi.dispatch_view | abi.decode | main-local | 1.7 (1.5–2.7) | 0.6 (0.5–0.7) | 0.6 (0.5–0.6) | 0.5 (0.5–0.6) |
| wasm | abi.dispatch_view | abi.decode | hist-e6ace96 | 1.5 (1.3–1.8) | 0.5 (0.5–0.7) | 0.5 (0.5–0.6) | 0.5 (0.5–0.6) |
| wasm | abi.dispatch_view | abi.free | rc4-published | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | rc4-local | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | main-local | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | hist-e6ace96 | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.dispatch_view | rc4-published | 60.2 (60.1–125.9) | 23.7 (19.8–25.8) | 21.1 (19.1–24.3) | 19.4 (18.7–22.8) |
| wasm | abi.dispatch_view | abi.dispatch_view | rc4-local | 64.6 (60.4–84.2) | 22.1 (19.6–23.6) | 19.3 (19.0–23.2) | 19.0 (18.7–22.2) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local | 69.4 (61.9–85.0) | 22.4 (20.8–28.0) | 23.5 (20.6–24.3) | 19.3 (18.9–22.4) |
| wasm | abi.dispatch_view | abi.dispatch_view | hist-e6ace96 | 73.4 (55.7–78.5) | 21.4 (19.6–26.2) | 18.7 (18.5–23.2) | 20.0 (17.8–21.6) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-published | 0.9 (0.7–1.2) | 0.3 (0.3–0.3) | 0.3 (0.2–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-local | 1.2 (1.1–1.2) | 0.3 (0.3–0.4) | 0.3 (0.2–0.4) | 0.3 (0.2–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local | 0.9 (0.9–1.0) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.2–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | hist-e6ace96 | 0.8 (0.8–0.9) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.2–0.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-published | 72.1 (67.0–84.2) | 25.9 (20.1–26.0) | 24.7 (19.3–24.7) | 21.8 (18.9–22.4) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-local | 71.1 (70.3–98.3) | 24.8 (20.9–33.2) | 22.4 (19.7–31.8) | 22.1 (19.0–23.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local | 62.8 (55.2–83.4) | 21.2 (21.0–23.2) | 20.2 (20.2–22.8) | 20.3 (19.5–23.6) |
| wasm | raw.dispatch_view | raw.dispatch_view | hist-e6ace96 | 67.6 (65.2–67.7) | 21.8 (21.8–25.0) | 19.3 (19.3–24.1) | 18.9 (18.3–22.6) |
| wasm | raw.dispatch_view | js.parse_view | rc4-published | 14.6 (14.3–16.5) | 7.3 (5.9–7.3) | 7.4 (5.9–7.5) | 7.1 (6.1–7.4) |
| wasm | raw.dispatch_view | js.parse_view | rc4-local | 14.1 (13.6–21.1) | 7.1 (5.9–9.8) | 6.7 (6.1–10.2) | 7.2 (6.2–7.9) |
| wasm | raw.dispatch_view | js.parse_view | main-local | 17.1 (13.4–17.4) | 6.1 (6.0–6.6) | 6.2 (6.1–6.9) | 6.5 (6.4–7.7) |
| wasm | raw.dispatch_view | js.parse_view | hist-e6ace96 | 13.0 (12.5–13.7) | 6.1 (5.9–7.0) | 5.9 (5.8–7.3) | 6.2 (5.9–7.4) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-published | 87.1 (83.0–104.9) | 33.5 (26.3–33.7) | 32.3 (25.4–32.5) | 29.2 (25.4–30.1) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-local | 88.8 (85.9–119.6) | 32.3 (27.2–43.6) | 29.4 (26.0–43.0) | 29.6 (25.5–31.6) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local | 80.8 (69.6–101.1) | 27.5 (27.4–30.2) | 26.6 (26.5–30.1) | 27.1 (26.2–31.6) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | hist-e6ace96 | 82.2 (79.3–84.4) | 28.3 (28.2–32.5) | 25.5 (25.3–31.7) | 25.4 (24.4–30.4) |

## glowcap-scaled-16

Glowcap with 12 more mushrooms declared after grove (16 in all): symbols, edges, tick rules and bindings grow with the mushroom count; the replay's first 2,000 events. Program `experiments/glowcap/caveat5/glowcap.cav` + 12 mushrooms (`468147924b4a`), stream `f93d60a5f44d`, 1 episode(s) × 1 = 2000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 29.8 (29.3–33.9) · p95 34.7 | 29.7 (29.0–34.4) · p95 35.1 | - |
| native | web.dispatch_view | web.dispatch_view | - | 46.3 (44.8–52.6) · p95 51.4 | 47.4 (45.4–53.3) · p95 53.5 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 172.1 (149.8–189.3) · p95 216.2 | 168.7 (150.3–184.3) · p95 194.2 | - |
| native | read | clone | - | 21.4 (17.9–22.1) · p95 24.9 | 20.7 (18.2–22.6) · p95 25.2 | - |
| native | read | clone.drop | - | 12.1 (10.1–12.5) · p95 14.3 | 11.7 (10.3–12.8) · p95 14.0 | - |
| native | read | save | - | 11.4 (9.7–11.8) · p95 13.8 | 11.0 (10.0–11.9) · p95 13.8 | - |
| native | read | save_json | - | 18.0 (15.5–18.7) · p95 21.4 | 17.8 (15.9–19.2) · p95 21.7 | - |
| native | read | snapshot | - | 70.2 (58.6–73.4) · p95 84.8 | 67.6 (59.9–73.6) · p95 85.4 | - |
| native | read | snapshot.drop | - | 27.7 (23.1–28.3) · p95 31.7 | 26.6 (23.3–29.3) · p95 32.2 | - |
| native | read | snapshot.serialize | - | 39.8 (33.3–41.0) · p95 54.9 | 38.7 (34.2–42.0) · p95 51.8 | - |
| native | read | snapshot.serialize_pretty | - | 86.4 (72.1–88.3) · p95 120.7 | 83.3 (73.5–90.2) · p95 115.8 | - |
| native | read | view | - | 7.2 (6.1–7.5) · p95 8.4 | 6.7 (6.1–7.4) · p95 8.2 | - |
| native | read | view.serialize | - | 10.6 (8.9–10.9) · p95 12.3 | 10.3 (9.1–11.2) · p95 12.9 | - |
| native | lifecycle | from_source | - | 13037.6 (11748.0–13175.4) · p95 13823.9 | 13562.7 (13003.8–15092.8) · p95 14916.6 | - |
| native | lifecycle | restore | - | 13334.1 (11889.9–13374.2) · p95 14286.6 | 13363.6 (13228.0–13976.9) · p95 15845.7 | - |
| native | lifecycle | restore.parse | - | 41.2 (39.6–43.2) · p95 46.4 | 42.6 (42.1–46.0) · p95 53.9 | - |
| native | lifecycle | restore_json | - | 13486.5 (12161.7–13691.7) · p95 14268.1 | 13727.8 (13338.6–13957.7) · p95 15375.4 | - |
| native | lifecycle | save | - | 40.3 (37.8–43.5) · p95 69.8 | 41.6 (38.7–47.6) · p95 55.4 | - |
| native | lifecycle | save_json | - | 30.8 (28.2–32.1) · p95 33.5 | 31.9 (30.9–34.6) · p95 38.2 | - |
| native | lifecycle | web.new | - | 13007.3 (11619.9–13076.2) · p95 13978.0 | 13667.6 (13104.5–14586.9) · p95 14954.9 | - |
| native | lifecycle | web.restore | - | 13404.6 (12010.7–13641.3) · p95 14341.0 | 13763.4 (13678.2–13947.9) · p95 15685.9 | - |
| native | lifecycle | web.save | - | 32.0 (31.8–33.6) · p95 41.2 | 33.0 (32.5–38.0) · p95 39.7 | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.4) · p95 0.6 | 0.4 (0.3–0.4) · p95 0.7 | 0.4 (0.3–0.4) · p95 0.7 | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 59.7 (56.3–73.6) · p95 74.3 | 65.7 (58.3–73.3) · p95 88.7 | 69.0 (56.8–70.3) · p95 86.9 | 59.5 (58.6–60.1) · p95 74.9 |
| wasm | raw.dispatch_view | js.parse_view | 38.4 (36.4–47.3) · p95 47.0 | 42.9 (37.5–47.0) · p95 56.1 | 45.0 (36.9–45.0) · p95 54.8 | 39.6 (39.2–39.8) · p95 47.9 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 98.4 (93.6–122.0) · p95 121.8 | 109.5 (95.6–120.4) · p95 144.7 | 114.4 (94.4–115.8) · p95 142.2 | 99.7 (98.5–100.3) · p95 122.4 |
| wasm | abi.dispatch_view | abi.encode_args | 0.2 (0.2–0.2) · p95 0.5 | 0.2 (0.2–0.2) · p95 0.5 | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | abi.dispatch_view | abi.exec | 52.4 (50.4–63.3) · p95 64.1 | 52.6 (50.6–63.4) · p95 70.9 | 55.0 (49.7–62.7) · p95 65.3 | 48.8 (48.1–60.2) · p95 58.6 |
| wasm | abi.dispatch_view | abi.decode | 5.8 (5.5–7.0) · p95 7.5 | 5.7 (5.7–7.0) · p95 7.7 | 6.0 (5.6–7.1) · p95 7.3 | 5.5 (5.4–6.9) · p95 6.9 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | abi.dispatch_view | abi.dispatch_view | 58.5 (56.3–70.8) · p95 72.4 | 58.5 (56.9–71.0) · p95 78.4 | 61.4 (55.8–70.2) · p95 73.0 | 54.8 (53.8–67.7) · p95 66.6 |
| wasm | read | raw.view | 24.3 (24.0–29.8) · p95 31.1 | 24.2 (24.2–27.9) · p95 29.8 | 25.0 (24.0–30.7) · p95 32.7 | 23.7 (22.1–27.1) · p95 29.2 |
| wasm | read | js.parse_view | 40.9 (39.7–49.5) · p95 50.5 | 40.5 (40.1–46.8) · p95 49.5 | 41.5 (39.9–50.9) · p95 52.6 | 41.1 (38.9–47.5) · p95 50.2 |
| wasm | read | raw.snapshot | 271.5 (270.6–330.8) · p95 323.8 | 273.3 (266.9–311.0) · p95 316.6 | 280.2 (269.8–343.6) · p95 344.5 | 275.7 (262.3–318.3) · p95 337.3 |
| wasm | read | js.parse_snapshot | 169.0 (164.9–202.0) · p95 206.3 | 166.8 (165.3–189.8) · p95 200.1 | 171.5 (163.9–208.3) · p95 213.8 | 171.0 (161.3–195.5) · p95 207.8 |
| wasm | read | raw.save | 23.2 (23.1–27.0) · p95 29.7 | 23.4 (23.1–26.2) · p95 29.8 | 23.5 (23.2–28.8) · p95 31.6 | 23.4 (22.4–26.6) · p95 30.4 |
| wasm | kit | kit.dispatch | 342.6 (339.9–429.8) · p95 434.4 | 340.4 (339.6–462.9) · p95 417.1 | 363.6 (335.8–382.2) · p95 477.5 | - |
| wasm | kit | kit.view | 66.2 (65.6–81.5) · p95 78.3 | 66.0 (65.8–88.3) · p95 75.2 | 70.1 (64.4–73.7) · p95 84.9 | - |
| wasm | kit | kit.dispatch+view | 408.8 (406.3–511.9) · p95 505.9 | 406.5 (406.3–552.9) · p95 486.3 | 434.0 (401.0–456.5) · p95 560.8 | - |
| wasm | lifecycle | raw.new | 10901.3 (10577.5–12647.3) · p95 14587.5 | 11181.3 (10539.5–13380.8) · p95 11618.4 | 12494.4 (10163.1–16220.0) · p95 13544.0 | 11851.0 (10049.3–13614.1) · p95 12312.6 |
| wasm | lifecycle | raw.save | 84.7 (78.7–85.8) · p95 107.9 | 79.3 (78.7–98.5) · p95 104.2 | 88.7 (65.7–103.5) · p95 132.1 | 73.7 (72.9–93.9) · p95 109.9 |
| wasm | lifecycle | raw.restore | 10955.1 (10498.0–12760.6) · p95 14267.7 | 11215.0 (10377.6–13175.1) · p95 11604.0 | 12262.4 (10294.3–18928.6) · p95 13229.4 | 11752.5 (10014.9–13491.1) · p95 12154.1 |
| wasm | lifecycle | kit.open | 11053.0 (10686.4–13319.6) · p95 15209.1 | 11267.3 (10582.0–13377.6) · p95 11721.9 | 12438.8 (10336.8–17259.1) · p95 13731.2 | - |
| wasm | lifecycle | kit.save | 167.5 (150.4–211.1) · p95 206.9 | 160.8 (157.7–178.0) · p95 191.0 | 172.2 (134.5–209.2) · p95 216.3 | - |
| wasm | lifecycle | kit.restore | 11035.3 (10610.9–13074.6) · p95 15025.9 | 11356.2 (10530.4–13312.2) · p95 11681.9 | 12333.4 (10326.5–19672.7) · p95 13306.0 | - |

## glowcap-scaled-64

Glowcap with 60 more mushrooms declared after grove (64 in all); the replay's first 2,000 events. Program `experiments/glowcap/caveat5/glowcap.cav` + 60 mushrooms (`29f4f9803de2`), stream `f93d60a5f44d`, 1 episode(s) × 1 = 2000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 107.9 (95.0–112.6) · p95 119.8 | 106.4 (93.1–120.2) · p95 117.6 | - |
| native | web.dispatch_view | web.dispatch_view | - | 164.0 (143.4–175.9) · p95 176.2 | 161.3 (143.5–178.9) · p95 177.6 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 535.7 (470.9–622.5) · p95 577.0 | 569.4 (471.7–586.5) · p95 640.3 | - |
| native | read | clone | - | 61.2 (59.8–75.9) · p95 70.5 | 61.0 (60.0–75.0) · p95 67.7 | - |
| native | read | clone.drop | - | 33.2 (33.0–41.2) · p95 38.7 | 33.5 (32.7–41.8) · p95 37.2 | - |
| native | read | save | - | 13.8 (13.4–17.1) · p95 16.2 | 13.9 (13.7–17.0) · p95 16.0 | - |
| native | read | save_json | - | 19.7 (19.7–24.4) · p95 22.4 | 20.0 (19.5–24.2) · p95 22.1 | - |
| native | read | snapshot | - | 197.5 (190.9–247.8) · p95 240.5 | 199.7 (194.8–243.5) · p95 230.7 | - |
| native | read | snapshot.drop | - | 71.0 (70.0–88.5) · p95 80.7 | 71.1 (69.7–87.1) · p95 77.9 | - |
| native | read | snapshot.serialize | - | 104.5 (102.9–132.2) · p95 128.2 | 107.1 (105.1–132.3) · p95 136.5 | - |
| native | read | snapshot.serialize_pretty | - | 219.9 (218.2–277.5) · p95 259.7 | 223.3 (219.6–273.7) · p95 252.3 | - |
| native | read | view | - | 23.2 (22.8–28.8) · p95 26.7 | 23.8 (23.1–28.9) · p95 26.3 | - |
| native | read | view.serialize | - | 25.5 (24.7–32.0) · p95 30.6 | 26.0 (25.4–31.6) · p95 30.4 | - |
| native | lifecycle | from_source | - | 92134.4 (90093.0–202524.4) · p95 110189.9 | 101365.3 (89143.9–175129.4) · p95 104737.2 | - |
| native | lifecycle | restore | - | 93271.7 (93048.6–203221.9) · p95 107467.3 | 102799.6 (90743.3–185626.6) · p95 108000.0 | - |
| native | lifecycle | restore.parse | - | 39.9 (39.9–84.9) · p95 49.1 | 41.0 (39.0–62.7) · p95 48.8 | - |
| native | lifecycle | restore_json | - | 93932.8 (92749.7–206494.9) · p95 109442.0 | 102508.8 (90151.9–186586.2) · p95 107371.1 | - |
| native | lifecycle | save | - | 63.6 (58.9–127.2) · p95 87.8 | 63.9 (57.8–109.5) · p95 200.8 | - |
| native | lifecycle | save_json | - | 33.4 (32.7–67.9) · p95 41.0 | 36.5 (32.9–58.2) · p95 39.8 | - |
| native | lifecycle | web.new | - | 92312.0 (90657.9–203249.2) · p95 110244.2 | 101682.4 (89497.2–178069.7) · p95 104332.3 | - |
| native | lifecycle | web.restore | - | 93868.8 (91781.1–207974.1) · p95 108454.1 | 102339.3 (91247.7–187847.0) · p95 105805.8 | - |
| native | lifecycle | web.save | - | 46.2 (44.9–89.0) · p95 63.0 | 46.4 (43.0–84.7) · p95 56.1 | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.5 (0.4–1.2) · p95 1.2 | 0.5 (0.4–0.9) · p95 1.0 | 0.4 (0.4–1.2) · p95 0.8 | 0.4 (0.4–0.8) · p95 0.7 |
| wasm | raw.dispatch_view | raw.dispatch_view | 222.3 (180.9–271.9) · p95 329.6 | 212.8 (199.6–248.2) · p95 302.9 | 200.7 (190.0–306.4) · p95 260.5 | 200.5 (180.4–236.9) · p95 251.6 |
| wasm | raw.dispatch_view | js.parse_view | 144.6 (118.0–174.3) · p95 188.0 | 139.3 (128.7–176.3) · p95 177.5 | 132.3 (124.9–196.7) · p95 164.7 | 135.4 (121.6–157.4) · p95 155.4 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 369.0 (300.5–458.4) · p95 506.2 | 354.3 (328.1–430.2) · p95 474.9 | 333.3 (314.6–508.4) · p95 423.6 | 336.4 (302.0–398.5) · p95 398.8 |
| wasm | abi.dispatch_view | abi.encode_args | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.3–0.4) · p95 0.6 | 0.3 (0.2–0.4) · p95 0.6 | 0.2 (0.2–0.4) · p95 0.5 |
| wasm | abi.dispatch_view | abi.exec | 186.8 (164.6–212.3) · p95 215.8 | 195.2 (167.5–229.9) · p95 216.3 | 198.6 (163.0–204.8) · p95 236.3 | 169.6 (159.0–208.6) · p95 195.0 |
| wasm | abi.dispatch_view | abi.decode | 20.9 (19.1–28.2) · p95 26.7 | 21.5 (19.4–27.8) · p95 25.1 | 22.1 (19.2–27.3) · p95 27.1 | 19.3 (19.1–27.4) · p95 24.8 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.2) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.2) · p95 0.2 |
| wasm | abi.dispatch_view | abi.dispatch_view | 208.4 (184.4–242.7) · p95 262.6 | 217.9 (187.4–260.2) · p95 250.8 | 221.9 (182.7–233.9) · p95 268.1 | 189.6 (178.7–238.9) · p95 223.4 |
| wasm | read | raw.view | 97.8 (89.8–111.5) · p95 109.8 | 94.8 (90.6–109.7) · p95 110.5 | 106.0 (96.5–122.1) · p95 141.4 | 94.0 (92.7–108.4) · p95 114.4 |
| wasm | read | js.parse_view | 135.1 (123.8–156.3) · p95 152.1 | 132.9 (128.0–156.7) · p95 152.4 | 147.5 (134.8–174.4) · p95 197.2 | 135.5 (133.3–157.5) · p95 169.4 |
| wasm | read | raw.snapshot | 932.4 (853.4–1067.2) · p95 1019.4 | 910.5 (877.3–1049.3) · p95 1031.0 | 1011.8 (944.9–1177.2) · p95 1326.1 | 939.6 (922.0–1082.3) · p95 1123.8 |
| wasm | read | js.parse_snapshot | 666.8 (612.6–773.2) · p95 737.3 | 657.0 (630.6–768.7) · p95 743.3 | 729.9 (667.6–868.0) · p95 966.7 | 669.4 (655.5–782.7) · p95 820.4 |
| wasm | read | raw.save | 36.7 (34.2–41.8) · p95 44.6 | 34.9 (34.1–41.8) · p95 44.5 | 40.7 (36.5–47.4) · p95 59.0 | 36.3 (35.1–42.5) · p95 46.4 |
| wasm | kit | kit.dispatch | 1437.9 (1313.7–1557.1) · p95 1829.8 | 1392.3 (1360.3–1632.9) · p95 1536.2 | 1380.2 (1338.7–1560.3) · p95 1627.0 | - |
| wasm | kit | kit.view | 254.1 (230.6–275.9) · p95 364.9 | 246.7 (240.5–291.3) · p95 828.7 | 245.6 (236.2–278.2) · p95 795.6 | - |
| wasm | kit | kit.dispatch+view | 1698.5 (1544.5–1836.6) · p95 2345.0 | 1644.6 (1604.3–1932.3) · p95 2202.0 | 1629.2 (1579.4–1848.3) · p95 2125.1 | - |
| wasm | lifecycle | raw.new | 88107.0 (86368.8–108011.2) · p95 117826.1 | 91348.8 (83021.9–111223.1) · p95 100846.7 | 101997.0 (81907.8–102266.2) · p95 120487.2 | 82459.8 (78507.5–105824.9) · p95 108408.2 |
| wasm | lifecycle | raw.save | 108.5 (100.5–113.1) · p95 135.7 | 101.8 (98.7–116.3) · p95 128.2 | 103.8 (96.3–111.6) · p95 126.4 | 98.6 (93.5–123.4) · p95 113.6 |
| wasm | lifecycle | raw.restore | 85493.4 (83610.1–109984.8) · p95 119280.0 | 90939.6 (81978.1–111118.1) · p95 106466.6 | 102737.5 (82412.6–102841.8) · p95 119518.9 | 80489.2 (78781.3–103904.0) · p95 99206.6 |
| wasm | lifecycle | kit.open | 83897.5 (83178.2–108161.5) · p95 117272.7 | 89921.6 (80401.3–110602.1) · p95 99725.9 | 100677.1 (81109.0–101502.0) · p95 125356.0 | - |
| wasm | lifecycle | kit.save | 186.3 (172.1–191.3) · p95 234.1 | 171.8 (163.0–194.6) · p95 204.5 | 183.0 (168.4–190.8) · p95 224.1 | - |
| wasm | lifecycle | kit.restore | 86577.1 (82695.6–109699.6) · p95 117958.0 | 92265.3 (81308.6–108999.7) · p95 101959.8 | 101464.5 (82143.3–101599.7) · p95 112970.4 | - |

## ledger-session

The agent ledger replaying the real 2026-09-24 session log: 20 events, five of them refused by policy. Played 100 times from a fresh session. Program `experiments/agent-ledger/ledger.cav` (`2206ac79f23c`), stream `ef2143311f86`, 1 episode(s) × 100 = 2000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 16.6 (15.8–20.4) · p95 24.2 | 16.1 (15.2–18.3) · p95 23.3 | - |
| native | dispatch_view_json | dispatch_view_json | - | 21.1 (20.9–28.2) · p95 29.0 | 22.3 (22.1–27.0) · p95 30.7 | - |
| native | dispatch_json | dispatch_json | - | 37.5 (36.6–43.1) · p95 46.8 | 36.8 (36.6–46.9) · p95 45.8 | - |
| native | dispatch_outcome_json | dispatch_outcome_json | - | 37.6 (37.2–43.9) · p95 47.2 | 41.1 (36.0–43.5) · p95 54.7 | - |
| native | web.dispatch_view | web.dispatch_view | - | 26.4 (24.7–29.2) · p95 37.3 | 26.4 (24.4–29.2) · p95 35.8 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 62.9 (62.7–72.5) · p95 81.2 | 63.6 (61.9–70.5) · p95 81.6 | - |
| native | web.dispatch | web.dispatch | - | 78.6 (73.8–97.3) · p95 93.6 | 81.5 (81.4–94.5) · p95 99.7 | - |
| native | read | clone | - | 1.8 (1.8–2.2) · p95 2.1 | 1.9 (1.9–2.5) · p95 2.2 | - |
| native | read | clone.drop | - | 1.1 (1.0–1.3) · p95 1.2 | 1.1 (1.1–1.5) · p95 1.3 | - |
| native | read | save | - | 7.0 (6.6–8.2) · p95 9.4 | 7.5 (7.1–9.4) · p95 10.1 | - |
| native | read | save_json | - | 12.5 (12.1–15.2) · p95 17.8 | 13.9 (12.7–16.9) · p95 18.3 | - |
| native | read | snapshot | - | 19.3 (18.2–22.9) · p95 22.2 | 20.0 (19.1–26.4) · p95 22.8 | - |
| native | read | snapshot.drop | - | 7.8 (7.4–9.2) · p95 8.8 | 8.0 (7.7–10.5) · p95 9.1 | - |
| native | read | snapshot.serialize | - | 14.7 (14.0–17.4) · p95 17.3 | 15.0 (14.7–20.2) · p95 18.0 | - |
| native | read | snapshot.serialize_pretty | - | 29.4 (28.2–34.9) · p95 34.0 | 29.7 (29.4–40.1) · p95 34.7 | - |
| native | read | view | - | 3.5 (3.3–4.1) · p95 4.9 | 3.5 (3.4–4.8) · p95 5.1 | - |
| native | read | view.serialize | - | 2.5 (2.4–3.0) · p95 3.2 | 2.6 (2.5–3.5) · p95 3.3 | - |
| native | web.read | web.save | - | 14.5 (13.8–16.6) · p95 20.4 | 14.0 (14.0–16.7) · p95 18.7 | - |
| native | web.read | web.snapshot | - | 60.0 (56.4–69.0) · p95 72.1 | 56.6 (56.4–68.4) · p95 67.3 | - |
| native | web.read | web.view | - | 6.1 (5.7–7.0) · p95 8.7 | 5.7 (5.6–6.9) · p95 8.1 | - |
| native | lifecycle | from_source | - | 1455.6 (1366.3–2382.6) · p95 1498.2 | 1450.6 (1388.5–1833.7) · p95 1523.1 | - |
| native | lifecycle | restore | - | 1473.0 (1436.1–2442.6) · p95 1593.7 | 1518.6 (1430.9–1870.6) · p95 1604.3 | - |
| native | lifecycle | restore.parse | - | 31.6 (25.3–52.1) · p95 36.2 | 27.6 (25.2–38.5) · p95 34.4 | - |
| native | lifecycle | restore_json | - | 1515.3 (1450.3–2504.1) · p95 1572.8 | 1526.4 (1486.5–1896.7) · p95 1615.8 | - |
| native | lifecycle | save | - | 19.2 (15.5–32.3) · p95 22.7 | 17.1 (16.1–25.0) · p95 20.6 | - |
| native | lifecycle | save_json | - | 24.1 (23.7–43.5) · p95 26.3 | 24.3 (23.5–31.2) · p95 30.0 | - |
| native | lifecycle | web.new | - | 1430.8 (1370.2–2335.6) · p95 1511.8 | 1456.9 (1389.9–1826.5) · p95 1514.5 | - |
| native | lifecycle | web.restore | - | 1490.8 (1435.9–2457.0) · p95 1581.9 | 1510.2 (1458.3–1906.0) · p95 1649.0 | - |
| native | lifecycle | web.save | - | 20.2 (18.0–33.9) · p95 27.9 | 18.1 (17.9–25.0) · p95 21.3 | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.4 (0.3–0.4) · p95 1.1 | 0.3 (0.3–0.4) · p95 1.1 | 0.3 (0.3–0.4) · p95 1.0 | 0.4 (0.3–0.4) · p95 1.1 |
| wasm | raw.dispatch_view | raw.dispatch_view | 31.7 (29.7–37.4) · p95 41.2 | 31.3 (29.7–36.1) · p95 40.6 | 31.4 (29.4–38.4) · p95 40.6 | 31.3 (28.1–33.0) · p95 40.9 |
| wasm | raw.dispatch_view | js.parse_view | 7.8 (7.4–9.3) · p95 10.9 | 7.5 (7.0–9.4) · p95 10.6 | 7.6 (7.2–9.6) · p95 11.0 | 8.0 (7.1–8.7) · p95 11.4 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 42.7 (40.4–51.1) · p95 52.8 | 41.9 (39.2–50.0) · p95 51.5 | 42.2 (39.7–52.7) · p95 51.4 | 43.1 (38.1–46.7) · p95 52.6 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 77.8 (77.3–91.8) · p95 108.4 | 76.0 (66.3–84.5) · p95 101.5 | 70.4 (69.7–92.8) · p95 98.6 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 56.1 (54.9–66.5) · p95 75.5 | 54.4 (47.9–64.1) · p95 69.6 | 50.0 (49.4–69.6) · p95 69.5 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 134.8 (132.8–159.4) · p95 182.3 | 131.2 (114.3–150.2) · p95 170.6 | 120.3 (120.1–163.7) · p95 167.1 | - |
| wasm | raw.dispatch | raw.dispatch | 119.2 (90.8–119.3) · p95 152.0 | 98.6 (90.0–100.1) · p95 148.9 | 117.7 (110.7–119.5) · p95 177.0 | 111.7 (102.6–173.8) · p95 240.5 |
| wasm | abi.dispatch_view | abi.encode_args | 0.4 (0.3–0.4) · p95 1.0 | 0.3 (0.3–0.4) · p95 0.9 | 0.4 (0.3–0.5) · p95 1.0 | 0.4 (0.3–0.5) · p95 1.1 |
| wasm | abi.dispatch_view | abi.exec | 34.0 (30.9–40.7) · p95 56.9 | 35.4 (30.8–40.2) · p95 53.9 | 35.2 (34.6–42.9) · p95 70.2 | 31.5 (29.8–41.8) · p95 79.2 |
| wasm | abi.dispatch_view | abi.decode | 0.7 (0.7–0.9) · p95 3.0 | 0.8 (0.7–0.9) · p95 2.8 | 0.9 (0.8–0.9) · p95 3.2 | 0.7 (0.7–0.9) · p95 3.2 |
| wasm | abi.dispatch_view | abi.free | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | abi.dispatch_view | abi.dispatch_view | 38.3 (35.7–46.4) · p95 64.8 | 39.8 (35.3–46.2) · p95 57.6 | 41.0 (39.5–48.4) · p95 76.3 | 36.5 (34.2–47.2) · p95 87.0 |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.4 (0.3–0.5) · p95 1.4 | 0.4 (0.4–0.5) · p95 1.4 | 0.4 (0.4–0.5) · p95 1.3 | - |
| wasm | abi.dispatch_outcome | abi.exec | 66.4 (64.2–90.4) · p95 155.5 | 70.4 (64.7–92.7) · p95 156.6 | 71.3 (65.0–93.2) · p95 160.8 | - |
| wasm | abi.dispatch_outcome | abi.decode | 2.5 (2.4–2.7) · p95 10.0 | 2.5 (2.5–2.7) · p95 10.4 | 2.5 (2.5–2.8) · p95 10.1 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.3 (0.2–0.4) · p95 0.7 | 0.3 (0.3–0.3) · p95 0.7 | 0.3 (0.3–0.4) · p95 0.7 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 70.3 (68.2–94.9) · p95 164.4 | 74.3 (68.9–96.9) · p95 169.9 | 75.1 (69.1–97.6) · p95 169.7 | - |
| wasm | read | raw.view | 12.3 (8.1–13.1) · p95 24.1 | 11.3 (8.1–13.2) · p95 19.6 | 9.5 (8.6–9.6) · p95 14.2 | 9.0 (7.9–9.6) · p95 14.4 |
| wasm | read | js.parse_view | 14.1 (8.8–14.6) · p95 25.6 | 12.9 (9.1–15.1) · p95 21.6 | 10.9 (9.9–11.3) · p95 16.4 | 10.7 (9.3–12.1) · p95 16.0 |
| wasm | read | raw.snapshot | 109.3 (74.8–120.7) · p95 208.9 | 103.1 (76.6–119.9) · p95 171.8 | 86.6 (80.2–89.1) · p95 119.3 | 85.1 (76.6–93.4) · p95 122.2 |
| wasm | read | js.parse_snapshot | 86.0 (57.1–93.0) · p95 155.1 | 79.8 (58.9–93.1) · p95 130.5 | 70.0 (63.1–70.8) · p95 92.3 | 67.6 (59.9–75.2) · p95 95.4 |
| wasm | read | raw.save | 28.4 (18.3–29.3) · p95 55.7 | 25.2 (18.8–30.4) · p95 45.5 | 21.3 (19.7–21.8) · p95 32.2 | 19.9 (18.3–22.1) · p95 32.8 |
| wasm | abi.read | abi.view.exec | 8.7 (7.1–9.1) · p95 13.0 | 8.5 (7.4–10.1) · p95 11.4 | 8.8 (7.4–11.4) · p95 12.0 | 8.1 (6.7–8.5) · p95 11.0 |
| wasm | abi.read | abi.view.decode | 0.5 (0.4–0.5) · p95 0.9 | 0.5 (0.4–0.7) · p95 0.9 | 0.5 (0.4–0.7) · p95 0.9 | 0.5 (0.4–0.5) · p95 0.9 |
| wasm | abi.read | abi.snapshot.exec | 77.0 (66.4–81.2) · p95 106.5 | 76.3 (67.1–88.0) · p95 94.0 | 79.5 (66.7–102.5) · p95 98.1 | 77.9 (65.3–80.2) · p95 91.6 |
| wasm | abi.read | abi.snapshot.decode | 3.9 (3.6–4.0) · p95 6.1 | 3.9 (3.5–4.5) · p95 6.0 | 3.8 (3.6–5.3) · p95 6.3 | 3.8 (3.8–4.0) · p95 6.3 |
| wasm | abi.read | abi.save.exec | 18.9 (16.0–20.0) · p95 29.2 | 18.9 (16.5–22.5) · p95 25.4 | 19.6 (16.3–25.4) · p95 27.0 | 18.4 (15.1–19.1) · p95 24.9 |
| wasm | abi.read | abi.save.decode | 0.9 (0.7–0.9) · p95 1.6 | 0.8 (0.7–1.1) · p95 1.4 | 0.9 (0.7–1.2) · p95 1.6 | 0.8 (0.7–1.0) · p95 1.4 |
| wasm | kit | kit.dispatch | 145.7 (115.3–146.8) · p95 188.6 | 145.5 (118.3–146.5) · p95 187.2 | 146.4 (118.9–149.2) · p95 182.2 | - |
| wasm | kit | kit.view | 19.1 (14.9–20.0) · p95 24.6 | 19.0 (15.5–19.7) · p95 24.8 | 19.0 (15.4–20.5) · p95 24.3 | - |
| wasm | kit | kit.dispatch+view | 164.2 (130.0–165.4) · p95 213.8 | 164.0 (133.6–165.5) · p95 212.5 | 165.0 (134.0–168.2) · p95 205.7 | - |
| wasm | kit.read | kit.view | 20.5 (16.4–22.9) · p95 25.4 | 19.4 (16.4–20.1) · p95 23.7 | 19.7 (16.1–22.5) · p95 25.4 | - |
| wasm | kit.read | kit.snapshot | 148.9 (119.4–166.4) · p95 180.4 | 140.8 (120.2–147.6) · p95 168.6 | 140.2 (117.8–161.0) · p95 180.9 | - |
| wasm | kit.read | kit.save | 39.7 (31.5–44.5) · p95 51.6 | 37.3 (31.7–38.8) · p95 49.3 | 37.7 (30.9–43.1) · p95 50.8 | - |
| wasm | lifecycle | raw.new | 1660.2 (1417.6–1750.8) · p95 2159.4 | 1782.8 (1459.8–1916.6) · p95 1996.4 | 1734.8 (1433.3–1837.4) · p95 2094.3 | 1803.8 (1457.6–1884.7) · p95 3125.4 |
| wasm | lifecycle | raw.save | 59.3 (48.7–59.7) · p95 84.7 | 59.4 (47.8–67.8) · p95 87.8 | 60.8 (51.6–62.3) · p95 90.5 | 63.1 (50.8–63.4) · p95 81.6 |
| wasm | lifecycle | raw.restore | 1777.6 (1505.1–1801.3) · p95 2275.8 | 1896.3 (1505.5–2091.5) · p95 2158.9 | 1879.1 (1541.2–1998.0) · p95 2345.9 | 1794.4 (1502.7–1833.5) · p95 2563.0 |
| wasm | lifecycle | kit.open | 1683.6 (1464.4–1744.2) · p95 2210.4 | 1780.5 (1469.7–1966.3) · p95 2171.6 | 1776.0 (1504.9–1848.6) · p95 2219.0 | - |
| wasm | lifecycle | kit.save | 108.4 (95.4–120.7) · p95 174.4 | 114.2 (90.8–136.8) · p95 148.8 | 120.9 (113.3–134.0) · p95 157.9 | - |
| wasm | lifecycle | kit.restore | 1767.5 (1509.5–1843.2) · p95 2110.2 | 1852.8 (1506.3–2055.3) · p95 2105.3 | 1841.0 (1563.6–1955.9) · p95 2500.1 | - |
| wasm | micro | timer.hrtime_pair | 0.1 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | micro | js.stringify_payload.first | 0.1 (0.1–0.3) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | micro | js.stringify_payload.last | 0.1 (0.1–0.2) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | micro | bridge.pass_ascii.payload | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | micro | kit.payloadText.last | 0.2 (0.2–0.2) · p95 0.2 | 0.2 (0.2–0.2) · p95 0.2 | 0.2 (0.2–0.2) · p95 0.2 | - |
| wasm | micro | bridge.decode.view | 0.4 (0.3–0.4) · p95 0.5 | 0.4 (0.3–0.4) · p95 0.5 | 0.3 (0.3–0.4) · p95 0.5 | 0.4 (0.3–0.4) · p95 0.5 |
| wasm | micro | bridge.pass_ascii.view | 2.6 (2.4–2.6) · p95 2.7 | 2.6 (2.2–2.6) · p95 2.7 | 2.3 (2.2–2.7) · p95 2.5 | 2.7 (2.3–2.9) · p95 2.7 |
| wasm | micro | js.parse.view | 7.7 (7.0–8.0) · p95 8.7 | 7.7 (6.7–8.4) · p95 8.1 | 8.0 (7.1–8.2) · p95 8.5 | 7.8 (6.9–9.0) · p95 8.3 |
| wasm | micro | bridge.decode.outcome | 0.2 (0.1–0.2) · p95 0.2 | 0.2 (0.1–0.2) · p95 0.2 | 0.2 (0.1–0.2) · p95 0.2 | - |
| wasm | micro | bridge.pass_ascii.outcome | 0.2 (0.2–0.2) · p95 0.2 | 0.2 (0.2–0.2) · p95 0.2 | 0.2 (0.2–0.2) · p95 0.2 | - |
| wasm | micro | js.parse.outcome | 0.4 (0.3–0.4) · p95 0.4 | 0.4 (0.3–0.4) · p95 0.4 | 0.4 (0.3–0.4) · p95 0.4 | - |
| wasm | micro | bridge.decode.snapshot | 2.4 (2.2–2.7) · p95 15.7 | 2.3 (2.2–2.6) · p95 16.4 | 2.3 (2.2–2.8) · p95 16.8 | 2.6 (2.2–3.0) · p95 2.9 |
| wasm | micro | bridge.pass_ascii.snapshot | 33.4 (28.5–34.3) · p95 35.4 | 33.4 (30.4–34.2) · p95 35.1 | 35.8 (29.5–36.3) · p95 38.6 | 34.2 (28.5–35.6) · p95 37.2 |
| wasm | micro | js.parse.snapshot | 58.3 (53.0–64.0) · p95 66.5 | 59.5 (57.6–65.3) · p95 64.5 | 60.2 (56.2–73.4) · p95 67.4 | 59.8 (52.5–73.9) · p95 68.2 |
| wasm | micro | bridge.decode.save | 0.7 (0.6–0.7) · p95 0.9 | 0.6 (0.6–0.7) · p95 0.8 | 0.7 (0.6–0.7) · p95 0.9 | 0.5 (0.5–0.6) · p95 0.7 |
| wasm | micro | bridge.pass_ascii.save | 5.6 (5.0–5.8) · p95 5.7 | 5.6 (5.4–5.8) · p95 5.8 | 5.8 (5.4–6.0) · p95 6.0 | 5.8 (5.0–6.4) · p95 5.9 |
| wasm | micro | js.parse.save | 15.9 (14.8–17.5) · p95 17.3 | 16.2 (15.6–18.4) · p95 17.3 | 16.6 (15.7–16.7) · p95 18.0 | 16.7 (14.6–21.2) · p95 21.8 |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 63.9 (50.6–64.4) · p95 164.2 | 58.7 (50.0–65.2) · p95 152.7 | - |
| native | dispatch_view_json | dispatch_view_json | - | 68.2 (53.2–77.0) · p95 164.8 | 71.2 (53.3–86.7) · p95 170.9 | - |
| native | dispatch_json | dispatch_json | - | 110.9 (83.6–112.6) · p95 215.7 | 110.5 (85.4–124.0) · p95 209.7 | - |
| native | dispatch_outcome_json | dispatch_outcome_json | - | 108.8 (85.9–116.1) · p95 211.2 | 111.5 (89.4–121.4) · p95 212.9 | - |
| native | web.dispatch_view | web.dispatch_view | - | 85.0 (63.3–104.4) · p95 189.9 | 92.5 (62.1–108.3) · p95 204.4 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 172.0 (128.5–190.8) · p95 286.4 | 153.4 (126.3–202.5) · p95 255.7 | - |
| native | web.dispatch | web.dispatch | - | 186.3 (184.2–239.1) · p95 327.3 | 165.0 (148.6–239.7) · p95 265.2 | - |
| native | read | clone | - | 11.3 (11.1–13.2) · p95 16.5 | 11.2 (11.0–13.5) · p95 13.4 | - |
| native | read | clone.drop | - | 6.1 (6.0–7.2) · p95 8.9 | 6.0 (6.0–7.3) · p95 7.2 | - |
| native | read | save | - | 6.1 (5.8–7.1) · p95 11.3 | 5.8 (5.8–7.4) · p95 10.2 | - |
| native | read | save_json | - | 10.4 (9.9–12.0) · p95 19.1 | 9.9 (9.8–12.3) · p95 17.1 | - |
| native | read | snapshot | - | 32.7 (32.3–38.7) · p95 47.5 | 32.5 (32.4–39.5) · p95 38.5 | - |
| native | read | snapshot.drop | - | 13.2 (12.9–15.6) · p95 19.4 | 12.9 (12.8–15.7) · p95 15.6 | - |
| native | read | snapshot.serialize | - | 20.5 (20.2–24.4) · p95 30.1 | 20.4 (20.4–24.9) · p95 24.4 | - |
| native | read | snapshot.serialize_pretty | - | 41.0 (40.4–48.8) · p95 60.2 | 40.9 (40.8–49.8) · p95 48.7 | - |
| native | read | view | - | 1.5 (1.4–1.7) · p95 2.7 | 1.4 (1.4–1.7) · p95 2.4 | - |
| native | read | view.serialize | - | 7.4 (7.1–8.6) · p95 11.4 | 7.2 (7.2–8.8) · p95 9.6 | - |
| native | web.read | web.save | - | 13.7 (11.6–14.2) · p95 23.4 | 12.4 (11.7–12.9) · p95 20.7 | - |
| native | web.read | web.snapshot | - | 98.9 (84.6–103.3) · p95 121.2 | 91.1 (85.3–95.1) · p95 105.3 | - |
| native | web.read | web.view | - | 7.7 (6.6–8.1) · p95 11.2 | 7.0 (6.5–7.3) · p95 9.9 | - |
| native | lifecycle | from_source | - | 3129.9 (3078.2–3960.7) · p95 3407.6 | 3431.5 (3101.0–3757.4) · p95 3611.3 | - |
| native | lifecycle | restore | - | 3233.8 (3190.3–4047.9) · p95 3661.3 | 3502.5 (3208.0–3818.9) · p95 3754.2 | - |
| native | lifecycle | restore.parse | - | 30.2 (26.7–30.8) · p95 35.4 | 29.3 (28.2–32.7) · p95 36.0 | - |
| native | lifecycle | restore_json | - | 3275.6 (3234.8–4075.3) · p95 3585.8 | 3615.0 (3255.5–3958.4) · p95 3900.1 | - |
| native | lifecycle | save | - | 19.7 (18.4–21.9) · p95 24.6 | 20.1 (18.2–22.2) · p95 24.4 | - |
| native | lifecycle | save_json | - | 17.9 (17.2–20.9) · p95 21.3 | 19.2 (17.3–20.5) · p95 22.6 | - |
| native | lifecycle | web.new | - | 3165.6 (3075.5–3927.8) · p95 3370.5 | 3427.8 (3122.0–3661.5) · p95 3809.4 | - |
| native | lifecycle | web.restore | - | 3241.1 (3214.5–4133.3) · p95 3629.0 | 3586.4 (3276.5–3862.8) · p95 4041.2 | - |
| native | lifecycle | web.save | - | 14.4 (13.3–16.8) · p95 17.6 | 15.7 (13.7–16.6) · p95 17.9 | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.7 (0.6–0.7) · p95 2.2 | 0.6 (0.6–0.7) · p95 2.1 | 0.7 (0.6–0.7) · p95 2.3 | - |
| wasm | raw.dispatch_view | raw.dispatch_view | 83.8 (79.1–87.8) · p95 169.8 | 79.1 (77.6–85.6) · p95 162.0 | 84.9 (78.9–89.9) · p95 174.0 | - |
| wasm | raw.dispatch_view | js.parse_view | 29.1 (27.2–30.3) · p95 39.2 | 28.4 (26.7–29.9) · p95 37.8 | 29.8 (26.6–30.9) · p95 39.3 | - |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 137.7 (130.4–144.5) · p95 209.3 | 131.5 (126.8–142.8) · p95 198.4 | 139.9 (128.9–146.8) · p95 212.3 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 160.8 (140.5–166.3) · p95 261.7 | 165.3 (143.8–166.2) · p95 265.1 | 151.3 (143.8–163.2) · p95 242.1 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 84.7 (76.2–87.2) · p95 105.6 | 85.4 (78.6–87.0) · p95 106.5 | 80.7 (77.3–88.1) · p95 94.9 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 248.4 (218.2–257.4) · p95 361.6 | 255.0 (224.9–255.8) · p95 365.4 | 233.7 (223.0–254.2) · p95 331.3 | - |
| wasm | raw.dispatch | raw.dispatch | 191.0 (170.5–194.4) · p95 287.0 | 193.3 (172.3–206.0) · p95 293.8 | 186.1 (169.4–197.2) · p95 278.1 | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.4 (0.4–0.4) · p95 1.1 | 0.4 (0.4–0.5) · p95 1.1 | 0.4 (0.4–0.5) · p95 1.1 | - |
| wasm | abi.dispatch_view | abi.exec | 74.6 (72.8–77.7) · p95 158.0 | 79.4 (72.5–81.5) · p95 166.2 | 75.5 (72.1–81.0) · p95 159.8 | - |
| wasm | abi.dispatch_view | abi.decode | 1.3 (1.3–1.4) · p95 4.4 | 1.3 (1.3–1.4) · p95 4.2 | 1.3 (1.3–1.4) · p95 4.3 | - |
| wasm | abi.dispatch_view | abi.free | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.3 | - |
| wasm | abi.dispatch_view | abi.dispatch_view | 103.1 (100.3–107.0) · p95 162.7 | 107.0 (98.4–108.6) · p95 172.1 | 104.2 (99.1–110.9) · p95 164.8 | - |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.5 (0.4–0.5) · p95 1.2 | 0.5 (0.4–0.5) · p95 1.2 | 0.5 (0.4–0.5) · p95 1.2 | - |
| wasm | abi.dispatch_outcome | abi.exec | 146.1 (140.7–152.9) · p95 239.4 | 140.9 (138.8–152.0) · p95 224.8 | 143.5 (137.6–160.1) · p95 232.7 | - |
| wasm | abi.dispatch_outcome | abi.decode | 3.3 (3.3–3.8) · p95 8.1 | 3.3 (3.2–3.7) · p95 7.7 | 3.3 (3.3–3.9) · p95 8.3 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.4 (0.4–0.5) · p95 0.7 | 0.4 (0.4–0.5) · p95 0.7 | 0.4 (0.4–0.5) · p95 0.7 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 151.8 (145.9–159.5) · p95 247.2 | 146.3 (144.0–158.9) · p95 232.5 | 149.1 (143.1–167.3) · p95 240.6 | - |
| wasm | read | raw.view | 11.3 (10.4–11.3) · p95 15.5 | 11.2 (10.9–11.8) · p95 15.3 | 11.3 (10.6–11.9) · p95 15.5 | - |
| wasm | read | js.parse_view | 29.3 (26.6–29.9) · p95 36.6 | 28.9 (28.4–29.9) · p95 38.3 | 29.3 (27.6–30.5) · p95 38.1 | - |
| wasm | read | raw.snapshot | 121.2 (110.2–121.6) · p95 145.7 | 117.8 (116.1–122.5) · p95 142.3 | 117.8 (114.6–125.7) · p95 149.8 | - |
| wasm | read | js.parse_snapshot | 92.9 (83.5–93.1) · p95 105.9 | 90.1 (89.0–93.0) · p95 105.7 | 91.6 (86.7–94.9) · p95 111.2 | - |
| wasm | read | raw.save | 16.5 (14.9–16.6) · p95 28.2 | 16.3 (15.7–16.8) · p95 28.3 | 16.4 (15.5–17.1) · p95 28.6 | - |
| wasm | abi.read | abi.view.exec | 10.6 (9.9–11.7) · p95 14.5 | 10.4 (9.9–10.7) · p95 13.8 | 10.5 (9.9–10.6) · p95 14.0 | - |
| wasm | abi.read | abi.view.decode | 1.0 (0.9–1.1) · p95 1.6 | 0.9 (0.9–1.0) · p95 1.4 | 0.9 (0.9–1.0) · p95 1.5 | - |
| wasm | abi.read | abi.snapshot.exec | 112.9 (105.7–121.3) · p95 140.4 | 110.5 (104.7–113.1) · p95 129.1 | 112.6 (105.4–115.6) · p95 132.8 | - |
| wasm | abi.read | abi.snapshot.decode | 4.9 (4.7–5.5) · p95 8.5 | 5.0 (5.0–5.7) · p95 7.6 | 5.1 (4.8–5.5) · p95 7.8 | - |
| wasm | abi.read | abi.save.exec | 15.6 (14.3–17.2) · p95 27.3 | 14.8 (14.4–15.4) · p95 25.4 | 15.4 (14.3–15.5) · p95 26.4 | - |
| wasm | abi.read | abi.save.decode | 0.5 (0.5–0.6) · p95 1.0 | 0.5 (0.5–0.5) · p95 0.9 | 0.5 (0.5–0.5) · p95 0.9 | - |
| wasm | kit | kit.dispatch | 260.2 (252.1–296.0) · p95 369.8 | 271.4 (254.5–294.7) · p95 396.6 | 268.2 (256.8–300.2) · p95 380.8 | - |
| wasm | kit | kit.view | 40.0 (38.9–45.8) · p95 51.7 | 41.6 (39.1–45.0) · p95 56.4 | 41.1 (39.1–46.4) · p95 53.2 | - |
| wasm | kit | kit.dispatch+view | 303.4 (293.7–342.1) · p95 419.4 | 316.2 (296.4–341.8) · p95 451.3 | 313.9 (299.0–346.1) · p95 431.5 | - |
| wasm | kit.read | kit.view | 40.8 (39.6–50.2) · p95 51.7 | 40.7 (39.2–53.6) · p95 51.2 | 45.3 (42.1–49.8) · p95 61.7 | - |
| wasm | kit.read | kit.snapshot | 214.1 (205.0–259.5) · p95 260.7 | 213.9 (204.4–265.1) · p95 253.8 | 236.8 (218.4–255.1) · p95 319.8 | - |
| wasm | kit.read | kit.save | 32.5 (31.5–42.3) · p95 53.8 | 32.6 (31.3–44.9) · p95 53.2 | 36.4 (33.1–41.9) · p95 62.4 | - |
| wasm | lifecycle | raw.new | 4367.7 (3844.3–4593.2) · p95 5015.5 | 3920.1 (3825.6–4159.1) · p95 4441.5 | 3230.5 (3101.2–4064.2) · p95 4210.3 | - |
| wasm | lifecycle | raw.save | 59.6 (54.3–102.8) · p95 81.0 | 58.8 (55.4–75.5) · p95 79.5 | 49.0 (45.6–82.6) · p95 78.1 | - |
| wasm | lifecycle | raw.restore | 4384.2 (3857.6–4572.9) · p95 4794.0 | 3968.3 (3817.8–4141.5) · p95 4463.8 | 3245.8 (3112.1–4220.1) · p95 4165.5 | - |
| wasm | lifecycle | kit.open | 3932.6 (3821.4–4171.0) · p95 4809.9 | 3988.4 (3703.9–4026.3) · p95 4289.9 | 3154.7 (3028.2–4116.8) · p95 3776.0 | - |
| wasm | lifecycle | kit.save | 119.4 (112.9–172.4) · p95 161.0 | 120.4 (117.4–159.8) · p95 160.6 | 103.0 (96.4–155.7) · p95 160.1 | - |
| wasm | lifecycle | kit.restore | 4234.8 (3809.1–4526.8) · p95 4838.7 | 3976.0 (3782.4–4108.4) · p95 4478.3 | 3260.7 (3109.7–4162.8) · p95 3819.7 | - |
| wasm | micro | timer.hrtime_pair | 0.2 (0.1–0.2) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.3) · p95 0.3 | - |
| wasm | micro | js.stringify_payload.first | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.2) · p95 0.2 | 0.1 (0.1–0.3) · p95 0.2 | - |
| wasm | micro | js.stringify_payload.last | 0.0 (0.0–0.1) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.1) · p95 0.0 | - |
| wasm | micro | bridge.pass_ascii.payload | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | - |
| wasm | micro | kit.payloadText.last | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | - |
| wasm | micro | bridge.decode.view | 0.7 (0.6–0.9) · p95 0.9 | 0.6 (0.5–0.7) · p95 0.8 | 0.7 (0.6–1.1) · p95 1.1 | - |
| wasm | micro | bridge.pass_ascii.view | 7.4 (5.9–7.7) · p95 8.0 | 7.3 (5.9–8.1) · p95 8.0 | 6.9 (5.4–15.8) · p95 7.8 | - |
| wasm | micro | js.parse.view | 23.7 (21.0–31.3) · p95 28.6 | 28.6 (26.4–32.2) · p95 34.2 | 22.7 (20.9–38.4) · p95 29.2 | - |
| wasm | micro | bridge.decode.outcome | 1.6 (1.6–2.3) · p95 3.1 | 1.5 (1.5–2.6) · p95 2.9 | 1.6 (1.4–4.8) · p95 2.4 | - |
| wasm | micro | bridge.pass_ascii.outcome | 20.6 (20.2–24.2) · p95 22.6 | 21.6 (21.5–27.8) · p95 25.9 | 21.5 (19.0–24.8) · p95 23.4 | - |
| wasm | micro | js.parse.outcome | 75.4 (69.5–96.5) · p95 81.8 | 78.0 (75.8–116.4) · p95 86.6 | 75.3 (68.2–93.4) · p95 83.3 | - |
| wasm | micro | bridge.decode.snapshot | 2.6 (2.6–2.6) · p95 3.5 | 2.9 (2.8–5.5) · p95 5.8 | 2.5 (2.2–7.8) · p95 3.2 | - |
| wasm | micro | bridge.pass_ascii.snapshot | 36.1 (35.2–47.2) · p95 40.0 | 37.5 (35.6–51.3) · p95 41.5 | 38.0 (33.6–47.3) · p95 43.0 | - |
| wasm | micro | js.parse.snapshot | 79.3 (73.7–104.5) · p95 88.6 | 83.0 (79.7–117.7) · p95 91.9 | 86.9 (84.1–105.6) · p95 94.2 | - |
| wasm | micro | bridge.decode.save | 0.3 (0.3–0.4) · p95 0.4 | 0.3 (0.3–0.3) · p95 0.4 | 0.3 (0.3–0.4) · p95 0.4 | - |
| wasm | micro | bridge.pass_ascii.save | 2.6 (2.2–6.5) · p95 2.9 | 2.6 (2.4–2.6) · p95 2.8 | 2.7 (2.6–2.8) · p95 3.0 | - |
| wasm | micro | js.parse.save | 8.4 (7.2–8.9) · p95 9.7 | 8.5 (8.3–21.9) · p95 9.6 | 8.7 (8.3–11.0) · p95 9.9 | - |

