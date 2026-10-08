# Frozen native comparison

Candidate f171d41e9beae588c480f09fc5c03afc7487efb4. 99 native runs; exact native equality true; both primary targets pass in every trial: true.

The after-release field equals before-release for modes without a release event. Archive drain results concern runtime requested heap; emitted archive bytes remain external output.

| Trial | Target | f5 bytes | 3d bytes | Candidate bytes | Added denominator | Candidate ceiling | Added reduction | Result |
|---|---|---:|---:|---:|---:|---:|---:|---|
| 1 | retained | 6,004,653 | 13,413,126 | 7,811,422 | 7,408,473 | 9,708,889 | 75.6121% | PASS |
| 1 | growthPeak | 6,139,810 | 14,176,145 | 8,574,441 | 8,036,335 | 10,157,977 | 69.7047% | PASS |
| 2 | retained | 6,004,653 | 13,413,126 | 7,811,422 | 7,408,473 | 9,708,889 | 75.6121% | PASS |
| 2 | growthPeak | 6,139,810 | 14,176,145 | 8,574,441 | 8,036,335 | 10,157,977 | 69.7047% | PASS |
| 3 | retained | 6,004,653 | 13,413,126 | 7,811,422 | 7,408,473 | 9,708,889 | 75.6121% | PASS |
| 3 | growthPeak | 6,139,810 | 14,176,145 | 8,574,441 | 8,036,335 | 10,157,977 | 69.7047% | PASS |

Allocation values below are medians across registered trial processes; full literal values and ranges are in JSON. All sizes have one trial except the registered three-trial stress rows.

| Workload | Cycles | Trials | Retained 3d → candidate | Growth peak 3d → candidate | Unrelated peak 3d → candidate | After release 3d → candidate |
|---|---:|---:|---|---|---|---|
| self | 60 | 1 | 15,445 → 14,513 | 22,162 → 20,582 | 17,534 → 17,534 | 15,445 → 14,513 |
| self | 300 | 1 | 15,460 → 14,527 | 22,197 → 20,616 | 17,541 → 17,541 | 15,460 → 14,527 |
| self | 1000 | 1 | 15,475 → 14,541 | 22,232 → 20,650 | 17,548 → 17,548 | 15,475 → 14,541 |
| self | 3000 | 3 | 15,475 → 14,541 | 22,232 → 20,650 | 17,548 → 17,548 | 15,475 → 14,541 |
| mutual | 60 | 1 | 17,538 → 15,674 | 28,121 → 24,961 | 18,871 → 18,871 | 17,538 → 15,674 |
| mutual | 300 | 1 | 17,568 → 15,702 | 28,191 → 25,029 | 18,885 → 18,885 | 17,568 → 15,702 |
| mutual | 1000 | 1 | 17,598 → 15,730 | 28,261 → 25,097 | 18,899 → 18,899 | 17,598 → 15,730 |
| mutual | 3000 | 3 | 17,598 → 15,730 | 28,261 → 25,097 | 18,899 → 18,899 | 17,598 → 15,730 |
| reachable-chain | 60 | 1 | 287,238 → 175,414 | 294,919 → 183,095 | 122,521 → 122,521 | 287,238 → 175,414 |
| reachable-chain | 300 | 1 | 1,350,926 → 791,426 | 1,418,228 → 858,728 | 516,374 → 516,374 | 1,350,926 → 791,426 |
| reachable-chain | 1000 | 1 | 4,586,798 → 2,720,830 | 4,832,177 → 2,966,209 | 1,797,547 → 1,797,547 | 4,586,798 → 2,720,830 |
| reachable-chain | 3000 | 3 | 13,413,126 → 7,811,422 | 14,176,145 → 8,574,441 | 4,865,371 → 4,865,371 | 13,413,126 → 7,811,422 |
| release-self | 60 | 1 | 179,377 → 127,305 | 157,573 → 102,873 | 66,921 → 66,921 | 72,464 → 71,532 |
| release-self | 300 | 1 | 759,590 → 499,172 | 782,568 → 506,217 | 266,800 → 266,800 | 206,104 → 205,171 |
| release-self | 1000 | 1 | 2,489,686 → 1,620,878 | 2,507,833 → 1,612,729 | 906,575 → 906,575 | 554,096 → 553,162 |
| release-self | 3000 | 1 | 7,227,642 → 4,618,746 | 7,076,955 → 4,441,265 | 2,434,415 → 2,434,415 | 1,488,288 → 1,487,354 |
| release-mutual | 60 | 1 | 342,771 → 238,803 | 320,424 → 210,544 | 122,428 → 122,428 | 118,276 → 116,412 |
| release-mutual | 300 | 1 | 1,485,973 → 964,961 | 1,550,166 → 998,032 | 516,282 → 516,282 | 385,556 → 383,690 |
| release-mutual | 1000 | 1 | 4,950,494 → 3,212,702 | 5,007,912 → 3,216,776 | 1,794,496 → 1,794,496 | 1,081,540 → 1,079,672 |
| release-mutual | 3000 | 3 | 14,470,210 → 9,252,154 | 14,185,460 → 8,913,064 | 4,866,520 → 4,866,520 | 2,949,924 → 2,948,056 |
| high-degree | 60 | 1 | 237,021 → 231,450 | 274,712 → 268,581 | 58,157 → 58,157 | 136,203 → 136,203 |
| high-degree | 300 | 1 | 237,021 → 231,450 | 274,712 → 268,581 | 58,157 → 58,157 | 136,203 → 136,203 |
| high-degree | 1000 | 3 | 238,099 → 232,511 | 276,120 → 269,972 | 58,361 → 58,361 | 136,271 → 136,271 |

Stress timings: median of the per-process phase medians, milliseconds (observed min–max). Allocator accounting and work instrumentation are enabled on 3d and candidate; these are native apply timings, excluding snapshot serialization and browser/WASM work.

| Workload/size | Phase | Trials | 3d ms (range) | Candidate ms (range) | Change | Peak 3d → candidate |
|---|---|---:|---|---|---|---|
| self/3000 | growth | 3 | 0.0121 (0.0110–0.0127) | 0.0120 (0.0105–0.0122) | -0.83% | 22,232 → 20,650 |
| self/3000 | growth_last_10_percent | 3 | 0.0124 (0.0107–0.0124) | 0.0119 (0.0103–0.0119) | -4.03% | 22,232 → 20,650 |
| self/3000 | steady | 3 | 0.0009 (0.0008–0.0009) | 0.0009 (0.0007–0.0009) | 0.00% | 604 → 604 |
| self/3000 | unrelated_changes | 3 | 0.0079 (0.0069–0.0081) | 0.0078 (0.0069–0.0081) | -1.27% | 17,548 → 17,548 |
| mutual/3000 | growth | 3 | 0.0201 (0.0194–0.0238) | 0.0247 (0.0235–0.0254) | 22.89% | 28,261 → 25,097 |
| mutual/3000 | growth_last_10_percent | 3 | 0.0192 (0.0191–0.0240) | 0.0237 (0.0232–0.0288) | 23.44% | 28,261 → 25,097 |
| mutual/3000 | steady | 3 | 0.0007 (0.0007–0.0009) | 0.0009 (0.0008–0.0010) | 28.57% | 1,175 → 1,175 |
| mutual/3000 | unrelated_changes | 3 | 0.0073 (0.0072–0.0090) | 0.0097 (0.0087–0.0107) | 32.88% | 18,899 → 18,899 |
| reachable-chain/3000 | growth | 3 | 8.3329 (7.8257–8.7944) | 7.8393 (7.5147–8.1895) | -5.92% | 14,176,145 → 8,574,441 |
| reachable-chain/3000 | growth_last_10_percent | 3 | 19.7771 (17.0557–19.9017) | 15.2257 (14.9786–19.8704) | -23.01% | 14,176,145 → 8,574,441 |
| reachable-chain/3000 | steady | 3 | 0.0008 (0.0007–0.0008) | 0.0007 (0.0007–0.0008) | -12.50% | 953 → 953 |
| reachable-chain/3000 | unrelated_changes | 3 | 3.1026 (3.0957–3.3265) | 3.1138 (3.0247–4.7689) | 0.36% | 4,865,371 → 4,865,371 |
| release-self/3000 | growth | 1 | 2.6437 (2.6437–2.6437) | 2.2564 (2.2564–2.2564) | -14.65% | 7,076,955 → 4,441,265 |
| release-self/3000 | growth_last_10_percent | 1 | 4.8432 (4.8432–4.8432) | 4.8633 (4.8633–4.8633) | 0.42% | 7,076,955 → 4,441,265 |
| release-self/3000 | steady | 1 | 0.0008 (0.0008–0.0008) | 0.0008 (0.0008–0.0008) | 0.00% | 499 → 499 |
| release-self/3000 | unrelated_changes | 1 | 1.2333 (1.2333–1.2333) | 1.4267 (1.4267–1.4267) | 15.68% | 2,434,415 → 2,434,415 |
| release-self/3000 | failed_release | 1 | 26.9411 (26.9411–26.9411) | 28.6236 (28.6236–28.6236) | 6.25% | 15,551,826 → 13,688,764 |
| release-self/3000 | release | 1 | 30.0382 (30.0382–30.0382) | 30.0970 (30.0970–30.0970) | 0.20% | 15,551,613 → 13,688,551 |
| release-mutual/3000 | growth | 3 | 6.9954 (6.7004–7.1237) | 6.2938 (5.9759–6.5871) | -10.03% | 14,185,460 → 8,913,064 |
| release-mutual/3000 | growth_last_10_percent | 3 | 14.4943 (13.5849–14.5495) | 12.3077 (12.0388–13.2815) | -15.09% | 14,185,460 → 8,913,064 |
| release-mutual/3000 | steady | 3 | 0.0008 (0.0007–0.0008) | 0.0009 (0.0008–0.0010) | 12.50% | 954 → 954 |
| release-mutual/3000 | unrelated_changes | 3 | 3.1217 (3.0696–3.5818) | 3.2371 (3.1923–3.2645) | 3.70% | 4,866,520 → 4,866,520 |
| release-mutual/3000 | failed_release | 3 | 89.0125 (70.7241–90.3961) | 73.3768 (68.6603–90.2335) | -17.57% | 31,128,532 → 27,403,464 |
| release-mutual/3000 | release | 3 | 93.5230 (78.8077–95.0654) | 80.2518 (77.1321–94.6491) | -14.19% | 31,128,319 → 27,403,251 |
| high-degree/1000 | growth | 3 | 0.8350 (0.8217–0.8892) | 0.8008 (0.7762–0.8526) | -4.10% | 276,120 → 269,972 |
| high-degree/1000 | growth_last_10_percent | 3 | 0.7471 (0.7236–0.8466) | 0.8555 (0.7167–0.8784) | 14.51% | 276,120 → 269,972 |
| high-degree/1000 | steady | 3 | 0.0007 (0.0007–0.0009) | 0.0008 (0.0008–0.0009) | 14.29% | 16,036 → 16,036 |
| high-degree/1000 | unrelated_changes | 3 | 0.0225 (0.0224–0.0277) | 0.0274 (0.0227–0.0283) | 21.78% | 58,361 → 58,361 |
| high-degree/1000 | failed_release | 3 | 0.1932 (0.1756–0.2061) | 0.1973 (0.1720–0.2071) | 2.12% | 114,885 → 114,237 |
| high-degree/1000 | release | 3 | 0.2402 (0.1903–0.2606) | 0.2573 (0.2053–0.2600) | 7.12% | 114,672 → 114,024 |

Save/archive values at the stress sizes are equal between 3d and candidate; f5 differences are retained in JSON. Archive totals include growth, unrelated departures, and successful release where applicable.

| Workload/size | Retired before → after | Save before → after | Archive bytes | Archive records / nodes |
|---|---|---|---:|---|
| self/3000 | 0 → 0 | 965 → 965 | 1,631,733 | 3,098 / 3,098 |
| mutual/3000 | 0 → 0 | 1,177 → 1,177 | 3,218,053 | 6,097 / 6,097 |
| reachable-chain/3000 | 5,998 → 5,998 | 1,238,709 → 1,238,709 | 45,413 | 99 / 99 |
| release-self/3000 | 2,999 → 0 | 646,070 → 149,525 | 1,637,039 | 3,098 / 3,098 |
| release-mutual/3000 | 5,998 → 0 | 1,290,931 → 298,583 | 3,228,566 | 6,097 / 6,097 |
| high-degree/1000 | 17 → 0 | 10,009 → 2,615 | 14,648,806 | 34,082 / 34,082 |

- Requested Rust heap excludes allocator internals, stacks and RSS.
- Growth peak is whole-dispatch additional allocation, not collector-only bytes.
- Native apply timings include allocator/counter overhead, omit snapshot serialization, and do not establish browser/WASM latency.
- f5 lacks collector-metrics and uses the original no-feature build; its zero counters mean unavailable.
- High-degree repeats a fixed 16-owner/16-subject shape; its cycle sweep is not a degree-scaling or universal bound claim.
- Release-self has one sample at each size; repeated successful release-mutual timing has three samples per variant, not a stable population p99.
