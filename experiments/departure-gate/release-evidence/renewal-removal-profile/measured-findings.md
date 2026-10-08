# Renewal occurrence removal: measured findings

Recommendation: defer renewal occurrence removal as the next broad latency priority for this measured population. The 3,000-cycle mutual release spends about 3 ms in the instrumented removal block—4.942% of successful apply and 5.516% of rejected apply at the median of four process values. Search and shifting deserve a bounded future algorithmic review if larger bulk releases matter, but these measurements do not support an assumed 10% dispatch improvement or any other speedup promise. If more latency investigation is authorized, identify the remaining departure costs before selecting an implementation. No optimization was made or authorized by this report.

This is a finite profiling result at `677ad1365398fe37bb24ff8fb6cfbae82d3ef5b5`, runtime tree `12547178224bdb4273bf51ae889c512b5cabba86`. The accepted semantic reference is `055e2af0` / runtime `aec511bf`, using its preserved `8a161dc`-built executable solely for semantic comparison. `run-002` contains all 26 fresh processes / 13 pairs. `run-001` contains an earlier frozen registration with zero dispatch attempts; it remains preserved. Agent-controlled builds/tests finished before serial timing; background/user load was not controlled.

Independent replay parsed the raw stdout, verified raw stream hashes, ledger order, frozen build/executable identity and finite semantic projections, then recomputed all available small-sample arithmetic. It agrees with the registered summary. Plain stdout emits phase summaries rather than individual samples; large diagnostic phases also omit individual samples. Those native aggregates cannot be reconstructed further from this capture.

## Four primary pairs

Times are milliseconds. Success is one dispatch per process. Rejection is the median of three dispatches within each process; block/apply and block/departure are separately computed medians of the three ratios, not ratios of medians. P is fresh plain; D is fresh diagnostic. Pair order is P→D, D→P, P→D, D→P. Across-process medians average the two central values.

| Pair | Outcome | P apply ms | D apply ms | D/P | Block ms | Block/apply | Block/departure | COW ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | success | 57.4195 | 60.6881 | 1.05692 | 2.9770 | 4.905% | 6.011% | 0.2205 |
| 1 | rejected | 51.9426 | 53.0982 | 1.02225 | 2.9220 | 5.523% | 6.255% | 0.1865 |
| 2 | success | 53.7041 | 58.2366 | 1.08440 | 2.9809 | 5.119% | 6.419% | 0.1898 |
| 2 | rejected | 48.6148 | 53.1640 | 1.09358 | 2.9493 | 5.564% | 6.190% | 0.1678 |
| 3 | success | 61.6123 | 63.9194 | 1.03745 | 3.0192 | 4.723% | 5.718% | 0.2354 |
| 3 | rejected | 58.2622 | 56.5895 | 0.97129 | 2.9765 | 5.215% | 5.839% | 0.1869 |
| 4 | success | 58.2396 | 60.8211 | 1.04433 | 3.0279 | 4.978% | 6.054% | 0.2131 |
| 4 | rejected | 53.8168 | 55.5091 | 1.03145 | 3.0244 | 5.509% | 6.253% | 0.1894 |

Across the four processes, successful P apply median is **57.82955 ms**, D is **60.75460 ms**; block median **3.00005 ms** (range 2.9770–3.0279). Rejected P process-median apply is **52.87970 ms**, D is **54.33655 ms**; block process-median is **2.96290 ms** (range 2.9220–3.0244). Block/apply ranges are **4.723–5.119%** and **5.215–5.564%** respectively.

All twelve raw rejected diagnostic dispatches, retained in original within-process order:

| Pair | Apply ms (three samples) | Block ms (three samples) | Block/apply (three samples) |
| --- | --- | --- | --- |
| 1 | 53.6136, 52.2852, 53.0982 | 2.8867, 2.9220, 2.9327 | 5.384%, 5.589%, 5.523% |
| 2 | 53.1640, 53.0083, 53.3177 | 2.9458, 2.9493, 2.9955 | 5.541%, 5.564%, 5.618% |
| 3 | 56.5895, 59.1195, 53.5924 | 2.9417, 3.0832, 2.9765 | 5.198%, 5.215%, 5.554% |
| 4 | 55.5091, 54.0173, 56.8076 | 2.9883, 3.0244, 3.1295 | 5.383%, 5.599%, 5.509% |

Instrumentation perturbation and host variability are material relative to this block: paired D/P successful median **1.05063**, range **1.03745–1.08440**; rejected median **1.02685**, range **0.97129–1.09358**. These are descriptive observed paired differences, not a calibrated overhead subtraction. One diagnostic rejected process is faster than its plain pair. Clocks, bookkeeping, post-block COW inventory and sample working sets all contribute; block time is not an achievable saving or a strict uninstrumented upper bound.

## Nested cost and exact work

| Outcome | Search ms | COW ms | get_mut/remove/drop ms | Block ms | Search/block | COW/block | Removal/block |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Success | 1.0484 | 0.2168 | 1.3609 | 3.0000 | 34.917% | 7.222% | 45.333% |
| Rejected process medians | 1.0415 | 0.1867 | 1.3773 | 2.9629 | 34.973% | 6.308% | 46.442% |

These are medians of nested recorded quantities, not an additive partition. At 3,000 mutual cycles **every** successful and rejected release attempts 5,998 searches/removals, has zero misses, performs **3,647,564 exact position probes** and shifts **5,361,432 String headers** (estimated **128,674,368 header bytes**). Each release has one removal-boundary map detach: **3 histories / 6,004 occurrence slots** cloned. The three rejected attempts repeat those counts, then roll back. COW is about 6–7% of this small block, so this capture does not make whole-map COW redesign the first choice within removal.

Probes are position predicate calls, not character comparisons. Shift bytes estimate repeated header movement, not allocated heap or string payload traffic. COW populations do not describe all cloned payload bytes and exclude detachment earlier in departure. All reading-stream counters are zero because the fixtures exercise renewal streams; this is not evidence of inexpensive reading removal.

Successful mutual-release size observations (one pair for small sizes; four identical count observations at 3,000):

| Cycles N | Removals | Probes | Shifted headers | Probes + shifts | Cloned occurrence slots | Block ms |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 0 | 0 | 0 | 0 | 0 | 0.0000 |
| 2 | 2 | 4 | 2 | 6 | 8 | 0.0007 |
| 60 | 118 | 842 | 2,934 | 3,776 | 124 | 0.0224 |
| 300 | 598 | 36,866 | 54,030 | 90,896 | 604 | 0.1410 |
| 1,000 | 1,998 | 94,008 | 908,988 | 1,002,996 | 2,004 | 0.5104 |
| 3,000 | 5,998 | 3,647,564 | 5,361,432 | 9,008,996 | 6,004 | 3.0000 |

For the observed N≥2 mutual sizes, probes+shifts equal `N² + 3N − 4`; 1,000→3,000 cycles increases this combined operation count from 1,002,996 to 9,008,996 while removals rise about threefold. The probe/shift split varies with removal order, so probes alone do not describe scaling. This establishes substantial repeated vector work in these fixtures; it is not a measured latency law outside them.

## Supplemental observations

Each row is one pair; times and ratios are descriptive, especially sub-millisecond cases. Rejected block/share values are within-process medians.

| Case | Success P/D ms | Rejected P/D ms | Success/rejected block µs | Success/rejected block share |
| --- | --- | --- | --- | --- |
| release-mutual-1 | 0.0095/0.0093 | 0.0093/0.0088 | 0.0/0.0 | 0.000%/0.000% |
| release-mutual-2 | 0.0216/0.0261 | 0.0235/0.0246 | 0.7/0.8 | 2.682%/3.252% |
| release-mutual-60 | 1.0708/1.0532 | 0.9572/0.9133 | 22.4/23.7 | 2.127%/2.407% |
| release-mutual-300 | 5.4797/5.2016 | 4.8312/4.9715 | 141.0/138.6 | 2.711%/2.769% |
| release-mutual-1000 | 17.0206/18.7233 | 16.0918/16.8985 | 510.4/537.1 | 2.726%/3.105% |
| release-self-3000 | 27.3233/29.8809 | 24.9099/28.4408 | 1565.7/1604.2 | 5.240%/5.640% |
| high-degree-60 | 0.2691/0.3326 | 0.2507/0.2656 | 6.4/6.7 | 1.924%/2.523% |
| high-degree-1000 | 0.2225/0.2502 | 0.2097/0.2037 | 5.7/5.9 | 2.278%/2.896% |

Reachable-chain 3,000 has no final release. Its growth/last-10%-growth median P/D times are **6.9079/6.9566 ms** and **14.5671/14.1918 ms**, with zero renewal-removal blocks. Mutual and self growth also record zero blocks. This removal follow-up therefore does not address the growth cost measured here. Across each 100 unrelated probes, exactly 99 removals perform 198 probes and 99 shifts, with zero removal-boundary COW; aggregate block time is only 13.2–31.1 µs across cases. Do not divide this phase total by one median dispatch.

High-degree is repeated fixed N=16, not growing degree. At 60/1,000 cycles growth records 2,006/33,966 removal blocks, 4,012/67,932 probes and 5,015/84,915 shifts; aggregate block time is 0.3781/5.9065 ms across the entire growth phase. Final release remains 17 removals, 34 probes and 17 shifts, one detach of 18 histories/53 slots. Growing cycle count in this fixture does not produce a growing final-release population.

## Requested-heap comparison

**Every one of 76 nonempty phase peak comparisons has D−P = 0 bytes. Every one of 26 retained-state comparisons has D−P = 0 bytes.** The table shows the absolute common P=D values; the four mutual-3,000 pairs are identical. Growth and growth-tail peaks are equal within every pair, so their common value shares one column. These phases overlap and must not be summed.

| Case | Growth/tail peak B | Steady peak B | Unrelated peak B | Rejected peak B | Success peak B |
| --- | --- | --- | --- | --- | --- |
| release-mutual-3000 | 8,913,064 | 954 | 4,866,520 | 27,403,464 | 27,403,251 |
| release-mutual-1 | 12,181 | 920 | 18,858 | 5,176 | 4,963 |
| release-mutual-2 | 13,805 | 924 | 19,366 | 26,386 | 26,173 |
| release-mutual-60 | 210,544 | 934 | 122,428 | 563,548 | 563,335 |
| release-mutual-300 | 998,032 | 944 | 516,282 | 2,747,170 | 2,746,957 |
| release-mutual-1000 | 3,216,776 | 954 | 1,794,496 | 9,316,824 | 9,316,611 |
| release-self-3000 | 4,441,265 | 499 | 2,434,415 | 13,688,764 | 13,688,551 |
| high-degree-60 | 268,581 | 15,883 | 58,157 | 114,009 | 113,796 |
| high-degree-1000 | 269,972 | 16,036 | 58,361 | 114,237 | 114,024 |
| reachable-chain-3000 | 8,574,441 | 953 | 4,865,371 | — | — |

| Case | Retained before B | Retained after drain B | Maximum observed drain drop B |
| --- | --- | --- | --- |
| release-mutual-3000 | 9,252,154 | 2,948,056 | 16,115,374 |
| release-mutual-1 | 56,225 | 55,983 | 2,940 |
| release-mutual-2 | 56,989 | 56,202 | 5,342 |
| release-mutual-60 | 238,803 | 116,412 | 315,790 |
| release-mutual-300 | 964,961 | 383,690 | 1,603,162 |
| release-mutual-1000 | 3,212,702 | 1,079,672 | 5,359,374 |
| release-self-3000 | 4,618,746 | 1,487,354 | 8,057,687 |
| high-degree-60 | 231,450 | 136,203 | 63,012 |
| high-degree-1000 | 232,511 | 136,271 | 63,148 |
| reachable-chain-3000 | 7,811,422 | 7,811,422 | 2,939 |

The common 3,000-mutual successful peak is **27,403,251 B**; rejected peak **27,403,464 B**. Retained heap falls **9,252,154→2,948,056 B** after successful release/drain; the largest observed drain drop is **16,115,374 B**. Dispatch peak includes transaction/archive work and is not removal-only scratch. The drain drop, retained state and peak have different baselines and cannot be summed into an allocation breakdown. No allocation-by-subsystem claim follows. Diagnostic sample vectors are preallocated before the retained baseline; the release vector is separately corrected. Equal reported requested heap does not establish equal RSS, cache use, stack use or allocator overhead.

## Semantic and replay scope

All 13 fresh plain rows match their accepted historical references, and all 13 fresh diagnostic rows match their fresh plain pair on augmented source hash, sample counts, exact save-string hashes/lengths, retired/withdrawal/queue counts, complete ordered archive NDJSON hashes/counts and successful collector-work counters. At 3,000 mutual cycles, retired records are **5,998→0**, withdrawals **6,000→2**, serialized save bytes **1,290,931→298,583**; total archive NDJSON is **3,228,566 B**, with **6,097 records + 6,097 provenance nodes** (including the unrelated-probe archive). No archive is produced in growth. The harness asserts each of three failed releases leaves the exact save unchanged and publishes no archive. Rejected ordinary collector work is a zero placeholder; attempted work is supplied only by diagnostic TLS. This is finite native digest/count evidence, not complete snapshot/view equality, error-code coverage, general reading coverage or a browser/WASM latency claim.

Independent arithmetic files: `independent-arithmetic.py` SHA-256 `6563227c089e8dba4a284af712248af0c604c769735fa03c47f3269691c1ba20`; `independent-arithmetic-result.json` SHA-256 `abbc402343ec9d410505017a32ec859de56b1a152254a10b032fe282c3dde0d5`. Registered summary `run-002/summary-2026-10-08T16-28-05.251Z.json` SHA-256 `93b863db019898bc6744a3165063791cb3a24f5f0aa406ef4c33e661387431d4`. Contract SHA-256 `375482ec4eb031232ca54fb740634b92788bc176697817ff76b52824bd9ed6aa`. The JSON replay contains every phase, raw small sample, count, ratio and artifact hash used here.

Fresh plain executable SHA-256 `accaddfcecfe0b38780346b6eb6c9b868410b2589a78f4bce231dea568a2a734`; diagnostic `7925d2d059afdf671ce029f6aa886f6fd4079e453e7c5fafbbdbd893a76148b3`. This analysis ran no native executable, Cargo build or test. It does not replace the coordinator’s separate source/package verification or readiness audit.
