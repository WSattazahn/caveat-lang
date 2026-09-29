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
(19:48–19:58 UTC). As the baseline found, the performance cores of this
laptop are not interchangeable, so absolute figures hold for their processors
and session; ratios and savings in µs are given for every session. Microseconds
(µs) throughout. A cell `38.9 [37.6–40.1] · p95 43.5` is the median over the
repeats of each repeat's median, the lowest and highest repeat, then the
median of the repeats' p95.

Every timed job here was judged by its own load, second by second, as it
ended, and run again in its place if it overlapped a burst of background load
(section 2); 32 such attempts were discarded across the three sessions, and no
figure here comes from one.

## Headline

Per event class, the existing kit path (`dispatch()` then `view()`), the new
kit path (`dispatchView()`), the web pages' raw path (`dispatch_view` then
`JSON.parse`) and the new export's raw path (`dispatch_view_outcome` then
`JSON.parse`), and the saving. Primary session, processors 10–13, 5 repeats.

| Event | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` | saving: kit `dispatch()` + `view()` − `dispatchView()` | old ÷ new | gap: `dispatchView()` − raw `dispatch_view` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 149.5 [147.6–158.5] · p95 166.9 | 38.9 [37.6–40.1] · p95 43.5 | 37.7 [36.8–39.8] · p95 43.9 | 37.9 [37.0–41.0] · p95 42.9 | 110.2 [109.2–119.6] | 3.84× [3.72–4.08] | +0.5 [-0.9 to +1.9] |
| Glowcap state-changing tick | 143.2 [140.1–146.0] · p95 177.7 | 40.4 [39.3–42.2] · p95 47.9 | 39.1 [37.2–42.3] · p95 51.1 | 39.8 [37.8–43.0] · p95 52.0 | 102.9 [99.7–103.8] | 3.51× [3.46–3.64] | +1.4 [-1.3 to +3.2] |
| Trail Rescue evidence | 271.7 [268.7–279.2] · p95 326.0 | 133.3 [131.8–138.5] · p95 173.3 | 131.5 [128.6–133.3] · p95 169.7 | 132.5 [128.2–133.5] · p95 171.6 | 139.7 [136.5–144.8] | 2.03× [2.01–2.08] | +2.6 [-1.2 to +7.0] |
| Trail Rescue commit | 225.5 [223.2–232.9] · p95 269.7 | 87.7 [85.7–90.3] · p95 112.7 | 85.9 [83.9–87.3] · p95 110.2 | 87.2 [84.2–87.7] · p95 113.7 | 139.8 [136.7–143.0] | 2.58× [2.56–2.63] | +1.8 [-1.3 to +5.0] |
| Trail Rescue reopen | 293.4 [291.5–300.5] · p95 342.2 | 152.6 [150.0–156.4] · p95 180.7 | 150.1 [147.0–151.3] · p95 178.8 | 151.6 [148.1–153.5] · p95 179.6 | 141.8 [140.8–147.3] | 1.94× [1.92–1.96] | +3.0 [-0.7 to +6.3] |
| Trail Rescue qualify | 270.3 [267.9–280.8] · p95 329.6 | 129.4 [126.5–133.0] · p95 172.3 | 126.7 [123.8–128.4] · p95 168.5 | 128.0 [125.9–131.0] · p95 174.1 | 143.6 [140.8–150.1] | 2.12× [2.09–2.16] | +2.8 [-1.7 to +6.3] |
| Ledger evidence | 118.4 [117.9–121.4] · p95 142.7 | 36.2 [35.0–36.8] · p95 44.2 | 35.2 [34.7–36.4] · p95 43.1 | 34.6 [34.1–35.2] · p95 42.0 | 82.1 [81.5–85.4] | 3.28× [3.21–3.44] | +0.7 [-0.2 to +1.8] |
| Trail Rescue refused | 36.2 [35.8–37.8] · p95 48.6 | 8.3 [8.1–8.6] · p95 12.7 | throws: the call 6.9 [6.7–7.0] · p95 12.2 | 7.4 [7.3–7.4] · p95 11.5 | 28.0 [27.5–29.2] | 4.42× [4.31–4.46] | - |
| Ledger refused | 21.5 [21.4–22.2] · p95 26.4 | 7.0 [6.9–7.3] · p95 8.4 | throws: the call 6.5 [6.4–6.7] · p95 8.0 | 6.4 [6.3–6.6] · p95 8.0 | 14.5 [14.2–15.2] | 3.07× [2.94–3.17] | - |

- **The saving is 82–144 µs per accepted event** on processors 10–13
  (MEASURED, paired by repeat): the old path takes 1.94× (Trail Rescue
  reopen) to 3.84× (Glowcap idle tick) as long as the new one, so 49–74% of
  its time is removed. The replicate session gives 81–142 µs (1.92–3.94×), and
  processors 0–1 give 71–128 µs (1.95–3.98×): the µs scale with the core,
  the ratios do not move (section 3).
- **`dispatchView()` costs about what the web pages' path costs.** Paired by
  repeat, it is +0.5 to +1.4 µs above raw `dispatch_view` + `JSON.parse` on
  Glowcap and the ledger in the primary session (−0.1 to +1.4 µs in every
  session), and +1.8 to +3.0 µs (2.0–2.2%) on Trail Rescue's accepted events;
  the replicate gives +2.7 to +6.5 µs (2.6–5.2%) there and processors 0–1
  +2.3 to +4.4 µs (3.0–4.1%). Section 4 takes the gap apart: the kit's
  payload check (0.2–0.3 µs on a tick, 0.3–1.3 µs on the ledger's and Trail
  Rescue's payloads) and `validOutcome` (0.1–0.2 µs); `JSON.parse` of the
  envelope is within ±0.5 µs of the bare view's; the new export's call is
  within −0.9 to +0.5 µs of `dispatch_view`'s on Glowcap and the ledger, but
  on Trail Rescue it differs by −0.4 to +2.7 µs (+0.8 to +2.7 µs on
  processors 0–1, where the ranges are tightest), most on reopen and qualify;
  and 1.0–5.3 µs on Trail Rescue is not attributed.
- **A refused event** costs 8.3 µs (Trail Rescue) and 7.0 µs (ledger) through
  `dispatchView()`, against 36.2 and 21.5 µs for `dispatch()` + `view()`, and
  10.0 and 8.4 µs for `dispatch()` alone. A refusal leaves the view as it was,
  so a host that already holds it saves 1.4–1.7 µs against `dispatch()`
  alone, and 14–28 µs against `dispatch()` + `view()`.
- **The existing paths are unchanged.** Paired by repeat, main against the
  branch, kit `dispatch()` + `view()` and raw `dispatch_view` + `JSON.parse`
  differ by −0.8% to +1.3% (medians) in all three sessions, within their
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
  of a459995. During timing its checkout was at d5938cc (a459995 and the
  first measurement commit, which changed only `experiments/`) with this
  commit's uncommitted harness changes, so the harness records it as
  `v0.1.0-rc.4-15-gd5938cc-dirty`; its `runtime/` and `kit/lib/` trees are
  a459995's (`836b9c2e…` and `b1882a44…`, recorded per run), and the built
  bytes are the same files as in the table.
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
- **Quiet machine.** Before each session a bounded gate (at most 20 minutes)
  waited for a minute (12 samples, 5 s apart) with total CPU below 12% and
  Windows Defender's scanner (`MsMpEng`) below 25% of one core; the three
  gates read a total of 2.1–3.1% on average (at most 5.4%). During each
  session the harness's monitor sampled, every second, total CPU, the pinned
  processors' use and clock, and the `System` process and `MsMpEng` in percent
  of one core (`load.csv`), starting 5 s before the first job
  (`--monitor-lead=5`).
- **The job guard** (`run.mjs --job-guard`, `lib/guard.mjs`). As each timed
  job ends, the harness judges it by the samples covering it, from the one
  covering its start to the one covering its end. It is kept when its mean
  total CPU is at most 12% and no sample is above 20% (the per-repeat rule,
  applied per job), and when in no sample `System` or `MsMpEng` uses more than
  30% of one core or more than 2 cores are busy outside the pinned processors
  (total CPU × 24 − the pinned processors' use). Otherwise the attempt is
  discarded, not kept as a replicate: the harness waits for 3 quiet samples in
  a row and runs the same job again in its place, before the next job, so the
  interleaving and pairing hold (up to 5 attempts; none needed more than 3).
  The thresholds were set before these sessions, from the load recorded in
  the first measurement and a 150 s idle trace: on this laptop the `System`
  process takes about one core for 3–4 s about once a minute (up to 114% of
  a core in a second; below 16% otherwise), and Defender scans in bursts. `load.mjs` then
  judges every kept job again from `load.csv` (all pass; the guard's verdict
  agrees for every one) and keeps a repeat when, over the samples covering its
  kept jobs, the mean total CPU is at most 12% and no sample is above 20%; a
  session is used only when every repeat and every job is kept. The benchmark
  itself is about 5% of the total.
- **Order.** Primary (processors 10–13) 19:48:35–19:58:49 UTC; processors 0–1
  20:00:02–20:05:38; replicate (processors 10–13) 20:06:57–20:16:22.

## 3. Results

### 3.1 Processors 10–13, primary session

Per event, on each path (DIRECT, µs):

| Event | samples per run | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 28182/28182/28182/28182/28182 | 149.5 [147.6–158.5] · p95 166.9 | 38.9 [37.6–40.1] · p95 43.5 | 37.7 [36.8–39.8] · p95 43.9 | 37.9 [37.0–41.0] · p95 42.9 |
| Glowcap state-changing tick | 1809/1809/1809/1809/1809 | 143.2 [140.1–146.0] · p95 177.7 | 40.4 [39.3–42.2] · p95 47.9 | 39.1 [37.2–42.3] · p95 51.1 | 39.8 [37.8–43.0] · p95 52.0 |
| Trail Rescue evidence | 2640/2640/2640/2640/2640 | 271.7 [268.7–279.2] · p95 326.0 | 133.3 [131.8–138.5] · p95 173.3 | 131.5 [128.6–133.3] · p95 169.7 | 132.5 [128.2–133.5] · p95 171.6 |
| Trail Rescue commit | 1440/1440/1440/1440/1440 | 225.5 [223.2–232.9] · p95 269.7 | 87.7 [85.7–90.3] · p95 112.7 | 85.9 [83.9–87.3] · p95 110.2 | 87.2 [84.2–87.7] · p95 113.7 |
| Trail Rescue reopen | 600/600/600/600/600 | 293.4 [291.5–300.5] · p95 342.2 | 152.6 [150.0–156.4] · p95 180.7 | 150.1 [147.0–151.3] · p95 178.8 | 151.6 [148.1–153.5] · p95 179.6 |
| Trail Rescue qualify | 360/360/360/360/360 | 270.3 [267.9–280.8] · p95 329.6 | 129.4 [126.5–133.0] · p95 172.3 | 126.7 [123.8–128.4] · p95 168.5 | 128.0 [125.9–131.0] · p95 174.1 |
| Ledger evidence | 4500/4500/4500/4500/4500 | 118.4 [117.9–121.4] · p95 142.7 | 36.2 [35.0–36.8] · p95 44.2 | 35.2 [34.7–36.4] · p95 43.1 | 34.6 [34.1–35.2] · p95 42.0 |

The saving and the gap to the raw paths (MEASURED, paired by repeat, µs):

| Event | kit `dispatch()` + `view()` − `dispatchView()` | share of the old path | old ÷ new | `dispatchView()` − raw `dispatch_view` + `JSON.parse` | `dispatchView()` − raw `dispatch_view_outcome` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 110.2 [109.2–119.6] | 74.0% [73.1–75.5] | 3.84× [3.72–4.08] | +0.5 [-0.9 to +1.9] | +0.2 [-2.1 to +1.7] |
| Glowcap state-changing tick | 102.9 [99.7–103.8] | 71.5% [71.1–72.5] | 3.51× [3.46–3.64] | +1.4 [-1.3 to +3.2] | +1.7 [-2.0 to +2.6] |
| Trail Rescue evidence | 139.7 [136.5–144.8] | 50.8% [50.2–51.9] | 2.03× [2.01–2.08] | +2.6 [-1.2 to +7.0] | +1.6 [-0.7 to +5.4] |
| Trail Rescue commit | 139.8 [136.7–143.0] | 61.2% [60.9–62.0] | 2.58× [2.56–2.63] | +1.8 [-1.3 to +5.0] | +1.3 [-2.0 to +3.0] |
| Trail Rescue reopen | 141.8 [140.8–147.3] | 48.5% [47.9–49.0] | 1.94× [1.92–1.96] | +3.0 [-0.7 to +6.3] | +1.8 [-1.5 to +2.9] |
| Trail Rescue qualify | 143.6 [140.8–150.1] | 52.8% [52.1–53.6] | 2.12× [2.09–2.16] | +2.8 [-1.7 to +6.3] | +1.9 [-4.3 to +3.1] |
| Ledger evidence | 82.1 [81.5–85.4] | 69.5% [68.9–70.9] | 3.28× [3.21–3.44] | +0.7 [-0.2 to +1.8] | +1.3 [-0.2 to +2.7] |

Refused events (DIRECT, µs; savings paired by repeat). A refusal leaves the
view as it was, so both savings are shown:

| Event | samples per run | existing: kit `dispatch()` + `view()` | existing: kit `dispatch()` alone | new: kit `dispatchView()` | existing: raw `dispatch_view` (throws) | new: raw `dispatch_view_outcome` + `JSON.parse` | kit `dispatch()` + `view()` − `dispatchView()` | kit `dispatch()` alone − `dispatchView()` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Trail Rescue refused | 1320/1320/1320/1320/1320 | 36.2 [35.8–37.8] · p95 48.6 | 10.0 [9.8–10.1] · p95 14.5 | 8.3 [8.1–8.6] · p95 12.7 | 6.9 [6.7–7.0] · p95 12.2 | 7.4 [7.3–7.4] · p95 11.5 | 28.0 [27.5–29.2] | +1.7 [+1.5 to +1.8] |
| Ledger refused | 1500/1500/1500/1500/1500 | 21.5 [21.4–22.2] · p95 26.4 | 8.4 [8.3–8.7] · p95 10.7 | 7.0 [6.9–7.3] · p95 8.4 | 6.5 [6.4–6.7] · p95 8.0 | 6.4 [6.3–6.6] · p95 8.0 | 14.5 [14.2–15.2] | +1.4 [+1.0 to +1.7] |

The existing paths on both builds (DIRECT medians; `opt` is the branch;
the change paired by repeat):

| Event | kit `dispatch()` + `view()`, main | opt | change | raw `dispatch_view` + `JSON.parse`, main | opt | change |
| --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 150.7 [149.0–154.2] · p95 172.5 | 149.5 [147.6–158.5] · p95 166.9 | -0.8% [-0.9 to +2.8] | 37.5 [37.1–40.0] · p95 41.5 | 37.7 [36.8–39.8] · p95 43.9 | -0.5% [-0.8 to +6.1] |
| Glowcap state-changing tick | 145.4 [140.7–159.5] · p95 184.3 | 143.2 [140.1–146.0] · p95 177.7 | -0.5% [-9.8 to +1.6] | 39.5 [39.1–41.7] · p95 56.1 | 39.1 [37.2–42.3] · p95 51.1 | 0.0% [-5.8 to +1.4] |
| Trail Rescue evidence | 272.9 [270.5–277.8] · p95 331.1 | 271.7 [268.7–279.2] · p95 326.0 | +0.2% [-0.7 to +1.6] | 129.9 [128.2–133.9] · p95 169.1 | 131.5 [128.6–133.3] · p95 169.7 | +0.6% [-1.8 to +3.7] |
| Trail Rescue commit | 226.5 [224.7–230.5] · p95 269.5 | 225.5 [223.2–232.9] · p95 269.7 | -0.1% [-0.7 to +2.2] | 85.2 [83.8–87.0] · p95 111.0 | 85.9 [83.9–87.3] · p95 110.2 | +0.8% [-2.0 to +3.8] |
| Trail Rescue reopen | 294.9 [293.8–300.1] · p95 338.5 | 293.4 [291.5–300.5] · p95 342.2 | -0.1% [-1.2 to +1.2] | 148.5 [145.8–152.6] · p95 178.3 | 150.1 [147.0–151.3] · p95 178.8 | +0.5% [-1.6 to +3.4] |
| Trail Rescue qualify | 273.0 [270.6–278.2] · p95 332.4 | 270.3 [267.9–280.8] · p95 329.6 | -0.1% [-1.3 to +2.0] | 124.1 [122.4–128.2] · p95 167.7 | 126.7 [123.8–128.4] · p95 168.5 | +0.6% [-1.2 to +4.9] |
| Ledger evidence | 118.7 [116.7–123.3] · p95 144.3 | 118.4 [117.9–121.4] · p95 142.7 | +0.6% [-1.5 to +1.5] | 34.9 [34.6–37.3] · p95 42.8 | 35.2 [34.7–36.4] · p95 43.1 | +0.3% [-2.4 to +1.1] |

### 3.2 Every session

In each session: the saving (kit `dispatch()` + `view()` − `dispatchView()`),
the gap (`dispatchView()` − raw `dispatch_view` + `JSON.parse`), both
MEASURED and paired by repeat, and `dispatchView()` itself (DIRECT median), µs:

| Event | 0x3C00: saving | 0x3C00: gap to raw | 0x3C00: `dispatchView()` | 0x3C00-replicate: saving | 0x3C00-replicate: gap to raw | 0x3C00-replicate: `dispatchView()` | 0x3: saving | 0x3: gap to raw | 0x3: `dispatchView()` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 110.2 [109.2–119.6] | +0.5 [-0.9 to +1.9] | 38.9 [37.6–40.1] | 112.3 [109.2–115.4] | +0.4 [+0.2 to +1.4] | 37.9 [37.3–38.7] | 93.5 [93.0–94.0] | -0.1 [-0.3 to +0.4] | 31.5 [31.4–31.9] |
| Glowcap state-changing tick | 102.9 [99.7–103.8] | +1.4 [-1.3 to +3.2] | 40.4 [39.3–42.2] | 103.2 [101.0–108.2] | +0.7 [-0.4 to +1.5] | 39.9 [39.2–40.6] | 86.3 [85.7–86.8] | 0.0 [-0.2 to +0.2] | 33.3 [33.2–33.6] |
| Trail Rescue evidence | 139.7 [136.5–144.8] | +2.6 [-1.2 to +7.0] | 133.3 [131.8–138.5] | 138.3 [131.0–142.8] | +3.3 [+1.4 to +10.0] | 135.5 [131.9–138.5] | 122.9 [120.8–123.8] | +3.7 [+3.2 to +4.4] | 115.9 [115.6–117.0] |
| Trail Rescue commit | 139.8 [136.7–143.0] | +1.8 [-1.3 to +5.0] | 87.7 [85.7–90.3] | 138.0 [133.7–140.0] | +2.7 [+1.5 to +7.0] | 88.9 [86.3–90.4] | 123.1 [121.2–123.4] | +2.3 [+1.9 to +2.5] | 75.4 [75.2–76.0] |
| Trail Rescue reopen | 141.8 [140.8–147.3] | +3.0 [-0.7 to +6.3] | 152.6 [150.0–156.4] | 142.4 [135.9–143.9] | +5.0 [+2.3 to +9.4] | 154.5 [149.7–155.4] | 126.2 [123.9–126.9] | +3.8 [+3.1 to +5.0] | 132.4 [132.3–132.6] |
| Trail Rescue qualify | 143.6 [140.8–150.1] | +2.8 [-1.7 to +6.3] | 129.4 [126.5–133.0] | 140.9 [134.1–144.9] | +6.5 [+1.6 to +11.5] | 131.7 [126.5–133.8] | 127.7 [125.8–127.8] | +4.4 [+3.8 to +5.4] | 111.9 [111.0–112.7] |
| Ledger evidence | 82.1 [81.5–85.4] | +0.7 [-0.2 to +1.8] | 36.2 [35.0–36.8] | 81.3 [79.2–82.7] | +0.1 [-1.1 to +1.0] | 35.6 [34.9–35.9] | 70.7 [70.7–71.8] | +0.5 [+0.1 to +0.6] | 30.6 [30.5–30.7] |
| Trail Rescue refused | 28.0 [27.5–29.2] | - | 8.3 [8.1–8.6] | 28.4 [27.6–28.5] | - | 8.4 [8.3–8.6] | 24.6 [24.3–24.8] | - | 7.4 [7.3–7.4] |
| Ledger refused | 14.5 [14.2–15.2] | - | 7.0 [6.9–7.3] | 14.2 [14.0–14.4] | - | 7.0 [6.8–7.1] | 12.5 [12.4–12.5] | - | 6.0 [6.0–6.1] |

- 0x3C00: run 2026-09-29T19-48-35-164-view-path, mask 0x3C00, 5 repeats, total CPU over the kept timed jobs mean 5.5%, highest repeat p95 8.7%, max 10.7%; 14 attempt(s) discarded by the job guard and run again
- 0x3C00-replicate: run 2026-09-29T20-06-57-466-view-path, mask 0x3C00, 5 repeats, total CPU over the kept timed jobs mean 5.4%, highest repeat p95 8.6%, max 10.4%; 9 attempt(s) discarded by the job guard and run again
- 0x3: run 2026-09-29T20-00-02-459-view-path, mask 0x3, 3 repeats, total CPU over the kept timed jobs mean 5.4%, highest repeat p95 10.7%, max 12.4%; 9 attempt(s) discarded by the job guard and run again

The replicate and the processors 0–1 session in full, as the primary's
headline:

Replicate, processors 10–13 (Run 2026-09-29T20-06-57-466-view-path; targets opt (d5938cc, reactive WebAssembly 0a19e4b85886), main (768275b, reactive WebAssembly e35944503272); mask 0x3C00, priority high; total CPU over the kept timed jobs mean 5.4%, highest repeat p95 8.6%, max 10.4%; 9 attempt(s) discarded by the job guard and run again):

| Event | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` | saving: kit `dispatch()` + `view()` − `dispatchView()` | old ÷ new | gap: `dispatchView()` − raw `dispatch_view` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 150.5 [147.8–153.3] · p95 174.8 | 37.9 [37.3–38.7] · p95 41.5 | 37.3 [36.9–38.0] · p95 41.8 | 37.7 [37.1–37.8] · p95 42.9 | 112.3 [109.2–115.4] | 3.94× [3.82–4.06] | +0.4 [+0.2 to +1.4] |
| Glowcap state-changing tick | 142.4 [141.6–148.1] · p95 176.3 | 39.9 [39.2–40.6] · p95 45.2 | 39.5 [38.4–39.7] · p95 53.7 | 39.6 [38.0–40.1] · p95 55.1 | 103.2 [101.0–108.2] | 3.62× [3.49–3.71] | +0.7 [-0.4 to +1.5] |
| Trail Rescue evidence | 274.5 [269.5–275.6] · p95 329.9 | 135.5 [131.9–138.5] · p95 178.3 | 131.1 [128.5–132.4] · p95 167.6 | 130.7 [128.3–132.1] · p95 168.2 | 138.3 [131.0–142.8] | 2.03× [1.95–2.08] | +3.3 [+1.4 to +10.0] |
| Trail Rescue commit | 227.1 [224.0–228.4] · p95 272.6 | 88.9 [86.3–90.4] · p95 117.0 | 85.6 [83.3–86.2] · p95 109.5 | 86.2 [83.6–86.5] · p95 111.4 | 138.0 [133.7–140.0] | 2.56× [2.48–2.61] | +2.7 [+1.5 to +7.0] |
| Trail Rescue reopen | 295.6 [290.9–298.6] · p95 341.2 | 154.5 [149.7–155.4] · p95 182.6 | 147.8 [145.6–149.5] · p95 174.6 | 149.6 [147.8–151.3] · p95 175.7 | 142.4 [135.9–143.9] | 1.92× [1.88–1.95] | +5.0 [+2.3 to +9.4] |
| Trail Rescue qualify | 271.8 [267.9–273.0] · p95 324.2 | 131.7 [126.5–133.8] · p95 181.1 | 124.5 [122.3–125.9] · p95 167.7 | 126.4 [124.7–128.4] · p95 172.1 | 140.9 [134.1–144.9] | 2.07× [2.00–2.14] | +6.5 [+1.6 to +11.5] |
| Ledger evidence | 116.6 [115.1–118.3] · p95 139.5 | 35.6 [34.9–35.9] · p95 43.7 | 35.5 [34.7–36.2] · p95 43.2 | 34.9 [34.5–36.6] · p95 43.3 | 81.3 [79.2–82.7] | 3.28× [3.21–3.33] | +0.1 [-1.1 to +1.0] |
| Trail Rescue refused | 36.7 [35.9–37.1] · p95 48.9 | 8.4 [8.3–8.6] · p95 12.6 | throws: the call 6.8 [6.6–7.0] · p95 12.1 | 7.3 [7.1–7.3] · p95 11.2 | 28.4 [27.6–28.5] | 4.33× [4.31–4.42] | - |
| Ledger refused | 21.2 [21.0–21.4] · p95 25.8 | 7.0 [6.8–7.1] · p95 8.4 | throws: the call 6.6 [6.4–6.7] · p95 8.1 | 6.5 [6.5–6.9] · p95 8.2 | 14.2 [14.0–14.4] | 3.03× [3.00–3.09] | - |

Processors 0–1 (Run 2026-09-29T20-00-02-459-view-path; targets opt (d5938cc, reactive WebAssembly 0a19e4b85886), main (768275b, reactive WebAssembly e35944503272); mask 0x3, priority high; total CPU over the kept timed jobs mean 5.4%, highest repeat p95 10.7%, max 12.4%; 9 attempt(s) discarded by the job guard and run again):

| Event | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` | saving: kit `dispatch()` + `view()` − `dispatchView()` | old ÷ new | gap: `dispatchView()` − raw `dispatch_view` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 124.9 [124.9–125.5] · p95 142.2 | 31.5 [31.4–31.9] · p95 34.8 | 31.6 [31.5–31.7] · p95 35.0 | 31.5 [31.4–31.9] · p95 35.1 | 93.5 [93.0–94.0] | 3.98× [3.92–3.98] | -0.1 [-0.3 to +0.4] |
| Glowcap state-changing tick | 119.5 [119.3–120.1] · p95 146.2 | 33.3 [33.2–33.6] · p95 39.1 | 33.4 [33.2–33.5] · p95 40.2 | 33.4 [33.1–33.5] · p95 40.2 | 86.3 [85.7–86.8] | 3.60× [3.55–3.61] | 0.0 [-0.2 to +0.2] |
| Trail Rescue evidence | 239.7 [236.4–239.9] · p95 291.2 | 115.9 [115.6–117.0] · p95 149.0 | 112.4 [112.2–112.6] · p95 144.0 | 113.6 [113.5–114.1] · p95 146.2 | 122.9 [120.8–123.8] | 2.05× [2.04–2.07] | +3.7 [+3.2 to +4.4] |
| Trail Rescue commit | 198.5 [196.4–199.4] · p95 237.8 | 75.4 [75.2–76.0] · p95 97.9 | 73.3 [73.1–73.5] · p95 94.0 | 73.8 [73.7–74.7] · p95 95.6 | 123.1 [121.2–123.4] | 2.62× [2.61–2.63] | +2.3 [+1.9 to +2.5] |
| Trail Rescue reopen | 258.8 [256.2–259.3] · p95 297.1 | 132.4 [132.3–132.6] · p95 153.7 | 128.6 [127.3–129.5] · p95 151.1 | 131.3 [130.1–131.9] · p95 153.3 | 126.2 [123.9–126.9] | 1.95× [1.94–1.96] | +3.8 [+3.1 to +5.0] |
| Trail Rescue qualify | 238.7 [238.5–239.7] · p95 290.2 | 111.9 [111.0–112.7] · p95 149.9 | 107.3 [107.2–107.5] · p95 143.4 | 110.4 [110.2–111.4] · p95 148.0 | 127.7 [125.8–127.8] | 2.14× [2.12–2.15] | +4.4 [+3.8 to +5.4] |
| Ledger evidence | 101.4 [101.2–102.4] · p95 122.7 | 30.6 [30.5–30.7] · p95 38.0 | 30.2 [30.0–30.4] · p95 37.0 | 29.8 [29.8–30.1] · p95 36.6 | 70.7 [70.7–71.8] | 3.32× [3.30–3.35] | +0.5 [+0.1 to +0.6] |
| Trail Rescue refused | 32.0 [31.6–32.2] · p95 43.8 | 7.4 [7.3–7.4] · p95 11.3 | throws: the call 6.0 [5.9–6.1] · p95 10.6 | 6.5 [6.4–6.6] · p95 9.8 | 24.6 [24.3–24.8] | 4.33× [4.32–4.35] | - |
| Ledger refused | 18.5 [18.5–18.5] · p95 23.0 | 6.0 [6.0–6.1] · p95 7.2 | throws: the call 5.6 [5.6–5.6] · p95 7.0 | 5.5 [5.5–5.6] · p95 6.9 | 12.5 [12.4–12.5] | 3.08× [3.03–3.08] | - |

- On processors 0–1 every path takes 0.81–0.89× the time it takes on 10–13
  (the baseline measured 0.72–0.78×, its section 7): processors 0 and 1 ran at
  5.10–5.20 GHz under the benchmark, against a median of 4.49 GHz for
  processor 10 in the primary session and 4.52 GHz in the replicate. The
  ratios old ÷ new agree across the three sessions within 0.13.
- Inside the mask `0x3C00` Windows placed most jobs on processor 10 (69 of the
  88 jobs whose processor the monitor identified in the primary session, 66 of
  85 in the replicate), the rest on 13 (17 and 19; median 4.32 and 4.39 GHz)
  or 11 (2). The widest run ranges follow that placement, not load: the
  branch's kit `dispatch()` + `view()` on an idle tick took 158.5 µs in
  repeat 1 of the primary session, on processor 13 at 4.15 GHz, against
  147.6–149.8 µs on processor 10 at 4.35–4.45 GHz (`results.json` records each
  job's processors and clock). Every path's median in the replicate is
  0.97–1.02× the primary's; savings and ratios agree within their ranges.
- main's own paths in this session, against the baseline's primary session
  (the same WebAssembly bytes, `e35944503272…`, on the same mask):

  | Event | kit `dispatch()` + `view()`: baseline primary | this session | change | raw `dispatch_view` + `JSON.parse`: baseline primary | this session | change |
  | --- | --- | --- | --- | --- | --- | --- |
  | Glowcap idle tick | 166.4 | 150.7 | −9.4% | 43.8 | 37.5 | −14.4% |
  | Glowcap state-changing tick | 159.5 | 145.4 | −8.8% | 46.6 | 39.5 | −15.2% |
  | Trail Rescue evidence | 300.8 | 272.9 | −9.3% | 141.7 | 129.9 | −8.3% |
  | Trail Rescue commit | 251.6 | 226.5 | −10.0% | 92.8 | 85.2 | −8.2% |
  | Trail Rescue reopen | 325.2 | 294.9 | −9.3% | 162.6 | 148.5 | −8.7% |
  | Trail Rescue qualify | 302.0 | 273.0 | −9.6% | 136.8 | 124.1 | −9.3% |
  | Ledger evidence | 129.0 | 118.7 | −8.0% | 38.1 | 34.9 | −8.4% |

  The same bytes ran 8–15% faster in this session, a little beyond the
  baseline's session-to-session spread (−9% to +8%): the busy processor ran
  at 4.0–4.6 GHz here (a median of 4.49 GHz on processor 10), against
  3.7–4.0 GHz on processors 10–13 in the baseline (its section 7), both read
  by typeperf. Absolute figures therefore compare only within a session,
  which is why every saving here is paired inside one.

## 4. What remains in the JavaScript wrapper

`dispatchView()` is −0.1 to +1.4 µs from raw `dispatch_view` +
`JSON.parse` on Glowcap and the ledger in every session, and +1.8 to +6.5 µs
(2.0–5.2%) above it on Trail Rescue's accepted events (medians per session,
paired by repeat). Its pieces, each DIRECT (medians, µs, [lowest–highest
run]), on processors 10–13 (primary session); the last five rows are MEASURED
differences paired by repeat:

| Piece | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | Trail Rescue qualify | Ledger evidence | Trail Rescue refused | Ledger refused |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| raw `dispatch_view` path: `JSON.stringify(payload)` | 0.3 [0.2–0.3] | 0.2 [0.2–0.3] | 0.8 [0.8–0.8] | 0.4 [0.4–0.4] | 0.4 [0.4–0.5] | 0.4 [0.4–0.4] | 0.3 [0.3–0.3] | 0.4 [0.4–0.4] | 0.2 [0.2–0.2] |
| raw `dispatch_view` path: the `dispatch_view` call | 22.6 [22.1–23.8] | 24.9 [23.5–26.7] | 105.9 [103.6–107.0] | 60.6 [59.2–61.6] | 124.2 [121.3–124.8] | 103.0 [100.5–104.9] | 28.4 [28.0–29.3] | 6.9 [6.7–7.0] | 6.5 [6.4–6.7] |
| raw `dispatch_view` path: `JSON.parse` of the view | 14.8 [14.4–15.7] | 14.0 [13.3–15.2] | 24.6 [23.9–25.1] | 24.1 [23.6–24.7] | 25.0 [24.2–25.6] | 23.2 [22.6–23.8] | 6.3 [6.2–6.6] | - | - |
| **raw `dispatch_view` + `JSON.parse`** | 37.7 [36.8–39.8] | 39.1 [37.2–42.3] | 131.5 [128.6–133.3] | 85.9 [83.9–87.3] | 150.1 [147.0–151.3] | 126.7 [123.8–128.4] | 35.2 [34.7–36.4] | - | - |
| raw `dispatch_view_outcome` path: `JSON.stringify(payload)` | 0.2 [0.2–0.3] | 0.2 [0.2–0.3] | 0.8 [0.7–0.8] | 0.4 [0.4–0.4] | 0.4 [0.4–0.5] | 0.4 [0.3–0.4] | 0.3 [0.3–0.3] | 0.4 [0.4–0.4] | 0.2 [0.2–0.2] |
| raw `dispatch_view_outcome` path: the `dispatch_view_outcome` call | 22.5 [22.1–24.7] | 25.1 [23.8–27.3] | 107.2 [103.2–107.6] | 61.3 [59.2–62.1] | 125.1 [121.6–126.2] | 104.3 [101.3–106.7] | 27.8 [27.3–28.3] | 5.7 [5.6–5.7] | 5.6 [5.5–5.7] |
| raw `dispatch_view_outcome` path: `JSON.parse` of the envelope | 15.1 [14.6–15.9] | 14.2 [13.7–15.5] | 24.1 [24.0–24.5] | 24.2 [24.0–24.6] | 25.3 [24.9–25.5] | 23.4 [23.3–23.7] | 6.5 [6.4–6.6] | 0.8 [0.8–0.8] | 0.6 [0.6–0.7] |
| **raw `dispatch_view_outcome` + `JSON.parse`** | 37.9 [37.0–41.0] | 39.8 [37.8–43.0] | 132.5 [128.2–133.5] | 87.2 [84.2–87.7] | 151.6 [148.1–153.5] | 128.0 [125.9–131.0] | 34.6 [34.1–35.2] | 7.4 [7.3–7.4] | 6.4 [6.3–6.6] |
| pieces: the kit's `payloadText(payload)` | 0.5 [0.4–0.5] | 0.5 [0.4–0.5] | 2.1 [2.0–2.2] | 1.0 [1.0–1.1] | 1.2 [1.1–1.2] | 1.0 [1.0–1.1] | 1.0 [1.0–1.1] | 1.0 [1.0–1.0] | 0.7 [0.7–0.7] |
| pieces: the `dispatch_view_outcome` call | 23.2 [22.6–23.8] | 25.8 [24.7–26.6] | 105.7 [103.8–108.3] | 60.7 [59.8–62.0] | 124.6 [122.0–127.4] | 103.6 [101.9–105.6] | 27.9 [27.3–30.1] | 5.7 [5.6–5.9] | 5.6 [5.5–6.1] |
| pieces: `JSON.parse` of the envelope | 15.2 [14.7–15.4] | 14.5 [13.9–15.0] | 23.3 [22.9–24.2] | 22.7 [22.6–23.3] | 23.1 [22.9–24.1] | 21.3 [21.0–21.9] | 6.3 [6.1–6.7] | 0.8 [0.8–0.9] | 0.6 [0.6–0.7] |
| pieces: the kit's `validOutcome(outcome, 'view')` | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.2 [0.2–0.2] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] |
| **pieces: the four, first timer to last** | 38.9 [37.9–39.8] | 40.9 [39.2–42.2] | 132.4 [129.8–135.1] | 85.1 [84.4–87.2] | 149.3 [147.3–153.1] | 125.9 [125.0–129.2] | 35.4 [34.6–38.2] | 8.4 [8.2–8.5] | 7.1 [6.9–7.6] |
| pieces: `JSON.parse` of the view text alone, after the envelope's | 14.4 [14.0–14.6] | 13.8 [13.2–14.2] | 17.5 [17.1–18.1] | 19.3 [19.1–19.9] | 19.9 [19.8–20.7] | 18.8 [18.8–19.6] | 5.1 [4.9–5.4] | - | - |
| **kit `dispatchView()`** | 38.9 [37.6–40.1] | 40.4 [39.3–42.2] | 133.3 [131.8–138.5] | 87.7 [85.7–90.3] | 152.6 [150.0–156.4] | 129.4 [126.5–133.0] | 36.2 [35.0–36.8] | 8.3 [8.1–8.6] | 7.0 [6.9–7.3] |
| `dispatchView()` − the four pieces (paired by repeat; the wrapper's state and argument checks, and the timer reads between pieces) | -0.3 [-0.9 to +1.2] | -0.5 [-1.2 to +1.5] | +2.4 [-1.3 to +3.4] | +2.7 [-1.3 to +3.1] | +3.3 [-0.3 to +5.3] | +3.5 [-0.9 to +4.0] | -0.1 [-2.0 to +1.7] | 0.0 [-0.3 to +0.1] | -0.1 [-0.6 to +0.1] |
| `dispatchView()` − raw `dispatch_view` + `JSON.parse` (paired by repeat) | +0.5 [-0.9 to +1.9] | +1.4 [-1.3 to +3.2] | +2.6 [-1.2 to +7.0] | +1.8 [-1.3 to +5.0] | +3.0 [-0.7 to +6.3] | +2.8 [-1.7 to +6.3] | +0.7 [-0.2 to +1.8] | - | - |
| the kit's payload check: `payloadText` − `JSON.stringify` of the raw `dispatch_view_outcome` path (paired by repeat) | +0.2 [+0.1 to +0.3] | +0.3 [+0.2 to +0.3] | +1.3 [+1.2 to +1.4] | +0.6 [+0.6 to +0.7] | +0.7 [+0.7 to +0.8] | +0.7 [+0.6 to +0.7] | +0.7 [+0.7 to +0.8] | +0.6 [+0.6 to +0.6] | +0.5 [+0.5 to +0.5] |
| envelope against view: `JSON.parse` of the envelope − of the view, each the first parse on its own raw path (paired by repeat) | +0.2 [+0.1 to +0.3] | +0.2 [+0.1 to +0.4] | -0.4 [-0.7 to +0.2] | +0.3 [-0.5 to +0.6] | +0.5 [-0.3 to +0.7] | +0.1 [-0.4 to +1.0] | +0.1 [-0.1 to +0.4] | - | - |
| the call: `dispatch_view_outcome` − `dispatch_view`, each on its own raw path (paired by repeat) | 0.0 [-0.1 to +0.9] | +0.3 [-0.3 to +0.6] | +0.8 [-0.4 to +1.7] | +0.5 [0.0 to +1.3] | +1.6 [+0.3 to +2.1] | +1.8 [+0.8 to +2.6] | -0.9 [-1.2 to +0.3] | -1.2 [-1.3 to -1.1] | -0.9 [-1.1 to -0.7] |

- **The kit's payload check** (`payloadText`: the walk that refuses NaN,
  holes and class instances, then `JSON.stringify`) costs 0.2–0.3 µs more
  than `JSON.stringify` on a tick's `{dt}` and 0.5–1.3 µs more on the
  ledger's and Trail Rescue's payloads (0.3–1.3 µs in the other sessions).
- **`validOutcome`**: 0.1–0.2 µs.
- **The envelope's parse**: `JSON.parse` of the outcome envelope and of the
  bare view, each the first parse on its own raw path, differ by −0.4 to
  +0.5 µs in every session: not measurable.
- **The call.** `dispatch_view_outcome` and `dispatch_view` differ by −0.9 to
  +0.5 µs on Glowcap and the ledger in every session. On Trail Rescue's
  accepted events the new export is slower: +0.5 to +1.8 µs in the primary
  session, −0.4 to +1.6 µs in the replicate and +0.8 to +2.7 µs on processors
  0–1, where both calls ran on one processor at 5.1–5.2 GHz and the ranges are
  tightest; the most on reopen (+2.4 [+1.7 to +2.7]) and qualify (+2.7
  [+2.4 to +3.3]) there. That is inside the runtime, not the wrapper (section
  8, item 4). On refusals the new export is 0.8–1.2 µs faster than the legacy
  `dispatch_view`, which throws.
- **Not attributed**: on Trail Rescue's accepted events `dispatchView()` is
  +2.4 to +3.5 µs above its four pieces timed one by one in the primary
  session, +3.7 to +5.3 µs in the replicate and +1.0 to +2.4 µs on processors
  0–1, and on Glowcap, the ledger and refusals −1.3 to +0.1 µs from them. What
  `dispatchView()` runs outside the four pieces is a state check, a method
  check and an event type check, each well under 0.1 µs, so the residual is
  more likely process state (JIT, garbage collection, the processor each
  process ran on) that differs between the two processes, which this method
  cannot separate. It is at most 5% of those events.
- In sum, on Trail Rescue about 1 µs of the gap is the kit's own code
  (payload check and `validOutcome`), up to about 3 µs is the new export's
  call, and the rest is not attributed; on Glowcap and the ledger there is
  nothing left to attribute.
- `JSON.parse` of the view text alone, timed right after the envelope's parse
  of the same bytes, is 0.2–6.7 µs cheaper than either first parse. That is
  the order (a second parse of warm bytes), not the envelope: compare the
  first parses above.

## 5. Size

`size.mjs`: each tree's `npm run build` output, and the kit staged and packed
as `scripts/test-kit-package.mjs` stages it (`results/size.json`). The
branch's checkout, at a459995, then held the first measurement's three
uncommitted harness files under `experiments/`, which the kit does not pack
(`size.json` counts them as `changedFiles`). This commit changes nothing the
size report reads, so `size.json` is unchanged.

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
  no correctness finding (the job guard also keeps any correctness note of a
  discarded attempt; there were none). The pieces mode checked for every accepted event
  that the outcome is exactly `{"schema":"caveat-dispatch/0.1","outcome":"accepted","view":`
  + view + `}` and that `validOutcome` accepts every outcome.
- The kit, the runtime and every existing test are as the implementation
  left them: this commit changes files under `experiments/` only.

## 7. Load, and the sessions not used

Per repeat, total CPU over the 24 logical processors in the samples covering
its kept timed jobs (the benchmark itself about 5%), and per timed job, with
every attempt the job guard discarded:

Primary, processors 10–13:

| Repeat | from (UTC) | seconds | timed jobs kept | attempts discarded | samples | total CPU mean | p95 | max | samples above 20% | kept |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 19:49:01 | 103.5 | 21 of 21 | 3 | 91 | 5.4% | 7.9% | 9.0% | 0 | yes |
| 2 | 19:50:46 | 109.4 | 21 of 21 | 2 | 90 | 5.5% | 8.2% | 10.3% | 0 | yes |
| 3 | 19:52:36 | 112.4 | 21 of 21 | 2 | 87 | 5.6% | 8.2% | 10.7% | 0 | yes |
| 4 | 19:54:29 | 129.7 | 21 of 21 | 4 | 88 | 5.4% | 8.7% | 10.5% | 0 | yes |
| 5 | 19:56:39 | 123.7 | 21 of 21 | 3 | 88 | 5.5% | 8.6% | 10.6% | 0 | yes |

Over the 105 kept timed jobs, judged again afterwards from load.csv: 0 broke the rule; the highest job mean total CPU 8.9%, the highest sample 10.7%, at most 1.63 cores busy outside the pinned processors, System at most 19.9% and MsMpEng at most 21.6% of one core. The guard's own verdict, as each job ended, agrees with it for every kept job.

| Discarded attempt | repeat | at (UTC) | seconds | why |
| --- | --- | --- | --- | --- |
| `opt-glowcap-replay raw.dispatch_view` #1 | 1 | 19:48:48 | 1.8 | System at 87% of a core > 30% |
| `opt-glowcap-replay raw.dispatch_view` #2 | 1 | 19:48:55 | 1.8 | MsMpEng at 31% of a core > 30% |
| `main-trail-rescue-scenarios raw.dispatch_view` #1 | 1 | 19:49:49 | 6.2 | System at 108% of a core > 30% |
| `opt-glowcap-replay raw.dispatch_view_outcome` #1 | 2 | 19:50:50 | 1.8 | System at 102% of a core > 30%; 2 cores busy outside the pinned processors > 2 |
| `opt-trail-rescue-scenarios raw.dispatch_view_outcome` #1 | 2 | 19:51:47 | 6.2 | System at 108% of a core > 30% |
| `main-glowcap-replay kit` #1 | 3 | 19:52:49 | 6.4 | System at 101% of a core > 30% |
| `opt-trail-rescue-scenarios kit` #1 | 3 | 19:53:45 | 7.6 | System at 100% of a core > 30% |
| `main-glowcap-replay kit` #1 | 4 | 19:54:35 | 6.3 | 2.1 cores busy outside the pinned processors > 2 |
| `main-glowcap-replay kit` #2 | 4 | 19:54:46 | 6.2 | System at 78% of a core > 30% |
| `opt-ledger-session kit.dispatchView` #1 | 4 | 19:55:28 | 0.9 | 2 cores busy outside the pinned processors > 2 |
| `opt-trail-rescue-scenarios raw.dispatch_view_outcome` #1 | 4 | 19:55:51 | 6.1 | System at 109% of a core > 30% |
| `main-glowcap-replay kit` #1 | 5 | 19:56:53 | 6.3 | System at 96% of a core > 30% |
| `opt-trail-rescue-scenarios kit` #1 | 5 | 19:57:50 | 7.4 | System at 107% of a core > 30% |
| `opt-trail-rescue-scenarios kit.dispatchView.pieces` #1 | 5 | 19:58:26 | 6.5 | MsMpEng at 40% of a core > 30%; 3.6 cores busy outside the pinned processors > 2 |

Replicate, processors 10–13:

| Repeat | from (UTC) | seconds | timed jobs kept | attempts discarded | samples | total CPU mean | p95 | max | samples above 20% | kept |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 20:07:10 | 100.3 | 21 of 21 | 1 | 89 | 5.5% | 8.2% | 10.4% | 0 | yes |
| 2 | 20:08:52 | 112.3 | 21 of 21 | 2 | 88 | 5.5% | 7.6% | 9.7% | 0 | yes |
| 3 | 20:10:45 | 113.5 | 21 of 21 | 2 | 88 | 5.3% | 8.0% | 10.4% | 0 | yes |
| 4 | 20:12:39 | 108.2 | 21 of 21 | 2 | 89 | 5.3% | 8.3% | 9.0% | 0 | yes |
| 5 | 20:14:28 | 108.4 | 21 of 21 | 2 | 88 | 5.5% | 8.6% | 10.2% | 0 | yes |

Over the 105 kept timed jobs, judged again afterwards from load.csv: 0 broke the rule; the highest job mean total CPU 8.3%, the highest sample 10.4%, at most 1.57 cores busy outside the pinned processors, System at most 18.4% and MsMpEng at most 15.3% of one core. The guard's own verdict, as each job ended, agrees with it for every kept job.

| Discarded attempt | repeat | at (UTC) | seconds | why |
| --- | --- | --- | --- | --- |
| `main-trail-rescue-scenarios raw.dispatch_view` #1 | 1 | 20:07:56 | 6.0 | System at 107% of a core > 30% |
| `main-glowcap-replay kit` #1 | 2 | 20:08:58 | 6.5 | System at 107% of a core > 30% |
| `opt-trail-rescue-scenarios raw.dispatch_view_outcome` #1 | 2 | 20:09:55 | 6.2 | System at 66% of a core > 30% |
| `main-glowcap-replay kit` #1 | 3 | 20:10:58 | 6.3 | System at 104% of a core > 30% |
| `opt-trail-rescue-scenarios kit` #1 | 3 | 20:11:55 | 7.4 | System at 93% of a core > 30% |
| `opt-glowcap-replay kit.dispatchView.pieces` #1 | 4 | 20:13:02 | 2.5 | System at 104% of a core > 30% |
| `opt-trail-rescue-scenarios kit.dispatchView` #1 | 4 | 20:14:03 | 6.0 | System at 102% of a core > 30% |
| `opt-ledger-session kit.dispatchView` #1 | 5 | 20:15:04 | 1.0 | System at 101% of a core > 30%; 2 cores busy outside the pinned processors > 2 |
| `opt-trail-rescue-scenarios kit.dispatchView.pieces` #1 | 5 | 20:15:57 | 6.3 | MsMpEng at 32% of a core > 30% |

Processors 0–1:

| Repeat | from (UTC) | seconds | timed jobs kept | attempts discarded | samples | total CPU mean | p95 | max | samples above 20% | kept |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 20:00:15 | 89.3 | 21 of 21 | 1 | 78 | 5.4% | 9.3% | 11.2% | 0 | yes |
| 2 | 20:01:45 | 126.8 | 21 of 21 | 6 | 80 | 5.4% | 10.3% | 12.4% | 0 | yes |
| 3 | 20:03:52 | 98.5 | 21 of 21 | 2 | 79 | 5.3% | 10.7% | 12.1% | 0 | yes |

Over the 63 kept timed jobs, judged again afterwards from load.csv: 0 broke the rule; the highest job mean total CPU 8.2%, the highest sample 12.4%, at most 1.93 cores busy outside the pinned processors, System at most 29.2% and MsMpEng at most 12.2% of one core. The guard's own verdict, as each job ended, agrees with it for every kept job.

| Discarded attempt | repeat | at (UTC) | seconds | why |
| --- | --- | --- | --- | --- |
| `main-trail-rescue-scenarios raw.dispatch_view` #1 | 1 | 20:00:56 | 5.2 | System at 108% of a core > 30%; 2.6 cores busy outside the pinned processors > 2 |
| `main-glowcap-replay kit` #1 | 2 | 20:01:51 | 5.4 | System at 73% of a core > 30% |
| `main-glowcap-replay kit` #2 | 2 | 20:02:02 | 5.5 | 2.3 cores busy outside the pinned processors > 2 |
| `main-ledger-session raw.dispatch_view` #1 | 2 | 20:02:30 | 0.9 | 2 cores busy outside the pinned processors > 2 |
| `main-ledger-session raw.dispatch_view` #2 | 2 | 20:02:36 | 0.9 | 2.2 cores busy outside the pinned processors > 2 |
| `opt-ledger-session raw.dispatch_view_outcome` #1 | 2 | 20:02:45 | 0.9 | 3 cores busy outside the pinned processors > 2 |
| `opt-ledger-session kit.dispatchView` #1 | 2 | 20:02:58 | 0.9 | System at 104% of a core > 30% |
| `opt-glowcap-replay raw.dispatch_view_outcome` #1 | 3 | 20:03:57 | 1.6 | System at 85% of a core > 30% |
| `opt-trail-rescue-scenarios kit` #1 | 3 | 20:04:54 | 6.6 | System at 107% of a core > 30% |

Of the 32 discarded attempts, 23 overlapped `System`'s burst (66–109% of a
core), 3 a Defender burst (31–40% of a core) and 6 other load of 2.0–3.0 cores
outside the pinned processors; at most two attempts of one job were
discarded. The guard (its wait for the sample covering each job's end, and
the reruns) made each session 2–3 minutes longer.

Earlier sessions were discarded or superseded and are not used anywhere
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
  was tightened to none, and P2 discarded.
- **S1–S3** (18:34–18:54 UTC): the first commit's three sessions, kept by the
  per-repeat rule and superseded by the job guard. The review found two jobs
  in S3 that overlapped bursts its own logs recorded (the branch's raw
  `dispatch_view` on Glowcap in repeat 1, 16.1% mean total CPU with Defender
  and `System` busy, 46.2 µs on an idle tick against 38.7–40.1 µs in the other
  repeats; and raw `dispatch_view_outcome` in repeat 3 during a `System`
  burst, 41.9 against 38.1–39.5 µs), which the per-repeat rule, averaged over
  80 s, could not see. Judged afterwards by the per-job rule, all three fail
  it: their monitor started too late to cover each first job's start and
  stopped before each last job's end, and 4 jobs of S1 and 3 of S3 overlapped
  a burst. Every session here was measured again with the guard; S1–S3's
  files remain in this branch's history at d5938cc.

## 8. Noticed for the next decision

Recorded, not acted on: this work changed nothing but the harness.

1. **The next cost on the view path is the view's JSON round trip.** On a
   Glowcap idle tick, `dispatchView()` takes 38.9 µs, of which the
   `dispatch_view_outcome` call is 23.2 µs and `JSON.parse` of the outcome
   15.2 µs (its pieces, primary session); the call builds and serializes the
   whole view, a median of 4,458 bytes (about 4.4 KiB) on these ticks. Yet on
   9,393 of the replay's 9,394 idle ticks the new view differs from the one
   before only in its top-level `sequence` (an untimed check on the branch
   build, not committed). A host that forwards the view as text, or only
   needs what changed, pays for the rest. A text variant (as `viewText()` is
   to `view()`) or a changed-parts signal would remove it; neither is built.
2. **On decision events the language dominates.** On Trail Rescue's accepted
   events `dispatchView()` takes 87.7–152.6 µs, of which the
   `dispatch_view_outcome` call is 60.7–124.6 µs and `JSON.parse` of the
   outcome 21.3–23.3 µs (all on the `dispatchView()` path: the kit mode and
   its pieces, primary session); a further gain there needs the runtime
   itself (the baseline's section 10), not the wrapper.
3. **The kit's payload check is cheap**: 0.2–1.3 µs per event. Nothing to do.
4. **The new export's call is 1–3 µs slower than `dispatch_view`'s on Trail
   Rescue's larger views** (section 4; +2.4 and +2.7 µs on reopen and qualify
   on processors 0–1), and not on Glowcap's or the ledger's. Not
   investigated: the difference is inside the runtime call, which this work
   does not touch. One difference between the two exports is that the new
   outcome is serialized as a struct with a flattened enum around the view
   (`runtime/src/reactive_outcome.rs`), where `dispatch_view` serializes the
   view alone; whether that is the cost is not measured.
5. **A refusal through the new export is about 1 µs cheaper than through the
   legacy `dispatch_view`**, which throws.
6. **Core placement inside a mask moves absolute figures by up to about 7%**
   (processor 13 at 4.1–4.5 GHz against 10 at 4.0–4.6 GHz, median 4.49, here),
   and the whole machine's clock by more between sessions (section 3.2):
   comparisons belong inside one interleaved session, as the savings here
   are.
7. **This laptop's `System` process takes about one core for 3–4 s about
   once a minute**, and a total-CPU rule averaged over a repeat does not see
   it, nor one busy core of 24 (Windows Defender scanning a stream of opened
   files, as in session A). A measurer here should judge every timed job by
   its own load at one-second resolution, watching `System` and `MsMpEng`, as
   the job guard now does, and should not run recursive `ls` over large trees
   on this machine while others measure.
8. **`dispatchView()` is new in 0.1.0-rc.5, unreleased.** Hosts on rc.4 keep
   `dispatch()` + `view()`, or the raw `dispatch_view` + `JSON.parse`, which
   costs within 0–5% of what `dispatchView()` costs.

## 9. Raw results and how to rerun

In [`results/`](results/) (large JSON gzipped; every script here reads the
`.gz` files directly):

| Directory / file | What |
| --- | --- |
| `view-path-0x3C00/` | the primary session: `results.json.gz` (every run's summaries, environment, targets with hashes, inputs, each job's processors, clock and job-guard verdict, and every discarded attempt under `environment.jobGuard`), `classes.json.gz` (per event class), `summary.md`, `run-log.txt` (with a line per discarded attempt), `load.csv` (typeperf every second: total CPU, the pinned processors' use and clock, `System` and `MsMpEng`; its header names every column), `load.json` (`load.mjs`: per repeat and per timed job), `quiet-gate.txt` (the minute of quiet before the session, local time, UTC−7) |
| `view-path-0x3C00-replicate/` | the replicate session, same files |
| `view-path-0x3/` | the processors 0–1 session, same files |
| `size.json` | the size report |
| `tables.md`, `tables-0x3C00-replicate.md`, `tables-0x3.md` | every table above, rendered by `report.mjs` from those files |
| `excluded/` | the index of the discarded and superseded sessions, whose raw files are not committed here |

The per-event samples (about 6.5 MB per session, gzipped) are not committed.
On a quiet machine, from a checkout of this branch built with `npm run build`,
and a checkout of main 768275b built the same way:

```sh
# Each session (then again with --affinity=0x3 --repeats=3 for processors 0-1)
node experiments/performance-0.1/run.mjs --suite=view-path --engines=wasm --build=auto --keep-samples \
  --repeats=5 --monitor-interval=1 --monitor-lead=5 --job-guard --affinity=0x3C00 --priority=high \
  --target=opt=tree:. --target=main=tree:PATH/TO/main-768275b --out=DIR
node experiments/performance-0.1/analyze.mjs DIR          # classes.json
node experiments/performance-opt-0.1/load.mjs DIR         # the keep rules, per job and per repeat
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
