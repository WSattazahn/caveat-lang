# Caveat performance baseline: 2026-09-29T20-06-57-466-view-path

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=view-path --engines=wasm --build=never --keep-samples --repeats=5 --monitor-interval=1 --monitor-lead=5 --job-guard --affinity=0x3C00 --priority=high --target=opt=tree:. --target=main=tree:../prOPT-main --out=C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prOPT-measure2\runs\G3-3C00`

## Method

- Suite `view-path`: 5 repeat(s) of every mode, each in a fresh process; within a repeat every target runs a mode back to back, the first target rotating.
- Per-event modes: 1 untimed warm-up pass(es), then 3 timed pass(es), each episode on a freshly opened session. `published-method` runs 3 round(s) with no warm-up, as the published harness did.
- Load, save and restore: 3 untimed then 30 timed calls; published resume method: 3 run(s).
- Statistic: quantile q(p) = sorted[floor(p·(n−1))] over the pooled samples of a run (the published rule; the median is the lower median). A cell is the median over runs of each run's pooled median, with the lowest and highest run in brackets, then the median over runs of each run's p95.
- Units: microseconds per operation unless the operation says otherwise.

## Targets

| Target | Kind | Revision | Reactive WebAssembly sha256 | Glue sha256 | Kit session.mjs sha256 | Native benchmark |
| --- | --- | --- | --- | --- | --- | --- |
| opt | tree | v0.1.0-rc.4-15-gd5938cc-dirty (d5938cc), tracked changes | `0a19e4b85886` (2057234 B, reused) | `3dcc4fd6ba4e` | `f40dad99c758` | - |
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
- Load before: total CPU 10/4/0%; busiest over the window: powershell 0.594s, Reallusion Hub 0.094s, claude 0.062s, QuickPanelOSD 0.031s, claude 0.016s, bash 0.016s
- Load after: total CPU 0/1/14%; busiest over the window: powershell 0.484s, Reallusion Hub 0.156s, IntelGraphicsSoftware 0.078s, claude 0.047s, claude 0.047s, claude 0.031s
- Load during: typeperf every 1 s for the whole run, 541 samples: total mean 5.3% p95 8.8% max 13.5%; processor 10 mean 56% p95 100% max 100%; processor 11 mean 9.9% p95 38.9% max 100%; processor 12 mean 9.8% p95 37.4% max 100%; processor 13 mean 18.3% p95 90.8% max 100%; processor 10 MHz mean 4410.9 MHz p95 4705 MHz max 4823.7 MHz; processor 11 MHz mean 4049.2 MHz p95 4688.3 MHz max 4885.5 MHz; processor 12 MHz mean 4175.1 MHz p95 4678.2 MHz max 5035.4 MHz; processor 13 MHz mean 4006.6 MHz p95 4833.5 MHz max 5253.4 MHz; process System mean 8.1% p95 30.8% max 108.5%; process MsMpEng mean 1.5% p95 6.1% max 32.1%; total above 10% in 9 and above 25% in 0 samples; pinned processors summing above 150% in 5

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
| wasm-bindgen dispatch_view call, returning the view text | 22.3 | 22.2 |
|   arguments copied into WebAssembly memory | - | - |
|   WebAssembly execution (resolve, apply, view, serialize) | - | - |
|   result decoded to a JavaScript string | - | - |
|   result freed | - | - |
|   the four pieces, timed together in one process | - | - |
| JSON.parse(view text) | 14.7 | 14.6 |
| WebAssembly view() alone, execution only | - | - |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | 124.5 | 123.3 |
| Kit dispatch + view() | 150.7 | 149.2 |
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
| wasm | raw.dispatch_view | js.stringify_payload | 0.2 (0.2–0.2) · p95 0.4 | 0.3 (0.2–0.3) · p95 0.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 22.4 (22.1–22.7) · p95 25.8 | 22.3 (22.2–23.3) · p95 25.6 |
| wasm | raw.dispatch_view | js.parse_view | 14.7 (14.5–15.0) · p95 16.5 | 14.5 (14.4–15.1) · p95 16.4 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 37.4 (36.9–38.0) · p95 42.1 | 37.1 (37.0–38.6) · p95 42.0 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.3 (0.2–0.3) · p95 0.4 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 22.7 (22.2–22.8) · p95 26.1 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 14.8 (14.6–14.8) · p95 16.9 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 37.8 (37.1–37.9) · p95 43.2 | - |
| wasm | kit | kit.dispatch | 124.2 (122.1–126.2) · p95 144.9 | 122.8 (121.4–126.5) · p95 139.3 |
| wasm | kit | kit.view | 25.9 (25.5–26.4) · p95 29.9 | 25.8 (25.5–26.5) · p95 29.0 |
| wasm | kit | kit.dispatch+view | 150.3 (147.8–152.7) · p95 174.9 | 148.6 (147.1–153.2) · p95 168.3 |
| wasm | kit.dispatchView | kit.dispatchView | 38.0 (37.3–38.7) · p95 41.8 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 0.5 (0.4–0.5) · p95 0.7 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 23.6 (22.5–24.3) · p95 27.3 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 15.0 (14.6–15.5) · p95 17.5 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 39.2 (37.8–40.5) · p95 45.3 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 14.2 (13.8–14.8) · p95 16.5 | - |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | opt | 0.8 (0.8–1.5) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.3 (0.2–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | main | 0.9 (0.7–1.7) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.3 (0.2–0.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | opt | 116.1 (109.9–120.2) | 24.9 (24.3–25.1) | 22.6 (22.2–23.1) | 22.3 (22.0–22.6) |
| wasm | raw.dispatch_view | raw.dispatch_view | main | 107.7 (104.6–116.9) | 25.0 (24.5–25.8) | 22.4 (22.3–23.4) | 22.2 (22.2–23.2) |
| wasm | raw.dispatch_view | js.parse_view | opt | 24.7 (23.7–25.3) | 14.1 (13.8–14.4) | 14.2 (14.1–14.7) | 14.7 (14.6–15.0) |
| wasm | raw.dispatch_view | js.parse_view | main | 24.4 (24.2–25.7) | 14.0 (13.9–14.5) | 14.0 (14.0–14.6) | 14.6 (14.5–15.2) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | opt | 146.6 (134.3–162.4) | 39.4 (38.4–39.7) | 37.1 (36.5–38.1) | 37.3 (36.9–37.9) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main | 142.3 (129.9–143.2) | 39.3 (38.7–40.5) | 36.7 (36.6–38.2) | 37.0 (37.0–38.7) |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | opt | 0.9 (0.7–1.4) | 0.2 (0.2–0.3) | 0.2 (0.2–0.2) | 0.3 (0.2–0.3) |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | opt | 114.5 (104.4–125.5) | 25.1 (23.9–25.5) | 22.8 (22.4–23.6) | 22.6 (22.1–22.7) |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | opt | 24.9 (23.4–26.1) | 14.2 (13.8–14.3) | 14.2 (14.2–14.8) | 14.8 (14.6–14.9) |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | opt | 146.9 (127.4–177.2) | 39.6 (38.0–40.1) | 37.4 (36.9–38.7) | 37.8 (37.0–37.8) |
| wasm | kit | kit.dispatch | opt | 254.5 (245.2–271.4) | 117.9 (117.1–122.3) | 118.9 (118.1–121.1) | 124.5 (122.2–127.2) |
| wasm | kit | kit.dispatch | main | 234.1 (229.6–274.0) | 117.6 (114.5–119.2) | 118.5 (116.1–120.3) | 123.3 (121.9–127.4) |
| wasm | kit | kit.view | opt | 34.2 (33.0–34.9) | 24.4 (24.2–25.3) | 24.8 (24.7–25.1) | 26.0 (25.6–26.6) |
| wasm | kit | kit.view | main | 35.0 (32.1–38.8) | 24.3 (23.5–24.5) | 24.9 (24.4–25.3) | 25.9 (25.6–26.7) |
| wasm | kit | kit.dispatch+view | opt | 291.9 (281.4–308.0) | 142.4 (141.6–148.0) | 144.0 (143.1–146.6) | 150.7 (148.0–153.8) |
| wasm | kit | kit.dispatch+view | main | 274.8 (268.9–303.5) | 141.9 (138.2–143.9) | 143.6 (140.6–146.3) | 149.2 (147.7–154.3) |
| wasm | kit.dispatchView | kit.dispatchView | opt | 167.5 (160.5–188.2) | 39.8 (39.2–40.6) | 37.6 (37.0–38.1) | 38.0 (37.3–38.7) |
| wasm | kit.dispatchView.pieces | kit.payloadText | opt | 2.5 (2.3–2.6) | 0.5 (0.5–0.5) | 0.4 (0.4–0.5) | 0.5 (0.4–0.5) |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | opt | 119.8 (114.9–129.7) | 25.4 (24.6–27.2) | 23.5 (22.8–24.5) | 23.5 (22.4–24.2) |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | opt | 25.0 (24.1–30.9) | 14.0 (14.0–15.0) | 14.5 (14.2–15.0) | 15.1 (14.7–15.6) |
| wasm | kit.dispatchView.pieces | kit.validOutcome | opt | 2.0 (1.2–2.2) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | opt | 170.2 (155.1–184.7) | 40.2 (39.2–42.8) | 38.8 (37.6–40.2) | 39.2 (37.8–40.4) |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | opt | 14.8 (14.1–15.4) | 13.3 (13.2–14.2) | 13.8 (13.5–14.2) | 14.3 (13.9–14.9) |

## ledger-session

The agent ledger replaying the real 2026-09-24 session log: 20 events, five of them refused by policy. Played 100 times from a fresh session. Program `experiments/agent-ledger/ledger.cav` (`2206ac79f23c`), stream `ef2143311f86`, 1 episode(s) × 100 = 2000 events per pass.

| Engine | Mode | Operation | opt | main |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.8 | 0.3 (0.3–0.3) · p95 0.9 |
| wasm | raw.dispatch_view | raw.dispatch_view | 26.6 (26.0–27.1) · p95 34.3 | 26.5 (26.2–27.6) · p95 34.6 |
| wasm | raw.dispatch_view | js.parse_view | 6.4 (6.2–6.5) · p95 9.0 | 6.4 (6.1–6.6) · p95 9.0 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 35.5 (34.7–36.2) · p95 43.2 | 35.6 (34.8–37.1) · p95 43.7 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.9 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 25.9 (25.5–27.4) · p95 33.9 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 6.3 (6.2–6.3) · p95 8.5 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 32.6 (32.1–34.1) · p95 42.3 | - |
| wasm | kit | kit.dispatch | 100.5 (99.4–102.1) · p95 121.2 | 100.9 (100.1–102.7) · p95 121.8 |
| wasm | kit | kit.view | 13.1 (12.9–13.4) · p95 16.1 | 13.1 (12.9–13.3) · p95 16.1 |
| wasm | kit | kit.dispatch+view | 113.4 (112.0–115.2) · p95 136.6 | 113.5 (112.7–115.9) · p95 137.3 |
| wasm | kit.dispatchView | kit.dispatchView | 33.0 (32.6–33.4) · p95 42.5 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 0.9 (0.9–0.9) · p95 2.3 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 25.9 (25.0–26.4) · p95 33.5 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 6.0 (5.9–6.1) · p95 8.5 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 33.2 (32.3–33.5) · p95 42.9 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 5.1 (5.0–5.1) · p95 6.3 | - |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | opt | main |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.5 (0.4–0.5) · p95 1.4 | 0.4 (0.4–0.5) · p95 1.4 |
| wasm | raw.dispatch_view | raw.dispatch_view | 66.2 (63.2–66.4) · p95 135.5 | 64.9 (63.3–68.4) · p95 134.4 |
| wasm | raw.dispatch_view | js.parse_view | 23.1 (22.8–23.3) · p95 27.7 | 23.3 (22.7–23.5) · p95 27.8 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 110.4 (108.3–111.1) · p95 163.0 | 109.2 (108.9–113.0) · p95 161.9 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.5 (0.4–0.5) · p95 1.4 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 66.4 (64.5–66.9) · p95 136.9 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 22.4 (22.1–22.8) · p95 27.5 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 92.1 (89.9–92.7) · p95 161.9 | - |
| wasm | kit | kit.dispatch | 200.4 (197.0–201.1) · p95 277.1 | 197.8 (195.5–201.9) · p95 273.7 |
| wasm | kit | kit.view | 30.8 (30.3–31.1) · p95 39.3 | 30.3 (30.1–31.0) · p95 38.6 |
| wasm | kit | kit.dispatch+view | 234.2 (230.7–235.1) · p95 314.3 | 231.1 (229.3–236.0) · p95 309.8 |
| wasm | kit.dispatchView | kit.dispatchView | 95.6 (93.2–97.1) · p95 168.2 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 1.2 (1.2–1.3) · p95 3.7 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 66.1 (65.8–67.3) · p95 136.6 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 20.9 (20.8–21.0) · p95 26.6 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.3 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 91.4 (90.7–92.0) · p95 160.9 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 17.9 (17.8–18.2) · p95 21.7 | - |

