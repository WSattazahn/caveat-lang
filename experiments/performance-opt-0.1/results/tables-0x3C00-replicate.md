Run 2026-09-29T20-06-57-466-view-path; targets opt (d5938cc, reactive WebAssembly 0a19e4b85886), main (768275b, reactive WebAssembly e35944503272); mask 0x3C00, priority high; total CPU over the kept timed jobs mean 5.4%, highest repeat p95 8.6%, max 10.4%; 9 attempt(s) discarded by the job guard and run again.

### Headline, accepted events: the paths side by side (DIRECT, µs) and the saving (paired by repeat, µs)

Refused events are not in this table, nor in any saving, ratio or range about the accepted events: they are in "Refused events" below.

| Event | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` | saving: kit `dispatch()` + `view()` − `dispatchView()` | old ÷ new | gap: `dispatchView()` − raw `dispatch_view` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 150.5 [147.8–153.3] · p95 174.8 | 37.9 [37.3–38.7] · p95 41.5 | 37.3 [36.9–38.0] · p95 41.8 | 37.7 [37.1–37.8] · p95 42.9 | 112.3 [109.2–115.4] | 3.94× [3.82–4.06] | +0.4 [+0.2 to +1.4] |
| Glowcap state-changing tick | 142.4 [141.6–148.1] · p95 176.3 | 39.9 [39.2–40.6] · p95 45.2 | 39.5 [38.4–39.7] · p95 53.7 | 39.6 [38.0–40.1] · p95 55.1 | 103.2 [101.0–108.2] | 3.62× [3.49–3.71] | +0.7 [-0.4 to +1.5] |
| Trail Rescue evidence | 274.5 [269.5–275.6] · p95 329.9 | 135.5 [131.9–138.5] · p95 178.3 | 131.1 [128.5–132.4] · p95 167.6 | 130.7 [128.3–132.1] · p95 168.2 | 138.3 [131.0–142.8] | 2.03× [1.95–2.08] | +3.3 [+1.4 to +10.0] |
| Trail Rescue commit | 227.1 [224.0–228.4] · p95 272.6 | 88.9 [86.3–90.4] · p95 117.0 | 85.6 [83.3–86.2] · p95 109.5 | 86.2 [83.6–86.5] · p95 111.4 | 138.0 [133.7–140.0] | 2.56× [2.48–2.61] | +2.7 [+1.5 to +7.0] |
| Trail Rescue reopen | 295.6 [290.9–298.6] · p95 341.2 | 154.5 [149.7–155.4] · p95 182.6 | 147.8 [145.6–149.5] · p95 174.6 | 149.6 [147.8–151.3] · p95 175.7 | 142.4 [135.9–143.9] | 1.92× [1.88–1.95] | +5.0 [+2.3 to +9.4] |
| Trail Rescue qualify | 271.8 [267.9–273.0] · p95 324.2 | 131.7 [126.5–133.8] · p95 181.1 | 124.5 [122.3–125.9] · p95 167.7 | 126.4 [124.7–128.4] · p95 172.1 | 140.9 [134.1–144.9] | 2.07× [2.00–2.14] | +6.5 [+1.6 to +11.5] |
| Ledger evidence | 116.6 [115.1–118.3] · p95 139.5 | 35.6 [34.9–35.9] · p95 43.7 | 35.5 [34.7–36.2] · p95 43.2 | 34.9 [34.5–36.6] · p95 43.3 | 81.3 [79.2–82.7] | 3.28× [3.21–3.33] | +0.1 [-1.1 to +1.0] |

### Per accepted event, on each path (DIRECT, µs)

| Event | samples per run | existing: kit `dispatch()` + `view()` | new: kit `dispatchView()` | existing: raw `dispatch_view` + `JSON.parse` | new: raw `dispatch_view_outcome` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 28182/28182/28182/28182/28182 | 150.5 [147.8–153.3] · p95 174.8 | 37.9 [37.3–38.7] · p95 41.5 | 37.3 [36.9–38.0] · p95 41.8 | 37.7 [37.1–37.8] · p95 42.9 |
| Glowcap state-changing tick | 1809/1809/1809/1809/1809 | 142.4 [141.6–148.1] · p95 176.3 | 39.9 [39.2–40.6] · p95 45.2 | 39.5 [38.4–39.7] · p95 53.7 | 39.6 [38.0–40.1] · p95 55.1 |
| Trail Rescue evidence | 2640/2640/2640/2640/2640 | 274.5 [269.5–275.6] · p95 329.9 | 135.5 [131.9–138.5] · p95 178.3 | 131.1 [128.5–132.4] · p95 167.6 | 130.7 [128.3–132.1] · p95 168.2 |
| Trail Rescue commit | 1440/1440/1440/1440/1440 | 227.1 [224.0–228.4] · p95 272.6 | 88.9 [86.3–90.4] · p95 117.0 | 85.6 [83.3–86.2] · p95 109.5 | 86.2 [83.6–86.5] · p95 111.4 |
| Trail Rescue reopen | 600/600/600/600/600 | 295.6 [290.9–298.6] · p95 341.2 | 154.5 [149.7–155.4] · p95 182.6 | 147.8 [145.6–149.5] · p95 174.6 | 149.6 [147.8–151.3] · p95 175.7 |
| Trail Rescue qualify | 360/360/360/360/360 | 271.8 [267.9–273.0] · p95 324.2 | 131.7 [126.5–133.8] · p95 181.1 | 124.5 [122.3–125.9] · p95 167.7 | 126.4 [124.7–128.4] · p95 172.1 |
| Ledger evidence | 4500/4500/4500/4500/4500 | 116.6 [115.1–118.3] · p95 139.5 | 35.6 [34.9–35.9] · p95 43.7 | 35.5 [34.7–36.2] · p95 43.2 | 34.9 [34.5–36.6] · p95 43.3 |

### The saving, accepted events (paired by repeat, µs)

| Event | kit `dispatch()` + `view()` − `dispatchView()` | share of the old path | old ÷ new | `dispatchView()` − raw `dispatch_view` + `JSON.parse` | `dispatchView()` − raw `dispatch_view_outcome` + `JSON.parse` |
| --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 112.3 [109.2–115.4] | 74.6% [73.8–75.4] | 3.94× [3.82–4.06] | +0.4 [+0.2 to +1.4] | +0.2 [-0.2 to +1.6] |
| Glowcap state-changing tick | 103.2 [101.0–108.2] | 72.3% [71.3–73.1] | 3.62× [3.49–3.71] | +0.7 [-0.4 to +1.5] | +0.3 [-0.7 to +2.6] |
| Trail Rescue evidence | 138.3 [131.0–142.8] | 50.6% [48.6–51.9] | 2.03× [1.95–2.08] | +3.3 [+1.4 to +10.0] | +5.2 [+1.4 to +7.8] |
| Trail Rescue commit | 138.0 [133.7–140.0] | 61.0% [59.7–61.6] | 2.56× [2.48–2.61] | +2.7 [+1.5 to +7.0] | +3.9 [+0.9 to +5.3] |
| Trail Rescue reopen | 142.4 [135.9–143.9] | 48.0% [46.7–48.8] | 1.92× [1.88–1.95] | +5.0 [+2.3 to +9.4] | +4.1 [+1.4 to +6.1] |
| Trail Rescue qualify | 140.9 [134.1–144.9] | 51.6% [50.1–53.2] | 2.07× [2.00–2.14] | +6.5 [+1.6 to +11.5] | +4.4 [-0.9 to +7.4] |
| Ledger evidence | 81.3 [79.2–82.7] | 69.5% [68.8–70.0] | 3.28× [3.21–3.33] | +0.1 [-1.1 to +1.0] | +0.7 [-1.7 to +1.2] |

### Refused events (DIRECT, µs; savings paired by repeat)

Not headline figures. A refusal leaves the view as it was, so a host that already holds the view need not call `view()` after one: against kit `dispatch()` alone, `dispatchView()` saves only 1.3–1.6 µs, and its saving of 14.2–28.4 µs against kit `dispatch()` + `view()` assumes the host would also have called `view()`. Raw `dispatch_view` throws on a refusal, where `dispatchView()` returns the structured refusal, so those two paths are not equivalent contracts; its cell is the throwing call alone.

| Event | samples per run | existing: kit `dispatch()` + `view()` | existing: kit `dispatch()` alone | new: kit `dispatchView()` | existing: raw `dispatch_view` (throws) | new: raw `dispatch_view_outcome` + `JSON.parse` | kit `dispatch()` + `view()` − `dispatchView()` | kit `dispatch()` alone − `dispatchView()` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Trail Rescue refused | 1320/1320/1320/1320/1320 | 36.7 [35.9–37.1] · p95 48.9 | 10.0 [9.9–10.2] · p95 14.8 | 8.4 [8.3–8.6] · p95 12.6 | 6.8 [6.6–7.0] · p95 12.1 | 7.3 [7.1–7.3] · p95 11.2 | 28.4 [27.6–28.5] | +1.6 [+1.5 to +1.8] |
| Ledger refused | 1500/1500/1500/1500/1500 | 21.2 [21.0–21.4] · p95 25.8 | 8.3 [8.2–8.3] · p95 10.4 | 7.0 [6.8–7.1] · p95 8.4 | 6.6 [6.4–6.7] · p95 8.1 | 6.5 [6.5–6.9] · p95 8.2 | 14.2 [14.0–14.4] | +1.3 [+1.2 to +1.4] |

### The existing paths, main against opt (DIRECT medians, µs; change paired by repeat)

| Event | kit `dispatch()` + `view()`, main | opt | change | raw `dispatch_view` + `JSON.parse`, main | opt | change |
| --- | --- | --- | --- | --- | --- | --- |
| Glowcap idle tick | 148.9 [147.4–153.8] · p95 167.8 | 150.5 [147.8–153.3] · p95 174.8 | -0.7% [-2.1 to +3.1] | 37.0 [37.0–38.5] · p95 41.0 | 37.3 [36.9–38.0] · p95 41.8 | 0.0% [-2.6 to +0.8] |
| Glowcap state-changing tick | 142.0 [138.3–143.9] · p95 177.9 | 142.4 [141.6–148.1] · p95 176.3 | +1.3% [-0.4 to +4.3] | 39.4 [38.8–40.5] · p95 52.0 | 39.5 [38.4–39.7] · p95 53.7 | 0.0% [-2.0 to +0.3] |
| Trail Rescue evidence | 271.7 [268.7–276.2] · p95 325.8 | 274.5 [269.5–275.6] · p95 329.9 | +0.3% [-0.2 to +1.0] | 128.8 [128.1–133.9] · p95 166.5 | 131.1 [128.5–132.4] · p95 167.6 | -0.2% [-2.1 to +3.1] |
| Trail Rescue commit | 224.3 [221.8–229.9] · p95 271.8 | 227.1 [224.0–228.4] · p95 272.6 | +0.5% [-0.7 to +1.6] | 83.8 [83.2–87.5] · p95 108.2 | 85.6 [83.3–86.2] · p95 109.5 | +0.1% [-1.9 to +2.9] |
| Trail Rescue reopen | 292.6 [290.4–299.9] · p95 334.3 | 295.6 [290.9–298.6] · p95 341.2 | 0.0% [-0.4 to +1.1] | 147.3 [145.4–153.3] · p95 173.7 | 147.8 [145.6–149.5] · p95 174.6 | +0.3% [-3.6 to +1.5] |
| Trail Rescue qualify | 270.2 [267.8–277.7] · p95 329.3 | 271.8 [267.9–273.0] · p95 324.2 | +0.1% [-1.7 to +1.2] | 123.7 [123.0–130.1] · p95 166.1 | 124.5 [122.3–125.9] · p95 167.7 | -0.3% [-4.3 to +1.2] |
| Ledger evidence | 117.1 [115.5–118.5] · p95 139.1 | 116.6 [115.1–118.3] · p95 139.5 | -0.3% [-1.2 to +0.3] | 35.6 [34.8–37.1] · p95 43.7 | 35.5 [34.7–36.2] · p95 43.2 | -0.3% [-4.3 to +1.7] |

### `dispatchView()` and the raw paths, piece by piece (DIRECT medians, µs, [lowest–highest run])

| Piece | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | Trail Rescue qualify | Ledger evidence | Trail Rescue refused | Ledger refused |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| raw `dispatch_view` path: `JSON.stringify(payload)` | 0.2 [0.2–0.3] | 0.2 [0.2–0.2] | 0.8 [0.7–0.8] | 0.4 [0.4–0.4] | 0.4 [0.4–0.5] | 0.4 [0.3–0.4] | 0.3 [0.3–0.3] | 0.4 [0.4–0.4] | 0.2 [0.2–0.2] |
| raw `dispatch_view` path: the `dispatch_view` call | 22.3 [22.0–22.6] | 25.0 [24.3–25.1] | 105.0 [103.5–106.8] | 60.7 [58.6–60.8] | 121.9 [120.5–123.7] | 101.1 [99.6–102.5] | 28.6 [27.9–29.4] | 6.8 [6.6–7.0] | 6.6 [6.4–6.7] |
| raw `dispatch_view` path: `JSON.parse` of the view | 14.7 [14.5–15.0] | 14.1 [13.8–14.4] | 24.2 [23.6–24.6] | 23.8 [23.4–24.1] | 24.7 [24.3–24.9] | 22.9 [22.6–23.4] | 6.4 [6.2–6.5] | - | - |
| **raw `dispatch_view` + `JSON.parse`** | 37.3 [36.9–38.0] | 39.5 [38.4–39.7] | 131.1 [128.5–132.4] | 85.6 [83.3–86.2] | 147.8 [145.6–149.5] | 124.5 [122.3–125.9] | 35.5 [34.7–36.2] | - | - |
| raw `dispatch_view_outcome` path: `JSON.stringify(payload)` | 0.3 [0.2–0.3] | 0.2 [0.2–0.3] | 0.8 [0.7–0.8] | 0.4 [0.4–0.4] | 0.4 [0.4–0.5] | 0.4 [0.3–0.4] | 0.3 [0.3–0.3] | 0.4 [0.4–0.4] | 0.2 [0.2–0.2] |
| raw `dispatch_view_outcome` path: the `dispatch_view_outcome` call | 22.6 [22.1–22.7] | 25.1 [23.9–25.6] | 105.9 [102.9–106.4] | 60.6 [58.6–60.9] | 123.6 [121.5–124.3] | 102.4 [101.2–104.1] | 27.9 [27.6–29.6] | 5.6 [5.5–5.7] | 5.7 [5.6–6.0] |
| raw `dispatch_view_outcome` path: `JSON.parse` of the envelope | 14.8 [14.6–14.8] | 14.2 [13.8–14.4] | 24.0 [23.6–24.4] | 24.1 [23.7–24.6] | 24.7 [24.6–25.3] | 23.1 [23.0–23.6] | 6.5 [6.4–6.5] | 0.8 [0.8–0.8] | 0.6 [0.6–0.6] |
| **raw `dispatch_view_outcome` + `JSON.parse`** | 37.7 [37.1–37.8] | 39.6 [38.0–40.1] | 130.7 [128.3–132.1] | 86.2 [83.6–86.5] | 149.6 [147.8–151.3] | 126.4 [124.7–128.4] | 34.9 [34.5–36.6] | 7.3 [7.1–7.3] | 6.5 [6.5–6.9] |
| pieces: the kit's `payloadText(payload)` | 0.5 [0.4–0.5] | 0.5 [0.5–0.5] | 2.1 [2.1–2.1] | 1.0 [1.0–1.0] | 1.1 [1.1–1.2] | 1.0 [1.0–1.1] | 1.0 [1.0–1.1] | 1.0 [1.0–1.0] | 0.6 [0.6–0.7] |
| pieces: the `dispatch_view_outcome` call | 23.5 [22.5–24.2] | 25.4 [24.6–27.2] | 105.6 [105.1–106.8] | 60.6 [60.0–61.0] | 124.1 [123.7–126.3] | 103.0 [102.8–103.9] | 28.5 [27.3–28.6] | 5.7 [5.6–5.8] | 5.7 [5.5–5.8] |
| pieces: `JSON.parse` of the envelope | 15.0 [14.6–15.6] | 14.0 [14.0–15.0] | 23.4 [23.2–23.7] | 22.6 [22.6–22.9] | 23.2 [23.1–23.5] | 21.3 [21.2–21.4] | 6.3 [6.2–6.4] | 0.8 [0.8–0.9] | 0.6 [0.6–0.7] |
| pieces: the kit's `validOutcome(outcome, 'view')` | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.2 [0.1–0.2] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] | 0.1 [0.1–0.1] |
| **pieces: the four, first timer to last** | 39.2 [37.8–40.4] | 40.2 [39.3–42.9] | 131.8 [131.2–133.5] | 84.9 [84.5–86.0] | 149.6 [149.2–151.3] | 125.7 [125.4–126.8] | 36.1 [34.7–36.2] | 8.4 [8.2–8.4] | 7.2 [6.9–7.3] |
| pieces: `JSON.parse` of the view text alone, after the envelope's | 14.2 [13.9–14.9] | 13.3 [13.2–14.2] | 17.3 [17.3–17.6] | 19.1 [19.0–19.5] | 19.8 [19.8–20.2] | 18.9 [18.8–19.0] | 5.1 [5.0–5.1] | - | - |
| **kit `dispatchView()`** | 37.9 [37.3–38.7] | 39.9 [39.2–40.6] | 135.5 [131.9–138.5] | 88.9 [86.3–90.4] | 154.5 [149.7–155.4] | 131.7 [126.5–133.8] | 35.6 [34.9–35.9] | 8.4 [8.3–8.6] | 7.0 [6.8–7.1] |
| `dispatchView()` − the four pieces (paired by repeat; the wrapper's state and argument checks, and the timer reads between pieces) | -1.3 [-2.5 to +0.9] | -0.1 [-3.7 to +0.4] | +3.7 [+0.7 to +5.2] | +4.0 [+1.8 to +4.9] | +4.1 [+0.5 to +5.0] | +5.3 [+1.1 to +8.1] | -0.3 [-0.6 to +0.4] | 0.0 [-0.1 to +0.2] | -0.2 [-0.2 to -0.1] |
| `dispatchView()` − raw `dispatch_view` + `JSON.parse` (paired by repeat) | +0.4 [+0.2 to +1.4] | +0.7 [-0.4 to +1.5] | +3.3 [+1.4 to +10.0] | +2.7 [+1.5 to +7.0] | +5.0 [+2.3 to +9.4] | +6.5 [+1.6 to +11.5] | +0.1 [-1.1 to +1.0] | - | - |
| the kit's payload check: `payloadText` − `JSON.stringify` of the raw `dispatch_view_outcome` path (paired by repeat) | +0.2 [+0.1 to +0.3] | +0.3 [+0.2 to +0.3] | +1.3 [+1.3 to +1.4] | +0.6 [+0.6 to +0.6] | +0.7 [+0.7 to +0.8] | +0.7 [+0.6 to +0.7] | +0.7 [+0.7 to +0.8] | +0.6 [+0.6 to +0.6] | +0.4 [+0.4 to +0.5] |
| envelope against view: `JSON.parse` of the envelope − of the view, each the first parse on its own raw path (paired by repeat) | -0.1 [-0.2 to +0.3] | -0.1 [-0.3 to +0.4] | -0.1 [-0.4 to +0.2] | +0.4 [0.0 to +0.7] | +0.3 [0.0 to +0.4] | +0.3 [+0.1 to +0.5] | +0.1 [0.0 to +0.2] | - | - |
| the call: `dispatch_view_outcome` − `dispatch_view`, each on its own raw path (paired by repeat) | +0.2 [-0.2 to +0.6] | +0.5 [-1.2 to +0.8] | -0.4 [-3.9 to +2.5] | -0.2 [-2.2 to +2.3] | +0.2 [-1.2 to +3.8] | +1.6 [+0.2 to +2.8] | -0.3 [-1.2 to +0.6] | -1.1 [-1.4 to -1.0] | -0.8 [-1.0 to -0.7] |

### Load per repeat (typeperf every second; a repeat is kept when, over the samples covering its timed jobs, the mean total CPU is at most 12% and no 1-s sample is above 20%, and every one of its timed jobs keeps the job guard's rule; a session is used only when every repeat is kept)

| Repeat | from (UTC) | seconds | timed jobs kept | attempts discarded | samples | total CPU mean | p95 | max | samples above 20% | kept |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 20:07:10 | 100.3 | 21 of 21 | 1 | 89 | 5.5% | 8.2% | 10.4% | 0 | yes |
| 2 | 20:08:52 | 112.3 | 21 of 21 | 2 | 88 | 5.5% | 7.6% | 9.7% | 0 | yes |
| 3 | 20:10:45 | 113.5 | 21 of 21 | 2 | 88 | 5.3% | 8.0% | 10.4% | 0 | yes |
| 4 | 20:12:39 | 108.2 | 21 of 21 | 2 | 89 | 5.3% | 8.3% | 9.0% | 0 | yes |
| 5 | 20:14:28 | 108.4 | 21 of 21 | 2 | 88 | 5.5% | 8.6% | 10.2% | 0 | yes |

### Load per timed job (a timed job is kept when, over the 1-s samples covering it, the mean total CPU is at most 12% and no sample is above 20%, and in no sample does System or MsMpEng use more than 30% of one core or are more than 2 cores busy outside the pinned processors; otherwise it is discarded and run again in place (up to 5 attempts), after 3 quiet samples in a row)

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

