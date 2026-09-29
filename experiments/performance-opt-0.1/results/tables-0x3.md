Run 2026-09-29T20-00-02-459-view-path; targets opt (d5938cc, reactive WebAssembly 0a19e4b85886), main (768275b, reactive WebAssembly e35944503272); mask 0x3, priority high; total CPU over the kept timed jobs mean 5.4%, highest repeat p95 10.7%, max 12.4%; 9 attempt(s) discarded by the job guard and run again.

### Headline, accepted events: the paths side by side (DIRECT, µs) and the saving (paired by repeat, µs)

Refused events are not in this table, nor in any saving, ratio or range about the accepted events: they are in "Refused events" below.

| Event | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` | saving: kit `dispatch()` + `view()` − `dispatchView()` | old ÷ new | gap: `dispatchView()` − raw `dispatch_view` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 124.9 [124.9–125.5] · p95 142.2 | 31.5 [31.4–31.9] · p95 34.8 | 31.6 [31.5–31.7] · p95 35.0 | 31.5 [31.4–31.9] · p95 35.1 | 93.5 [93.0–94.0] | 3.98× [3.92–3.98] | -0.1 [-0.3 to +0.4] |
| Glowcap state-changing tick | 119.5 [119.3–120.1] · p95 146.2 | 33.3 [33.2–33.6] · p95 39.1 | 33.4 [33.2–33.5] · p95 40.2 | 33.4 [33.1–33.5] · p95 40.2 | 86.3 [85.7–86.8] | 3.60× [3.55–3.61] | 0.0 [-0.2 to +0.2] |
| Trail Rescue evidence | 239.7 [236.4–239.9] · p95 291.2 | 115.9 [115.6–117.0] · p95 149.0 | 112.4 [112.2–112.6] · p95 144.0 | 113.6 [113.5–114.1] · p95 146.2 | 122.9 [120.8–123.8] | 2.05× [2.04–2.07] | +3.7 [+3.2 to +4.4] |
| Trail Rescue commit | 198.5 [196.4–199.4] · p95 237.8 | 75.4 [75.2–76.0] · p95 97.9 | 73.3 [73.1–73.5] · p95 94.0 | 73.8 [73.7–74.7] · p95 95.6 | 123.1 [121.2–123.4] | 2.62× [2.61–2.63] | +2.3 [+1.9 to +2.5] |
| Trail Rescue reopen | 258.8 [256.2–259.3] · p95 297.1 | 132.4 [132.3–132.6] · p95 153.7 | 128.6 [127.3–129.5] · p95 151.1 | 131.3 [130.1–131.9] · p95 153.3 | 126.2 [123.9–126.9] | 1.95× [1.94–1.96] | +3.8 [+3.1 to +5.0] |
| Trail Rescue qualify | 238.7 [238.5–239.7] · p95 290.2 | 111.9 [111.0–112.7] · p95 149.9 | 107.3 [107.2–107.5] · p95 143.4 | 110.4 [110.2–111.4] · p95 148.0 | 127.7 [125.8–127.8] | 2.14× [2.12–2.15] | +4.4 [+3.8 to +5.4] |
| Ledger evidence | 101.4 [101.2–102.4] · p95 122.7 | 30.6 [30.5–30.7] · p95 38.0 | 30.2 [30.0–30.4] · p95 37.0 | 29.8 [29.8–30.1] · p95 36.6 | 70.7 [70.7–71.8] | 3.32× [3.30–3.35] | +0.5 [+0.1 to +0.6] |

### Per accepted event, on each path (DIRECT, µs)

| Event | samples per run | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 28182/28182/28182 | 124.9 [124.9–125.5] · p95 142.2 | 31.5 [31.4–31.9] · p95 34.8 | 31.6 [31.5–31.7] · p95 35.0 | 31.5 [31.4–31.9] · p95 35.1 |
| Glowcap state-changing tick | 1809/1809/1809 | 119.5 [119.3–120.1] · p95 146.2 | 33.3 [33.2–33.6] · p95 39.1 | 33.4 [33.2–33.5] · p95 40.2 | 33.4 [33.1–33.5] · p95 40.2 |
| Trail Rescue evidence | 2640/2640/2640 | 239.7 [236.4–239.9] · p95 291.2 | 115.9 [115.6–117.0] · p95 149.0 | 112.4 [112.2–112.6] · p95 144.0 | 113.6 [113.5–114.1] · p95 146.2 |
| Trail Rescue commit | 1440/1440/1440 | 198.5 [196.4–199.4] · p95 237.8 | 75.4 [75.2–76.0] · p95 97.9 | 73.3 [73.1–73.5] · p95 94.0 | 73.8 [73.7–74.7] · p95 95.6 |
| Trail Rescue reopen | 600/600/600 | 258.8 [256.2–259.3] · p95 297.1 | 132.4 [132.3–132.6] · p95 153.7 | 128.6 [127.3–129.5] · p95 151.1 | 131.3 [130.1–131.9] · p95 153.3 |
| Trail Rescue qualify | 360/360/360 | 238.7 [238.5–239.7] · p95 290.2 | 111.9 [111.0–112.7] · p95 149.9 | 107.3 [107.2–107.5] · p95 143.4 | 110.4 [110.2–111.4] · p95 148.0 |
| Ledger evidence | 4500/4500/4500 | 101.4 [101.2–102.4] · p95 122.7 | 30.6 [30.5–30.7] · p95 38.0 | 30.2 [30.0–30.4] · p95 37.0 | 29.8 [29.8–30.1] · p95 36.6 |

### The saving, accepted events (paired by repeat, µs)

| Event | kit `dispatch()` + `view()` − `dispatchView()` | share of the old path | old ÷ new | `dispatchView()` − raw `dispatch_view` + `JSON.parse` | `dispatchView()` − raw `dispatch_view_outcome` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 93.5 [93.0–94.0] | 74.9% [74.5–74.9] | 3.98× [3.92–3.98] | -0.1 [-0.3 to +0.4] | -0.1 [-0.4 to +0.5] |
| Glowcap state-changing tick | 86.3 [85.7–86.8] | 72.2% [71.8–72.3] | 3.60× [3.55–3.61] | 0.0 [-0.2 to +0.2] | -0.2 [-0.2 to +0.5] |
| Trail Rescue evidence | 122.9 [120.8–123.8] | 51.2% [51.1–51.6] | 2.05× [2.04–2.07] | +3.7 [+3.2 to +4.4] | +2.1 [+1.8 to +3.4] |
| Trail Rescue commit | 123.1 [121.2–123.4] | 61.9% [61.7–62.0] | 2.62× [2.61–2.63] | +2.3 [+1.9 to +2.5] | +1.4 [+1.3 to +1.7] |
| Trail Rescue reopen | 126.2 [123.9–126.9] | 48.8% [48.4–48.9] | 1.95× [1.94–1.96] | +3.8 [+3.1 to +5.0] | +1.1 [+0.7 to +2.2] |
| Trail Rescue qualify | 127.7 [125.8–127.8] | 53.3% [52.7–53.5] | 2.14× [2.12–2.15] | +4.4 [+3.8 to +5.4] | +1.3 [+0.8 to +1.5] |
| Ledger evidence | 70.7 [70.7–71.8] | 69.9% [69.7–70.1] | 3.32× [3.30–3.35] | +0.5 [+0.1 to +0.6] | +0.7 [+0.6 to +0.8] |

### Refused events (DIRECT, µs; savings paired by repeat)

Not headline figures. A refusal leaves the view as it was, so a host that already holds the view need not call `view()` after one: against kit `dispatch()` alone, `dispatchView()` saves only 1.2–1.5 µs, and its saving of 12.5–24.6 µs against kit `dispatch()` + `view()` assumes the host would also have called `view()`. Raw `dispatch_view` throws on a refusal, where `dispatchView()` returns the structured refusal, so those two paths are not equivalent contracts; its cell is the throwing call alone.

| Event | samples per run | existing: kit `dispatch()` + `view()` | existing: kit `dispatch()` alone | new: kit `dispatchView()` | existing: raw `dispatch_view` (throws) | new: raw `dispatch_view_outcome` + `JSON.parse` | kit `dispatch()` + `view()` − `dispatchView()` | kit `dispatch()` alone − `dispatchView()` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Trail Rescue refused | 1320/1320/1320 | 32.0 [31.6–32.2] · p95 43.8 | 8.9 [8.8–9.0] · p95 12.8 | 7.4 [7.3–7.4] · p95 11.3 | 6.0 [5.9–6.1] · p95 10.6 | 6.5 [6.4–6.6] · p95 9.8 | 24.6 [24.3–24.8] | +1.5 [+1.5 to +1.6] |
| Ledger refused | 1500/1500/1500 | 18.5 [18.5–18.5] · p95 23.0 | 7.2 [7.2–7.3] · p95 9.8 | 6.0 [6.0–6.1] · p95 7.2 | 5.6 [5.6–5.6] · p95 7.0 | 5.5 [5.5–5.6] · p95 6.9 | 12.5 [12.4–12.5] | +1.2 [+1.1 to +1.3] |

### The existing paths, main against opt (DIRECT medians, µs; change paired by repeat)

| Event | kit `dispatch()` + `view()`, main | opt | change | raw `dispatch_view` + `JSON.parse`, main | opt | change |
| --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 125.7 [125.5–126.2] · p95 143.3 | 124.9 [124.9–125.5] · p95 142.2 | -0.6% [-0.6 to -0.5] | 31.3 [31.2–31.3] · p95 34.4 | 31.6 [31.5–31.7] · p95 35.0 | +1.0% [+1.0 to +1.3] |
| Glowcap state-changing tick | 120.2 [120.1–120.5] · p95 149.2 | 119.5 [119.3–120.1] · p95 146.2 | -0.6% [-0.7 to -0.3] | 33.1 [33.0–33.1] · p95 39.5 | 33.4 [33.2–33.5] · p95 40.2 | +1.2% [+0.3 to +1.2] |
| Trail Rescue evidence | 239.3 [238.7–240.9] · p95 293.0 | 239.7 [236.4–239.9] · p95 291.2 | -0.4% [-1.0 to +0.2] | 112.4 [112.0–112.8] · p95 143.8 | 112.4 [112.2–112.6] · p95 144.0 | +0.2% [-0.5 to +0.4] |
| Trail Rescue commit | 198.7 [197.7–199.7] · p95 236.8 | 198.5 [196.4–199.4] · p95 237.8 | -0.6% [-0.7 to +0.4] | 73.4 [73.1–73.6] · p95 93.8 | 73.3 [73.1–73.5] · p95 94.0 | -0.1% [-0.4 to +0.3] |
| Trail Rescue reopen | 258.8 [257.8–262.4] · p95 299.0 | 258.8 [256.2–259.3] · p95 297.1 | -0.6% [-1.2 to 0.0] | 128.6 [127.8–129.7] · p95 149.3 | 128.6 [127.3–129.5] · p95 151.1 | -0.2% [-0.4 to 0.0] |
| Trail Rescue qualify | 238.3 [238.0–241.5] · p95 294.5 | 238.7 [238.5–239.7] · p95 290.2 | +0.2% [-0.7 to +0.2] | 107.5 [107.3–108.5] · p95 144.0 | 107.3 [107.2–107.5] · p95 143.4 | -0.1% [-1.1 to 0.0] |
| Ledger evidence | 101.4 [101.2–101.9] · p95 124.3 | 101.4 [101.2–102.4] · p95 122.7 | +0.2% [-0.2 to +0.5] | 30.3 [30.2–30.7] · p95 37.2 | 30.2 [30.0–30.4] · p95 37.0 | 0.0% [-2.3 to +0.3] |

### `dispatchView()` and the raw paths, piece by piece (DIRECT medians, µs, [lowest–highest run])

| Piece | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | Trail Rescue qualify | Ledger evidence | Trail Rescue refused | Ledger refused |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| raw `dispatch_view` path: `JSON.stringify(payload)` | 0.2 [0.2–0.2] | 0.2 [0.2–0.2] | 0.7 [0.7–0.7] | 0.4 [0.3–0.4] | 0.4 [0.4–0.4] | 0.3 [0.3–0.4] | 0.3 [0.3–0.3] | 0.3 [0.3–0.4] | 0.2 [0.2–0.2] |
| raw `dispatch_view` path: the `dispatch_view` call | 18.9 [18.8–19.0] | 21.2 [21.0–21.4] | 90.0 [89.9–90.0] | 51.8 [51.4–51.9] | 106.0 [105.0–106.2] | 87.0 [86.8–87.3] | 24.3 [24.3–24.5] | 6.0 [5.9–6.1] | 5.6 [5.6–5.6] |
| raw `dispatch_view` path: `JSON.parse` of the view | 12.5 [12.4–12.6] | 12.0 [11.9–12.1] | 21.2 [21.2–21.3] | 20.8 [20.5–20.9] | 21.5 [21.2–21.6] | 20.2 [20.0–20.3] | 5.4 [5.4–5.5] | - | - |
| **raw `dispatch_view` + `JSON.parse`** | 31.6 [31.5–31.7] | 33.4 [33.2–33.5] | 112.4 [112.2–112.6] | 73.3 [73.1–73.5] | 128.6 [127.3–129.5] | 107.3 [107.2–107.5] | 30.2 [30.0–30.4] | - | - |
| raw `dispatch_view_outcome` path: `JSON.stringify(payload)` | 0.2 [0.2–0.2] | 0.2 [0.2–0.2] | 0.7 [0.6–0.7] | 0.3 [0.3–0.3] | 0.4 [0.4–0.4] | 0.3 [0.3–0.3] | 0.3 [0.3–0.3] | 0.3 [0.3–0.4] | 0.2 [0.2–0.2] |
| raw `dispatch_view_outcome` path: the `dispatch_view_outcome` call | 18.9 [18.8–18.9] | 21.0 [21.0–21.3] | 90.8 [90.6–91.1] | 52.2 [51.7–52.6] | 108.4 [106.7–108.9] | 90.0 [89.2–90.3] | 23.9 [23.9–24.2] | 5.0 [5.0–5.2] | 4.8 [4.8–4.8] |
| raw `dispatch_view_outcome` path: `JSON.parse` of the envelope | 12.4 [12.4–12.7] | 11.9 [11.9–12.3] | 21.4 [21.0–21.5] | 21.2 [20.7–21.2] | 21.7 [21.4–21.8] | 20.5 [20.0–20.6] | 5.5 [5.5–5.6] | 0.7 [0.7–0.7] | 0.5 [0.5–0.5] |
| **raw `dispatch_view_outcome` + `JSON.parse`** | 31.5 [31.4–31.9] | 33.4 [33.1–33.5] | 113.6 [113.5–114.1] | 73.8 [73.7–74.7] | 131.3 [130.1–131.9] | 110.4 [110.2–111.4] | 29.8 [29.8–30.1] | 6.5 [6.4–6.6] | 5.5 [5.5–5.6] |
| pieces: the kit's `payloadText(payload)` | 0.4 [0.4–0.4] | 0.4 [0.4–0.5] | 1.8 [1.8–1.9] | 0.9 [0.8–0.9] | 1.0 [1.0–1.1] | 0.9 [0.9–0.9] | 0.8 [0.8–0.9] | 0.9 [0.9–0.9] | 0.5 [0.5–0.5] |
| pieces: the `dispatch_view_outcome` call | 19.2 [18.9–19.3] | 21.5 [21.4–21.9] | 91.5 [91.5–92.0] | 52.8 [52.8–53.3] | 108.6 [108.6–109.6] | 90.2 [90.0–90.6] | 24.1 [24.1–24.2] | 5.1 [5.1–5.1] | 4.9 [4.9–4.9] |
| pieces: `JSON.parse` of the envelope | 12.4 [12.4–12.5] | 12.1 [11.9–12.1] | 20.7 [20.5–20.8] | 19.8 [19.7–19.8] | 20.2 [20.2–20.3] | 18.5 [18.5–18.9] | 5.4 [5.3–5.4] | 0.7 [0.7–0.7] | 0.6 [0.5–0.6] |
| pieces: the kit's `validOutcome(outcome, 'view')` | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] |
| **pieces: the four, first timer to last** | 32.1 [32.0–32.2] | 34.0 [34.0–34.5] | 115.2 [114.8–115.7] | 74.1 [74.1–74.8] | 131.3 [130.6–131.5] | 109.5 [109.4–110.9] | 30.5 [30.4–30.6] | 7.3 [7.3–7.4] | 6.1 [6.1–6.1] |
| pieces: `JSON.parse` of the view text alone, after the envelope's | 11.9 [11.8–11.9] | 11.5 [11.3–11.5] | 15.2 [15.1–15.3] | 16.7 [16.6–16.7] | 17.3 [17.3–17.4] | 16.5 [16.5–16.6] | 4.4 [4.4–4.4] | - | - |
| **kit `dispatchView()`** | 31.5 [31.4–31.9] | 33.3 [33.2–33.6] | 115.9 [115.6–117.0] | 75.4 [75.2–76.0] | 132.4 [132.3–132.6] | 111.9 [111.0–112.7] | 30.6 [30.5–30.7] | 7.4 [7.3–7.4] | 6.0 [6.0–6.1] |
| `dispatchView()` − the four pieces (paired by repeat; the wrapper's state and argument checks, and the timer reads between pieces) | -0.6 [-0.7 to -0.2] | -0.8 [-0.9 to -0.7] | +1.1 [-0.1 to +1.8] | +1.3 [+0.4 to +1.9] | +1.0 [+0.9 to +2.0] | +2.4 [+0.1 to +3.3] | +0.1 [-0.1 to +0.3] | +0.1 [-0.1 to +0.1] | -0.1 [-0.1 to 0.0] |
| `dispatchView()` − raw `dispatch_view` + `JSON.parse` (paired by repeat) | -0.1 [-0.3 to +0.4] | 0.0 [-0.2 to +0.2] | +3.7 [+3.2 to +4.4] | +2.3 [+1.9 to +2.5] | +3.8 [+3.1 to +5.0] | +4.4 [+3.8 to +5.4] | +0.5 [+0.1 to +0.6] | - | - |
| the kit's payload check: `payloadText` − `JSON.stringify` of the raw `dispatch_view_outcome` path (paired by repeat) | +0.2 [+0.2 to +0.2] | +0.2 [+0.2 to +0.3] | +1.1 [+1.1 to +1.3] | +0.6 [+0.5 to +0.6] | +0.6 [+0.6 to +0.7] | +0.6 [+0.6 to +0.6] | +0.5 [+0.5 to +0.6] | +0.6 [+0.5 to +0.6] | +0.3 [+0.3 to +0.3] |
| envelope against view: `JSON.parse` of the envelope − of the view, each the first parse on its own raw path (paired by repeat) | -0.1 [-0.2 to +0.3] | -0.1 [-0.2 to +0.4] | +0.2 [-0.3 to +0.3] | +0.3 [-0.1 to +0.7] | +0.1 [-0.1 to +0.6] | +0.2 [-0.2 to +0.6] | +0.1 [0.0 to +0.2] | - | - |
| the call: `dispatch_view_outcome` − `dispatch_view`, each on its own raw path (paired by repeat) | 0.0 [-0.1 to 0.0] | -0.2 [-0.4 to +0.3] | +0.8 [+0.7 to +1.1] | +0.8 [-0.2 to +0.8] | +2.4 [+1.7 to +2.7] | +2.7 [+2.4 to +3.3] | -0.4 [-0.6 to -0.1] | -1.0 [-1.1 to -0.7] | -0.8 [-0.8 to -0.8] |

### Load per repeat (typeperf every second; a repeat is kept when, over the samples covering its timed jobs, the mean total CPU is at most 12% and no 1-s sample is above 20%, and every one of its timed jobs keeps the job guard's rule; a session is used only when every repeat is kept)

| Repeat | from (UTC) | seconds | timed jobs kept | attempts discarded | samples | total CPU mean | p95 | max | samples above 20% | kept |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 20:00:15 | 89.3 | 21 of 21 | 1 | 78 | 5.4% | 9.3% | 11.2% | 0 | yes |
| 2 | 20:01:45 | 126.8 | 21 of 21 | 6 | 80 | 5.4% | 10.3% | 12.4% | 0 | yes |
| 3 | 20:03:52 | 98.5 | 21 of 21 | 2 | 79 | 5.3% | 10.7% | 12.1% | 0 | yes |

### Load per timed job (a timed job is kept when, over the 1-s samples covering it, the mean total CPU is at most 12% and no sample is above 20%, and in no sample does System or MsMpEng use more than 30% of one core or are more than 2 cores busy outside the pinned processors; otherwise it is discarded and run again in place (up to 5 attempts), after 3 quiet samples in a row)

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

