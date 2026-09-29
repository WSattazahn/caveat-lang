# Caveat performance baseline: 2026-09-29T20-00-02-459-view-path

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=view-path --engines=wasm --build=never --keep-samples --repeats=3 --monitor-interval=1 --monitor-lead=5 --job-guard --affinity=0x3 --priority=high --target=opt=tree:. --target=main=tree:../prOPT-main --out=C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prOPT-measure2\runs\G2-0x3`

## Method

- Suite `view-path`: 3 repeat(s) of every mode, each in a fresh process; within a repeat every target runs a mode back to back, the first target rotating.
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
- Pinning: affinity 0x3 (logical processors 0, 1), priority high, via cmd /c start /b /wait /<priority> /affinity <mask>
- Node v24.11.1, V8 13.6.233.10-node.28, npm 11.12.1
- Rust: rustc 1.98.1 (48a229cea 2026-09-01) (LLVM version: 22.1.8); cargo 1.98.1 (797e8a9bc 2026-08-05); 1.98.1-x86_64-pc-windows-msvc (overridden by 'C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prOPT\rust-toolchain.toml')
- wasm-bindgen 0.2.104; wasm-opt not installed (the build does not use it)
- Load before: total CPU 1/1/0%; busiest over the window: powershell 0.453s, Reallusion Hub 0.156s, bash 0.109s, claude 0.109s, nvsphelper64 0.094s, PredatorSense 0.078s
- Load after: total CPU 3/0/0%; busiest over the window: powershell 0.469s, Reallusion Hub 0.125s, claude 0.078s, OpenConsole 0.047s, claude 0.047s, IntelGraphicsSoftware 0.047s
- Load during: typeperf every 1 s for the whole run, 315 samples: total mean 5.1% p95 10.5% max 17.3%; processor 0 mean 23.2% p95 60% max 100%; processor 1 mean 56.3% p95 100% max 100%; processor 0 MHz mean 4807.9 MHz p95 5195.5 MHz max 5233.4 MHz; processor 1 MHz mean 4913.7 MHz p95 5193.7 MHz max 5201.8 MHz; process System mean 8.3% p95 21.3% max 108.4%; process MsMpEng mean 1.5% p95 6.1% max 12.2%; total above 10% in 22 and above 25% in 0 samples; pinned processors summing above 150% in 0

## Published reference

- Dispatch + view, Glowcap replay: median 51.6 µs, p95 59.1 µs, max 646.7 µs (experiments/glowcap/evidence/round6-replay.json timing.dispatchAndShippedOutput); runtime 8d8596a, reactive WebAssembly `d897787f414a`, repository e6ace96.
- Resume ten minutes of play: median 3.2416 ms of runs 7.154, 3.2416, 3.1401 ms; save 2045 bytes (experiments/glowcap/evidence/round6-replay.json timing.resume (caveat5)).

## Final-state consistency

Within each target, every run of every mode and engine ended each workload in the same saved state (63 runs compared).

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

## One Glowcap event + view, piece by piece

Medians over runs of each run's median on the steady idle ticks (events 1300–9999), where the pooled median sits; each row is its own process, so parts need not sum exactly to the whole.

| Piece | opt | main |
| --- | --- | --- |
| Published method: adapter dispatch + view | - | - |
| Adapter dispatch (stringify, WebAssembly dispatch_view, JSON.parse) | - | - |
| Adapter view() (JavaScript reshaping only) | - | - |
| JSON.stringify(payload) | 0.2 | 0.2 |
| wasm-bindgen dispatch_view call, returning the view text | 18.9 | 18.7 |
|   arguments copied into WebAssembly memory | - | - |
|   WebAssembly execution (resolve, apply, view, serialize) | - | - |
|   result decoded to a JavaScript string | - | - |
|   result freed | - | - |
|   the four pieces, timed together in one process | - | - |
| JSON.parse(view text) | 12.5 | 12.3 |
| WebAssembly view() alone, execution only | - | - |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | 103.4 | 103.9 |
| Kit dispatch + view() | 125.1 | 125.9 |
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
| wasm | raw.dispatch_view | js.stringify_payload | 0.2 (0.2–0.2) · p95 0.4 | 0.2 (0.2–0.2) · p95 0.4 |
| wasm | raw.dispatch_view | raw.dispatch_view | 18.9 (18.8–19.0) · p95 21.6 | 18.8 (18.7–18.8) · p95 21.3 |
| wasm | raw.dispatch_view | js.parse_view | 12.5 (12.3–12.6) · p95 13.9 | 12.3 (12.3–12.3) · p95 13.4 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 31.6 (31.6–31.8) · p95 35.2 | 31.3 (31.2–31.4) · p95 34.7 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.2 (0.2–0.2) · p95 0.4 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 19.0 (18.8–19.0) · p95 21.4 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 12.4 (12.3–12.7) · p95 13.7 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 31.6 (31.4–31.9) · p95 35.3 | - |
| wasm | kit | kit.dispatch | 103.2 (103.1–103.4) · p95 118.0 | 103.6 (103.3–103.9) · p95 118.6 |
| wasm | kit | kit.view | 21.5 (21.4–21.7) · p95 24.3 | 21.9 (21.8–21.9) · p95 24.8 |
| wasm | kit | kit.dispatch+view | 124.8 (124.7–125.4) · p95 142.6 | 125.6 (125.4–126.0) · p95 143.7 |
| wasm | kit.dispatchView | kit.dispatchView | 31.5 (31.4–31.9) · p95 35.1 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 0.4 (0.4–0.4) · p95 0.6 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 19.2 (19.0–19.3) · p95 22.1 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 12.4 (12.4–12.5) · p95 14.4 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.1 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 32.2 (32.0–32.2) · p95 36.8 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 11.8 (11.7–11.9) · p95 13.6 | - |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | opt | 0.9 (0.9–1.0) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | raw.dispatch_view | js.stringify_payload | main | 1.0 (0.8–1.4) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | raw.dispatch_view | raw.dispatch_view | opt | 112.8 (111.1–121.4) | 21.2 (21.0–21.4) | 19.1 (19.0–19.1) | 18.9 (18.8–19.0) |
| wasm | raw.dispatch_view | raw.dispatch_view | main | 110.6 (108.7–111.3) | 21.0 (21.0–21.1) | 18.9 (18.8–19.0) | 18.7 (18.6–18.8) |
| wasm | raw.dispatch_view | js.parse_view | opt | 24.2 (23.1–27.0) | 12.0 (11.9–12.1) | 12.1 (11.9–12.2) | 12.5 (12.4–12.6) |
| wasm | raw.dispatch_view | js.parse_view | main | 24.1 (23.4–24.8) | 11.9 (11.8–11.9) | 11.9 (11.9–11.9) | 12.3 (12.3–12.3) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | opt | 148.2 (132.0–154.1) | 33.4 (33.2–33.5) | 31.4 (31.2–31.4) | 31.6 (31.5–31.8) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main | 132.4 (132.3–164.8) | 33.1 (33.0–33.1) | 31.0 (30.9–31.1) | 31.3 (31.2–31.4) |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | opt | 0.8 (0.7–0.8) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | opt | 110.2 (97.8–118.2) | 21.0 (21.0–21.3) | 19.2 (18.9–19.4) | 18.9 (18.8–18.9) |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | opt | 23.9 (23.0–24.2) | 11.9 (11.9–12.2) | 12.0 (11.9–12.4) | 12.4 (12.4–12.7) |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | opt | 128.3 (121.8–144.9) | 33.4 (33.1–33.5) | 31.7 (31.0–31.8) | 31.5 (31.4–31.9) |
| wasm | kit | kit.dispatch | opt | 224.0 (212.7–230.1) | 99.0 (98.8–99.4) | 99.3 (99.1–100.1) | 103.4 (103.3–103.7) |
| wasm | kit | kit.dispatch | main | 212.1 (211.7–214.3) | 99.5 (99.3–99.6) | 100.2 (99.6–100.4) | 103.9 (103.6–104.2) |
| wasm | kit | kit.view | opt | 34.4 (33.0–35.3) | 20.3 (20.2–20.4) | 20.7 (20.6–20.9) | 21.6 (21.5–21.8) |
| wasm | kit | kit.view | main | 31.8 (31.0–37.8) | 20.6 (20.4–20.6) | 21.0 (20.9–21.0) | 21.9 (21.8–21.9) |
| wasm | kit | kit.dispatch+view | opt | 265.4 (254.1–266.0) | 119.4 (119.3–120.0) | 120.1 (119.9–121.3) | 125.1 (125.0–125.7) |
| wasm | kit | kit.dispatch+view | main | 251.1 (249.2–252.2) | 120.2 (120.1–120.4) | 121.6 (120.8–121.6) | 125.9 (125.7–126.3) |
| wasm | kit.dispatchView | kit.dispatchView | opt | 131.0 (128.6–156.0) | 33.3 (33.2–33.6) | 31.2 (31.1–31.5) | 31.5 (31.4–31.9) |
| wasm | kit.dispatchView.pieces | kit.payloadText | opt | 2.5 (2.3–3.3) | 0.4 (0.4–0.5) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | opt | 110.4 (105.8–114.3) | 21.5 (21.4–21.9) | 19.6 (19.3–19.7) | 19.2 (18.9–19.2) |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | opt | 23.5 (23.5–23.7) | 12.1 (11.9–12.1) | 12.1 (12.0–12.2) | 12.5 (12.5–12.5) |
| wasm | kit.dispatchView.pieces | kit.validOutcome | opt | 1.3 (1.3–1.3) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | opt | 154.1 (145.8–164.0) | 34.0 (34.0–34.5) | 32.1 (32.0–32.2) | 32.1 (31.9–32.2) |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | opt | 14.2 (13.9–14.6) | 11.5 (11.3–11.5) | 11.5 (11.4–11.6) | 11.9 (11.8–11.9) |

## ledger-session

The agent ledger replaying the real 2026-09-24 session log: 20 events, five of them refused by policy. Played 100 times from a fresh session. Program `experiments/agent-ledger/ledger.cav` (`2206ac79f23c`), stream `ef2143311f86`, 1 episode(s) × 100 = 2000 events per pass.

| Engine | Mode | Operation | opt | main |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.2 (0.2–0.2) · p95 0.8 | 0.2 (0.2–0.2) · p95 0.8 |
| wasm | raw.dispatch_view | raw.dispatch_view | 22.8 (22.8–23.0) · p95 29.1 | 23.1 (22.9–23.1) · p95 29.1 |
| wasm | raw.dispatch_view | js.parse_view | 5.4 (5.4–5.5) · p95 7.8 | 5.5 (5.5–5.6) · p95 7.7 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 30.2 (30.0–30.4) · p95 37.0 | 30.3 (30.2–30.7) · p95 37.2 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.2 (0.2–0.2) · p95 0.7 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 22.4 (22.4–22.5) · p95 27.9 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 5.3 (5.3–5.4) · p95 7.3 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 28.1 (28.0–28.2) · p95 35.1 | - |
| wasm | kit | kit.dispatch | 87.7 (87.5–88.7) · p95 106.3 | 87.6 (87.6–88.2) · p95 106.9 |
| wasm | kit | kit.view | 11.5 (11.5–11.5) · p95 14.0 | 11.4 (11.3–11.5) · p95 14.1 |
| wasm | kit | kit.dispatch+view | 98.9 (98.8–100.0) · p95 120.2 | 98.9 (98.8–99.5) · p95 119.9 |
| wasm | kit.dispatchView | kit.dispatchView | 28.7 (28.6–28.7) · p95 36.2 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 0.7 (0.7–0.8) · p95 1.9 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 22.7 (22.6–22.8) · p95 28.4 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 5.2 (5.1–5.2) · p95 7.4 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 28.7 (28.5–28.9) · p95 36.8 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 4.4 (4.4–4.4) · p95 5.4 | - |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | opt | main |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.4 (0.4–0.4) · p95 1.4 | 0.4 (0.4–0.4) · p95 1.3 |
| wasm | raw.dispatch_view | raw.dispatch_view | 56.6 (56.4–57.8) · p95 116.8 | 56.2 (55.8–57.9) · p95 116.9 |
| wasm | raw.dispatch_view | js.parse_view | 20.2 (20.0–20.2) · p95 23.8 | 20.0 (19.9–20.1) · p95 23.7 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 95.4 (94.9–95.4) · p95 140.5 | 95.5 (95.0–96.0) · p95 140.4 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.4 (0.4–0.4) · p95 1.3 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 57.9 (57.2–58.4) · p95 118.7 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 19.8 (19.3–19.8) · p95 24.0 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 80.1 (79.9–80.6) · p95 140.5 | - |
| wasm | kit | kit.dispatch | 174.7 (172.9–174.8) · p95 243.5 | 175.1 (175.0–176.9) · p95 244.7 |
| wasm | kit | kit.view | 26.9 (26.5–27.1) · p95 34.3 | 26.7 (26.7–26.7) · p95 34.0 |
| wasm | kit | kit.dispatch+view | 204.9 (202.4–205.5) · p95 275.9 | 204.9 (204.7–206.8) · p95 276.9 |
| wasm | kit.dispatchView | kit.dispatchView | 81.4 (80.9–82.3) · p95 143.3 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 1.1 (1.1–1.1) · p95 3.4 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 58.5 (57.9–59.2) · p95 118.1 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 18.3 (18.2–18.3) · p95 23.2 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.3 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 80.0 (80.0–80.7) · p95 140.0 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 15.7 (15.6–15.7) · p95 18.9 | - |

