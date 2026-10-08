# Withdrawal batch target measurements

Frozen candidate 8a161dcdfdc8a2a5b3ca43b4183552401002cca7; 40 processes, exact native equality true, registered target pass true.

| Phase | Baseline median ms | Candidate median ms | Ratio of medians | Median pair ratio | Latency gate | Every pair peak gate |
|---|---:|---:|---:|---:|---|---|
| release | 73.0027 | 56.8997 | 0.779420 | 0.779269 | PASS | PASS |
| failed_release | 68.7640 | 52.9905 | 0.770614 | 0.768582 | PASS | PASS |

| Phase / pair | Baseline ms | Candidate ms | C/B | Baseline peak bytes | Candidate peak bytes | Peak gate |
|---|---:|---:|---:|---:|---:|---|
| release / 1 | 76.2470 | 52.4589 | 0.688013 | 27,403,251 | 27,403,251 | PASS |
| release / 2 | 74.7056 | 57.1339 | 0.764787 | 27,403,251 | 27,403,251 | PASS |
| release / 3 | 72.0271 | 58.6703 | 0.814559 | 27,403,251 | 27,403,251 | PASS |
| release / 4 | 70.7410 | 56.0666 | 0.792562 | 27,403,251 | 27,403,251 | PASS |
| release / 5 | 73.9782 | 56.6656 | 0.765977 | 27,403,251 | 27,403,251 | PASS |
| release / 6 | 70.0496 | 57.9388 | 0.827111 | 27,403,251 | 27,403,251 | PASS |
| release / 7 | 70.3319 | 58.4387 | 0.830899 | 27,403,251 | 27,403,251 | PASS |
| release / 8 | 76.3057 | 56.6214 | 0.742034 | 27,403,251 | 27,403,251 | PASS |
| failed_release / 1 | 70.3056 | 50.3948 | 0.716796 | 27,403,464 | 27,403,464 | PASS |
| failed_release / 2 | 73.6199 | 54.0882 | 0.734695 | 27,403,464 | 27,403,464 | PASS |
| failed_release / 3 | 70.3898 | 55.9294 | 0.794567 | 27,403,464 | 27,403,464 | PASS |
| failed_release / 4 | 67.0018 | 53.8841 | 0.804219 | 27,403,464 | 27,403,464 | PASS |
| failed_release / 5 | 73.4196 | 51.3695 | 0.699670 | 27,403,464 | 27,403,464 | PASS |
| failed_release / 6 | 67.2224 | 52.3292 | 0.778449 | 27,403,464 | 27,403,464 | PASS |
| failed_release / 7 | 65.8238 | 53.6518 | 0.815082 | 27,403,464 | 27,403,464 | PASS |
| failed_release / 8 | 66.8051 | 50.6860 | 0.758715 | 27,403,464 | 27,403,464 | PASS |

## Every observed phase peak increase

No phase peak increase occurred in the registered matched processes. This is finite coverage, not a universal memory-regression proof.

## All phase medians and absolute peaks

Eight-process rows use medians of per-process medians; one-process supplementary rows are diagnostic. Complete individual values, p99/max, observed ranges and work are in JSON.

| Workload / cycles | Phase | Trials | Median ms B → C | Peak bytes B → C |
|---|---|---:|---|---|
| release-mutual/3000 | growth | 8 | 5.4772 → 5.4700 | 8,913,064 → 8,913,064 |
| release-mutual/3000 | growth_last_10_percent | 8 | 10.7472 → 10.6401 | 8,913,064 → 8,913,064 |
| release-mutual/3000 | steady | 8 | 0.0009 → 0.0008 | 954 → 954 |
| release-mutual/3000 | unrelated_changes | 8 | 3.1252 → 3.0488 | 4,866,520 → 4,866,520 |
| release-mutual/3000 | failed_release | 8 | 68.7640 → 52.9905 | 27,403,464 → 27,403,464 |
| release-mutual/3000 | release | 8 | 73.0027 → 56.8997 | 27,403,251 → 27,403,251 |
| release-mutual/1 | growth | 1 | 0.1295 → 0.1616 | 12,181 → 12,181 |
| release-mutual/1 | growth_last_10_percent | 1 | 0.1295 → 0.1616 | 12,181 → 12,181 |
| release-mutual/1 | steady | 1 | 0.0008 → 0.0008 | 920 → 920 |
| release-mutual/1 | unrelated_changes | 1 | 0.0081 → 0.0089 | 18,858 → 18,858 |
| release-mutual/1 | failed_release | 1 | 0.0087 → 0.0098 | 5,176 → 5,176 |
| release-mutual/1 | release | 1 | 0.0076 → 0.0088 | 4,963 → 4,963 |
| release-mutual/2 | growth | 1 | 0.1889 → 0.1270 | 13,805 → 13,805 |
| release-mutual/2 | growth_last_10_percent | 1 | 0.1889 → 0.0497 | 13,805 → 13,805 |
| release-mutual/2 | steady | 1 | 0.0008 → 0.0008 | 924 → 924 |
| release-mutual/2 | unrelated_changes | 1 | 0.0091 → 0.0099 | 19,366 → 19,366 |
| release-mutual/2 | failed_release | 1 | 0.0249 → 0.0264 | 26,386 → 26,386 |
| release-mutual/2 | release | 1 | 0.0239 → 0.0238 | 26,173 → 26,173 |
| release-mutual/60 | growth | 1 | 0.2052 → 0.2293 | 210,544 → 210,544 |
| release-mutual/60 | growth_last_10_percent | 1 | 0.4034 → 0.4387 | 210,544 → 210,544 |
| release-mutual/60 | steady | 1 | 0.0008 → 0.0008 | 934 → 934 |
| release-mutual/60 | unrelated_changes | 1 | 0.0608 → 0.0654 | 122,428 → 122,428 |
| release-mutual/60 | failed_release | 1 | 0.8517 → 0.9188 | 563,548 → 563,548 |
| release-mutual/60 | release | 1 | 0.9217 → 0.9775 | 563,335 → 563,335 |
| release-mutual/300 | growth | 1 | 1.1455 → 1.1092 | 998,032 → 998,032 |
| release-mutual/300 | growth_last_10_percent | 1 | 2.1950 → 2.2459 | 998,032 → 998,032 |
| release-mutual/300 | steady | 1 | 0.0008 → 0.0008 | 944 → 944 |
| release-mutual/300 | unrelated_changes | 1 | 0.3712 → 0.3718 | 516,282 → 516,282 |
| release-mutual/300 | failed_release | 1 | 4.7235 → 4.8232 | 2,747,170 → 2,747,170 |
| release-mutual/300 | release | 1 | 5.0182 → 5.0704 | 2,746,957 → 2,746,957 |
| release-mutual/1000 | growth | 1 | 2.7561 → 2.7622 | 3,216,776 → 3,216,776 |
| release-mutual/1000 | growth_last_10_percent | 1 | 4.9771 → 5.0025 | 3,216,776 → 3,216,776 |
| release-mutual/1000 | steady | 1 | 0.0008 → 0.0008 | 954 → 954 |
| release-mutual/1000 | unrelated_changes | 1 | 0.9582 → 0.9854 | 1,794,496 → 1,794,496 |
| release-mutual/1000 | failed_release | 1 | 16.7376 → 15.7108 | 9,316,824 → 9,316,824 |
| release-mutual/1000 | release | 1 | 18.7846 → 17.4124 | 9,316,611 → 9,316,611 |
| release-self/60 | growth | 1 | 0.1137 → 0.1062 | 102,873 → 102,873 |
| release-self/60 | growth_last_10_percent | 1 | 0.2008 → 0.1985 | 102,873 → 102,873 |
| release-self/60 | steady | 1 | 0.0008 → 0.0008 | 499 → 499 |
| release-self/60 | unrelated_changes | 1 | 0.0347 → 0.0354 | 66,921 → 66,921 |
| release-self/60 | failed_release | 1 | 0.3989 → 0.4188 | 275,998 → 275,998 |
| release-self/60 | release | 1 | 0.4144 → 0.4192 | 275,785 → 275,785 |
| release-self/3000 | growth | 1 | 2.5480 → 2.4998 | 4,441,265 → 4,441,265 |
| release-self/3000 | growth_last_10_percent | 1 | 4.4375 → 4.4650 | 4,441,265 → 4,441,265 |
| release-self/3000 | steady | 1 | 0.0009 → 0.0009 | 499 → 499 |
| release-self/3000 | unrelated_changes | 1 | 1.3870 → 1.3791 | 2,434,415 → 2,434,415 |
| release-self/3000 | failed_release | 1 | 26.9366 → 23.1352 | 13,688,764 → 13,688,764 |
| release-self/3000 | release | 1 | 29.3925 → 25.5849 | 13,688,551 → 13,688,551 |
| high-degree/60 | growth | 1 | 0.9149 → 0.9013 | 268,581 → 268,581 |
| high-degree/60 | growth_last_10_percent | 1 | 0.9435 → 0.9144 | 268,581 → 268,581 |
| high-degree/60 | steady | 1 | 0.0009 → 0.0009 | 15,883 → 15,883 |
| high-degree/60 | unrelated_changes | 1 | 0.0259 → 0.0259 | 58,157 → 58,157 |
| high-degree/60 | failed_release | 1 | 0.2267 → 0.2140 | 114,009 → 114,009 |
| high-degree/60 | release | 1 | 0.2756 → 0.2594 | 113,796 → 113,796 |
| high-degree/1000 | growth | 1 | 0.8532 → 0.8556 | 269,972 → 269,972 |
| high-degree/1000 | growth_last_10_percent | 1 | 0.8119 → 0.8164 | 269,972 → 269,972 |
| high-degree/1000 | steady | 1 | 0.0009 → 0.0008 | 16,036 → 16,036 |
| high-degree/1000 | unrelated_changes | 1 | 0.0260 → 0.0258 | 58,361 → 58,361 |
| high-degree/1000 | failed_release | 1 | 0.1845 → 0.1748 | 114,237 → 114,237 |
| high-degree/1000 | release | 1 | 0.2051 → 0.2510 | 114,024 → 114,024 |
| reachable-chain/3000 | growth | 1 | 7.1293 → 6.6368 | 8,574,441 → 8,574,441 |
| reachable-chain/3000 | growth_last_10_percent | 1 | 13.8907 → 13.8021 | 8,574,441 → 8,574,441 |
| reachable-chain/3000 | steady | 1 | 0.0008 → 0.0009 | 953 → 953 |
| reachable-chain/3000 | unrelated_changes | 1 | 2.9673 → 2.9637 | 4,865,371 → 4,865,371 |
| self/60 | growth | 1 | 0.0114 → 0.0123 | 20,582 → 20,582 |
| self/60 | growth_last_10_percent | 1 | 0.0105 → 0.0114 | 20,582 → 20,582 |
| self/60 | steady | 1 | 0.0007 → 0.0007 | 601 → 601 |
| self/60 | unrelated_changes | 1 | 0.0071 → 0.0075 | 17,534 → 17,534 |
| mutual/60 | growth | 1 | 0.0213 → 0.0236 | 24,961 → 24,961 |
| mutual/60 | growth_last_10_percent | 1 | 0.0200 → 0.0229 | 24,961 → 24,961 |
| mutual/60 | steady | 1 | 0.0008 → 0.0008 | 1,151 → 1,151 |
| mutual/60 | unrelated_changes | 1 | 0.0084 → 0.0093 | 18,871 → 18,871 |

## Retention, save and external archive

| Workload / cycles | Retained before B → C | Retained after B → C | Save before / after C | Archive bytes C | Archive records / nodes C |
|---|---|---|---|---:|---|
| release-mutual/3000 | 9,252,154 → 9,252,154 | 2,948,056 → 2,948,056 | 1,290,931 / 298,583 | 3,228,566 | 6,097 / 6,097 |
| release-mutual/1 | 56,225 → 56,225 | 55,983 → 55,983 | 1,231 / 826 | 45,215 | 99 / 99 |
| release-mutual/2 | 56,989 → 56,989 | 56,202 → 56,202 | 1,603 / 932 | 46,225 | 101 / 101 |
| release-mutual/60 | 238,803 → 238,803 | 116,412 → 116,412 | 24,320 / 6,503 | 105,721 | 217 / 217 |
| release-mutual/300 | 964,961 → 964,961 | 383,690 → 383,690 | 123,319 / 29,962 | 356,057 | 697 / 697 |
| release-mutual/1000 | 3,212,702 → 3,212,702 | 1,079,672 → 1,079,672 | 414,635 / 98,583 | 1,092,566 | 2,097 / 2,097 |
| release-self/60 | 127,305 → 127,305 | 71,532 → 71,532 | 12,582 / 3,483 | 75,468 | 158 / 158 |
| release-self/3000 | 4,618,746 → 4,618,746 | 1,487,354 → 1,487,354 | 646,070 / 149,525 | 1,637,039 | 3,098 / 3,098 |
| high-degree/60 | 231,450 → 231,450 | 136,203 → 136,203 | 9,512 / 2,562 | 895,578 | 2,122 / 2,122 |
| high-degree/1000 | 232,511 → 232,511 | 136,271 → 136,271 | 10,009 / 2,615 | 14,648,806 | 34,082 / 34,082 |
| reachable-chain/3000 | 7,811,422 → 7,811,422 | 7,811,422 → 7,811,422 | 1,238,709 / 1,238,709 | 45,413 | 99 / 99 |
| self/60 | 14,513 → 14,513 | 14,513 → 14,513 | 947 / 947 | 75,176 | 158 / 158 |
| mutual/60 | 15,674 → 15,674 | 15,674 → 15,674 | 1,143 / 1,143 | 105,236 | 217 / 217 |

- Requested heap excludes allocator internals, stacks and RSS.
- Dispatch peaks include copies, collector work, departure and archive; not isolated collector scratch.
- Native apply omits snapshot serialization/browser/WASM work.
- Diagnostic per-block clocks/TLS stores perturb timing; diagnostic executable is never used for target acceptance.
- Position result implies exact predicate-call count; shifted bytes estimate moved Withdrawal headers, not String payloads or measured traffic.
- N16 high-degree cycles repeat a fixed shape, not increasing degree.
