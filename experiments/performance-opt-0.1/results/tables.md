Run 2026-09-29T19-48-35-164-view-path; targets opt (d5938cc, reactive WebAssembly 0a19e4b85886), main (768275b, reactive WebAssembly e35944503272); mask 0x3C00, priority high; total CPU over the kept timed jobs mean 5.5%, highest repeat p95 8.7%, max 10.7%; 14 attempt(s) discarded by the job guard and run again.

### Headline, accepted events: the paths side by side (DIRECT, µs) and the saving (paired by repeat, µs)

Refused events are not in this table, nor in any saving, ratio or range about the accepted events: they are in "Refused events" below.

| Event | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` | saving: kit `dispatch()` + `view()` − `dispatchView()` | old ÷ new | gap: `dispatchView()` − raw `dispatch_view` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 149.5 [147.6–158.5] · p95 166.9 | 38.9 [37.6–40.1] · p95 43.5 | 37.7 [36.8–39.8] · p95 43.9 | 37.9 [37.0–41.0] · p95 42.9 | 110.2 [109.2–119.6] | 3.84× [3.72–4.08] | +0.5 [-0.9 to +1.9] |
| Glowcap state-changing tick | 143.2 [140.1–146.0] · p95 177.7 | 40.4 [39.3–42.2] · p95 47.9 | 39.1 [37.2–42.3] · p95 51.1 | 39.8 [37.8–43.0] · p95 52.0 | 102.9 [99.7–103.8] | 3.51× [3.46–3.64] | +1.4 [-1.3 to +3.2] |
| Trail Rescue evidence | 271.7 [268.7–279.2] · p95 326.0 | 133.3 [131.8–138.5] · p95 173.3 | 131.5 [128.6–133.3] · p95 169.7 | 132.5 [128.2–133.5] · p95 171.6 | 139.7 [136.5–144.8] | 2.03× [2.01–2.08] | +2.6 [-1.2 to +7.0] |
| Trail Rescue commit | 225.5 [223.2–232.9] · p95 269.7 | 87.7 [85.7–90.3] · p95 112.7 | 85.9 [83.9–87.3] · p95 110.2 | 87.2 [84.2–87.7] · p95 113.7 | 139.8 [136.7–143.0] | 2.58× [2.56–2.63] | +1.8 [-1.3 to +5.0] |
| Trail Rescue reopen | 293.4 [291.5–300.5] · p95 342.2 | 152.6 [150.0–156.4] · p95 180.7 | 150.1 [147.0–151.3] · p95 178.8 | 151.6 [148.1–153.5] · p95 179.6 | 141.8 [140.8–147.3] | 1.94× [1.92–1.96] | +3.0 [-0.7 to +6.3] |
| Trail Rescue qualify | 270.3 [267.9–280.8] · p95 329.6 | 129.4 [126.5–133.0] · p95 172.3 | 126.7 [123.8–128.4] · p95 168.5 | 128.0 [125.9–131.0] · p95 174.1 | 143.6 [140.8–150.1] | 2.12× [2.09–2.16] | +2.8 [-1.7 to +6.3] |
| Ledger evidence | 118.4 [117.9–121.4] · p95 142.7 | 36.2 [35.0–36.8] · p95 44.2 | 35.2 [34.7–36.4] · p95 43.1 | 34.6 [34.1–35.2] · p95 42.0 | 82.1 [81.5–85.4] | 3.28× [3.21–3.44] | +0.7 [-0.2 to +1.8] |

### Per accepted event, on each path (DIRECT, µs)

| Event | samples per run | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 28182/28182/28182/28182/28182 | 149.5 [147.6–158.5] · p95 166.9 | 38.9 [37.6–40.1] · p95 43.5 | 37.7 [36.8–39.8] · p95 43.9 | 37.9 [37.0–41.0] · p95 42.9 |
| Glowcap state-changing tick | 1809/1809/1809/1809/1809 | 143.2 [140.1–146.0] · p95 177.7 | 40.4 [39.3–42.2] · p95 47.9 | 39.1 [37.2–42.3] · p95 51.1 | 39.8 [37.8–43.0] · p95 52.0 |
| Trail Rescue evidence | 2640/2640/2640/2640/2640 | 271.7 [268.7–279.2] · p95 326.0 | 133.3 [131.8–138.5] · p95 173.3 | 131.5 [128.6–133.3] · p95 169.7 | 132.5 [128.2–133.5] · p95 171.6 |
| Trail Rescue commit | 1440/1440/1440/1440/1440 | 225.5 [223.2–232.9] · p95 269.7 | 87.7 [85.7–90.3] · p95 112.7 | 85.9 [83.9–87.3] · p95 110.2 | 87.2 [84.2–87.7] · p95 113.7 |
| Trail Rescue reopen | 600/600/600/600/600 | 293.4 [291.5–300.5] · p95 342.2 | 152.6 [150.0–156.4] · p95 180.7 | 150.1 [147.0–151.3] · p95 178.8 | 151.6 [148.1–153.5] · p95 179.6 |
| Trail Rescue qualify | 360/360/360/360/360 | 270.3 [267.9–280.8] · p95 329.6 | 129.4 [126.5–133.0] · p95 172.3 | 126.7 [123.8–128.4] · p95 168.5 | 128.0 [125.9–131.0] · p95 174.1 |
| Ledger evidence | 4500/4500/4500/4500/4500 | 118.4 [117.9–121.4] · p95 142.7 | 36.2 [35.0–36.8] · p95 44.2 | 35.2 [34.7–36.4] · p95 43.1 | 34.6 [34.1–35.2] · p95 42.0 |

### The saving, accepted events (paired by repeat, µs)

| Event | kit `dispatch()` + `view()` − `dispatchView()` | share of the old path | old ÷ new | `dispatchView()` − raw `dispatch_view` + `JSON.parse` | `dispatchView()` − raw `dispatch_view_outcome` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 110.2 [109.2–119.6] | 74.0% [73.1–75.5] | 3.84× [3.72–4.08] | +0.5 [-0.9 to +1.9] | +0.2 [-2.1 to +1.7] |
| Glowcap state-changing tick | 102.9 [99.7–103.8] | 71.5% [71.1–72.5] | 3.51× [3.46–3.64] | +1.4 [-1.3 to +3.2] | +1.7 [-2.0 to +2.6] |
| Trail Rescue evidence | 139.7 [136.5–144.8] | 50.8% [50.2–51.9] | 2.03× [2.01–2.08] | +2.6 [-1.2 to +7.0] | +1.6 [-0.7 to +5.4] |
| Trail Rescue commit | 139.8 [136.7–143.0] | 61.2% [60.9–62.0] | 2.58× [2.56–2.63] | +1.8 [-1.3 to +5.0] | +1.3 [-2.0 to +3.0] |
| Trail Rescue reopen | 141.8 [140.8–147.3] | 48.5% [47.9–49.0] | 1.94× [1.92–1.96] | +3.0 [-0.7 to +6.3] | +1.8 [-1.5 to +2.9] |
| Trail Rescue qualify | 143.6 [140.8–150.1] | 52.8% [52.1–53.6] | 2.12× [2.09–2.16] | +2.8 [-1.7 to +6.3] | +1.9 [-4.3 to +3.1] |
| Ledger evidence | 82.1 [81.5–85.4] | 69.5% [68.9–70.9] | 3.28× [3.21–3.44] | +0.7 [-0.2 to +1.8] | +1.3 [-0.2 to +2.7] |

### Refused events (DIRECT, µs; savings paired by repeat)

Not headline figures. A refusal leaves the view as it was, so a host that already holds the view need not call `view()` after one: against kit `dispatch()` alone, `dispatchView()` saves only 1.4–1.7 µs, and its saving of 14.5–28.0 µs against kit `dispatch()` + `view()` assumes the host would also have called `view()`. Raw `dispatch_view` throws on a refusal, where `dispatchView()` returns the structured refusal, so those two paths are not equivalent contracts; its cell is the throwing call alone.

| Event | samples per run | existing: kit `dispatch()` + `view()` | existing: kit `dispatch()` alone | new: kit `dispatchView()` | existing: raw `dispatch_view` (throws) | new: raw `dispatch_view_outcome` + `JSON.parse` | kit `dispatch()` + `view()` − `dispatchView()` | kit `dispatch()` alone − `dispatchView()` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Trail Rescue refused | 1320/1320/1320/1320/1320 | 36.2 [35.8–37.8] · p95 48.6 | 10.0 [9.8–10.1] · p95 14.5 | 8.3 [8.1–8.6] · p95 12.7 | 6.9 [6.7–7.0] · p95 12.2 | 7.4 [7.3–7.4] · p95 11.5 | 28.0 [27.5–29.2] | +1.7 [+1.5 to +1.8] |
| Ledger refused | 1500/1500/1500/1500/1500 | 21.5 [21.4–22.2] · p95 26.4 | 8.4 [8.3–8.7] · p95 10.7 | 7.0 [6.9–7.3] · p95 8.4 | 6.5 [6.4–6.7] · p95 8.0 | 6.4 [6.3–6.6] · p95 8.0 | 14.5 [14.2–15.2] | +1.4 [+1.0 to +1.7] |

### The existing paths, main against opt (DIRECT medians, µs; change paired by repeat)

| Event | kit `dispatch()` + `view()`, main | opt | change | raw `dispatch_view` + `JSON.parse`, main | opt | change |
| --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 150.7 [149.0–154.2] · p95 172.5 | 149.5 [147.6–158.5] · p95 166.9 | -0.8% [-0.9 to +2.8] | 37.5 [37.1–40.0] · p95 41.5 | 37.7 [36.8–39.8] · p95 43.9 | -0.5% [-0.8 to +6.1] |
| Glowcap state-changing tick | 145.4 [140.7–159.5] · p95 184.3 | 143.2 [140.1–146.0] · p95 177.7 | -0.5% [-9.8 to +1.6] | 39.5 [39.1–41.7] · p95 56.1 | 39.1 [37.2–42.3] · p95 51.1 | 0.0% [-5.8 to +1.4] |
| Trail Rescue evidence | 272.9 [270.5–277.8] · p95 331.1 | 271.7 [268.7–279.2] · p95 326.0 | +0.2% [-0.7 to +1.6] | 129.9 [128.2–133.9] · p95 169.1 | 131.5 [128.6–133.3] · p95 169.7 | +0.6% [-1.8 to +3.7] |
| Trail Rescue commit | 226.5 [224.7–230.5] · p95 269.5 | 225.5 [223.2–232.9] · p95 269.7 | -0.1% [-0.7 to +2.2] | 85.2 [83.8–87.0] · p95 111.0 | 85.9 [83.9–87.3] · p95 110.2 | +0.8% [-2.0 to +3.8] |
| Trail Rescue reopen | 294.9 [293.8–300.1] · p95 338.5 | 293.4 [291.5–300.5] · p95 342.2 | -0.1% [-1.2 to +1.2] | 148.5 [145.8–152.6] · p95 178.3 | 150.1 [147.0–151.3] · p95 178.8 | +0.5% [-1.6 to +3.4] |
| Trail Rescue qualify | 273.0 [270.6–278.2] · p95 332.4 | 270.3 [267.9–280.8] · p95 329.6 | -0.1% [-1.3 to +2.0] | 124.1 [122.4–128.2] · p95 167.7 | 126.7 [123.8–128.4] · p95 168.5 | +0.6% [-1.2 to +4.9] |
| Ledger evidence | 118.7 [116.7–123.3] · p95 144.3 | 118.4 [117.9–121.4] · p95 142.7 | +0.6% [-1.5 to +1.5] | 34.9 [34.6–37.3] · p95 42.8 | 35.2 [34.7–36.4] · p95 43.1 | +0.3% [-2.4 to +1.1] |

### `dispatchView()` and the raw paths, piece by piece (DIRECT medians, µs, [lowest–highest run])

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

### Load per repeat (typeperf every second; a repeat is kept when, over the samples covering its timed jobs, the mean total CPU is at most 12% and no 1-s sample is above 20%, and every one of its timed jobs keeps the job guard's rule; a session is used only when every repeat is kept)

| Repeat | from (UTC) | seconds | timed jobs kept | attempts discarded | samples | total CPU mean | p95 | max | samples above 20% | kept |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 19:49:01 | 103.5 | 21 of 21 | 3 | 91 | 5.4% | 7.9% | 9.0% | 0 | yes |
| 2 | 19:50:46 | 109.4 | 21 of 21 | 2 | 90 | 5.5% | 8.2% | 10.3% | 0 | yes |
| 3 | 19:52:36 | 112.4 | 21 of 21 | 2 | 87 | 5.6% | 8.2% | 10.7% | 0 | yes |
| 4 | 19:54:29 | 129.7 | 21 of 21 | 4 | 88 | 5.4% | 8.7% | 10.5% | 0 | yes |
| 5 | 19:56:39 | 123.7 | 21 of 21 | 3 | 88 | 5.5% | 8.6% | 10.6% | 0 | yes |

### Load per timed job (a timed job is kept when, over the 1-s samples covering it, the mean total CPU is at most 12% and no sample is above 20%, and in no sample does System or MsMpEng use more than 30% of one core or are more than 2 cores busy outside the pinned processors; otherwise it is discarded and run again in place (up to 5 attempts), after 3 quiet samples in a row)

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

### Every session, accepted events: the saving, kit `dispatch()` + `view()` − `dispatchView()`, and the gap to the raw path, `dispatchView()` − raw `dispatch_view` + `JSON.parse` (both paired by repeat, µs); `dispatchView()` (DIRECT median, µs)

| Event | 0x3C00: saving | 0x3C00: gap to raw | 0x3C00: `dispatchView()` | 0x3C00-replicate: saving | 0x3C00-replicate: gap to raw | 0x3C00-replicate: `dispatchView()` | 0x3: saving | 0x3: gap to raw | 0x3: `dispatchView()` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 110.2 [109.2–119.6] | +0.5 [-0.9 to +1.9] | 38.9 [37.6–40.1] | 112.3 [109.2–115.4] | +0.4 [+0.2 to +1.4] | 37.9 [37.3–38.7] | 93.5 [93.0–94.0] | -0.1 [-0.3 to +0.4] | 31.5 [31.4–31.9] |
| Glowcap state-changing tick | 102.9 [99.7–103.8] | +1.4 [-1.3 to +3.2] | 40.4 [39.3–42.2] | 103.2 [101.0–108.2] | +0.7 [-0.4 to +1.5] | 39.9 [39.2–40.6] | 86.3 [85.7–86.8] | 0.0 [-0.2 to +0.2] | 33.3 [33.2–33.6] |
| Trail Rescue evidence | 139.7 [136.5–144.8] | +2.6 [-1.2 to +7.0] | 133.3 [131.8–138.5] | 138.3 [131.0–142.8] | +3.3 [+1.4 to +10.0] | 135.5 [131.9–138.5] | 122.9 [120.8–123.8] | +3.7 [+3.2 to +4.4] | 115.9 [115.6–117.0] |
| Trail Rescue commit | 139.8 [136.7–143.0] | +1.8 [-1.3 to +5.0] | 87.7 [85.7–90.3] | 138.0 [133.7–140.0] | +2.7 [+1.5 to +7.0] | 88.9 [86.3–90.4] | 123.1 [121.2–123.4] | +2.3 [+1.9 to +2.5] | 75.4 [75.2–76.0] |
| Trail Rescue reopen | 141.8 [140.8–147.3] | +3.0 [-0.7 to +6.3] | 152.6 [150.0–156.4] | 142.4 [135.9–143.9] | +5.0 [+2.3 to +9.4] | 154.5 [149.7–155.4] | 126.2 [123.9–126.9] | +3.8 [+3.1 to +5.0] | 132.4 [132.3–132.6] |
| Trail Rescue qualify | 143.6 [140.8–150.1] | +2.8 [-1.7 to +6.3] | 129.4 [126.5–133.0] | 140.9 [134.1–144.9] | +6.5 [+1.6 to +11.5] | 131.7 [126.5–133.8] | 127.7 [125.8–127.8] | +4.4 [+3.8 to +5.4] | 111.9 [111.0–112.7] |
| Ledger evidence | 82.1 [81.5–85.4] | +0.7 [-0.2 to +1.8] | 36.2 [35.0–36.8] | 81.3 [79.2–82.7] | +0.1 [-1.1 to +1.0] | 35.6 [34.9–35.9] | 70.7 [70.7–71.8] | +0.5 [+0.1 to +0.6] | 30.6 [30.5–30.7] |

### Every session, refused events (not headline figures): the saving against kit `dispatch()` + `view()` and against kit `dispatch()` alone (both paired by repeat, µs); `dispatchView()` (DIRECT median, µs)

The saving against `dispatch()` + `view()` assumes the host would also have called `view()`; raw `dispatch_view` throws on a refusal, where `dispatchView()` returns the structured refusal, so there is no gap to it.

| Event | 0x3C00: saving against `dispatch()` + `view()` | 0x3C00: saving against `dispatch()` alone | 0x3C00: `dispatchView()` | 0x3C00-replicate: saving against `dispatch()` + `view()` | 0x3C00-replicate: saving against `dispatch()` alone | 0x3C00-replicate: `dispatchView()` | 0x3: saving against `dispatch()` + `view()` | 0x3: saving against `dispatch()` alone | 0x3: `dispatchView()` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Trail Rescue refused | 28.0 [27.5–29.2] | +1.7 [+1.5 to +1.8] | 8.3 [8.1–8.6] | 28.4 [27.6–28.5] | +1.6 [+1.5 to +1.8] | 8.4 [8.3–8.6] | 24.6 [24.3–24.8] | +1.5 [+1.5 to +1.6] | 7.4 [7.3–7.4] |
| Ledger refused | 14.5 [14.2–15.2] | +1.4 [+1.0 to +1.7] | 7.0 [6.9–7.3] | 14.2 [14.0–14.4] | +1.3 [+1.2 to +1.4] | 7.0 [6.8–7.1] | 12.5 [12.4–12.5] | +1.2 [+1.1 to +1.3] | 6.0 [6.0–6.1] |

- 0x3C00: run 2026-09-29T19-48-35-164-view-path, mask 0x3C00, 5 repeats, total CPU over the kept timed jobs mean 5.5%, highest repeat p95 8.7%, max 10.7%; 14 attempt(s) discarded by the job guard and run again
- 0x3C00-replicate: run 2026-09-29T20-06-57-466-view-path, mask 0x3C00, 5 repeats, total CPU over the kept timed jobs mean 5.4%, highest repeat p95 8.6%, max 10.4%; 9 attempt(s) discarded by the job guard and run again
- 0x3: run 2026-09-29T20-00-02-459-view-path, mask 0x3, 3 repeats, total CPU over the kept timed jobs mean 5.4%, highest repeat p95 10.7%, max 12.4%; 9 attempt(s) discarded by the job guard and run again

### Size (bytes)

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

