# Event-phase profiling: raw arithmetic findings

Revision `11c2a6a741609d3b640f27224ab6c9943b07f72c`; runtime tree `89f3a5bde145a3071ce6e41652a866e6b722f3b1`. All 12 cells / 48 process observations were read from preserved raw stdout, with artifact/stream/ledger hashes checked. The 12 accepted-reference processes provide semantic equality only; none of their timing or memory values enters the following comparisons.

P = fresh ordinary counting-allocator control, T = fixed-boundary timing mode, O = allocation-origin mode with phase clocks disabled. O includes header/TLS/counter/peak-copy overhead and is not an alternative timing estimate. Each primary rejection value first reduces three samples within its process; distributions then retain four process values. Individual ranges are reported without treating those twelve events as independent processes.

## Primary process values

| Pair | Outcome | P apply ms | T apply ms | O apply ms | T/P | O/P | T residual ms |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | release | 57.85510 | 57.35550 | 66.47580 | 0.99136 | 1.14901 | 0.00030 |
| 1 | failed_release | 52.37120 | 52.66280 | 62.58220 | 1.00557 | 1.19497 | 0.00020 |
| 2 | release | 62.44820 | 60.69250 | 70.42270 | 0.97189 | 1.12770 | 0.00020 |
| 2 | failed_release | 56.99550 | 56.11140 | 68.18790 | 0.98449 | 1.19637 | 0.00030 |
| 3 | release | 65.02420 | 64.80960 | 68.95910 | 0.99670 | 1.06051 | 0.00020 |
| 3 | failed_release | 57.87750 | 62.57150 | 63.48740 | 1.08110 | 1.09693 | 0.00030 |
| 4 | release | 63.89080 | 66.85660 | 67.40610 | 1.04642 | 1.05502 | 0.00030 |
| 4 | failed_release | 56.05330 | 61.08840 | 65.69330 | 1.08983 | 1.17198 | 0.00030 |

**release** process-median distributions (min / median / max):

| Measure | Min | Median | Max |
| --- | --- | --- | --- |
| plain_native_median_ns | 57.85510 | 63.16950 | 65.02420 |
| timing_native_median_ns | 57.35550 | 62.75105 | 66.85660 |
| origins_native_median_ns | 66.47580 | 68.18260 | 70.42270 |
| timing_to_plain_ratio | 0.97189 | 0.99403 | 1.04642 |
| origins_to_plain_ratio | 1.05502 | 1.09411 | 1.14901 |
| residual_process_median_ns | 0.00020 | 0.00025 | 0.00030 |
| residual_process_median_share | 0.000% | 0.000% | 0.001% |

Individual T apply range: 57.35550–66.85660 ms. Peak locations across O events: `{"archive_construction_and_removal": 4}`.

| Exclusive phase | Process median ms, min/median/max | Per-event share, process min/median/max |
| --- | --- | --- |
| outside | 0.00000 / 0.00000 / 0.00000 | 0.000% / 0.000% / 0.000% |
| transaction_preparation | 0.00340 / 0.00395 / 0.00650 | 0.005% / 0.006% / 0.011% |
| evaluation | 1.82010 / 1.97410 / 2.13170 | 3.095% / 3.181% / 3.201% |
| collection | 21.17940 / 22.49850 / 24.35500 | 35.446% / 36.359% / 36.927% |
| compaction | 0.11110 / 0.16010 / 0.16920 | 0.194% / 0.247% / 0.270% |
| archive_construction_and_removal | 24.80510 / 27.33100 / 29.76670 | 43.248% / 43.555% / 44.523% |
| binding_evaluation | 0.00530 / 0.00600 / 0.00620 | 0.009% / 0.009% / 0.011% |
| final_settlement | 5.52020 / 6.06245 / 6.26770 | 9.135% / 9.648% / 9.915% |
| commit_cleanup | 3.90550 / 4.19950 / 5.18350 | 6.453% / 6.770% / 7.998% |
| rollback_cleanup | 0.00000 / 0.00000 / 0.00000 | 0.000% / 0.000% / 0.000% |
| apply_wrapper | 0.00110 / 0.00125 / 0.00180 | 0.002% / 0.002% / 0.003% |

**failed_release** process-median distributions (min / median / max):

| Measure | Min | Median | Max |
| --- | --- | --- | --- |
| plain_native_median_ns | 52.37120 | 56.52440 | 57.87750 |
| timing_native_median_ns | 52.66280 | 58.59990 | 62.57150 |
| origins_native_median_ns | 62.58220 | 64.59035 | 68.18790 |
| timing_to_plain_ratio | 0.98449 | 1.04334 | 1.08983 |
| origins_to_plain_ratio | 1.09693 | 1.18348 | 1.19637 |
| residual_process_median_ns | 0.00020 | 0.00030 | 0.00030 |
| residual_process_median_share | 0.000% | 0.001% | 0.001% |

Individual T apply range: 50.97620–63.37570 ms. Peak locations across O events: `{"archive_construction_and_removal": 12}`.

| Exclusive phase | Process median ms, min/median/max | Per-event share, process min/median/max |
| --- | --- | --- |
| outside | 0.00000 / 0.00000 / 0.00000 | 0.000% / 0.000% / 0.000% |
| transaction_preparation | 0.00380 / 0.00415 / 0.00460 | 0.006% / 0.007% / 0.008% |
| evaluation | 1.77770 / 2.05020 / 2.18480 | 3.376% / 3.447% / 3.585% |
| collection | 21.17930 / 23.17235 / 24.01770 | 38.996% / 39.683% / 40.080% |
| compaction | 0.12080 / 0.16985 / 0.17830 | 0.237% / 0.279% / 0.295% |
| archive_construction_and_removal | 25.80740 / 28.89195 / 31.32710 | 48.711% / 48.938% / 49.431% |
| binding_evaluation | 0.01590 / 0.01615 / 0.01680 | 0.026% / 0.028% / 0.030% |
| final_settlement | 0.00000 / 0.00000 / 0.00000 | 0.000% / 0.000% / 0.000% |
| commit_cleanup | 0.00000 / 0.00000 / 0.00000 | 0.000% / 0.000% / 0.000% |
| rollback_cleanup | 3.82340 / 4.11345 / 4.78350 | 6.703% / 7.320% / 7.645% |
| apply_wrapper | 0.00200 / 0.00245 / 0.00320 | 0.004% / 0.004% / 0.005% |

Phase shares are calculated from each event before taking medians. Independent phase medians need not add to median total apply; only each raw event reconciles exclusive phase sum plus residual. The raw JSON contains every primary individual T/O successful and rejected sample; ordinary controls emit phase summaries only.

## Global requested peak and live origins

Each row below is one actual O event peak partition, not a sum of phase maxima or independently aggregated origin medians. Preevent storage includes the live harness/sample buffers and, during rejection, the saved rollback-reference string. Current-event origin is the last successful allocation/reallocation phase; realloc counts a full logical replacement, not physical copying.

| Process/event | Peak location | Start B | Absolute peak B | Additional peak B | Net end−start B | Live origin partition at that peak, B |
| --- | --- | --- | --- | --- | --- | --- |
| 1/release/1 | archive_construction_and_removal | 14,126,963 | 41,530,214 | 27,403,251 | 9,811,276 | transaction_preparation=445; evaluation=2,275; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=14,126,963 |
| 1/failed_release/1 | archive_construction_and_removal | 16,224,115 | 43,627,579 | 27,403,464 | 49 | transaction_preparation=445; evaluation=2,488; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=16,224,115 |
| 1/failed_release/2 | archive_construction_and_removal | 16,224,115 | 43,627,579 | 27,403,464 | 49 | transaction_preparation=445; evaluation=2,488; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=16,224,115 |
| 1/failed_release/3 | archive_construction_and_removal | 16,224,115 | 43,627,579 | 27,403,464 | 49 | transaction_preparation=445; evaluation=2,488; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=16,224,115 |
| 2/release/1 | archive_construction_and_removal | 14,126,963 | 41,530,214 | 27,403,251 | 9,811,276 | transaction_preparation=445; evaluation=2,275; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=14,126,963 |
| 2/failed_release/1 | archive_construction_and_removal | 16,224,115 | 43,627,579 | 27,403,464 | 49 | transaction_preparation=445; evaluation=2,488; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=16,224,115 |
| 2/failed_release/2 | archive_construction_and_removal | 16,224,115 | 43,627,579 | 27,403,464 | 49 | transaction_preparation=445; evaluation=2,488; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=16,224,115 |
| 2/failed_release/3 | archive_construction_and_removal | 16,224,115 | 43,627,579 | 27,403,464 | 49 | transaction_preparation=445; evaluation=2,488; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=16,224,115 |
| 3/release/1 | archive_construction_and_removal | 14,126,963 | 41,530,214 | 27,403,251 | 9,811,276 | transaction_preparation=445; evaluation=2,275; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=14,126,963 |
| 3/failed_release/1 | archive_construction_and_removal | 16,224,115 | 43,627,579 | 27,403,464 | 49 | transaction_preparation=445; evaluation=2,488; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=16,224,115 |
| 3/failed_release/2 | archive_construction_and_removal | 16,224,115 | 43,627,579 | 27,403,464 | 49 | transaction_preparation=445; evaluation=2,488; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=16,224,115 |
| 3/failed_release/3 | archive_construction_and_removal | 16,224,115 | 43,627,579 | 27,403,464 | 49 | transaction_preparation=445; evaluation=2,488; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=16,224,115 |
| 4/release/1 | archive_construction_and_removal | 14,126,963 | 41,530,214 | 27,403,251 | 9,811,276 | transaction_preparation=445; evaluation=2,275; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=14,126,963 |
| 4/failed_release/1 | archive_construction_and_removal | 16,224,115 | 43,627,579 | 27,403,464 | 49 | transaction_preparation=445; evaluation=2,488; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=16,224,115 |
| 4/failed_release/2 | archive_construction_and_removal | 16,224,115 | 43,627,579 | 27,403,464 | 49 | transaction_preparation=445; evaluation=2,488; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=16,224,115 |
| 4/failed_release/3 | archive_construction_and_removal | 16,224,115 | 43,627,579 | 27,403,464 | 49 | transaction_preparation=445; evaluation=2,488; collection=3,171,130; archive_construction_and_removal=24,229,401; prior_event=16,224,115 |

| Process/event | Metadata at requested peak B | Padding at requested peak B | System request at requested peak B | Separate System peak B / location |
| --- | --- | --- | --- | --- |
| 1/release/1 | 4,878,848 | 820,290 | 47,229,352 | 47,229,352 / archive_construction_and_removal |
| 1/failed_release/1 | 4,878,880 | 820,293 | 49,326,752 | 49,326,752 / archive_construction_and_removal |
| 1/failed_release/2 | 4,878,880 | 820,293 | 49,326,752 | 49,326,752 / archive_construction_and_removal |
| 1/failed_release/3 | 4,878,880 | 820,293 | 49,326,752 | 49,326,752 / archive_construction_and_removal |
| 2/release/1 | 4,878,848 | 820,290 | 47,229,352 | 47,229,352 / archive_construction_and_removal |
| 2/failed_release/1 | 4,878,880 | 820,293 | 49,326,752 | 49,326,752 / archive_construction_and_removal |
| 2/failed_release/2 | 4,878,880 | 820,293 | 49,326,752 | 49,326,752 / archive_construction_and_removal |
| 2/failed_release/3 | 4,878,880 | 820,293 | 49,326,752 | 49,326,752 / archive_construction_and_removal |
| 3/release/1 | 4,878,848 | 820,290 | 47,229,352 | 47,229,352 / archive_construction_and_removal |
| 3/failed_release/1 | 4,878,880 | 820,293 | 49,326,752 | 49,326,752 / archive_construction_and_removal |
| 3/failed_release/2 | 4,878,880 | 820,293 | 49,326,752 | 49,326,752 / archive_construction_and_removal |
| 3/failed_release/3 | 4,878,880 | 820,293 | 49,326,752 | 49,326,752 / archive_construction_and_removal |
| 4/release/1 | 4,878,848 | 820,290 | 47,229,352 | 47,229,352 / archive_construction_and_removal |
| 4/failed_release/1 | 4,878,880 | 820,293 | 49,326,752 | 49,326,752 / archive_construction_and_removal |
| 4/failed_release/2 | 4,878,880 | 820,293 | 49,326,752 | 49,326,752 / archive_construction_and_removal |
| 4/failed_release/3 | 4,878,880 | 820,293 | 49,326,752 | 49,326,752 / archive_construction_and_removal |

**release:** across-process medians of per-process requested traffic/calls. These cumulative logical flows are different from live-at-peak origins; per-phase median rows must not be summed as a median event total.

| Operation phase | Requested inflow B | Requested freed B | alloc / zeroed / realloc / dealloc calls |
| --- | --- | --- | --- |
| outside | 0 | 0 | 0 / 0 / 0 / 0 |
| transaction_preparation | 571 | 0 | 12 / 0 / 0 / 0 |
| evaluation | 1,025,359 | 328,294 | 19,894 / 0 / 48 / 6,024 |
| collection | 7,481,331 | 5,005,117 | 184,687 / 0 / 0 / 138,697 |
| compaction | 0 | 0 | 0 / 0 / 0 / 0 |
| archive_construction_and_removal | 39,840,627 | 25,049,937 | 189,809 / 0 / 33 / 147,614 |
| binding_evaluation | 363 | 117 | 9 / 0 / 0 / 2 |
| final_settlement | 7,901,824 | 6,321,438 | 78,771 / 0 / 12 / 54,786 |
| commit_cleanup | 1,439,520 | 11,173,416 | 1 / 0 / 0 / 141,614 |
| rollback_cleanup | 0 | 0 | 0 / 0 / 0 / 0 |
| apply_wrapper | 0 | 0 | 0 / 0 / 0 / 0 |

**failed_release:** across-process medians of per-process requested traffic/calls. These cumulative logical flows are different from live-at-peak origins; per-phase median rows must not be summed as a median event total.

| Operation phase | Requested inflow B | Requested freed B | alloc / zeroed / realloc / dealloc calls |
| --- | --- | --- | --- |
| outside | 0 | 0 | 0 / 0 / 0 / 0 |
| transaction_preparation | 571 | 0 | 12 / 0 / 0 / 0 |
| evaluation | 1,025,586 | 328,301 | 19,897 / 0 / 48 / 6,025 |
| collection | 7,481,331 | 5,005,124 | 184,687 / 0 / 0 / 138,698 |
| compaction | 0 | 0 | 0 / 0 / 0 / 0 |
| archive_construction_and_removal | 39,840,627 | 25,049,937 | 189,809 / 0 / 33 / 147,614 |
| binding_evaluation | 761 | 693 | 15 / 0 / 2 / 14 |
| final_settlement | 0 | 0 | 0 / 0 / 0 / 0 |
| commit_cleanup | 0 | 0 | 0 / 0 / 0 / 0 |
| rollback_cleanup | 0 | 17,964,753 | 0 / 0 / 0 / 102,068 |
| apply_wrapper | 49 | 68 | 1 / 0 / 0 / 1 |

## Growth and supplemental controls

Single-triple supplemental timing is descriptive. The growth tail is the final 10% of the same growth samples, not a separate/additional workload. Tail origin rows use the single event with the largest absolute requested peak within that tail; they are observed partitions, not phase-peak sums.

| Case | Tail P ms | Tail T ms | Tail O ms | Three largest median event shares | Selected absolute O peak B | Selected O peak origins B |
| --- | --- | --- | --- | --- | --- | --- |
| release-mutual-3000-t1 | 11.24070 | 10.74810 | 13.72040 | evaluation 44.598%; collection 36.787%; commit_cleanup 18.300% | 23,180,871 | transaction_preparation=861; evaluation=6,697,381; collection=2,214,822; prior_event=14,267,807 |
| release-mutual-3000-t2 | 12.24790 | 11.43090 | 14.85930 | evaluation 44.404%; collection 36.600%; commit_cleanup 18.467% | 23,180,871 | transaction_preparation=861; evaluation=6,697,381; collection=2,214,822; prior_event=14,267,807 |
| release-mutual-3000-t3 | 11.79910 | 11.40750 | 15.87620 | evaluation 44.190%; collection 36.381%; commit_cleanup 18.947% | 23,180,871 | transaction_preparation=861; evaluation=6,697,381; collection=2,214,822; prior_event=14,267,807 |
| release-mutual-3000-t4 | 12.05550 | 12.53750 | 15.39900 | evaluation 42.689%; collection 37.063%; commit_cleanup 19.564% | 23,180,871 | transaction_preparation=861; evaluation=6,697,381; collection=2,214,822; prior_event=14,267,807 |
| release-mutual-60-t1 | 0.52790 | 0.47610 | 0.68690 | evaluation 45.717%; collection 41.919%; commit_cleanup 12.043% | 855,978 | transaction_preparation=861; evaluation=160,654; collection=49,029; prior_event=645,434 |
| release-mutual-300-t1 | 2.78200 | 2.43800 | 2.76220 | collection 45.945%; evaluation 43.122%; commit_cleanup 10.809% | 2,741,667 | transaction_preparation=861; evaluation=757,046; collection=240,125; prior_event=1,743,635 |
| release-mutual-1000-t1 | 5.31120 | 5.14200 | 6.18360 | evaluation 43.570%; collection 42.410%; commit_cleanup 12.894% | 8,309,689 | transaction_preparation=861; evaluation=2,452,837; collection=763,078; prior_event=5,092,913 |
| release-self-3000-t1 | 4.96690 | 4.74360 | 5.99510 | evaluation 40.319%; collection 39.839%; commit_cleanup 19.290% | 14,004,293 | transaction_preparation=445; evaluation=3,340,038; collection=1,100,782; prior_event=9,563,028 |
| high-degree-1000-t1 | 0.86610 | 0.86080 | 0.99270 | evaluation 53.541%; collection 29.474%; archive_construction_and_removal 6.817% | 2,361,485 | transaction_preparation=14,173; evaluation=100,988; collection=50,620; archive_construction_and_removal=104,191; prior_event=2,091,513 |
| reachable-chain-300-t1 | 1.51000 | 1.44660 | 1.64170 | collection 57.727%; evaluation 29.061%; commit_cleanup 13.712% | 2,425,056 | transaction_preparation=860; evaluation=660,330; collection=197,538; prior_event=1,566,328 |
| reachable-chain-1000-t1 | 4.85870 | 4.71480 | 5.57990 | collection 61.335%; evaluation 23.660%; commit_cleanup 12.925% | 7,564,819 | transaction_preparation=860; evaluation=2,294,003; collection=671,346; prior_event=4,598,610 |
| reachable-chain-3000-t1 | 14.16880 | 14.51530 | 17.87910 | collection 64.301%; evaluation 22.605%; commit_cleanup 12.736% | 21,399,646 | transaction_preparation=860; evaluation=6,527,491; collection=2,046,090; prior_event=12,825,205 |

| Supplement | Outcome | P/T/O apply medians ms | T/P; O/P | Largest median timing phase / share |
| --- | --- | --- | --- | --- |
| release-mutual-60-t1 | release | 1.22810 / 1.11170 / 1.23490 | 0.90522; 1.00554 | archive_construction_and_removal / 52.325% |
| release-mutual-60-t1 | failed_release | 1.04690 / 1.00390 / 1.10770 | 0.95893; 1.05808 | archive_construction_and_removal / 54.291% |
| release-mutual-300-t1 | release | 6.85150 / 5.81420 / 6.08330 | 0.84860; 0.88788 | archive_construction_and_removal / 48.653% |
| release-mutual-300-t1 | failed_release | 5.64240 / 5.34390 / 5.40380 | 0.94710; 0.95771 | archive_construction_and_removal / 53.699% |
| release-mutual-1000-t1 | release | 18.19360 / 20.00620 / 21.07400 | 1.09963; 1.15832 | archive_construction_and_removal / 45.924% |
| release-mutual-1000-t1 | failed_release | 16.60860 / 16.78660 / 18.90460 | 1.01072; 1.13824 | archive_construction_and_removal / 48.628% |
| release-self-3000-t1 | release | 27.39370 / 26.98060 / 32.29260 | 0.98492; 1.17883 | archive_construction_and_removal / 40.790% |
| release-self-3000-t1 | failed_release | 24.79080 / 24.90350 / 29.00870 | 1.00455; 1.17014 | archive_construction_and_removal / 47.646% |
| high-degree-1000-t1 | release | 0.21740 / 0.25800 / 0.25240 | 1.18675; 1.16099 | collection / 31.434% |
| high-degree-1000-t1 | failed_release | 0.18470 / 0.18470 / 0.22400 | 1.00000; 1.21278 | collection / 45.613% |

## Peak, retained heap, save and archive comparisons

| Case | T/O sample-buffer requested B | T/O pre-session harness baseline B |
| --- | --- | --- |
| release-mutual-3000-t1 | 948,384 / 4,870,080 | 952,320 / 4,874,014 |
| release-mutual-3000-t2 | 948,384 / 4,870,080 | 952,320 / 4,874,014 |
| release-mutual-3000-t3 | 948,384 / 4,870,080 | 952,320 / 4,874,014 |
| release-mutual-3000-t4 | 948,384 / 4,870,080 | 952,320 / 4,874,014 |
| release-mutual-60-t1 | 78,144 / 401,280 | 82,078 / 405,212 |
| release-mutual-300-t1 | 149,184 / 766,080 | 153,119 / 770,013 |
| release-mutual-1000-t1 | 356,384 / 1,830,080 | 360,320 / 1,834,014 |
| release-self-3000-t1 | 948,384 / 4,870,080 | 951,772 / 4,873,466 |
| high-degree-1000-t1 | 356,384 / 1,830,080 | 368,539 / 1,842,233 |
| reachable-chain-300-t1 | 149,184 / 766,080 | 150,235 / 767,129 |
| reachable-chain-1000-t1 | 356,384 / 1,830,080 | 357,436 / 1,831,130 |
| reachable-chain-3000-t1 | 948,384 / 4,870,080 | 949,436 / 4,871,130 |

| Case/group | P requested peak B | T−P B | O−P B |
| --- | --- | --- | --- |
| release-mutual-3000-t1/growth | 8,913,064 | 0 | 0 |
| release-mutual-3000-t1/growth_last_10_percent | 8,913,064 | 0 | 0 |
| release-mutual-3000-t1/steady | 954 | 0 | 0 |
| release-mutual-3000-t1/unrelated_changes | 4,866,520 | 0 | 0 |
| release-mutual-3000-t1/failed_release | 27,403,464 | 0 | 0 |
| release-mutual-3000-t1/release | 27,403,251 | 0 | 0 |
| release-mutual-3000-t2/growth | 8,913,064 | 0 | 0 |
| release-mutual-3000-t2/growth_last_10_percent | 8,913,064 | 0 | 0 |
| release-mutual-3000-t2/steady | 954 | 0 | 0 |
| release-mutual-3000-t2/unrelated_changes | 4,866,520 | 0 | 0 |
| release-mutual-3000-t2/failed_release | 27,403,464 | 0 | 0 |
| release-mutual-3000-t2/release | 27,403,251 | 0 | 0 |
| release-mutual-3000-t3/growth | 8,913,064 | 0 | 0 |
| release-mutual-3000-t3/growth_last_10_percent | 8,913,064 | 0 | 0 |
| release-mutual-3000-t3/steady | 954 | 0 | 0 |
| release-mutual-3000-t3/unrelated_changes | 4,866,520 | 0 | 0 |
| release-mutual-3000-t3/failed_release | 27,403,464 | 0 | 0 |
| release-mutual-3000-t3/release | 27,403,251 | 0 | 0 |
| release-mutual-3000-t4/growth | 8,913,064 | 0 | 0 |
| release-mutual-3000-t4/growth_last_10_percent | 8,913,064 | 0 | 0 |
| release-mutual-3000-t4/steady | 954 | 0 | 0 |
| release-mutual-3000-t4/unrelated_changes | 4,866,520 | 0 | 0 |
| release-mutual-3000-t4/failed_release | 27,403,464 | 0 | 0 |
| release-mutual-3000-t4/release | 27,403,251 | 0 | 0 |
| release-mutual-60-t1/growth | 210,544 | 0 | 0 |
| release-mutual-60-t1/growth_last_10_percent | 210,544 | 0 | 0 |
| release-mutual-60-t1/steady | 934 | 0 | 0 |
| release-mutual-60-t1/unrelated_changes | 122,428 | 0 | 0 |
| release-mutual-60-t1/failed_release | 563,548 | 0 | 0 |
| release-mutual-60-t1/release | 563,335 | 0 | 0 |
| release-mutual-300-t1/growth | 998,032 | 0 | 0 |
| release-mutual-300-t1/growth_last_10_percent | 998,032 | 0 | 0 |
| release-mutual-300-t1/steady | 944 | 0 | 0 |
| release-mutual-300-t1/unrelated_changes | 516,282 | 0 | 0 |
| release-mutual-300-t1/failed_release | 2,747,170 | 0 | 0 |
| release-mutual-300-t1/release | 2,746,957 | 0 | 0 |
| release-mutual-1000-t1/growth | 3,216,776 | 0 | 0 |
| release-mutual-1000-t1/growth_last_10_percent | 3,216,776 | 0 | 0 |
| release-mutual-1000-t1/steady | 954 | 0 | 0 |
| release-mutual-1000-t1/unrelated_changes | 1,794,496 | 0 | 0 |
| release-mutual-1000-t1/failed_release | 9,316,824 | 0 | 0 |
| release-mutual-1000-t1/release | 9,316,611 | 0 | 0 |
| release-self-3000-t1/growth | 4,441,265 | 0 | 0 |
| release-self-3000-t1/growth_last_10_percent | 4,441,265 | 0 | 0 |
| release-self-3000-t1/steady | 499 | 0 | 0 |
| release-self-3000-t1/unrelated_changes | 2,434,415 | 0 | 0 |
| release-self-3000-t1/failed_release | 13,688,764 | 0 | 0 |
| release-self-3000-t1/release | 13,688,551 | 0 | 0 |
| high-degree-1000-t1/growth | 269,972 | 0 | 0 |
| high-degree-1000-t1/growth_last_10_percent | 269,972 | 0 | 0 |
| high-degree-1000-t1/steady | 16,036 | 0 | 0 |
| high-degree-1000-t1/unrelated_changes | 58,361 | 0 | 0 |
| high-degree-1000-t1/failed_release | 114,237 | 0 | 0 |
| high-degree-1000-t1/release | 114,024 | 0 | 0 |
| reachable-chain-300-t1/growth | 858,728 | 0 | 0 |
| reachable-chain-300-t1/growth_last_10_percent | 858,728 | 0 | 0 |
| reachable-chain-300-t1/steady | 943 | 0 | 0 |
| reachable-chain-300-t1/unrelated_changes | 516,374 | 0 | 0 |
| reachable-chain-1000-t1/growth | 2,966,209 | 0 | 0 |
| reachable-chain-1000-t1/growth_last_10_percent | 2,966,209 | 0 | 0 |
| reachable-chain-1000-t1/steady | 953 | 0 | 0 |
| reachable-chain-1000-t1/unrelated_changes | 1,797,547 | 0 | 0 |
| reachable-chain-3000-t1/growth | 8,574,441 | 0 | 0 |
| reachable-chain-3000-t1/growth_last_10_percent | 8,574,441 | 0 | 0 |
| reachable-chain-3000-t1/steady | 953 | 0 | 0 |
| reachable-chain-3000-t1/unrelated_changes | 4,865,371 | 0 | 0 |

| Case | P retained before/after B | T−P retained before/after B | O−P retained before/after B | Save before/after B | Archive growth/total B |
| --- | --- | --- | --- | --- | --- |
| release-mutual-3000-t1 | 9,252,154 / 2,948,056 | 0 / 0 | 0 / 0 | 1,290,931 / 298,583 | 0 / 3,228,566 |
| release-mutual-3000-t2 | 9,252,154 / 2,948,056 | 0 / 0 | 0 / 0 | 1,290,931 / 298,583 | 0 / 3,228,566 |
| release-mutual-3000-t3 | 9,252,154 / 2,948,056 | 0 / 0 | 0 / 0 | 1,290,931 / 298,583 | 0 / 3,228,566 |
| release-mutual-3000-t4 | 9,252,154 / 2,948,056 | 0 / 0 | 0 / 0 | 1,290,931 / 298,583 | 0 / 3,228,566 |
| release-mutual-60-t1 | 238,803 / 116,412 | 0 / 0 | 0 / 0 | 24,320 / 6,503 | 0 / 105,721 |
| release-mutual-300-t1 | 964,961 / 383,690 | 0 / 0 | 0 / 0 | 123,319 / 29,962 | 0 / 356,057 |
| release-mutual-1000-t1 | 3,212,702 / 1,079,672 | 0 / 0 | 0 / 0 | 414,635 / 98,583 | 0 / 1,092,566 |
| release-self-3000-t1 | 4,618,746 / 1,487,354 | 0 / 0 | 0 / 0 | 646,070 / 149,525 | 0 / 1,637,039 |
| high-degree-1000-t1 | 232,511 / 136,271 | 0 / 0 | 0 / 0 | 10,009 / 2,615 | 14,593,848 / 14,648,806 |
| reachable-chain-300-t1 | 791,426 / 791,426 | 0 / 0 | 0 / 0 | 118,670 / 118,670 | 0 / 45,116 |
| reachable-chain-1000-t1 | 2,720,830 / 2,720,830 | 0 / 0 | 0 / 0 | 398,709 / 398,709 | 0 / 45,413 |
| reachable-chain-3000-t1 | 7,811,422 / 7,811,422 | 0 / 0 | 0 / 0 | 1,238,709 / 1,238,709 | 0 / 45,413 |

Requested heap includes requested container capacity but excludes allocator metadata/free lists, stack and OS reservations/RSS. Sample buffers are excluded from retained-session baselines but stay real live bytes in absolute peak origin partitions. Emitted archive bytes are serialization volume, not a persistent buffer in this draining harness. Unchanged required-chain retention must remain separate from transient allocations and removal work.

## Candidate source attribution, pending review

release: archive_construction_and_removal 43.555%; collection 36.359%; final_settlement 9.648%.
failed_release: archive_construction_and_removal 48.938%; collection 39.683%; rollback_cleanup 7.320%.

This ranking identifies broad instrumented phases only. It does not establish which container or copy operation caused their cost. A concrete next implementation and numerical acceptance target remain unselected pending inspection of these phase results against the exact source, control perturbation, required information and rollback boundaries. If that mapping is ambiguous, the result must remain explicitly unattributed rather than naming an unsupported optimization. No optimization is implemented or authorized by this arithmetic report.

Contract SHA-256 `0cdf1d185f43859c4a3934ca76851af2d15d320303ec003f9c19e10561f66097`. Analysis script SHA-256 `991bb058df5f908fbd0a890d9c90e69757583ec09daf6ea30504a02cf9667c5d`. Complete per-process phase/time/origin/flow distributions, individual primary samples, retained/archive/count projections and checked input hashes are in the JSON companion. Ordinary control raw event times are unavailable; only its native summaries can be compared. No native executable, test or build was launched by this script.
