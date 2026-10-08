# High-degree confirmation: retain with a measured release-latency tradeoff

**Recommendation: retain `590fae59aa4295ac5209f930a1e58ac152bd4603` as the experimental memory-improvement baseline for this workload. Successful-release latency crossed the new review trigger and remains unresolved. Do not adopt this result as meeting a strict ≤5% successful-release degradation budget.** No repair or further experiment is started by this recommendation.

The separately registered comparison completed all eight alternating-order pairs against `11c2a6a741609d3b640f27224ab6c9943b07f72c`. The candidate remained frozen. The successful-release slowdown repeated under the chosen criterion; the earlier growth and rejected-release increases did not repeat above that threshold. This does not retroactively change the completed bulk-memory experiment's result.

## What was measured

Sixteen fresh processes ran serially on 8 October 2026, 20:37:44–20:38:03 UTC, with no retry, exclusion or added sample. Each used the existing fixed-N16 high-degree fixture, 1,000 growth cycles, draining every event, 100 steady probes, 100 unrelated-change probes, three rejected final-release attempts and one successful final release. It repeats a 16-subject graph; it does not grow the degree to 1,000.

The exact native executables from the completed early-boxing experiment were reused, not rebuilt. Both use Rust 1.98.1/MSVC and comparable release/locked/no-default-features/collector-metrics settings. Attribution features were disabled; the counting allocator and work counters remain present. These are ordinary measurement builds, not completely uninstrumented applications. Their source snapshots, raw build records and executable hashes are preserved.

The new contract was frozen before capture. For each phase, the ordinary median of eight paired AFTER/BEFORE process latency ratios must be **strictly greater than 1.05** to trigger review. Prior single-pair timings are excluded. Controlled builds, tests and compression were held during capture; the Windows host was not globally isolated from background activity.

## Three separate latency results

| Phase | BEFORE median of process medians | AFTER median of process medians | Median paired change | Median paired time change | Pairs above 5% | Trigger |
|---|---:|---:|---:|---:|---:|---|
| Full growth | 0.86475 ms | 0.87620 ms | +0.82% | +7.15 µs | 1/8 | No |
| Successful release | 0.22290 ms | 0.24425 ms | **+6.67%** | **+15.45 µs** | **6/8** | **Yes** |
| Rejected release | 0.19705 ms | 0.19725 ms | +2.76% | +5.45 µs | 2/8 | No |

Paired percentages and paired differences are calculated from individual pairs. They cannot be recovered by dividing or subtracting the two marginal medians in the table. Rejection illustrates this: its marginal difference is +0.20 µs, while the median paired difference is +5.45 µs.

Successful-release ratios range from 0.78130 to 1.40886. BEFORE-first pairs have a median ratio of 1.155689; AFTER-first pairs, 1.030349. All four BEFORE-first pairs and two of four AFTER-first pairs exceed 1.05. The pooled trigger stands. The order variation limits causal attribution and prevents treating 6.67% as a stable penalty across environments; it does not justify excluding pairs or calling the trigger noise.

Every recorded process summary, paired ratio, difference, order split and allocation peak is retained in the [all-pair tables](run-001/analysis-2026-10-08T20-38-11.961032+00-00.md) and matching JSON. Growth uses the original harness's median of 1,000 events; rejection uses the median of three attempts within each process. Individual event timing arrays were not emitted. Overlapping last-decile and other probe summaries are descriptive, not extra review criteria or reliable short-sample tail estimates.

## Storage and correctness boundaries

The following maximum additional requested-heap values repeated exactly in all eight pairs:

| Phase | BEFORE | AFTER | Reduction |
|---|---:|---:|---:|
| Full growth | 269,972 B | 242,900 B | 27,072 B |
| Successful release | 114,024 B | 103,456 B | 10,568 B |
| Rejected release | 114,237 B | 98,596 B | 15,641 B |

These are whole-apply requested allocation peaks above each event's starting live baseline, including capacity and transaction/archive staging. They are not total process memory, isolated collector allocations or archive storage. They exclude allocator internals, stacks and RSS; phase maxima must not be summed. Dispatch timing excludes the subsequent metrics queries, save comparisons and archive drain serialization.

Retained requested heap is unchanged between variants: 232,511 B before release and 136,271 B after release/drain. Saves are 10,009→2,615 B; retired/withdrawal counts are 17→0. Cumulative serialized archive output is 14,648,806 B, with 34,082 records and 34,082 provenance nodes. Moving history into that archive has not eliminated its storage cost. Required-chain growth and retained-capacity costs were not reinvestigated here.

All paired and same-variant historical comparisons matched the selected exact save/archive digests, ordered counts, successful collector-work values and retained heap. The native harness also checks exact rollback saves and zero published archive for the rejected attempts. This is finite tested coverage, not universal semantic equivalence; rejected work counters are placeholders.

## Why retain, and where to restrict use

This experiment shows no >5% median trigger for growth or rejection, reproduces the high-degree peak reductions, and finds no difference in its finite behavioral projections. The separately accepted bulk-release memory saving remains intact. Those grounds favor retaining the experimental implementation over undoing the improvement or attempting an unmeasured repair.

The successful-release cost is still a real review finding. For the tested mixture—1,000 growth events followed by one successful final release—the observed +15.45 µs median paired increment accompanies a 10,568 B release-peak reduction. That is a defensible experimental tradeoff, not an application performance budget. A workload requiring ≤5% successful-release degradation has not passed its requirement here; keep adoption for that budget restricted pending separately authorized repair or stronger workload-specific evidence. Neither the submillisecond times nor the word “experimental” establishes suitability for unrestricted interactive or deadline-sensitive use.

The next decision is therefore **retain with this explicit restriction**, not immediate optimization. Broader memory work remains on the engineering agenda, but this confirmation does not select or authorize its next implementation.

## Evidence and delivery scope

An independently written saved-data checker recalculates the ratios using exact rational arithmetic and verifies the 16-process ledger, 32 streams, source/build identities and finite projections without importing the capture driver or executing native code. Its final receipt binds this report; replay instructions are in [REPRODUCE.md](REPRODUCE.md). Data checks can pass while the successful-release review trigger remains true.

The supplied independent review ZIP passed path, CRC and manifest checks. Its reported Linux WASM tests—including the new reading/journal matrix—are attributed reviewer evidence, not tests rerun here. The preserved 14-check correctness receipts and source bundle are historical context. No fresh full correctness gates, Rust build or candidate CI run are claimed for this source-frozen timing task.

At the read-only 20:32:27 UTC observation, PRs #174–#177 remained open, draft and unmerged at their preserved heads, each with 18 successful checks. Those checks belong to the earlier PR revisions. This task changed no runtime source and performed no push, merge, publication or release.

The review bundle includes the frozen source history, native artifacts/build inputs, all new raw outputs, protocol, independent replay, supplied-review intake and separately labeled prior verification. Private receipt stores, issuer keys, signed CLI logs and build caches are excluded. See [BUNDLE-CONTENTS.md](BUNDLE-CONTENTS.md).
