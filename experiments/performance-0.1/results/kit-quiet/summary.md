# Caveat performance baseline: 2026-09-29T05-20-51-056-baseline

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=baseline --repeats=3 --build=never --keep-samples --engines=wasm --workloads=glowcap-replay,trail-rescue-scenarios,ledger-session --modes=published-method,adapter,raw.dispatch_view,raw.dispatch_outcome,kit,kit.read --target=rc4-published=package:C:\Users\walte\caveat-rc4\package-files\package --target=main-local=tree:C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF --out=C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\perf-scratch\m\runs\quiet-kit`

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
| main-local | tree | v0.1.0-rc.4-5-gdbfd2e3-dirty (dbfd2e3), tracked changes | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | - |

## Environment

- CPU: Intel(R) Core(TM) Ultra 9 275HX; 24 cores, 24 logical processors; performance cores 0, 1, 10, 11, 12, 13, 22, 23
- Memory: 63.4 GiB
- OS: Microsoft Windows 11 Home 10.0.26200 (build 26200)
- Power: Power Scheme GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c  (High performance); battery status 2 (2 = on mains power)
- Pinning: affinity 0x3C00 (logical processors 10, 11, 12, 13), priority high, via cmd /c start /b /wait /<priority> /affinity <mask>
- Node v24.11.1, V8 13.6.233.10-node.28, npm 11.12.1
- Rust: rustc 1.98.1 (48a229cea 2026-09-01) (LLVM version: 22.1.8); cargo 1.98.1 (797e8a9bc 2026-08-05); 1.98.1-x86_64-pc-windows-msvc (overridden by 'C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prPERF\rust-toolchain.toml')
- wasm-bindgen 0.2.104; wasm-opt not installed (the build does not use it)
- Load before: total CPU 8/21/6%; busiest over the window: claude 1.266s, powershell 0.734s, claude 0.438s, ChatGPT 0.188s, IntelGraphicsSoftware 0.141s, ChatGPT 0.141s
- Load after: total CPU 5/8/0%; busiest over the window: claude 0.844s, powershell 0.641s, claude 0.344s, ChatGPT 0.281s, IntelGraphicsSoftware 0.125s, ChatGPT 0.062s
- Load during: typeperf every 5 s for the whole run, 94 samples: total mean 8.7% p95 10.7% max 13.1%; processor 10 mean 30.8% p95 89.1% max 100%; processor 11 mean 18.1% p95 38.8% max 59.1%; processor 12 mean 15.4% p95 35.9% max 56.4%; processor 13 mean 63.7% p95 100% max 100%; total above 10% in 10 and above 25% in 0 samples; pinned processors summing above 150% in 3

## Published reference

- Dispatch + view, Glowcap replay: median 51.6 µs, p95 59.1 µs, max 646.7 µs (experiments/glowcap/evidence/round6-replay.json timing.dispatchAndShippedOutput); runtime 8d8596a, reactive WebAssembly `d897787f414a`, repository e6ace96.
- Resume ten minutes of play: median 3.2416 ms of runs 7.154, 3.2416, 3.1401 ms; save 2045 bytes (experiments/glowcap/evidence/round6-replay.json timing.resume (caveat5)).

## Final-state consistency

Within each target, every run of every mode and engine ended each workload in the same saved state (84 runs compared).

Between targets, by workload and kind of state (targets in one group ended in identical states):

- glowcap-replay adapter-save-json: rc4-published, main-local `bcc2ec60c6f5`
- glowcap-replay save-text: rc4-published, main-local `fda3d6109816`
- ledger-session save-text: rc4-published, main-local `895698524f68`
- trail-rescue-scenarios save-text: rc4-published, main-local `bb72a4e76921`

## One Glowcap event + view, piece by piece

Medians over runs of each run's median on the steady idle ticks (events 1300–9999), where the pooled median sits; each row is its own process, so parts need not sum exactly to the whole.

| Piece | rc4-published | main-local |
| --- | --- | --- |
| Published method: adapter dispatch + view | 45.9 | 45.1 |
| Adapter dispatch (stringify, WebAssembly dispatch_view, JSON.parse) | 42.6 | 42.3 |
| Adapter view() (JavaScript reshaping only) | 2.2 | 2.2 |
| JSON.stringify(payload) | 0.3 | 0.3 |
| wasm-bindgen dispatch_view call, returning the view text | 25.5 | 25.2 |
|   arguments copied into WebAssembly memory | - | - |
|   WebAssembly execution (resolve, apply, view, serialize) | - | - |
|   result decoded to a JavaScript string | - | - |
|   result freed | - | - |
|   the four pieces, timed together in one process | - | - |
| JSON.parse(view text) | 16.8 | 16.8 |
| WebAssembly view() alone, execution only | - | - |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | 141.8 | 142.9 |
| Kit dispatch + view() | 171.6 | 172.7 |
| Native web::dispatch_view (the exported function, natively) | - | - |
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

| Engine | Mode | Operation | rc4-published | main-local |
| --- | --- | --- | --- | --- |
| wasm | published-method | adapter.dispatch+view | 46.1 (44.3–46.3) · p95 57.1 | 45.3 (44.5–46.5) · p95 57.0 |
| wasm | adapter | adapter.dispatch | 42.6 (42.5–43.6) · p95 51.2 | 42.3 (41.9–43.5) · p95 50.5 |
| wasm | adapter | adapter.view | 2.2 (2.2–2.3) · p95 3.5 | 2.2 (2.2–2.3) · p95 3.3 |
| wasm | adapter | adapter.dispatch+view | 45.0 (44.9–46.0) · p95 54.2 | 44.6 (44.2–45.9) · p95 53.4 |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.5 | 0.3 (0.3–0.3) · p95 0.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 25.6 (25.5–26.0) · p95 30.4 | 25.3 (24.8–26.2) · p95 30.0 |
| wasm | raw.dispatch_view | js.parse_view | 16.7 (16.5–17.1) · p95 19.5 | 16.8 (16.2–17.0) · p95 19.1 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 42.8 (42.4–43.5) · p95 49.9 | 42.6 (41.4–43.6) · p95 48.9 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 74.6 (73.2–75.0) · p95 85.7 | 73.2 (72.9–74.8) · p95 86.2 |
| wasm | raw.dispatch_outcome | js.parse_outcome | 65.8 (65.1–65.9) · p95 72.0 | 65.7 (64.4–66.1) · p95 73.3 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 140.1 (139.6–141.3) · p95 155.0 | 139.7 (137.7–140.9) · p95 158.1 |
| wasm | kit | kit.dispatch | 141.4 (140.4–143.6) · p95 160.2 | 141.8 (141.7–143.5) · p95 160.8 |
| wasm | kit | kit.view | 29.5 (29.1–29.6) · p95 33.4 | 29.6 (29.4–29.9) · p95 33.6 |
| wasm | kit | kit.dispatch+view | 171.0 (169.6–173.5) · p95 193.0 | 171.6 (171.4–173.7) · p95 194.5 |
| wasm | kit.read | kit.view | 32.1 (30.9–32.5) · p95 35.4 | 32.0 (31.1–32.1) · p95 35.1 |
| wasm | kit.read | kit.snapshot | 176.2 (169.3–176.6) · p95 196.4 | 175.0 (170.6–176.1) · p95 198.0 |
| wasm | kit.read | kit.save | 42.2 (40.4–42.9) · p95 49.1 | 42.0 (40.8–42.1) · p95 49.1 |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| wasm | published-method | adapter.dispatch+view | rc4-published | 180.6 (158.5–222.1) | 54.3 (50.7–55.1) | 46.2 (45.0–46.2) | 45.9 (43.9–46.1) |
| wasm | published-method | adapter.dispatch+view | main-local | 175.4 (169.4–197.4) | 54.2 (52.1–55.3) | 45.5 (45.2–47.3) | 45.1 (44.3–46.3) |
| wasm | adapter | adapter.dispatch | rc4-published | 161.6 (161.1–163.9) | 46.9 (46.5–47.3) | 41.6 (40.1–43.0) | 42.6 (42.4–43.6) |
| wasm | adapter | adapter.dispatch | main-local | 169.1 (142.5–176.6) | 45.5 (44.0–46.6) | 42.4 (41.2–43.5) | 42.3 (41.8–43.4) |
| wasm | adapter | adapter.view | rc4-published | 28.3 (27.5–44.1) | 2.7 (2.6–2.8) | 2.3 (2.2–2.4) | 2.2 (2.2–2.3) |
| wasm | adapter | adapter.view | main-local | 32.1 (25.0–34.2) | 2.6 (2.5–2.6) | 2.3 (2.3–2.4) | 2.2 (2.2–2.2) |
| wasm | adapter | adapter.dispatch+view | rc4-published | 190.9 (186.4–226.9) | 50.0 (49.3–50.4) | 44.0 (42.4–45.5) | 45.0 (44.7–45.9) |
| wasm | adapter | adapter.dispatch+view | main-local | 182.8 (181.2–210.7) | 48.3 (46.7–49.5) | 44.8 (43.6–45.9) | 44.5 (44.1–45.7) |
| wasm | raw.dispatch_view | js.stringify_payload | rc4-published | 0.9 (0.7–1.5) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | main-local | 1.1 (0.8–1.4) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | rc4-published | 129.5 (127.6–143.6) | 29.1 (27.0–29.1) | 25.4 (24.8–26.6) | 25.5 (25.4–25.9) |
| wasm | raw.dispatch_view | raw.dispatch_view | main-local | 136.2 (134.1–152.3) | 29.0 (28.3–29.4) | 25.9 (25.4–26.3) | 25.2 (24.7–26.1) |
| wasm | raw.dispatch_view | js.parse_view | rc4-published | 29.8 (29.3–33.8) | 16.5 (15.1–16.7) | 15.9 (15.5–16.8) | 16.8 (16.7–17.1) |
| wasm | raw.dispatch_view | js.parse_view | main-local | 30.4 (28.2–32.3) | 16.3 (15.9–16.9) | 16.3 (15.7–16.7) | 16.8 (16.2–17.1) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | rc4-published | 177.1 (158.3–189.7) | 46.1 (42.5–46.3) | 41.7 (40.7–43.8) | 42.7 (42.6–43.4) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main-local | 173.8 (167.7–200.6) | 45.8 (44.4–46.8) | 43.0 (41.5–43.1) | 42.4 (41.3–43.6) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | rc4-published | 190.3 (186.5–203.6) | 71.7 (68.5–73.1) | 70.7 (69.2–72.1) | 74.8 (73.3–75.6) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | main-local | 201.8 (180.3–208.9) | 71.3 (71.3–71.8) | 70.5 (69.4–70.8) | 73.3 (73.2–75.3) |
| wasm | raw.dispatch_outcome | js.parse_outcome | rc4-published | 77.7 (74.4–81.7) | 60.1 (56.6–60.3) | 62.7 (60.5–63.6) | 66.2 (65.4–66.3) |
| wasm | raw.dispatch_outcome | js.parse_outcome | main-local | 77.7 (75.8–151.9) | 59.6 (59.4–60.8) | 61.6 (61.2–63.3) | 66.2 (65.0–66.4) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | rc4-published | 266.1 (264.8–280.6) | 132.4 (125.4–133.4) | 134.7 (130.3–135.4) | 140.6 (140.0–142.5) |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | main-local | 277.5 (260.8–360.8) | 131.4 (131.2–133.0) | 132.8 (131.1–134.4) | 140.1 (138.4–141.9) |
| wasm | kit | kit.dispatch | rc4-published | 279.5 (272.8–337.0) | 135.8 (131.7–138.3) | 136.2 (132.5–138.5) | 141.8 (141.4–144.1) |
| wasm | kit | kit.dispatch | main-local | 268.3 (259.9–288.9) | 135.6 (134.6–137.2) | 137.3 (135.8–137.9) | 142.9 (142.1–144.0) |
| wasm | kit | kit.view | rc4-published | 36.1 (35.4–38.4) | 27.8 (27.1–28.1) | 28.5 (27.6–28.6) | 29.6 (29.3–29.8) |
| wasm | kit | kit.view | main-local | 41.4 (40.8–41.8) | 27.9 (27.9–28.3) | 28.6 (28.4–28.8) | 29.7 (29.7–30.0) |
| wasm | kit | kit.dispatch+view | rc4-published | 331.1 (312.0–372.4) | 164.0 (159.2–166.8) | 164.8 (160.3–167.3) | 171.6 (170.8–174.1) |
| wasm | kit | kit.dispatch+view | main-local | 308.9 (295.9–338.8) | 163.9 (162.8–165.7) | 166.5 (164.4–166.9) | 172.7 (172.1–174.3) |
| wasm | kit.read | kit.view | rc4-published | 44.7 (40.0–45.5) | 29.9 (28.6–30.3) | 30.6 (29.7–30.8) | 32.2 (31.0–32.7) |
| wasm | kit.read | kit.view | main-local | 41.9 (40.9–44.2) | 29.7 (28.8–29.9) | 30.5 (30.0–30.5) | 32.1 (31.2–32.3) |
| wasm | kit.read | kit.snapshot | rc4-published | 172.9 (167.2–179.4) | 160.9 (152.6–162.0) | 163.8 (159.8–166.0) | 177.1 (170.1–177.4) |
| wasm | kit.read | kit.snapshot | main-local | 176.3 (165.1–178.9) | 160.1 (154.8–160.1) | 163.6 (160.7–164.3) | 175.9 (171.6–177.0) |
| wasm | kit.read | kit.save | rc4-published | 41.8 (40.5–41.9) | 32.4 (31.1–32.8) | 39.0 (37.0–39.3) | 42.4 (40.7–43.1) |
| wasm | kit.read | kit.save | main-local | 42.8 (40.0–45.7) | 32.4 (31.3–32.6) | 38.3 (37.8–38.7) | 42.2 (41.1–42.4) |

## ledger-session

The agent ledger replaying the real 2026-09-24 session log: 20 events, five of them refused by policy. Played 100 times from a fresh session. Program `experiments/agent-ledger/ledger.cav` (`2206ac79f23c`), stream `ef2143311f86`, 1 episode(s) × 100 = 2000 events per pass.

| Engine | Mode | Operation | rc4-published | main-local |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.3) · p95 1.0 | 0.3 (0.3–0.3) · p95 1.0 |
| wasm | raw.dispatch_view | raw.dispatch_view | 30.4 (29.8–30.5) · p95 39.4 | 29.9 (29.1–30.3) · p95 39.0 |
| wasm | raw.dispatch_view | js.parse_view | 7.2 (7.1–7.4) · p95 10.5 | 7.2 (7.0–7.3) · p95 10.2 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 40.5 (39.4–41.1) · p95 50.5 | 40.4 (39.2–40.6) · p95 49.0 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 66.9 (66.5–67.4) · p95 83.1 | 66.8 (66.4–68.3) · p95 85.1 |
| wasm | raw.dispatch_outcome | js.parse_outcome | 47.7 (46.9–47.8) · p95 56.8 | 47.4 (47.3–48.6) · p95 58.9 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 115.1 (113.4–115.8) · p95 138.9 | 114.4 (114.3–117.4) · p95 142.6 |
| wasm | kit | kit.dispatch | 116.8 (115.2–118.0) · p95 145.6 | 117.4 (116.9–118.3) · p95 143.3 |
| wasm | kit | kit.view | 15.2 (14.9–15.2) · p95 18.9 | 15.3 (15.2–15.5) · p95 19.0 |
| wasm | kit | kit.dispatch+view | 131.4 (129.7–132.9) · p95 164.0 | 132.3 (131.6–133.5) · p95 161.6 |
| wasm | kit.read | kit.view | 16.7 (16.2–17.1) · p95 20.4 | 16.5 (16.4–16.9) · p95 20.9 |
| wasm | kit.read | kit.snapshot | 122.1 (117.8–123.7) · p95 142.4 | 121.4 (119.6–123.1) · p95 145.5 |
| wasm | kit.read | kit.save | 32.2 (31.3–32.7) · p95 41.8 | 32.0 (31.3–32.4) · p95 42.6 |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | rc4-published | main-local |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.6 (0.5–0.6) · p95 1.8 | 0.6 (0.6–0.6) · p95 1.7 |
| wasm | raw.dispatch_view | raw.dispatch_view | 75.6 (73.9–76.5) · p95 152.3 | 75.0 (73.9–75.2) · p95 154.6 |
| wasm | raw.dispatch_view | js.parse_view | 26.7 (26.0–26.9) · p95 31.8 | 26.8 (26.0–26.8) · p95 32.5 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 125.0 (123.8–128.2) · p95 183.1 | 127.1 (122.4–127.1) · p95 186.3 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome | 145.3 (145.0–160.0) · p95 230.7 | 145.7 (145.6–166.7) · p95 231.7 |
| wasm | raw.dispatch_outcome | js.parse_outcome | 78.9 (78.5–84.2) · p95 89.7 | 78.8 (78.5–88.9) · p95 89.8 |
| wasm | raw.dispatch_outcome | raw.dispatch_outcome+parse | 226.6 (226.5–247.5) · p95 315.1 | 227.3 (226.1–257.0) · p95 317.5 |
| wasm | kit | kit.dispatch | 230.2 (227.1–230.4) · p95 321.4 | 229.8 (228.5–230.1) · p95 318.7 |
| wasm | kit | kit.view | 35.4 (35.0–35.6) · p95 45.8 | 35.3 (35.3–35.7) · p95 45.0 |
| wasm | kit | kit.dispatch+view | 268.3 (265.7–269.6) · p95 365.9 | 268.3 (267.3–269.2) · p95 361.8 |
| wasm | kit.read | kit.view | 35.9 (35.6–36.8) · p95 45.9 | 35.8 (35.7–36.2) · p95 45.4 |
| wasm | kit.read | kit.snapshot | 186.2 (185.6–191.9) · p95 228.3 | 186.6 (186.2–189.0) · p95 223.0 |
| wasm | kit.read | kit.save | 28.2 (28.1–28.7) · p95 47.5 | 28.2 (27.9–28.7) · p95 46.8 |

