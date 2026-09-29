# Caveat performance baseline: 2026-09-29T19-48-35-164-view-path

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=view-path --engines=wasm --build=never --keep-samples --repeats=5 --monitor-interval=1 --monitor-lead=5 --job-guard --affinity=0x3C00 --priority=high --target=opt=tree:. --target=main=tree:../prOPT-main --out=C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prOPT-measure2\runs\G1-3C00`

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
- Load before: total CPU 9/0/0%; busiest over the window: powershell 0.453s, claude 0.281s, Reallusion Hub 0.094s, claude 0.078s, IntelGraphicsSoftware 0.062s, OVRServer_x64 0.047s
- Load after: total CPU 0/0/18%; busiest over the window: powershell 0.453s, Reallusion Hub 0.172s, claude 0.094s, IntelGraphicsSoftware 0.078s, PredatorSense 0.047s, uihost 0.031s
- Load during: typeperf every 1 s for the whole run, 589 samples: total mean 5.4% p95 9% max 19.6%; processor 10 mean 52.9% p95 100% max 100%; processor 11 mean 11.3% p95 47.5% max 100%; processor 12 mean 10.9% p95 46.5% max 100%; processor 13 mean 16.8% p95 87.7% max 100%; processor 10 MHz mean 4388.6 MHz p95 4731.1 MHz max 4992.2 MHz; processor 11 MHz mean 4080 MHz p95 4735.4 MHz max 4976 MHz; processor 12 MHz mean 4131.4 MHz p95 4702 MHz max 5029.8 MHz; processor 13 MHz mean 3931.8 MHz p95 4818.5 MHz max 5091 MHz; process System mean 9.1% p95 29.1% max 108.5%; process MsMpEng mean 2.2% p95 9.2% max 40%; total above 10% in 14 and above 25% in 0 samples; pinned processors summing above 150% in 7

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
| wasm-bindgen dispatch_view call, returning the view text | 22.6 | 22.4 |
|   arguments copied into WebAssembly memory | - | - |
|   WebAssembly execution (resolve, apply, view, serialize) | - | - |
|   result decoded to a JavaScript string | - | - |
|   result freed | - | - |
|   the four pieces, timed together in one process | - | - |
| JSON.parse(view text) | 14.8 | 14.8 |
| WebAssembly view() alone, execution only | - | - |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | 123.6 | 124.6 |
| Kit dispatch + view() | 149.7 | 150.8 |
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
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.2–0.3) · p95 0.4 | 0.3 (0.2–0.3) · p95 0.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 22.7 (22.2–23.9) · p95 26.5 | 22.5 (22.2–24.0) · p95 25.7 |
| wasm | raw.dispatch_view | js.parse_view | 14.8 (14.4–15.7) · p95 17.2 | 14.7 (14.6–15.7) · p95 16.5 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 37.8 (36.9–39.9) · p95 44.3 | 37.6 (37.2–40.1) · p95 42.2 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.2 (0.2–0.3) · p95 0.4 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 22.5 (22.2–24.8) · p95 26.3 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 15.1 (14.6–15.9) · p95 17.2 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 38.0 (37.1–41.1) · p95 43.4 | - |
| wasm | kit | kit.dispatch | 123.3 (121.8–130.2) · p95 138.4 | 124.2 (122.8–127.7) · p95 143.3 |
| wasm | kit | kit.view | 25.8 (25.4–27.3) · p95 29.1 | 26.0 (25.6–26.5) · p95 29.7 |
| wasm | kit | kit.dispatch+view | 149.3 (147.3–157.7) · p95 167.4 | 150.5 (148.8–154.4) · p95 172.9 |
| wasm | kit.dispatchView | kit.dispatchView | 39.0 (37.6–40.1) · p95 44.2 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 0.5 (0.4–0.5) · p95 0.7 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 23.3 (22.7–23.8) · p95 26.9 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 15.2 (14.7–15.4) · p95 17.3 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 39.0 (37.9–39.8) · p95 44.8 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 14.4 (14.0–14.6) · p95 16.5 | - |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | opt | 1.0 (0.8–1.7) | 0.2 (0.2–0.3) | 0.2 (0.2–0.3) | 0.3 (0.2–0.3) |
| wasm | raw.dispatch_view | js.stringify_payload | main | 0.8 (0.7–1.1) | 0.2 (0.2–0.3) | 0.2 (0.2–0.3) | 0.3 (0.2–0.3) |
| wasm | raw.dispatch_view | raw.dispatch_view | opt | 119.0 (104.5–125.9) | 24.8 (23.5–26.7) | 22.6 (22.6–24.1) | 22.6 (22.1–23.9) |
| wasm | raw.dispatch_view | raw.dispatch_view | main | 119.5 (115.7–159.6) | 25.1 (24.8–26.5) | 22.5 (22.4–24.0) | 22.4 (22.2–23.9) |
| wasm | raw.dispatch_view | js.parse_view | opt | 24.3 (23.2–28.1) | 14.0 (13.3–15.2) | 14.3 (14.1–15.3) | 14.8 (14.4–15.7) |
| wasm | raw.dispatch_view | js.parse_view | main | 25.2 (24.7–30.2) | 14.2 (14.1–15.0) | 14.3 (14.2–15.1) | 14.8 (14.6–15.7) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | opt | 144.1 (130.0–171.6) | 39.1 (37.2–42.3) | 37.2 (37.1–39.7) | 37.8 (36.8–40.0) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main | 145.2 (138.1–190.6) | 39.5 (39.1–41.7) | 37.2 (36.9–39.5) | 37.5 (37.1–40.0) |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | opt | 0.9 (0.8–1.5) | 0.2 (0.2–0.3) | 0.2 (0.2–0.3) | 0.2 (0.2–0.3) |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | opt | 119.3 (112.7–133.5) | 25.1 (23.8–27.2) | 23.1 (22.6–25.0) | 22.4 (22.1–24.7) |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | opt | 25.1 (24.4–27.7) | 14.2 (13.7–15.5) | 14.4 (14.3–15.5) | 15.1 (14.6–16.0) |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | opt | 143.3 (138.0–166.5) | 39.8 (37.8–43.0) | 37.7 (37.2–40.8) | 37.9 (37.0–41.0) |
| wasm | kit | kit.dispatch | opt | 247.7 (237.9–261.0) | 118.5 (116.2–120.7) | 120.5 (117.3–122.1) | 123.6 (122.3–131.5) |
| wasm | kit | kit.dispatch | main | 257.8 (239.1–285.8) | 120.2 (116.4–132.2) | 120.3 (117.6–129.2) | 124.6 (123.2–127.5) |
| wasm | kit | kit.view | opt | 34.7 (33.3–36.8) | 24.4 (23.7–24.9) | 25.2 (24.5–25.7) | 25.9 (25.5–27.6) |
| wasm | kit | kit.view | main | 33.4 (32.5–41.0) | 24.6 (23.9–26.9) | 25.1 (24.5–26.8) | 26.0 (25.7–26.4) |
| wasm | kit | kit.dispatch+view | opt | 284.5 (266.9–307.2) | 143.1 (140.0–145.9) | 146.0 (141.9–148.1) | 149.7 (147.9–159.2) |
| wasm | kit | kit.dispatch+view | main | 297.6 (273.7–326.9) | 145.4 (140.7–159.5) | 145.6 (142.5–156.4) | 150.8 (149.3–154.1) |
| wasm | kit.dispatchView | kit.dispatchView | opt | 160.4 (154.9–165.1) | 40.4 (39.2–42.2) | 39.4 (37.4–39.6) | 38.9 (37.6–40.1) |
| wasm | kit.dispatchView.pieces | kit.payloadText | opt | 2.5 (2.3–4.1) | 0.5 (0.4–0.5) | 0.4 (0.4–0.5) | 0.5 (0.4–0.5) |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | opt | 116.2 (111.7–121.3) | 25.8 (24.7–26.6) | 23.1 (22.5–23.9) | 23.2 (22.6–23.8) |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | opt | 25.0 (24.3–27.3) | 14.5 (13.9–15.0) | 14.5 (14.1–15.0) | 15.2 (14.8–15.4) |
| wasm | kit.dispatchView.pieces | kit.validOutcome | opt | 1.9 (1.5–3.6) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | opt | 165.4 (147.5–168.6) | 40.9 (39.2–42.2) | 38.1 (37.1–39.5) | 39.0 (37.9–39.8) |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | opt | 15.4 (14.5–15.9) | 13.8 (13.2–14.2) | 13.7 (13.5–14.1) | 14.5 (14.0–14.6) |

## ledger-session

The agent ledger replaying the real 2026-09-24 session log: 20 events, five of them refused by policy. Played 100 times from a fresh session. Program `experiments/agent-ledger/ledger.cav` (`2206ac79f23c`), stream `ef2143311f86`, 1 episode(s) × 100 = 2000 events per pass.

| Engine | Mode | Operation | opt | main |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.9 | 0.3 (0.3–0.3) · p95 0.9 |
| wasm | raw.dispatch_view | raw.dispatch_view | 26.2 (26.1–26.3) · p95 33.9 | 26.3 (26.0–27.8) · p95 34.1 |
| wasm | raw.dispatch_view | js.parse_view | 6.3 (6.2–6.6) · p95 9.0 | 6.2 (6.2–6.7) · p95 8.8 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 35.2 (34.7–36.4) · p95 43.1 | 34.9 (34.6–37.3) · p95 42.8 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.3 (0.3–0.3) · p95 0.8 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 25.5 (25.2–26.0) · p95 32.8 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 6.2 (6.1–6.3) · p95 8.5 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 32.1 (31.7–32.5) · p95 41.2 | - |
| wasm | kit | kit.dispatch | 102.5 (101.9–104.4) · p95 123.9 | 101.8 (101.0–104.7) · p95 125.6 |
| wasm | kit | kit.view | 13.4 (13.3–13.7) · p95 16.4 | 13.3 (13.1–13.7) · p95 16.5 |
| wasm | kit | kit.dispatch+view | 115.6 (115.0–117.6) · p95 139.8 | 114.7 (114.0–118.1) · p95 141.5 |
| wasm | kit.dispatchView | kit.dispatchView | 33.4 (32.7–33.9) · p95 42.8 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 0.9 (0.9–1.0) · p95 2.3 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 25.8 (25.0–27.5) · p95 33.3 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 6.0 (5.8–6.4) · p95 8.5 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 32.9 (31.9–35.2) · p95 43.1 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 5.1 (4.9–5.4) · p95 6.4 | - |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | opt | main |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.5 (0.5–0.5) · p95 1.5 | 0.5 (0.4–0.5) · p95 1.4 |
| wasm | raw.dispatch_view | raw.dispatch_view | 66.7 (64.4–67.1) · p95 136.8 | 66.5 (64.7–67.3) · p95 136.3 |
| wasm | raw.dispatch_view | js.parse_view | 23.4 (22.9–24.0) · p95 28.1 | 23.1 (22.8–23.7) · p95 27.7 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 111.4 (109.5–113.0) · p95 164.5 | 110.2 (108.6–113.0) · p95 163.9 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.5 (0.4–0.5) · p95 1.4 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 67.3 (64.9–67.9) · p95 138.1 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 22.6 (22.5–23.0) · p95 27.9 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 93.2 (91.0–93.8) · p95 163.9 | - |
| wasm | kit | kit.dispatch | 199.1 (195.9–204.8) · p95 275.8 | 199.1 (197.9–203.3) · p95 275.7 |
| wasm | kit | kit.view | 30.4 (30.1–31.7) · p95 38.9 | 30.4 (30.2–31.3) · p95 38.8 |
| wasm | kit | kit.dispatch+view | 232.5 (229.6–239.2) · p95 312.2 | 232.7 (231.3–237.1) · p95 313.5 |
| wasm | kit.dispatchView | kit.dispatchView | 93.9 (91.7–97.3) · p95 165.5 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 1.2 (1.2–1.3) · p95 3.7 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 65.7 (64.9–68.5) · p95 136.4 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 20.8 (20.8–21.5) · p95 26.4 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.3 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 91.1 (90.2–94.1) · p95 161.1 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 18.1 (17.7–18.8) · p95 21.6 | - |

