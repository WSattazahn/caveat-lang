# Performance Optimization 0.1: the view path, measured

What `session.dispatchView()` costs against the two paths a host that wants
the view had before, on the same machine, builds and events, in the
[Performance Baseline 0.1](../performance-0.1/RESULTS.md) harness and
protocol. Every timing is DIRECT: timed around exactly that path, per event.
Every saving is MEASURED: the difference of two DIRECT medians of the same
event class in the same repeat of the same session (the two processes ran in
one interleaved block), then its median [lowest–highest] over the repeats. No
share, model or earlier session enters any saving here; in particular the
baseline's INFERRED 91–161 µs is not used as a result.

Unless a table says otherwise, every absolute figure is for logical
processors 10–13 (`0x3C00`) at High priority, in the primary session
(18:34–18:41 UTC). As the baseline found, the performance cores of this
laptop are not interchangeable, so absolute figures hold for their processors
and session; ratios and savings in µs are given for every session. Microseconds
(µs) throughout. A cell `40.3 [39.9–41.1] · p95 44.6` is the median over the
repeats of each repeat's median, the lowest and highest repeat, then the
median of the repeats' p95.

## Headline

Per event class, the existing kit path (`dispatch()` then `view()`), the new
kit path (`dispatchView()`), the web pages' raw path (`dispatch_view` then
`JSON.parse`) and the new export's raw path (`dispatch_view_outcome` then
`JSON.parse`), and the saving. Primary session, processors 10–13, 5 repeats.

| Event | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` | saving: kit `dispatch()` + `view()` − `dispatchView()` | old ÷ new | gap: `dispatchView()` − raw `dispatch_view` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 158.3 [157.4–160.5] · p95 176.7 | 40.3 [39.9–41.1] · p95 44.6 | 40.0 [39.6–40.3] · p95 44.4 | 39.9 [39.8–40.4] · p95 44.6 | 118.0 [117.0–119.8] | 3.93× [3.90–4.00] | +0.4 [-0.1 to +0.8] |
| Glowcap state-changing tick | 150.1 [149.7–152.9] · p95 188.3 | 42.5 [41.9–43.0] · p95 48.5 | 41.6 [41.0–41.9] · p95 56.4 | 42.3 [41.1–42.9] · p95 56.7 | 107.9 [107.2–109.9] | 3.56× [3.52–3.58] | +1.1 [0.0 to +1.5] |
| Trail Rescue evidence | 289.5 [288.1–298.1] · p95 357.4 | 140.7 [139.6–142.4] · p95 179.9 | 138.8 [137.6–142.4] · p95 179.1 | 137.7 [137.2–140.0] · p95 176.6 | 149.3 [147.7–155.7] | 2.06× [2.04–2.09] | +1.8 [0.0 to +3.0] |
| Trail Rescue commit | 240.1 [238.1–245.0] · p95 288.6 | 91.8 [90.6–93.0] · p95 118.0 | 90.2 [90.0–91.8] · p95 116.0 | 90.2 [89.1–92.1] · p95 116.1 | 147.7 [146.8–152.0] | 2.61× [2.60–2.65] | +1.2 [+0.5 to +2.2] |
| Trail Rescue reopen | 313.3 [311.0–320.0] · p95 360.9 | 161.3 [157.6–161.8] · p95 187.2 | 157.7 [155.6–159.4] · p95 186.6 | 158.8 [157.6–160.4] · p95 188.3 | 151.7 [151.2–158.2] | 1.95× [1.94–1.99] | +2.5 [+1.5 to +4.1] |
| Trail Rescue qualify | 288.7 [286.5–294.8] · p95 351.5 | 135.0 [134.0–135.8] · p95 179.6 | 132.6 [130.6–134.9] · p95 175.6 | 134.3 [132.3–135.7] · p95 178.8 | 154.4 [151.3–159.2] | 2.15× [2.11–2.17] | +2.3 [+0.7 to +3.4] |
| Ledger evidence | 126.4 [125.2–128.0] · p95 154.9 | 38.0 [37.6–38.8] · p95 46.3 | 37.8 [37.2–39.0] · p95 46.0 | 36.9 [36.6–37.7] · p95 44.5 | 88.4 [87.5–89.2] | 3.32× [3.30–3.34] | +0.5 [-0.2 to +0.5] |
| Trail Rescue refused | 39.0 [38.8–39.6] · p95 53.5 | 8.8 [8.7–9.0] · p95 13.5 | throws: the call 7.3 [7.2–7.4] · p95 13.3 | 7.8 [7.6–7.9] · p95 11.8 | 30.3 [30.0–30.6] | 4.41× [4.38–4.49] | - |
| Ledger refused | 22.7 [22.7–23.1] · p95 28.3 | 7.5 [7.4–7.6] · p95 9.0 | throws: the call 6.9 [6.9–7.3] · p95 8.6 | 6.8 [6.7–7.0] · p95 8.4 | 15.3 [15.2–15.5] | 3.04× [3.03–3.07] | - |

- **The saving is 88–154 µs per accepted event** on processors 10–13
  (MEASURED, paired by repeat): the old path takes 1.95× (Trail Rescue
  reopen) to 3.93× (Glowcap idle tick) as long as the new one, so 49–75% of
  its time is removed. The replicate session gives 84–148 µs (1.95–3.92×), and
  processors 0–1 give 72–128 µs (1.96–3.98×): the µs scale with the core,
  the ratios do not move (section 3).
- **`dispatchView()` costs what the web pages' path costs.** It is 0.4–2.5 µs
  (≤ 2.6%) above raw `dispatch_view` + `JSON.parse` in the primary session,
  −0.5 to +4.0 µs (≤ 3.3%) in the replicate and +0.2 to +3.7 µs (≤ 3.4%) on
  processors 0–1, paired by repeat. That is not material; section 4 still
  takes it apart. The part that is attributed is the kit's payload check
  (0.2 µs on a tick, 0.3–1.5 µs on the ledger's and Trail Rescue's payloads)
  and `validOutcome` (0.1–0.2 µs); the envelope costs nothing measurable (its
  `JSON.parse` is within ±0.5 µs of the bare view's, and the call −0.8 to
  +1.8 µs from `dispatch_view`'s, both inside the run-to-run noise).
- **A refused event** costs 8.8 µs (Trail Rescue) and 7.5 µs (ledger) through
  `dispatchView()`, against 39.0 and 22.7 µs for `dispatch()` + `view()`, and
  10.8 and 8.9 µs for `dispatch()` alone. A refusal leaves the view as it was,
  so a host that already holds it saves 1.4–1.9 µs against `dispatch()`
  alone, and 15–30 µs against `dispatch()` + `view()`.
- **The existing paths are unchanged.** Paired by repeat, main against the
  branch, kit `dispatch()` + `view()` and raw `dispatch_view` + `JSON.parse`
  differ by −0.8% to +1.0% (medians) in all three sessions, within their
  run ranges (section 3). Every mode on both builds ended every workload in
  the same saved state (section 6).
- **Size:** the reactive WebAssembly the kit bundles grows by 4,378 bytes
  (+0.21%; +1,252 gzipped), its glue by 1,279 bytes, the kit's
  `lib/session.mjs` by 1,547 bytes and the kit tarball by 2,606 bytes, with
  the same 59 files (section 5).

## Contents

1. Machine, builds and inputs · 2. Method · 3. Results · 4. What remains in the
JavaScript wrapper · 5. Size · 6. Correctness evidence · 7. Load, and the
sessions not used · 8. Noticed for the next decision · 9. Raw results and how to
rerun

## 1. Machine, builds and inputs

| | |
| --- | --- |
| CPU | Intel Core Ultra 9 275HX, 24 logical processors; performance cores 0, 1, 10–13, 22, 23 (`GetSystemCpuSetInformation`, efficiency class 1) |
| Memory | 63.4 GiB |
| OS | Windows 11 Home 10.0.26200 |
| Power | High performance plan (`8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c`) on mains power (battery status 2), checked before starting and by the harness before every timed session |
| Pinning | every timed process `start /b /wait /high /affinity MASK`: `0x3C00` (processors 10–13) in the primary and replicate sessions, `0x3` (processors 0–1) in one session; each process read back its mask (Node's `os.availableParallelism()`: 4 and 2) |
| Node | v24.11.1 (V8 13.6.233.10), `--expose-gc` |
| Build toolchain | rustc 1.98.1 (48a229cea 2026-09-01), pinned by `rust-toolchain.toml`; wasm-bindgen 0.2.104; no wasm-opt |

| Build | Revision | Reactive WebAssembly (sha256, bytes) | Glue `caveat_runtime.js` | Kit `lib/session.mjs` |
| --- | --- | --- | --- | --- |
| main (the base) | 768275b, clean | `e35944503272…`, 2,052,856 | `9066c9a19dcb…`, 14,575 | `b171d128027a…`, 10,461 |
| branch `perf/kit-dispatch-view` | a459995 | `0a19e4b85886…`, 2,057,234 | `3dcc4fd6ba4e…`, 15,854 | `f40dad99c758…`, 12,008 |

- Each tree was built twice: with `npm run build` (for the size table and
  the correctness checks) and by the harness (`runtime/target/perf-baseline/pkg-reactive`).
  In each tree the two `pkg-reactive` builds are byte-identical, WebAssembly
  and glue. main's WebAssembly is the same bytes the baseline measured as
  `main-local` (`e35944503272…`). The branch was built from a clean checkout
  of a459995. During timing its working tree held only this commit's
  uncommitted harness files under `experiments/` (the harness records it as
  `-dirty`); the runtime, kit and built bytes are a459995's.
- Each target runs its own kit: main's `kit/lib` for `kit`, the branch's for
  `kit`, `kit.dispatchView` and `kit.dispatchView.pieces`. main has no
  `dispatch_view_outcome` and no `dispatchView`, so it skips those three
  modes (45 skipped jobs in each 5-repeat session, 27 in the 3-repeat one).
- Inputs: the baseline's frozen inputs, from this checkout, with their
  sha256 checked by the harness: the Glowcap replay (10,000 events a pass), the
  agent ledger's 2026-09-24 session (20 events, 5 refused by policy, 100 times
  a pass) and Trail Rescue's 24 registered scenarios (140 runtime events, 20
  times a pass). Event classes come from the baseline's
  `inputs/event-classes.json`, labelled before any timing from what each
  event did. Per run and path, the samples per class are: Glowcap idle tick
  28,182, state-changing tick 1,809; Trail Rescue evidence 2,640, commit
  1,440, reopen 600, qualify 360, refused 1,320; ledger evidence 4,500,
  refused 1,500.

## 2. Method

- **Harness.** The baseline's `run.mjs` and `wasm-bench.mjs`, unchanged in
  method, with the `view-path` suite: the modes `raw.dispatch_view`,
  `raw.dispatch_view_outcome`, `kit`, `kit.dispatchView` and
  `kit.dispatchView.pieces` on the Glowcap replay, the ledger and Trail
  Rescue. What each times:

  | Mode | Timed, per event |
  | --- | --- |
  | `kit` | `session.dispatch(event, payload)`, then `session.view()`: the existing kit path, and each call alone |
  | `kit.dispatchView` | `session.dispatchView(event, payload)`: the new kit path |
  | `raw.dispatch_view` | `JSON.stringify(payload)`, `WebReactiveSession.dispatch_view`, `JSON.parse` of the view: the web pages' path, each piece and the whole |
  | `raw.dispatch_view_outcome` | the same with the new export and `JSON.parse` of its outcome |
  | `kit.dispatchView.pieces` | `dispatchView`'s pieces one by one, in its order, on a raw session of the same runtime: the kit's own `payloadText`, the `dispatch_view_outcome` call, `JSON.parse` of the envelope, the kit's own `validOutcome(outcome, 'view')` (from a copy of the target's `lib/session.mjs` with one added export line; both hashes recorded), and the four from first timer to last; then, for an accepted event, `JSON.parse` of the view text alone, cut from the envelope untimed |

- **Processes, interleaving, repeats.** One process per (target, workload,
  mode). Within a repeat, for each workload and each mode, both targets run
  back to back, and the target that goes first alternates each repeat. 5
  repeats on processors 10–13 in each of two sessions; 3 repeats on
  processors 0–1.
- **Passes.** One untimed warm-up pass, then 3 timed passes, each episode on a
  freshly opened session (opening and closing untimed), with a forced
  garbage collection before each pass.
- **Timers.** `process.hrtime.bigint()` (QueryPerformanceCounter, 100 ns
  ticks); a timer pair costs a median 0.1 µs in these processes.
- **Statistics.** The baseline's: per run, every timed sample of an event
  class is pooled and q(p) = sorted[floor(p·(n−1))]. Medians do not add, so no
  table adds independently timed medians into a total.
- **Quiet machine.** Before each session a bounded gate (at most 9 minutes)
  waited for a minute (12 samples, 5 s apart) with total CPU below 12% and
  Windows Defender's scanner (`MsMpEng`) below 25% of one core. The kept
  sessions' gates read a total of 2.2–2.7% on average (at most 4.2%). During
  each session the harness sampled total CPU and the pinned processors' use
  and clock every second (`load.csv`), and a typeperf pinned to efficient
  cores 2–9 sampled `MsMpEng` and `System` every 5 s (`processes.csv`). A
  repeat is kept when its mean total CPU is at most 12% and no sample is above
  20%, and a session is used only when every repeat is kept (`load.mjs`; the
  benchmark itself is about 5% of the total). Three sessions failed the rule
  and were run again (section 7).
- **Order.** Primary (processors 10–13) 18:34:05–18:41:05 UTC; processors 0–1
  18:42:33–18:46:12; replicate (processors 10–13) 18:47:25–18:54:17.

## 3. Results

### 3.1 Processors 10–13, primary session

Per event, on each path (DIRECT, µs):

| Event | samples per run | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 28182/28182/28182/28182/28182 | 158.3 [157.4–160.5] · p95 176.7 | 40.3 [39.9–41.1] · p95 44.6 | 40.0 [39.6–40.3] · p95 44.4 | 39.9 [39.8–40.4] · p95 44.6 |
| Glowcap state-changing tick | 1809/1809/1809/1809/1809 | 150.1 [149.7–152.9] · p95 188.3 | 42.5 [41.9–43.0] · p95 48.5 | 41.6 [41.0–41.9] · p95 56.4 | 42.3 [41.1–42.9] · p95 56.7 |
| Trail Rescue evidence | 2640/2640/2640/2640/2640 | 289.5 [288.1–298.1] · p95 357.4 | 140.7 [139.6–142.4] · p95 179.9 | 138.8 [137.6–142.4] · p95 179.1 | 137.7 [137.2–140.0] · p95 176.6 |
| Trail Rescue commit | 1440/1440/1440/1440/1440 | 240.1 [238.1–245.0] · p95 288.6 | 91.8 [90.6–93.0] · p95 118.0 | 90.2 [90.0–91.8] · p95 116.0 | 90.2 [89.1–92.1] · p95 116.1 |
| Trail Rescue reopen | 600/600/600/600/600 | 313.3 [311.0–320.0] · p95 360.9 | 161.3 [157.6–161.8] · p95 187.2 | 157.7 [155.6–159.4] · p95 186.6 | 158.8 [157.6–160.4] · p95 188.3 |
| Trail Rescue qualify | 360/360/360/360/360 | 288.7 [286.5–294.8] · p95 351.5 | 135.0 [134.0–135.8] · p95 179.6 | 132.6 [130.6–134.9] · p95 175.6 | 134.3 [132.3–135.7] · p95 178.8 |
| Ledger evidence | 4500/4500/4500/4500/4500 | 126.4 [125.2–128.0] · p95 154.9 | 38.0 [37.6–38.8] · p95 46.3 | 37.8 [37.2–39.0] · p95 46.0 | 36.9 [36.6–37.7] · p95 44.5 |

The saving and the gap to the raw paths (MEASURED, paired by repeat, µs):

| Event | kit `dispatch()` + `view()` − `dispatchView()` | share of the old path | old ÷ new | `dispatchView()` − raw `dispatch_view` + `JSON.parse` | `dispatchView()` − raw `dispatch_view_outcome` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 118.0 [117.0–119.8] | 74.5% [74.3–75.0] | 3.93× [3.90–4.00] | +0.4 [-0.1 to +0.8] | +0.4 [-0.1 to +0.7] |
| Glowcap state-changing tick | 107.9 [107.2–109.9] | 71.9% [71.6–72.1] | 3.56× [3.52–3.58] | +1.1 [0.0 to +1.5] | +0.2 [-0.8 to +1.4] |
| Trail Rescue evidence | 149.3 [147.7–155.7] | 51.5% [51.0–52.2] | 2.06× [2.04–2.09] | +1.8 [0.0 to +3.0] | +2.6 [+1.9 to +4.3] |
| Trail Rescue commit | 147.7 [146.8–152.0] | 61.7% [61.5–62.3] | 2.61× [2.60–2.65] | +1.2 [+0.5 to +2.2] | +1.6 [+0.9 to +2.2] |
| Trail Rescue reopen | 151.7 [151.2–158.2] | 48.6% [48.4–49.9] | 1.95× [1.94–1.99] | +2.5 [+1.5 to +4.1] | +1.6 [-1.2 to +3.6] |
| Trail Rescue qualify | 154.4 [151.3–159.2] | 53.5% [52.7–54.0] | 2.15× [2.11–2.17] | +2.3 [+0.7 to +3.4] | +0.5 [-0.1 to +3.5] |
| Ledger evidence | 88.4 [87.5–89.2] | 69.9% [69.7–70.0] | 3.32× [3.30–3.34] | +0.5 [-0.2 to +0.5] | +1.1 [+0.7 to +1.2] |

Refused events (DIRECT, µs; savings paired by repeat). A refusal leaves the
view as it was, so both savings are shown:

| Event | samples per run | existing: kit `dispatch()` + `view()` | existing: kit `dispatch()` alone | new: kit `dispatchView()` | existing: raw `dispatch_view` (throws) | new: raw `dispatch_view_outcome` + `JSON.parse` | kit `dispatch()` + `view()` − `dispatchView()` | kit `dispatch()` alone − `dispatchView()` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Trail Rescue refused | 1320/1320/1320/1320/1320 | 39.0 [38.8–39.6] · p95 53.5 | 10.8 [10.7–10.9] · p95 16.0 | 8.8 [8.7–9.0] · p95 13.5 | 7.3 [7.2–7.4] · p95 13.3 | 7.8 [7.6–7.9] · p95 11.8 | 30.3 [30.0–30.6] | +1.9 [+1.9 to +2.2] |
| Ledger refused | 1500/1500/1500/1500/1500 | 22.7 [22.7–23.1] · p95 28.3 | 8.9 [8.7–9.0] · p95 11.8 | 7.5 [7.4–7.6] · p95 9.0 | 6.9 [6.9–7.3] · p95 8.6 | 6.8 [6.7–7.0] · p95 8.4 | 15.3 [15.2–15.5] | +1.4 [+1.3 to +1.4] |

The existing paths on both builds (DIRECT medians; `opt` is the branch;
the change paired by repeat):

| Event | kit `dispatch()` + `view()`, main | opt | change | raw `dispatch_view` + `JSON.parse`, main | opt | change |
| --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 158.7 [156.6–161.2] · p95 175.3 | 158.3 [157.4–160.5] · p95 176.7 | -0.3% [-1.6 to +0.8] | 39.9 [39.4–40.3] · p95 44.2 | 40.0 [39.6–40.3] · p95 44.4 | +0.5% [-0.7 to +1.5] |
| Glowcap state-changing tick | 150.2 [149.8–153.8] · p95 182.0 | 150.1 [149.7–152.9] · p95 188.3 | -0.3% [-2.0 to +0.2] | 42.0 [40.9–42.5] · p95 51.9 | 41.6 [41.0–41.9] · p95 56.4 | -0.7% [-2.8 to +1.7] |
| Trail Rescue evidence | 290.0 [288.4–293.6] · p95 354.8 | 289.5 [288.1–298.1] · p95 357.4 | 0.0% [-1.0 to +1.5] | 138.8 [137.4–141.1] · p95 177.3 | 138.8 [137.6–142.4] · p95 179.1 | +0.8% [-1.3 to +0.9] |
| Trail Rescue commit | 239.7 [237.8–243.2] · p95 289.8 | 240.1 [238.1–245.0] · p95 288.6 | +0.2% [-1.8 to +1.1] | 91.2 [89.6–93.6] · p95 116.5 | 90.2 [90.0–91.8] · p95 116.0 | -0.5% [-1.9 to +0.7] |
| Trail Rescue reopen | 313.7 [311.7–316.1] · p95 362.8 | 313.3 [311.0–320.0] · p95 360.9 | +0.3% [-1.5 to +1.2] | 158.3 [156.5–160.7] · p95 187.9 | 157.7 [155.6–159.4] · p95 186.6 | -0.4% [-1.7 to +0.8] |
| Trail Rescue qualify | 288.7 [286.3–292.9] · p95 346.9 | 288.7 [286.5–294.8] · p95 351.5 | +0.5% [-1.2 to +0.6] | 132.9 [132.2–134.9] · p95 176.0 | 132.6 [130.6–134.9] · p95 175.6 | -0.2% [-1.7 to 0.0] |
| Ledger evidence | 125.0 [123.8–129.2] · p95 154.5 | 126.4 [125.2–128.0] · p95 154.9 | +0.7% [-0.9 to +1.2] | 37.3 [36.8–38.8] · p95 45.7 | 37.8 [37.2–39.0] · p95 46.0 | +0.5% [0.0 to +1.9] |

### 3.2 Every session

In each session: the saving (kit `dispatch()` + `view()` − `dispatchView()`),
the gap (`dispatchView()` − raw `dispatch_view` + `JSON.parse`), both
MEASURED and paired by repeat, and `dispatchView()` itself (DIRECT median), µs:

| Event | 0x3C00: saving | 0x3C00: gap to raw | 0x3C00: `dispatchView()` | 0x3C00-replicate: saving | 0x3C00-replicate: gap to raw | 0x3C00-replicate: `dispatchView()` | 0x3: saving | 0x3: gap to raw | 0x3: `dispatchView()` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 118.0 [117.0–119.8] | +0.4 [-0.1 to +0.8] | 40.3 [39.9–41.1] | 115.2 [112.2–122.7] | -0.5 [-6.1 to +0.4] | 39.1 [38.5–40.1] | 94.8 [94.1–95.2] | +0.2 [-0.1 to +0.3] | 31.7 [31.6–31.9] |
| Glowcap state-changing tick | 107.9 [107.2–109.9] | +1.1 [0.0 to +1.5] | 42.5 [41.9–43.0] | 108.2 [105.3–112.6] | 0.0 [-3.6 to +0.8] | 40.3 [39.3–41.7] | 87.3 [87.2–87.5] | +0.6 [+0.4 to +0.8] | 33.9 [33.7–33.9] |
| Trail Rescue evidence | 149.3 [147.7–155.7] | +1.8 [0.0 to +3.0] | 140.7 [139.6–142.4] | 145.3 [137.8–151.2] | +3.9 [0.0 to +12.0] | 137.7 [135.2–143.3] | 123.9 [120.5–124.0] | +3.6 [+2.2 to +6.1] | 117.7 [116.3–120.7] |
| Trail Rescue commit | 147.7 [146.8–152.0] | +1.2 [+0.5 to +2.2] | 91.8 [90.6–93.0] | 144.6 [139.3–148.9] | +2.9 [-0.5 to +8.9] | 89.9 [88.2–94.5] | 124.6 [122.7–124.6] | +1.9 [+1.0 to +2.0] | 76.2 [75.7–76.7] |
| Trail Rescue reopen | 151.7 [151.2–158.2] | +2.5 [+1.5 to +4.1] | 161.3 [157.6–161.8] | 147.3 [142.7–152.4] | +4.0 [-0.2 to +9.4] | 156.8 [153.8–159.5] | 127.2 [126.7–128.5] | +3.4 [+2.2 to +4.5] | 133.6 [132.9–135.1] |
| Trail Rescue qualify | 154.4 [151.3–159.2] | +2.3 [+0.7 to +3.4] | 135.0 [134.0–135.8] | 147.9 [140.8–153.9] | +3.8 [-0.6 to +13.6] | 133.1 [129.9–138.8] | 127.9 [125.9–128.8] | +3.7 [+2.6 to +3.8] | 112.9 [112.0–113.8] |
| Ledger evidence | 88.4 [87.5–89.2] | +0.5 [-0.2 to +0.5] | 38.0 [37.6–38.8] | 84.3 [83.6–87.4] | +0.1 [-0.6 to +1.6] | 36.7 [35.6–37.2] | 72.1 [71.8–72.6] | +0.2 [-0.1 to +0.3] | 30.9 [30.6–31.0] |
| Trail Rescue refused | 30.3 [30.0–30.6] | - | 8.8 [8.7–9.0] | 29.2 [29.0–30.3] | - | 8.5 [8.4–8.8] | 25.2 [25.1–25.3] | - | 7.4 [7.3–7.6] |
| Ledger refused | 15.3 [15.2–15.5] | - | 7.5 [7.4–7.6] | 14.6 [14.4–15.3] | - | 7.3 [7.0–7.4] | 12.6 [12.5–12.7] | - | 6.1 [6.0–6.2] |

- 0x3C00: run 2026-09-29T18-34-05-618-view-path, mask 0x3C00, 5 repeats, total CPU during the run mean 7.2%, p95 11.1%, max 15.7%
- 0x3C00-replicate: run 2026-09-29T18-47-25-866-view-path, mask 0x3C00, 5 repeats, total CPU during the run mean 7.1%, p95 10.9%, max 17.4%
- 0x3: run 2026-09-29T18-42-33-481-view-path, mask 0x3, 3 repeats, total CPU during the run mean 7%, p95 9.3%, max 13.2%

The replicate and the processors 0–1 session in full, as the primary's
headline:

Replicate, processors 10–13 (Run 2026-09-29T18-47-25-866-view-path; targets opt (a459995, reactive WebAssembly 0a19e4b85886), main (768275b, reactive WebAssembly e35944503272); mask 0x3C00, priority high; total CPU during the run mean 7.1%, p95 10.9%, max 17.4%):

| Event | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` | saving: kit `dispatch()` + `view()` − `dispatchView()` | old ÷ new | gap: `dispatchView()` − raw `dispatch_view` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 155.3 [151.3–161.2] · p95 181.1 | 39.1 [38.5–40.1] · p95 45.5 | 39.0 [38.7–46.2] · p95 44.6 | 39.1 [38.1–41.9] · p95 47.2 | 115.2 [112.2–122.7] | 3.92× [3.87–4.19] | -0.5 [-6.1 to +0.4] |
| Glowcap state-changing tick | 147.5 [146.8–152.8] · p95 190.4 | 40.3 [39.3–41.7] · p95 50.2 | 40.2 [39.3–45.3] · p95 57.6 | 41.2 [40.3–44.9] · p95 59.6 | 108.2 [105.3–112.6] | 3.69× [3.52–3.80] | 0.0 [-3.6 to +0.8] |
| Trail Rescue evidence | 281.5 [280.3–288.9] · p95 349.5 | 137.7 [135.2–143.3] · p95 177.1 | 133.0 [131.3–137.7] · p95 172.0 | 134.4 [132.7–136.7] · p95 178.0 | 145.3 [137.8–151.2] | 2.07× [1.96–2.10] | +3.9 [0.0 to +12.0] |
| Trail Rescue commit | 234.6 [232.5–238.8] · p95 289.6 | 89.9 [88.2–94.5] · p95 116.4 | 87.0 [85.6–90.4] · p95 112.5 | 88.2 [86.4–89.6] · p95 116.3 | 144.6 [139.3–148.9] | 2.61× [2.48–2.68] | +2.9 [-0.5 to +8.9] |
| Trail Rescue reopen | 304.0 [302.2–309.2] · p95 362.6 | 156.8 [153.8–159.5] · p95 184.3 | 152.0 [149.2–157.0] · p95 179.8 | 153.6 [151.3–156.3] · p95 187.3 | 147.3 [142.7–152.4] | 1.95× [1.90–1.99] | +4.0 [-0.2 to +9.4] |
| Trail Rescue qualify | 282.0 [278.3–287.0] · p95 350.8 | 133.1 [129.9–138.8] · p95 182.4 | 127.8 [125.2–133.7] · p95 172.6 | 129.6 [128.6–132.8] · p95 177.7 | 147.9 [140.8–153.9] | 2.12× [2.02–2.18] | +3.8 [-0.6 to +13.6] |
| Ledger evidence | 121.4 [119.2–124.1] · p95 148.3 | 36.7 [35.6–37.2] · p95 44.9 | 36.2 [35.1–37.3] · p95 44.6 | 35.4 [34.9–36.2] · p95 44.5 | 84.3 [83.6–87.4] | 3.32× [3.27–3.38] | +0.1 [-0.6 to +1.6] |
| Trail Rescue refused | 37.8 [37.6–38.8] · p95 51.6 | 8.5 [8.4–8.8] · p95 13.0 | throws: the call 7.1 [6.9–7.3] · p95 12.9 | 7.5 [7.4–7.7] · p95 11.6 | 29.2 [29.0–30.3] | 4.48× [4.32–4.57] | - |
| Ledger refused | 21.8 [21.6–22.6] · p95 26.8 | 7.3 [7.0–7.4] · p95 8.7 | throws: the call 6.7 [6.5–7.1] · p95 8.4 | 6.6 [6.5–6.8] · p95 8.4 | 14.6 [14.4–15.3] | 3.04× [2.95–3.10] | - |

Processors 0–1 (Run 2026-09-29T18-42-33-481-view-path; targets opt (a459995, reactive WebAssembly 0a19e4b85886), main (768275b, reactive WebAssembly e35944503272); mask 0x3, priority high; total CPU during the run mean 7%, p95 9.3%, max 13.2%):

| Event | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` | saving: kit `dispatch()` + `view()` − `dispatchView()` | old ÷ new | gap: `dispatchView()` − raw `dispatch_view` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 126.5 [125.7–127.1] · p95 145.3 | 31.7 [31.6–31.9] · p95 35.4 | 31.6 [31.4–31.8] · p95 35.3 | 31.7 [31.7–31.8] · p95 35.4 | 94.8 [94.1–95.2] | 3.98× [3.98–3.99] | +0.2 [-0.1 to +0.3] |
| Glowcap state-changing tick | 121.1 [121.0–121.4] · p95 156.1 | 33.9 [33.7–33.9] · p95 40.8 | 33.3 [33.1–33.3] · p95 42.3 | 33.7 [33.6–34.0] · p95 43.8 | 87.3 [87.2–87.5] | 3.58× [3.57–3.59] | +0.6 [+0.4 to +0.8] |
| Trail Rescue evidence | 241.2 [240.2–241.7] · p95 300.3 | 117.7 [116.3–120.7] · p95 160.1 | 114.1 [114.1–114.6] · p95 152.5 | 114.2 [113.9–114.7] · p95 151.2 | 123.9 [120.5–124.0] | 2.05× [2.00–2.06] | +3.6 [+2.2 to +6.1] |
| Trail Rescue commit | 200.3 [199.4–200.8] · p95 245.6 | 76.2 [75.7–76.7] · p95 99.0 | 74.7 [74.2–74.8] · p95 96.0 | 74.4 [74.4–74.7] · p95 96.1 | 124.6 [122.7–124.6] | 2.63× [2.60–2.65] | +1.9 [+1.0 to +2.0] |
| Trail Rescue reopen | 261.8 [260.1–262.1] · p95 304.9 | 133.6 [132.9–135.1] · p95 156.9 | 130.6 [130.2–130.7] · p95 156.4 | 131.3 [131.0–132.6] · p95 155.9 | 127.2 [126.7–128.5] | 1.96× [1.94–1.96] | +3.4 [+2.2 to +4.5] |
| Trail Rescue qualify | 239.9 [239.7–241.7] · p95 297.1 | 112.9 [112.0–113.8] · p95 154.3 | 109.4 [109.2–110.0] · p95 147.6 | 110.9 [110.7–111.8] · p95 148.0 | 127.9 [125.9–128.8] | 2.14× [2.11–2.14] | +3.7 [+2.6 to +3.8] |
| Ledger evidence | 103.0 [102.8–103.2] · p95 127.8 | 30.9 [30.6–31.0] · p95 39.1 | 30.7 [30.6–30.8] · p95 38.4 | 30.2 [30.1–30.3] · p95 37.2 | 72.1 [71.8–72.6] | 3.33× [3.32–3.37] | +0.2 [-0.1 to +0.3] |
| Trail Rescue refused | 32.7 [32.5–32.7] · p95 45.2 | 7.4 [7.3–7.6] · p95 11.6 | throws: the call 6.2 [6.2–6.2] · p95 11.6 | 6.6 [6.4–6.7] · p95 10.0 | 25.2 [25.1–25.3] | 4.42× [4.30–4.45] | - |
| Ledger refused | 18.7 [18.7–18.7] · p95 23.3 | 6.1 [6.0–6.2] · p95 7.3 | throws: the call 5.7 [5.7–5.8] · p95 7.4 | 5.6 [5.6–5.7] · p95 6.9 | 12.6 [12.5–12.7] | 3.07× [3.02–3.12] | - |

- On processors 0–1 every path takes 0.79–0.85× the time it takes on 10–13
  (the baseline measured 0.72–0.78×, its section 7): processors 0 and 1 ran at
  about 5.1 GHz under the benchmark, against about 4.2 GHz for processor 13 in
  the primary session and 4.4 GHz for processor 10 in the replicate. The ratios
  old ÷ new agree across the three sessions within 0.13.
- Inside the mask `0x3C00` Windows placed the primary session's work on
  processor 13 and the replicate's mostly on processor 10 (also 11 and 12),
  and processor 10 ran faster; the replicate's absolute figures are 1–5%
  lower, and its run ranges wider. Savings and ratios agree within their
  ranges.
- main's own paths in this session, against the baseline's primary session
  (the same WebAssembly bytes, `e35944503272…`, on the same mask):

  | Event | kit `dispatch()` + `view()`: baseline primary | this session | change | raw `dispatch_view` + `JSON.parse`: baseline primary | this session | change |
  | --- | --- | --- | --- | --- | --- | --- |
  | Glowcap idle tick | 166.4 | 158.7 | −4.6% | 43.8 | 39.9 | −8.9% |
  | Glowcap state-changing tick | 159.5 | 150.2 | −5.8% | 46.6 | 42.0 | −9.9% |
  | Trail Rescue evidence | 300.8 | 290.0 | −3.6% | 141.7 | 138.8 | −2.0% |
  | Trail Rescue commit | 251.6 | 239.7 | −4.7% | 92.8 | 91.2 | −1.7% |
  | Trail Rescue reopen | 325.2 | 313.7 | −3.5% | 162.6 | 158.3 | −2.6% |
  | Trail Rescue qualify | 302.0 | 288.7 | −4.4% | 136.8 | 132.9 | −2.9% |
  | Ledger evidence | 129.0 | 125.0 | −3.1% | 38.1 | 37.3 | −2.1% |

  The same bytes ran 2–10% faster in this session, inside the baseline's
  session-to-session spread (−9% to +8%). Absolute figures therefore compare
  only within a session, which is why every saving here is paired inside one.

## 4. What remains in the JavaScript wrapper

`dispatchView()` differs from raw `dispatch_view` + `JSON.parse` by −0.5 to
+4.0 µs (at most 3.4%) in every session, so the wrapper is not materially
slower. Its
pieces, each DIRECT (medians, µs, [lowest–highest run]), on processors 10–13
(primary session); the last five rows are MEASURED differences paired by repeat:

| Piece | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | Trail Rescue qualify | Ledger evidence | Trail Rescue refused | Ledger refused |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| raw `dispatch_view` path: `JSON.stringify(payload)` | 0.3 [0.3–0.3] | 0.2 [0.2–0.2] | 0.9 [0.8–0.9] | 0.4 [0.4–0.4] | 0.5 [0.5–0.5] | 0.4 [0.4–0.6] | 0.3 [0.3–0.4] | 0.4 [0.4–0.5] | 0.2 [0.2–0.2] |
| raw `dispatch_view` path: the `dispatch_view` call | 23.9 [23.7–24.2] | 26.4 [25.9–26.6] | 111.8 [110.3–114.6] | 63.9 [63.3–65.0] | 130.5 [127.3–131.6] | 108.2 [105.5–108.9] | 30.4 [29.8–31.4] | 7.3 [7.2–7.4] | 6.9 [6.9–7.3] |
| raw `dispatch_view` path: `JSON.parse` of the view | 15.8 [15.5–15.9] | 15.0 [14.8–15.2] | 25.8 [25.6–26.3] | 25.6 [24.9–25.6] | 26.5 [25.5–26.6] | 24.6 [24.1–24.9] | 6.8 [6.6–7.0] | - | - |
| **raw `dispatch_view` + `JSON.parse`** | 40.0 [39.6–40.3] | 41.6 [41.0–41.9] | 138.8 [137.6–142.4] | 90.2 [90.0–91.8] | 157.7 [155.6–159.4] | 132.6 [130.6–134.9] | 37.8 [37.2–39.0] | - | - |
| raw `dispatch_view_outcome` path: `JSON.stringify(payload)` | 0.3 [0.3–0.3] | 0.3 [0.3–0.3] | 0.8 [0.8–0.9] | 0.4 [0.4–0.4] | 0.5 [0.5–0.6] | 0.4 [0.4–0.5] | 0.3 [0.3–0.3] | 0.4 [0.4–0.5] | 0.2 [0.2–0.2] |
| raw `dispatch_view_outcome` path: the `dispatch_view_outcome` call | 23.8 [23.7–24.0] | 26.7 [25.9–27.0] | 111.2 [110.7–112.1] | 63.3 [62.6–64.5] | 130.1 [129.9–132.4] | 108.9 [107.7–110.1] | 29.5 [29.2–30.1] | 6.0 [6.0–6.1] | 5.9 [5.8–6.0] |
| raw `dispatch_view_outcome` path: `JSON.parse` of the envelope | 15.8 [15.7–16.1] | 15.3 [14.9–15.6] | 25.5 [25.4–26.0] | 25.6 [25.2–26.1] | 26.4 [26.2–26.9] | 24.8 [24.3–25.0] | 6.9 [6.7–7.0] | 0.9 [0.8–0.9] | 0.7 [0.6–0.7] |
| **raw `dispatch_view_outcome` + `JSON.parse`** | 39.9 [39.8–40.4] | 42.3 [41.1–42.9] | 137.7 [137.2–140.0] | 90.2 [89.1–92.1] | 158.8 [157.6–160.4] | 134.3 [132.3–135.7] | 36.9 [36.6–37.7] | 7.8 [7.6–7.9] | 6.8 [6.7–7.0] |
| pieces: the kit's `payloadText(payload)` | 0.5 [0.5–0.5] | 0.5 [0.5–0.6] | 2.3 [2.2–2.3] | 1.1 [1.1–1.2] | 1.3 [1.3–1.4] | 1.2 [1.1–1.2] | 1.1 [1.0–1.1] | 1.1 [1.1–1.2] | 0.7 [0.7–0.7] |
| pieces: the `dispatch_view_outcome` call | 24.5 [24.2–25.6] | 27.4 [26.9–28.4] | 112.0 [111.2–113.1] | 63.9 [63.8–64.9] | 130.8 [129.7–132.5] | 108.4 [107.9–111.2] | 30.1 [29.8–30.5] | 6.2 [6.1–6.2] | 6.1 [6.0–6.1] |
| pieces: `JSON.parse` of the envelope | 16.0 [15.6–16.3] | 15.4 [15.1–15.6] | 24.9 [24.7–25.4] | 23.9 [23.5–24.2] | 24.7 [24.4–24.8] | 22.5 [22.2–22.7] | 6.7 [6.7–6.8] | 0.8 [0.8–0.9] | 0.7 [0.7–0.7] |
| pieces: the kit's `validOutcome(outcome, 'view')` | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.2 [0.2–0.2] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] |
| **pieces: the four, first timer to last** | 41.3 [40.6–42.5] | 43.6 [42.9–44.5] | 139.9 [139.2–141.6] | 89.9 [89.7–91.1] | 158.3 [155.8–159.1] | 132.5 [131.9–135.5] | 38.1 [37.9–38.5] | 8.9 [8.7–9.0] | 7.6 [7.5–7.7] |
| pieces: `JSON.parse` of the view text alone, after the envelope's | 15.2 [14.9–15.5] | 14.5 [14.4–14.7] | 18.4 [18.3–18.6] | 20.2 [19.9–20.4] | 20.9 [20.9–21.2] | 20.0 [19.6–20.1] | 5.4 [5.4–5.5] | - | - |
| **kit `dispatchView()`** | 40.3 [39.9–41.1] | 42.5 [41.9–43.0] | 140.7 [139.6–142.4] | 91.8 [90.6–93.0] | 161.3 [157.6–161.8] | 135.0 [134.0–135.8] | 38.0 [37.6–38.8] | 8.8 [8.7–9.0] | 7.5 [7.4–7.6] |
| `dispatchView()` − the four pieces (paired by repeat; the wrapper's state and argument checks, and the timer reads between pieces) | -1.0 [-1.4 to -0.4] | -1.5 [-1.9 to -0.2] | +0.8 [-0.2 to +1.9] | +1.7 [+0.9 to +2.5] | +2.4 [+1.4 to +4.7] | +1.6 [+0.1 to +3.3] | -0.4 [-0.5 to +0.4] | 0.0 [-0.3 to 0.0] | -0.1 [-0.2 to 0.0] |
| `dispatchView()` − raw `dispatch_view` + `JSON.parse` (paired by repeat) | +0.4 [-0.1 to +0.8] | +1.1 [0.0 to +1.5] | +1.8 [0.0 to +3.0] | +1.2 [+0.5 to +2.2] | +2.5 [+1.5 to +4.1] | +2.3 [+0.7 to +3.4] | +0.5 [-0.2 to +0.5] | - | - |
| the kit's payload check: `payloadText` − `JSON.stringify` of the raw `dispatch_view_outcome` path (paired by repeat) | +0.2 [+0.2 to +0.2] | +0.2 [+0.2 to +0.3] | +1.5 [+1.4 to +1.5] | +0.7 [+0.7 to +0.8] | +0.8 [+0.7 to +0.9] | +0.8 [+0.7 to +0.8] | +0.8 [+0.7 to +0.8] | +0.7 [+0.6 to +0.7] | +0.5 [+0.5 to +0.5] |
| envelope against view: `JSON.parse` of the envelope − of the view, each the first parse on its own raw path (paired by repeat) | +0.2 [-0.2 to +0.3] | +0.4 [-0.1 to +0.5] | -0.3 [-0.5 to 0.0] | +0.1 [-0.3 to +0.7] | +0.2 [-0.1 to +0.7] | +0.1 [-0.2 to +0.5] | 0.0 [0.0 to +0.1] | - | - |
| the call: `dispatch_view_outcome` − `dispatch_view`, each on its own raw path (paired by repeat) | -0.1 [-0.2 to +0.3] | +0.3 [0.0 to +0.7] | -0.8 [-2.5 to +0.4] | -0.6 [-0.7 to -0.4] | +0.8 [-0.6 to +2.8] | +1.2 [-0.3 to +2.2] | -0.6 [-1.3 to -0.6] | -1.3 [-1.4 to -1.1] | -1.1 [-1.3 to -1.0] |

- **The kit's payload check** (`payloadText`: the walk that refuses NaN,
  holes and class instances, then `JSON.stringify`) costs 0.2 µs more than
  `JSON.stringify` on a tick's `{dt}` and 0.5–1.5 µs more on the ledger's and
  Trail Rescue's payloads (0.3–1.3 µs in the other sessions). It is the
  largest attributed part of the gap.
- **`validOutcome`**: 0.1–0.2 µs.
- **The envelope**: `JSON.parse` of the outcome envelope and of the bare view,
  each the first parse on its own raw path, differ by −0.4 to +0.5 µs; the
  `dispatch_view_outcome` and `dispatch_view` calls differ by −0.8 to +1.8 µs
  on accepted events (paired medians, all sessions). Neither is measurable.
  On refusals the new export is 0.9–1.3 µs faster than the legacy
  `dispatch_view`, which throws.
- **Not attributed**: on Trail Rescue's accepted events `dispatchView()` is
  0.8–4.0 µs (medians per session) above the sum of its four pieces, and on
  Glowcap, the ledger and refusals within ±1.5 µs of it. What `dispatchView()`
  runs outside the four pieces is a state check, a method check and an event
  type check, each well under 0.1 µs; the residual is more likely process
  state (JIT and garbage collection) that differs between the two processes,
  which this method cannot separate. It is at most 3% of those events.
- `JSON.parse` of the view text alone, timed right after the envelope's parse
  of the same bytes, is 0.5–7.1 µs cheaper than either first parse. That is the
  order (a second parse of warm bytes), not the envelope: compare the first
  parses above.

## 5. Size

`size.mjs`: each tree's `npm run build` output, and the kit staged and packed
as `scripts/test-kit-package.mjs` stages it (`results/size.json`). The
branch's checkout then held this commit's three uncommitted harness files
under `experiments/`, which the kit does not pack (`size.json` counts them as
`changedFiles`).

|  | base 768275b | branch | change |
| --- | --- | --- | --- |
| reactive WebAssembly (`pkg-reactive`, bundled in the kit) | 2,052,856 | 2,057,234 | +4378 |
|   gzip -9 | 578,371 | 579,623 | +1252 |
|   its JavaScript glue | 14,575 | 15,854 | +1279 |
| full WebAssembly (`pkg`, the web pages) | 2,307,079 | 2,311,521 | +4442 |
|   gzip -9 | 653,470 | 654,388 | +918 |
|   its JavaScript glue | 33,840 | 35,119 | +1279 |
| kit `lib/session.mjs` | 10,461 | 12,008 | +1547 |
| kit tarball (`npm pack`) | 718,097 | 720,703 | +2606 |
|   unpacked | 2,502,572 | 2,514,416 | +11844 |
|   files in it | 59 | 59 | 0 |

## 6. Correctness evidence

Correctness evidence, not measurements. The implementation's own evidence
([correctness/](correctness/README.md)), and what this session repeated or
added:

- **Existing exports unchanged.** `correctness/legacy-identity.mjs`, rerun
  here against this session's `npm run build` of main 768275b: over 19,760
  events of 80 programs and the Glowcap, ledger and Trail Rescue streams, in
  both builds (`pkg-reactive` and `pkg`), `dispatch_outcome`, the legacy
  `dispatch_view` and the legacy `dispatch` returned or threw the same text,
  and `snapshot()`, `view()` and `save()` after every event were the same
  text (59,280 dispatch calls and 177,429 reads per build). Its output is
  byte-identical to the committed `legacy-identity.json` (the same WebAssembly
  hashes).
- **The differential and the kit's tests.** `npm run test:kit` on the branch
  build: 136 of 136 pass, among them the twins over every tracked program that
  opens (80 programs, 9,600 random events: 2,988 accepted, 6,475 rejected over
  seven codes, 137 fatal) and over the Glowcap replay, Glowcap with 16
  mushrooms, the ledger and Trail Rescue, comparing outcome, view, save and
  snapshot after every event; and the refusal, fatal, trap, closed-session and
  missing-export tests of `dispatch-view.test.mjs`.
- **Every timed session.** Every mode of both builds ended every episode of
  each workload in the same saved state: one save hash per workload across
  the two targets and all their modes and repeats (Glowcap `fda3d610…`,
  ledger `89569852…`, Trail Rescue `bb72a4e7…`), in all three sessions, with
  no correctness finding. The pieces mode checked for every accepted event
  that the outcome is exactly `{"schema":"caveat-dispatch/0.1","outcome":"accepted","view":`
  + view + `}` and that `validOutcome` accepts every outcome.
- The kit, the runtime and every existing test are as the implementation
  left them: this commit changes files under `experiments/` only.

## 7. Load, and the sessions not used

Per repeat, total CPU over the 24 logical processors while the repeat ran
(the benchmark itself about 5%):

Primary, processors 10–13:

| Repeat | from (UTC) | seconds | samples | total CPU mean | p95 | max | samples above 20% | kept |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 18:34:15 | 82.2 | 81 | 7.3% | 10.7% | 12.6% | 0 | yes |
| 2 | 18:35:37 | 81 | 80 | 7.0% | 9.7% | 12.4% | 0 | yes |
| 3 | 18:36:58 | 80.9 | 81 | 7.5% | 12.4% | 15.7% | 0 | yes |
| 4 | 18:38:19 | 80.6 | 80 | 7.2% | 11.1% | 13.3% | 0 | yes |
| 5 | 18:39:39 | 80.3 | 80 | 7.0% | 11.1% | 15.1% | 0 | yes |

Replicate, processors 10–13:

| Repeat | from (UTC) | seconds | samples | total CPU mean | p95 | max | samples above 20% | kept |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 18:47:33 | 81 | 80 | 7.2% | 11.0% | 17.4% | 0 | yes |
| 2 | 18:48:54 | 78.9 | 78 | 7.0% | 9.9% | 12.8% | 0 | yes |
| 3 | 18:50:13 | 78.8 | 78 | 7.1% | 11.0% | 16.5% | 0 | yes |
| 4 | 18:51:32 | 78.9 | 79 | 7.0% | 10.5% | 13.2% | 0 | yes |
| 5 | 18:52:51 | 78.7 | 78 | 7.0% | 10.2% | 13.2% | 0 | yes |

Processors 0–1:

| Repeat | from (UTC) | seconds | samples | total CPU mean | p95 | max | samples above 20% | kept |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 18:42:41 | 68.4 | 67 | 7.0% | 9.8% | 11.3% | 0 | yes |
| 2 | 18:43:49 | 68.4 | 68 | 7.1% | 9.2% | 13.2% | 0 | yes |
| 3 | 18:44:58 | 68.4 | 68 | 6.9% | 9.0% | 13.2% | 0 | yes |

In the kept sessions `MsMpEng` used at most 15% of one core, 2.4–2.7% on
average (`processes.csv`).

Three earlier sessions were discarded and are not used anywhere
([results/excluded/](results/excluded/README.md)):

- **A** (17:52–17:59 UTC): mean 12.7%, every repeat above 12%. Windows
  Defender's scanner was using about 0.9 of a core, most likely because an
  orphaned `ls -R .cache` left by another session (running since 14:48 UTC)
  opened about 250 files a second; my own process watcher added about 1%.
  Both the `ls` and the scanning had stopped by 18:15. A gate on total CPU
  alone had passed (one busy core is 4% of 24), so the later gates also
  watched `MsMpEng`.
- **P1** (18:16–18:23): a burst of up to 33% in repeat 2 (Defender and the
  System process).
- **P2** (18:25–18:32): a burst of up to 52% for 4 s in repeat 1 as the
  ChatGPT desktop app exited, during the branch's `kit` Glowcap job. P2 had
  passed the rule's first version (at most 5% of samples above 20%); the rule
  was tightened to none, and P2 discarded, before the kept sessions ran.

## 8. Noticed for the next decision

Recorded, not acted on: this work changed nothing but the harness.

1. **The next cost on the view path is the view's JSON round trip.** On a
   Glowcap idle tick `JSON.parse` of the view is 15.8 of 40.3 µs, and the
   24 µs `dispatch_view_outcome` call builds and serializes the whole view
   (4.5 KB). Yet on 9,393 of the replay's 9,394 idle ticks the new view
   differs from the one before only in its top-level `sequence` (a
   throwaway untimed check on the branch build, not committed). A host that forwards the view as text, or
   only needs what changed, pays for the rest. A text variant (as
   `viewText()` is to `view()`) or a changed-parts signal would remove it;
   neither is built.
2. **On decision events the language dominates.** On Trail Rescue the call is
   64–131 µs of 90–161 µs, and `JSON.parse` 22–26 µs; a further gain there
   needs the runtime itself (the baseline's section 10), not the wrapper.
3. **The kit's payload check is cheap**: 0.2–1.5 µs per event. Nothing to do.
4. **A refusal through the new export is about 1 µs cheaper than through the
   legacy `dispatch_view`**, which throws.
5. **Core placement inside a mask moves absolute figures by a few percent**
   (processor 13 at about 4.2 GHz against 10 at 4.4 GHz here): comparisons
   belong inside one interleaved session, as the savings here are.
6. **A total-CPU gate does not see one busy core.** Windows Defender scanning
   a stream of opened files (here, an orphaned recursive `ls` from another
   session) held a core while total CPU stayed under 12%. A measurer should
   also watch `MsMpEng`, and should not run recursive `ls` over large trees
   on this machine while others measure.
7. **`dispatchView()` is new in 0.1.0-rc.5, unreleased.** Hosts on rc.4 keep
   `dispatch()` + `view()`, or the raw `dispatch_view` + `JSON.parse`, which
   already costs what `dispatchView()` costs.

## 9. Raw results and how to rerun

In [`results/`](results/) (large JSON gzipped; every script here reads the
`.gz` files directly):

| Directory / file | What |
| --- | --- |
| `view-path-0x3C00/` | the primary session: `results.json.gz` (every run's summaries, environment, targets with hashes, inputs, each job's processors and clock), `classes.json.gz` (per event class), `summary.md`, `run-log.txt`, `load.csv` (typeperf every second), `load.json` (per repeat), `processes.csv` (`MsMpEng` and `System` every 5 s), `quiet-gate.txt` (the minute of quiet before the session, local time, UTC−7) |
| `view-path-0x3C00-replicate/` | the replicate session, same files |
| `view-path-0x3/` | the processors 0–1 session, same files |
| `size.json` | the size report |
| `tables.md`, `tables-0x3C00-replicate.md`, `tables-0x3.md` | every table above, rendered by `report.mjs` from those files |
| `excluded/` | the index of the discarded sessions, whose raw files are not committed |

The per-event samples (about 6.5 MB per session, gzipped) are not committed.
On a quiet machine, from a checkout of this branch built with `npm run build`,
and a checkout of main 768275b built the same way:

```sh
# Each session (then again with --affinity=0x3 --repeats=3 for processors 0-1)
node experiments/performance-0.1/run.mjs --suite=view-path --engines=wasm --build=auto --keep-samples \
  --repeats=5 --monitor-interval=1 --affinity=0x3C00 --priority=high \
  --target=opt=tree:. --target=main=tree:PATH/TO/main-768275b --out=DIR
node experiments/performance-0.1/analyze.mjs DIR          # classes.json
node experiments/performance-opt-0.1/load.mjs DIR         # the keep rule, per repeat
# Size, then the tables
node experiments/performance-opt-0.1/size.mjs --base=PATH/TO/main-768275b,PATH/TO/main-768275b/dist \
  --branch=.,dist --out=experiments/performance-opt-0.1/results/size.json
node experiments/performance-opt-0.1/report.mjs --run=experiments/performance-opt-0.1/results/view-path-0x3C00 \
  --size=experiments/performance-opt-0.1/results/size.json \
  --replicate=0x3C00=experiments/performance-opt-0.1/results/view-path-0x3C00 \
  --replicate=0x3C00-replicate=experiments/performance-opt-0.1/results/view-path-0x3C00-replicate \
  --replicate=0x3=experiments/performance-opt-0.1/results/view-path-0x3 \
  --out=experiments/performance-opt-0.1/results/tables.md
```
