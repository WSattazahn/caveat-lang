# Caveat performance baseline: 2026-09-29T18-42-33-481-view-path

Written by `experiments/performance-0.1/run.mjs` from results.json in this directory. Command: `node experiments/performance-0.1/run.mjs --suite=view-path --engines=wasm --build=never --keep-samples --repeats=3 --monitor-interval=1 --affinity=0x3 --priority=high --target=opt=tree:. --target=main=tree:../prOPT-main --out=C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prOPT-measure\runs\B1-0x3`

## Method

- Suite `view-path`: 3 repeat(s) of every mode, each in a fresh process; within a repeat every target runs a mode back to back, the first target rotating.
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
- Pinning: affinity 0x3 (logical processors 0, 1), priority high, via cmd /c start /b /wait /<priority> /affinity <mask>
- Node v24.11.1, V8 13.6.233.10-node.28, npm 11.12.1
- Rust: rustc 1.98.1 (48a229cea 2026-09-01) (LLVM version: 22.1.8); cargo 1.98.1 (797e8a9bc 2026-08-05); 1.98.1-x86_64-pc-windows-msvc (overridden by 'C:\Users\walte\AppData\Local\Temp\claude\C--Dev-GPT-SandBox-Web\3bc468a2-40f8-4895-bd0a-fb452fae7549\scratchpad\prOPT\rust-toolchain.toml')
- wasm-bindgen 0.2.104; wasm-opt not installed (the build does not use it)
- Load before: total CPU 10/0/0%; busiest over the window: powershell 0.422s, claude 0.344s, claude 0.094s, Reallusion Hub 0.078s, claude 0.078s, uihost 0.047s
- Load after: total CPU 4/0/8%; busiest over the window: powershell 0.469s, claude 0.344s, claude 0.125s, claude 0.094s, IntelGraphicsSoftware 0.062s, Reallusion Hub 0.047s
- Load during: typeperf every 1 s for the whole run, 203 samples: total mean 7% p95 9.3% max 13.2%; processor 0 mean 41.1% p95 76.7% max 95.4%; processor 1 mean 71% p95 98.4% max 100%; processor 0 MHz mean 5038.2 MHz p95 5150.5 MHz max 5174 MHz; processor 1 MHz mean 5116.1 MHz p95 5155.7 MHz max 5160.3 MHz; total above 10% in 9 and above 25% in 0 samples; pinned processors summing above 150% in 0

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
| wasm-bindgen dispatch_view call, returning the view text | 18.8 | 19.0 |
|   arguments copied into WebAssembly memory | - | - |
|   WebAssembly execution (resolve, apply, view, serialize) | - | - |
|   result decoded to a JavaScript string | - | - |
|   result freed | - | - |
|   the four pieces, timed together in one process | - | - |
| JSON.parse(view text) | 12.4 | 12.4 |
| WebAssembly view() alone, execution only | - | - |
| Kit dispatch (dispatch_outcome with the full snapshot, parsed) | 104.5 | 105.4 |
| Kit dispatch + view() | 126.6 | 127.4 |
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
| wasm | raw.dispatch_view | raw.dispatch_view | 18.9 (18.8–19.1) · p95 21.6 | 19.0 (19.0–19.1) · p95 21.8 |
| wasm | raw.dispatch_view | js.parse_view | 12.4 (12.4–12.5) · p95 14.2 | 12.4 (12.3–12.5) · p95 14.2 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 31.6 (31.5–31.8) · p95 35.7 | 31.7 (31.6–31.9) · p95 35.8 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.2 (0.2–0.2) · p95 0.4 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 19.0 (18.9–19.0) · p95 21.8 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 12.6 (12.4–12.6) · p95 14.1 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 31.8 (31.7–31.9) · p95 35.7 | - |
| wasm | kit | kit.dispatch | 104.2 (103.6–104.7) · p95 119.8 | 105.0 (104.4–105.1) · p95 123.5 |
| wasm | kit | kit.view | 21.9 (21.7–22.0) · p95 24.7 | 21.9 (21.8–22.0) · p95 25.1 |
| wasm | kit | kit.dispatch+view | 126.3 (125.5–126.9) · p95 146.0 | 127.0 (126.6–127.4) · p95 149.6 |
| wasm | kit.dispatchView | kit.dispatchView | 31.8 (31.7–32.0) · p95 35.8 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 0.4 (0.4–0.4) · p95 0.7 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 19.2 (19.2–19.4) · p95 22.1 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 12.5 (12.5–12.5) · p95 14.5 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.1 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 32.2 (32.2–32.5) · p95 36.9 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 11.9 (11.8–11.9) · p95 13.8 | - |

Segment medians (events observations 0–2, decay 3–602, transition 603–1299, steady 1300–9999):

| Engine | Mode | Operation | Target | observations | decay | transition | steady |
| --- | --- | --- | --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | opt | 0.7 (0.7–0.8) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | raw.dispatch_view | js.stringify_payload | main | 0.8 (0.7–0.8) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | raw.dispatch_view | raw.dispatch_view | opt | 114.7 (114.3–116.0) | 21.0 (21.0–21.1) | 19.0 (18.9–19.0) | 18.8 (18.7–19.1) |
| wasm | raw.dispatch_view | raw.dispatch_view | main | 111.5 (105.9–113.9) | 21.4 (21.2–21.4) | 19.2 (19.1–19.3) | 19.0 (19.0–19.0) |
| wasm | raw.dispatch_view | js.parse_view | opt | 23.8 (23.6–24.2) | 11.9 (11.9–12.0) | 12.1 (12.0–12.1) | 12.4 (12.4–12.5) |
| wasm | raw.dispatch_view | js.parse_view | main | 23.5 (23.3–24.1) | 11.9 (11.9–12.0) | 12.0 (11.9–12.1) | 12.4 (12.3–12.5) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | opt | 139.5 (138.5–142.6) | 33.3 (33.1–33.3) | 31.3 (31.2–31.3) | 31.6 (31.4–31.8) |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | main | 134.4 (131.3–138.1) | 33.4 (33.4–33.5) | 31.4 (31.4–31.5) | 31.7 (31.6–31.8) |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | opt | 0.7 (0.6–0.8) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) | 0.2 (0.2–0.2) |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | opt | 110.9 (103.9–113.7) | 21.4 (21.2–21.6) | 19.4 (19.2–19.4) | 18.9 (18.8–19.0) |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | opt | 24.0 (22.5–26.8) | 12.2 (12.0–12.2) | 12.3 (12.1–12.3) | 12.7 (12.4–12.7) |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | opt | 131.1 (127.2–131.4) | 33.7 (33.6–34.0) | 31.7 (31.6–31.9) | 31.7 (31.7–31.8) |
| wasm | kit | kit.dispatch | opt | 211.2 (209.6–216.2) | 100.2 (99.9–100.3) | 101.3 (100.0–101.5) | 104.5 (103.9–105.0) |
| wasm | kit | kit.dispatch | main | 213.9 (213.8–274.0) | 100.4 (100.2–100.4) | 101.0 (100.5–101.3) | 105.4 (104.7–105.4) |
| wasm | kit | kit.view | opt | 32.3 (30.9–33.1) | 20.8 (20.5–20.8) | 21.3 (20.9–21.3) | 22.0 (21.7–22.1) |
| wasm | kit | kit.view | main | 33.5 (32.5–40.8) | 20.5 (20.5–20.7) | 21.2 (20.9–21.2) | 21.9 (21.8–22.1) |
| wasm | kit | kit.dispatch+view | opt | 248.7 (248.6–258.0) | 121.1 (120.9–121.4) | 122.8 (121.2–123.2) | 126.6 (125.8–127.3) |
| wasm | kit | kit.dispatch+view | main | 261.3 (257.9–319.6) | 121.2 (121.0–121.5) | 122.5 (121.6–122.8) | 127.4 (126.9–127.8) |
| wasm | kit.dispatchView | kit.dispatchView | opt | 155.5 (153.6–164.9) | 33.9 (33.7–33.9) | 31.5 (31.4–31.6) | 31.7 (31.7–31.9) |
| wasm | kit.dispatchView.pieces | kit.payloadText | opt | 2.1 (2.0–2.2) | 0.4 (0.4–0.5) | 0.4 (0.4–0.4) | 0.4 (0.4–0.4) |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | opt | 113.0 (104.7–120.0) | 21.5 (21.5–21.9) | 19.6 (19.4–20.0) | 19.2 (19.1–19.4) |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | opt | 23.6 (23.6–24.1) | 12.0 (12.0–12.1) | 12.1 (12.1–12.3) | 12.5 (12.5–12.6) |
| wasm | kit.dispatchView.pieces | kit.validOutcome | opt | 1.4 (1.3–1.4) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) | 0.1 (0.1–0.1) |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | opt | 156.6 (150.4–163.4) | 34.1 (34.1–34.6) | 32.1 (32.0–32.7) | 32.2 (32.1–32.4) |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | opt | 13.9 (13.9–14.5) | 11.4 (11.4–11.5) | 11.5 (11.5–11.6) | 11.9 (11.8–11.9) |

## ledger-session

The agent ledger replaying the real 2026-09-24 session log: 20 events, five of them refused by policy. Played 100 times from a fresh session. Program `experiments/agent-ledger/ledger.cav` (`2206ac79f23c`), stream `ef2143311f86`, 1 episode(s) × 100 = 2000 events per pass.

| Engine | Mode | Operation | opt | main |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.2 (0.2–0.2) · p95 0.8 | 0.2 (0.2–0.2) · p95 0.8 |
| wasm | raw.dispatch_view | raw.dispatch_view | 23.3 (23.3–23.4) · p95 29.5 | 23.2 (23.1–23.3) · p95 29.7 |
| wasm | raw.dispatch_view | js.parse_view | 5.6 (5.5–5.6) · p95 8.2 | 5.5 (5.5–5.6) · p95 8.0 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 30.7 (30.6–30.8) · p95 38.4 | 30.7 (30.7–30.7) · p95 38.1 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.2 (0.2–0.2) · p95 0.8 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 22.8 (22.7–22.9) · p95 28.2 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 5.4 (5.4–5.5) · p95 7.6 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 28.4 (28.4–28.6) · p95 35.8 | - |
| wasm | kit | kit.dispatch | 89.0 (88.9–89.0) · p95 109.5 | 89.3 (88.8–89.3) · p95 109.0 |
| wasm | kit | kit.view | 11.6 (11.5–11.7) · p95 14.3 | 11.6 (11.5–11.6) · p95 14.2 |
| wasm | kit | kit.dispatch+view | 100.4 (100.2–100.5) · p95 123.8 | 100.6 (100.1–100.6) · p95 122.7 |
| wasm | kit.dispatchView | kit.dispatchView | 28.9 (28.8–29.1) · p95 37.1 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 0.8 (0.7–0.8) · p95 2.0 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 23.0 (22.8–23.2) · p95 28.7 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 5.3 (5.2–5.3) · p95 7.6 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.2 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 29.3 (28.7–29.4) · p95 37.4 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 4.4 (4.4–4.5) · p95 5.5 | - |

## trail-rescue-scenarios

Trail Rescue: the runtime events of the 24 registered scenarios as web/trail-rescue-policy.js sends them (lib/record-trail-rescue.mjs), each scenario from a fresh session. Played 20 times. Program `game/trail_rescue.cav` (`9744ae625637`), stream `496602507c96`, 24 episode(s) × 20 = 2800 events per pass.

| Engine | Mode | Operation | opt | main |
| --- | --- | --- | --- | --- |
| wasm | raw.dispatch_view | js.stringify_payload | 0.5 (0.4–0.5) · p95 1.5 | 0.5 (0.4–0.5) · p95 1.5 |
| wasm | raw.dispatch_view | raw.dispatch_view | 58.7 (58.4–58.8) · p95 119.3 | 58.1 (57.9–58.2) · p95 119.3 |
| wasm | raw.dispatch_view | js.parse_view | 20.5 (20.3–20.6) · p95 24.7 | 20.3 (20.3–20.4) · p95 24.7 |
| wasm | raw.dispatch_view | raw.dispatch_view+parse | 97.1 (97.0–97.3) · p95 143.7 | 97.3 (97.0–97.6) · p95 143.7 |
| wasm | raw.dispatch_view_outcome | js.stringify_payload | 0.5 (0.5–0.5) · p95 1.5 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome | 58.9 (57.9–58.9) · p95 119.7 | - |
| wasm | raw.dispatch_view_outcome | js.parse_view_outcome | 19.7 (19.6–19.9) · p95 24.4 | - |
| wasm | raw.dispatch_view_outcome | raw.dispatch_view_outcome+parse | 80.9 (80.7–81.1) · p95 142.1 | - |
| wasm | kit | kit.dispatch | 176.5 (176.0–177.0) · p95 248.5 | 178.1 (177.9–178.4) · p95 252.6 |
| wasm | kit | kit.view | 27.2 (27.1–27.3) · p95 34.8 | 27.5 (27.2–27.5) · p95 35.2 |
| wasm | kit | kit.dispatch+view | 206.8 (206.6–207.6) · p95 281.9 | 209.1 (208.6–209.2) · p95 285.7 |
| wasm | kit.dispatchView | kit.dispatchView | 82.9 (82.2–83.9) · p95 145.8 | - |
| wasm | kit.dispatchView.pieces | kit.payloadText | 1.1 (1.1–1.1) · p95 3.7 | - |
| wasm | kit.dispatchView.pieces | raw.dispatch_view_outcome | 59.6 (59.6–59.6) · p95 121.0 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_outcome | 18.6 (18.5–18.7) · p95 24.2 | - |
| wasm | kit.dispatchView.pieces | kit.validOutcome | 0.1 (0.1–0.1) · p95 0.3 | - |
| wasm | kit.dispatchView.pieces | kit.dispatchView.pieces | 81.8 (81.5–82.0) · p95 143.1 | - |
| wasm | kit.dispatchView.pieces | js.parse_view_alone | 15.9 (15.9–16.0) · p95 19.3 | - |

