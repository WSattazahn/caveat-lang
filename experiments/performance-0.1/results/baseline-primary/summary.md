# Caveat performance baseline: 2026-09-29T05-29-28-544-baseline

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=baseline --repeats=3 --build=never --keep-samples --target=rc4-published=package:C:\Users\walte\caveat-rc4\package-files\package --target=rc4-local=tree:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF-rc4 --target=main-local=tree:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF --target=hist-e6ace96=runtime:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\perf-scratch\scout\hist-e6ace96\pkg-reactive --out=C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\perf-scratch\m\runs\baseline3`

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
- Load before: total CPU 7/5/0%; busiest over the window: claude 1.375s, powershell 0.625s, claude 0.609s, ChatGPT 0.25s, IntelGraphicsSoftware 0.141s, Reallusion Hub 0.078s
- Load after: total CPU 1/15/8%; busiest over the window: claude 1.188s, powershell 0.625s, claude 0.297s, ChatGPT 0.172s, IntelGraphicsSoftware 0.125s, ChatGPT 0.078s
- Load during: typeperf every 5 s for the whole run, 604 samples: total mean 8.7% p95 10.6% max 21.6%; processor 10 mean 62.6% p95 99.1% max 99.7%; processor 11 mean 19.8% p95 44.4% max 87.8%; processor 12 mean 19.7% p95 47.6% max 86%; processor 13 mean 25.6% p95 99.7% max 100%; total above 10% in 67 and above 25% in 0 samples; pinned processors summing above 150% in 37

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
| Published method: adapter dispatch + view | 46.6 | 46.4 | 46.6 | 44.7 |
| Adapter dispatch (stringify, WebAssembly dispatch_view, JSON.parse) | 43.9 | 43.3 | 43.6 | 42.2 |
| Adapter view() (JavaScript reshaping only) | 2.2 | 2.2 | 2.2 | 2.2 |
| JSON.stringify(payload) | 0.3 | 0.3 | 0.3 | 0.3 |
| wasm-bindgen dispatch_view call, returning the view text | 26.1 | 26.0 | 26.3 | 24.9 |
|   arguments copied into WebAssembly memory | 0.1 | 0.1 | 0.1 | 0.1 |
|   WebAssembly execution (resolve, apply, view, serialize) | 22.7 | 22.4 | 22.5 | 21.2 |
|   result decoded to a JavaScript string | 2.6 | 2.5 | 2.5 | 2.5 |
|   result freed | 0.1 | 0.1 | 0.1 | 0.1 |
|   the four pieces, timed together in one process | 25.5 | 25.2 | 25.2 | 23.9 |
| JSON.parse(view text) | 17.1 | 17.3 | 17.1 | 17.0 |
| WebAssembly view() alone, execution only | 8.3 | 8.8 | 8.5 | 7.5 |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | 135.4 | 141.3 | 137.6 | - |
| Kit dispatch + view() | 163.7 | 171.1 | 166.4 | - |
| Native web::dispatch_view (the exported function, natively) | - | 20.0 | 20.0 | - |
| Native apply (numeric parameters: rules and bindings only) | - | 13.1 | 13.0 | - |
| Native dispatch_view_json (resolve + apply + view, not serialized) | - | 15.7 | 15.8 | - |
| Native view() build | - | 2.5 | 2.6 | - |
| Native view serialize | - | 4.6 | 5.0 | - |
| Native snapshot() build | - | 24.8 | 26.4 | - |
| Native session clone (upper bound of the transaction copy) | - | 7.1 | 7.6 | - |
| Native apply, the same program without bindings (glowcap-unbound) | - | 12.0 | 12.8 | - |
| *derived:* native resolve_payload ≈ dispatch_view_json − apply − view | - | 0.1 | 0.2 | - |
| *derived:* native binding evaluation ≈ apply − apply without bindings | - | 1.1 | 0.2 | - |
| *derived:* WebAssembly ÷ native, same exported function (ratio) | - | 1.1 | 1.1 | - |

## glowcap-replay

The Glowcap replay program and the exact event stream behind the published 51.6 us figure (experiments/glowcap/harness.mjs bench(), runtime/examples/profile_dispatch.rs): absorb cave glowcap, absorb pool duskcap, taste ruin glowcap, then 9,997 ticks of dt 0.05. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 13.1 (13.1–13.8) · p95 16.0 | 13.1 (13.0–13.8) · p95 16.1 | - |
| native | dispatch_view_json | dispatch_view_json | - | 15.8 (15.7–16.7) · p95 18.1 | 15.9 (15.8–16.8) · p95 18.4 | - |
| native | dispatch_json | dispatch_json | - | 36.8 (36.6–39.0) · p95 40.2 | 36.6 (36.3–38.6) · p95 40.7 | - |
| native | dispatch_outcome_json | dispatch_outcome_json | - | 36.5 (36.3–39.4) · p95 40.1 | 38.5 (36.7–39.3) · p95 41.9 | - |
| native | web.dispatch_view | web.dispatch_view | - | 20.1 (20.0–21.3) · p95 22.8 | 20.0 (20.0–21.3) · p95 22.5 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 64.0 (63.6–68.2) · p95 69.0 | 64.7 (64.4–70.2) · p95 69.2 | - |
| native | web.dispatch | web.dispatch | - | 80.2 (79.9–89.3) · p95 85.7 | 81.6 (81.5–86.7) · p95 90.7 | - |
| native | read | clone | - | 7.0 (7.0–7.6) · p95 7.6 | 7.6 (7.1–7.6) · p95 8.1 | - |
| native | read | clone.drop | - | 3.8 (3.8–4.2) · p95 4.1 | 4.1 (3.8–4.1) · p95 4.4 | - |
| native | read | save | - | 9.0 (8.9–9.6) · p95 9.7 | 9.6 (8.9–10.0) · p95 10.4 | - |
| native | read | save_json | - | 14.5 (14.5–15.7) · p95 15.6 | 15.6 (14.7–15.8) · p95 16.7 | - |
| native | read | snapshot | - | 24.7 (24.6–26.7) · p95 26.7 | 26.2 (24.5–27.0) · p95 28.4 | - |
| native | read | snapshot.drop | - | 10.6 (10.6–11.5) · p95 11.5 | 11.3 (10.6–11.5) · p95 12.1 | - |
| native | read | snapshot.serialize | - | 14.8 (14.7–16.0) · p95 16.1 | 15.9 (15.0–16.2) · p95 17.3 | - |
| native | read | snapshot.serialize_pretty | - | 31.1 (31.0–33.7) · p95 33.7 | 33.5 (31.6–34.2) · p95 36.3 | - |
| native | read | view | - | 2.5 (2.5–2.7) · p95 2.7 | 2.6 (2.4–2.6) · p95 2.9 | - |
| native | read | view.serialize | - | 4.6 (4.6–5.0) · p95 5.0 | 5.0 (4.7–5.1) · p95 5.4 | - |
| native | web.read | web.save | - | 16.5 (15.4–16.6) · p95 17.7 | 16.8 (15.5–17.3) · p95 17.9 | - |
| native | web.read | web.snapshot | - | 70.4 (65.8–70.4) · p95 74.9 | 70.7 (65.3–72.3) · p95 76.4 | - |
| native | web.read | web.view | - | 6.8 (6.3–6.8) · p95 7.3 | 6.8 (6.3–6.9) · p95 7.3 | - |
| native | lifecycle | from_source | - | 2632.1 (2519.7–2668.7) · p95 2839.7 | 2711.0 (2710.9–2848.2) · p95 2918.2 | - |
| native | lifecycle | restore | - | 2737.1 (2662.9–2799.4) · p95 2865.5 | 2830.8 (2824.2–2938.8) · p95 2974.7 | - |
| native | lifecycle | restore.parse | - | 25.9 (25.7–26.3) · p95 30.5 | 26.4 (24.9–27.0) · p95 30.6 | - |
| native | lifecycle | restore_json | - | 2818.4 (2690.1–2845.3) · p95 2918.4 | 2952.1 (2917.3–3031.1) · p95 3067.2 | - |
| native | lifecycle | save | - | 21.4 (19.8–24.0) · p95 28.4 | 23.7 (20.7–24.5) · p95 32.8 | - |
| native | lifecycle | save_json | - | 27.1 (24.2–37.6) · p95 34.0 | 29.1 (27.4–30.8) · p95 43.8 | - |
| native | lifecycle | web.new | - | 2604.7 (2504.2–2654.7) · p95 2739.5 | 2723.2 (2707.1–2809.4) · p95 2871.1 | - |
| native | lifecycle | web.restore | - | 2808.5 (2637.4–2879.3) · p95 2892.4 | 2927.0 (2902.1–3041.9) · p95 3036.4 | - |
| native | lifecycle | web.save | - | 18.8 (18.5–19.1) · p95 21.4 | 19.0 (18.9–20.8) · p95 25.3 | - |
| wasm | published-method | adapter.dispatch+view | 46.8 (46.2–47.0) · p95 57.2 | 46.6 (46.0–46.7) · p95 56.1 | 46.7 (43.8–47.2) · p95 56.6 | 44.8 (41.3–45.1) · p95 53.2 |
| wasm | adapter | adapter.dispatch | 44.0 (43.6–44.2) · p95 49.7 | 43.3 (41.2–43.9) · p95 49.6 | 43.7 (40.9–43.7) · p95 49.0 | 42.2 (39.3–42.5) · p95 47.7 |
| wasm | adapter | adapter.view | 2.3 (2.3–2.3) · p95 3.4 | 2.2 (2.2–2.3) · p95 3.6 | 2.2 (2.2–2.3) · p95 3.6 | 2.2 (2.1–2.2) · p95 3.4 |
| wasm | adapter | adapter.dispatch+view | 46.3 (45.9–46.5) · p95 52.7 | 45.6 (43.4–46.3) · p95 52.8 | 46.0 (43.1–46.1) · p95 52.1 | 44.4 (41.5–44.8) · p95 50.4 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 26.2 (26.2–26.3) · p95 30.0 | 26.1 (25.1–26.2) · p95 29.9 | 26.4 (24.6–26.9) · p95 30.2 | 25.0 (23.3–25.0) · p95 28.4 |
| wasm | raw.dispatch_view | js.parse_view | 17.1 (16.8–17.2) · p95 19.1 | 17.2 (16.6–17.2) · p95 19.3 | 17.1 (16.1–17.9) · p95 19.2 | 16.9 (15.9–17.1) · p95 18.9 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 43.8 (43.5–43.9) · p95 48.9 | 43.7 (42.2–43.9) · p95 48.9 | 43.9 (41.0–45.3) · p95 49.0 | 42.4 (39.6–42.5) · p95 47.1 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 74.0 (72.1–75.1) · p95 89.0 | 73.7 (73.4–73.9) · p95 85.3 | 73.8 (72.6–81.0) · p95 86.1 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 65.7 (64.1–67.0) · p95 75.3 | 64.4 (64.4–65.2) · p95 71.5 | 64.7 (64.7–71.4) · p95 72.4 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 139.9 (136.8–142.5) · p95 163.3 | 138.8 (138.2–139.0) · p95 153.9 | 139.1 (137.8–152.8) · p95 157.4 | - |
| wasm | raw.dispatch | raw.dispatch | 103.1 (101.5–105.5) · p95 118.5 | 101.0 (97.9–101.4) · p95 113.4 | 100.3 (97.2–102.6) · p95 114.8 | 99.8 (95.1–103.9) · p95 113.7 |
| wasm | abi.dispatch_view | abi.encode_args | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 |
| wasm | abi.dispatch_view | abi.exec | 22.8 (22.6–23.3) · p95 26.1 | 22.5 (21.8–22.7) · p95 26.1 | 22.6 (21.2–22.6) · p95 26.2 | 21.3 (20.5–21.7) · p95 26.6 |
| wasm | abi.dispatch_view | abi.decode | 2.6 (2.5–2.6) · p95 3.2 | 2.5 (2.5–2.5) · p95 3.1 | 2.5 (2.4–2.5) · p95 3.2 | 2.5 (2.4–2.5) · p95 3.2 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | abi.dispatch_view | abi.dispatch_view | 25.6 (25.4–26.1) · p95 29.3 | 25.3 (24.5–25.5) · p95 29.2 | 25.4 (23.9–25.4) · p95 29.7 | 24.1 (23.1–24.5) · p95 30.1 |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.2 (0.2–0.2) · p95 0.3 | 0.2 (0.1–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.3 | - |
| wasm | abi.dispatch_outcome | abi.exec | 61.3 (58.3–61.6) · p95 69.6 | 60.7 (58.6–63.1) · p95 70.0 | 61.0 (60.9–61.4) · p95 68.4 | - |
| wasm | abi.dispatch_outcome | abi.decode | 7.7 (7.4–7.9) · p95 10.7 | 7.8 (7.5–8.0) · p95 10.6 | 7.8 (7.7–7.8) · p95 10.3 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 69.7 (66.2–69.9) · p95 80.2 | 69.0 (66.6–71.7) · p95 80.1 | 69.2 (69.1–69.8) · p95 79.0 | - |
| wasm | read | raw.view | 11.3 (11.0–12.0) · p95 13.1 | 11.5 (11.0–12.3) · p95 13.1 | 11.4 (11.1–11.4) · p95 12.9 | 10.3 (10.0–10.8) · p95 11.8 |
| wasm | read | js.parse_view | 18.9 (18.1–19.2) · p95 21.6 | 18.7 (18.1–19.9) · p95 21.3 | 18.1 (18.1–18.6) · p95 20.8 | 18.0 (17.5–18.4) · p95 20.6 |
| wasm | read | raw.snapshot | 93.4 (90.4–97.6) · p95 111.6 | 95.0 (90.0–100.8) · p95 108.1 | 90.5 (89.9–95.0) · p95 107.3 | 93.0 (92.4–95.0) · p95 109.5 |
| wasm | read | js.parse_snapshot | 71.7 (69.2–72.8) · p95 81.0 | 72.1 (68.4–76.3) · p95 80.2 | 69.0 (68.7–71.8) · p95 78.9 | 69.3 (68.0–71.7) · p95 78.8 |
| wasm | read | raw.save | 19.2 (18.7–20.1) · p95 23.9 | 19.5 (18.8–20.6) · p95 23.2 | 18.9 (18.8–19.3) · p95 23.3 | 18.7 (18.3–18.9) · p95 22.8 |
| wasm | abi.read | abi.view.exec | 8.3 (8.2–8.6) · p95 9.3 | 8.7 (8.2–8.8) · p95 9.5 | 8.5 (8.2–8.7) · p95 9.4 | 7.5 (7.4–7.9) · p95 8.6 |
| wasm | abi.read | abi.view.decode | 2.3 (2.3–2.4) · p95 2.8 | 2.3 (2.2–2.4) · p95 2.8 | 2.3 (2.2–2.4) · p95 2.8 | 2.2 (2.2–2.3) · p95 2.7 |
| wasm | abi.read | abi.snapshot.exec | 70.2 (69.0–73.3) · p95 82.3 | 71.8 (68.4–74.0) · p95 82.7 | 69.3 (68.8–73.1) · p95 81.1 | 72.0 (71.1–75.4) · p95 84.5 |
| wasm | abi.read | abi.snapshot.decode | 12.0 (11.9–12.5) · p95 16.4 | 12.5 (12.0–12.6) · p95 16.3 | 12.0 (11.9–12.6) · p95 16.2 | 11.9 (11.8–12.4) · p95 16.1 |
| wasm | abi.read | abi.save.exec | 14.9 (14.8–15.8) · p95 18.1 | 15.7 (14.8–16.1) · p95 19.1 | 15.1 (15.0–15.9) · p95 18.7 | 15.1 (15.0–15.8) · p95 18.9 |
| wasm | abi.read | abi.save.decode | 0.6 (0.6–0.6) · p95 0.8 | 0.6 (0.6–0.6) · p95 0.8 | 0.6 (0.6–0.6) · p95 0.8 | 0.6 (0.6–0.6) · p95 0.9 |
| wasm | kit | kit.dispatch | 135.2 (134.5–141.6) · p95 158.6 | 140.7 (135.6–141.9) · p95 160.2 | 137.4 (135.4–145.1) · p95 161.1 | - |
| wasm | kit | kit.view | 28.2 (28.0–29.4) · p95 33.1 | 29.5 (28.3–29.6) · p95 33.5 | 28.7 (28.3–30.2) · p95 33.7 | - |
| wasm | kit | kit.dispatch+view | 163.5 (162.7–171.2) · p95 191.4 | 170.4 (164.0–171.7) · p95 193.3 | 166.2 (163.9–175.6) · p95 193.9 | - |
| wasm | kit.read | kit.view | 30.8 (30.5–32.6) · p95 36.3 | 31.2 (30.3–32.5) · p95 35.5 | 31.5 (30.8–32.5) · p95 36.4 | - |
| wasm | kit.read | kit.snapshot | 170.5 (169.0–176.7) · p95 202.6 | 172.1 (166.6–176.2) · p95 203.3 | 169.6 (168.2–176.9) · p95 201.4 | - |
| wasm | kit.read | kit.save | 40.7 (40.2–42.8) · p95 49.4 | 41.1 (39.9–42.1) · p95 49.0 | 40.6 (40.4–42.9) · p95 49.3 | - |
| wasm | lifecycle | raw.new | 2744.1 (2641.6–2781.3) · p95 2985.2 | 2539.8 (2441.0–2594.9) · p95 2852.1 | 2576.3 (2495.8–2576.5) · p95 2878.9 | 2455.1 (2376.4–2515.5) · p95 2721.1 |
| wasm | lifecycle | raw.save | 48.9 (48.6–49.5) · p95 67.6 | 46.1 (45.9–46.6) · p95 57.2 | 47.7 (47.3–48.6) · p95 59.3 | 43.1 (42.5–46.0) · p95 66.8 |
| wasm | lifecycle | raw.restore | 2829.7 (2730.7–2879.4) · p95 3123.6 | 2651.4 (2567.1–2672.0) · p95 2906.5 | 2635.4 (2590.2–2687.9) · p95 3178.5 | 2598.4 (2454.4–2623.3) · p95 2917.2 |
| wasm | lifecycle | kit.open | 2693.6 (2548.3–2752.9) · p95 2934.0 | 2477.9 (2408.6–2532.4) · p95 2699.1 | 2485.9 (2393.6–2487.2) · p95 2821.7 | - |
| wasm | lifecycle | kit.save | 94.3 (87.6–101.9) · p95 130.2 | 89.8 (87.4–90.3) · p95 119.7 | 90.4 (89.4–95.4) · p95 120.1 | - |
| wasm | lifecycle | kit.restore | 2773.6 (2688.6–2836.0) · p95 2956.7 | 2596.9 (2512.7–2667.2) · p95 2856.9 | 2609.0 (2554.1–2614.1) · p95 3003.1 | - |
| wasm | micro | timer.hrtime_pair | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | micro | js.stringify_payload.first | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | micro | js.stringify_payload.last | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | micro | bridge.pass_ascii.payload | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | micro | kit.payloadText.last | 0.2 (0.2–0.2) · p95 0.2 | 0.2 (0.2–0.2) · p95 0.2 | 0.2 (0.2–0.2) · p95 0.2 | - |
| wasm | micro | bridge.decode.view | 2.3 (2.2–2.4) · p95 2.7 | 2.1 (2.1–2.2) · p95 2.5 | 2.0 (1.8–2.2) · p95 2.5 | 2.1 (1.8–2.1) · p95 7.2 |
| wasm | micro | bridge.pass_ascii.view | 4.5 (4.4–4.8) · p95 4.8 | 4.1 (3.7–4.6) · p95 4.3 | 4.0 (3.8–4.4) · p95 4.3 | 4.0 (3.8–4.2) · p95 4.2 |
| wasm | micro | js.parse.view | 16.7 (16.4–17.9) · p95 18.7 | 16.2 (15.7–16.9) · p95 18.6 | 15.8 (15.3–16.8) · p95 17.2 | 15.4 (15.2–16.9) · p95 17.8 |
| wasm | micro | bridge.decode.outcome | 6.3 (5.7–6.6) · p95 7.6 | 6.5 (6.1–6.6) · p95 7.4 | 6.2 (6.1–6.5) · p95 6.7 | - |
| wasm | micro | bridge.pass_ascii.outcome | 17.7 (16.6–20.4) · p95 19.2 | 17.8 (17.1–20.0) · p95 20.6 | 18.2 (16.9–18.4) · p95 19.3 | - |
| wasm | micro | js.parse.outcome | 63.8 (60.0–64.6) · p95 70.8 | 62.5 (60.6–66.8) · p95 71.9 | 60.5 (60.2–65.0) · p95 67.8 | - |
| wasm | micro | bridge.decode.snapshot | 10.3 (10.1–11.1) · p95 13.0 | 10.3 (10.0–10.8) · p95 11.4 | 10.2 (10.2–10.9) · p95 11.7 | 11.0 (10.8–11.4) · p95 12.0 |
| wasm | micro | bridge.pass_ascii.snapshot | 30.4 (28.6–31.8) · p95 32.7 | 31.2 (29.3–33.5) · p95 34.4 | 31.0 (28.9–31.4) · p95 33.2 | 33.2 (32.5–34.1) · p95 35.9 |
| wasm | micro | js.parse.snapshot | 66.4 (63.8–67.6) · p95 72.6 | 64.6 (64.6–69.8) · p95 71.1 | 69.7 (63.9–74.5) · p95 75.4 | 74.3 (69.7–74.4) · p95 80.5 |
| wasm | micro | bridge.decode.save | 0.4 (0.4–0.4) · p95 0.5 | 0.4 (0.4–0.4) · p95 0.6 | 0.4 (0.4–0.5) · p95 0.6 | 0.5 (0.4–0.5) · p95 0.6 |
| wasm | micro | bridge.pass_ascii.save | 3.8 (3.6–3.9) · p95 4.0 | 3.9 (3.8–4.2) · p95 4.0 | 4.2 (3.8–4.4) · p95 4.4 | 4.3 (4.0–4.5) · p95 4.7 |
| wasm | micro | js.parse.save | 13.0 (12.6–13.4) · p95 14.2 | 13.6 (12.9–13.7) · p95 14.8 | 13.7 (13.0–14.9) · p95 15.2 | 14.4 (13.6–14.6) · p95 15.6 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | rc4-local | 83.8 (82.1–84.6) | 16.3 (16.2–17.2) | 13.4 (13.3–14.2) | 13.1 (13.0–13.8) |
| native | apply | apply | main-local | 79.9 (78.5–88.2) | 16.5 (16.2–17.0) | 13.8 (13.3–14.2) | 13.0 (12.9–13.8) |
| native | dispatch_view_json | dispatch_view_json | rc4-local | 92.8 (90.3–95.7) | 18.3 (18.2–19.2) | 15.9 (15.9–16.7) | 15.7 (15.7–16.6) |
| native | dispatch_view_json | dispatch_view_json | main-local | 93.1 (87.4–97.5) | 18.6 (18.4–19.6) | 16.0 (15.9–17.0) | 15.8 (15.7–16.8) |
| native | dispatch_json | dispatch_json | rc4-local | 122.7 (122.3–125.4) | 38.2 (37.7–39.7) | 35.3 (35.2–37.1) | 36.9 (36.6–39.1) |
| native | dispatch_json | dispatch_json | main-local | 121.1 (119.9–128.5) | 38.0 (37.6–39.4) | 35.4 (34.8–36.8) | 36.6 (36.3–38.6) |
| native | dispatch_outcome_json | dispatch_outcome_json | rc4-local | 121.2 (118.0–129.8) | 37.5 (37.2–41.1) | 35.1 (34.9–38.3) | 36.5 (36.3–39.4) |
| native | dispatch_outcome_json | dispatch_outcome_json | main-local | 133.2 (118.6–138.3) | 39.6 (38.1–39.9) | 36.9 (35.2–37.6) | 38.6 (36.7–39.3) |
| native | web.dispatch_view | web.dispatch_view | rc4-local | 113.9 (111.1–113.9) | 22.2 (22.1–23.9) | 20.1 (19.9–21.3) | 20.0 (20.0–21.3) |
| native | web.dispatch_view | web.dispatch_view | main-local | 111.8 (108.6–118.1) | 22.3 (22.1–23.7) | 19.9 (19.9–21.3) | 20.0 (19.9–21.3) |
| native | web.dispatch_outcome | web.dispatch_outcome | rc4-local | 148.6 (147.2–163.0) | 61.7 (61.1–65.7) | 61.1 (60.7–64.6) | 64.2 (63.8–68.6) |
| native | web.dispatch_outcome | web.dispatch_outcome | main-local | 160.6 (159.1–175.2) | 63.0 (62.5–68.0) | 61.9 (61.7–67.0) | 64.9 (64.6–70.5) |
| native | web.dispatch | web.dispatch | rc4-local | 175.0 (167.7–176.7) | 77.8 (77.0–84.2) | 78.0 (77.3–84.7) | 80.3 (80.0–89.6) |
| native | web.dispatch | web.dispatch | main-local | 166.7 (166.5–175.3) | 78.0 (78.0–83.1) | 78.7 (78.6–83.7) | 81.8 (81.8–86.9) |
| native | read | clone | rc4-local | 6.7 (6.7–7.3) | 5.9 (5.9–6.4) | 6.2 (6.2–6.7) | 7.1 (7.1–7.7) |
| native | read | clone | main-local | 7.3 (6.9–7.4) | 6.5 (6.1–6.5) | 6.6 (6.2–6.6) | 7.6 (7.1–7.7) |
| native | read | clone.drop | rc4-local | 3.7 (3.7–4.0) | 3.3 (3.3–3.6) | 3.4 (3.4–3.7) | 3.9 (3.8–4.2) |
| native | read | clone.drop | main-local | 4.0 (3.8–4.1) | 3.6 (3.4–3.6) | 3.7 (3.4–3.7) | 4.1 (3.8–4.2) |
| native | read | save | rc4-local | 5.7 (5.7–6.3) | 6.3 (6.2–6.7) | 8.4 (8.4–9.0) | 9.0 (8.9–9.7) |
| native | read | save | main-local | 5.9 (5.7–6.1) | 6.7 (6.3–6.8) | 9.0 (8.5–9.4) | 9.6 (9.0–10.0) |
| native | read | save_json | rc4-local | 9.4 (9.4–10.3) | 10.4 (10.4–11.2) | 14.0 (13.9–14.9) | 14.6 (14.5–15.8) |
| native | read | save_json | main-local | 10.0 (9.0–10.2) | 11.3 (10.7–11.3) | 14.9 (14.1–15.0) | 15.7 (14.7–15.9) |
| native | read | snapshot | rc4-local | 35.0 (33.9–35.1) | 21.5 (21.5–23.2) | 23.2 (23.1–24.8) | 24.8 (24.7–26.9) |
| native | read | snapshot | main-local | 30.9 (26.7–31.2) | 23.0 (21.5–23.1) | 24.4 (23.0–25.0) | 26.4 (24.6–27.2) |
| native | read | snapshot.drop | rc4-local | 10.2 (10.2–11.2) | 9.2 (9.2–9.9) | 9.9 (9.8–10.6) | 10.7 (10.7–11.6) |
| native | read | snapshot.drop | main-local | 10.8 (10.2–11.0) | 9.9 (9.3–9.9) | 10.5 (9.9–10.6) | 11.3 (10.7–11.6) |
| native | read | snapshot.serialize | rc4-local | 16.3 (16.1–17.5) | 13.3 (13.3–14.3) | 14.4 (14.2–15.2) | 14.9 (14.8–16.1) |
| native | read | snapshot.serialize | main-local | 18.7 (16.9–19.5) | 14.4 (13.6–14.6) | 15.3 (14.6–15.6) | 15.9 (15.1–16.3) |
| native | read | snapshot.serialize_pretty | rc4-local | 33.2 (32.5–35.1) | 28.3 (28.3–30.4) | 30.3 (30.1–32.2) | 31.2 (31.1–33.9) |
| native | read | snapshot.serialize_pretty | main-local | 35.0 (33.4–35.7) | 30.5 (28.6–30.7) | 32.3 (30.6–32.9) | 33.8 (31.7–34.4) |
| native | read | view | rc4-local | 2.2 (2.0–2.3) | 1.9 (1.9–2.0) | 2.3 (2.3–2.4) | 2.5 (2.5–2.7) |
| native | read | view | main-local | 2.2 (2.1–2.3) | 2.0 (1.8–2.0) | 2.4 (2.2–2.4) | 2.6 (2.4–2.6) |
| native | read | view.serialize | rc4-local | 6.0 (5.9–6.4) | 4.3 (4.2–4.6) | 4.4 (4.4–4.7) | 4.6 (4.6–5.0) |
| native | read | view.serialize | main-local | 6.4 (6.0–6.6) | 4.6 (4.3–4.7) | 4.7 (4.5–4.8) | 5.0 (4.7–5.1) |
| native | web.read | web.save | rc4-local | 11.8 (11.6–12.2) | 12.4 (11.6–12.5) | 16.0 (14.9–16.1) | 16.6 (15.5–16.7) |
| native | web.read | web.save | main-local | 11.7 (11.2–12.9) | 12.5 (11.7–12.9) | 16.1 (15.1–16.5) | 16.8 (15.6–17.4) |
| native | web.read | web.snapshot | rc4-local | 78.1 (77.7–79.8) | 62.5 (58.9–63.0) | 66.9 (62.7–67.3) | 70.7 (66.0–70.7) |
| native | web.read | web.snapshot | main-local | 81.9 (79.0–82.1) | 63.3 (59.0–64.6) | 67.1 (63.0–69.5) | 71.3 (65.5–72.8) |
| native | web.read | web.view | rc4-local | 6.7 (6.4–6.8) | 5.7 (5.3–5.8) | 6.3 (5.9–6.4) | 6.8 (6.4–6.8) |
| native | web.read | web.view | main-local | 6.8 (6.4–7.1) | 5.8 (5.4–5.9) | 6.3 (5.9–6.5) | 6.9 (6.3–7.0) |
| wasm | published-method | adapter.dispatch+view | rc4-published | 178.9 (147.2–218.5) | 52.5 (52.3–52.8) | 47.2 (44.6–48.4) | 46.6 (46.1–46.8) |
| wasm | published-method | adapter.dispatch+view | rc4-local | 174.6 (169.4–186.7) | 51.0 (48.8–54.3) | 46.0 (44.0–47.4) | 46.4 (45.8–46.5) |
| wasm | published-method | adapter.dispatch+view | main-local | 167.5 (161.9–171.0) | 51.6 (51.0–54.3) | 47.5 (42.8–47.6) | 46.6 (43.6–47.1) |
| wasm | published-method | adapter.dispatch+view | hist-e6ace96 | 253.0 (159.6–417.4) | 49.4 (47.7–50.9) | 44.8 (42.3–45.4) | 44.7 (41.1–45.0) |
| wasm | adapter | adapter.dispatch | rc4-published | 167.0 (155.8–179.9) | 46.2 (42.6–46.3) | 43.0 (40.4–43.9) | 43.9 (43.5–44.5) |
| wasm | adapter | adapter.dispatch | rc4-local | 159.6 (158.2–165.3) | 45.7 (45.4–47.8) | 42.4 (42.4–43.8) | 43.3 (41.0–43.8) |
| wasm | adapter | adapter.dispatch | main-local | 173.1 (162.6–177.1) | 46.6 (46.0–47.3) | 42.2 (39.8–43.4) | 43.6 (40.9–43.7) |
| wasm | adapter | adapter.dispatch | hist-e6ace96 | 143.8 (139.6–165.5) | 44.0 (42.6–44.3) | 41.4 (39.0–42.2) | 42.2 (39.2–42.5) |
| wasm | adapter | adapter.view | rc4-published | 23.4 (21.6–34.8) | 2.7 (2.6–2.8) | 2.3 (2.2–2.3) | 2.2 (2.2–2.3) |
| wasm | adapter | adapter.view | rc4-local | 37.1 (33.0–38.1) | 2.7 (2.7–2.7) | 2.3 (2.3–2.4) | 2.2 (2.1–2.3) |
| wasm | adapter | adapter.view | main-local | 31.0 (24.5–40.4) | 2.7 (2.6–2.8) | 2.3 (2.2–2.3) | 2.2 (2.2–2.3) |
| wasm | adapter | adapter.view | hist-e6ace96 | 32.9 (29.7–41.8) | 2.6 (2.5–2.6) | 2.3 (2.2–2.3) | 2.2 (2.1–2.2) |
| wasm | adapter | adapter.dispatch+view | rc4-published | 186.5 (174.6–210.0) | 49.1 (45.6–49.3) | 45.3 (42.8–46.4) | 46.2 (45.8–46.9) |
| wasm | adapter | adapter.dispatch+view | rc4-local | 190.0 (170.1–212.5) | 48.7 (48.2–50.8) | 44.9 (44.8–46.3) | 45.5 (43.2–46.2) |
| wasm | adapter | adapter.dispatch+view | main-local | 202.1 (200.9–205.5) | 49.6 (48.9–50.1) | 44.6 (42.2–45.8) | 46.0 (43.1–46.0) |
| wasm | adapter | adapter.dispatch+view | hist-e6ace96 | 181.5 (176.0–181.6) | 46.5 (45.3–47.1) | 43.7 (41.4–44.6) | 44.4 (41.4–44.7) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-published | 1.0 (0.8–1.1) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-local | 0.9 (0.9–1.0) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local | 0.8 (0.7–0.9) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | hist-e6ace96 | 0.8 (0.8–1.1) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-published | 134.7 (133.0–140.3) | 28.8 (28.7–29.5) | 26.8 (26.3–26.9) | 26.1 (26.1–26.2) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-local | 132.9 (124.7–136.9) | 28.9 (27.2–29.0) | 26.1 (24.1–26.6) | 26.0 (25.1–26.1) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local | 124.0 (119.7–127.6) | 29.4 (27.4–29.6) | 26.6 (25.6–27.1) | 26.3 (24.4–26.7) |
| wasm | raw.dispatch_view | raw.dispatch_view | hist-e6ace96 | 131.0 (127.7–143.8) | 27.4 (27.2–28.1) | 25.2 (23.4–25.5) | 24.9 (23.3–24.9) |
| wasm | raw.dispatch_view | js.parse_view | rc4-published | 30.3 (28.5–32.9) | 16.3 (16.2–16.7) | 16.7 (16.5–16.8) | 17.1 (16.9–17.2) |
| wasm | raw.dispatch_view | js.parse_view | rc4-local | 29.4 (29.2–29.7) | 16.3 (15.9–16.6) | 16.6 (15.5–16.8) | 17.3 (16.7–17.3) |
| wasm | raw.dispatch_view | js.parse_view | main-local | 28.5 (26.3–29.0) | 16.7 (15.3–16.8) | 16.7 (15.9–17.2) | 17.1 (16.1–18.0) |
| wasm | raw.dispatch_view | js.parse_view | hist-e6ace96 | 28.1 (26.7–28.1) | 16.0 (15.9–16.5) | 16.6 (15.5–16.6) | 17.0 (16.0–17.2) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-published | 167.9 (161.9–180.8) | 45.5 (45.3–46.5) | 43.9 (43.2–44.1) | 43.7 (43.5–43.8) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-local | 163.9 (157.0–166.3) | 45.7 (43.4–46.0) | 43.1 (40.0–43.7) | 43.7 (42.3–43.9) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local | 157.6 (153.7–175.0) | 46.6 (43.0–46.6) | 43.7 (41.9–44.8) | 43.8 (40.8–45.2) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | hist-e6ace96 | 173.5 (161.4–176.2) | 43.9 (43.6–44.9) | 42.2 (39.2–42.4) | 42.3 (39.6–42.5) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | rc4-published | 195.7 (192.4–200.3) | 70.9 (68.7–72.6) | 70.2 (70.0–72.1) | 74.4 (72.2–75.4) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | rc4-local | 195.2 (184.8–198.8) | 72.4 (71.3–72.4) | 71.0 (69.9–71.1) | 74.1 (73.5–74.2) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | main-local | 185.5 (179.7–192.9) | 72.6 (71.9–77.5) | 72.2 (70.6–77.4) | 73.9 (72.7–81.7) |
| wasm | raw.dispatch_outcome | js.parse_outcome | rc4-published | 83.9 (79.7–85.1) | 59.5 (57.9–61.0) | 62.1 (61.4–64.1) | 66.5 (64.3–67.3) |
| wasm | raw.dispatch_outcome | js.parse_outcome | rc4-local | 80.3 (76.3–83.1) | 59.9 (58.5–60.4) | 62.3 (61.6–63.3) | 65.0 (64.7–65.5) |
| wasm | raw.dispatch_outcome | js.parse_outcome | main-local | 77.9 (77.1–82.4) | 60.0 (59.8–64.4) | 63.0 (62.4–68.4) | 65.0 (64.9–72.1) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | rc4-published | 279.6 (268.8–282.2) | 131.1 (126.9–134.2) | 132.7 (131.9–136.9) | 140.9 (137.1–143.1) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | rc4-local | 271.0 (267.3–287.1) | 132.8 (129.8–133.1) | 133.9 (132.1–134.8) | 139.3 (139.2–139.4) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | main-local | 269.7 (269.2–295.7) | 133.0 (132.6–143.0) | 135.8 (133.8–146.3) | 139.4 (138.2–154.1) |
| wasm | raw.dispatch | raw.dispatch | rc4-published | 202.5 (190.0–209.1) | 95.8 (93.3–101.3) | 99.0 (96.3–103.8) | 103.9 (102.1–105.7) |
| wasm | raw.dispatch | raw.dispatch | rc4-local | 213.2 (191.1–224.1) | 94.2 (91.6–97.5) | 94.9 (92.6–97.6) | 101.5 (98.7–101.7) |
| wasm | raw.dispatch | raw.dispatch | main-local | 205.5 (202.6–233.6) | 95.8 (93.8–96.7) | 97.3 (92.8–98.1) | 100.6 (97.6–103.1) |
| wasm | raw.dispatch | raw.dispatch | hist-e6ace96 | 212.0 (191.2–217.8) | 94.7 (90.7–97.6) | 96.0 (91.5–100.5) | 100.2 (95.4–104.3) |
| wasm | abi.dispatch_view | abi.encode_args | rc4-published | 0.8 (0.7–0.8) | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | rc4-local | 0.9 (0.9–0.9) | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | main-local | 0.8 (0.8–0.9) | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | hist-e6ace96 | 0.8 (0.8–0.9) | 0.2 (0.2–0.2) | 0.1 (0.1–0.2) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.exec | rc4-published | 129.2 (112.8–139.7) | 25.5 (24.6–25.7) | 23.1 (22.9–23.1) | 22.7 (22.5–23.1) |
| wasm | abi.dispatch_view | abi.exec | rc4-local | 118.9 (112.1–121.3) | 25.6 (25.4–27.6) | 23.3 (22.9–24.2) | 22.4 (21.6–22.6) |
| wasm | abi.dispatch_view | abi.exec | main-local | 114.2 (109.0–140.5) | 25.2 (23.9–25.6) | 23.4 (23.2–23.4) | 22.5 (21.1–22.5) |
| wasm | abi.dispatch_view | abi.exec | hist-e6ace96 | 133.8 (118.8–140.4) | 24.9 (24.7–25.8) | 22.4 (21.4–22.8) | 21.2 (20.4–21.6) |
| wasm | abi.dispatch_view | abi.decode | rc4-published | 4.9 (3.6–6.3) | 2.5 (2.5–2.6) | 2.5 (2.5–2.5) | 2.6 (2.5–2.6) |
| wasm | abi.dispatch_view | abi.decode | rc4-local | 3.7 (3.5–4.0) | 2.5 (2.5–2.7) | 2.5 (2.5–2.6) | 2.5 (2.5–2.5) |
| wasm | abi.dispatch_view | abi.decode | main-local | 4.0 (3.4–5.4) | 2.5 (2.4–2.5) | 2.5 (2.5–2.5) | 2.5 (2.4–2.5) |
| wasm | abi.dispatch_view | abi.decode | hist-e6ace96 | 3.8 (3.8–5.3) | 2.5 (2.5–2.5) | 2.5 (2.4–2.6) | 2.5 (2.4–2.5) |
| wasm | abi.dispatch_view | abi.free | rc4-published | 0.2 (0.2–0.5) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | rc4-local | 0.2 (0.1–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | main-local | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | hist-e6ace96 | 0.3 (0.3–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.dispatch_view | rc4-published | 133.4 (119.4–154.4) | 28.4 (27.4–28.7) | 25.8 (25.7–25.8) | 25.5 (25.3–26.0) |
| wasm | abi.dispatch_view | abi.dispatch_view | rc4-local | 122.1 (116.7–125.4) | 28.4 (28.2–30.8) | 26.0 (25.7–27.0) | 25.2 (24.3–25.4) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local | 119.2 (112.9–155.4) | 28.2 (26.6–28.5) | 26.2 (26.1–26.2) | 25.2 (23.7–25.3) |
| wasm | abi.dispatch_view | abi.dispatch_view | hist-e6ace96 | 144.4 (123.3–152.1) | 27.8 (27.5–28.6) | 25.1 (24.1–25.7) | 23.9 (23.0–24.4) |
| wasm | abi.dispatch_outcome | abi.encode_args | rc4-published | 0.9 (0.8–1.0) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) | 0.2 (0.1–0.2) |
| wasm | abi.dispatch_outcome | abi.encode_args | rc4-local | 0.9 (0.8–0.9) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) | 0.2 (0.1–0.2) |
| wasm | abi.dispatch_outcome | abi.encode_args | main-local | 0.9 (0.9–1.0) | 0.3 (0.3–0.3) | 0.2 (0.2–0.2) | 0.2 (0.1–0.2) |
| wasm | abi.dispatch_outcome | abi.exec | rc4-published | 169.8 (166.3–184.2) | 60.5 (59.3–64.2) | 59.5 (56.0–59.9) | 61.4 (58.3–61.7) |
| wasm | abi.dispatch_outcome | abi.exec | rc4-local | 164.3 (160.4–171.9) | 60.3 (59.8–63.2) | 59.2 (59.1–61.4) | 60.8 (58.5–63.2) |
| wasm | abi.dispatch_outcome | abi.exec | main-local | 166.1 (159.4–182.9) | 59.9 (57.1–61.1) | 60.2 (59.0–60.2) | 61.1 (60.9–61.5) |
| wasm | abi.dispatch_outcome | abi.decode | rc4-published | 7.8 (7.3–8.1) | 7.0 (6.9–7.2) | 7.5 (7.1–7.5) | 7.7 (7.4–7.9) |
| wasm | abi.dispatch_outcome | abi.decode | rc4-local | 8.2 (7.6–9.6) | 6.9 (6.9–7.3) | 7.5 (7.2–7.7) | 7.8 (7.5–8.1) |
| wasm | abi.dispatch_outcome | abi.decode | main-local | 8.8 (8.5–10.2) | 7.0 (6.7–7.0) | 7.5 (7.4–7.6) | 7.8 (7.7–7.8) |
| wasm | abi.dispatch_outcome | abi.free | rc4-published | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.free | rc4-local | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.free | main-local | 0.2 (0.2–0.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | rc4-published | 179.1 (174.3–192.7) | 68.1 (66.8–72.1) | 67.6 (63.7–67.9) | 69.9 (66.3–70.0) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | rc4-local | 172.0 (169.1–181.2) | 67.7 (67.3–71.2) | 67.1 (66.8–69.7) | 69.1 (66.5–71.9) |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | main-local | 176.8 (168.3–195.1) | 67.4 (64.4–68.8) | 68.3 (66.8–68.4) | 69.4 (69.2–69.9) |
| wasm | read | raw.view | rc4-published | 10.6 (10.0–11.8) | 9.9 (9.7–10.5) | 10.5 (10.2–11.2) | 11.5 (11.0–12.0) |
| wasm | read | raw.view | rc4-local | 11.0 (10.1–12.0) | 9.9 (9.6–10.1) | 10.7 (10.4–11.0) | 11.6 (11.1–12.4) |
| wasm | read | raw.view | main-local | 11.5 (11.4–11.9) | 10.4 (10.2–10.6) | 11.2 (10.8–11.2) | 11.4 (11.1–11.5) |
| wasm | read | raw.view | hist-e6ace96 | 9.9 (9.8–10.5) | 9.2 (8.9–9.4) | 9.8 (9.6–10.1) | 10.4 (10.1–10.8) |
| wasm | read | js.parse_view | rc4-published | 30.5 (25.0–34.0) | 17.9 (17.2–18.4) | 18.1 (17.2–18.4) | 19.0 (18.2–19.3) |
| wasm | read | js.parse_view | rc4-local | 28.1 (27.1–29.9) | 17.5 (17.1–18.1) | 18.0 (17.2–18.5) | 18.8 (18.1–20.0) |
| wasm | read | js.parse_view | main-local | 27.9 (27.8–28.8) | 18.4 (17.8–18.6) | 18.8 (17.9–18.9) | 18.1 (18.0–18.7) |
| wasm | read | js.parse_view | hist-e6ace96 | 27.9 (26.5–31.7) | 17.3 (16.7–17.7) | 17.2 (17.2–17.8) | 18.1 (17.5–18.4) |
| wasm | read | raw.snapshot | rc4-published | 99.1 (98.0–103.8) | 83.0 (80.9–88.5) | 86.3 (83.5–91.6) | 94.1 (90.9–98.1) |
| wasm | read | raw.snapshot | rc4-local | 99.3 (96.5–100.6) | 82.8 (79.8–85.3) | 87.9 (84.0–89.0) | 95.5 (90.5–101.5) |
| wasm | read | raw.snapshot | main-local | 99.9 (98.4–102.2) | 87.2 (85.8–88.9) | 90.9 (89.3–91.6) | 90.5 (90.0–95.4) |
| wasm | read | raw.snapshot | hist-e6ace96 | 99.5 (99.2–100.3) | 84.0 (80.1–85.2) | 89.2 (86.4–89.9) | 94.6 (92.7–95.5) |
| wasm | read | js.parse_snapshot | rc4-published | 77.7 (72.7–78.3) | 63.8 (61.8–66.0) | 67.8 (65.7–69.6) | 72.4 (69.6–73.1) |
| wasm | read | js.parse_snapshot | rc4-local | 76.9 (72.0–77.2) | 63.3 (61.0–65.1) | 68.8 (65.8–69.2) | 72.4 (68.7–76.8) |
| wasm | read | js.parse_snapshot | main-local | 77.8 (73.6–78.5) | 66.6 (64.8–67.7) | 69.5 (69.0–71.1) | 69.0 (68.7–72.0) |
| wasm | read | js.parse_snapshot | hist-e6ace96 | 75.6 (74.6–76.0) | 63.8 (61.4–65.4) | 67.4 (66.3–69.3) | 70.3 (68.1–71.9) |
| wasm | read | raw.save | rc4-published | 16.3 (14.9–17.3) | 14.1 (13.8–15.1) | 17.4 (16.8–18.0) | 19.4 (18.8–20.2) |
| wasm | read | raw.save | rc4-local | 15.5 (15.4–15.9) | 14.1 (13.6–14.7) | 17.4 (16.8–17.7) | 19.6 (18.9–20.7) |
| wasm | read | raw.save | main-local | 16.5 (16.0–17.2) | 14.9 (14.5–15.0) | 17.6 (17.1–18.4) | 18.9 (18.9–19.4) |
| wasm | read | raw.save | hist-e6ace96 | 16.0 (15.7–16.9) | 14.5 (13.8–14.5) | 17.8 (17.4–18.4) | 18.9 (18.4–19.1) |
| wasm | abi.read | abi.view.exec | rc4-published | 8.8 (8.7–9.2) | 7.3 (7.1–7.6) | 7.8 (7.7–8.1) | 8.3 (8.2–8.6) |
| wasm | abi.read | abi.view.exec | rc4-local | 9.0 (8.6–9.0) | 7.6 (7.5–8.0) | 8.3 (7.7–8.5) | 8.8 (8.3–8.8) |
| wasm | abi.read | abi.view.exec | main-local | 9.0 (8.7–9.6) | 7.5 (7.3–7.6) | 8.1 (7.9–8.5) | 8.5 (8.2–8.7) |
| wasm | abi.read | abi.view.exec | hist-e6ace96 | 8.7 (8.0–9.6) | 6.8 (6.5–7.0) | 7.3 (7.0–7.5) | 7.5 (7.4–7.9) |
| wasm | abi.read | abi.view.decode | rc4-published | 1.8 (1.8–2.0) | 2.2 (2.1–2.3) | 2.2 (2.2–2.3) | 2.3 (2.3–2.4) |
| wasm | abi.read | abi.view.decode | rc4-local | 2.0 (1.8–2.1) | 2.3 (2.3–2.4) | 2.3 (2.2–2.5) | 2.3 (2.2–2.4) |
| wasm | abi.read | abi.view.decode | main-local | 1.9 (1.8–2.0) | 2.2 (2.2–2.3) | 2.3 (2.2–2.4) | 2.3 (2.3–2.4) |
| wasm | abi.read | abi.view.decode | hist-e6ace96 | 2.0 (1.8–2.0) | 2.2 (2.1–2.3) | 2.2 (2.2–2.3) | 2.2 (2.2–2.3) |
| wasm | abi.read | abi.snapshot.exec | rc4-published | 83.3 (83.0–84.7) | 63.4 (62.5–67.2) | 65.9 (65.8–70.0) | 70.5 (69.2–73.6) |
| wasm | abi.read | abi.snapshot.exec | rc4-local | 84.1 (82.1–90.1) | 67.3 (64.7–70.0) | 70.9 (64.8–72.7) | 71.9 (68.6–74.2) |
| wasm | abi.read | abi.snapshot.exec | main-local | 84.8 (80.1–92.7) | 65.2 (64.2–66.9) | 69.7 (67.4–72.8) | 69.5 (68.7–73.4) |
| wasm | abi.read | abi.snapshot.exec | hist-e6ace96 | 85.5 (83.6–93.6) | 64.7 (62.1–66.7) | 69.7 (67.3–71.0) | 72.4 (71.6–75.9) |
| wasm | abi.read | abi.snapshot.decode | rc4-published | 9.8 (8.9–10.5) | 10.7 (10.3–11.1) | 11.3 (11.3–11.9) | 12.1 (12.0–12.6) |
| wasm | abi.read | abi.snapshot.decode | rc4-local | 9.6 (9.5–9.8) | 11.5 (11.2–11.6) | 12.2 (11.1–12.9) | 12.6 (12.0–12.7) |
| wasm | abi.read | abi.snapshot.decode | main-local | 9.5 (9.0–9.7) | 11.3 (10.9–11.4) | 12.0 (11.3–12.4) | 12.0 (12.0–12.7) |
| wasm | abi.read | abi.snapshot.decode | hist-e6ace96 | 9.5 (8.6–9.5) | 11.0 (10.4–11.2) | 11.6 (11.4–11.6) | 12.0 (11.9–12.5) |
| wasm | abi.read | abi.save.exec | rc4-published | 14.5 (14.0–14.8) | 11.6 (11.3–12.4) | 13.9 (13.9–14.9) | 15.0 (14.9–15.9) |
| wasm | abi.read | abi.save.exec | rc4-local | 14.7 (14.4–14.9) | 12.4 (11.8–12.7) | 14.9 (13.7–15.4) | 15.8 (14.8–16.2) |
| wasm | abi.read | abi.save.exec | main-local | 14.3 (13.0–14.6) | 11.8 (11.6–12.4) | 14.9 (14.4–15.4) | 15.2 (15.1–15.9) |
| wasm | abi.read | abi.save.exec | hist-e6ace96 | 14.0 (13.9–14.5) | 11.4 (11.1–11.9) | 14.6 (14.6–15.3) | 15.2 (15.2–15.9) |
| wasm | abi.read | abi.save.decode | rc4-published | 0.5 (0.4–0.5) | 0.4 (0.4–0.5) | 0.5 (0.5–0.6) | 0.6 (0.6–0.6) |
| wasm | abi.read | abi.save.decode | rc4-local | 0.5 (0.4–0.5) | 0.5 (0.5–0.5) | 0.6 (0.5–0.6) | 0.6 (0.6–0.6) |
| wasm | abi.read | abi.save.decode | main-local | 0.5 (0.4–0.5) | 0.4 (0.4–0.5) | 0.6 (0.5–0.6) | 0.6 (0.6–0.6) |
| wasm | abi.read | abi.save.decode | hist-e6ace96 | 0.4 (0.4–0.5) | 0.4 (0.4–0.5) | 0.6 (0.5–0.6) | 0.6 (0.6–0.7) |
| wasm | kit | kit.dispatch | rc4-published | 281.4 (261.1–281.9) | 133.5 (126.4–134.7) | 135.4 (130.2–136.7) | 135.4 (134.7–142.1) |
| wasm | kit | kit.dispatch | rc4-local | 269.4 (267.2–289.7) | 133.8 (128.9–134.5) | 136.1 (131.1–139.5) | 141.3 (135.9–142.3) |
| wasm | kit | kit.dispatch | main-local | 296.8 (289.0–297.5) | 132.1 (131.3–138.7) | 136.4 (129.0–140.0) | 137.6 (136.0–145.6) |
| wasm | kit | kit.view | rc4-published | 37.1 (34.3–40.7) | 27.3 (26.1–27.7) | 28.2 (27.1–28.3) | 28.2 (28.0–29.5) |
| wasm | kit | kit.view | rc4-local | 35.9 (33.2–38.5) | 27.6 (26.5–27.8) | 28.5 (27.5–29.3) | 29.6 (28.3–29.7) |
| wasm | kit | kit.view | main-local | 42.6 (35.0–42.7) | 27.3 (27.0–28.3) | 28.7 (26.9–29.0) | 28.7 (28.4–30.4) |
| wasm | kit | kit.dispatch+view | rc4-published | 322.1 (300.0–335.1) | 160.9 (152.9–162.6) | 164.0 (157.5–165.3) | 163.7 (163.0–171.7) |
| wasm | kit | kit.dispatch+view | rc4-local | 310.8 (299.9–347.3) | 161.8 (155.3–162.4) | 164.8 (158.8–169.2) | 171.1 (164.4–172.2) |
| wasm | kit | kit.dispatch+view | main-local | 354.8 (346.0–360.9) | 159.4 (158.6–167.4) | 165.4 (156.2–169.3) | 166.4 (164.5–176.1) |
| wasm | kit.read | kit.view | rc4-published | 42.2 (40.8–42.4) | 28.6 (28.2–30.1) | 29.7 (29.5–30.8) | 31.0 (30.6–32.8) |
| wasm | kit.read | kit.view | rc4-local | 41.5 (34.5–47.2) | 30.2 (28.0–30.6) | 29.5 (29.2–30.5) | 31.5 (30.4–32.6) |
| wasm | kit.read | kit.view | main-local | 41.3 (41.1–44.6) | 30.1 (28.5–31.7) | 30.8 (29.1–31.0) | 31.5 (30.9–32.6) |
| wasm | kit.read | kit.snapshot | rc4-published | 171.0 (161.2–171.1) | 154.1 (152.3–160.9) | 162.0 (160.2–165.1) | 171.7 (169.9–177.6) |
| wasm | kit.read | kit.snapshot | rc4-local | 170.3 (158.9–182.1) | 160.9 (150.6–165.0) | 159.8 (157.4–162.7) | 174.0 (167.0–177.2) |
| wasm | kit.read | kit.snapshot | main-local | 162.0 (154.0–173.2) | 162.1 (151.6–168.4) | 165.3 (156.3–166.5) | 169.8 (169.1–177.8) |
| wasm | kit.read | kit.save | rc4-published | 40.4 (38.4–42.4) | 31.7 (30.9–32.9) | 36.6 (36.6–39.5) | 41.1 (40.5–43.0) |
| wasm | kit.read | kit.save | rc4-local | 41.4 (38.0–41.5) | 32.9 (30.6–33.5) | 37.2 (36.9–38.0) | 41.6 (40.1–42.3) |
| wasm | kit.read | kit.save | main-local | 39.5 (39.1–40.1) | 33.4 (31.1–34.5) | 37.8 (36.8–39.4) | 41.0 (40.7–43.1) |

## glowcap-resume

The published resume measurement's stream (experiments/glowcap/resume-bench.mjs): absorb cave glowcap, then 9,600 ticks of dt 0.0625 (ten minutes of play), then save and restore. Program `experiments/glowcap/caveat5/glowcap.cav` (`6b5acb2b87d2`), stream `0407c19120d4`, 1 episode(s) × 1 = 9601 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | lifecycle | from_source | - | 2547.7 (2490.6–2682.9) · p95 2702.7 | 2567.8 (2537.2–2778.1) · p95 2663.6 | - |
| native | lifecycle | restore | - | 2647.8 (2628.1–2800.3) · p95 2790.6 | 2655.0 (2630.8–2792.4) · p95 2831.0 | - |
| native | lifecycle | restore.parse | - | 18.4 (17.3–19.0) · p95 23.0 | 19.4 (18.8–19.8) · p95 25.3 | - |
| native | lifecycle | restore_json | - | 2674.6 (2623.0–2794.0) · p95 2850.4 | 2666.8 (2641.4–2783.4) · p95 2821.0 | - |
| native | lifecycle | save | - | 12.7 (12.4–13.6) · p95 14.9 | 13.3 (13.1–13.4) · p95 17.7 | - |
| native | lifecycle | save_json | - | 15.3 (14.9–16.3) · p95 17.3 | 23.6 (16.3–25.0) · p95 30.6 | - |
| native | lifecycle | web.new | - | 2591.4 (2527.7–2677.8) · p95 2692.8 | 2565.2 (2546.1–2794.2) · p95 2663.2 | - |
| native | lifecycle | web.restore | - | 2673.3 (2642.5–2811.5) · p95 2812.6 | 2672.5 (2647.0–2807.8) · p95 2856.8 | - |
| native | lifecycle | web.save | - | 10.7 (10.4–11.5) · p95 12.1 | 11.3 (11.2–11.9) · p95 13.5 | - |
| wasm | lifecycle | raw.new | 2636.0 (2429.0–2842.5) · p95 2991.6 | 2672.0 (2609.8–2905.0) · p95 3003.5 | 2595.1 (2498.7–2687.5) · p95 2832.4 | 2705.8 (2494.8–2733.3) · p95 3026.2 |
| wasm | lifecycle | raw.save | 34.1 (33.6–37.1) · p95 45.5 | 33.4 (33.2–36.3) · p95 44.9 | 34.1 (32.4–34.3) · p95 46.2 | 34.8 (32.8–35.9) · p95 44.3 |
| wasm | lifecycle | raw.restore | 2600.1 (2451.7–2848.7) · p95 2823.4 | 2656.9 (2580.6–2857.3) · p95 3103.2 | 2581.2 (2491.0–2669.9) · p95 2786.1 | 2676.5 (2459.7–2733.0) · p95 2954.2 |
| wasm | lifecycle | kit.open | 2541.4 (2391.5–2784.7) · p95 2735.0 | 2577.3 (2545.9–2784.3) · p95 2984.8 | 2536.6 (2437.2–2628.5) · p95 2756.4 | - |
| wasm | lifecycle | kit.save | 73.0 (70.7–77.1) · p95 101.0 | 78.9 (71.6–79.4) · p95 102.2 | 72.8 (72.8–73.4) · p95 110.8 | - |
| wasm | lifecycle | kit.restore | 2558.2 (2383.5–2817.0) · p95 2801.4 | 2642.1 (2578.4–2835.2) · p95 3011.4 | 2572.9 (2487.8–2685.7) · p95 2778.0 | - |
| wasm | adapter-resume | adapter.resume.parse | 14.1 (13.7–14.4) · p95 14.1 | 15.0 (14.5–15.5) · p95 15.0 | 13.9 (13.4–14.6) · p95 13.9 | 14.9 (14.9–15.5) · p95 14.9 |
| wasm | adapter-resume | adapter.resume.createPolicy | 3361.8 (3146.8–3632.5) · p95 3361.8 | 3352.3 (3342.7–3475.2) · p95 3352.3 | 3680.9 (3433.2–3716.7) · p95 3680.9 | 3253.5 (3195.1–3317.4) · p95 3253.5 |
| wasm | adapter-resume | adapter.resume.view | 44.3 (42.5–47.9) · p95 44.3 | 42.8 (42.1–43.4) · p95 42.8 | 43.1 (40.1–55.3) · p95 43.1 | 49.9 (45.4–54.9) · p95 49.9 |
| wasm | adapter-resume | adapter.resume | 3410.2 (3203.0–3680.5) · p95 3410.2 | 3401.0 (3397.7–3522.1) · p95 3401.0 | 3730.5 (3480.4–3764.3) · p95 3730.5 | 3308.3 (3257.3–3371.6) · p95 3308.3 |

## glowcap-unbound

The Glowcap replay program with its 39 one-line bind statements removed, on the replay's full stream: the same rules, states and transaction with no bindings to evaluate, so apply here against apply on glowcap-replay isolates binding evaluation. Program `experiments/glowcap/caveat5/glowcap.cav` (`ffa078c4cb75`), stream `4089c5d81e6f`, 1 episode(s) × 1 = 10000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 12.0 (11.8–12.6) · p95 13.3 | 12.8 (11.8–13.0) · p95 14.2 | - |
| native | dispatch_view_json | dispatch_view_json | - | 14.6 (14.5–15.7) · p95 16.3 | 14.5 (14.5–15.5) · p95 15.9 | - |
| native | web.dispatch_view | web.dispatch_view | - | 17.1 (16.7–18.3) · p95 19.0 | 16.9 (16.8–18.0) · p95 18.3 | - |
| native | read | clone | - | 0.3 (0.3–0.4) · p95 0.4 | 0.3 (0.3–0.3) · p95 0.4 | - |
| native | read | clone.drop | - | 0.3 (0.3–0.3) · p95 0.3 | 0.3 (0.3–0.3) · p95 0.4 | - |
| native | read | save | - | 9.2 (8.7–9.6) · p95 9.9 | 9.0 (8.9–9.4) · p95 10.2 | - |
| native | read | save_json | - | 14.8 (13.9–15.5) · p95 15.8 | 14.4 (14.3–15.0) · p95 16.1 | - |
| native | read | snapshot | - | 18.7 (17.6–19.7) · p95 20.1 | 18.0 (17.8–18.8) · p95 20.1 | - |
| native | read | snapshot.drop | - | 7.4 (7.0–7.9) · p95 7.9 | 7.1 (7.0–7.3) · p95 7.8 | - |
| native | read | snapshot.serialize | - | 12.1 (11.5–12.7) · p95 13.0 | 11.8 (11.6–12.2) · p95 13.2 | - |
| native | read | snapshot.serialize_pretty | - | 24.4 (23.2–25.8) · p95 26.1 | 23.8 (23.5–24.7) · p95 26.3 | - |
| native | read | view | - | 2.6 (2.4–2.7) · p95 2.8 | 2.4 (2.4–2.6) · p95 2.8 | - |
| native | read | view.serialize | - | 2.4 (2.3–2.5) · p95 2.7 | 2.4 (2.4–2.5) · p95 2.8 | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.2) · p95 0.3 |
| wasm | abi.dispatch_view | abi.exec | 17.8 (17.2–18.8) · p95 21.7 | 17.8 (17.2–18.5) · p95 22.2 | 18.1 (17.4–18.7) · p95 21.5 | 17.5 (16.4–19.2) · p95 21.3 |
| wasm | abi.dispatch_view | abi.decode | 0.5 (0.5–0.5) · p95 0.9 | 0.5 (0.5–0.5) · p95 0.9 | 0.5 (0.5–0.5) · p95 0.9 | 0.5 (0.4–0.5) · p95 0.9 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | abi.dispatch_view | abi.dispatch_view | 18.7 (18.0–19.6) · p95 22.7 | 18.6 (18.0–19.3) · p95 23.2 | 18.8 (18.1–19.6) · p95 22.6 | 18.3 (17.2–20.1) · p95 22.3 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.2 (0.2–0.3) · p95 0.4 | 0.3 (0.2–0.3) · p95 0.4 | 0.2 (0.2–0.3) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | raw.dispatch_view | raw.dispatch_view | 19.5 (18.7–22.6) · p95 24.2 | 19.8 (18.3–21.5) · p95 23.0 | 18.6 (18.4–19.7) · p95 22.4 | 18.9 (17.4–19.0) · p95 22.9 |
| wasm | raw.dispatch_view | js.parse_view | 6.3 (5.9–7.2) · p95 7.5 | 6.3 (5.9–6.9) · p95 7.5 | 5.9 (5.9–6.3) · p95 7.3 | 6.2 (5.8–6.2) · p95 7.3 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 26.1 (24.9–30.1) · p95 31.9 | 26.5 (24.4–28.8) · p95 30.7 | 24.8 (24.6–26.3) · p95 30.0 | 25.4 (23.5–25.4) · p95 30.3 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | rc4-local | 31.8 (30.6–34.0) | 13.2 (13.2–13.8) | 12.4 (12.3–12.9) | 12.0 (11.8–12.5) |
| native | apply | apply | main-local | 32.8 (31.2–33.8) | 14.2 (13.0–14.7) | 13.2 (12.2–14.0) | 12.8 (11.8–13.0) |
| native | dispatch_view_json | dispatch_view_json | rc4-local | 37.0 (35.1–41.4) | 15.6 (15.2–16.7) | 14.9 (14.6–15.9) | 14.5 (14.5–15.7) |
| native | dispatch_view_json | dispatch_view_json | main-local | 36.4 (34.9–36.9) | 15.4 (15.2–16.3) | 14.8 (14.6–15.7) | 14.4 (14.4–15.4) |
| native | web.dispatch_view | web.dispatch_view | rc4-local | 42.0 (41.3–45.6) | 17.8 (17.4–18.9) | 17.2 (16.9–18.5) | 17.0 (16.7–18.3) |
| native | web.dispatch_view | web.dispatch_view | main-local | 43.7 (40.1–46.2) | 17.6 (17.5–18.8) | 16.9 (16.8–18.2) | 16.8 (16.7–18.0) |
| native | read | clone | rc4-local | 0.6 (0.5–0.6) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) | 0.3 (0.3–0.4) |
| native | read | clone | main-local | 0.6 (0.6–0.6) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| native | read | clone.drop | rc4-local | 0.4 (0.4–0.5) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| native | read | clone.drop | main-local | 0.4 (0.4–0.5) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| native | read | save | rc4-local | 5.5 (5.3–5.7) | 6.3 (5.9–6.8) | 8.8 (8.1–9.2) | 9.2 (8.7–9.7) |
| native | read | save | main-local | 5.4 (5.3–5.6) | 6.4 (6.1–6.7) | 8.5 (8.4–8.9) | 9.0 (8.9–9.5) |
| native | read | save_json | rc4-local | 9.4 (8.7–9.6) | 10.8 (10.1–11.5) | 14.6 (13.5–15.4) | 14.8 (13.9–15.5) |
| native | read | save_json | main-local | 9.3 (9.0–9.7) | 10.8 (10.3–11.3) | 14.0 (14.0–14.7) | 14.4 (14.4–15.1) |
| native | read | snapshot | rc4-local | 22.9 (18.7–23.0) | 16.2 (15.2–17.3) | 18.2 (16.8–19.3) | 18.8 (17.7–19.8) |
| native | read | snapshot | main-local | 18.5 (18.1–19.6) | 16.2 (15.5–17.1) | 17.9 (17.2–18.3) | 18.1 (17.9–18.9) |
| native | read | snapshot.drop | rc4-local | 6.8 (6.5–7.0) | 6.3 (5.9–6.9) | 7.1 (6.6–7.7) | 7.4 (7.1–8.0) |
| native | read | snapshot.drop | main-local | 6.7 (6.3–6.8) | 6.3 (6.1–6.7) | 7.0 (6.7–7.1) | 7.1 (7.0–7.4) |
| native | read | snapshot.serialize | rc4-local | 15.3 (14.1–15.5) | 11.1 (10.4–11.8) | 11.9 (11.1–12.5) | 12.2 (11.6–12.8) |
| native | read | snapshot.serialize | main-local | 13.6 (12.9–15.2) | 11.3 (10.7–11.8) | 12.0 (11.3–12.2) | 11.8 (11.7–12.3) |
| native | read | snapshot.serialize_pretty | rc4-local | 26.2 (24.1–26.4) | 22.1 (20.8–23.6) | 24.1 (22.5–25.5) | 24.5 (23.3–25.9) |
| native | read | snapshot.serialize_pretty | main-local | 25.3 (24.7–25.5) | 22.3 (21.3–23.6) | 24.2 (22.9–24.4) | 23.8 (23.6–24.8) |
| native | read | view | rc4-local | 2.4 (2.1–2.4) | 2.0 (1.8–2.1) | 2.4 (2.2–2.6) | 2.6 (2.4–2.7) |
| native | read | view | main-local | 2.1 (2.1–2.2) | 2.0 (1.9–2.0) | 2.3 (2.2–2.4) | 2.4 (2.4–2.6) |
| native | read | view.serialize | rc4-local | 3.2 (2.9–3.4) | 2.2 (2.1–2.4) | 2.3 (2.2–2.5) | 2.4 (2.3–2.5) |
| native | read | view.serialize | main-local | 3.2 (2.9–3.2) | 2.3 (2.2–2.4) | 2.4 (2.3–2.5) | 2.5 (2.4–2.5) |
| wasm | abi.dispatch_view | abi.encode_args | rc4-published | 0.8 (0.7–0.9) | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | rc4-local | 0.9 (0.8–1.3) | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | main-local | 0.8 (0.8–0.9) | 0.3 (0.3–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.encode_args | hist-e6ace96 | 0.8 (0.7–0.8) | 0.2 (0.2–0.3) | 0.1 (0.1–0.2) | 0.1 (0.1–0.2) |
| wasm | abi.dispatch_view | abi.exec | rc4-published | 60.2 (58.8–65.0) | 20.2 (17.1–20.5) | 19.2 (18.1–19.9) | 17.7 (17.2–18.7) |
| wasm | abi.dispatch_view | abi.exec | rc4-local | 51.4 (50.4–58.1) | 19.8 (19.1–20.3) | 19.4 (19.0–21.3) | 17.7 (17.1–18.4) |
| wasm | abi.dispatch_view | abi.exec | main-local | 53.7 (53.5–55.6) | 20.3 (19.6–20.4) | 19.5 (18.5–19.5) | 17.9 (17.2–18.6) |
| wasm | abi.dispatch_view | abi.exec | hist-e6ace96 | 50.3 (48.2–52.6) | 20.2 (18.7–20.5) | 18.3 (17.4–20.3) | 17.4 (16.4–19.0) |
| wasm | abi.dispatch_view | abi.decode | rc4-published | 1.5 (1.2–3.0) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) |
| wasm | abi.dispatch_view | abi.decode | rc4-local | 1.2 (1.1–2.1) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) |
| wasm | abi.dispatch_view | abi.decode | main-local | 1.3 (1.3–1.3) | 0.5 (0.5–0.6) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) |
| wasm | abi.dispatch_view | abi.decode | hist-e6ace96 | 1.4 (1.4–1.5) | 0.5 (0.5–0.5) | 0.5 (0.5–0.5) | 0.5 (0.4–0.5) |
| wasm | abi.dispatch_view | abi.free | rc4-published | 0.3 (0.3–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | rc4-local | 0.2 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | main-local | 0.3 (0.2–0.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.free | hist-e6ace96 | 0.2 (0.2–0.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | abi.dispatch_view | abi.dispatch_view | rc4-published | 64.3 (61.5–67.1) | 21.1 (17.9–21.4) | 20.0 (18.7–20.7) | 18.5 (18.0–19.5) |
| wasm | abi.dispatch_view | abi.dispatch_view | rc4-local | 54.1 (54.0–60.8) | 20.7 (20.1–21.2) | 20.2 (19.7–22.1) | 18.5 (17.8–19.2) |
| wasm | abi.dispatch_view | abi.dispatch_view | main-local | 56.0 (55.7–57.4) | 21.2 (20.6–21.3) | 20.3 (19.2–20.3) | 18.7 (17.9–19.4) |
| wasm | abi.dispatch_view | abi.dispatch_view | hist-e6ace96 | 52.0 (50.7–54.2) | 21.0 (19.5–21.5) | 19.1 (18.1–21.2) | 18.2 (17.1–20.0) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-published | 0.9 (0.8–0.9) | 0.3 (0.2–0.3) | 0.3 (0.2–0.3) | 0.2 (0.2–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-local | 0.9 (0.8–1.0) | 0.3 (0.3–0.3) | 0.3 (0.2–0.3) | 0.3 (0.2–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local | 0.9 (0.8–1.2) | 0.2 (0.2–0.3) | 0.2 (0.2–0.3) | 0.2 (0.2–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | hist-e6ace96 | 0.8 (0.7–0.9) | 0.3 (0.2–0.3) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-published | 69.2 (56.9–79.9) | 22.9 (20.4–25.2) | 20.1 (18.7–24.1) | 19.4 (18.6–22.2) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-local | 55.3 (51.5–83.2) | 20.4 (19.8–22.6) | 20.3 (18.4–21.3) | 19.8 (18.3–21.5) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local | 70.5 (56.5–76.1) | 19.8 (18.9–21.2) | 18.8 (18.5–20.5) | 18.6 (18.3–19.5) |
| wasm | raw.dispatch_view | raw.dispatch_view | hist-e6ace96 | 54.4 (52.3–60.3) | 21.3 (19.7–21.8) | 19.3 (18.3–19.4) | 18.8 (17.3–18.9) |
| wasm | raw.dispatch_view | js.parse_view | rc4-published | 15.5 (13.1–18.5) | 6.3 (5.8–7.2) | 6.2 (5.7–7.2) | 6.3 (5.9–7.1) |
| wasm | raw.dispatch_view | js.parse_view | rc4-local | 12.2 (11.1–12.7) | 5.8 (5.6–6.5) | 6.1 (5.6–6.4) | 6.4 (5.9–7.0) |
| wasm | raw.dispatch_view | js.parse_view | main-local | 12.3 (10.4–13.9) | 5.6 (5.3–6.0) | 5.7 (5.6–6.2) | 6.0 (5.9–6.3) |
| wasm | raw.dispatch_view | js.parse_view | hist-e6ace96 | 12.3 (10.1–12.6) | 6.0 (5.6–6.2) | 5.8 (5.8–6.0) | 6.2 (5.8–6.2) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-published | 85.4 (71.0–99.7) | 29.6 (26.6–32.9) | 26.5 (24.6–31.8) | 26.0 (24.8–29.8) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-local | 68.5 (63.5–94.4) | 26.5 (25.7–29.5) | 26.7 (24.3–28.0) | 26.5 (24.4–28.9) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local | 83.2 (67.9–91.2) | 25.7 (24.5–27.5) | 24.8 (24.3–27.0) | 24.8 (24.6–26.2) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | hist-e6ace96 | 68.5 (65.4–73.6) | 27.7 (25.7–28.4) | 25.3 (24.2–25.7) | 25.3 (23.3–25.4) |

## glowcap-scaled-16

Glowcap with 12 more mushrooms declared after grove (16 in all): symbols, edges, tick rules and bindings grow with the mushroom count; the replay's first 2,000 events. Program `experiments/glowcap/caveat5/glowcap.cav` + 12 mushrooms (`468147924b4a`), stream `f93d60a5f44d`, 1 episode(s) × 1 = 2000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 30.2 (28.5–37.3) · p95 35.5 | 30.1 (28.1–36.1) · p95 35.6 | - |
| native | web.dispatch_view | web.dispatch_view | - | 46.5 (43.4–47.4) · p95 52.4 | 46.3 (44.2–47.0) · p95 51.6 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 150.7 (144.3–162.9) · p95 166.2 | 144.4 (143.5–152.6) · p95 172.3 | - |
| native | read | clone | - | 17.5 (17.0–18.4) · p95 20.1 | 18.2 (17.7–18.4) · p95 20.2 | - |
| native | read | clone.drop | - | 9.7 (9.6–10.3) · p95 11.1 | 10.2 (10.0–10.4) · p95 11.3 | - |
| native | read | save | - | 9.4 (9.3–9.9) · p95 11.2 | 9.7 (9.5–10.3) · p95 11.5 | - |
| native | read | save_json | - | 15.0 (15.0–15.7) · p95 17.3 | 15.4 (15.1–16.0) · p95 17.9 | - |
| native | read | snapshot | - | 57.3 (56.0–59.9) · p95 68.8 | 59.5 (57.1–60.3) · p95 67.6 | - |
| native | read | snapshot.drop | - | 22.4 (22.1–23.4) · p95 25.5 | 23.5 (22.6–23.5) · p95 25.5 | - |
| native | read | snapshot.serialize | - | 32.2 (31.6–33.7) · p95 41.0 | 34.3 (32.2–34.5) · p95 41.6 | - |
| native | read | snapshot.serialize_pretty | - | 69.9 (68.3–73.0) · p95 100.2 | 74.1 (69.4–74.3) · p95 87.2 | - |
| native | read | view | - | 6.0 (5.8–6.2) · p95 6.8 | 6.1 (5.9–6.1) · p95 6.8 | - |
| native | read | view.serialize | - | 8.6 (8.5–9.0) · p95 10.0 | 9.1 (8.7–9.2) · p95 10.3 | - |
| native | lifecycle | from_source | - | 10829.3 (10650.3–11255.3) · p95 11164.5 | 11066.9 (10599.6–11238.3) · p95 11802.4 | - |
| native | lifecycle | restore | - | 10906.9 (10895.7–11622.7) · p95 11458.5 | 11386.7 (10877.8–11519.1) · p95 12136.1 | - |
| native | lifecycle | restore.parse | - | 36.2 (35.6–36.7) · p95 44.3 | 37.0 (36.4–37.7) · p95 47.2 | - |
| native | lifecycle | restore_json | - | 10918.7 (10915.0–11661.1) · p95 11877.8 | 11579.1 (10892.2–11655.0) · p95 12047.4 | - |
| native | lifecycle | save | - | 34.5 (32.7–35.3) · p95 44.3 | 33.1 (32.5–33.1) · p95 44.6 | - |
| native | lifecycle | save_json | - | 26.5 (25.4–27.8) · p95 28.9 | 26.4 (25.6–26.4) · p95 29.2 | - |
| native | lifecycle | web.new | - | 10768.7 (10438.6–11220.3) · p95 11147.1 | 11080.8 (10659.4–11320.0) · p95 11706.9 | - |
| native | lifecycle | web.restore | - | 10925.4 (10895.0–11747.9) · p95 11610.5 | 11480.0 (10851.4–11516.0) · p95 12424.7 | - |
| native | lifecycle | web.save | - | 27.1 (27.0–29.6) · p95 34.0 | 28.1 (26.6–28.3) · p95 32.2 | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.6 | 0.3 (0.3–0.3) · p95 0.6 | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 59.4 (58.5–62.6) · p95 70.8 | 58.6 (56.9–59.5) · p95 69.8 | 59.4 (59.1–64.5) · p95 71.3 | 55.5 (54.8–57.7) · p95 65.6 |
| wasm | raw.dispatch_view | js.parse_view | 38.6 (38.3–41.2) · p95 44.5 | 38.5 (36.5–39.6) · p95 43.9 | 38.4 (37.4–41.9) · p95 44.1 | 36.8 (36.1–38.3) · p95 42.0 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 98.7 (97.4–104.4) · p95 114.7 | 97.6 (93.7–99.8) · p95 112.7 | 98.5 (96.6–107.3) · p95 115.7 | 92.9 (91.5–96.7) · p95 107.7 |
| wasm | abi.dispatch_view | abi.encode_args | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | abi.dispatch_view | abi.exec | 49.6 (49.3–52.3) · p95 61.5 | 51.1 (50.3–52.0) · p95 60.4 | 52.3 (49.5–52.6) · p95 60.8 | 49.9 (48.9–51.0) · p95 59.7 |
| wasm | abi.dispatch_view | abi.decode | 5.5 (5.4–5.7) · p95 6.8 | 5.6 (5.5–5.7) · p95 6.8 | 5.6 (5.4–5.7) · p95 6.9 | 5.7 (5.6–5.7) · p95 7.0 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | abi.dispatch_view | abi.dispatch_view | 55.5 (55.1–58.5) · p95 68.7 | 57.0 (56.2–58.2) · p95 67.7 | 58.4 (55.3–58.8) · p95 68.3 | 56.1 (54.9–57.2) · p95 67.0 |
| wasm | read | raw.view | 24.4 (23.4–24.6) · p95 29.1 | 23.8 (23.4–24.8) · p95 29.2 | 23.6 (23.1–24.7) · p95 28.9 | 22.8 (21.9–22.8) · p95 27.2 |
| wasm | read | js.parse_view | 39.8 (38.4–40.6) · p95 46.1 | 39.0 (38.6–40.7) · p95 47.1 | 39.6 (38.8–41.3) · p95 47.7 | 39.0 (37.8–39.5) · p95 46.5 |
| wasm | read | raw.snapshot | 265.9 (258.1–275.5) · p95 301.1 | 266.5 (261.1–273.2) · p95 303.6 | 263.6 (257.6–273.7) · p95 304.2 | 262.9 (258.2–264.3) · p95 303.6 |
| wasm | read | js.parse_snapshot | 164.3 (158.5–168.5) · p95 186.0 | 161.4 (158.9–168.2) · p95 186.7 | 163.0 (160.0–170.2) · p95 190.4 | 162.5 (157.6–163.3) · p95 188.4 |
| wasm | read | raw.save | 22.6 (22.5–23.7) · p95 28.4 | 22.9 (22.3–23.8) · p95 28.8 | 22.9 (22.1–23.7) · p95 28.6 | 21.8 (21.5–22.4) · p95 28.1 |
| wasm | kit | kit.dispatch | 344.2 (327.7–350.3) · p95 409.1 | 331.4 (328.5–346.6) · p95 394.3 | 328.0 (326.4–346.7) · p95 396.8 | - |
| wasm | kit | kit.view | 66.1 (63.6–67.3) · p95 77.3 | 64.4 (63.9–67.2) · p95 75.4 | 64.1 (63.4–66.9) · p95 75.3 | - |
| wasm | kit | kit.dispatch+view | 411.0 (391.5–418.2) · p95 483.8 | 396.2 (393.1–414.8) · p95 467.3 | 392.7 (390.6–414.5) · p95 470.2 | - |
| wasm | lifecycle | raw.new | 9960.7 (9929.5–10597.3) · p95 11109.0 | 10164.7 (9973.5–10710.6) · p95 10948.3 | 9908.8 (9823.8–10839.7) · p95 10787.3 | 10156.9 (9668.2–10261.1) · p95 10736.5 |
| wasm | lifecycle | raw.save | 61.4 (59.2–61.6) · p95 86.2 | 63.6 (62.3–66.9) · p95 84.2 | 61.6 (58.8–63.2) · p95 87.0 | 58.3 (56.9–58.3) · p95 74.9 |
| wasm | lifecycle | raw.restore | 10004.8 (9932.0–10753.7) · p95 11087.4 | 10012.5 (9927.3–10628.7) · p95 11121.6 | 9963.6 (9955.0–10662.3) · p95 11288.4 | 10196.6 (9465.7–10276.8) · p95 11103.5 |
| wasm | lifecycle | kit.open | 9997.2 (9902.8–10950.1) · p95 11284.3 | 10225.9 (10058.1–10732.3) · p95 11378.9 | 10762.0 (9996.7–10844.3) · p95 11302.7 | - |
| wasm | lifecycle | kit.save | 121.9 (114.2–129.0) · p95 165.6 | 128.2 (126.5–128.9) · p95 155.1 | 126.4 (123.9–131.7) · p95 161.5 | - |
| wasm | lifecycle | kit.restore | 10095.4 (10064.0–10877.1) · p95 11381.7 | 10142.3 (10124.2–10761.6) · p95 11354.4 | 10062.5 (10043.7–10846.5) · p95 11146.8 | - |

## glowcap-scaled-64

Glowcap with 60 more mushrooms declared after grove (64 in all); the replay's first 2,000 events. Program `experiments/glowcap/caveat5/glowcap.cav` + 60 mushrooms (`29f4f9803de2`), stream `f93d60a5f44d`, 1 episode(s) × 1 = 2000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 93.4 (88.7–101.8) · p95 104.7 | 90.3 (88.1–92.6) · p95 103.3 | - |
| native | web.dispatch_view | web.dispatch_view | - | 146.3 (138.4–147.1) · p95 158.7 | 139.3 (137.8–146.6) · p95 153.1 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 456.3 (453.2–481.3) · p95 500.7 | 459.4 (456.5–487.1) · p95 511.8 | - |
| native | read | clone | - | 58.5 (58.5–61.8) · p95 66.8 | 59.4 (58.1–62.2) · p95 67.6 | - |
| native | read | clone.drop | - | 32.2 (31.9–34.0) · p95 36.9 | 32.8 (32.1–34.3) · p95 37.2 | - |
| native | read | save | - | 13.1 (13.0–13.7) · p95 15.4 | 13.3 (13.2–14.2) · p95 15.9 | - |
| native | read | save_json | - | 19.1 (18.9–19.9) · p95 21.9 | 19.1 (19.0–20.0) · p95 21.5 | - |
| native | read | snapshot | - | 188.1 (187.9–197.5) · p95 209.4 | 191.2 (188.1–202.0) · p95 225.4 | - |
| native | read | snapshot.drop | - | 67.8 (67.6–72.2) · p95 78.1 | 69.4 (67.7–73.1) · p95 78.5 | - |
| native | read | snapshot.serialize | - | 100.9 (100.5–105.4) · p95 114.6 | 102.6 (102.0–108.7) · p95 151.0 | - |
| native | read | snapshot.serialize_pretty | - | 213.3 (212.9–223.5) · p95 235.7 | 214.6 (214.4–227.2) · p95 247.4 | - |
| native | read | view | - | 22.6 (22.5–23.7) · p95 25.5 | 22.5 (22.2–23.7) · p95 25.7 | - |
| native | read | view.serialize | - | 24.4 (24.3–25.3) · p95 28.5 | 24.7 (24.5–26.2) · p95 30.6 | - |
| native | lifecycle | from_source | - | 84755.2 (82709.2–87675.3) · p95 90035.0 | 83410.7 (82900.7–87693.6) · p95 87292.6 | - |
| native | lifecycle | restore | - | 85583.9 (84868.2–88728.8) · p95 91362.4 | 84435.3 (83738.7–89582.3) · p95 88101.5 | - |
| native | lifecycle | restore.parse | - | 37.5 (36.3–37.9) · p95 45.6 | 38.2 (36.6–39.1) · p95 49.0 | - |
| native | lifecycle | restore_json | - | 84663.0 (84118.0–89206.4) · p95 92394.0 | 84174.4 (83688.8–89219.4) · p95 87027.6 | - |
| native | lifecycle | save | - | 54.8 (53.7–55.5) · p95 71.7 | 53.0 (52.4–55.6) · p95 74.8 | - |
| native | lifecycle | save_json | - | 30.3 (29.5–31.5) · p95 35.1 | 29.6 (29.2–31.0) · p95 34.8 | - |
| native | lifecycle | web.new | - | 84797.4 (84538.3–87920.0) · p95 90666.8 | 84380.0 (82550.0–88181.9) · p95 88474.7 | - |
| native | lifecycle | web.restore | - | 86050.2 (83868.0–89166.2) · p95 91196.4 | 84363.7 (84272.3–89923.6) · p95 87562.9 | - |
| native | lifecycle | web.save | - | 41.4 (40.8–43.0) · p95 49.8 | 39.5 (39.3–41.8) · p95 51.4 | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.4 (0.3–0.4) · p95 0.7 | 0.4 (0.4–0.4) · p95 0.7 | 0.4 (0.4–0.4) · p95 0.7 | 0.4 (0.3–0.4) · p95 0.7 |
| wasm | raw.dispatch_view | raw.dispatch_view | 190.2 (184.6–190.5) · p95 236.4 | 184.9 (180.2–188.1) · p95 235.6 | 188.2 (179.6–194.7) · p95 237.2 | 183.1 (176.5–190.0) · p95 237.8 |
| wasm | raw.dispatch_view | js.parse_view | 121.5 (120.1–124.3) · p95 136.2 | 119.1 (116.2–122.8) · p95 138.5 | 122.2 (117.0–124.9) · p95 137.8 | 122.1 (116.5–126.7) · p95 134.7 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 311.7 (304.8–316.1) · p95 370.3 | 304.6 (297.0–312.5) · p95 367.6 | 312.0 (297.5–320.5) · p95 364.4 | 306.7 (293.2–317.4) · p95 364.9 |
| wasm | abi.dispatch_view | abi.encode_args | 0.2 (0.2–0.2) · p95 0.5 | 0.2 (0.2–0.3) · p95 0.5 | 0.2 (0.2–0.3) · p95 0.5 | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | abi.dispatch_view | abi.exec | 158.8 (156.4–164.3) · p95 180.6 | 167.5 (165.6–188.5) · p95 195.5 | 165.9 (157.3–166.9) · p95 186.7 | 155.6 (155.0–161.0) · p95 175.6 |
| wasm | abi.dispatch_view | abi.decode | 17.7 (17.5–18.4) · p95 21.5 | 18.7 (18.1–21.0) · p95 23.5 | 18.5 (17.6–18.7) · p95 22.0 | 17.8 (17.7–18.4) · p95 21.7 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | abi.dispatch_view | abi.dispatch_view | 177.2 (174.7–183.4) · p95 207.4 | 186.9 (184.5–210.6) · p95 224.6 | 185.2 (175.7–186.7) · p95 219.0 | 174.0 (173.5–180.1) · p95 207.1 |
| wasm | read | raw.view | 87.8 (85.7–90.7) · p95 98.9 | 90.9 (86.3–92.2) · p95 99.9 | 89.1 (85.9–90.9) · p95 100.0 | 87.2 (83.7–87.3) · p95 99.5 |
| wasm | read | js.parse_view | 122.7 (120.8–126.4) · p95 137.1 | 127.1 (119.9–129.8) · p95 136.3 | 123.4 (122.1–127.5) · p95 136.8 | 126.0 (120.5–126.5) · p95 143.3 |
| wasm | read | raw.snapshot | 834.3 (823.2–866.8) · p95 916.5 | 876.1 (815.9–885.6) · p95 924.0 | 838.7 (814.7–868.7) · p95 913.9 | 866.7 (830.1–872.2) · p95 966.1 |
| wasm | read | js.parse_snapshot | 605.8 (602.4–626.3) · p95 668.9 | 628.3 (592.9–642.0) · p95 666.6 | 611.7 (601.7–627.2) · p95 664.8 | 621.9 (594.3–624.6) · p95 691.6 |
| wasm | read | raw.save | 32.6 (32.5–33.9) · p95 39.3 | 33.8 (32.5–34.3) · p95 41.8 | 33.2 (32.2–34.1) · p95 40.7 | 32.3 (32.0–33.0) · p95 39.9 |
| wasm | kit | kit.dispatch | 1241.4 (1241.4–1285.8) · p95 1430.4 | 1291.5 (1257.8–1305.6) · p95 1445.9 | 1293.9 (1291.5–1309.3) · p95 1431.9 | - |
| wasm | kit | kit.view | 220.5 (218.9–227.6) · p95 663.0 | 230.7 (224.9–231.3) · p95 671.5 | 231.6 (227.9–233.5) · p95 272.4 | - |
| wasm | kit | kit.dispatch+view | 1466.6 (1460.4–1515.6) · p95 1924.8 | 1530.7 (1487.5–1541.0) · p95 1948.4 | 1549.7 (1522.0–1571.6) · p95 1709.2 | - |
| wasm | lifecycle | raw.new | 79195.1 (78299.2–84116.8) · p95 80573.7 | 82552.1 (77409.0–83423.2) · p95 87987.7 | 80804.1 (78613.5–83497.1) · p95 85097.8 | 79457.2 (76058.7–80511.0) · p95 81826.4 |
| wasm | lifecycle | raw.save | 96.6 (93.6–108.2) · p95 133.7 | 98.4 (96.6–98.5) · p95 128.7 | 95.1 (91.6–95.5) · p95 128.3 | 92.1 (90.4–98.4) · p95 125.8 |
| wasm | lifecycle | raw.restore | 79377.2 (78214.0–83508.7) · p95 80594.0 | 80364.3 (78517.3–82072.8) · p95 83012.3 | 80059.1 (78362.3–81681.8) · p95 82314.2 | 80033.6 (75966.0–80195.8) · p95 81766.2 |
| wasm | lifecycle | kit.open | 78395.7 (76853.9–82808.4) · p95 81780.5 | 78713.2 (76832.6–81427.5) · p95 82879.4 | 78842.9 (77623.0–80944.4) · p95 81572.8 | - |
| wasm | lifecycle | kit.save | 169.3 (166.0–179.1) · p95 212.8 | 170.0 (167.8–181.1) · p95 216.7 | 170.1 (165.8–174.8) · p95 216.8 | - |
| wasm | lifecycle | kit.restore | 78767.2 (77585.1–82432.2) · p95 80404.8 | 79065.1 (78583.8–80780.9) · p95 83085.1 | 78893.1 (77290.2–80919.9) · p95 83040.0 | - |

## ledger-session

The agent ledger replaying the real 2026-09-24 session log: 20 events, five of them refused by policy. Played 100 times from a fresh session. Program `experiments/agent-ledger/ledger.cav` (`2206ac79f23c`), stream `ef2143311f86`, 1 episode(s) × 100 = 2000 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 14.7 (14.6–14.9) · p95 21.5 | 16.2 (14.9–16.7) · p95 23.5 | - |
| native | dispatch_view_json | dispatch_view_json | - | 20.2 (20.1–22.0) · p95 28.0 | 20.4 (20.0–22.0) · p95 28.2 | - |
| native | dispatch_json | dispatch_json | - | 35.4 (35.2–38.5) · p95 44.1 | 35.4 (35.2–35.9) · p95 43.8 | - |
| native | dispatch_outcome_json | dispatch_outcome_json | - | 35.4 (35.2–35.5) · p95 44.4 | 35.3 (35.2–35.6) · p95 44.1 | - |
| native | web.dispatch_view | web.dispatch_view | - | 25.0 (22.8–25.8) · p95 34.0 | 24.2 (23.1–24.9) · p95 33.6 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 56.7 (56.4–60.0) · p95 74.4 | 56.6 (56.1–63.7) · p95 71.9 | - |
| native | web.dispatch | web.dispatch | - | 69.5 (68.9–69.8) · p95 83.1 | 71.3 (71.0–75.6) · p95 86.6 | - |
| native | read | clone | - | 1.7 (1.7–1.8) · p95 2.0 | 1.7 (1.7–1.7) · p95 1.9 | - |
| native | read | clone.drop | - | 1.0 (0.9–1.0) · p95 1.1 | 1.0 (1.0–1.0) · p95 1.1 | - |
| native | read | save | - | 6.3 (6.2–6.7) · p95 8.5 | 6.4 (6.3–6.5) · p95 8.7 | - |
| native | read | save_json | - | 11.4 (11.3–12.1) · p95 16.3 | 11.4 (11.4–11.5) · p95 16.0 | - |
| native | read | snapshot | - | 17.3 (17.1–18.7) · p95 19.9 | 17.3 (17.1–17.6) · p95 19.7 | - |
| native | read | snapshot.drop | - | 6.9 (6.9–7.5) · p95 7.8 | 6.9 (6.8–7.0) · p95 7.8 | - |
| native | read | snapshot.serialize | - | 13.4 (13.2–14.3) · p95 16.0 | 13.3 (13.2–13.4) · p95 15.5 | - |
| native | read | snapshot.serialize_pretty | - | 26.8 (26.4–28.6) · p95 32.9 | 26.5 (26.3–26.7) · p95 30.0 | - |
| native | read | view | - | 3.1 (3.1–3.4) · p95 4.5 | 3.1 (3.1–3.1) · p95 4.4 | - |
| native | read | view.serialize | - | 2.3 (2.2–2.4) · p95 2.9 | 2.3 (2.3–2.3) · p95 2.9 | - |
| native | web.read | web.save | - | 12.3 (12.1–12.4) · p95 16.5 | 12.4 (12.4–13.6) · p95 16.9 | - |
| native | web.read | web.snapshot | - | 50.6 (49.8–50.6) · p95 57.7 | 50.8 (50.6–55.4) · p95 59.3 | - |
| native | web.read | web.view | - | 5.1 (5.1–5.1) · p95 7.3 | 5.1 (5.1–5.6) · p95 7.3 | - |
| native | lifecycle | from_source | - | 1318.0 (1310.7–1446.8) · p95 1468.3 | 1308.8 (1286.9–1317.1) · p95 1419.3 | - |
| native | lifecycle | restore | - | 1377.4 (1371.7–1498.3) · p95 1449.9 | 1335.9 (1328.0–1359.8) · p95 1443.9 | - |
| native | lifecycle | restore.parse | - | 23.1 (22.6–24.9) · p95 26.1 | 22.7 (21.8–22.8) · p95 24.8 | - |
| native | lifecycle | restore_json | - | 1402.8 (1395.3–1471.6) · p95 1507.8 | 1379.2 (1360.2–1392.7) · p95 1475.7 | - |
| native | lifecycle | save | - | 15.1 (14.3–15.3) · p95 16.9 | 13.7 (13.6–13.7) · p95 14.9 | - |
| native | lifecycle | save_json | - | 21.2 (21.0–23.0) · p95 23.5 | 20.9 (20.6–21.1) · p95 22.8 | - |
| native | lifecycle | web.new | - | 1322.6 (1308.8–1431.6) · p95 1439.8 | 1286.7 (1283.7–1303.6) · p95 1356.0 | - |
| native | lifecycle | web.restore | - | 1407.8 (1390.2–1494.8) · p95 1490.7 | 1349.1 (1345.9–1355.1) · p95 1475.8 | - |
| native | lifecycle | web.save | - | 15.9 (15.7–17.5) · p95 16.5 | 15.8 (15.4–15.9) · p95 16.6 | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.9 | 0.3 (0.3–0.3) · p95 0.9 | 0.3 (0.3–0.3) · p95 0.9 | 0.3 (0.3–0.3) · p95 0.9 |
| wasm | raw.dispatch_view | raw.dispatch_view | 28.7 (28.3–29.6) · p95 37.8 | 28.5 (28.5–28.8) · p95 38.3 | 28.8 (28.1–29.1) · p95 38.5 | 27.4 (27.0–27.7) · p95 37.2 |
| wasm | raw.dispatch_view | js.parse_view | 6.9 (6.8–7.1) · p95 9.9 | 6.9 (6.7–7.0) · p95 9.8 | 6.8 (6.7–6.9) · p95 9.6 | 7.1 (6.8–7.2) · p95 9.8 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 38.1 (37.7–40.0) · p95 48.8 | 37.9 (37.8–38.3) · p95 48.6 | 38.1 (37.6–38.7) · p95 48.7 | 37.4 (36.7–38.4) · p95 47.2 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 66.1 (61.2–66.7) · p95 86.1 | 63.8 (62.2–65.5) · p95 83.5 | 63.1 (62.5–63.3) · p95 80.4 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 47.2 (43.9–47.6) · p95 59.0 | 44.3 (44.1–45.1) · p95 54.6 | 44.5 (44.5–44.6) · p95 55.3 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 114.2 (105.7–114.9) · p95 144.2 | 109.3 (107.0–109.5) · p95 137.7 | 107.8 (107.1–108.4) · p95 135.5 | - |
| wasm | raw.dispatch | raw.dispatch | 82.7 (80.8–86.2) · p95 109.7 | 82.1 (82.0–82.4) · p95 101.9 | 83.3 (80.5–84.1) · p95 105.1 | 87.5 (82.5–88.1) · p95 105.9 |
| wasm | abi.dispatch_view | abi.encode_args | 0.3 (0.3–0.3) · p95 0.6 | 0.3 (0.2–0.3) · p95 0.6 | 0.3 (0.3–0.3) · p95 0.6 | 0.3 (0.2–0.3) · p95 0.6 |
| wasm | abi.dispatch_view | abi.exec | 27.0 (27.0–27.9) · p95 36.5 | 27.1 (26.9–27.1) · p95 36.8 | 27.6 (27.6–28.3) · p95 37.7 | 26.0 (25.7–26.0) · p95 34.8 |
| wasm | abi.dispatch_view | abi.decode | 0.6 (0.6–0.6) · p95 1.8 | 0.6 (0.6–0.6) · p95 1.8 | 0.6 (0.6–0.6) · p95 2.0 | 0.6 (0.6–0.6) · p95 1.7 |
| wasm | abi.dispatch_view | abi.free | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 | 0.1 (0.1–0.1) · p95 0.3 |
| wasm | abi.dispatch_view | abi.dispatch_view | 30.9 (30.6–30.9) · p95 39.5 | 30.6 (30.4–31.1) · p95 39.5 | 31.7 (31.6–32.4) · p95 40.3 | 29.6 (29.2–29.6) · p95 37.4 |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.3 (0.3–0.3) · p95 0.7 | 0.3 (0.3–0.3) · p95 0.7 | 0.3 (0.3–0.3) · p95 0.7 | - |
| wasm | abi.dispatch_outcome | abi.exec | 58.7 (57.0–60.5) · p95 75.1 | 59.0 (58.3–60.8) · p95 76.2 | 57.9 (57.5–58.6) · p95 73.8 | - |
| wasm | abi.dispatch_outcome | abi.decode | 2.3 (2.2–2.3) · p95 6.8 | 2.3 (2.3–2.4) · p95 7.3 | 2.3 (2.2–2.3) · p95 7.2 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.4 (0.4–0.4) · p95 0.8 | 0.4 (0.4–0.4) · p95 0.8 | 0.4 (0.4–0.4) · p95 0.8 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 62.2 (60.6–64.1) · p95 80.0 | 62.6 (61.8–64.7) · p95 81.1 | 61.5 (61.1–62.2) · p95 79.0 | - |
| wasm | read | raw.view | 7.1 (7.1–7.2) · p95 10.0 | 7.1 (6.9–7.2) · p95 10.2 | 7.3 (7.2–7.3) · p95 10.3 | 6.7 (6.5–6.7) · p95 9.5 |
| wasm | read | js.parse_view | 7.9 (7.7–7.9) · p95 10.3 | 7.9 (7.6–7.9) · p95 10.4 | 8.1 (7.9–8.1) · p95 10.6 | 7.9 (7.8–8.0) · p95 10.4 |
| wasm | read | raw.snapshot | 68.3 (67.1–68.4) · p95 83.2 | 68.7 (66.1–68.9) · p95 84.9 | 69.4 (68.1–69.9) · p95 86.4 | 67.4 (66.3–67.7) · p95 83.1 |
| wasm | read | js.parse_snapshot | 51.6 (50.7–51.8) · p95 60.6 | 51.3 (50.4–51.5) · p95 62.2 | 52.7 (51.8–53.4) · p95 64.1 | 51.4 (51.1–51.6) · p95 62.1 |
| wasm | read | raw.save | 16.5 (16.2–16.6) · p95 22.4 | 16.5 (15.9–16.5) · p95 22.7 | 16.6 (16.5–16.8) · p95 23.5 | 15.5 (15.5–15.6) · p95 21.9 |
| wasm | abi.read | abi.view.exec | 6.7 (6.6–6.7) · p95 9.1 | 6.8 (6.6–7.0) · p95 9.1 | 6.7 (6.6–6.9) · p95 9.2 | 6.3 (6.1–6.4) · p95 8.6 |
| wasm | abi.read | abi.view.decode | 0.4 (0.4–0.4) · p95 0.5 | 0.4 (0.3–0.4) · p95 0.5 | 0.4 (0.4–0.4) · p95 0.5 | 0.4 (0.3–0.4) · p95 0.5 |
| wasm | abi.read | abi.snapshot.exec | 60.9 (60.5–61.3) · p95 73.3 | 61.3 (61.1–65.2) · p95 72.8 | 61.1 (60.7–63.3) · p95 73.1 | 61.5 (59.4–61.9) · p95 73.1 |
| wasm | abi.read | abi.snapshot.decode | 3.5 (3.4–3.6) · p95 6.1 | 3.5 (3.5–3.6) · p95 6.3 | 3.6 (3.5–3.7) · p95 6.2 | 3.6 (3.6–3.6) · p95 6.4 |
| wasm | abi.read | abi.save.exec | 15.0 (14.8–15.1) · p95 20.1 | 15.0 (14.9–15.5) · p95 20.1 | 14.8 (14.8–15.2) · p95 20.2 | 14.1 (13.7–14.3) · p95 19.8 |
| wasm | abi.read | abi.save.decode | 0.6 (0.6–0.6) · p95 0.9 | 0.6 (0.6–0.6) · p95 0.9 | 0.6 (0.6–0.6) · p95 0.9 | 0.6 (0.6–0.6) · p95 0.9 |
| wasm | kit | kit.dispatch | 111.7 (111.2–112.2) · p95 143.0 | 111.8 (110.3–112.1) · p95 142.0 | 110.6 (110.1–111.7) · p95 140.4 | - |
| wasm | kit | kit.view | 14.6 (14.5–14.7) · p95 18.8 | 14.6 (14.4–14.6) · p95 18.6 | 14.5 (14.2–14.6) · p95 18.6 | - |
| wasm | kit | kit.dispatch+view | 125.8 (125.4–126.2) · p95 162.0 | 125.9 (124.1–126.1) · p95 160.2 | 124.5 (124.0–125.8) · p95 158.4 | - |
| wasm | kit.read | kit.view | 16.0 (15.9–16.5) · p95 20.7 | 15.9 (15.9–16.1) · p95 19.9 | 15.9 (15.9–16.4) · p95 21.0 | - |
| wasm | kit.read | kit.snapshot | 117.1 (115.4–119.5) · p95 144.2 | 116.0 (115.9–116.4) · p95 141.8 | 116.4 (116.1–118.9) · p95 146.4 | - |
| wasm | kit.read | kit.save | 30.7 (30.6–31.5) · p95 41.9 | 30.5 (30.4–30.8) · p95 40.5 | 30.7 (30.5–31.4) · p95 42.0 | - |
| wasm | lifecycle | raw.new | 1575.8 (1427.1–1602.4) · p95 1739.6 | 1515.7 (1442.7–1576.3) · p95 1724.4 | 1605.4 (1481.9–1616.1) · p95 1730.8 | 1504.5 (1476.7–1597.7) · p95 2118.0 |
| wasm | lifecycle | raw.save | 49.3 (46.4–49.9) · p95 67.7 | 48.5 (46.4–48.9) · p95 69.2 | 49.3 (45.4–50.4) · p95 70.6 | 51.1 (47.3–51.5) · p95 78.1 |
| wasm | lifecycle | raw.restore | 1706.7 (1530.9–1709.4) · p95 1891.2 | 1581.2 (1569.7–1696.5) · p95 1933.3 | 1719.3 (1565.9–1722.6) · p95 1946.5 | 1595.4 (1493.8–1618.8) · p95 2383.2 |
| wasm | lifecycle | kit.open | 1649.4 (1453.7–1661.1) · p95 1841.2 | 1540.1 (1493.2–1622.9) · p95 1911.0 | 1634.0 (1512.1–1671.5) · p95 1775.6 | - |
| wasm | lifecycle | kit.save | 89.0 (85.0–93.4) · p95 118.7 | 90.4 (89.5–91.6) · p95 126.7 | 92.4 (83.6–94.1) · p95 126.4 | - |
| wasm | lifecycle | kit.restore | 1692.1 (1554.8–1700.9) · p95 1920.5 | 1612.8 (1574.3–1656.4) · p95 1864.9 | 1675.5 (1533.2–1680.5) · p95 1839.2 | - |
| wasm | micro | timer.hrtime_pair | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 |
| wasm | micro | js.stringify_payload.first | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | micro | js.stringify_payload.last | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.1–0.1) · p95 0.1 |
| wasm | micro | bridge.pass_ascii.payload | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 |
| wasm | micro | kit.payloadText.last | 0.2 (0.2–0.2) · p95 0.2 | 0.1 (0.1–0.2) · p95 0.2 | 0.2 (0.2–0.2) · p95 0.2 | - |
| wasm | micro | bridge.decode.view | 0.3 (0.3–0.3) · p95 0.4 | 0.3 (0.2–0.3) · p95 0.4 | 0.3 (0.3–0.3) · p95 0.4 | 0.3 (0.3–0.4) · p95 0.5 |
| wasm | micro | bridge.pass_ascii.view | 2.2 (2.1–2.4) · p95 2.3 | 2.2 (2.0–2.2) · p95 2.3 | 2.2 (2.1–2.4) · p95 2.6 | 2.3 (2.3–2.4) · p95 2.4 |
| wasm | micro | js.parse.view | 6.8 (6.6–7.0) · p95 7.5 | 6.8 (6.7–7.5) · p95 7.6 | 7.1 (6.8–7.1) · p95 7.6 | 6.6 (6.5–6.8) · p95 7.4 |
| wasm | micro | bridge.decode.outcome | 0.2 (0.1–0.2) · p95 0.2 | 0.1 (0.1–0.2) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | micro | bridge.pass_ascii.outcome | 0.2 (0.2–0.2) · p95 0.2 | 0.2 (0.2–0.2) · p95 0.2 | 0.2 (0.2–0.2) · p95 0.2 | - |
| wasm | micro | js.parse.outcome | 0.3 (0.3–0.3) · p95 0.4 | 0.3 (0.3–0.3) · p95 0.4 | 0.3 (0.3–0.3) · p95 0.3 | - |
| wasm | micro | bridge.decode.snapshot | 2.1 (2.0–2.2) · p95 15.6 | 2.1 (2.0–2.2) · p95 15.7 | 2.0 (2.0–2.1) · p95 15.3 | 2.0 (2.0–2.1) · p95 2.3 |
| wasm | micro | bridge.pass_ascii.snapshot | 28.1 (28.1–28.7) · p95 30.7 | 30.7 (26.9–31.5) · p95 32.5 | 28.3 (28.0–30.5) · p95 32.1 | 27.4 (27.3–27.9) · p95 29.6 |
| wasm | micro | js.parse.snapshot | 54.2 (51.1–55.8) · p95 58.7 | 55.0 (51.0–58.9) · p95 59.7 | 50.7 (49.2–52.8) · p95 55.2 | 50.5 (50.2–50.8) · p95 57.9 |
| wasm | micro | bridge.decode.save | 0.5 (0.5–0.6) · p95 0.6 | 0.5 (0.5–0.5) · p95 0.6 | 0.5 (0.4–0.5) · p95 0.6 | 0.5 (0.4–0.5) · p95 0.5 |
| wasm | micro | bridge.pass_ascii.save | 5.1 (4.9–5.3) · p95 5.5 | 5.2 (4.7–5.5) · p95 5.5 | 4.7 (4.7–4.7) · p95 4.9 | 4.7 (4.7–4.7) · p95 4.9 |
| wasm | micro | js.parse.save | 14.1 (14.1–14.8) · p95 16.0 | 14.0 (12.4–16.4) · p95 15.7 | 14.0 (13.9–14.6) · p95 15.2 | 14.4 (13.8–14.4) · p95 18.7 |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | rc4-published | rc4-local | main-local | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- | --- |
| native | apply | apply | - | 47.4 (46.6–50.6) · p95 121.6 | 46.7 (46.1–47.3) · p95 121.8 | - |
| native | dispatch_view_json | dispatch_view_json | - | 51.4 (49.8–51.5) · p95 127.5 | 52.9 (49.7–53.5) · p95 130.8 | - |
| native | dispatch_json | dispatch_json | - | 81.9 (80.9–82.5) · p95 157.7 | 82.0 (79.6–82.1) · p95 158.9 | - |
| native | dispatch_outcome_json | dispatch_outcome_json | - | 87.1 (80.9–90.1) · p95 168.8 | 83.3 (81.4–85.6) · p95 159.8 | - |
| native | web.dispatch_view | web.dispatch_view | - | 58.0 (57.6–60.5) · p95 132.6 | 58.4 (57.7–59.4) · p95 132.1 | - |
| native | web.dispatch_outcome | web.dispatch_outcome | - | 117.5 (115.6–118.2) · p95 193.9 | 117.2 (116.2–125.8) · p95 194.5 | - |
| native | web.dispatch | web.dispatch | - | 137.7 (135.8–148.2) · p95 215.7 | 139.1 (135.8–140.6) · p95 217.8 | - |
| native | read | clone | - | 11.0 (10.3–11.0) · p95 12.9 | 10.2 (10.2–10.3) · p95 12.0 | - |
| native | read | clone.drop | - | 6.0 (5.6–6.0) · p95 7.0 | 5.6 (5.5–5.6) · p95 6.5 | - |
| native | read | save | - | 5.7 (5.3–5.9) · p95 9.9 | 5.3 (5.3–5.3) · p95 9.1 | - |
| native | read | save_json | - | 9.8 (9.1–10.0) · p95 16.8 | 9.0 (9.0–9.1) · p95 15.6 | - |
| native | read | snapshot | - | 31.8 (29.8–32.0) · p95 36.7 | 29.9 (29.5–29.9) · p95 33.9 | - |
| native | read | snapshot.drop | - | 12.8 (11.9–12.9) · p95 15.3 | 11.9 (11.8–11.9) · p95 14.1 | - |
| native | read | snapshot.serialize | - | 19.9 (18.6–20.1) · p95 23.2 | 18.8 (18.7–18.9) · p95 21.4 | - |
| native | read | snapshot.serialize_pretty | - | 40.1 (37.3–40.4) · p95 47.5 | 37.5 (37.4–37.8) · p95 43.5 | - |
| native | read | view | - | 1.4 (1.3–1.4) · p95 2.4 | 1.3 (1.3–1.3) · p95 2.2 | - |
| native | read | view.serialize | - | 7.0 (6.6–7.1) · p95 9.2 | 6.6 (6.6–6.6) · p95 8.4 | - |
| native | web.read | web.save | - | 11.0 (10.9–11.0) · p95 18.3 | 10.8 (10.7–10.8) · p95 18.1 | - |
| native | web.read | web.snapshot | - | 79.4 (79.4–80.3) · p95 92.4 | 79.8 (79.4–80.0) · p95 91.3 | - |
| native | web.read | web.view | - | 6.1 (6.1–6.2) · p95 8.7 | 6.1 (6.1–6.1) · p95 8.6 | - |
| native | lifecycle | from_source | - | 2968.2 (2865.3–3149.9) · p95 3160.4 | 2891.2 (2888.9–2967.7) · p95 3136.7 | - |
| native | lifecycle | restore | - | 3035.3 (2965.8–3194.8) · p95 3302.2 | 2990.7 (2975.0–3034.0) · p95 3147.8 | - |
| native | lifecycle | restore.parse | - | 17.6 (17.4–20.5) · p95 21.5 | 19.1 (18.7–19.2) · p95 24.7 | - |
| native | lifecycle | restore_json | - | 3009.9 (3006.1–3273.4) · p95 3164.6 | 3045.8 (3002.8–3055.5) · p95 3189.9 | - |
| native | lifecycle | save | - | 12.0 (11.5–13.9) · p95 18.0 | 12.9 (12.0–13.6) · p95 15.7 | - |
| native | lifecycle | save_json | - | 14.7 (14.5–16.1) · p95 16.8 | 14.7 (14.1–17.3) · p95 16.9 | - |
| native | lifecycle | web.new | - | 2936.0 (2876.1–3132.3) · p95 3135.2 | 2913.2 (2893.8–2945.0) · p95 3038.4 | - |
| native | lifecycle | web.restore | - | 3045.0 (2977.3–3240.0) · p95 3107.7 | 3037.2 (3016.5–3072.1) · p95 3229.2 | - |
| native | lifecycle | web.save | - | 9.9 (9.8–11.0) · p95 11.5 | 10.5 (9.9–10.7) · p95 12.9 | - |
| wasm | raw.dispatch_view | js.stringify_payload | 0.5 (0.5–0.6) · p95 1.7 | 0.6 (0.6–0.6) · p95 1.7 | 0.5 (0.5–0.6) · p95 1.7 | - |
| wasm | raw.dispatch_view | raw.dispatch_view | 72.2 (71.5–73.0) · p95 147.5 | 72.7 (72.1–73.1) · p95 149.4 | 71.6 (70.7–75.0) · p95 147.6 | - |
| wasm | raw.dispatch_view | js.parse_view | 25.1 (25.1–25.2) · p95 31.3 | 25.4 (25.4–25.5) · p95 31.5 | 25.4 (24.9–25.9) · p95 31.0 | - |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 120.4 (119.4–121.1) · p95 177.9 | 120.7 (119.2–121.5) · p95 181.4 | 120.1 (118.9–123.7) · p95 178.2 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 142.8 (139.2–144.2) · p95 230.3 | 142.0 (140.6–144.4) · p95 229.0 | 142.2 (137.8–143.0) · p95 228.4 | - |
| wasm | raw.dispatch_outcome | js.parse_outcome | 76.5 (74.7–76.8) · p95 92.5 | 76.6 (75.7–80.4) · p95 94.0 | 75.9 (74.1–76.2) · p95 90.9 | - |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 222.4 (215.5–224.2) · p95 319.5 | 223.8 (220.2–224.2) · p95 313.8 | 220.5 (213.7–221.5) · p95 314.4 | - |
| wasm | raw.dispatch | raw.dispatch | 169.9 (164.8–184.4) · p95 255.6 | 167.6 (166.9–167.8) · p95 251.9 | 170.1 (166.3–173.6) · p95 258.0 | - |
| wasm | abi.dispatch_view | abi.encode_args | 0.4 (0.4–0.4) · p95 1.0 | 0.4 (0.4–0.4) · p95 1.0 | 0.4 (0.4–0.4) · p95 1.0 | - |
| wasm | abi.dispatch_view | abi.exec | 70.6 (68.8–71.0) · p95 146.1 | 71.1 (70.5–71.2) · p95 147.1 | 70.6 (70.0–72.1) · p95 147.5 | - |
| wasm | abi.dispatch_view | abi.decode | 1.2 (1.2–1.2) · p95 3.8 | 1.2 (1.2–1.3) · p95 3.9 | 1.2 (1.2–1.2) · p95 3.9 | - |
| wasm | abi.dispatch_view | abi.free | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 | - |
| wasm | abi.dispatch_view | abi.dispatch_view | 95.4 (93.9–95.8) · p95 151.6 | 95.6 (95.5–96.3) · p95 152.5 | 95.4 (94.9–98.6) · p95 153.0 | - |
| wasm | abi.dispatch_outcome | abi.encode_args | 0.4 (0.4–0.4) · p95 1.1 | 0.4 (0.4–0.5) · p95 1.1 | 0.4 (0.4–0.4) · p95 1.1 | - |
| wasm | abi.dispatch_outcome | abi.exec | 133.2 (131.8–135.0) · p95 217.5 | 131.9 (129.8–134.6) · p95 213.1 | 132.2 (129.1–135.1) · p95 214.3 | - |
| wasm | abi.dispatch_outcome | abi.decode | 3.2 (3.2–3.3) · p95 6.5 | 3.2 (3.2–3.2) · p95 6.5 | 3.2 (3.1–3.2) · p95 6.4 | - |
| wasm | abi.dispatch_outcome | abi.free | 0.4 (0.4–0.4) · p95 0.8 | 0.4 (0.4–0.4) · p95 0.8 | 0.4 (0.4–0.4) · p95 0.8 | - |
| wasm | abi.dispatch_outcome | abi.dispatch_outcome | 138.8 (137.0–140.8) · p95 225.5 | 137.2 (135.3–140.2) · p95 220.5 | 137.6 (134.5–140.6) · p95 221.7 | - |
| wasm | read | raw.view | 10.1 (10.1–10.2) · p95 13.8 | 10.0 (9.9–10.3) · p95 13.9 | 10.1 (9.9–10.2) · p95 13.8 | - |
| wasm | read | js.parse_view | 26.1 (25.8–26.4) · p95 32.7 | 25.9 (25.9–26.1) · p95 32.5 | 25.7 (25.6–25.8) · p95 33.1 | - |
| wasm | read | raw.snapshot | 107.0 (106.9–109.0) · p95 132.5 | 107.0 (106.8–107.7) · p95 134.3 | 106.6 (106.4–106.9) · p95 136.0 | - |
| wasm | read | js.parse_snapshot | 81.5 (81.1–81.8) · p95 96.7 | 81.1 (80.9–81.1) · p95 96.9 | 80.5 (80.1–80.8) · p95 98.8 | - |
| wasm | read | raw.save | 14.5 (14.3–14.8) · p95 25.0 | 14.5 (14.5–14.5) · p95 25.1 | 14.4 (14.4–14.5) · p95 25.1 | - |
| wasm | abi.read | abi.view.exec | 9.3 (9.1–9.6) · p95 12.6 | 9.1 (9.0–9.4) · p95 12.5 | 9.2 (9.1–9.4) · p95 12.5 | - |
| wasm | abi.read | abi.view.decode | 0.9 (0.8–0.9) · p95 1.3 | 0.9 (0.8–0.9) · p95 1.3 | 0.9 (0.8–0.9) · p95 1.3 | - |
| wasm | abi.read | abi.snapshot.exec | 100.7 (97.8–102.5) · p95 122.2 | 98.9 (97.7–99.3) · p95 120.7 | 99.2 (99.0–101.4) · p95 119.2 | - |
| wasm | abi.read | abi.snapshot.decode | 4.9 (4.7–4.9) · p95 8.3 | 4.8 (4.7–4.8) · p95 8.2 | 4.8 (4.8–4.8) · p95 8.1 | - |
| wasm | abi.read | abi.save.exec | 13.8 (13.3–13.9) · p95 23.7 | 13.4 (13.1–13.5) · p95 23.3 | 13.4 (13.4–13.7) · p95 23.2 | - |
| wasm | abi.read | abi.save.decode | 0.5 (0.5–0.5) · p95 0.8 | 0.5 (0.4–0.5) · p95 0.8 | 0.5 (0.4–0.5) · p95 0.8 | - |
| wasm | kit | kit.dispatch | 224.3 (223.4–226.0) · p95 319.1 | 221.6 (220.3–226.9) · p95 312.2 | 223.0 (221.5–223.5) · p95 316.8 | - |
| wasm | kit | kit.view | 34.6 (34.4–34.9) · p95 44.7 | 34.2 (33.5–35.2) · p95 44.3 | 34.2 (34.0–34.4) · p95 44.8 | - |
| wasm | kit | kit.dispatch+view | 261.2 (260.9–263.0) · p95 361.2 | 259.1 (257.2–264.4) · p95 353.3 | 260.3 (258.8–261.1) · p95 359.5 | - |
| wasm | kit.read | kit.view | 34.7 (34.4–34.7) · p95 44.4 | 34.9 (34.6–35.8) · p95 45.1 | 35.7 (34.4–36.7) · p95 45.8 | - |
| wasm | kit.read | kit.snapshot | 180.3 (178.3–180.4) · p95 223.3 | 181.4 (179.4–186.6) · p95 227.0 | 185.8 (179.8–190.5) · p95 229.4 | - |
| wasm | kit.read | kit.save | 27.2 (26.9–27.3) · p95 45.8 | 27.4 (27.2–28.2) · p95 46.1 | 27.9 (26.8–28.7) · p95 47.1 | - |
| wasm | lifecycle | raw.new | 2955.0 (2765.0–3338.1) · p95 3502.1 | 3005.3 (2774.7–3524.9) · p95 3294.4 | 2988.5 (2985.2–3286.6) · p95 3174.3 | - |
| wasm | lifecycle | raw.save | 36.6 (34.9–41.1) · p95 66.6 | 36.4 (35.3–44.9) · p95 59.6 | 36.6 (35.4–40.8) · p95 50.3 | - |
| wasm | lifecycle | raw.restore | 2970.1 (2841.2–3325.3) · p95 3597.8 | 3039.8 (2803.7–3514.0) · p95 3258.6 | 3001.4 (2988.1–3287.0) · p95 3201.3 | - |
| wasm | lifecycle | kit.open | 2895.5 (2709.0–3221.6) · p95 3429.2 | 2966.2 (2699.4–3449.9) · p95 3532.3 | 2956.6 (2939.1–3188.7) · p95 3123.6 | - |
| wasm | lifecycle | kit.save | 72.3 (65.3–82.4) · p95 107.3 | 72.7 (71.1–91.5) · p95 102.6 | 76.5 (67.4–77.8) · p95 97.8 | - |
| wasm | lifecycle | kit.restore | 2975.7 (2787.1–3341.1) · p95 3499.7 | 3005.8 (2795.3–3470.1) · p95 3363.9 | 3014.6 (2960.9–3282.4) · p95 3158.5 | - |
| wasm | micro | timer.hrtime_pair | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | micro | js.stringify_payload.first | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.2 | 0.1 (0.1–0.1) · p95 0.1 | - |
| wasm | micro | js.stringify_payload.last | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | - |
| wasm | micro | bridge.pass_ascii.payload | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | 0.0 (0.0–0.0) · p95 0.0 | - |
| wasm | micro | kit.payloadText.last | 0.1 (0.1–0.1) · p95 0.1 | 0.1 (0.0–0.1) · p95 0.1 | 0.0 (0.0–0.1) · p95 0.1 | - |
| wasm | micro | bridge.decode.view | 0.6 (0.6–0.6) · p95 0.7 | 0.5 (0.4–0.6) · p95 0.7 | 0.6 (0.5–0.6) · p95 0.7 | - |
| wasm | micro | bridge.pass_ascii.view | 5.2 (5.1–5.5) · p95 5.6 | 5.1 (4.8–5.3) · p95 5.5 | 5.0 (5.0–5.2) · p95 5.6 | - |
| wasm | micro | js.parse.view | 20.3 (20.0–20.8) · p95 25.3 | 20.2 (20.2–21.4) · p95 24.6 | 20.1 (19.9–20.1) · p95 24.8 | - |
| wasm | micro | bridge.decode.outcome | 1.5 (1.4–1.5) · p95 1.7 | 1.4 (1.4–1.5) · p95 1.9 | 1.3 (1.3–1.4) · p95 1.5 | - |
| wasm | micro | bridge.pass_ascii.outcome | 18.7 (18.1–19.1) · p95 19.5 | 18.7 (18.4–20.5) · p95 21.5 | 19.1 (18.8–19.6) · p95 19.9 | - |
| wasm | micro | js.parse.outcome | 67.4 (66.1–68.1) · p95 77.5 | 65.4 (63.6–70.0) · p95 72.6 | 65.7 (64.1–75.3) · p95 75.3 | - |
| wasm | micro | bridge.decode.snapshot | 2.6 (2.4–2.6) · p95 3.1 | 2.4 (2.0–2.5) · p95 2.9 | 2.3 (2.1–2.4) · p95 2.6 | - |
| wasm | micro | bridge.pass_ascii.snapshot | 33.6 (31.9–35.2) · p95 37.2 | 35.7 (32.3–36.1) · p95 38.7 | 32.3 (31.0–33.1) · p95 34.9 | - |
| wasm | micro | js.parse.snapshot | 70.4 (69.4–78.2) · p95 83.0 | 72.7 (72.7–79.6) · p95 78.0 | 69.6 (69.0–73.9) · p95 83.2 | - |
| wasm | micro | bridge.decode.save | 0.3 (0.3–0.3) · p95 0.4 | 0.3 (0.3–0.3) · p95 0.3 | 0.2 (0.2–0.3) · p95 0.3 | - |
| wasm | micro | bridge.pass_ascii.save | 2.2 (2.1–2.2) · p95 2.3 | 2.3 (2.3–2.5) · p95 2.5 | 2.1 (2.0–2.3) · p95 2.2 | - |
| wasm | micro | js.parse.save | 7.3 (7.1–8.5) · p95 7.9 | 7.5 (7.4–7.5) · p95 8.1 | 6.8 (6.1–7.4) · p95 7.4 | - |

