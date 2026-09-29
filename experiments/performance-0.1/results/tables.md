## Control: the rc.4 and main builds

Byte-identical WebAssembly and identical runtime sources, so this is a control, not a comparison: each row gives both targets' median over runs, and the difference against the run-to-run range.

| Workload | Engine | Mode / operation | main-local µs [runs] | rc4-local µs [runs] | rc4 − main | largest run range | within noise |
| --- | --- | --- | --- | --- | --- | --- | --- |
| glowcap-replay | native | apply / apply | 13.1 [13.0–13.8] | 13.1 [13.1–13.8] | 0.00 (0.00%) | 0.80 | yes |
| glowcap-replay | native | web.dispatch_view / web.dispatch_view | 20.0 [20.0–21.3] | 20.1 [20.0–21.3] | 0.10 (0.50%) | 1.30 | yes |
| glowcap-replay | native | web.dispatch_outcome / web.dispatch_outcome | 64.7 [64.4–70.2] | 64.0 [63.6–68.2] | -0.70 (-1.08%) | 5.80 | yes |
| glowcap-replay | native | read / view | 2.60 [2.40–2.60] | 2.50 [2.50–2.70] | -0.10 (-3.85%) | 0.20 | yes |
| glowcap-replay | native | read / snapshot | 26.2 [24.5–27.0] | 24.7 [24.6–26.7] | -1.50 (-5.73%) | 2.50 | yes |
| glowcap-replay | wasm | abi.dispatch_view / abi.exec | 22.6 [21.2–22.6] | 22.5 [21.8–22.7] | -0.10 (-0.44%) | 1.40 | yes |
| glowcap-replay | wasm | raw.dispatch_view / raw.dispatch_view+parse | 43.9 [41.0–45.3] | 43.7 [42.2–43.9] | -0.20 (-0.46%) | 4.30 | yes |
| glowcap-replay | wasm | kit / kit.dispatch+view | 166.2 [163.9–175.6] | 170.4 [164.0–171.7] | 4.20 (2.53%) | 11.7 | yes |
| glowcap-replay | wasm | published-method / adapter.dispatch+view | 46.7 [43.8–47.2] | 46.6 [46.0–46.7] | -0.10 (-0.21%) | 3.40 | yes |
| ledger-session | native | apply / apply | 16.2 [14.9–16.7] | 14.7 [14.6–14.9] | -1.50 (-9.26%) | 1.80 | yes |
| ledger-session | native | web.dispatch_view / web.dispatch_view | 24.2 [23.1–24.9] | 25.0 [22.8–25.8] | 0.80 (3.31%) | 3.00 | yes |
| ledger-session | native | web.dispatch_outcome / web.dispatch_outcome | 56.6 [56.1–63.7] | 56.7 [56.4–60.0] | 0.10 (0.18%) | 7.60 | yes |
| ledger-session | native | read / view | 3.10 [3.10–3.10] | 3.10 [3.10–3.40] | 0.00 (0.00%) | 0.30 | yes |
| ledger-session | native | read / snapshot | 17.3 [17.1–17.6] | 17.3 [17.1–18.7] | 0.00 (0.00%) | 1.60 | yes |
| ledger-session | wasm | abi.dispatch_view / abi.exec | 27.6 [27.6–28.3] | 27.1 [26.9–27.1] | -0.50 (-1.81%) | 0.70 | yes |
| ledger-session | wasm | raw.dispatch_view / raw.dispatch_view+parse | 38.1 [37.6–38.7] | 37.9 [37.8–38.3] | -0.20 (-0.52%) | 1.10 | yes |
| ledger-session | wasm | kit / kit.dispatch+view | 124.5 [124.0–125.8] | 125.9 [124.1–126.1] | 1.40 (1.12%) | 2.00 | yes |
| trail-rescue-scenarios | native | apply / apply | 46.7 [46.1–47.3] | 47.4 [46.6–50.6] | 0.70 (1.50%) | 4.00 | yes |
| trail-rescue-scenarios | native | web.dispatch_view / web.dispatch_view | 58.4 [57.7–59.4] | 58.0 [57.6–60.5] | -0.40 (-0.68%) | 2.90 | yes |
| trail-rescue-scenarios | native | web.dispatch_outcome / web.dispatch_outcome | 117.2 [116.2–125.8] | 117.5 [115.6–118.2] | 0.30 (0.26%) | 9.60 | yes |
| trail-rescue-scenarios | native | read / view | 1.30 [1.30–1.30] | 1.40 [1.30–1.40] | 0.10 (7.69%) | 0.10 | yes |
| trail-rescue-scenarios | native | read / snapshot | 29.9 [29.5–29.9] | 31.8 [29.8–32.0] | 1.90 (6.35%) | 2.20 | yes |
| trail-rescue-scenarios | wasm | abi.dispatch_view / abi.exec | 70.6 [70.0–72.1] | 71.1 [70.5–71.2] | 0.50 (0.71%) | 2.10 | yes |
| trail-rescue-scenarios | wasm | raw.dispatch_view / raw.dispatch_view+parse | 120.1 [118.9–123.7] | 120.7 [119.2–121.5] | 0.60 (0.50%) | 4.80 | yes |
| trail-rescue-scenarios | wasm | kit / kit.dispatch+view | 260.3 [258.8–261.1] | 259.1 [257.2–264.4] | -1.20 (-0.46%) | 7.20 | yes |

## Background load during each timed session

Mean total CPU use (24 logical processors; the benchmark itself is about 4%) while each job ran, averaged over the jobs of each workload in each repeat. From each run's `run.log` (`run-log.txt` under `results/`) and `load.csv` (typeperf every 5 s).

| Session | Workload | repeat 1 | repeat 2 | repeat 3 |
| --- | --- | --- | --- | --- |
| baseline (primary) | glowcap-replay | 9.34% | 8.87% | 9.10% |
| baseline (primary) | glowcap-resume | 10.4% | 10.7% | 9.66% |
| baseline (primary) | glowcap-unbound | 11.0% | 9.42% | 9.20% |
| baseline (primary) | glowcap-scaled-16 | 10.1% | 9.30% | 9.41% |
| baseline (primary) | glowcap-scaled-64 | 8.58% | 8.50% | 9.59% |
| baseline (primary) | ledger-session | 9.60% | 9.31% | 9.58% |
| baseline (primary) | trail-rescue-scenarios | 8.74% | 8.46% | 8.20% |
| baseline (primary) | **whole run** | mean 8.7%, p95 10.6%, max 21.6% |  |  |
| baseline 02:18 UTC | glowcap-replay | 23.4% | 22.0% | 28.0% |
| baseline 02:18 UTC | glowcap-resume | 17.2% | 23.0% | 20.2% |
| baseline 02:18 UTC | glowcap-unbound | 21.5% | 17.9% | 15.7% |
| baseline 02:18 UTC | glowcap-scaled-16 | 21.2% | 19.9% | 20.1% |
| baseline 02:18 UTC | glowcap-scaled-64 | 20.8% | 14.9% | 38.5% |
| baseline 02:18 UTC | ledger-session | 39.9% | 34.8% | 38.9% |
| baseline 02:18 UTC | trail-rescue-scenarios | 25.9% | 33.8% | 18.0% |
| baseline 02:18 UTC | **whole run** | mean 27.3%, p95 70.2%, max 91.2% |  |  |
| baseline 03:53 UTC | glowcap-replay | 10.7% | 27.5% | 81.8% |
| baseline 03:53 UTC | glowcap-resume | 9.66% | 50.4% | 93.2% |
| baseline 03:53 UTC | glowcap-unbound | 8.70% | 52.3% | 67.6% |
| baseline 03:53 UTC | glowcap-scaled-16 | 10.1% | 53.4% | 99.2% |
| baseline 03:53 UTC | glowcap-scaled-64 | 42.5% | 62.3% | 90.0% |
| baseline 03:53 UTC | ledger-session | 23.6% | 47.3% | 100.0% |
| baseline 03:53 UTC | trail-rescue-scenarios | 25.5% | 54.6% | 84.3% |
| baseline 03:53 UTC | **whole run** | mean 57%, p95 100%, max 100% |  |  |
| baseline 05:20 UTC | glowcap-replay | 8.47% | 8.63% | 8.55% |
| baseline 05:20 UTC | ledger-session | 9.51% | 9.63% | 9.19% |
| baseline 05:20 UTC | trail-rescue-scenarios | 9.40% | 8.46% | 8.18% |
| baseline 05:20 UTC | **whole run** | mean 8.7%, p95 10.7%, max 13.1% |  |  |
| instrumented | glowcap-replay | 11.6% | 11.2% | 11.7% |
| instrumented | glowcap-unbound | 11.4% | 10.7% | 12.7% |
| instrumented | glowcap-resume | 11.0% | 16.2% | 19.9% |
| instrumented | ledger-session | 11.9% | 12.6% | 11.8% |
| instrumented | trail-rescue-scenarios | 11.0% | 11.7% | 9.82% |
| instrumented | **whole run** | mean 11.3%, p95 14.9%, max 21.1% |  |  |

## Replication: the same suite in other sessions

The primary baseline (2026-09-29T05-29-28-544-baseline; total CPU during the run mean 8.7%, p95 10.6%) against the other complete runs of the same suite (2026-09-29T02-18-48-081-baseline: mean 27.3%, p95 70.2%; 2026-09-29T03-53-23-652-baseline: mean 57%, p95 100%; 2026-09-29T05-20-51-056-baseline: mean 8.7%, p95 10.7%). Each cell: median over 3 runs [lowest–highest run], and the change from the primary.

| Target | Workload | Operation | primary µs | 02:18 UTC session µs | 03:53 UTC session µs | 05:20 UTC session µs |
| --- | --- | --- | --- | --- | --- | --- |
| main-local | glowcap-replay | native apply / apply | 13.1 [13.0–13.8] | 18.6 [15.2–18.7] (+42.0%) | 13.9 [13.7–18.5] (+6.11%) | - |
| main-local | glowcap-replay | native web.dispatch_view / web.dispatch_view | 20.0 [20.0–21.3] | 27.6 [25.7–29.8] (+38.0%) | 21.8 [21.5–26.4] (+9.00%) | - |
| main-local | glowcap-replay | native read / view | 2.60 [2.40–2.60] | 2.80 [2.50–3.60] (+7.69%) | 2.60 [2.60–2.70] (+0.00%) | - |
| main-local | glowcap-replay | native read / snapshot | 26.2 [24.5–27.0] | 29.2 [25.8–36.6] (+11.5%) | 26.1 [26.1–28.2] (-0.38%) | - |
| main-local | glowcap-replay | wasm published-method / adapter.dispatch+view | 46.7 [43.8–47.2] | 53.5 [46.4–66.0] (+14.6%) | 51.4 [44.0–62.9] (+10.1%) | 45.3 [44.5–46.5] (-3.00%) |
| main-local | glowcap-replay | wasm abi.dispatch_view / abi.exec | 22.6 [21.2–22.6] | 24.8 [21.3–25.8] (+9.73%) | 23.3 [22.1–30.8] (+3.10%) | - |
| main-local | glowcap-replay | wasm raw.dispatch_view / js.parse_view | 17.1 [16.1–17.9] | 19.1 [17.0–24.5] (+11.7%) | 18.1 [16.3–23.1] (+5.85%) | 16.8 [16.2–17.0] (-1.75%) |
| main-local | glowcap-replay | wasm kit / kit.dispatch+view | 166.2 [163.9–175.6] | 188.9 [179.9–220.5] (+13.7%) | 185.0 [169.9–236.1] (+11.3%) | 171.6 [171.4–173.7] (+3.25%) |
| main-local | trail-rescue-scenarios | native apply / apply | 46.7 [46.1–47.3] | 58.7 [50.0–65.2] (+25.7%) | 68.0 [64.3–74.6] (+45.6%) | - |
| main-local | trail-rescue-scenarios | wasm abi.dispatch_view / abi.exec | 70.6 [70.0–72.1] | 75.5 [72.1–81.0] (+6.94%) | 102.3 [72.7–102.6] (+44.9%) | - |
| main-local | trail-rescue-scenarios | wasm kit / kit.dispatch+view | 260.3 [258.8–261.1] | 313.9 [299.0–346.1] (+20.6%) | 348.6 [292.3–387.6] (+33.9%) | 268.3 [267.3–269.2] (+3.07%) |
| main-local | ledger-session | wasm abi.dispatch_view / abi.exec | 27.6 [27.6–28.3] | 35.2 [34.6–42.9] (+27.5%) | 35.2 [34.7–40.1] (+27.5%) | - |
| main-local | ledger-session | wasm kit / kit.dispatch+view | 124.5 [124.0–125.8] | 165.0 [134.0–168.2] (+32.5%) | 158.4 [152.3–189.7] (+27.2%) | 132.3 [131.6–133.5] (+6.27%) |
| rc4-published | glowcap-replay | wasm published-method / adapter.dispatch+view | 46.8 [46.2–47.0] | 51.2 [46.4–68.8] (+9.40%) | 46.5 [43.7–62.6] (-0.64%) | 46.1 [44.3–46.3] (-1.50%) |
| rc4-published | glowcap-replay | wasm abi.dispatch_view / abi.exec | 22.8 [22.6–23.3] | 25.3 [21.8–25.4] (+11.0%) | 24.8 [22.4–29.3] (+8.77%) | - |
| rc4-published | glowcap-replay | wasm raw.dispatch_view / js.parse_view | 17.1 [16.8–17.2] | 24.5 [16.5–25.4] (+43.3%) | 18.1 [16.3–22.2] (+5.85%) | 16.7 [16.5–17.1] (-2.34%) |
| rc4-published | glowcap-replay | wasm kit / kit.dispatch+view | 163.5 [162.7–171.2] | 176.7 [172.4–205.8] (+8.07%) | 180.6 [172.0–235.9] (+10.5%) | 171.0 [169.6–173.5] (+4.59%) |
| rc4-published | trail-rescue-scenarios | wasm abi.dispatch_view / abi.exec | 70.6 [68.8–71.0] | 74.6 [72.8–77.7] (+5.67%) | 91.7 [74.5–108.5] (+29.9%) | - |
| rc4-published | trail-rescue-scenarios | wasm kit / kit.dispatch+view | 261.2 [260.9–263.0] | 303.4 [293.7–342.1] (+16.2%) | 349.7 [312.3–382.8] (+33.9%) | 268.3 [265.7–269.6] (+2.72%) |
| rc4-published | ledger-session | wasm abi.dispatch_view / abi.exec | 27.0 [27.0–27.9] | 34.0 [30.9–40.7] (+25.9%) | 35.2 [32.6–40.6] (+30.4%) | - |
| rc4-published | ledger-session | wasm kit / kit.dispatch+view | 125.8 [125.4–126.2] | 164.2 [130.0–165.4] (+30.5%) | 155.6 [147.5–189.6] (+23.7%) | 131.4 [129.7–132.9] (+4.45%) |

## Results per build and operation (questions a–h)

All events of each workload, pooled per run; median over the 3 runs [lowest–highest run] · median p95. DIRECT. Native rows exist only for the trees; the published package and the historical runtime are WebAssembly only.

### glowcap-replay

| Question | Engine, mode / operation | main-local (i1 for the bench row) | rc4-local | rc4-published | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- |
| (a) transaction only, numeric parameters | native apply / apply | 13.1 [13.0–13.8] · p95 16.1 | 13.1 [13.1–13.8] · p95 16.0 | - | - |
| (a/b) payload + transaction, no reporting (i1) | native bench.dispatch_only / bench.dispatch_only | 15.2 [14.6–15.9] · p95 18.2 | - | - | - |
| (b) + view built, not serialized | native dispatch_view_json / dispatch_view_json | 15.9 [15.8–16.8] · p95 18.4 | 15.8 [15.7–16.7] · p95 18.1 | - | - |
| (b) + full snapshot built (legacy dispatch_json) | native dispatch_json / dispatch_json | 36.6 [36.3–38.6] · p95 40.7 | 36.8 [36.6–39.0] · p95 40.2 | - | - |
| (b) + outcome with snapshot built (dispatch_outcome_json) | native dispatch_outcome_json / dispatch_outcome_json | 38.5 [36.7–39.3] · p95 41.9 | 36.5 [36.3–39.4] · p95 40.1 | - | - |
| (c) view() alone | native read / view | 2.60 [2.40–2.60] · p95 2.90 | 2.50 [2.50–2.70] · p95 2.70 | - | - |
| (c) view JSON | native read / view.serialize | 5.00 [4.70–5.10] · p95 5.40 | 4.60 [4.60–5.00] · p95 5.00 | - | - |
| (d) snapshot() alone | native read / snapshot | 26.2 [24.5–27.0] · p95 28.4 | 24.7 [24.6–26.7] · p95 26.7 | - | - |
| (d) snapshot JSON | native read / snapshot.serialize | 15.9 [15.0–16.2] · p95 17.3 | 14.8 [14.7–16.0] · p95 16.1 | - | - |
| session clone (upper bound on the transaction copy) | native read / clone | 7.60 [7.10–7.60] · p95 8.10 | 7.00 [7.00–7.60] · p95 7.60 | - | - |
| (e) exported dispatch_view | native web.dispatch_view / web.dispatch_view | 20.0 [20.0–21.3] · p95 22.5 | 20.1 [20.0–21.3] · p95 22.8 | - | - |
| (8) exported dispatch_outcome | native web.dispatch_outcome / web.dispatch_outcome | 64.7 [64.4–70.2] · p95 69.2 | 64.0 [63.6–68.2] · p95 69.0 | - | - |
| (f) save | native web.read / web.save | 16.8 [15.5–17.3] · p95 17.9 | 16.5 [15.4–16.6] · p95 17.7 | - | - |
| (e) published method, dispatch + view | wasm published-method / adapter.dispatch+view | 46.7 [43.8–47.2] · p95 56.6 | 46.6 [46.0–46.7] · p95 56.1 | 46.8 [46.2–47.0] · p95 57.2 | 44.8 [41.3–45.1] · p95 53.2 |
| (e) adapter dispatch + view, warmed | wasm adapter / adapter.dispatch+view | 46.0 [43.1–46.1] · p95 52.1 | 45.6 [43.4–46.3] · p95 52.8 | 46.3 [45.9–46.5] · p95 52.7 | 44.4 [41.5–44.8] · p95 50.4 |
| (e) dispatch_view + JSON.parse | wasm raw.dispatch_view / raw.dispatch_view+parse | 43.9 [41.0–45.3] · p95 49.0 | 43.7 [42.2–43.9] · p95 48.9 | 43.8 [43.5–43.9] · p95 48.9 | 42.4 [39.6–42.5] · p95 47.1 |
| (e) dispatch_view, wasm execution only | wasm abi.dispatch_view / abi.exec | 22.6 [21.2–22.6] · p95 26.2 | 22.5 [21.8–22.7] · p95 26.1 | 22.8 [22.6–23.3] · p95 26.1 | 21.3 [20.5–21.7] · p95 26.6 |
| (8) dispatch_outcome, wasm execution only | wasm abi.dispatch_outcome / abi.exec | 61.0 [60.9–61.4] · p95 68.4 | 60.7 [58.6–63.1] · p95 70.0 | 61.3 [58.3–61.6] · p95 69.6 | - |
| (8) dispatch_outcome + JSON.parse | wasm raw.dispatch_outcome / raw.dispatch_outcome+parse | 139.1 [137.8–152.8] · p95 157.4 | 138.8 [138.2–139.0] · p95 153.9 | 139.9 [136.8–142.5] · p95 163.3 | - |
| (8) kit dispatch + view | wasm kit / kit.dispatch+view | 166.2 [163.9–175.6] · p95 193.9 | 170.4 [164.0–171.7] · p95 193.3 | 163.5 [162.7–171.2] · p95 191.4 | - |
| (c) view(), wasm execution only | wasm abi.read / abi.view.exec | 8.50 [8.20–8.70] · p95 9.40 | 8.70 [8.20–8.80] · p95 9.50 | 8.30 [8.20–8.60] · p95 9.30 | 7.50 [7.40–7.90] · p95 8.60 |
| (d) snapshot() (pretty), wasm execution only | wasm abi.read / abi.snapshot.exec | 69.3 [68.8–73.1] · p95 81.1 | 71.8 [68.4–74.0] · p95 82.7 | 70.2 [69.0–73.3] · p95 82.3 | 72.0 [71.1–75.4] · p95 84.5 |
| (f) save, wasm execution only | wasm abi.read / abi.save.exec | 15.1 [15.0–15.9] · p95 18.7 | 15.7 [14.8–16.1] · p95 19.1 | 14.9 [14.8–15.8] · p95 18.1 | 15.1 [15.0–15.8] · p95 18.9 |

### trail-rescue-scenarios

| Question | Engine, mode / operation | main-local (i1 for the bench row) | rc4-local | rc4-published | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- |
| (a) transaction only, numeric parameters | native apply / apply | 46.7 [46.1–47.3] · p95 121.8 | 47.4 [46.6–50.6] · p95 121.6 | - | - |
| (a/b) payload + transaction, no reporting (i1) | native bench.dispatch_only / bench.dispatch_only | 52.6 [52.4–53.7] · p95 133.8 | - | - | - |
| (b) + view built, not serialized | native dispatch_view_json / dispatch_view_json | 52.9 [49.7–53.5] · p95 130.8 | 51.4 [49.8–51.5] · p95 127.5 | - | - |
| (b) + full snapshot built (legacy dispatch_json) | native dispatch_json / dispatch_json | 82.0 [79.6–82.1] · p95 158.9 | 81.9 [80.9–82.5] · p95 157.7 | - | - |
| (b) + outcome with snapshot built (dispatch_outcome_json) | native dispatch_outcome_json / dispatch_outcome_json | 83.3 [81.4–85.6] · p95 159.8 | 87.1 [80.9–90.1] · p95 168.8 | - | - |
| (c) view() alone | native read / view | 1.30 [1.30–1.30] · p95 2.20 | 1.40 [1.30–1.40] · p95 2.40 | - | - |
| (c) view JSON | native read / view.serialize | 6.60 [6.60–6.60] · p95 8.40 | 7.00 [6.60–7.10] · p95 9.20 | - | - |
| (d) snapshot() alone | native read / snapshot | 29.9 [29.5–29.9] · p95 33.9 | 31.8 [29.8–32.0] · p95 36.7 | - | - |
| (d) snapshot JSON | native read / snapshot.serialize | 18.8 [18.7–18.9] · p95 21.4 | 19.9 [18.6–20.1] · p95 23.2 | - | - |
| session clone (upper bound on the transaction copy) | native read / clone | 10.2 [10.2–10.3] · p95 12.0 | 11.0 [10.3–11.0] · p95 12.9 | - | - |
| (e) exported dispatch_view | native web.dispatch_view / web.dispatch_view | 58.4 [57.7–59.4] · p95 132.1 | 58.0 [57.6–60.5] · p95 132.6 | - | - |
| (8) exported dispatch_outcome | native web.dispatch_outcome / web.dispatch_outcome | 117.2 [116.2–125.8] · p95 194.5 | 117.5 [115.6–118.2] · p95 193.9 | - | - |
| (f) save | native web.read / web.save | 10.8 [10.7–10.8] · p95 18.1 | 11.0 [10.9–11.0] · p95 18.3 | - | - |
| (e) dispatch_view + JSON.parse | wasm raw.dispatch_view / raw.dispatch_view+parse | 120.1 [118.9–123.7] · p95 178.2 | 120.7 [119.2–121.5] · p95 181.4 | 120.4 [119.4–121.1] · p95 177.9 | - |
| (e) dispatch_view, wasm execution only | wasm abi.dispatch_view / abi.exec | 70.6 [70.0–72.1] · p95 147.5 | 71.1 [70.5–71.2] · p95 147.1 | 70.6 [68.8–71.0] · p95 146.1 | - |
| (8) dispatch_outcome, wasm execution only | wasm abi.dispatch_outcome / abi.exec | 132.2 [129.1–135.1] · p95 214.3 | 131.9 [129.8–134.6] · p95 213.1 | 133.2 [131.8–135.0] · p95 217.5 | - |
| (8) dispatch_outcome + JSON.parse | wasm raw.dispatch_outcome / raw.dispatch_outcome+parse | 220.5 [213.7–221.5] · p95 314.4 | 223.8 [220.2–224.2] · p95 313.8 | 222.4 [215.5–224.2] · p95 319.5 | - |
| (8) kit dispatch + view | wasm kit / kit.dispatch+view | 260.3 [258.8–261.1] · p95 359.5 | 259.1 [257.2–264.4] · p95 353.3 | 261.2 [260.9–263.0] · p95 361.2 | - |
| (c) view(), wasm execution only | wasm abi.read / abi.view.exec | 9.20 [9.10–9.40] · p95 12.5 | 9.10 [9.00–9.40] · p95 12.5 | 9.30 [9.10–9.60] · p95 12.6 | - |
| (d) snapshot() (pretty), wasm execution only | wasm abi.read / abi.snapshot.exec | 99.2 [99.0–101.4] · p95 119.2 | 98.9 [97.7–99.3] · p95 120.7 | 100.7 [97.8–102.5] · p95 122.2 | - |
| (f) save, wasm execution only | wasm abi.read / abi.save.exec | 13.4 [13.4–13.7] · p95 23.2 | 13.4 [13.1–13.5] · p95 23.3 | 13.8 [13.3–13.9] · p95 23.7 | - |

### ledger-session

| Question | Engine, mode / operation | main-local (i1 for the bench row) | rc4-local | rc4-published | hist-e6ace96 |
| --- | --- | --- | --- | --- | --- |
| (a) transaction only, numeric parameters | native apply / apply | 16.2 [14.9–16.7] · p95 23.5 | 14.7 [14.6–14.9] · p95 21.5 | - | - |
| (a/b) payload + transaction, no reporting (i1) | native bench.dispatch_only / bench.dispatch_only | 18.2 [18.2–19.6] · p95 26.0 | - | - | - |
| (b) + view built, not serialized | native dispatch_view_json / dispatch_view_json | 20.4 [20.0–22.0] · p95 28.2 | 20.2 [20.1–22.0] · p95 28.0 | - | - |
| (b) + full snapshot built (legacy dispatch_json) | native dispatch_json / dispatch_json | 35.4 [35.2–35.9] · p95 43.8 | 35.4 [35.2–38.5] · p95 44.1 | - | - |
| (b) + outcome with snapshot built (dispatch_outcome_json) | native dispatch_outcome_json / dispatch_outcome_json | 35.3 [35.2–35.6] · p95 44.1 | 35.4 [35.2–35.5] · p95 44.4 | - | - |
| (c) view() alone | native read / view | 3.10 [3.10–3.10] · p95 4.40 | 3.10 [3.10–3.40] · p95 4.50 | - | - |
| (c) view JSON | native read / view.serialize | 2.30 [2.30–2.30] · p95 2.90 | 2.30 [2.20–2.40] · p95 2.90 | - | - |
| (d) snapshot() alone | native read / snapshot | 17.3 [17.1–17.6] · p95 19.7 | 17.3 [17.1–18.7] · p95 19.9 | - | - |
| (d) snapshot JSON | native read / snapshot.serialize | 13.3 [13.2–13.4] · p95 15.5 | 13.4 [13.2–14.3] · p95 16.0 | - | - |
| session clone (upper bound on the transaction copy) | native read / clone | 1.70 [1.70–1.70] · p95 1.90 | 1.70 [1.70–1.80] · p95 2.00 | - | - |
| (e) exported dispatch_view | native web.dispatch_view / web.dispatch_view | 24.2 [23.1–24.9] · p95 33.6 | 25.0 [22.8–25.8] · p95 34.0 | - | - |
| (8) exported dispatch_outcome | native web.dispatch_outcome / web.dispatch_outcome | 56.6 [56.1–63.7] · p95 71.9 | 56.7 [56.4–60.0] · p95 74.4 | - | - |
| (f) save | native web.read / web.save | 12.4 [12.4–13.6] · p95 16.9 | 12.3 [12.1–12.4] · p95 16.5 | - | - |
| (e) dispatch_view + JSON.parse | wasm raw.dispatch_view / raw.dispatch_view+parse | 38.1 [37.6–38.7] · p95 48.7 | 37.9 [37.8–38.3] · p95 48.6 | 38.1 [37.7–40.0] · p95 48.8 | 37.4 [36.7–38.4] · p95 47.2 |
| (e) dispatch_view, wasm execution only | wasm abi.dispatch_view / abi.exec | 27.6 [27.6–28.3] · p95 37.7 | 27.1 [26.9–27.1] · p95 36.8 | 27.0 [27.0–27.9] · p95 36.5 | 26.0 [25.7–26.0] · p95 34.8 |
| (8) dispatch_outcome, wasm execution only | wasm abi.dispatch_outcome / abi.exec | 57.9 [57.5–58.6] · p95 73.8 | 59.0 [58.3–60.8] · p95 76.2 | 58.7 [57.0–60.5] · p95 75.1 | - |
| (8) dispatch_outcome + JSON.parse | wasm raw.dispatch_outcome / raw.dispatch_outcome+parse | 107.8 [107.1–108.4] · p95 135.5 | 109.3 [107.0–109.5] · p95 137.7 | 114.2 [105.7–114.9] · p95 144.2 | - |
| (8) kit dispatch + view | wasm kit / kit.dispatch+view | 124.5 [124.0–125.8] · p95 158.4 | 125.9 [124.1–126.1] · p95 160.2 | 125.8 [125.4–126.2] · p95 162.0 | - |
| (c) view(), wasm execution only | wasm abi.read / abi.view.exec | 6.70 [6.60–6.90] · p95 9.20 | 6.80 [6.60–7.00] · p95 9.10 | 6.70 [6.60–6.70] · p95 9.10 | 6.30 [6.10–6.40] · p95 8.60 |
| (d) snapshot() (pretty), wasm execution only | wasm abi.read / abi.snapshot.exec | 61.1 [60.7–63.3] · p95 73.1 | 61.3 [61.1–65.2] · p95 72.8 | 60.9 [60.5–61.3] · p95 73.3 | 61.5 [59.4–61.9] · p95 73.1 |
| (f) save, wasm execution only | wasm abi.read / abi.save.exec | 14.8 [14.8–15.2] · p95 20.2 | 15.0 [14.9–15.5] · p95 20.1 | 15.0 [14.8–15.1] · p95 20.1 | 14.1 [13.7–14.3] · p95 19.8 |

## Reproducing the published 51.6 µs

| Method | Runtime | Reactive wasm | caveat5 median µs [runs] | p95 µs [runs] | per-run medians | ts per-run medians |
| --- | --- | --- | --- | --- | --- | --- |
| `harness.mjs --bench` verbatim, unpinned (as published), session 03:22 UTC | e6ace96 | `d897787f` | 49.9 [35.5–52.6] | 58.7 [58.1–60.0] | 35.5, 49.9, 52.6 | 2.50, 2.30, 2.30 |
| `harness.mjs --bench` verbatim, unpinned (as published), session 03:22 UTC | rc4 | `e3594450` | 53.2 [51.5–53.4] | 61.0 [59.5–61.7] | 53.4, 51.5, 53.2 | 2.40, 2.30, 2.30 |
| `harness.mjs --bench` verbatim, unpinned (as published), session 03:22 UTC | main | `e3594450` | 52.2 [50.9–52.3] | 59.0 [58.6–59.7] | 52.3, 52.2, 50.9 | 2.40, 2.30, 2.00 |
| `harness.mjs --bench` verbatim, pinned 0x3C00 high, session 03:25 UTC | e6ace96 | `d897787f` | 48.9 [48.8–49.1] | 55.3 [55.0–55.9] | 48.8, 49.1, 48.9 | 2.40, 2.40, 2.20 |
| `harness.mjs --bench` verbatim, pinned 0x3C00 high, session 03:25 UTC | rc4 | `e3594450` | 50.8 [50.0–51.1] | 56.8 [56.4–57.4] | 50.8, 50.0, 51.1 | 2.30, 2.40, 2.30 |
| `harness.mjs --bench` verbatim, pinned 0x3C00 high, session 03:25 UTC | main | `e3594450` | 50.4 [50.4–50.6] | 56.4 [56.3–56.4] | 50.4, 50.4, 50.6 | 2.30, 2.30, 2.40 |
| `harness.mjs --bench` verbatim, unpinned (as published), session 06:26 UTC | e6ace96 | `d897787f` | 48.1 [45.8–48.2] | 55.1 [54.4–56.9] | 48.2, 45.8, 48.1 | 2.00, 1.90, 2.10 |
| `harness.mjs --bench` verbatim, unpinned (as published), session 06:26 UTC | rc4 | `e3594450` | 47.7 [45.2–49.9] | 56.4 [55.4–57.6] | 49.9, 47.7, 45.2 | 2.00, 2.10, 2.20 |
| `harness.mjs --bench` verbatim, unpinned (as published), session 06:26 UTC | main | `e3594450` | 47.5 [44.9–49.6] | 55.7 [53.8–58.2] | 44.9, 49.6, 47.5 | 2.20, 2.10, 1.90 |
| `harness.mjs --bench` verbatim, pinned 0x3C00 high, session 06:23 UTC | e6ace96 | `d897787f` | 44.3 [43.9–44.6] | 49.5 [48.7–50.6] | 43.9, 44.3, 44.6 | 2.00, 2.00, 2.00 |
| `harness.mjs --bench` verbatim, pinned 0x3C00 high, session 06:23 UTC | rc4 | `e3594450` | 45.4 [45.0–45.5] | 50.4 [49.9–51.1] | 45.0, 45.4, 45.5 | 2.00, 2.00, 2.10 |
| `harness.mjs --bench` verbatim, pinned 0x3C00 high, session 06:23 UTC | main | `e3594450` | 45.7 [45.3–45.9] | 50.9 [50.2–52.3] | 45.9, 45.3, 45.7 | 2.40, 1.90, 1.90 |
| `published-method` (caveat5 alone, fresh process, pinned 0x3C00 high) | hist-e6ace96 | `d897787f` | 44.8 [41.3–45.1] | 53.2 [53.0–55.3] |  |  |
| `published-method` (caveat5 alone, fresh process, pinned 0x3C00 high) | rc4-published | `698a0d0f` | 46.8 [46.2–47.0] | 57.2 [56.8–58.5] |  |  |
| `published-method` (caveat5 alone, fresh process, pinned 0x3C00 high) | rc4-local | `e3594450` | 46.6 [46.0–46.7] | 56.1 [55.5–59.4] |  |  |
| `published-method` (caveat5 alone, fresh process, pinned 0x3C00 high) | main-local | `e3594450` | 46.7 [43.8–47.2] | 56.6 [56.6–57.8] |  |  |

The published method's own samples (3 rounds, no warm-up), split by event kind:

| Runtime | Event kind | median µs [runs] · p95 · samples per run |
| --- | --- | --- |
| hist-e6ace96 | idle tick | 44.7 [41.1–45.0] · p95 50.9 · n 28182/28182/28182 |
| hist-e6ace96 | state-changing tick | 49.4 [47.7–50.9] · p95 98.6 · n 1809/1809/1809 |
| hist-e6ace96 | observation (commit) | 286.2 [275.6–907.6] · p95 286.2 · n 3/3/3 |
| hist-e6ace96 | observation (reopen) | 253.0 [149.7–486.7] · p95 253.0 · n 3/3/3 |
| hist-e6ace96 | observation (evidence) | 135.6 [132.4–196.3] · p95 135.6 · n 3/3/3 |
| hist-e6ace96 | all | 44.8 [41.3–45.1] · p95 53.2 · n 30000/30000/30000 |
| rc4-published | idle tick | 46.7 [46.0–46.8] · p95 53.9 · n 28182/28182/28182 |
| rc4-published | state-changing tick | 52.5 [52.4–52.9] · p95 97.5 · n 1809/1809/1809 |
| rc4-published | observation (commit) | 305.9 [290.6–317.6] · p95 305.9 · n 3/3/3 |
| rc4-published | observation (reopen) | 177.9 [147.2–218.5] · p95 177.9 · n 3/3/3 |
| rc4-published | observation (evidence) | 142.0 [130.7–178.9] · p95 142.0 · n 3/3/3 |
| rc4-published | all | 46.8 [46.2–47.0] · p95 57.2 · n 30000/30000/30000 |
| main-local | idle tick | 46.6 [43.6–47.1] · p95 54.1 · n 28182/28182/28182 |
| main-local | state-changing tick | 51.7 [51.1–54.3] · p95 105.3 · n 1809/1809/1809 |
| main-local | observation (commit) | 433.3 [308.2–461.2] · p95 433.3 · n 3/3/3 |
| main-local | observation (reopen) | 165.0 [157.3–167.5] · p95 165.0 · n 3/3/3 |
| main-local | observation (evidence) | 143.3 [140.8–171.0] · p95 143.3 · n 3/3/3 |
| main-local | all | 46.7 [43.8–47.2] · p95 56.6 · n 30000/30000/30000 |

## The published method in-process against the same loop alone

Paired, back to back in one quiet window, main-local, both pinned to 0x3C00 at High priority: the published command (caveat5 after five other implementations in one process) and `published-method` (the same loop alone in a fresh process). Pooled medians of each run's 30,000 samples.

| Pair | in-process (verbatim) µs | alone (published-method) µs | in-process − alone µs |
| --- | --- | --- | --- |
| 2 | 43.2 | 46.3 | -3.10 |
| 3 | 44.9 | 45.4 | -0.50 |
| 4 | 45.4 | 46.1 | -0.70 |

Median difference -0.70 µs [-3.10–-0.50] (INFERRED, paired).

## Glowcap replay by event kind, every path (main-local)

Categories (1) idle tick, (2) state-changing tick and (3) observation, on the published stream. Every cell is DIRECT. Each observation, renewal and qualification occurs once per pass (3 samples a run): those columns are unreliable and shown only for completeness.

| Path (µs: median [lowest–highest run] · p95) | idle tick (n 28182/run) | state-changing tick (n 1809/run) | renew (n 3/run) | qualify (n 3/run) | observation (commit) (n 3/run) | observation (reopen) (n 3/run) | observation (evidence) (n 3/run) | all (n 30000/run) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| native: apply (numeric parameters; the transaction only) | 13.0 [13.0–13.8] · p95 14.4 | 16.5 [16.2–17.1] · p95 17.8 | 95.8 [94.7–99.7] · p95 95.8 | 96.4 [92.2–102.6] · p95 96.4 | 82.5 [80.6–88.2] · p95 82.5 | 88.2 [79.9–88.2] · p95 88.2 | 77.9 [77.3–87.6] · p95 77.9 | 13.1 [13.0–13.8] · p95 16.1 |
| native: WebReactiveSession::dispatch_view (the exported function, JSON in and out) | 20.0 [19.9–21.3] · p95 21.7 | 22.3 [22.1–23.7] · p95 23.8 | 113.2 [109.7–125.0] · p95 113.2 | 107.8 [102.2–107.9] · p95 107.8 | 121.2 [104.3–128.3] · p95 121.2 | 110.4 [107.2–110.5] · p95 110.4 | 113.2 [108.7–115.6] · p95 113.2 | 20.0 [20.0–21.3] · p95 22.5 |
| native: WebReactiveSession::dispatch_outcome (the kit's call) | 64.8 [64.5–70.3] · p95 69.2 | 63.0 [62.5–68.1] · p95 67.8 | 161.2 [160.4–171.6] · p95 161.2 | 165.2 [165.0–174.4] · p95 165.2 | 194.4 [193.8–198.6] · p95 194.4 | 160.6 [159.1–175.2] · p95 160.6 | 137.4 [132.9–146.2] · p95 137.4 | 64.7 [64.4–70.2] · p95 69.2 |
| wasm: dispatch_view, WebAssembly execution only | 22.5 [21.2–22.5] · p95 25.3 | 25.2 [23.9–25.6] · p95 32.3 | 153.8 [150.2–155.2] · p95 153.8 | 145.7 [142.2–154.0] · p95 145.7 | 146.2 [135.7–148.5] · p95 146.2 | 110.1 [107.3–114.2] · p95 110.1 | 105.7 [104.1–109.3] · p95 105.7 | 22.6 [21.2–22.6] · p95 26.2 |
| wasm: dispatch_view through the glue + JSON.parse (the web pages' path) | 43.8 [40.9–45.2] · p95 48.2 | 46.6 [43.0–46.7] · p95 62.3 | 186.6 [169.6–203.3] · p95 186.6 | 178.5 [169.7–184.1] · p95 178.5 | 238.0 [219.5–250.7] · p95 238.0 | 157.6 [146.0–175.0] · p95 157.6 | 129.4 [122.8–132.7] · p95 129.4 | 43.9 [41.0–45.3] · p95 49.0 |
| wasm: the published method verbatim (the adapter, no warm-up) | 46.6 [43.6–47.1] · p95 54.1 | 51.7 [51.1–54.3] · p95 105.3 | 222.4 [171.4–269.6] · p95 222.4 | 199.0 [164.5–213.2] · p95 199.0 | 433.3 [308.2–461.2] · p95 433.3 | 165.0 [157.3–167.5] · p95 165.0 | 143.3 [140.8–171.0] · p95 143.3 | 46.7 [43.8–47.2] · p95 56.6 |
| wasm: the Glowcap adapter, dispatch + view (the published path, warmed) | 45.9 [43.1–46.0] · p95 50.8 | 49.6 [48.9–50.2] · p95 74.9 | 200.5 [184.6–202.0] · p95 200.5 | 194.4 [184.0–205.2] · p95 194.4 | 242.3 [214.9–256.0] · p95 242.3 | 183.7 [169.5–194.9] · p95 183.7 | 202.1 [200.9–205.5] · p95 202.1 | 46.0 [43.1–46.1] · p95 52.1 |
| wasm: dispatch_outcome, WebAssembly execution only | 61.1 [60.9–61.5] · p95 68.0 | 59.9 [57.2–61.1] · p95 78.9 | 218.7 [217.8–240.6] · p95 218.7 | 210.2 [196.7–211.3] · p95 210.2 | 206.5 [187.6–211.5] · p95 206.5 | 159.4 [156.9–161.5] · p95 159.4 | 165.2 [145.1–217.0] · p95 165.2 | 61.0 [60.9–61.4] · p95 68.4 |
| wasm: dispatch_outcome through the glue + JSON.parse | 139.3 [138.0–153.5] · p95 156.8 | 133.1 [132.7–143.1] · p95 172.4 | 343.1 [338.2–354.6] · p95 343.1 | 311.1 [302.2–416.7] · p95 311.1 | 398.9 [373.8–411.8] · p95 398.9 | 269.2 [262.0–295.7] · p95 269.2 | 242.2 [228.9–253.4] · p95 242.2 | 139.1 [137.8–152.8] · p95 157.4 |
| wasm: kit session.dispatch() alone | 137.6 [135.6–145.4] · p95 160.1 | 132.2 [131.3–138.8] · p95 172.3 | 319.0 [304.8–330.0] · p95 319.0 | 320.1 [290.8–331.2] · p95 320.1 | 398.1 [372.2–404.8] · p95 398.1 | 291.5 [289.0–297.5] · p95 291.5 | 259.0 [252.9–260.5] · p95 259.0 | 137.4 [135.4–145.1] · p95 161.1 |
| wasm: kit session.dispatch() + session.view() | 166.4 [164.1–175.9] · p95 192.8 | 159.5 [158.7–167.4] · p95 212.4 | 355.1 [340.9–369.3] · p95 355.1 | 353.3 [324.1–368.3] · p95 353.3 | 467.6 [443.0–542.4] · p95 467.6 | 354.8 [346.0–389.0] · p95 354.8 | 293.6 [283.1–297.5] · p95 293.6 | 166.2 [163.9–175.6] · p95 193.9 |

## Trail Rescue by event class, every path (main-local)

Category (3) on a decision workload: 24 scenarios, 140 runtime events, played 20 times per pass. Every cell is DIRECT.

| Path (µs: median [lowest–highest run] · p95) | idle (n 720/run) | state-changing (n 1320/run) | qualify (n 360/run) | evidence (n 2640/run) | commit (n 1440/run) | reopen (n 600/run) | refused (n 1320/run) | all (n 8400/run) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| native: apply (numeric parameters; the transaction only) | 12.6 [12.6–12.7] · p95 33.5 | 18.1 [17.8–18.1] · p95 49.1 | 88.0 [87.7–88.4] · p95 125.3 | 89.3 [88.8–89.6] · p95 132.8 | 41.9 [41.8–42.2] · p95 60.1 | 108.8 [108.7–109.4] · p95 133.4 | 3.30 [3.30–3.30] · p95 6.60 | 46.7 [46.1–47.3] · p95 121.8 |
| native: WebReactiveSession::dispatch_view (the exported function, JSON in and out) | 18.5 [18.0–18.8] · p95 40.6 | 26.9 [26.9–27.2] · p95 59.0 | 98.3 [97.8–98.5] · p95 136.2 | 103.4 [102.6–104.9] · p95 142.0 | 54.2 [53.8–55.1] · p95 73.6 | 120.3 [120.2–121.5] · p95 146.1 | 4.30 [4.30–4.40] · p95 8.60 | 58.4 [57.7–59.4] · p95 132.1 |
| native: WebReactiveSession::dispatch_outcome (the kit's call) | 64.3 [63.2–69.2] · p95 94.0 | 83.9 [83.6–90.6] · p95 119.3 | 157.5 [156.9–170.2] · p95 200.2 | 161.9 [160.6–173.8] · p95 205.0 | 112.5 [112.1–120.7] · p95 137.1 | 181.8 [180.0–190.8] · p95 209.8 | 4.60 [4.60–5.00] · p95 9.50 | 117.2 [116.2–125.8] · p95 194.5 |
| wasm: dispatch_view, WebAssembly execution only | 23.3 [23.1–24.9] · p95 46.7 | 32.1 [31.8–32.4] · p95 73.5 | 110.2 [109.4–112.2] · p95 155.1 | 113.3 [113.1–117.4] · p95 157.7 | 63.9 [63.5–65.5] · p95 89.9 | 132.7 [132.3–136.0] · p95 166.4 | 5.60 [5.50–5.70] · p95 10.3 | 70.6 [70.0–72.1] · p95 147.5 |
| wasm: dispatch_view through the glue + JSON.parse (the web pages' path) | 45.1 [44.4–46.1] · p95 72.9 | 58.9 [57.2–60.8] · p95 114.6 | 136.8 [134.0–141.7] · p95 184.9 | 141.7 [140.3–145.2] · p95 183.8 | 92.8 [91.0–95.7] · p95 120.1 | 162.6 [158.9–165.7] · p95 192.6 | - | 120.1 [118.9–123.7] · p95 178.2 |
| wasm: dispatch_outcome, WebAssembly execution only | 70.7 [69.7–72.5] · p95 109.9 | 93.2 [89.9–94.5] · p95 140.8 | 175.9 [172.0–179.0] · p95 228.5 | 176.0 [174.1–182.5] · p95 229.1 | 125.4 [122.9–129.5] · p95 161.9 | 193.0 [189.5–197.3] · p95 234.8 | 5.00 [4.80–5.20] · p95 10.6 | 132.2 [129.1–135.1] · p95 214.3 |
| wasm: dispatch_outcome through the glue + JSON.parse | 148.9 [141.5–150.5] · p95 196.3 | 178.4 [172.2–179.0] · p95 246.8 | 264.5 [255.6–265.1] · p95 330.9 | 263.6 [257.1–265.9] · p95 331.8 | 212.1 [206.8–212.8] · p95 264.4 | 282.5 [275.6–282.9] · p95 343.4 | 8.30 [8.10–8.40] · p95 12.7 | 220.5 [213.7–221.5] · p95 314.4 |
| wasm: kit session.dispatch() alone | 151.2 [147.2–152.4] · p95 204.5 | 178.4 [177.1–179.1] · p95 249.6 | 263.9 [262.3–266.4] · p95 335.3 | 267.2 [266.9–271.3] · p95 334.1 | 213.5 [212.9–215.3] · p95 266.5 | 284.7 [283.5–285.4] · p95 348.1 | 11.0 [10.9–11.2] · p95 16.8 | 223.0 [221.5–223.5] · p95 316.8 |
| wasm: kit session.dispatch() + session.view() | 180.3 [174.6–182.8] · p95 240.5 | 214.2 [213.3–215.8] · p95 291.9 | 302.0 [299.5–305.5] · p95 378.8 | 300.8 [300.5–305.9] · p95 377.7 | 251.6 [250.7–253.5] · p95 313.4 | 325.2 [323.8–325.3] · p95 396.6 | 40.9 [40.6–41.0] · p95 55.8 | 260.3 [258.8–261.1] · p95 359.5 |

## Agent ledger by event class, every path (main-local)

The ledger log (20 events, 5 refused by policy), 100 times per pass. Every cell is DIRECT.

| Path (µs: median [lowest–highest run] · p95) | evidence (n 4500/run) | pushed evidence [renew,reveal] (n 1200/run) | refused (n 1500/run) | all (n 6000/run) |
| --- | --- | --- | --- | --- |
| native: apply (numeric parameters; the transaction only) | 19.4 [18.3–20.5] · p95 23.8 | 20.4 [18.9–21.2] · p95 22.9 | 4.10 [3.80–4.30] · p95 5.80 | 16.2 [14.9–16.7] · p95 23.5 |
| native: WebReactiveSession::dispatch_view (the exported function, JSON in and out) | 27.2 [25.6–27.6] · p95 34.1 | 27.7 [25.9–28.0] · p95 30.6 | 4.80 [4.60–4.90] · p95 6.70 | 24.2 [23.1–24.9] · p95 33.6 |
| native: WebReactiveSession::dispatch_outcome (the kit's call) | 58.7 [58.2–66.0] · p95 74.3 | 58.1 [57.8–65.6] · p95 73.1 | 5.70 [5.50–6.40] · p95 7.20 | 56.6 [56.1–63.7] · p95 71.9 |
| wasm: dispatch_view, WebAssembly execution only | 30.5 [30.4–31.0] · p95 38.4 | 31.8 [31.5–32.2] · p95 37.4 | 5.90 [5.90–6.00] · p95 7.90 | 27.6 [27.6–28.3] · p95 37.7 |
| wasm: dispatch_view through the glue + JSON.parse (the web pages' path) | 38.1 [37.6–38.7] · p95 48.7 | 38.2 [37.6–39.1] · p95 47.9 | - | 38.1 [37.6–38.7] · p95 48.7 |
| wasm: dispatch_outcome, WebAssembly execution only | 60.5 [60.3–61.4] · p95 75.9 | 60.6 [60.5–61.6] · p95 74.7 | 5.80 [5.70–5.90] · p95 7.80 | 57.9 [57.5–58.6] · p95 73.8 |
| wasm: dispatch_outcome through the glue + JSON.parse | 112.1 [111.2–113.2] · p95 138.6 | 110.3 [110.1–111.3] · p95 132.3 | 7.60 [7.50–7.70] · p95 10.1 | 107.8 [107.1–108.4] · p95 135.5 |
| wasm: kit session.dispatch() alone | 114.6 [114.0–115.6] · p95 146.0 | 112.6 [112.4–113.5] · p95 136.9 | 9.10 [9.00–9.10] · p95 12.1 | 110.6 [110.1–111.7] · p95 140.4 |
| wasm: kit session.dispatch() + session.view() | 129.0 [128.3–130.1] · p95 164.9 | 125.8 [125.4–127.0] · p95 152.7 | 23.6 [23.1–23.7] · p95 30.0 | 124.5 [124.0–125.8] · p95 158.4 |

## Glowcap replay by event kind (instrumented session)

The same main-local build measured in the instrumented session (2026-09-29T03-28-17-772-instrumented; total CPU mean 11.3%), the session the attribution in sections 9–10 comes from; it ran these paths but not the adapter or the kit. Every cell is DIRECT.

| Path (µs: median [lowest–highest run] · p95) | idle tick (n 28182/run) | state-changing tick (n 1809/run) | renew (n 3/run) | qualify (n 3/run) | observation (commit) (n 3/run) | observation (reopen) (n 3/run) | observation (evidence) (n 3/run) | all (n 30000/run) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| native: apply (numeric parameters; the transaction only) | 15.0 [14.2–15.4] · p95 16.5 | 19.0 [17.5–19.1] · p95 20.9 | 116.6 [104.3–119.0] · p95 116.6 | 107.3 [101.6–117.8] · p95 107.3 | 91.3 [88.4–98.2] · p95 91.3 | 97.2 [91.7–99.9] · p95 97.2 | 82.8 [78.2–83.7] · p95 82.8 | 15.0 [14.3–15.4] · p95 18.6 |
| native: WebReactiveSession::dispatch_view (the exported function, JSON in and out) | 22.7 [21.8–23.5] · p95 24.9 | 25.2 [24.6–26.2] · p95 28.6 | 149.8 [132.7–152.9] · p95 149.8 | 125.4 [124.0–128.8] · p95 125.4 | 132.5 [110.5–134.1] · p95 132.5 | 119.7 [114.4–122.1] · p95 119.7 | 119.5 [114.1–125.1] · p95 119.5 | 22.7 [21.9–23.5] · p95 26.1 |
| native: WebReactiveSession::dispatch_outcome (the kit's call) | 75.2 [69.9–77.8] · p95 84.0 | 73.1 [66.9–73.8] · p95 78.7 | 204.3 [199.7–205.0] · p95 204.3 | 190.1 [187.5–193.8] · p95 190.1 | 199.6 [181.0–215.5] · p95 199.6 | 166.7 [163.4–173.5] · p95 166.7 | 150.6 [139.2–156.1] · p95 150.6 | 75.1 [69.6–77.6] · p95 83.8 |
| wasm: dispatch_view, WebAssembly execution only | 24.8 [24.6–25.6] · p95 31.4 | 28.2 [27.1–28.7] · p95 41.5 | 170.7 [164.6–179.3] · p95 170.7 | 162.8 [153.0–164.5] · p95 162.8 | 156.0 [155.7–179.1] · p95 156.0 | 126.9 [125.1–129.2] · p95 126.9 | 123.7 [111.7–128.3] · p95 123.7 | 24.9 [24.7–25.8] · p95 32.1 |
| wasm: dispatch_view through the glue + JSON.parse (the web pages' path) | 46.7 [46.3–48.3] · p95 58.0 | 52.0 [51.2–53.4] · p95 74.3 | 200.9 [154.4–204.6] · p95 200.9 | 199.5 [198.8–200.1] · p95 199.5 | 325.1 [303.3–333.9] · p95 325.1 | 172.3 [170.1–197.8] · p95 172.3 | 145.0 [143.3–151.4] · p95 145.0 | 46.8 [46.5–48.4] · p95 59.6 |
| wasm: dispatch_outcome, WebAssembly execution only | 65.8 [65.1–67.1] · p95 77.9 | 64.7 [64.6–69.0] · p95 90.6 | 242.2 [241.1–248.9] · p95 242.2 | 240.8 [229.4–255.8] · p95 240.8 | 236.4 [232.8–251.3] · p95 236.4 | 176.3 [170.9–192.5] · p95 176.3 | 162.3 [160.2–170.6] · p95 162.3 | 65.7 [65.1–67.3] · p95 78.2 |
| wasm: dispatch_outcome through the glue + JSON.parse | 152.3 [148.9–160.1] · p95 176.4 | 141.5 [136.3–147.3] · p95 186.0 | 337.0 [328.6–351.8] · p95 337.0 | 340.2 [333.1–344.8] · p95 340.2 | 467.9 [452.4–471.3] · p95 467.9 | 293.6 [284.1–318.1] · p95 293.6 | 254.5 [239.4–261.2] · p95 254.5 | 151.7 [148.6–159.3] · p95 176.6 |

## Trail Rescue by event class (instrumented session)

The same main-local build measured in the instrumented session (2026-09-29T03-28-17-772-instrumented; total CPU mean 11.3%), the session the attribution in sections 9–10 comes from; it ran these paths but not the adapter or the kit. Every cell is DIRECT.

| Path (µs: median [lowest–highest run] · p95) | idle (n 720/run) | state-changing (n 1320/run) | qualify (n 360/run) | evidence (n 2640/run) | commit (n 1440/run) | reopen (n 600/run) | refused (n 1320/run) | all (n 8400/run) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| native: apply (numeric parameters; the transaction only) | 13.8 [13.1–15.0] · p95 37.5 | 20.2 [18.8–21.4] · p95 57.1 | 98.3 [91.2–106.8] · p95 139.4 | 100.4 [92.5–106.8] · p95 148.3 | 47.0 [44.4–50.3] · p95 68.7 | 121.3 [113.6–122.7] · p95 148.4 | 3.60 [3.40–3.70] · p95 7.40 | 52.7 [48.0–54.7] · p95 135.9 |
| native: WebReactiveSession::dispatch_view (the exported function, JSON in and out) | 20.5 [19.5–21.3] · p95 45.2 | 29.9 [27.9–31.7] · p95 66.0 | 108.6 [102.2–117.4] · p95 153.4 | 114.7 [107.7–121.6] · p95 156.7 | 59.5 [56.5–64.3] · p95 82.3 | 132.8 [127.2–137.9] · p95 164.1 | 4.80 [4.60–5.10] · p95 9.50 | 65.5 [61.1–68.6] · p95 146.8 |
| native: WebReactiveSession::dispatch_outcome (the kit's call) | 68.5 [66.8–69.6] · p95 102.7 | 92.1 [87.5–92.1] · p95 133.2 | 174.6 [164.2–175.1] · p95 222.3 | 176.5 [169.0–176.6] · p95 226.3 | 121.9 [116.5–121.9] · p95 151.7 | 198.9 [191.3–199.4] · p95 232.7 | 5.10 [4.80–5.10] · p95 10.3 | 128.0 [123.2–128.6] · p95 215.0 |
| wasm: dispatch_view, WebAssembly execution only | 25.3 [23.9–28.2] · p95 49.8 | 34.9 [33.0–37.9] · p95 80.5 | 118.6 [112.3–132.9] · p95 165.0 | 121.9 [117.0–135.6] · p95 171.1 | 68.9 [66.4–76.8] · p95 96.1 | 140.9 [136.5–152.0] · p95 177.9 | 6.30 [5.90–6.70] · p95 11.7 | 75.8 [73.2–82.2] · p95 157.9 |
| wasm: dispatch_view through the glue + JSON.parse (the web pages' path) | 50.8 [46.9–51.0] · p95 85.3 | 64.8 [61.1–65.3] · p95 126.3 | 152.5 [142.8–159.1] · p95 214.3 | 157.4 [147.4–161.1] · p95 212.5 | 104.7 [97.0–106.3] · p95 141.8 | 179.9 [168.1–180.1] · p95 221.7 | - | 134.3 [125.7–134.9] · p95 203.5 |
| wasm: dispatch_outcome, WebAssembly execution only | 83.8 [79.1–85.1] · p95 130.9 | 106.5 [102.4–109.1] · p95 169.8 | 203.0 [194.3–207.3] · p95 276.5 | 204.9 [194.3–214.7] · p95 274.0 | 144.8 [137.9–150.1] · p95 197.2 | 222.1 [209.8–224.6] · p95 288.3 | 6.10 [5.80–6.20] · p95 12.4 | 154.5 [145.0–157.2] · p95 256.2 |
| wasm: dispatch_outcome through the glue + JSON.parse | 163.0 [155.3–167.1] · p95 219.2 | 193.6 [188.2–194.9] · p95 274.3 | 287.1 [281.9–289.6] · p95 365.4 | 286.9 [278.0–291.1] · p95 361.0 | 230.9 [222.9–234.9] · p95 293.1 | 306.0 [298.3–308.3] · p95 374.3 | 9.00 [8.80–9.20] · p95 14.2 | 241.4 [233.7–243.7] · p95 342.8 |

## Agent ledger by event class (instrumented session)

The same main-local build measured in the instrumented session (2026-09-29T03-28-17-772-instrumented; total CPU mean 11.3%), the session the attribution in sections 9–10 comes from; it ran these paths but not the adapter or the kit. Every cell is DIRECT.

| Path (µs: median [lowest–highest run] · p95) | evidence (n 4500/run) | pushed evidence [renew,reveal] (n 1200/run) | refused (n 1500/run) | all (n 6000/run) |
| --- | --- | --- | --- | --- |
| native: apply (numeric parameters; the transaction only) | 20.6 [20.1–24.2] · p95 24.9 | 21.4 [20.9–25.5] · p95 24.0 | 4.30 [4.20–5.10] · p95 6.10 | 16.9 [16.8–21.0] · p95 24.6 |
| native: WebReactiveSession::dispatch_view (the exported function, JSON in and out) | 28.4 [28.4–30.4] · p95 35.8 | 28.8 [28.7–31.0] · p95 32.5 | 5.10 [5.10–5.50] · p95 7.00 | 26.1 [25.6–27.7] · p95 35.2 |
| native: WebReactiveSession::dispatch_outcome (the kit's call) | 65.8 [64.2–67.4] · p95 83.5 | 65.2 [63.6–66.7] · p95 77.1 | 6.30 [6.20–6.50] · p95 8.20 | 63.2 [61.9–64.7] · p95 80.8 |
| wasm: dispatch_view, WebAssembly execution only | 33.5 [32.8–33.8] · p95 43.6 | 34.5 [33.5–36.4] · p95 43.7 | 6.50 [6.40–6.60] · p95 8.90 | 30.6 [30.0–30.7] · p95 42.6 |
| wasm: dispatch_view through the glue + JSON.parse (the web pages' path) | 42.8 [41.8–43.5] · p95 55.6 | 43.0 [42.0–44.0] · p95 54.3 | - | 42.8 [41.8–43.5] · p95 55.6 |
| wasm: dispatch_outcome, WebAssembly execution only | 69.4 [68.1–72.7] · p95 89.1 | 70.0 [68.5–73.1] · p95 87.8 | 6.60 [6.50–7.00] · p95 9.20 | 66.1 [64.7–68.7] · p95 86.3 |
| wasm: dispatch_outcome through the glue + JSON.parse | 130.8 [125.5–134.5] · p95 166.4 | 128.8 [124.4–132.4] · p95 156.9 | 9.00 [8.50–9.20] · p95 12.2 | 123.4 [121.6–127.9] · p95 162.8 |

## (4) Core runtime work: the event without view or snapshot

The instrumented session: the ordinary main-local build and the i1 build side by side, interleaved.

| Quantity (µs) | Label | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | ledger evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| transaction, numeric parameters (native apply) | DIRECT | 15.0 [14.2–15.4] · p95 16.5 | 19.0 [17.5–19.1] · p95 20.9 | 100.4 [92.5–106.8] · p95 148.3 | 47.0 [44.4–50.3] · p95 68.7 | 121.3 [113.6–122.7] · p95 148.4 | 20.6 [20.1–24.2] · p95 24.9 |
| payload parse + names + transaction (native, i1 bench_dispatch_only) | DIRECT | 15.2 [14.6–15.9] · p95 16.8 | 18.8 [17.7–20.1] · p95 20.3 | 100.4 [100.2–102.8] · p95 146.5 | 47.9 [47.0–48.9] · p95 68.1 | 118.4 [117.1–119.8] · p95 149.3 | 21.5 [21.2–22.3] · p95 26.5 |
| payload parse + names + transaction (wasm execution, i1 bench_dispatch_only) | DIRECT | 14.6 [14.5–14.7] · p95 18.4 | 18.8 [17.3–20.3] · p95 27.6 | 108.9 [108.7–110.6] · p95 160.3 | 54.0 [53.4–55.4] · p95 80.4 | 126.9 [126.7–130.8] · p95 164.9 | 23.7 [23.2–25.3] · p95 31.0 |
| payload parse + name resolution (native) | INFERRED [4] | 0.30 [0.30–0.30] ±1.30 | 0.60 [0.10–0.90] ±2.00 | 2.80 [1.50–7.00] ±5.25 | 2.30 [1.40–2.90] ±2.65 | 0.00 [-0.10–2.80] ±4.15 | 1.00 [0.90–1.50] ±0.80 |
| WebAssembly execution over native, the same work | INFERRED [5] | -0.70 [-1.30–0.10] ±0.75 | -1.30 [-1.50–2.60] ±2.70 | 8.50 [5.90–10.4] ±2.25 | 6.40 [5.10–7.50] ±1.95 | 9.60 [7.10–12.4] ±3.40 | 2.20 [0.90–4.10] ±1.60 |

- payload parse + names + transaction (native, i1 bench_dispatch_only): throwaway i1 build
- payload parse + names + transaction (wasm execution, i1 bench_dispatch_only): throwaway i1 build
- [4] i1 bench.dispatch_only − i1 apply
- [5] i1 abi.dispatch_only.exec − i1 bench.dispatch_only

## (5) Snapshot and reporting work

The instrumented session.

| Quantity (µs) | Label | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | ledger evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| snapshot() built (native, not serialized, not dropped) | DIRECT | 28.4 [27.7–29.3] · p95 32.8 | 24.4 [24.1–25.1] · p95 27.2 | 32.9 [31.6–33.6] · p95 37.9 | 33.3 [32.0–33.9] · p95 38.4 | 34.6 [33.0–34.9] · p95 40.0 | 19.3 [19.2–19.7] · p95 24.1 |
| snapshot compact JSON (native serde_json) | DIRECT | 17.1 [16.8–17.7] · p95 19.6 | 15.5 [15.1–15.8] · p95 17.1 | 21.1 [20.3–21.5] · p95 24.4 | 21.0 [20.2–21.3] · p95 24.1 | 21.8 [20.8–22.0] · p95 24.8 | 14.7 [14.6–15.1] · p95 18.0 |
| snapshot drop (native) | DIRECT | 12.2 [11.8–12.5] · p95 13.9 | 10.5 [10.4–10.8] · p95 11.6 | 12.9 [12.4–13.2] · p95 14.8 | 13.3 [12.8–13.7] · p95 15.9 | 14.3 [13.7–14.5] · p95 16.1 | 7.70 [7.60–7.80] · p95 8.80 |
| snapshot built + dropped (wasm, i1) | DIRECT | 32.5 [29.6–32.6] · p95 38.8 | 28.2 [27.1–29.0] · p95 38.0 | 48.6 [47.8–49.7] · p95 59.3 | 47.1 [46.2–47.2] · p95 56.9 | 49.9 [49.3–50.4] · p95 60.3 | 25.1 [24.4–27.4] · p95 31.4 |
| dispatch_outcome over dispatch-only (native): snapshot + outcome + JSON | INFERRED [5] | 57.1 [54.5–57.5] ±2.80 | 51.2 [49.2–52.2] ±3.40 | 76.8 [67.7–77.0] ±7.05 | 74.0 [69.7–76.1] ±5.10 | 81.3 [72.3–81.5] ±7.30 | 44.4 [43.7–46.5] ±2.50 |
| dispatch_outcome over dispatch-only (wasm execution) | INFERRED [6] | 50.2 [49.1–51.3] ±1.10 | 47.3 [47.0–50.4] ±4.70 | 89.9 [88.9–99.0] ±5.60 | 86.4 [85.5–93.3] ±4.15 | 85.7 [83.0–97.5] ±7.85 | 44.3 [43.5–46.1] ±1.70 |
| JS JSON.parse of the outcome | DIRECT | 70.9 [69.6–74.4] · p95 81.3 | 64.0 [62.2–66.3] · p95 77.4 | 86.0 [83.6–87.5] · p95 104.4 | 84.1 [81.6–84.7] · p95 99.6 | 86.3 [83.9–86.8] · p95 101.8 | 53.6 [51.6–55.6] · p95 67.6 |

- snapshot built + dropped (wasm, i1): throwaway i1 build
- [5] i1 web.dispatch_outcome − i1 bench.dispatch_only
- [6] i1 abi.dispatch_outcome exec − i1 abi.dispatch_only exec

## (6) View construction

The instrumented session.

| Quantity (µs) | Label | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | ledger evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| view() built (native, not serialized, not dropped) | DIRECT | 2.80 [2.70–2.90] · p95 3.20 | 2.10 [2.10–2.10] · p95 2.40 | 1.10 [1.10–1.20] · p95 1.90 | 1.70 [1.60–1.70] · p95 2.70 | 1.80 [1.80–1.90] · p95 2.60 | 3.30 [3.20–3.30] · p95 4.30 |
| view compact JSON (native serde_json) | DIRECT | 5.40 [5.30–5.50] · p95 6.20 | 4.90 [4.90–5.00] · p95 5.50 | 7.80 [7.50–7.90] · p95 12.0 | 7.70 [7.40–7.80] · p95 9.00 | 8.10 [7.80–8.20] · p95 9.40 | 2.60 [2.60–2.60] · p95 3.30 |
| view built + dropped (native, i1) | DIRECT | 4.30 [4.00–4.50] · p95 4.60 | 3.40 [3.20–3.60] · p95 3.70 | 2.30 [2.20–2.30] · p95 3.10 | 3.40 [3.30–3.40] · p95 4.80 | 3.10 [3.10–3.20] · p95 5.00 | 4.80 [4.80–5.00] · p95 6.30 |
| view built + dropped (wasm, i1) | DIRECT | 3.90 [3.60–4.00] · p95 4.60 | 3.00 [2.90–3.20] · p95 4.10 | 2.60 [2.50–2.70] · p95 3.60 | 3.60 [3.50–3.60] · p95 5.30 | 3.50 [3.50–3.60] · p95 5.10 | 5.00 [4.90–5.30] · p95 6.60 |
| view exported: built + JSON + dropped (wasm) | DIRECT | 10.2 [9.40–10.4] · p95 11.8 | 9.00 [8.50–9.40] · p95 11.9 | 14.4 [14.1–14.8] · p95 18.2 | 15.1 [14.9–15.2] · p95 19.3 | 15.9 [15.8–16.0] · p95 20.5 | 8.00 [7.80–8.60] · p95 10.6 |
| view JSON inside wasm | INFERRED [6] | 6.30 [5.80–6.40] ±0.70 | 6.00 [5.60–6.20] ±0.60 | 11.8 [11.6–12.1] ±0.45 | 11.5 [11.4–11.6] ±0.20 | 12.3 [12.3–12.5] ±0.15 | 3.00 [2.90–3.30] ±0.60 |
| JS JSON.parse of the view | DIRECT | 18.2 [18.2–18.4] · p95 22.9 | 18.4 [17.9–19.0] · p95 25.9 | 28.9 [27.5–29.4] · p95 37.3 | 29.0 [27.1–29.5] · p95 36.2 | 29.8 [28.3–31.0] · p95 37.4 | 7.80 [7.50–8.00] · p95 11.2 |

- view built + dropped (native, i1): throwaway i1 build
- view built + dropped (wasm, i1): throwaway i1 build
- [6] i1 abi.view.exec − i1 abi.view_build.exec (same step, same state)

## (7) WebAssembly, JSON and JS overhead on dispatch_view

The ordinary main-local build in the instrumented session, every part DIRECT or a paired difference.

| Quantity (µs) | Label | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | ledger evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| payload JSON.stringify (JS) | DIRECT | 0.30 [0.30–0.40] · p95 0.60 | 0.40 [0.30–0.40] · p95 0.70 | 1.00 [1.00–1.10] · p95 2.20 | 0.50 [0.50–0.50] · p95 0.90 | 0.60 [0.60–0.60] · p95 1.00 | 0.40 [0.40–0.40] · p95 1.20 |
| argument copy into wasm memory (passStringToWasm0 ×2) | DIRECT | 0.10 [0.10–0.10] · p95 0.30 | 0.30 [0.30–0.30] · p95 0.60 | 0.70 [0.60–0.70] · p95 1.40 | 0.40 [0.40–0.40] · p95 0.90 | 0.40 [0.30–0.40] · p95 0.70 | 0.30 [0.30–0.30] · p95 0.80 |
| wasm execution of dispatch_view | DIRECT | 24.8 [24.6–25.6] · p95 31.4 | 28.2 [27.1–28.7] · p95 41.5 | 121.9 [117.0–135.6] · p95 171.1 | 68.9 [66.4–76.8] · p95 96.1 | 140.9 [136.5–152.0] · p95 177.9 | 33.5 [32.8–33.8] · p95 43.6 |
| result decode (TextDecoder, fatal) | DIRECT | 2.90 [2.70–2.90] · p95 3.80 | 2.80 [2.80–2.90] · p95 4.30 | 2.10 [2.00–2.20] · p95 5.20 | 1.30 [1.30–1.50] · p95 3.80 | 1.40 [1.30–1.50] · p95 4.30 | 0.70 [0.60–0.70] · p95 2.10 |
| result free (__wbindgen_free) | DIRECT | 0.10 [0.10–0.10] · p95 0.10 | 0.10 [0.10–0.10] · p95 0.20 | 0.30 [0.20–0.30] · p95 0.50 | 0.20 [0.20–0.20] · p95 0.40 | 0.20 [0.10–0.20] · p95 0.40 | 0.10 [0.10–0.10] · p95 0.30 |
| JS JSON.parse of the view | DIRECT | 18.2 [18.2–18.4] · p95 22.9 | 18.4 [17.9–19.0] · p95 25.9 | 28.9 [27.5–29.4] · p95 37.3 | 29.0 [27.1–29.5] · p95 36.2 | 29.8 [28.3–31.0] · p95 37.4 | 7.80 [7.50–8.00] · p95 11.2 |
| the same exported function natively | DIRECT | 22.7 [21.8–23.5] · p95 24.9 | 25.2 [24.6–26.2] · p95 28.6 | 114.7 [107.7–121.6] · p95 156.7 | 59.5 [56.5–64.3] · p95 82.3 | 132.8 [127.2–137.9] · p95 164.1 | 28.4 [28.4–30.4] · p95 35.8 |
| wasm execution penalty over native, same function | INFERRED [8] | 1.90 [1.30–3.80] ±1.35 | 3.00 [0.90–4.10] ±1.60 | 9.30 [0.30–20.9] ±16.3 | 9.90 [4.60–17.3] ±9.10 | 9.30 [3.00–19.2] ±13.1 | 4.40 [3.10–5.40] ±1.50 |
| glue overhead beyond the ABI pieces | INFERRED [9] | -0.20 [-1.20–1.90] ±1.55 | 2.00 [2.00–2.30] ±1.50 | -1.60 [-10.0–2.20] ±15.1 | -0.30 [-3.90–1.50] ±8.80 | -0.30 [-4.80–5.20] ±14.1 | -0.40 [-0.80–0.00] ±1.15 |

- [8] wasm abi.exec − native web.dispatch_view
- [9] raw.dispatch_view − abi.dispatch_view (whole)

## (8) The kit session path against dispatch_view

The primary baseline, main-local; the kit and raw calls ran in the same interleaved blocks, so the differences are paired.

| Quantity (µs) | Label | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | ledger evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| kit session.dispatch() (dispatch_outcome + full snapshot + JSON.parse + checks) | DIRECT | 137.6 [135.6–145.4] · p95 160.1 | 132.2 [131.3–138.8] · p95 172.3 | 267.2 [266.9–271.3] · p95 334.1 | 213.5 [212.9–215.3] · p95 266.5 | 284.7 [283.5–285.4] · p95 348.1 | 114.6 [114.0–115.6] · p95 146.0 |
| kit session.view() after it | DIRECT | 28.7 [28.3–30.3] · p95 33.5 | 27.3 [27.1–28.3] · p95 35.3 | 33.3 [33.3–33.7] · p95 41.3 | 37.5 [37.1–37.6] · p95 47.2 | 39.6 [39.5–39.7] · p95 48.8 | 14.5 [14.3–14.6] · p95 18.9 |
| kit dispatch + view | DIRECT | 166.4 [164.1–175.9] · p95 192.8 | 159.5 [158.7–167.4] · p95 212.4 | 300.8 [300.5–305.9] · p95 377.7 | 251.6 [250.7–253.5] · p95 313.4 | 325.2 [323.8–325.3] · p95 396.6 | 129.0 [128.3–130.1] · p95 164.9 |
| raw dispatch_view + JSON.parse | DIRECT | 43.8 [40.9–45.2] · p95 48.2 | 46.6 [43.0–46.7] · p95 62.3 | 141.7 [140.3–145.2] · p95 183.8 | 92.8 [91.0–95.7] · p95 120.1 | 162.6 [158.9–165.7] · p95 192.6 | 38.1 [37.6–38.7] · p95 48.7 |
| kit path over dispatch_view | INFERRED [5] | 125.5 [118.9–132.1] ±8.05 | 116.5 [112.1–120.7] ±6.20 | 158.8 [155.6–165.6] ±5.15 | 157.9 [155.9–162.5] ±3.75 | 161.2 [159.6–166.3] ±4.15 | 90.9 [89.6–92.5] ±1.45 |
| kit JS layer over the raw outcome call | INFERRED [6] | -3.70 [-15.9–7.40] ±12.7 | -1.80 [-10.9–6.10] ±8.95 | 3.60 [1.00–14.2] ±6.60 | 1.40 [0.10–8.50] ±4.20 | 1.80 [1.00–9.80] ±4.60 | 3.40 [0.80–3.50] ±1.80 |

- [5] kit.dispatch+view − raw.dispatch_view+parse
- [6] kit.dispatch − raw.dispatch_outcome+parse

## Attribution on the dispatch_view path (web pages, adapter): language execution, view building, serialization and bridge

Each part timed on its own (i1 throwaway build for the first two; the ordinary build for the rest) in the instrumented run. They are disjoint pieces of one dispatch_view call, but timed in separate calls, so their medians are not added; the whole is shown as its own DIRECT measurement.

| Quantity (µs) | Label | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | ledger evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| language execution: payload parse, names, transaction, rules, change detection, bindings, commit (wasm, i1 bench_dispatch_only) | DIRECT | 14.6 [14.5–14.7] · p95 18.4 | 18.8 [17.3–20.3] · p95 27.6 | 108.9 [108.7–110.6] · p95 160.3 | 54.0 [53.4–55.4] · p95 80.4 | 126.9 [126.7–130.8] · p95 164.9 | 23.7 [23.2–25.3] · p95 31.0 |
| view building: name map, sort, records, assembly, drop (wasm, i1 bench_view_build) | DIRECT | 3.90 [3.60–4.00] · p95 4.60 | 3.00 [2.90–3.20] · p95 4.10 | 2.60 [2.50–2.70] · p95 3.60 | 3.60 [3.50–3.60] · p95 5.30 | 3.50 [3.50–3.60] · p95 5.10 | 5.00 [4.90–5.30] · p95 6.60 |
| view serialization inside wasm (serde_json) | INFERRED [3] | 6.30 [5.80–6.40] ±0.70 | 6.00 [5.60–6.20] ±0.60 | 11.8 [11.6–12.1] ±0.45 | 11.5 [11.4–11.6] ±0.20 | 12.3 [12.3–12.5] ±0.15 | 3.00 [2.90–3.30] ±0.60 |
| bridge: argument copy | DIRECT | 0.10 [0.10–0.10] · p95 0.30 | 0.30 [0.30–0.30] · p95 0.60 | 0.70 [0.60–0.70] · p95 1.40 | 0.40 [0.40–0.40] · p95 0.90 | 0.40 [0.30–0.40] · p95 0.70 | 0.30 [0.30–0.30] · p95 0.80 |
| bridge: result decode to a JS string | DIRECT | 2.90 [2.70–2.90] · p95 3.80 | 2.80 [2.80–2.90] · p95 4.30 | 2.10 [2.00–2.20] · p95 5.20 | 1.30 [1.30–1.50] · p95 3.80 | 1.40 [1.30–1.50] · p95 4.30 | 0.70 [0.60–0.70] · p95 2.10 |
| bridge: result free | DIRECT | 0.10 [0.10–0.10] · p95 0.10 | 0.10 [0.10–0.10] · p95 0.20 | 0.30 [0.20–0.30] · p95 0.50 | 0.20 [0.20–0.20] · p95 0.40 | 0.20 [0.10–0.20] · p95 0.40 | 0.10 [0.10–0.10] · p95 0.30 |
| JS: JSON.parse of the view | DIRECT | 18.2 [18.2–18.4] · p95 22.9 | 18.4 [17.9–19.0] · p95 25.9 | 28.9 [27.5–29.4] · p95 37.3 | 29.0 [27.1–29.5] · p95 36.2 | 29.8 [28.3–31.0] · p95 37.4 | 7.80 [7.50–8.00] · p95 11.2 |
| whole: dispatch_view through the glue + JSON.parse | DIRECT | 46.7 [46.3–48.3] · p95 58.0 | 52.0 [51.2–53.4] · p95 74.3 | 157.4 [147.4–161.1] · p95 212.5 | 104.7 [97.0–106.3] · p95 141.8 | 179.9 [168.1–180.1] · p95 221.7 | 42.8 [41.8–43.5] · p95 55.6 |
| whole, wasm execution only | DIRECT | 24.8 [24.6–25.6] · p95 31.4 | 28.2 [27.1–28.7] · p95 41.5 | 121.9 [117.0–135.6] · p95 171.1 | 68.9 [66.4–76.8] · p95 96.1 | 140.9 [136.5–152.0] · p95 177.9 | 33.5 [32.8–33.8] · p95 43.6 |

- [3] i1 abi.view.exec − i1 abi.view_build.exec, the same step

## Attribution on the kit path: language execution, snapshot reporting, serialization and bridge

As above, for the `dispatch_outcome` call the kit's `session.dispatch()` makes, all from the instrumented run; the kit's own JavaScript and its extra `view()` call are in section (8).

| Quantity (µs) | Label | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | ledger evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| language execution (wasm, i1 bench_dispatch_only) | DIRECT | 14.6 [14.5–14.7] · p95 18.4 | 18.8 [17.3–20.3] · p95 27.6 | 108.9 [108.7–110.6] · p95 160.3 | 54.0 [53.4–55.4] · p95 80.4 | 126.9 [126.7–130.8] · p95 164.9 | 23.7 [23.2–25.3] · p95 31.0 |
| snapshot + outcome built, serialized and dropped inside wasm | INFERRED [2] | 50.2 [49.1–51.3] ±1.10 | 47.3 [47.0–50.4] ±4.70 | 89.9 [88.9–99.0] ±5.60 | 86.4 [85.5–93.3] ±4.15 | 85.7 [83.0–97.5] ±7.85 | 44.3 [43.5–46.1] ±1.70 |
| bridge: outcome decode to a JS string | DIRECT | 8.80 [8.60–8.90] · p95 12.5 | 8.00 [7.70–8.30] · p95 11.3 | 4.50 [4.50–4.80] · p95 11.0 | 3.70 [3.70–3.90] · p95 10.5 | 3.80 [3.70–4.00] · p95 11.1 | 2.80 [2.80–2.90] · p95 10.0 |
| JS: JSON.parse of the outcome | DIRECT | 70.9 [69.6–74.4] · p95 81.3 | 64.0 [62.2–66.3] · p95 77.4 | 86.0 [83.6–87.5] · p95 104.4 | 84.1 [81.6–84.7] · p95 99.6 | 86.3 [83.9–86.8] · p95 101.8 | 53.6 [51.6–55.6] · p95 67.6 |
| whole: raw dispatch_outcome + JSON.parse | DIRECT | 152.3 [148.9–160.1] · p95 176.4 | 141.5 [136.3–147.3] · p95 186.0 | 286.9 [278.0–291.1] · p95 361.0 | 230.9 [222.9–234.9] · p95 293.1 | 306.0 [298.3–308.3] · p95 374.3 | 130.8 [125.5–134.5] · p95 166.4 |

- [2] i1 abi.dispatch_outcome exec − i1 abi.dispatch_only exec

## Binding evaluation, by difference (glowcap-unbound)

The unbound program is the replay program without its 39 one-line bind statements; the same stream. Native apply, paired by repeat.

| Session | Event kind | apply, replay (DIRECT) | apply, unbound (DIRECT) | binding evaluation (INFERRED) |
| --- | --- | --- | --- | --- |
| primary baseline | idle tick | 13.0 [13.0–13.8] · p95 14.4 | 12.8 [11.8–13.0] · p95 13.7 | 0.20 [0.00–2.00] ±1.00 |
| primary baseline | state-changing tick | 16.5 [16.2–17.1] · p95 17.8 | 14.2 [13.0–14.7] · p95 14.9 | 2.00 [1.80–4.10] ±1.30 |
| primary baseline | all | 13.1 [13.0–13.8] · p95 16.1 | 12.8 [11.8–13.0] · p95 14.2 | 0.20 [0.10–2.00] ±1.00 |
| instrumented session | idle tick | 15.0 [14.2–15.4] · p95 16.5 | 15.4 [13.7–16.7] · p95 16.5 | -1.20 [-1.70–1.70] ±2.10 |
| instrumented session | state-changing tick | 19.0 [17.5–19.1] · p95 20.9 | 17.2 [15.6–18.8] · p95 18.9 | 0.30 [0.20–3.50] ±2.40 |
| instrumented session | all | 15.0 [14.3–15.4] · p95 18.6 | 15.4 [13.7–16.8] · p95 17.1 | -1.10 [-1.80–1.70] ±2.10 |

## Size scaling: Glowcap with 4, 16 and 64 mushrooms (main-local)

The first 2,000 events of the replay stream on each program; idle ticks only. DIRECT.

| Operation | 4 mushrooms (replay) | 16 | 64 |
| --- | --- | --- | --- |
| native apply / apply | 13.0 [13.0–13.8] · p95 14.4 | 29.5 [27.6–34.6] · p95 31.4 | 88.3 [86.9–91.3] · p95 96.5 |
| native read / view | 2.60 [2.40–2.60] · p95 2.90 | 6.30 [6.00–6.30] · p95 6.80 | 22.9 [22.3–23.8] · p95 25.7 |
| native read / view.serialize | 5.00 [4.70–5.10] · p95 5.40 | 9.20 [8.70–9.30] · p95 10.3 | 24.9 [24.6–26.3] · p95 30.8 |
| native read / snapshot | 26.3 [24.5–27.1] · p95 28.5 | 60.4 [57.7–61.2] · p95 67.5 | 192.5 [189.1–203.1] · p95 224.9 |
| native read / snapshot.serialize | 15.9 [15.0–16.3] · p95 17.3 | 34.8 [32.4–34.8] · p95 40.2 | 103.1 [102.3–109.1] · p95 149.7 |
| native read / clone | 7.60 [7.10–7.70] · p95 8.10 | 18.6 [17.9–18.7] · p95 20.2 | 60.2 [58.7–62.9] · p95 68.1 |
| wasm abi.dispatch_view / abi.exec | 22.5 [21.2–22.5] · p95 25.3 | 50.7 [48.4–51.5] · p95 58.1 | 162.2 [155.1–163.9] · p95 178.6 |
| wasm raw.dispatch_view / js.parse_view | 17.1 [16.1–17.9] · p95 19.1 | 38.5 [37.2–42.3] · p95 43.4 | 122.3 [116.7–124.6] · p95 135.5 |
| wasm kit / kit.dispatch+view | 166.4 [164.1–175.9] · p95 192.8 | 393.8 [392.5–415.0] · p95 469.2 | 1523 [1477–1536] · p95 1701 |

## Growth within a session: the ledger's 20 events by position

Each ledger episode starts from a fresh session and its graph, journal and renewals grow over its 20 events, so the same kind of event can be compared early and late. DIRECT; each position has 300 samples a run (100 plays × 3 passes). Native and wasm execution from the instrumented session; the kit path from the primary baseline.

| Position and event | native apply µs | wasm dispatch_view exec µs | wasm dispatch_outcome exec µs | kit dispatch + view µs |
| --- | --- | --- | --- | --- |
| 00 pushed evidence [reveal] | 17.5 [17.2–20.9] | 32.7 [31.5–34.7] | 74.5 [73.5–76.6] | 143.6 [143.5–145.0] |
| 01 pushed evidence [renew,reveal] | 22.5 [22.1–26.6] | 37.1 [35.8–39.3] | 73.1 [72.0–75.8] | 130.0 [129.2–130.7] |
| 02 pushed evidence [renew,reveal] | 20.7 [20.3–24.6] | 34.2 [33.5–36.6] | 69.7 [68.7–72.9] | 124.6 [124.1–125.4] |
| 03 checks evidence [sample] | 23.1 [22.5–26.7] | 36.2 [35.2–38.3] | 72.0 [70.9–74.0] | 128.2 [127.8–129.6] |
| 04 merge refused [] reject | 3.50 [3.40–4.10] | 5.90 [5.80–6.10] | 5.70 [5.70–6.00] | 19.5 [19.4–19.8] |
| 05 pushed evidence [reveal] | 15.0 [14.6–17.6] | 26.0 [25.5–27.3] | 60.3 [59.1–62.8] | 114.9 [114.9–116.6] |
| 06 pushed evidence [renew,reveal] | 20.9 [20.3–24.6] | 33.3 [32.4–35.2] | 68.0 [66.2–70.6] | 123.3 [123.1–124.8] |
| 07 pushed evidence [renew,reveal] | 21.4 [20.8–25.7] | 33.7 [32.8–35.9] | 68.8 [67.2–71.8] | 124.6 [124.4–125.5] |
| 08 checks evidence [sample] | 22.7 [22.2–26.9] | 36.4 [35.4–38.9] | 71.6 [69.7–75.3] | 128.8 [128.6–130.2] |
| 09 merged_elsewhere evidence [reveal] | 13.9 [13.4–16.2] | 25.4 [24.9–26.9] | 61.3 [59.2–63.7] | 119.0 [118.0–120.0] |
| 10 gates evidence [sample] | 20.4 [19.9–24.6] | 33.9 [32.9–36.1] | 70.1 [68.8–72.7] | 131.1 [130.5–132.6] |
| 11 approved evidence [reveal] | 15.4 [15.0–18.5] | 27.1 [26.4–28.5] | 63.7 [62.1–66.2] | 125.4 [123.9–125.8] |
| 12 merge refused [] reject | 3.30 [3.30–3.90] | 5.50 [5.50–5.80] | 5.50 [5.50–5.70] | 21.8 [21.5–22.0] |
| 13 pushed evidence [reveal] | 16.1 [15.6–19.3] | 28.1 [27.2–29.3] | 64.2 [63.0–66.8] | 126.1 [125.6–127.9] |
| 14 merge refused [] reject | 4.30 [4.20–5.00] | 6.40 [6.30–6.60] | 6.50 [6.40–6.80] | 22.8 [22.7–22.9] |
| 15 pushed evidence [reveal] | 15.6 [15.3–18.6] | 27.8 [26.9–29.3] | 63.7 [62.0–66.1] | 126.1 [125.9–128.1] |
| 16 checks evidence [sample] | 24.1 [23.4–29.1] | 39.3 [38.5–42.1] | 77.5 [75.5–80.9] | 141.9 [141.9–144.7] |
| 17 merge refused [] reject | 5.00 [4.80–5.80] | 7.00 [6.80–7.40] | 7.30 [7.10–7.50] | 25.0 [24.9–25.3] |
| 18 checks evidence [sample] | 24.4 [23.8–28.4] | 39.7 [38.6–42.4] | 78.2 [76.5–81.3] | 144.4 [144.3–147.0] |
| 19 merge refused [] reject | 5.90 [5.70–6.90] | 8.10 [8.00–8.50] | 8.10 [8.10–8.50] | 26.2 [25.9–26.3] |

## (h) Native against WebAssembly, the same exported function

Native `web::WebReactiveSession` against the WebAssembly export's execution alone (`abi.*.exec`, no glue), main-local, all events of each workload. Ratio = median over runs of (wasm median / native median).

| Session | Workload | Operation | native µs (DIRECT) | wasm µs (DIRECT) | ratio | difference (INFERRED) |
| --- | --- | --- | --- | --- | --- | --- |
| primary baseline | glowcap-replay | dispatch_view | 20.0 [20.0–21.3] · p95 22.5 | 22.6 [21.2–22.6] · p95 26.2 | 1.06× [1.06–1.13] | 1.30 [1.20–2.60] ±1.35 |
| primary baseline | glowcap-replay | dispatch_outcome | 64.7 [64.4–70.2] · p95 69.2 | 61.0 [60.9–61.4] · p95 68.4 | 0.95× [0.87–0.95] | -3.40 [-9.30–-3.30] ±3.15 |
| primary baseline | glowcap-replay | view | 6.80 [6.30–6.90] · p95 7.30 | 8.50 [8.20–8.70] · p95 9.40 | 1.28× [1.23–1.30] | 1.90 [1.60–1.90] ±0.55 |
| primary baseline | glowcap-replay | snapshot (pretty) | 70.7 [65.3–72.3] · p95 76.4 | 69.3 [68.8–73.1] · p95 81.1 | 1.03× [0.95–1.06] | 2.40 [-3.50–4.00] ±5.65 |
| primary baseline | glowcap-replay | save | 16.8 [15.5–17.3] · p95 17.9 | 15.1 [15.0–15.9] · p95 18.7 | 0.95× [0.87–0.97] | -0.90 [-2.20–-0.50] ±1.35 |
| primary baseline | glowcap-replay | new (load) | 2723 [2707–2809] · p95 2871 | 2576 [2496–2577] · p95 2879 | 0.92× [0.92–0.95] | -211.3 [-232.9–-146.9] ±91.5 |
| primary baseline | glowcap-replay | restore | 2927 [2902–3042] · p95 3036 | 2635 [2590–2688] · p95 3179 | 0.89× [0.87–0.92] | -311.9 [-406.5–-239.1] ±118.8 |
| primary baseline | trail-rescue-scenarios | dispatch_view | 58.4 [57.7–59.4] · p95 132.1 | 70.6 [70.0–72.1] · p95 147.5 | 1.22× [1.18–1.23] | 12.9 [10.6–13.7] ±1.90 |
| primary baseline | trail-rescue-scenarios | dispatch_outcome | 117.2 [116.2–125.8] · p95 194.5 | 132.2 [129.1–135.1] · p95 214.3 | 1.11× [1.05–1.15] | 12.9 [6.40–17.9] ±7.80 |
| primary baseline | trail-rescue-scenarios | view | 6.10 [6.10–6.10] · p95 8.60 | 9.20 [9.10–9.40] · p95 12.5 | 1.51× [1.49–1.54] | 3.10 [3.00–3.30] ±0.15 |
| primary baseline | trail-rescue-scenarios | snapshot (pretty) | 79.8 [79.4–80.0] · p95 91.3 | 99.2 [99.0–101.4] · p95 119.2 | 1.25× [1.24–1.27] | 19.6 [19.4–21.4] ±1.50 |
| primary baseline | trail-rescue-scenarios | save | 10.8 [10.7–10.8] · p95 18.1 | 13.4 [13.4–13.7] · p95 23.2 | 1.25× [1.24–1.27] | 2.70 [2.60–2.90] ±0.20 |
| primary baseline | trail-rescue-scenarios | new (load) | 2913 [2894–2945] · p95 3038 | 2989 [2985–3287] · p95 3174 | 1.03× [1.01–1.13] | 94.7 [40.2–373.4] ±176.3 |
| primary baseline | trail-rescue-scenarios | restore | 3037 [3017–3072] · p95 3229 | 3001 [2988–3287] · p95 3201 | 0.99× [0.97–1.08] | -15.1 [-84.0–249.8] ±177.3 |
| primary baseline | ledger-session | dispatch_view | 24.2 [23.1–24.9] · p95 33.6 | 27.6 [27.6–28.3] · p95 37.7 | 1.17× [1.11–1.19] | 4.10 [2.70–4.50] ±1.25 |
| primary baseline | ledger-session | dispatch_outcome | 56.6 [56.1–63.7] · p95 71.9 | 57.9 [57.5–58.6] · p95 73.8 | 1.02× [0.91–1.04] | 1.40 [-5.80–2.00] ±4.35 |
| primary baseline | ledger-session | view | 5.10 [5.10–5.60] · p95 7.30 | 6.70 [6.60–6.90] · p95 9.20 | 1.29× [1.23–1.31] | 1.50 [1.30–1.60] ±0.40 |
| primary baseline | ledger-session | snapshot (pretty) | 50.8 [50.6–55.4] · p95 59.3 | 61.1 [60.7–63.3] · p95 73.1 | 1.20× [1.14–1.20] | 10.1 [7.90–10.3] ±3.70 |
| primary baseline | ledger-session | save | 12.4 [12.4–13.6] · p95 16.9 | 14.8 [14.8–15.2] · p95 20.2 | 1.19× [1.12–1.19] | 2.40 [1.60–2.40] ±0.80 |
| primary baseline | ledger-session | new (load) | 1287 [1284–1304] · p95 1356 | 1605 [1482–1616] · p95 1731 | 1.25× [1.14–1.26] | 321.7 [178.3–329.4] ±77.0 |
| primary baseline | ledger-session | restore | 1349 [1346–1355] · p95 1476 | 1719 [1566–1723] · p95 1947 | 1.27× [1.16–1.28] | 367.5 [216.8–373.4] ±83.0 |
| instrumented session | glowcap-replay | dispatch_view | 22.7 [21.9–23.5] · p95 26.1 | 24.9 [24.7–25.8] · p95 32.1 | 1.09× [1.06–1.18] | 2.00 [1.40–3.90] ±1.35 |
| instrumented session | glowcap-replay | dispatch_outcome | 75.1 [69.6–77.6] · p95 83.8 | 65.7 [65.1–67.3] · p95 78.2 | 0.87× [0.84–0.97] | -9.40 [-12.5–-2.30] ±5.10 |
| instrumented session | glowcap-replay | view | 7.50 [7.30–7.60] · p95 8.70 | 9.60 [9.20–10.1] · p95 11.1 | 1.32× [1.21–1.35] | 2.30 [1.60–2.60] ±0.60 |
| instrumented session | glowcap-replay | snapshot (pretty) | 77.6 [76.4–79.9] · p95 89.1 | 80.1 [77.2–84.6] · p95 93.5 | 1.05× [0.97–1.09] | 3.70 [-2.70–7.00] ±5.45 |
| instrumented session | glowcap-replay | save | 18.5 [18.1–18.8] · p95 21.4 | 17.1 [16.7–18.2] · p95 21.0 | 0.94× [0.89–0.98] | -1.00 [-2.10–-0.30] ±1.10 |
| instrumented session | glowcap-replay | new (load) | 3034 [2840–3188] · p95 3170 | 2768 [2706–3180] · p95 3296 | 0.89× [0.87–1.12] | -327.8 [-419.5–340.0] ±410.9 |
| instrumented session | glowcap-replay | restore | 3314 [3099–3395] · p95 3419 | 2795 [2770–3315] · p95 3609 | 0.84× [0.82–1.07] | -519.1 [-625.0–215.7] ±420.4 |
| instrumented session | trail-rescue-scenarios | dispatch_view | 65.5 [61.1–68.6] · p95 146.8 | 75.8 [73.2–82.2] · p95 157.9 | 1.20× [1.10–1.25] | 12.1 [7.20–16.7] ±8.25 |
| instrumented session | trail-rescue-scenarios | dispatch_outcome | 128.0 [123.2–128.6] · p95 215.0 | 154.5 [145.0–157.2] · p95 256.2 | 1.22× [1.13–1.25] | 28.6 [17.0–31.3] ±8.80 |
| instrumented session | trail-rescue-scenarios | view | 6.80 [6.70–6.80] · p95 9.60 | 10.1 [9.60–10.4] · p95 13.9 | 1.49× [1.41–1.55] | 3.30 [2.80–3.70] ±0.45 |
| instrumented session | trail-rescue-scenarios | snapshot (pretty) | 86.8 [86.7–88.1] · p95 103.0 | 109.8 [102.6–110.3] · p95 131.8 | 1.25× [1.18–1.27] | 22.2 [15.8–23.1] ±4.55 |
| instrumented session | trail-rescue-scenarios | save | 11.9 [11.8–12.0] · p95 20.2 | 14.9 [14.0–15.0] · p95 25.9 | 1.24× [1.18–1.27] | 2.90 [2.10–3.20] ±0.60 |
| instrumented session | trail-rescue-scenarios | new (load) | 3164 [3150–3232] · p95 3361 | 3241 [3221–3583] · p95 3886 | 1.02× [1.00–1.14] | 56.3 [8.90–433.1] ±222.2 |
| instrumented session | trail-rescue-scenarios | restore | 3297 [3284–3318] · p95 3524 | 3271 [3213–3539] · p95 3879 | 1.00× [0.97–1.07] | -13.7 [-105.2–242.0] ±179.8 |
| instrumented session | ledger-session | dispatch_view | 26.1 [25.6–27.7] · p95 35.2 | 30.6 [30.0–30.7] · p95 42.6 | 1.15× [1.11–1.20] | 3.90 [3.00–5.00] ±1.40 |
| instrumented session | ledger-session | dispatch_outcome | 63.2 [61.9–64.7] · p95 80.8 | 66.1 [64.7–68.7] · p95 86.3 | 1.06× [1.02–1.07] | 4.00 [1.50–4.20] ±3.40 |
| instrumented session | ledger-session | view | 5.70 [5.60–5.90] · p95 8.20 | 7.80 [7.40–7.90] · p95 10.9 | 1.39× [1.25–1.39] | 2.20 [1.50–2.20] ±0.40 |
| instrumented session | ledger-session | snapshot (pretty) | 57.2 [56.4–58.6] · p95 69.1 | 71.6 [66.8–73.9] · p95 88.0 | 1.27× [1.14–1.29] | 15.2 [8.20–16.7] ±4.65 |
| instrumented session | ledger-session | save | 14.0 [13.9–14.4] · p95 18.7 | 16.9 [16.2–17.4] · p95 24.3 | 1.22× [1.13–1.24] | 3.00 [1.80–3.40] ±0.85 |
| instrumented session | ledger-session | new (load) | 1542 [1460–1589] · p95 1693 | 1645 [1587–1778] · p95 1881 | 1.12× [1.03–1.13] | 184.7 [45.2–188.5] ±159.9 |
| instrumented session | ledger-session | restore | 1593 [1514–1659] · p95 1687 | 1780 [1736–1851] · p95 2106 | 1.12× [1.09–1.18] | 191.4 [143.4–266.7] ±130.0 |

Load and restore rows use the WebAssembly glue calls (`raw.new`, `raw.restore`), which add the source and save copies into memory.

## (f) save and (g) restore

At each workload's first episode's final state (the resume stream: ten minutes of play). DIRECT, 30 timed calls per run after 3 untimed.

| Workload | Target | Engine | Operation | µs: median [runs] · p95 |
| --- | --- | --- | --- | --- |
| glowcap-resume | main-local | native | save_json | 23.6 [16.3–25.0] · p95 30.6 |
| glowcap-resume | main-local | native | save | 13.3 [13.1–13.4] · p95 17.7 |
| glowcap-resume | main-local | native | restore.parse | 19.4 [18.8–19.8] · p95 25.3 |
| glowcap-resume | main-local | native | restore | 2655 [2631–2792] · p95 2831 |
| glowcap-resume | main-local | native | restore_json | 2667 [2641–2783] · p95 2821 |
| glowcap-resume | main-local | native | from_source | 2568 [2537–2778] · p95 2664 |
| glowcap-resume | main-local | wasm | raw.save | 34.1 [32.4–34.3] · p95 46.2 |
| glowcap-resume | main-local | wasm | raw.restore | 2581 [2491–2670] · p95 2786 |
| glowcap-resume | main-local | wasm | raw.new | 2595 [2499–2688] · p95 2832 |
| glowcap-resume | main-local | wasm | kit.save | 72.8 [72.8–73.4] · p95 110.8 |
| glowcap-resume | main-local | wasm | kit.restore | 2573 [2488–2686] · p95 2778 |
| glowcap-resume | rc4-published | wasm | raw.save | 34.1 [33.6–37.1] · p95 45.5 |
| glowcap-resume | rc4-published | wasm | raw.restore | 2600 [2452–2849] · p95 2823 |
| glowcap-resume | rc4-published | wasm | raw.new | 2636 [2429–2843] · p95 2992 |
| glowcap-resume | rc4-published | wasm | kit.save | 73.0 [70.7–77.1] · p95 101.0 |
| glowcap-resume | rc4-published | wasm | kit.restore | 2558 [2384–2817] · p95 2801 |
| glowcap-replay | main-local | native | save_json | 29.1 [27.4–30.8] · p95 43.8 |
| glowcap-replay | main-local | native | save | 23.7 [20.7–24.5] · p95 32.8 |
| glowcap-replay | main-local | native | restore.parse | 26.4 [24.9–27.0] · p95 30.6 |
| glowcap-replay | main-local | native | restore | 2831 [2824–2939] · p95 2975 |
| glowcap-replay | main-local | native | restore_json | 2952 [2917–3031] · p95 3067 |
| glowcap-replay | main-local | native | from_source | 2711 [2711–2848] · p95 2918 |
| glowcap-replay | main-local | wasm | raw.save | 47.7 [47.3–48.6] · p95 59.3 |
| glowcap-replay | main-local | wasm | raw.restore | 2635 [2590–2688] · p95 3179 |
| glowcap-replay | main-local | wasm | raw.new | 2576 [2496–2577] · p95 2879 |
| glowcap-replay | main-local | wasm | kit.save | 90.4 [89.4–95.4] · p95 120.1 |
| glowcap-replay | main-local | wasm | kit.restore | 2609 [2554–2614] · p95 3003 |
| glowcap-replay | rc4-published | wasm | raw.save | 48.9 [48.6–49.5] · p95 67.6 |
| glowcap-replay | rc4-published | wasm | raw.restore | 2830 [2731–2879] · p95 3124 |
| glowcap-replay | rc4-published | wasm | raw.new | 2744 [2642–2781] · p95 2985 |
| glowcap-replay | rc4-published | wasm | kit.save | 94.3 [87.6–101.9] · p95 130.2 |
| glowcap-replay | rc4-published | wasm | kit.restore | 2774 [2689–2836] · p95 2957 |
| trail-rescue-scenarios | main-local | native | save_json | 14.7 [14.1–17.3] · p95 16.9 |
| trail-rescue-scenarios | main-local | native | save | 12.9 [12.0–13.6] · p95 15.7 |
| trail-rescue-scenarios | main-local | native | restore.parse | 19.1 [18.7–19.2] · p95 24.7 |
| trail-rescue-scenarios | main-local | native | restore | 2991 [2975–3034] · p95 3148 |
| trail-rescue-scenarios | main-local | native | restore_json | 3046 [3003–3056] · p95 3190 |
| trail-rescue-scenarios | main-local | native | from_source | 2891 [2889–2968] · p95 3137 |
| trail-rescue-scenarios | main-local | wasm | raw.save | 36.6 [35.4–40.8] · p95 50.3 |
| trail-rescue-scenarios | main-local | wasm | raw.restore | 3001 [2988–3287] · p95 3201 |
| trail-rescue-scenarios | main-local | wasm | raw.new | 2989 [2985–3287] · p95 3174 |
| trail-rescue-scenarios | main-local | wasm | kit.save | 76.5 [67.4–77.8] · p95 97.8 |
| trail-rescue-scenarios | main-local | wasm | kit.restore | 3015 [2961–3282] · p95 3159 |
| trail-rescue-scenarios | rc4-published | wasm | raw.save | 36.6 [34.9–41.1] · p95 66.6 |
| trail-rescue-scenarios | rc4-published | wasm | raw.restore | 2970 [2841–3325] · p95 3598 |
| trail-rescue-scenarios | rc4-published | wasm | raw.new | 2955 [2765–3338] · p95 3502 |
| trail-rescue-scenarios | rc4-published | wasm | kit.save | 72.3 [65.3–82.4] · p95 107.3 |
| trail-rescue-scenarios | rc4-published | wasm | kit.restore | 2976 [2787–3341] · p95 3500 |
| ledger-session | main-local | native | save_json | 20.9 [20.6–21.1] · p95 22.8 |
| ledger-session | main-local | native | save | 13.7 [13.6–13.7] · p95 14.9 |
| ledger-session | main-local | native | restore.parse | 22.7 [21.8–22.8] · p95 24.8 |
| ledger-session | main-local | native | restore | 1336 [1328–1360] · p95 1444 |
| ledger-session | main-local | native | restore_json | 1379 [1360–1393] · p95 1476 |
| ledger-session | main-local | native | from_source | 1309 [1287–1317] · p95 1419 |
| ledger-session | main-local | wasm | raw.save | 49.3 [45.4–50.4] · p95 70.6 |
| ledger-session | main-local | wasm | raw.restore | 1719 [1566–1723] · p95 1947 |
| ledger-session | main-local | wasm | raw.new | 1605 [1482–1616] · p95 1731 |
| ledger-session | main-local | wasm | kit.save | 92.4 [83.6–94.1] · p95 126.4 |
| ledger-session | main-local | wasm | kit.restore | 1676 [1533–1681] · p95 1839 |
| ledger-session | rc4-published | wasm | raw.save | 49.3 [46.4–49.9] · p95 67.7 |
| ledger-session | rc4-published | wasm | raw.restore | 1707 [1531–1709] · p95 1891 |
| ledger-session | rc4-published | wasm | raw.new | 1576 [1427–1602] · p95 1740 |
| ledger-session | rc4-published | wasm | kit.save | 89.0 [85.0–93.4] · p95 118.7 |
| ledger-session | rc4-published | wasm | kit.restore | 1692 [1555–1701] · p95 1921 |
| glowcap-resume (published resume method) | hist-e6ace96 | wasm | adapter.resume.parse | 14.9 [14.9–15.5] · p95 14.9 |
| glowcap-resume (published resume method) | hist-e6ace96 | wasm | adapter.resume.createPolicy | 3254 [3195–3317] · p95 3254 |
| glowcap-resume (published resume method) | hist-e6ace96 | wasm | adapter.resume.view | 49.9 [45.4–54.9] · p95 49.9 |
| glowcap-resume (published resume method) | hist-e6ace96 | wasm | adapter.resume | 3308 [3257–3372] · p95 3308 |
| glowcap-resume (published resume method) | rc4-published | wasm | adapter.resume.parse | 14.1 [13.7–14.4] · p95 14.1 |
| glowcap-resume (published resume method) | rc4-published | wasm | adapter.resume.createPolicy | 3362 [3147–3633] · p95 3362 |
| glowcap-resume (published resume method) | rc4-published | wasm | adapter.resume.view | 44.3 [42.5–47.9] · p95 44.3 |
| glowcap-resume (published resume method) | rc4-published | wasm | adapter.resume | 3410 [3203–3681] · p95 3410 |
| glowcap-resume (published resume method) | main-local | wasm | adapter.resume.parse | 13.9 [13.4–14.6] · p95 13.9 |
| glowcap-resume (published resume method) | main-local | wasm | adapter.resume.createPolicy | 3681 [3433–3717] · p95 3681 |
| glowcap-resume (published resume method) | main-local | wasm | adapter.resume.view | 43.1 [40.1–55.3] · p95 43.1 |
| glowcap-resume (published resume method) | main-local | wasm | adapter.resume | 3731 [3480–3764] · p95 3731 |

## Instrumentation safeguard

Each instrumented build against the ordinary build on the same events, in the same interleaved run: all events of the Glowcap replay (it has no refusals), and the accepted classes of the decision workloads (the ordinary `raw.*+parse` operations have no sample for a refused event, while a probe total does, so their pooled "all" would not compare like with like). i1 adds functions and changes none; i2 adds marks inside the runtime, so its totals include the marks' own cost.

| Workload: events | Instrumented | Ordinary (main-local) | Statistic | instrumented µs | ordinary µs | overhead (INFERRED) | overhead % |
| --- | --- | --- | --- | --- | --- | --- | --- |
| glowcap-replay: all | i1 native web.dispatch_view / web.dispatch_view | web.dispatch_view / web.dispatch_view | median | 22.8 | 22.7 | 0.10 [0.10–1.10] ±2.10 | 0.44% |
| glowcap-replay: all | i1 native web.dispatch_view / web.dispatch_view | web.dispatch_view / web.dispatch_view | mean | 23.1 | 23.0 | 0.10 [0.05–1.23] ±1.72 | 0.42% |
| glowcap-replay: all | i1 native web.dispatch_outcome / web.dispatch_outcome | web.dispatch_outcome / web.dispatch_outcome | median | 72.2 | 75.1 | -1.80 [-5.40–-0.60] ±6.15 | -2.40% |
| glowcap-replay: all | i1 native web.dispatch_outcome / web.dispatch_outcome | web.dispatch_outcome / web.dispatch_outcome | mean | 72.9 | 75.3 | -1.79 [-5.04–-1.25] ±4.43 | -2.38% |
| glowcap-replay: all | i1 native apply / apply | apply / apply | median | 15.0 | 15.0 | 0.10 [0.00–0.20] ±1.15 | 0.67% |
| glowcap-replay: all | i1 native apply / apply | apply / apply | mean | 15.4 | 15.5 | 0.08 [-0.10–0.14] ±0.91 | 0.54% |
| glowcap-replay: all | i1 native read / view | read / view | median | 2.70 | 2.70 | 0.00 [0.00–0.10] ±0.15 | 0.00% |
| glowcap-replay: all | i1 native read / view | read / view | mean | 2.74 | 2.75 | 0.02 [-0.03–0.09] ±0.17 | 0.66% |
| glowcap-replay: all | i1 wasm abi.dispatch_view / abi.exec | abi.dispatch_view / abi.exec | median | 24.3 | 24.9 | -0.40 [-2.90–0.10] ±1.60 | -1.61% |
| glowcap-replay: all | i1 wasm abi.dispatch_view / abi.exec | abi.dispatch_view / abi.exec | mean | 25.1 | 26.2 | -0.55 [-2.33–-0.38] ±1.26 | -2.08% |
| glowcap-replay: all | i1 wasm abi.dispatch_outcome / abi.exec | abi.dispatch_outcome / abi.exec | median | 64.8 | 65.7 | -0.90 [-3.40–0.70] ±2.05 | -1.37% |
| glowcap-replay: all | i1 wasm abi.dispatch_outcome / abi.exec | abi.dispatch_outcome / abi.exec | mean | 66.3 | 67.3 | -1.02 [-2.40–0.41] ±1.41 | -1.51% |
| glowcap-replay: all | i1 wasm raw.dispatch_view / raw.dispatch_view+parse | raw.dispatch_view / raw.dispatch_view+parse | median | 47.5 | 46.8 | 0.70 [-1.60–1.60] ±1.60 | 1.50% |
| glowcap-replay: all | i1 wasm raw.dispatch_view / raw.dispatch_view+parse | raw.dispatch_view / raw.dispatch_view+parse | mean | 49.4 | 49.7 | -1.58 [-2.67–0.33] ±2.17 | -3.19% |
| glowcap-replay: all | i2 native probe.dispatch_view / total | web.dispatch_view / web.dispatch_view | median | 23.6 | 22.7 | 0.90 [0.60–0.90] ±1.45 | 3.96% |
| glowcap-replay: all | i2 native probe.dispatch_view / total | web.dispatch_view / web.dispatch_view | mean | 23.9 | 23.0 | 0.89 [0.68–0.93] ±1.00 | 3.89% |
| glowcap-replay: all | i2 native probe.dispatch_outcome / total | web.dispatch_outcome / web.dispatch_outcome | median | 74.4 | 75.1 | 0.30 [-3.20–1.70] ±7.45 | 0.40% |
| glowcap-replay: all | i2 native probe.dispatch_outcome / total | web.dispatch_outcome / web.dispatch_outcome | mean | 74.4 | 75.3 | -0.28 [-3.53–3.25] ±6.46 | -0.38% |
| glowcap-replay: all | i2 wasm probe.dispatch_view / total | raw.dispatch_view / raw.dispatch_view+parse | median | 49.7 | 46.8 | 2.70 [1.30–2.90] ±1.20 | 5.77% |
| glowcap-replay: all | i2 wasm probe.dispatch_view / total | raw.dispatch_view / raw.dispatch_view+parse | mean | 52.9 | 49.7 | 2.88 [2.49–3.29] ±2.76 | 5.81% |
| glowcap-replay: all | i2 wasm probe.dispatch_outcome / total | raw.dispatch_outcome / raw.dispatch_outcome+parse | median | 159.9 | 151.7 | 5.30 [2.20–11.3] ±7.60 | 3.49% |
| glowcap-replay: all | i2 wasm probe.dispatch_outcome / total | raw.dispatch_outcome / raw.dispatch_outcome+parse | mean | 169.3 | 154.9 | 12.7 [9.96–20.3] ±6.03 | 8.17% |
| trail-rescue-scenarios: evidence | i1 native web.dispatch_view / web.dispatch_view | web.dispatch_view / web.dispatch_view | median | 113.4 | 114.7 | -0.10 [-8.20–1.30] ±11.2 | -0.09% |
| trail-rescue-scenarios: evidence | i1 native web.dispatch_view / web.dispatch_view | web.dispatch_view / web.dispatch_view | mean | 118.1 | 119.1 | -0.73 [-8.12–2.02] ±11.1 | -0.61% |
| trail-rescue-scenarios: evidence | i1 native web.dispatch_outcome / web.dispatch_outcome | web.dispatch_outcome / web.dispatch_outcome | median | 177.2 | 176.5 | 0.70 [-0.90–3.00] ±9.55 | 0.40% |
| trail-rescue-scenarios: evidence | i1 native web.dispatch_outcome / web.dispatch_outcome | web.dispatch_outcome / web.dispatch_outcome | mean | 182.6 | 181.6 | 0.95 [0.01–2.98] ±9.89 | 0.52% |
| trail-rescue-scenarios: evidence | i1 native apply / apply | apply / apply | median | 97.4 | 100.4 | 0.90 [-9.40–0.90] ±11.1 | 0.90% |
| trail-rescue-scenarios: evidence | i1 native apply / apply | apply / apply | mean | 104.2 | 106.5 | 0.88 [-6.97–1.00] ±9.40 | 0.83% |
| trail-rescue-scenarios: evidence | i1 native read / view | read / view | median | 1.20 | 1.10 | 0.00 [0.00–0.10] ±0.10 | 0.00% |
| trail-rescue-scenarios: evidence | i1 native read / view | read / view | mean | 1.27 | 1.22 | 0.05 [0.04–0.07] ±0.08 | 3.84% |
| trail-rescue-scenarios: evidence | i1 wasm abi.dispatch_view / abi.exec | abi.dispatch_view / abi.exec | median | 125.6 | 121.9 | 3.50 [-5.20–3.70] ±14.3 | 2.87% |
| trail-rescue-scenarios: evidence | i1 wasm abi.dispatch_view / abi.exec | abi.dispatch_view / abi.exec | mean | 131.0 | 127.3 | 3.66 [-4.59–3.76] ±13.2 | 2.88% |
| trail-rescue-scenarios: evidence | i1 wasm abi.dispatch_outcome / abi.exec | abi.dispatch_outcome / abi.exec | median | 199.5 | 204.9 | 3.00 [-16.1–5.20] ±14.8 | 1.46% |
| trail-rescue-scenarios: evidence | i1 wasm abi.dispatch_outcome / abi.exec | abi.dispatch_outcome / abi.exec | mean | 204.2 | 212.0 | 0.75 [-11.8–4.36] ±12.9 | 0.35% |
| trail-rescue-scenarios: evidence | i1 wasm raw.dispatch_view / raw.dispatch_view+parse | raw.dispatch_view / raw.dispatch_view+parse | median | 158.5 | 157.4 | 1.40 [-2.60–2.80] ±11.2 | 0.89% |
| trail-rescue-scenarios: evidence | i1 wasm raw.dispatch_view / raw.dispatch_view+parse | raw.dispatch_view / raw.dispatch_view+parse | mean | 165.1 | 164.8 | 0.24 [-1.29–1.32] ±11.0 | 0.15% |
| trail-rescue-scenarios: evidence | i2 native probe.dispatch_view / total | web.dispatch_view / web.dispatch_view | median | 115.8 | 114.7 | -0.20 [-1.30–8.10] ±9.85 | -0.17% |
| trail-rescue-scenarios: evidence | i2 native probe.dispatch_view / total | web.dispatch_view / web.dispatch_view | mean | 121.3 | 119.1 | 0.03 [-1.08–8.39] ±9.68 | 0.03% |
| trail-rescue-scenarios: evidence | i2 native probe.dispatch_outcome / total | web.dispatch_outcome / web.dispatch_outcome | median | 184.0 | 176.5 | 7.40 [2.20–15.4] ±6.65 | 4.19% |
| trail-rescue-scenarios: evidence | i2 native probe.dispatch_outcome / total | web.dispatch_outcome / web.dispatch_outcome | mean | 189.8 | 181.6 | 7.81 [1.92–16.4] ±7.42 | 4.30% |
| trail-rescue-scenarios: evidence | i2 wasm probe.dispatch_view / total | raw.dispatch_view / raw.dispatch_view+parse | median | 166.3 | 157.4 | 5.20 [4.00–20.7] ±10.2 | 3.30% |
| trail-rescue-scenarios: evidence | i2 wasm probe.dispatch_view / total | raw.dispatch_view / raw.dispatch_view+parse | mean | 171.4 | 164.8 | 4.95 [2.05–19.0] ±9.29 | 3.00% |
| trail-rescue-scenarios: evidence | i2 wasm probe.dispatch_outcome / total | raw.dispatch_outcome / raw.dispatch_outcome+parse | median | 306.2 | 286.9 | 20.1 [15.1–28.5] ±15.2 | 7.01% |
| trail-rescue-scenarios: evidence | i2 wasm probe.dispatch_outcome / total | raw.dispatch_outcome / raw.dispatch_outcome+parse | mean | 313.8 | 296.0 | 20.2 [13.7–27.7] ±13.5 | 6.84% |
| trail-rescue-scenarios: commit | i1 native web.dispatch_view / web.dispatch_view | web.dispatch_view / web.dispatch_view | median | 59.5 | 59.5 | -0.20 [-4.80–1.00] ±6.00 | -0.34% |
| trail-rescue-scenarios: commit | i1 native web.dispatch_view / web.dispatch_view | web.dispatch_view / web.dispatch_view | mean | 62.1 | 62.6 | -0.24 [-4.22–1.19] ±6.36 | -0.39% |
| trail-rescue-scenarios: commit | i1 native web.dispatch_outcome / web.dispatch_outcome | web.dispatch_outcome / web.dispatch_outcome | median | 121.9 | 121.9 | 0.20 [0.00–3.10] ±6.85 | 0.16% |
| trail-rescue-scenarios: commit | i1 native web.dispatch_outcome / web.dispatch_outcome | web.dispatch_outcome / web.dispatch_outcome | mean | 124.7 | 124.6 | 0.08 [0.04–2.36] ±6.82 | 0.06% |
| trail-rescue-scenarios: commit | i1 native apply / apply | apply / apply | median | 45.6 | 47.0 | -0.30 [-4.70–0.50] ±4.65 | -0.64% |
| trail-rescue-scenarios: commit | i1 native apply / apply | apply / apply | mean | 48.8 | 49.9 | 0.34 [-3.40–0.45] ±4.43 | 0.68% |
| trail-rescue-scenarios: commit | i1 native read / view | read / view | median | 1.70 | 1.70 | 0.00 [0.00–0.00] ±0.10 | 0.00% |
| trail-rescue-scenarios: commit | i1 native read / view | read / view | mean | 1.84 | 1.80 | 0.04 [-0.03–0.05] ±0.09 | 2.17% |
| trail-rescue-scenarios: commit | i1 wasm abi.dispatch_view / abi.exec | abi.dispatch_view / abi.exec | median | 71.0 | 68.9 | 1.50 [-2.20–2.10] ±8.55 | 2.18% |
| trail-rescue-scenarios: commit | i1 wasm abi.dispatch_view / abi.exec | abi.dispatch_view / abi.exec | mean | 74.6 | 72.3 | 1.77 [-1.78–2.30] ±7.53 | 2.45% |
| trail-rescue-scenarios: commit | i1 wasm abi.dispatch_outcome / abi.exec | abi.dispatch_outcome / abi.exec | median | 140.9 | 144.8 | 1.90 [-9.70–3.00] ±9.25 | 1.31% |
| trail-rescue-scenarios: commit | i1 wasm abi.dispatch_outcome / abi.exec | abi.dispatch_outcome / abi.exec | mean | 144.2 | 149.4 | 0.44 [-7.72–2.97] ±8.53 | 0.30% |
| trail-rescue-scenarios: commit | i1 wasm raw.dispatch_view / raw.dispatch_view+parse | raw.dispatch_view / raw.dispatch_view+parse | median | 104.6 | 104.7 | 0.20 [-1.70–1.70] ±7.75 | 0.19% |
| trail-rescue-scenarios: commit | i1 wasm raw.dispatch_view / raw.dispatch_view+parse | raw.dispatch_view / raw.dispatch_view+parse | mean | 108.4 | 109.4 | -0.28 [-1.05–0.84] ±7.15 | -0.25% |
| trail-rescue-scenarios: commit | i2 native probe.dispatch_view / total | web.dispatch_view / web.dispatch_view | median | 61.6 | 59.5 | 0.70 [-1.30–5.10] ±5.30 | 1.18% |
| trail-rescue-scenarios: commit | i2 native probe.dispatch_view / total | web.dispatch_view / web.dispatch_view | mean | 64.3 | 62.6 | 0.52 [-0.13–5.51] ±5.25 | 0.83% |
| trail-rescue-scenarios: commit | i2 native probe.dispatch_outcome / total | web.dispatch_outcome / web.dispatch_outcome | median | 126.9 | 121.9 | 5.40 [1.40–10.4] ±4.70 | 4.43% |
| trail-rescue-scenarios: commit | i2 native probe.dispatch_outcome / total | web.dispatch_outcome / web.dispatch_outcome | mean | 129.3 | 124.6 | 4.57 [1.39–10.8] ±4.80 | 3.67% |
| trail-rescue-scenarios: commit | i2 wasm probe.dispatch_view / total | raw.dispatch_view / raw.dispatch_view+parse | median | 111.8 | 104.7 | 5.50 [3.00–15.0] ±6.80 | 5.25% |
| trail-rescue-scenarios: commit | i2 wasm probe.dispatch_view / total | raw.dispatch_view / raw.dispatch_view+parse | mean | 115.6 | 109.4 | 8.60 [2.51–13.8] ±6.98 | 7.86% |
| trail-rescue-scenarios: commit | i2 wasm probe.dispatch_outcome / total | raw.dispatch_outcome / raw.dispatch_outcome+parse | median | 247.2 | 230.9 | 19.8 [12.3–25.1] ±12.7 | 8.58% |
| trail-rescue-scenarios: commit | i2 wasm probe.dispatch_outcome / total | raw.dispatch_outcome / raw.dispatch_outcome+parse | mean | 258.5 | 238.9 | 23.2 [17.7–27.0] ±11.3 | 9.70% |
| trail-rescue-scenarios: reopen | i1 native web.dispatch_view / web.dispatch_view | web.dispatch_view / web.dispatch_view | median | 131.4 | 132.8 | -1.70 [-6.50–2.60] ±10.3 | -1.28% |
| trail-rescue-scenarios: reopen | i1 native web.dispatch_view / web.dispatch_view | web.dispatch_view / web.dispatch_view | mean | 126.3 | 128.4 | -1.41 [-9.16–1.73] ±11.9 | -1.10% |
| trail-rescue-scenarios: reopen | i1 native web.dispatch_outcome / web.dispatch_outcome | web.dispatch_outcome / web.dispatch_outcome | median | 199.7 | 198.9 | 0.30 [-1.90–2.40] ±10.0 | 0.15% |
| trail-rescue-scenarios: reopen | i1 native web.dispatch_outcome / web.dispatch_outcome | web.dispatch_outcome / web.dispatch_outcome | mean | 192.9 | 192.8 | 0.06 [-0.07–3.57] ±11.1 | 0.03% |
| trail-rescue-scenarios: reopen | i1 native apply / apply | apply / apply | median | 118.4 | 121.3 | -1.40 [-4.30–0.70] ±7.35 | -1.15% |
| trail-rescue-scenarios: reopen | i1 native apply / apply | apply / apply | mean | 113.5 | 116.3 | 0.22 [-7.59–0.86] ±9.66 | 0.19% |
| trail-rescue-scenarios: reopen | i1 native read / view | read / view | median | 1.90 | 1.80 | 0.00 [0.00–0.10] ±0.10 | 0.00% |
| trail-rescue-scenarios: reopen | i1 native read / view | read / view | mean | 1.96 | 1.95 | 0.02 [-0.12–0.05] ±0.10 | 1.08% |
| trail-rescue-scenarios: reopen | i1 wasm abi.dispatch_view / abi.exec | abi.dispatch_view / abi.exec | median | 146.3 | 140.9 | 4.50 [-1.50–5.40] ±12.5 | 3.19% |
| trail-rescue-scenarios: reopen | i1 wasm abi.dispatch_view / abi.exec | abi.dispatch_view / abi.exec | mean | 144.5 | 139.3 | 5.18 [-3.10–5.24] ±15.9 | 3.72% |
| trail-rescue-scenarios: reopen | i1 wasm abi.dispatch_outcome / abi.exec | abi.dispatch_outcome / abi.exec | median | 213.8 | 222.1 | 2.10 [-12.0–4.00] ±13.2 | 0.95% |
| trail-rescue-scenarios: reopen | i1 wasm abi.dispatch_outcome / abi.exec | abi.dispatch_outcome / abi.exec | mean | 215.5 | 222.7 | 0.93 [-12.4–4.10] ±13.5 | 0.42% |
| trail-rescue-scenarios: reopen | i1 wasm raw.dispatch_view / raw.dispatch_view+parse | raw.dispatch_view / raw.dispatch_view+parse | median | 180.1 | 179.9 | 1.00 [0.20–3.80] ±10.6 | 0.56% |
| trail-rescue-scenarios: reopen | i1 wasm raw.dispatch_view / raw.dispatch_view+parse | raw.dispatch_view / raw.dispatch_view+parse | mean | 178.6 | 177.9 | 0.72 [-1.97–3.21] ±12.8 | 0.40% |
| trail-rescue-scenarios: reopen | i2 native probe.dispatch_view / total | web.dispatch_view / web.dispatch_view | median | 133.9 | 132.8 | 0.80 [0.50–6.70] ±7.75 | 0.60% |
| trail-rescue-scenarios: reopen | i2 native probe.dispatch_view / total | web.dispatch_view / web.dispatch_view | mean | 131.0 | 128.4 | 0.22 [0.05–9.42] ±10.3 | 0.17% |
| trail-rescue-scenarios: reopen | i2 native probe.dispatch_outcome / total | web.dispatch_outcome / web.dispatch_outcome | median | 202.6 | 198.9 | 3.70 [-0.40–12.1] ±6.25 | 1.86% |
| trail-rescue-scenarios: reopen | i2 native probe.dispatch_outcome / total | web.dispatch_outcome / web.dispatch_outcome | mean | 200.8 | 192.8 | 7.76 [1.39–18.0] ±8.39 | 4.02% |
| trail-rescue-scenarios: reopen | i2 wasm probe.dispatch_view / total | raw.dispatch_view / raw.dispatch_view+parse | median | 183.4 | 179.9 | 3.50 [3.30–19.5] ±8.10 | 1.95% |
| trail-rescue-scenarios: reopen | i2 wasm probe.dispatch_view / total | raw.dispatch_view / raw.dispatch_view+parse | mean | 185.3 | 177.9 | 2.83 [1.31–20.6] ±11.9 | 1.59% |
| trail-rescue-scenarios: reopen | i2 wasm probe.dispatch_outcome / total | raw.dispatch_outcome / raw.dispatch_outcome+parse | median | 326.4 | 306.0 | 20.4 [18.0–21.4] ±11.7 | 6.67% |
| trail-rescue-scenarios: reopen | i2 wasm probe.dispatch_outcome / total | raw.dispatch_outcome / raw.dispatch_outcome+parse | mean | 328.2 | 309.0 | 19.1 [15.5–30.1] ±14.3 | 6.17% |
| ledger-session: evidence | i1 native web.dispatch_view / web.dispatch_view | web.dispatch_view / web.dispatch_view | median | 29.0 | 28.4 | 0.40 [-0.30–0.60] ±2.35 | 1.41% |
| ledger-session: evidence | i1 native web.dispatch_view / web.dispatch_view | web.dispatch_view / web.dispatch_view | mean | 28.7 | 28.3 | -0.21 [-0.34–0.39] ±2.61 | -0.76% |
| ledger-session: evidence | i1 native web.dispatch_outcome / web.dispatch_outcome | web.dispatch_outcome / web.dispatch_outcome | median | 65.9 | 65.8 | 0.70 [0.10–1.40] ±3.55 | 1.06% |
| ledger-session: evidence | i1 native web.dispatch_outcome / web.dispatch_outcome | web.dispatch_outcome / web.dispatch_outcome | mean | 67.9 | 68.0 | 2.28 [-0.63–2.98] ±4.73 | 3.35% |
| ledger-session: evidence | i1 native apply / apply | apply / apply | median | 20.5 | 20.6 | -0.10 [-3.40–0.20] ±2.30 | -0.49% |
| ledger-session: evidence | i1 native apply / apply | apply / apply | mean | 19.8 | 19.8 | -0.01 [-3.49–0.31] ±2.48 | -0.05% |
| ledger-session: evidence | i1 native read / view | read / view | median | 3.30 | 3.30 | 0.00 [0.00–0.10] ±0.15 | 0.00% |
| ledger-session: evidence | i1 native read / view | read / view | mean | 3.36 | 3.36 | 0.00 [-0.01–0.04] ±0.14 | 0.03% |
| ledger-session: evidence | i1 wasm abi.dispatch_view / abi.exec | abi.dispatch_view / abi.exec | median | 33.6 | 33.5 | -0.10 [-0.20–0.10] ±1.05 | -0.30% |
| ledger-session: evidence | i1 wasm abi.dispatch_view / abi.exec | abi.dispatch_view / abi.exec | mean | 33.8 | 33.5 | 0.08 [-0.47–0.27] ±2.02 | 0.24% |
| ledger-session: evidence | i1 wasm abi.dispatch_outcome / abi.exec | abi.dispatch_outcome / abi.exec | median | 68.8 | 69.4 | -0.60 [-3.40–-0.10] ±2.95 | -0.86% |
| ledger-session: evidence | i1 wasm abi.dispatch_outcome / abi.exec | abi.dispatch_outcome / abi.exec | mean | 70.5 | 71.3 | -0.70 [-3.85–-0.51] ±3.30 | -0.98% |
| ledger-session: evidence | i1 wasm raw.dispatch_view / raw.dispatch_view+parse | raw.dispatch_view / raw.dispatch_view+parse | median | 42.8 | 42.8 | 0.50 [0.00–1.40] ±2.15 | 1.17% |
| ledger-session: evidence | i1 wasm raw.dispatch_view / raw.dispatch_view+parse | raw.dispatch_view / raw.dispatch_view+parse | mean | 43.8 | 44.0 | 0.80 [-0.24–1.30] ±2.69 | 1.83% |
| ledger-session: evidence | i2 native probe.dispatch_view / total | web.dispatch_view / web.dispatch_view | median | 29.5 | 28.4 | 0.70 [-0.60–1.10] ±1.35 | 2.46% |
| ledger-session: evidence | i2 native probe.dispatch_view / total | web.dispatch_view / web.dispatch_view | mean | 29.4 | 28.3 | 1.05 [-0.94–1.21] ±1.59 | 3.72% |
| ledger-session: evidence | i2 native probe.dispatch_outcome / total | web.dispatch_outcome / web.dispatch_outcome | median | 66.3 | 65.8 | 0.60 [0.50–1.10] ±2.95 | 0.91% |
| ledger-session: evidence | i2 native probe.dispatch_outcome / total | web.dispatch_outcome / web.dispatch_outcome | mean | 68.8 | 68.0 | 0.79 [0.71–3.12] ±2.90 | 1.17% |
| ledger-session: evidence | i2 wasm probe.dispatch_view / total | raw.dispatch_view / raw.dispatch_view+parse | median | 44.9 | 42.8 | 2.10 [0.20–5.00] ±2.40 | 4.91% |
| ledger-session: evidence | i2 wasm probe.dispatch_view / total | raw.dispatch_view / raw.dispatch_view+parse | mean | 46.8 | 44.0 | 2.76 [0.05–4.80] ±2.37 | 6.27% |
| ledger-session: evidence | i2 wasm probe.dispatch_outcome / total | raw.dispatch_outcome / raw.dispatch_outcome+parse | median | 134.3 | 130.8 | 7.90 [-6.70–8.80] ±9.95 | 6.04% |
| ledger-session: evidence | i2 wasm probe.dispatch_outcome / total | raw.dispatch_outcome / raw.dispatch_outcome+parse | mean | 140.0 | 135.1 | 9.00 [-6.69–10.4] ±10.6 | 6.66% |

The cost of one mark (batches of 100):

| Engine | Operation | µs per call: median [runs] · p95 |
| --- | --- | --- |
| native | probe.mark | 0.04 [0.04–0.04] · p95 0.04 |
| native | probe.now | 0.04 [0.04–0.04] · p95 0.04 |
| wasm | probe.mark | 0.05 [0.05–0.06] · p95 0.06 |
| wasm | js.performance_now | 0.04 [0.04–0.06] · p95 0.07 |

## Where the time goes inside one call (i2 marks: proportions only)

Mean time per event of each disjoint region between consecutive marks, as a share of the instrumented call's mean total. The regions of a call add up exactly to its total, so these shares add to 100%. Means, not medians, because only means add; the marks read a 100 ns clock and cost about the mark cost above each, so the absolute values are not production costs.

### glowcap-replay, native dispatch_view

| Region | idle tick | state-changing tick |
| --- | --- | --- |
| bridge in (JS call, argument copy) | 0.25% (0.06 µs) | 0.22% (0.06 µs) |
| payload parse + name resolution | 1.70% (0.40 µs) | 1.52% (0.41 µs) |
| transaction setup + session copy | 1.94% (0.46 µs) | 1.69% (0.45 µs) |
| clock and due qualifications | 0.35% (0.08 µs) | 0.38% (0.10 µs) |
| rules | 51.3% (12.1 µs) | 52.2% (14.1 µs) |
| change detection | 1.60% (0.38 µs) | 1.30% (0.35 µs) |
| binding evaluation | 5.55% (1.31 µs) | 13.3% (3.57 µs) |
| commit (old session dropped) or rollback | 5.21% (1.23 µs) | 4.91% (1.32 µs) |
| view: NodeId→name map | 1.86% (0.44 µs) | 1.37% (0.37 µs) |
| view: symbol sort | 2.61% (0.62 µs) | 1.72% (0.46 µs) |
| view: commitment records | 2.19% (0.52 µs) | 1.66% (0.45 µs) |
| view: relation records | 5.87% (1.39 µs) | 4.17% (1.12 µs) |
| view: assemble + drop map | 0.26% (0.06 µs) | 0.23% (0.06 µs) |
| serialize (serde_json) | 16.4% (3.90 µs) | 13.2% (3.56 µs) |
| drop the built view/outcome | 2.65% (0.64 µs) | 1.97% (0.53 µs) |
| bridge out (return, decode, free) | 0.19% (0.05 µs) | 0.17% (0.04 µs) |
| **instrumented total** | mean 23.7 µs | mean 27.0 µs |

### glowcap-replay, wasm dispatch_view

| Region | idle tick | state-changing tick |
| --- | --- | --- |
| bridge in (JS call, argument copy) | 0.46% (0.24 µs) | 0.92% (0.51 µs) |
| payload parse + name resolution | 0.94% (0.50 µs) | 0.99% (0.56 µs) |
| transaction setup + session copy | 0.60% (0.31 µs) | 0.61% (0.35 µs) |
| clock and due qualifications | 0.16% (0.09 µs) | 0.21% (0.12 µs) |
| rules | 24.3% (12.8 µs) | 28.4% (15.8 µs) |
| change detection | 1.28% (0.68 µs) | 1.08% (0.61 µs) |
| binding evaluation | 3.40% (1.79 µs) | 8.26% (4.60 µs) |
| commit (old session dropped) or rollback | 1.67% (0.86 µs) | 1.83% (1.03 µs) |
| view: NodeId→name map | 1.34% (0.70 µs) | 1.07% (0.61 µs) |
| view: symbol sort | 1.35% (0.70 µs) | 0.99% (0.56 µs) |
| view: commitment records | 1.26% (0.65 µs) | 1.07% (0.60 µs) |
| view: relation records | 2.21% (1.16 µs) | 1.77% (0.99 µs) |
| view: assemble + drop map | 0.15% (0.08 µs) | 0.13% (0.07 µs) |
| serialize (serde_json) | 12.2% (6.41 µs) | 11.0% (6.25 µs) |
| drop the built view/outcome | 1.03% (0.55 µs) | 0.91% (0.51 µs) |
| bridge out (return, decode, free) | 9.68% (5.10 µs) | 5.61% (3.15 µs) |
| JS JSON.parse | 38.2% (20.1 µs) | 35.1% (19.6 µs) |
| **instrumented total** | mean 52.6 µs | mean 56.2 µs |

### glowcap-replay, native dispatch_outcome

| Region | idle tick | state-changing tick |
| --- | --- | --- |
| bridge in (JS call, argument copy) | 0.08% (0.06 µs) | 0.08% (0.06 µs) |
| payload parse + name resolution | 0.69% (0.51 µs) | 0.71% (0.52 µs) |
| transaction setup + session copy | 0.65% (0.50 µs) | 0.66% (0.49 µs) |
| clock and due qualifications | 0.19% (0.14 µs) | 0.22% (0.16 µs) |
| rules | 17.8% (13.1 µs) | 20.7% (15.1 µs) |
| change detection | 0.66% (0.49 µs) | 0.62% (0.45 µs) |
| binding evaluation | 1.85% (1.37 µs) | 5.99% (4.38 µs) |
| commit (old session dropped) or rollback | 1.79% (1.33 µs) | 2.07% (1.55 µs) |
| snapshot: name map + symbol sort | 2.05% (1.58 µs) | 1.74% (1.29 µs) |
| snapshot: symbols + commitment records | 5.55% (4.12 µs) | 4.87% (3.56 µs) |
| snapshot: bindings, values, records, static world (clones) | 27.5% (20.7 µs) | 25.3% (18.5 µs) |
| snapshot: relation records + assemble | 2.45% (1.82 µs) | 2.00% (1.47 µs) |
| outcome wrapper | 0.19% (0.14 µs) | 0.19% (0.14 µs) |
| serialize (serde_json) | 22.4% (16.6 µs) | 20.6% (15.0 µs) |
| drop the built view/outcome | 15.9% (11.8 µs) | 14.3% (10.4 µs) |
| bridge out (return, decode, free) | 0.07% (0.06 µs) | 0.07% (0.06 µs) |
| **instrumented total** | mean 74.4 µs | mean 73.0 µs |

### glowcap-replay, wasm dispatch_outcome

| Region | idle tick | state-changing tick |
| --- | --- | --- |
| bridge in (JS call, argument copy) | 0.19% (0.32 µs) | 0.40% (0.66 µs) |
| payload parse + name resolution | 0.36% (0.61 µs) | 0.44% (0.70 µs) |
| transaction setup + session copy | 0.23% (0.38 µs) | 0.25% (0.39 µs) |
| clock and due qualifications | 0.06% (0.10 µs) | 0.08% (0.13 µs) |
| rules | 8.40% (14.3 µs) | 10.5% (17.1 µs) |
| change detection | 0.47% (0.79 µs) | 0.43% (0.70 µs) |
| binding evaluation | 1.15% (1.96 µs) | 3.05% (5.05 µs) |
| commit (old session dropped) or rollback | 0.74% (1.26 µs) | 0.82% (1.31 µs) |
| snapshot: name map + symbol sort | 1.00% (1.70 µs) | 0.83% (1.36 µs) |
| snapshot: symbols + commitment records | 2.03% (3.46 µs) | 1.89% (3.06 µs) |
| snapshot: bindings, values, records, static world (clones) | 9.32% (15.8 µs) | 9.65% (15.4 µs) |
| snapshot: relation records + assemble | 0.87% (1.51 µs) | 0.80% (1.31 µs) |
| outcome wrapper | 0.06% (0.10 µs) | 0.06% (0.10 µs) |
| serialize (serde_json) | 14.2% (24.2 µs) | 13.9% (23.1 µs) |
| drop the built view/outcome | 8.27% (13.9 µs) | 7.68% (12.4 µs) |
| bridge out (return, decode, free) | 7.82% (13.0 µs) | 5.81% (9.56 µs) |
| JS JSON.parse | 44.9% (76.3 µs) | 43.4% (71.9 µs) |
| **instrumented total** | mean 169.9 µs | mean 163.9 µs |

### trail-rescue-scenarios, native dispatch_view

| Region | evidence | commit | reopen | refused |
| --- | --- | --- | --- | --- |
| bridge in (JS call, argument copy) | 0.08% (0.09 µs) | 0.09% (0.06 µs) | 0.05% (0.06 µs) | 1.21% (0.07 µs) |
| payload parse + name resolution | 2.46% (3.00 µs) | 2.11% (1.38 µs) | 0.77% (1.00 µs) | 21.6% (1.19 µs) |
| transaction setup + session copy | 0.87% (1.06 µs) | 1.31% (0.86 µs) | 0.55% (0.72 µs) | 11.9% (0.66 µs) |
| clock and due qualifications | 0.14% (0.17 µs) | 0.34% (0.22 µs) | 3.70% (4.79 µs) | 2.55% (0.14 µs) |
| rules | 52.6% (63.8 µs) | 33.7% (21.6 µs) | 45.5% (59.3 µs) | 4.49% (0.24 µs) |
| change detection | 1.28% (1.54 µs) | 0.41% (0.26 µs) | 0.91% (1.17 µs) | 0.00% (0.00 µs) |
| binding evaluation | 31.2% (37.7 µs) | 40.3% (25.9 µs) | 36.7% (48.5 µs) | 0.00% (0.00 µs) |
| commit (old session dropped) or rollback | 2.23% (2.70 µs) | 4.33% (2.76 µs) | 2.63% (3.44 µs) | 52.6% (2.87 µs) |
| view: NodeId→name map | 0.29% (0.36 µs) | 0.47% (0.31 µs) | 0.23% (0.32 µs) | 0.00% (0.00 µs) |
| view: symbol sort | 0.59% (0.72 µs) | 1.04% (0.67 µs) | 0.42% (0.56 µs) | 0.00% (0.00 µs) |
| view: commitment records | 0.24% (0.29 µs) | 1.32% (0.85 µs) | 0.58% (0.75 µs) | 0.00% (0.00 µs) |
| view: relation records | 0.63% (0.76 µs) | 1.50% (0.97 µs) | 0.85% (1.12 µs) | 0.00% (0.00 µs) |
| view: assemble + drop map | 0.05% (0.07 µs) | 0.09% (0.06 µs) | 0.05% (0.07 µs) | 0.00% (0.00 µs) |
| serialize (serde_json) | 7.02% (8.56 µs) | 12.0% (7.72 µs) | 6.43% (8.42 µs) | 0.00% (0.00 µs) |
| drop the built view/outcome | 0.30% (0.37 µs) | 0.97% (0.62 µs) | 0.55% (0.72 µs) | 0.00% (0.00 µs) |
| bridge out (return, decode, free) | 0.04% (0.05 µs) | 0.07% (0.05 µs) | 0.04% (0.05 µs) | 5.66% (0.31 µs) |
| **instrumented total** | mean 121.3 µs | mean 64.3 µs | mean 131.0 µs | mean 5.47 µs |

### trail-rescue-scenarios, wasm dispatch_view

| Region | evidence | commit | reopen | refused |
| --- | --- | --- | --- | --- |
| bridge in (JS call, argument copy) | 0.57% (0.98 µs) | 0.58% (0.67 µs) | 0.30% (0.56 µs) | 4.76% (0.61 µs) |
| payload parse + name resolution | 2.28% (3.93 µs) | 1.56% (1.78 µs) | 0.79% (1.46 µs) | 12.7% (1.67 µs) |
| transaction setup + session copy | 0.60% (1.02 µs) | 0.61% (0.71 µs) | 0.36% (0.67 µs) | 4.45% (0.56 µs) |
| clock and due qualifications | 0.11% (0.19 µs) | 0.19% (0.23 µs) | 2.98% (5.53 µs) | 1.32% (0.17 µs) |
| rules | 41.3% (70.8 µs) | 22.2% (25.4 µs) | 36.5% (66.9 µs) | 1.78% (0.23 µs) |
| change detection | 1.08% (1.86 µs) | 0.34% (0.39 µs) | 0.87% (1.62 µs) | 0.00% (0.00 µs) |
| binding evaluation | 23.1% (39.6 µs) | 26.6% (30.7 µs) | 28.0% (51.9 µs) | 0.00% (0.00 µs) |
| commit (old session dropped) or rollback | 1.97% (3.32 µs) | 3.09% (3.49 µs) | 2.64% (4.83 µs) | 25.4% (3.35 µs) |
| view: NodeId→name map | 0.35% (0.60 µs) | 0.48% (0.54 µs) | 0.30% (0.55 µs) | 0.00% (0.00 µs) |
| view: symbol sort | 0.54% (0.94 µs) | 0.76% (0.90 µs) | 0.43% (0.80 µs) | 0.00% (0.00 µs) |
| view: commitment records | 0.24% (0.41 µs) | 0.87% (0.99 µs) | 0.46% (0.85 µs) | 0.00% (0.00 µs) |
| view: relation records | 0.37% (0.63 µs) | 0.79% (0.92 µs) | 0.59% (1.09 µs) | 0.00% (0.00 µs) |
| view: assemble + drop map | 0.06% (0.11 µs) | 0.08% (0.09 µs) | 0.05% (0.10 µs) | 0.00% (0.00 µs) |
| serialize (serde_json) | 6.94% (12.0 µs) | 10.5% (12.2 µs) | 6.61% (12.3 µs) | 0.00% (0.00 µs) |
| drop the built view/outcome | 0.25% (0.42 µs) | 0.74% (0.86 µs) | 0.54% (1.00 µs) | 0.00% (0.00 µs) |
| bridge out (return, decode, free) | 1.76% (2.93 µs) | 1.72% (1.92 µs) | 1.12% (2.07 µs) | 46.7% (5.93 µs) |
| JS JSON.parse | 18.4% (31.4 µs) | 28.3% (32.7 µs) | 17.5% (32.8 µs) | 0.00% (0.00 µs) |
| **instrumented total** | mean 171.4 µs | mean 115.6 µs | mean 185.3 µs | mean 13.2 µs |

### trail-rescue-scenarios, native dispatch_outcome

| Region | evidence | commit | reopen | refused |
| --- | --- | --- | --- | --- |
| bridge in (JS call, argument copy) | 0.06% (0.11 µs) | 0.05% (0.06 µs) | 0.03% (0.07 µs) | 1.18% (0.07 µs) |
| payload parse + name resolution | 1.72% (3.26 µs) | 1.22% (1.58 µs) | 0.61% (1.23 µs) | 20.8% (1.32 µs) |
| transaction setup + session copy | 0.61% (1.14 µs) | 0.68% (0.88 µs) | 0.39% (0.77 µs) | 10.9% (0.69 µs) |
| clock and due qualifications | 0.09% (0.17 µs) | 0.15% (0.20 µs) | 2.60% (5.22 µs) | 2.23% (0.14 µs) |
| rules | 35.1% (66.7 µs) | 17.9% (23.2 µs) | 31.2% (62.6 µs) | 3.85% (0.24 µs) |
| change detection | 0.86% (1.63 µs) | 0.22% (0.28 µs) | 0.63% (1.27 µs) | 0.00% (0.00 µs) |
| binding evaluation | 20.7% (39.2 µs) | 21.4% (27.7 µs) | 25.4% (51.0 µs) | 0.00% (0.00 µs) |
| commit (old session dropped) or rollback | 1.51% (2.87 µs) | 2.28% (2.95 µs) | 1.86% (3.73 µs) | 49.9% (3.20 µs) |
| snapshot: name map + symbol sort | 0.70% (1.33 µs) | 0.97% (1.26 µs) | 0.55% (1.10 µs) | 0.00% (0.00 µs) |
| snapshot: symbols + commitment records | 1.55% (2.90 µs) | 2.43% (3.14 µs) | 1.56% (3.13 µs) | 0.00% (0.00 µs) |
| snapshot: bindings, values, records, static world (clones) | 16.0% (30.5 µs) | 22.8% (29.5 µs) | 15.2% (30.6 µs) | 0.00% (0.00 µs) |
| snapshot: relation records + assemble | 0.54% (1.01 µs) | 0.98% (1.24 µs) | 0.72% (1.41 µs) | 0.00% (0.00 µs) |
| outcome wrapper | 0.13% (0.24 µs) | 0.12% (0.16 µs) | 0.08% (0.16 µs) | 3.55% (0.23 µs) |
| serialize (serde_json) | 13.1% (24.9 µs) | 17.6% (22.8 µs) | 11.7% (23.4 µs) | 5.75% (0.36 µs) |
| drop the built view/outcome | 7.20% (13.7 µs) | 10.9% (14.2 µs) | 7.44% (14.9 µs) | 0.97% (0.06 µs) |
| bridge out (return, decode, free) | 0.03% (0.05 µs) | 0.05% (0.06 µs) | 0.03% (0.06 µs) | 0.74% (0.05 µs) |
| **instrumented total** | mean 189.8 µs | mean 129.3 µs | mean 200.8 µs | mean 6.37 µs |

### trail-rescue-scenarios, wasm dispatch_outcome

| Region | evidence | commit | reopen | refused |
| --- | --- | --- | --- | --- |
| bridge in (JS call, argument copy) | 0.35% (1.13 µs) | 0.34% (0.89 µs) | 0.21% (0.71 µs) | 6.68% (0.70 µs) |
| payload parse + name resolution | 1.35% (4.34 µs) | 0.87% (2.21 µs) | 0.54% (1.76 µs) | 18.7% (1.90 µs) |
| transaction setup + session copy | 0.34% (1.09 µs) | 0.30% (0.76 µs) | 0.23% (0.74 µs) | 6.19% (0.65 µs) |
| clock and due qualifications | 0.05% (0.17 µs) | 0.08% (0.21 µs) | 1.67% (5.49 µs) | 1.37% (0.14 µs) |
| rules | 22.7% (70.8 µs) | 10.2% (26.2 µs) | 20.6% (67.1 µs) | 2.45% (0.25 µs) |
| change detection | 0.59% (1.87 µs) | 0.16% (0.42 µs) | 0.49% (1.62 µs) | 0.00% (0.00 µs) |
| binding evaluation | 12.7% (39.7 µs) | 12.1% (30.8 µs) | 16.2% (52.5 µs) | 0.00% (0.00 µs) |
| commit (old session dropped) or rollback | 1.10% (3.48 µs) | 1.45% (3.74 µs) | 1.49% (4.89 µs) | 34.5% (3.41 µs) |
| snapshot: name map + symbol sort | 0.58% (1.86 µs) | 0.65% (1.68 µs) | 0.48% (1.56 µs) | 0.00% (0.00 µs) |
| snapshot: symbols + commitment records | 0.96% (3.06 µs) | 1.31% (3.43 µs) | 1.00% (3.24 µs) | 0.00% (0.00 µs) |
| snapshot: bindings, values, records, static world (clones) | 9.05% (28.4 µs) | 10.4% (26.8 µs) | 8.28% (27.2 µs) | 0.00% (0.00 µs) |
| snapshot: relation records + assemble | 0.23% (0.74 µs) | 0.34% (0.91 µs) | 0.30% (1.03 µs) | 0.00% (0.00 µs) |
| outcome wrapper | 0.06% (0.19 µs) | 0.06% (0.16 µs) | 0.05% (0.16 µs) | 2.67% (0.28 µs) |
| serialize (serde_json) | 11.1% (35.0 µs) | 12.9% (34.1 µs) | 10.2% (34.3 µs) | 5.48% (0.57 µs) |
| drop the built view/outcome | 7.10% (22.3 µs) | 8.97% (23.2 µs) | 7.37% (24.2 µs) | 0.75% (0.08 µs) |
| bridge out (return, decode, free) | 2.61% (8.19 µs) | 4.98% (13.2 µs) | 3.09% (10.5 µs) | 7.36% (0.76 µs) |
| JS JSON.parse | 29.2% (91.6 µs) | 34.6% (89.4 µs) | 27.7% (90.7 µs) | 14.3% (1.49 µs) |
| **instrumented total** | mean 313.8 µs | mean 258.5 µs | mean 328.2 µs | mean 10.3 µs |

### ledger-session, native dispatch_view

| Region | evidence | refused |
| --- | --- | --- |
| bridge in (JS call, argument copy) | 0.18% (0.05 µs) | 0.87% (0.05 µs) |
| payload parse + name resolution | 4.18% (1.23 µs) | 13.6% (0.79 µs) |
| transaction setup + session copy | 2.46% (0.72 µs) | 10.5% (0.63 µs) |
| clock and due qualifications | 0.57% (0.17 µs) | 3.00% (0.17 µs) |
| rules | 41.2% (12.1 µs) | 3.19% (0.19 µs) |
| change detection | 0.72% (0.21 µs) | 0.00% (0.00 µs) |
| binding evaluation | 16.5% (4.89 µs) | 0.00% (0.00 µs) |
| commit (old session dropped) or rollback | 9.24% (2.73 µs) | 65.3% (3.90 µs) |
| view: NodeId→name map | 2.00% (0.59 µs) | 0.00% (0.00 µs) |
| view: symbol sort | 7.23% (2.12 µs) | 0.00% (0.00 µs) |
| view: commitment records | 1.89% (0.55 µs) | 0.00% (0.00 µs) |
| view: relation records | 3.95% (1.16 µs) | 0.00% (0.00 µs) |
| view: assemble + drop map | 0.21% (0.06 µs) | 0.00% (0.00 µs) |
| serialize (serde_json) | 7.99% (2.34 µs) | 0.00% (0.00 µs) |
| drop the built view/outcome | 1.42% (0.42 µs) | 0.00% (0.00 µs) |
| bridge out (return, decode, free) | 0.15% (0.04 µs) | 3.40% (0.20 µs) |
| **instrumented total** | mean 29.4 µs | mean 5.97 µs |

### ledger-session, wasm dispatch_view

| Region | evidence | refused |
| --- | --- | --- |
| bridge in (JS call, argument copy) | 0.93% (0.43 µs) | 3.40% (0.35 µs) |
| payload parse + name resolution | 4.10% (1.94 µs) | 11.2% (1.19 µs) |
| transaction setup + session copy | 1.38% (0.65 µs) | 5.21% (0.54 µs) |
| clock and due qualifications | 0.38% (0.18 µs) | 1.66% (0.17 µs) |
| rules | 28.5% (13.3 µs) | 2.38% (0.24 µs) |
| change detection | 0.69% (0.32 µs) | 0.00% (0.00 µs) |
| binding evaluation | 11.6% (5.28 µs) | 0.00% (0.00 µs) |
| commit (old session dropped) or rollback | 8.77% (4.11 µs) | 41.7% (4.42 µs) |
| view: NodeId→name map | 1.97% (0.93 µs) | 0.00% (0.00 µs) |
| view: symbol sort | 4.95% (2.32 µs) | 0.00% (0.00 µs) |
| view: commitment records | 1.64% (0.77 µs) | 0.00% (0.00 µs) |
| view: relation records | 2.11% (0.98 µs) | 0.00% (0.00 µs) |
| view: assemble + drop map | 0.19% (0.09 µs) | 0.00% (0.00 µs) |
| serialize (serde_json) | 8.51% (3.96 µs) | 0.00% (0.00 µs) |
| drop the built view/outcome | 1.52% (0.71 µs) | 0.00% (0.00 µs) |
| bridge out (return, decode, free) | 2.44% (1.13 µs) | 34.5% (3.64 µs) |
| JS JSON.parse | 19.3% (9.10 µs) | 0.00% (0.00 µs) |
| **instrumented total** | mean 46.8 µs | mean 10.6 µs |

### ledger-session, native dispatch_outcome

| Region | evidence | refused |
| --- | --- | --- |
| bridge in (JS call, argument copy) | 0.08% (0.05 µs) | 0.70% (0.05 µs) |
| payload parse + name resolution | 2.28% (1.59 µs) | 16.3% (1.13 µs) |
| transaction setup + session copy | 1.09% (0.76 µs) | 9.65% (0.66 µs) |
| clock and due qualifications | 0.25% (0.17 µs) | 2.55% (0.17 µs) |
| rules | 18.4% (12.7 µs) | 3.71% (0.26 µs) |
| change detection | 0.33% (0.23 µs) | 0.00% (0.00 µs) |
| binding evaluation | 7.56% (5.23 µs) | 0.00% (0.00 µs) |
| commit (old session dropped) or rollback | 4.25% (2.92 µs) | 58.9% (4.09 µs) |
| snapshot: name map + symbol sort | 4.57% (3.21 µs) | 0.00% (0.00 µs) |
| snapshot: symbols + commitment records | 8.19% (5.77 µs) | 0.00% (0.00 µs) |
| snapshot: bindings, values, records, static world (clones) | 16.5% (11.4 µs) | 0.00% (0.00 µs) |
| snapshot: relation records + assemble | 2.20% (1.51 µs) | 0.00% (0.00 µs) |
| outcome wrapper | 0.26% (0.18 µs) | 2.05% (0.14 µs) |
| serialize (serde_json) | 22.4% (15.6 µs) | 4.83% (0.34 µs) |
| drop the built view/outcome | 11.2% (7.69 µs) | 0.88% (0.06 µs) |
| bridge out (return, decode, free) | 0.08% (0.05 µs) | 0.65% (0.04 µs) |
| **instrumented total** | mean 68.8 µs | mean 6.95 µs |

### ledger-session, wasm dispatch_outcome

| Region | evidence | refused |
| --- | --- | --- |
| bridge in (JS call, argument copy) | 0.39% (0.56 µs) | 4.59% (0.49 µs) |
| payload parse + name resolution | 1.63% (2.28 µs) | 14.7% (1.52 µs) |
| transaction setup + session copy | 0.50% (0.68 µs) | 5.71% (0.57 µs) |
| clock and due qualifications | 0.14% (0.20 µs) | 1.82% (0.19 µs) |
| rules | 9.47% (13.3 µs) | 2.61% (0.27 µs) |
| change detection | 0.24% (0.34 µs) | 0.00% (0.00 µs) |
| binding evaluation | 3.99% (5.59 µs) | 0.00% (0.00 µs) |
| commit (old session dropped) or rollback | 3.25% (4.52 µs) | 45.0% (4.67 µs) |
| snapshot: name map + symbol sort | 2.69% (3.76 µs) | 0.00% (0.00 µs) |
| snapshot: symbols + commitment records | 2.56% (3.65 µs) | 0.00% (0.00 µs) |
| snapshot: bindings, values, records, static world (clones) | 6.04% (8.46 µs) | 0.00% (0.00 µs) |
| snapshot: relation records + assemble | 0.66% (0.93 µs) | 0.00% (0.00 µs) |
| outcome wrapper | 0.09% (0.13 µs) | 2.10% (0.22 µs) |
| serialize (serde_json) | 16.1% (22.5 µs) | 4.94% (0.52 µs) |
| drop the built view/outcome | 7.92% (11.1 µs) | 0.93% (0.10 µs) |
| bridge out (return, decode, free) | 5.15% (7.21 µs) | 6.68% (0.71 µs) |
| JS JSON.parse | 39.2% (54.8 µs) | 10.4% (1.07 µs) |
| **instrumented total** | mean 140.0 µs | mean 10.4 µs |

## The three buckets inside one call (i2 marks: proportions only)

The region shares above, summed into language execution, view or snapshot building, and serialization + bridge + JS parse. Shares of the instrumented call's mean total, which these rows (without the "of which" lines) add up to exactly. Proportions only: the i2 totals carry the marks' cost.

### wasm dispatch_view

| Bucket: share (mean µs) | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | ledger evidence |
| --- | --- | --- | --- | --- | --- | --- |
| language execution | 32.4% (17.0) | 41.5% (23.3) | 70.4% (120.7) | 54.6% (62.7) | 72.2% (133.0) | 55.7% (26.1) |
|   of which rules | 24.3% (12.8) | 28.4% (15.8) | 41.3% (70.8) | 22.2% (25.4) | 36.5% (66.9) | 28.5% (13.3) |
|   of which binding evaluation | 3.40% (1.79) | 8.26% (4.60) | 23.1% (39.6) | 26.6% (30.7) | 28.0% (51.9) | 11.6% (5.28) |
|   of which session copy + old session dropped | 2.00% (1.04) | 2.16% (1.21) | 2.20% (3.69) | 3.45% (3.93) | 2.83% (5.21) | 9.50% (4.45) |
| view building | 6.29% (3.28) | 5.03% (2.87) | 1.57% (2.69) | 3.01% (3.45) | 1.82% (3.38) | 10.9% (5.13) |
| serialization (serde_json) + drop of what was serialized | 13.2% (6.92) | 11.9% (6.74) | 7.19% (12.4) | 11.3% (13.0) | 7.16% (13.3) | 10.0% (4.67) |
| bridge in and out (JS call, argument copy, return, decode, free) | 10.1% (5.34) | 6.48% (3.63) | 2.34% (3.91) | 2.32% (2.60) | 1.43% (2.63) | 3.37% (1.56) |
| JS JSON.parse | 38.2% (20.1) | 35.1% (19.6) | 18.4% (31.4) | 28.3% (32.7) | 17.5% (32.8) | 19.3% (9.10) |
| instrumented mean total, µs | 52.6 | 56.2 | 171.4 | 115.6 | 185.3 | 46.8 |

### wasm dispatch_outcome

| Bucket: share (mean µs) | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | ledger evidence |
| --- | --- | --- | --- | --- | --- | --- |
| language execution | 11.4% (19.4) | 15.5% (25.4) | 38.8% (121.6) | 25.2% (64.1) | 41.4% (134.1) | 19.2% (26.9) |
|   of which rules | 8.40% (14.3) | 10.5% (17.1) | 22.7% (70.8) | 10.2% (26.2) | 20.6% (67.1) | 9.47% (13.3) |
|   of which binding evaluation | 1.15% (1.96) | 3.05% (5.05) | 12.7% (39.7) | 12.1% (30.8) | 16.2% (52.5) | 3.99% (5.59) |
|   of which session copy + old session dropped | 0.85% (1.46) | 0.94% (1.50) | 1.24% (3.94) | 1.63% (4.20) | 1.62% (5.29) | 3.49% (4.87) |
| snapshot building (outcome) | 13.3% (22.5) | 13.2% (21.1) | 10.9% (34.2) | 12.7% (32.9) | 10.1% (33.2) | 12.1% (16.9) |
| serialization (serde_json) + drop of what was serialized | 22.5% (38.1) | 21.7% (35.5) | 18.2% (57.1) | 21.9% (56.9) | 17.6% (58.2) | 24.0% (33.6) |
| bridge in and out (JS call, argument copy, return, decode, free) | 8.01% (13.3) | 6.21% (10.2) | 2.99% (9.40) | 5.31% (14.1) | 3.30% (11.2) | 5.56% (7.79) |
| JS JSON.parse | 44.9% (76.3) | 43.4% (71.9) | 29.2% (91.6) | 34.6% (89.4) | 27.7% (90.7) | 39.2% (54.8) |
| instrumented mean total, µs | 169.9 | 163.9 | 313.8 | 258.5 | 328.2 | 140.0 |

### native dispatch_view

| Bucket: share (mean µs) | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | ledger evidence |
| --- | --- | --- | --- | --- | --- | --- |
| language execution | 67.7% (16.0) | 75.2% (20.3) | 90.8% (110.0) | 82.4% (53.0) | 90.8% (118.9) | 75.0% (22.0) |
|   of which rules | 51.3% (12.1) | 52.2% (14.1) | 52.6% (63.8) | 33.7% (21.6) | 45.5% (59.3) | 41.2% (12.1) |
|   of which binding evaluation | 5.55% (1.31) | 13.3% (3.57) | 31.2% (37.7) | 40.3% (25.9) | 36.7% (48.5) | 16.5% (4.89) |
|   of which session copy + old session dropped | 6.73% (1.59) | 6.24% (1.68) | 2.65% (3.21) | 5.28% (3.40) | 3.01% (3.94) | 10.9% (3.20) |
| view building | 12.8% (3.02) | 9.14% (2.46) | 1.81% (2.20) | 4.43% (2.86) | 2.14% (2.84) | 15.3% (4.49) |
| serialization (serde_json) + drop of what was serialized | 19.1% (4.53) | 15.2% (4.09) | 7.33% (8.92) | 13.0% (8.34) | 6.97% (9.14) | 9.43% (2.76) |
| bridge in and out (JS call, argument copy, return, decode, free) | 0.44% (0.10) | 0.40% (0.11) | 0.12% (0.14) | 0.16% (0.11) | 0.08% (0.10) | 0.32% (0.10) |
| instrumented mean total, µs | 23.7 | 27.0 | 121.3 | 64.3 | 131.0 | 29.4 |

### native dispatch_outcome

| Bucket: share (mean µs) | Glowcap idle tick | Glowcap state-changing tick | Trail Rescue evidence | Trail Rescue commit | Trail Rescue reopen | ledger evidence |
| --- | --- | --- | --- | --- | --- | --- |
| language execution | 23.6% (17.5) | 30.9% (22.6) | 60.6% (115.1) | 43.9% (56.8) | 62.7% (125.9) | 34.3% (23.6) |
|   of which rules | 17.8% (13.1) | 20.7% (15.1) | 35.1% (66.7) | 17.9% (23.2) | 31.2% (62.6) | 18.4% (12.7) |
|   of which binding evaluation | 1.85% (1.37) | 5.99% (4.38) | 20.7% (39.2) | 21.4% (27.7) | 25.4% (51.0) | 7.56% (5.23) |
|   of which session copy + old session dropped | 2.29% (1.71) | 2.56% (1.89) | 1.79% (3.39) | 2.74% (3.55) | 2.12% (4.24) | 4.94% (3.40) |
| snapshot building (outcome) | 37.7% (28.4) | 34.1% (24.9) | 18.9% (36.0) | 27.4% (35.4) | 18.1% (36.4) | 31.7% (22.2) |
| serialization (serde_json) + drop of what was serialized | 38.5% (28.5) | 34.9% (25.4) | 20.3% (38.6) | 28.6% (37.0) | 19.1% (38.4) | 33.7% (23.3) |
| bridge in and out (JS call, argument copy, return, decode, free) | 0.16% (0.12) | 0.16% (0.12) | 0.09% (0.16) | 0.10% (0.12) | 0.06% (0.12) | 0.15% (0.10) |
| instrumented mean total, µs | 74.4 | 73.0 | 189.8 | 129.3 | 200.8 | 68.8 |

