# High-degree confirmation: one review trigger

**Successful release crossed the registered review trigger; full growth and rejected release did not.** The successful-release slowdown repeats under this eight-pair criterion, with material variation by run order. This is a separate finding, not a retroactive failure of the completed peak-memory experiment.

All 16 fresh processes completed from 2026-10-08 20:37:44 to 20:38:03 UTC, in the fixed alternating order, without failure or retry. The exact preserved ordinary executables were reused: BEFORE `11c2a6a741609d3b640f27224ab6c9943b07f72c` and AFTER `590fae59aa4295ac5209f930a1e58ac152bd4603`. Neither source nor build changed. The workload remained the existing fixed-N16 high-degree fixture, 1,000 cycles, drain every event and 100 probes. Historical timings were excluded from every new aggregate.

## Frozen criteria and results

Each phase uses the median of its eight paired AFTER/BEFORE process latency ratios. Strictly greater than 1.05 triggers review. Successful release contributes one time per process; rejected release contributes each process's median of three attempts; growth contributes the original harness's median of all 1,000 cycle events. Those are three independent criteria.

| Phase | BEFORE process medians: min / median / max | AFTER process medians: min / median / max | Median paired ratio | Pairs above 1.05 | Trigger |
|---|---:|---:|---:|---:|---|
| Full growth | 0.78530 / 0.86475 / 0.89480 ms | 0.85560 / 0.87620 / 0.90100 ms | 1.008215 (+0.82%) | 1/8 | No |
| Successful release | 0.19640 / 0.22290 / 0.26840 ms | 0.20970 / 0.24425 / 0.27670 ms | 1.066711 (+6.67%) | 6/8 | **Yes** |
| Rejected release | 0.17970 / 0.19705 / 0.20440 ms | 0.18160 / 0.19725 / 0.20590 ms | 1.027581 (+2.76%) | 2/8 | No |

The registered statistic is the median of paired ratios, not a ratio of the two marginal medians. Median paired absolute differences are +7.15 microseconds for growth, +15.45 microseconds for successful release and +5.45 microseconds for rejection. These paired differences likewise need not equal differences between the table's marginal medians: rejection's marginal medians differ by only +0.20 microseconds, while its median paired difference is +5.45 microseconds. Neither substitutes for the registered paired ratio.

The earlier one-pair growth increase of about 19% and rejected-release increase of about 12% did not repeat above the new 5% median criterion. That does not prove their true cost is zero. The successful-release increase did repeat above the criterion. Its absolute size is reported for workload decisions, not used to dismiss a submillisecond cost.

## Variation and execution order

| Phase | Paired ratio min / max | BEFORE-first median (4 pairs) | AFTER-first median (4 pairs) |
|---|---:|---:|---:|
| Full growth | 0.97050 / 1.14733 | 1.008770 | 1.008215 |
| Successful release | 0.78130 / 1.40886 | 1.155689 | 1.030349 |
| Rejected release | 0.90303 / 1.07179 | 1.029470 | 1.027581 |

All four BEFORE-first successful-release pairs exceed 1.05; two of four AFTER-first pairs do. The remaining successful-release pairs are approximately 0.04% and 21.87% faster. The combined registered median remains above its trigger; the order split is not an excuse to select a different population. It does limit interpretation: this short confirmation supports a measured review trigger, not a stable universal 6.67% penalty or proof that the code change alone caused every difference. All controlled builds/tests/compression work was held during capture, but user/background host activity was not globally isolated.

Other summaries are descriptive: last 10% growth has paired median ratio 1.006827, steady 1.000000 and unrelated changes 1.034325. Last 10% overlaps full growth. The unchanged harness retains median/mean/max/p99 summaries, not individual rejected-event times; short-sample p99 is not a reliable tail estimate. No pair was removed or rerun.

## Allocation, retention and semantic equality

Every reported peak repeated its same-variant historical value in all eight pairs. No new memory ceiling was introduced.

| Reported phase | BEFORE whole-apply additional peak | AFTER whole-apply additional peak | Delta |
|---|---:|---:|---:|
| Full growth | 269,972 B | 242,900 B | −27,072 B |
| Last 10% growth | 269,972 B | 242,900 B | −27,072 B |
| Steady | 16,036 B | 16,036 B | 0 B |
| Unrelated changes | 58,361 B | 51,555 B | −6,806 B |
| Successful release | 114,024 B | 103,456 B | −10,568 B |
| Rejected release | 114,237 B | 98,596 B | −15,641 B |

These are whole-apply additional requested heap peaks, including container capacity and transaction/archive staging. They exclude allocator internal overhead/free lists, stack and OS reservation/RSS. They are not independent amounts to sum, archive-origin-only bytes or reduced retained history.

Every within-pair and same-variant historical comparison matches exact save/archive digests and counts, successful collector work, phase counts and retained requested heap. The harness's three rejected events per process also retain its exact rollback-save and zero-published-archive assertions. Rejected work counters are placeholders, not attempted-work evidence.

In both variants, retained requested heap is 232,511 B before release and 136,271 B after release/drain; save size 10,009→2,615 B; retired/withdrawal counts 17→0; archive output 14,648,806 B with 34,082 records and 34,082 provenance nodes. Growth contributes 14,593,848 archive bytes. This fixed-degree workload repeatedly emits archive while retaining a bounded current graph; it is not an increasing-degree or required-chain study. The candidate does not change these lifetime/output costs.

## Recommendation

**Retain the frozen candidate as the experimental memory-improvement baseline, with an explicit successful-release latency tradeoff for this workload. Restrict any claim that high-degree release performance has no regression.** The new confirmation triggers that review: six pairs exceed 5%, and the registered median exceeds 5%. It would be incorrect to report an all-clear or dismiss the finding as noise.

The case for retaining the experimental candidate is that the separate bulk-memory result remains valid, these high-degree allocation reductions repeat exactly, all finite semantic/retention checks match, and neither repeated full-growth nor rejected-release median crosses its review threshold. The successful-release order sensitivity prevents treating the measured penalty as a stable general constant. This is an explicit tradeoff recommendation, not acceptance of an unbounded regression or a universal performance endorsement. If high-degree successful release must itself meet a ≤5% degradation requirement, this artifact does not satisfy that measured criterion; targeted repair would be separate authorized work. No repair, workload-dependent runtime branch or further profiling was started.

## Evidence

- Contract SHA256: `7783052f1d2df037aec2c28087fedb93f4af341cff74cd2fa5f106fa48a2a495`.
- Attempts ledger SHA256: `223c210244c015b67e27b771620adfa9e7cbbf0ef75fc38bac1b1fb5e540f586`.
- Complete arithmetic: `run-001/analysis-2026-10-08T20-38-11.961032+00-00.json`, SHA256 `5fe4a1ff33619e7ab6a3056ca9459ea32f00f9c1c88a6394c32bebc3df3cab5a`.
- All-pair/all-phase tables: matching `.md`, SHA256 `1d65fb9da1831b5bcc8d4463aa4923314cd3d28508ab2f5fad0b76599e164248`.
- Raw stdout/stderr, process/source checks and comparisons remain under `run-001/capture/`; reusable prior inputs, identities and snapshots remain under `run-001/prior/`.

This report recomputes the registered captured population and changes neither source nor thresholds. Previous correctness, memory, chain-growth and retained-capacity findings retain their separate scopes.
