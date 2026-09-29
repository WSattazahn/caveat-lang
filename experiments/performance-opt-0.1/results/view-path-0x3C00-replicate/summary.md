# Caveat performance baseline: 2026-09-29T18-47-25-866-view-path

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=view-path --engines=wasm --build=never --keep-samples --repeats=5 --monitor-interval=1 --affinity=0x3C00 --priority=high --target=opt=tree:. --target=main=tree:../prOPT-main --out=C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prOPT-measure\runs\R1-3C00`

## Method

- Suite `view-path`: 5 repeat(s) of every mode, each in a fresh process; within a repeat every target runs a mode back to back, the first target rotating.
- Per-event modes: 1 untimed warm-up pass(es), then 3 timed pass(es), each episode on a freshly opened session. `published-method` runs 3 round(s) with no warm-up, as the published harness did.
- Load, save and restore: 3 untimed then 30 timed calls; published resume method: 3 run(s).
- Statistic: quantile q(p) = sorted[floor(p·(n−1))] over the pooled samples of a run (the published rule; the median is the lower median). A cell is the median over runs of each run's pooled median, with the lowest and highest run in brackets, then the median over runs of each run's p95.
- Units: microseconds per operation unless the operation says otherwise.

## Targets

| Target | Kind | Revision | Reactive WebAssembly sha256 | Glue sha256 | Kit session.mjs sha256 | Native benchmark |
| --- | --- | --- | --- | --- | --- | --- |
| opt | tree | v0.1.0-rc.4-14-ga459995-dirty (a459995), tracked changes | `0a19e4b85886` (2057234 B, reused) | `3dcc4fd6ba4e` | `f40dad99c758` | - |
| main | tree | v0.1.0-rc.4-12-g768275b (768275b) | `e35944503272` (2052856 B, reused) | `9066c9a19dcb` | `b171d128027a` | - |

## Environment

- CPU: Intel(R) Core(TM) Ultra 9 275HX; 24 cores, 24 logical processors; performance cores 0, 1, 10, 11, 12, 13, 22, 23
- Memory: 63.4 GiB
- OS: Microsoft Windows 11 Home 10.0.26200 (build 26200)
- Power: Power Scheme GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c  (High performance); battery status 2 (2 = on mains power)
- Pinning: affinity 0x3C00 (logical processors 10, 11, 12, 13), priority high, via cmd /c start /b /wait /<priority> /affinity <mask>
- Node v24.11.1, V8 13.6.233.10-node.28, npm 11.12.1
- Rust: rustc 1.98.1 (48a229cea 2026-09-01) (LLVM version: 22.1.8); cargo 1.98.1 (797e8a9bc 2026-08-05); 1.98.1-x86_64-pc-windows-msvc (overridden by 'C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prOPT\rust-toolchain.toml')
- wasm-bindgen 0.2.104; wasm-opt not installed (the build does not use it)
- Load before: total CPU 8/9/6%; busiest over the window: claude 0.875s, claude 0.609s, powershell 0.562s, claude 0.484s, uihost 0.125s, Reallusion Hub 0.109s
- Load after: total CPU 0/16/21%; busiest over the window: powershell 0.469s, claude 0.453s, claude 0.156s, claude 0.141s, Reallusion Hub 0.125s, IntelGraphicsSoftware 0.062s
- Load during: typeperf every 1 s for the whole run, 393 samples: total mean 7.1% p95 10.9% max 17.4%; processor 10 mean 78.6% p95 100% max 100%; processor 11 mean 21.7% p95 61.2% max 100%; processor 12 mean 21.6% p95 66% max 100%; processor 13 mean 6.2% p95 23.7% max 65.8%; processor 10 MHz mean 4385.7 MHz p95 4507.1 MHz max 4653.1 MHz; processor 11 MHz mean 4236 MHz p95 4560.2 MHz max 5114.6 MHz; processor 12 MHz mean 4205.6 MHz p95 4523.5 MHz max 4918.4 MHz; processor 13 MHz mean 3654.1 MHz p95 4693.2 MHz max 4803.7 MHz; total above 10% in 32 and above 25% in 0 samples; pinned processors summing above 150% in 56

## Published reference

- Dispatch + view, Glowcap replay: median 51.6 µs, p95 59.1 µs, max 646.7 µs (experiments/glowcap/evidence/round6-replay.json timing.dispatchAndShippedOutput); runtime 8d8596a, reactive WebAssembly `d897787f414a`, repository e6ace96.
- Resume ten minutes of play: median 3.2416 ms of runs 7.154, 3.2416, 3.1401 ms; save 2045 bytes (experiments/glowcap/evidence/round6-replay.json timing.resume (caveat5)).

## Final-state consistency

Within each target, every run of every mode and engine ended each workload in the same saved state (105 runs compared).

Between targets, by workload and kind of state (targets in one group ended in identical states):

- glowcap-replay save-text: opt, main `fda3d6109816`
- ledger-session save-text: opt, main `895698524f68`
- trail-rescue-scenarios save-text: opt, main `bb72a4e76921`

Skipped jobs (the target cannot run them):
- 0004-main-glowcap-replay-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0008-main-glowcap-replay-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0010-main-glowcap-replay-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0014-main-ledger-session-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0018-main-ledger-session-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0020-main-ledger-session-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0024-main-trail-rescue-scenarios-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0028-main-trail-rescue-scenarios-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0030-main-trail-rescue-scenarios-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0033-main-glowcap-replay-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0037-main-glowcap-replay-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0039-main-glowcap-replay-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0043-main-ledger-session-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0047-main-ledger-session-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0049-main-ledger-session-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0053-main-trail-rescue-scenarios-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0057-main-trail-rescue-scenarios-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0059-main-trail-rescue-scenarios-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0064-main-glowcap-replay-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0068-main-glowcap-replay-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0070-main-glowcap-replay-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0074-main-ledger-session-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0078-main-ledger-session-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0080-main-ledger-session-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0084-main-trail-rescue-scenarios-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0088-main-trail-rescue-scenarios-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0090-main-trail-rescue-scenarios-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0093-main-glowcap-replay-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0097-main-glowcap-replay-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0099-main-glowcap-replay-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0103-main-ledger-session-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0107-main-ledger-session-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0109-main-ledger-session-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0113-main-trail-rescue-scenarios-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0117-main-trail-rescue-scenarios-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0119-main-trail-rescue-scenarios-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0124-main-glowcap-replay-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0128-main-glowcap-replay-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0130-main-glowcap-replay-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0134-main-ledger-session-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0138-main-ledger-session-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0140-main-ledger-session-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome
- 0144-main-trail-rescue-scenarios-wasm-raw.dispatch_view_outcome: this runtime has no dispatch_view_outcome
- 0148-main-trail-rescue-scenarios-wasm-kit.dispatchView: this kit has no session.dispatchView
- 0150-main-trail-rescue-scenarios-wasm-kit.dispatchView.pieces: this runtime has no dispatch_view_outcome

## One Glowcap event + view, piece by piece

Medians over runs of each run's median on the steady idle ticks (events 1300–9999), where the pooled median sits; each row is its own process, so parts need not sum exactly to the whole.

| Piece | opt | main |
| --- | --- | --- |
| Published method: adapter dispatch + view | - | - |
| Adapter dispatch (stringify, WebAssembly dispatch_view, JSON.parse) | - | - |
| Adapter view() (JavaScript reshaping only) | - | - |
| JSON.stringify(payload) | 0.3 | 0.3 |
| wasm-bindgen dispatch_view call, returning the view text | 23.5 | 23.2 |
|   arguments copied into WebAssembly memory | - | - |
|   WebAssembly execution (resolve, apply, view, serialize) | - | - |
|   result decoded to a JavaScript string | - | - |
|   result freed | - | - |
|   the four pieces, timed together in one process | - | - |
| JSON.parse(view text) | 15.4 | 15.5 |
| WebAssembly view() alone, execution only | - | - |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | 128.6 | 126.8 |
| Kit dispatch + view() | 155.5 | 153.5 |
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

| Engine | Mode | Operation | opt | main |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.2–0.3) · p95 0.5 | 0.3 (0.2–0.3) · p95 0.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 23.5 (23.2–27.7) · p95 27.4 | 23.4 (23.0–24.7) · p95 28.0 |
| wasm | raw.dispatch_view | js.parse_view | 15.3 (15.2–18.0) · p95 17.8 | 15.5 (14.9–16.0) · p95 18.0 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 39.0 (38.8–46.1) · p95 45.4 | 39.2 (38.3–41.1) · p95 46.4 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.3 (0.2–0.3) · p95 0.5 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 23.6 (22.9–25.2) · p95 28.9 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 15.3 (14.9–16.4) · p95 18.5 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 39.1 (38.2–42.1) · p95 47.7 | - |
| wasm | kit | kit.dispatch | 128.1 (124.9–132.7) · p95 150.2 | 126.4 (125.6–132.6) · p95 149.6 |
| wasm | kit | kit.view | 26.6 (26.0–27.5) · p95 31.4 | 26.4 (26.2–27.6) · p95 31.1 |
| wasm | kit | kit.dispatch+view | 154.9 (151.2–160.4) · p95 181.4 | 153.0 (151.9–160.3) · p95 180.8 |
| wasm | kit.dispatchView | kit.dispatchView | 39.2 (38.6–40.2) · p95 45.8 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 0.5 (0.4–0.5) · p95 0.7 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 23.6 (23.4–25.5) · p95 28.8 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 15.4 (15.1–16.5) · p95 18.2 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 39.6 (39.0–42.6) · p95 47.4 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 14.6 (14.4–15.7) · p95 17.4 | - |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | opt | 1.0 (0.7–1.4) | 0.3 (0.2–0.3) | 0.2 (0.2–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | main | 0.8 (0.8–1.0) | 0.3 (0.2–0.3) | 0.2 (0.2–0.3) | 0.3 (0.3–0.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | opt | 113.4 (106.9–142.3) | 25.4 (24.8–28.6) | 23.3 (23.0–27.6) | 23.5 (23.1–27.7) |
| wasm | raw.dispatch_view | raw.dispatch_view | main | 117.9 (115.9–119.7) | 26.2 (24.8–27.0) | 23.3 (23.0–24.1) | 23.2 (23.0–24.8) |
| wasm | raw.dispatch_view | js.parse_view | opt | 26.8 (25.1–31.4) | 14.5 (14.1–16.2) | 14.8 (14.3–17.2) | 15.4 (15.3–18.2) |
| wasm | raw.dispatch_view | js.parse_view | main | 26.1 (24.9–26.5) | 15.0 (14.1–15.3) | 14.5 (14.4–15.2) | 15.5 (15.0–16.1) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | opt | 147.0 (136.6–169.6) | 40.2 (39.2–45.2) | 38.5 (37.6–45.3) | 39.2 (38.7–46.3) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main | 146.5 (143.5–173.5) | 41.5 (39.2–42.9) | 38.1 (37.7–39.5) | 39.1 (38.3–41.4) |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | opt | 0.8 (0.8–1.2) | 0.3 (0.2–0.3) | 0.3 (0.2–0.3) | 0.3 (0.2–0.3) |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | opt | 112.1 (111.2–120.4) | 26.2 (25.6–28.3) | 23.6 (23.2–25.2) | 23.5 (22.8–25.0) |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | opt | 25.3 (24.4–32.5) | 14.8 (14.5–16.2) | 14.8 (14.5–15.8) | 15.4 (15.0–16.5) |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | opt | 134.6 (130.7–151.5) | 41.2 (40.3–44.8) | 38.6 (38.0–41.3) | 39.1 (38.1–42.0) |
| wasm | kit | kit.dispatch | opt | 252.6 (237.8–265.8) | 122.1 (121.4–126.2) | 122.7 (120.7–127.4) | 128.6 (125.1–134.1) |
| wasm | kit | kit.dispatch | main | 252.6 (232.0–295.7) | 121.3 (119.4–122.7) | 121.5 (120.0–128.2) | 126.8 (126.1–133.4) |
| wasm | kit | kit.view | opt | 34.7 (30.7–36.9) | 25.1 (25.0–26.0) | 25.6 (25.3–26.6) | 26.8 (26.1–28.0) |
| wasm | kit | kit.view | main | 34.7 (32.7–36.6) | 25.0 (24.5–25.6) | 25.3 (25.0–26.8) | 26.5 (26.3–27.7) |
| wasm | kit | kit.dispatch+view | opt | 291.5 (266.0–312.8) | 147.4 (146.8–152.7) | 148.5 (146.3–154.3) | 155.5 (151.3–162.3) |
| wasm | kit | kit.dispatch+view | main | 300.2 (267.0–330.0) | 146.7 (143.9–148.5) | 147.0 (145.4–155.5) | 153.5 (152.5–161.2) |
| wasm | kit.dispatchView | kit.dispatchView | opt | 160.9 (157.7–178.6) | 40.3 (39.2–41.7) | 38.4 (37.6–39.9) | 39.2 (38.5–40.1) |
| wasm | kit.dispatchView.pieces | kit.payloadText | opt | 2.3 (2.1–3.6) | 0.5 (0.5–0.5) | 0.5 (0.4–0.5) | 0.5 (0.4–0.5) |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | opt | 117.8 (110.5–123.8) | 25.5 (25.2–26.2) | 23.7 (22.9–24.7) | 23.5 (23.3–25.5) |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | opt | 25.2 (23.7–27.1) | 14.3 (14.1–14.9) | 14.8 (14.3–15.4) | 15.4 (15.1–16.6) |
| wasm | kit.dispatchView.pieces | kit.validOutcome | opt | 1.7 (1.6–1.9) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | opt | 173.2 (161.5–175.4) | 40.5 (39.9–41.6) | 39.1 (37.7–40.8) | 39.5 (39.0–42.7) |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | opt | 14.9 (14.5–16.4) | 13.6 (13.3–14.1) | 14.0 (13.6–14.6) | 14.7 (14.4–15.8) |

## ledger-session

The agent ledger replaying the real 2026-09-24 session log: 20 events, five of them refused by policy. Played 100 times from a fresh session. Program `experiments/agent-ledger/ledger.cav` (`2206ac79f23c`), stream `ef2143311f86`, 1 episode(s) × 100 = 2000 events per pass.

| Engine | Mode | Operation | opt | main |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.9 | 0.3 (0.3–0.3) · p95 0.9 |
| wasm | raw.dispatch_view | raw.dispatch_view | 27.1 (26.5–27.6) · p95 35.4 | 26.9 (26.5–27.7) · p95 34.7 |
| wasm | raw.dispatch_view | js.parse_view | 6.6 (6.3–6.7) · p95 9.3 | 6.5 (6.4–6.7) · p95 9.0 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 36.2 (35.1–37.3) · p95 44.6 | 35.9 (35.6–37.1) · p95 44.2 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.8 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 26.2 (26.0–26.6) · p95 34.7 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 6.3 (6.2–6.5) · p95 8.7 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 33.1 (32.6–33.5) · p95 43.3 | - |
| wasm | kit | kit.dispatch | 105.1 (103.1–105.6) · p95 127.6 | 105.1 (104.1–106.4) · p95 131.9 |
| wasm | kit | kit.view | 13.5 (13.4–13.9) · p95 16.8 | 13.7 (13.6–13.8) · p95 17.0 |
| wasm | kit | kit.dispatch+view | 118.3 (116.2–118.9) · p95 144.4 | 118.4 (117.4–119.6) · p95 148.5 |
| wasm | kit.dispatchView | kit.dispatchView | 33.8 (33.4–34.1) · p95 43.2 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 1.0 (0.9–1.0) · p95 2.4 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 26.4 (25.7–27.1) · p95 34.2 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 6.1 (5.9–6.3) · p95 8.7 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 34.2 (33.1–34.3) · p95 44.2 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 5.2 (5.1–5.4) · p95 6.6 | - |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | opt | main |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.5 (0.5–0.6) · p95 1.5 | 0.5 (0.5–0.5) · p95 1.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 67.3 (65.6–70.8) · p95 138.2 | 68.0 (67.4–70.0) · p95 139.3 |
| wasm | raw.dispatch_view | js.parse_view | 23.5 (23.2–24.3) · p95 28.3 | 23.9 (23.5–24.1) · p95 29.2 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 112.6 (111.0–116.8) · p95 166.3 | 113.5 (112.0–116.6) · p95 167.7 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.5 (0.5–0.5) · p95 1.5 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 68.7 (66.6–69.7) · p95 141.1 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 23.1 (22.9–23.6) · p95 29.4 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 95.1 (93.0–95.9) · p95 167.8 | - |
| wasm | kit | kit.dispatch | 207.1 (204.5–210.5) · p95 292.8 | 205.9 (204.6–208.5) · p95 290.3 |
| wasm | kit | kit.view | 31.8 (31.6–32.5) · p95 41.1 | 31.4 (31.2–31.9) · p95 41.0 |
| wasm | kit | kit.dispatch+view | 241.4 (239.7–246.2) · p95 331.7 | 240.8 (239.1–243.6) · p95 328.4 |
| wasm | kit.dispatchView | kit.dispatchView | 96.1 (95.0–99.6) · p95 169.6 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 1.3 (1.3–1.3) · p95 3.9 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 68.8 (67.0–69.4) · p95 139.0 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 21.6 (21.5–21.9) · p95 27.8 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.3 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 94.4 (92.8–95.3) · p95 164.0 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 18.5 (18.4–19.1) · p95 22.8 | - |

