Run 2026-09-29T18-42-33-481-view-path; targets opt (a459995, reactive WebAssembly 0a19e4b85886), main (768275b, reactive WebAssembly e35944503272); mask 0x3, priority high; total CPU during the run mean 7%, p95 9.3%, max 13.2%.

### Headline: the paths side by side (DIRECT, µs) and the saving (paired by repeat, µs)

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

### Per event, on each path (DIRECT, µs)

| Event | samples per run | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 28182/28182/28182 | 126.5 [125.7–127.1] · p95 145.3 | 31.7 [31.6–31.9] · p95 35.4 | 31.6 [31.4–31.8] · p95 35.3 | 31.7 [31.7–31.8] · p95 35.4 |
| Glowcap state-changing tick | 1809/1809/1809 | 121.1 [121.0–121.4] · p95 156.1 | 33.9 [33.7–33.9] · p95 40.8 | 33.3 [33.1–33.3] · p95 42.3 | 33.7 [33.6–34.0] · p95 43.8 |
| Trail Rescue evidence | 2640/2640/2640 | 241.2 [240.2–241.7] · p95 300.3 | 117.7 [116.3–120.7] · p95 160.1 | 114.1 [114.1–114.6] · p95 152.5 | 114.2 [113.9–114.7] · p95 151.2 |
| Trail Rescue commit | 1440/1440/1440 | 200.3 [199.4–200.8] · p95 245.6 | 76.2 [75.7–76.7] · p95 99.0 | 74.7 [74.2–74.8] · p95 96.0 | 74.4 [74.4–74.7] · p95 96.1 |
| Trail Rescue reopen | 600/600/600 | 261.8 [260.1–262.1] · p95 304.9 | 133.6 [132.9–135.1] · p95 156.9 | 130.6 [130.2–130.7] · p95 156.4 | 131.3 [131.0–132.6] · p95 155.9 |
| Trail Rescue qualify | 360/360/360 | 239.9 [239.7–241.7] · p95 297.1 | 112.9 [112.0–113.8] · p95 154.3 | 109.4 [109.2–110.0] · p95 147.6 | 110.9 [110.7–111.8] · p95 148.0 |
| Ledger evidence | 4500/4500/4500 | 103.0 [102.8–103.2] · p95 127.8 | 30.9 [30.6–31.0] · p95 39.1 | 30.7 [30.6–30.8] · p95 38.4 | 30.2 [30.1–30.3] · p95 37.2 |

### The saving (paired by repeat, µs)

| Event | kit `dispatch()` + `view()` − `dispatchView()` | share of the old path | old ÷ new | `dispatchView()` − raw `dispatch_view` + `JSON.parse` | `dispatchView()` − raw `dispatch_view_outcome` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 94.8 [94.1–95.2] | 74.9% [74.9–74.9] | 3.98× [3.98–3.99] | +0.2 [-0.1 to +0.3] | 0.0 [-0.1 to +0.1] |
| Glowcap state-changing tick | 87.3 [87.2–87.5] | 72.1% [72.0–72.1] | 3.58× [3.57–3.59] | +0.6 [+0.4 to +0.8] | +0.1 [-0.1 to +0.2] |
| Trail Rescue evidence | 123.9 [120.5–124.0] | 51.3% [50.0–51.6] | 2.05× [2.00–2.06] | +3.6 [+2.2 to +6.1] | +3.8 [+1.6 to +6.5] |
| Trail Rescue commit | 124.6 [122.7–124.6] | 62.1% [61.5–62.2] | 2.63× [2.60–2.65] | +1.9 [+1.0 to +2.0] | +1.8 [+1.0 to +2.3] |
| Trail Rescue reopen | 127.2 [126.7–128.5] | 48.9% [48.4–49.0] | 1.96× [1.94–1.96] | +3.4 [+2.2 to +4.5] | +2.3 [+0.3 to +4.1] |
| Trail Rescue qualify | 127.9 [125.9–128.8] | 53.3% [52.5–53.3] | 2.14× [2.11–2.14] | +3.7 [+2.6 to +3.8] | +2.2 [+0.2 to +2.9] |
| Ledger evidence | 72.1 [71.8–72.6] | 70.0% [69.8–70.3] | 3.33× [3.32–3.37] | +0.2 [-0.1 to +0.3] | +0.8 [+0.3 to +0.8] |

### Refused events (DIRECT, µs; savings paired by repeat)

| Event | samples per run | existing: kit `dispatch()` + `view()` | existing: kit `dispatch()` alone | new: kit `dispatchView()` | existing: raw `dispatch_view` (throws) | new: raw `dispatch_view_outcome` + `JSON.parse` | kit `dispatch()` + `view()` − `dispatchView()` | kit `dispatch()` alone − `dispatchView()` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Trail Rescue refused | 1320/1320/1320 | 32.7 [32.5–32.7] · p95 45.2 | 9.1 [9.0–9.2] · p95 13.5 | 7.4 [7.3–7.6] · p95 11.6 | 6.2 [6.2–6.2] · p95 11.6 | 6.6 [6.4–6.7] · p95 10.0 | 25.2 [25.1–25.3] | +1.7 [+1.4 to +1.9] |
| Ledger refused | 1500/1500/1500 | 18.7 [18.7–18.7] · p95 23.3 | 7.3 [7.2–7.3] · p95 10.0 | 6.1 [6.0–6.2] · p95 7.3 | 5.7 [5.7–5.8] · p95 7.4 | 5.6 [5.6–5.7] · p95 6.9 | 12.6 [12.5–12.7] | +1.2 [+1.1 to +1.2] |

### The existing paths, main against opt (DIRECT medians, µs; change paired by repeat)

| Event | kit `dispatch()` + `view()`, main | opt | change | raw `dispatch_view` + `JSON.parse`, main | opt | change |
| --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 127.2 [126.7–127.6] · p95 149.0 | 126.5 [125.7–127.1] · p95 145.3 | -0.8% [-0.9 to -0.1] | 31.6 [31.6–31.8] · p95 35.5 | 31.6 [31.4–31.8] · p95 35.3 | 0.0% [-1.3 to +0.6] |
| Glowcap state-changing tick | 121.3 [121.0–121.6] · p95 160.3 | 121.1 [121.0–121.4] · p95 156.1 | 0.0% [-0.4 to +0.1] | 33.4 [33.4–33.5] · p95 41.7 | 33.3 [33.1–33.3] · p95 42.3 | -0.6% [-0.9 to -0.3] |
| Trail Rescue evidence | 242.5 [242.3–243.5] · p95 305.1 | 241.2 [240.2–241.7] · p95 300.3 | -0.7% [-0.9 to -0.5] | 114.6 [113.7–114.7] · p95 151.7 | 114.1 [114.1–114.6] · p95 152.5 | -0.1% [-0.4 to +0.4] |
| Trail Rescue commit | 202.0 [201.7–202.1] · p95 251.3 | 200.3 [199.4–200.8] · p95 245.6 | -0.8% [-1.1 to -0.6] | 74.4 [74.4–74.5] · p95 96.9 | 74.7 [74.2–74.8] · p95 96.0 | +0.4% [-0.3 to +0.4] |
| Trail Rescue reopen | 262.2 [261.4–263.3] · p95 317.6 | 261.8 [260.1–262.1] · p95 304.9 | 0.0% [-1.2 to +0.2] | 130.8 [130.0–131.2] · p95 153.6 | 130.6 [130.2–130.7] · p95 156.4 | -0.2% [-0.8 to +0.5] |
| Trail Rescue qualify | 242.8 [241.1–243.2] · p95 303.9 | 239.9 [239.7–241.7] · p95 297.1 | -0.6% [-1.3 to -0.5] | 109.4 [108.9–110.0] · p95 147.0 | 109.4 [109.2–110.0] · p95 147.6 | +0.5% [-0.7 to +0.5] |
| Ledger evidence | 103.3 [102.5–103.4] · p95 126.4 | 103.0 [102.8–103.2] · p95 127.8 | -0.2% [-0.5 to +0.5] | 30.7 [30.7–30.7] · p95 38.1 | 30.7 [30.6–30.8] · p95 38.4 | 0.0% [-0.3 to +0.3] |

### `dispatchView()` and the raw paths, piece by piece (DIRECT medians, µs, [lowest–highest run])

| Piece | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | Trail Rescue qualify | Ledger evidence | Trail Rescue refused | Ledger refused |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| raw `dispatch_view` path: `JSON.stringify(payload)` | 0.2 [0.2–0.2] | 0.2 [0.2–0.2] | 0.8 [0.7–0.8] | 0.4 [0.4–0.4] | 0.4 [0.4–0.5] | 0.4 [0.4–0.4] | 0.3 [0.3–0.3] | 0.4 [0.4–0.4] | 0.2 [0.2–0.2] |
| raw `dispatch_view` path: the `dispatch_view` call | 18.9 [18.8–19.1] | 21.0 [21.0–21.1] | 91.7 [91.4–91.9] | 52.5 [52.4–52.6] | 107.0 [106.9–107.4] | 88.5 [88.5–88.8] | 24.7 [24.6–24.8] | 6.2 [6.2–6.2] | 5.7 [5.7–5.8] |
| raw `dispatch_view` path: `JSON.parse` of the view | 12.4 [12.4–12.5] | 11.9 [11.9–12.0] | 21.6 [21.5–21.6] | 21.1 [20.8–21.3] | 21.9 [21.5–21.9] | 20.3 [20.2–20.7] | 5.6 [5.5–5.6] | - | - |
| **raw `dispatch_view` + `JSON.parse`** | 31.6 [31.4–31.8] | 33.3 [33.1–33.3] | 114.1 [114.1–114.6] | 74.7 [74.2–74.8] | 130.6 [130.2–130.7] | 109.4 [109.2–110.0] | 30.7 [30.6–30.8] | - | - |
| raw `dispatch_view_outcome` path: `JSON.stringify(payload)` | 0.2 [0.2–0.2] | 0.2 [0.2–0.2] | 0.8 [0.8–0.8] | 0.4 [0.4–0.4] | 0.5 [0.4–0.5] | 0.4 [0.4–0.4] | 0.3 [0.3–0.3] | 0.4 [0.4–0.4] | 0.2 [0.2–0.2] |
| raw `dispatch_view_outcome` path: the `dispatch_view_outcome` call | 18.9 [18.8–19.0] | 21.4 [21.3–21.6] | 91.5 [91.4–92.2] | 52.4 [52.2–52.9] | 108.0 [107.4–108.7] | 89.5 [89.4–90.7] | 24.2 [24.2–24.4] | 5.0 [5.0–5.2] | 4.8 [4.8–4.9] |
| raw `dispatch_view_outcome` path: `JSON.parse` of the envelope | 12.6 [12.4–12.6] | 12.2 [12.0–12.2] | 21.2 [21.2–21.4] | 21.0 [20.9–21.3] | 22.0 [21.8–22.1] | 20.7 [20.6–20.8] | 5.6 [5.6–5.7] | 0.7 [0.7–0.7] | 0.5 [0.5–0.6] |
| **raw `dispatch_view_outcome` + `JSON.parse`** | 31.7 [31.7–31.8] | 33.7 [33.6–34.0] | 114.2 [113.9–114.7] | 74.4 [74.4–74.7] | 131.3 [131.0–132.6] | 110.9 [110.7–111.8] | 30.2 [30.1–30.3] | 6.6 [6.4–6.7] | 5.6 [5.6–5.7] |
| pieces: the kit's `payloadText(payload)` | 0.4 [0.4–0.4] | 0.4 [0.4–0.5] | 1.9 [1.9–2.0] | 0.9 [0.9–0.9] | 1.1 [1.1–1.1] | 0.9 [0.9–0.9] | 0.9 [0.8–0.9] | 0.9 [0.9–0.9] | 0.5 [0.5–0.6] |
| pieces: the `dispatch_view_outcome` call | 19.2 [19.1–19.4] | 21.5 [21.5–22.0] | 92.5 [92.4–92.7] | 53.3 [53.2–53.4] | 109.5 [108.6–109.6] | 90.4 [90.0–91.1] | 24.5 [24.3–24.8] | 5.1 [5.1–5.2] | 5.0 [4.9–5.0] |
| pieces: `JSON.parse` of the envelope | 12.5 [12.5–12.6] | 12.0 [12.0–12.1] | 21.0 [20.9–21.1] | 20.2 [20.1–20.2] | 20.5 [20.5–20.6] | 18.8 [18.8–18.8] | 5.5 [5.4–5.5] | 0.7 [0.7–0.7] | 0.6 [0.5–0.6] |
| pieces: the kit's `validOutcome(outcome, 'view')` | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.2 [0.1–0.2] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] |
| **pieces: the four, first timer to last** | 32.2 [32.1–32.4] | 34.1 [34.1–34.6] | 116.5 [116.4–116.5] | 75.1 [75.0–75.2] | 131.6 [130.5–132.3] | 110.3 [109.7–111.0] | 31.1 [30.7–31.2] | 7.4 [7.4–7.5] | 6.2 [6.2–6.3] |
| pieces: `JSON.parse` of the view text alone, after the envelope's | 11.9 [11.8–11.9] | 11.4 [11.4–11.5] | 15.5 [15.4–15.5] | 17.0 [17.0–17.1] | 17.6 [17.5–17.7] | 16.9 [16.7–17.0] | 4.4 [4.4–4.5] | - | - |
| **kit `dispatchView()`** | 31.7 [31.6–31.9] | 33.9 [33.7–33.9] | 117.7 [116.3–120.7] | 76.2 [75.7–76.7] | 133.6 [132.9–135.1] | 112.9 [112.0–113.8] | 30.9 [30.6–31.0] | 7.4 [7.3–7.6] | 6.1 [6.0–6.2] |
| `dispatchView()` − the four pieces (paired by repeat; the wrapper's state and argument checks, and the timer reads between pieces) | -0.5 [-0.7 to -0.3] | -0.4 [-0.7 to -0.2] | +1.3 [-0.2 to +4.2] | +1.0 [+0.7 to +1.6] | +2.0 [+0.6 to +4.6] | +2.6 [+1.0 to +4.1] | -0.3 [-0.5 to +0.3] | 0.0 [-0.2 to +0.2] | -0.1 [-0.3 to 0.0] |
| `dispatchView()` − raw `dispatch_view` + `JSON.parse` (paired by repeat) | +0.2 [-0.1 to +0.3] | +0.6 [+0.4 to +0.8] | +3.6 [+2.2 to +6.1] | +1.9 [+1.0 to +2.0] | +3.4 [+2.2 to +4.5] | +3.7 [+2.6 to +3.8] | +0.2 [-0.1 to +0.3] | - | - |
| the kit's payload check: `payloadText` − `JSON.stringify` of the raw `dispatch_view_outcome` path (paired by repeat) | +0.2 [+0.2 to +0.2] | +0.2 [+0.2 to +0.3] | +1.1 [+1.1 to +1.2] | +0.5 [+0.5 to +0.5] | +0.6 [+0.6 to +0.7] | +0.5 [+0.5 to +0.5] | +0.6 [+0.5 to +0.6] | +0.5 [+0.5 to +0.5] | +0.3 [+0.3 to +0.4] |
| envelope against view: `JSON.parse` of the envelope − of the view, each the first parse on its own raw path (paired by repeat) | +0.1 [0.0 to +0.2] | +0.3 [0.0 to +0.3] | -0.4 [-0.4 to -0.1] | -0.1 [-0.4 to +0.5] | +0.2 [-0.1 to +0.5] | +0.5 [-0.1 to +0.5] | +0.1 [0.0 to +0.1] | - | - |
| the call: `dispatch_view_outcome` − `dispatch_view`, each on its own raw path (paired by repeat) | 0.0 [-0.3 to +0.2] | +0.4 [+0.2 to +0.6] | -0.2 [-0.5 to +0.8] | -0.2 [-0.2 to +0.4] | +1.1 [0.0 to +1.7] | +1.0 [+0.9 to +1.9] | -0.4 [-0.5 to -0.4] | -1.2 [-1.2 to -1.0] | -0.9 [-1.0 to -0.8] |

### Load per repeat (typeperf; a repeat is kept when its mean total CPU is at most 12% and no 1-s sample is above 20%; a session is used only when every repeat is kept)

| Repeat | from (UTC) | seconds | samples | total CPU mean | p95 | max | samples above 20% | kept |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 18:42:41 | 68.4 | 67 | 7.0% | 9.8% | 11.3% | 0 | yes |
| 2 | 18:43:49 | 68.4 | 68 | 7.1% | 9.2% | 13.2% | 0 | yes |
| 3 | 18:44:58 | 68.4 | 68 | 6.9% | 9.0% | 13.2% | 0 | yes |

