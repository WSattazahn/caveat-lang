# Event-phase investigation: recommendation and review evidence

The profiling-only investigation is complete at local commit **`11c2a6a741609d3b640f27224ab6c9943b07f72c`**, based on accepted PR #177 at `055e2af0b411d8385941dd8522201e5afc73cb34`. It supports deferring renewal batching and proposing one narrower implementation next: **box archive entries before staging them for ordered publication, avoiding the large inline staging buffer**. No optimization was implemented. This is a proposal for approval, not a demonstrated saving or release recommendation.

The proposed acceptance target is **at least 10% less total additional requested heap at both successful and rejected 3,000-cycle mutual-release peaks**, measured with fresh ordinary controls. The current peaks are 27,403,251 B and 27,403,464 B; the corresponding integer ceilings would be **24,662,925 B and 24,663,117 B**. Required records, exact reasons, qualifications, saves, ordered archive content, event admission and transactional rollback must pass the same comparisons. No retention weakening, deferred collection, schema change or lossy summary is part of this proposal.

## What was measured

The frozen population comprises 12 cells and 48 serial processes: four primary 3,000-cycle mutual-release sets; mutual controls at 60, 300 and 1,000 cycles; self-release at 3,000; high degree at 1,000; and reachable chains at 300, 1,000 and 3,000. Each cell has a fresh accepted-runtime reference and three current builds: ordinary counting-allocator control, fixed-boundary timing, and allocation origins. Diagnostic orders rotate. The 12 reference processes are used only for finite semantic comparison; their timing does not enter the reported comparisons.

The timing recorder uses a fixed number of transitions per event, independent of history length: 13 for successful nonempty departure, 12 for late rejection, 11 for empty departure, and four for unknown-event rejection. Its slices do not overlap. The separate origins build disables phase clocks and records one global requested-heap peak with the simultaneous live allocation origins at that instant. There are **24,099 raw events in each diagnostic mode**, including initialization. Growth-tail samples overlap growth and are not an extra population.

Runs used Windows 11, an Intel Core Ultra 9 275HX, Rust 1.98.1/MSVC release builds and Node 24.11.1. Builds and tests were idle during capture; other host activity was not exclusively controlled. Ordinary controls retain their original summary output, so their individual event timings cannot be recovered from these receipts. Primary results are four process observations. Each rejected process value first reduces three events; twelve rejected events are not twelve independent processes.

## Larger costs and the timing boundary

| Measure | Successful release, ms | Rejected release, ms |
|---|---:|---:|
| Ordinary control apply | 63.16950 | 56.52440 |
| Timing-build apply | 62.75105 | 58.59990 |
| Transaction preparation | 0.00395 | 0.00415 |
| Evaluation | 1.97410 | 2.05020 |
| Collection | 22.49850 | 23.17235 |
| Compaction | 0.16010 | 0.16985 |
| Archive construction and removal | 27.33100 | 28.89195 |
| Binding evaluation | 0.00600 | 0.01615 |
| Final settlement and frame cleanup | 6.06245 | 0 |
| Commit cleanup | 4.19950 | 0 |
| Rollback cleanup | 0 | 4.11345 |
| Apply wrapper | 0.00125 | 0.00245 |
| Unattributed outer remainder | 0.00025 | 0.00030 |

These are medians across process values, not an additive partition of the median apply time. Each individual timing event reconciles its exclusive phase sum plus remainder. Per-event shares, aggregated in that same order, put collection at **36.359% / 39.683%** and archive construction/removal at **43.555% / 48.938%** for success/rejection.

Archive construction and removal are interleaved in the implementation. This boundary also contains identity snapshots, withdrawal extraction, journal transfer, ordering and local destruction; it cannot honestly be labeled pure serialization. Final settlement includes destruction of the event frame. Preparation includes input validation and shallow shared-state cloning: later copy-on-write work appears in the phase that triggers it. Its small initial time does not establish that transactional copying is cheap overall. Exact source boundaries and consumers are in `phase-boundaries.md` and `source-cost-inventory.md`.

Instrumentation effects remain visible. The timing/control paired ratios have medians **0.9940 / 1.0433**, with ranges **0.9719–1.0464 / 0.9845–1.0898**. Origins/control ratios have medians **1.0941 / 1.1835**, with ranges **1.0550–1.1490 / 1.0969–1.1964**. Origins-mode latency includes header, TLS, locking and counter work and is not an ordinary latency estimate. A tiny timing remainder does not show negligible observer effect. No fixed overhead is subtracted.

## Where memory peaks, and where the live allocations began

All 16 primary origin-mode release events reached their requested-heap maximum in archive construction/removal. At each maximum the current-event origins were:

| Live requested origin at the same peak | Success, B | Rejection, B |
|---|---:|---:|
| Transaction preparation | 445 | 445 |
| Evaluation | 2,275 | 2,488 |
| Collection | 3,171,130 | 3,171,130 |
| Archive construction/removal | 24,229,401 | 24,229,401 |
| Current-event total / additional peak | **27,403,251** | **27,403,464** |
| Prior-event live storage | 14,126,963 | 16,224,115 |
| Absolute requested peak in origins mode | **41,530,214** | **43,627,579** |

This is a simultaneous partition, not a sum of phase peaks. Earlier collection allocations remain alive at the later archive peak. Prior-event storage includes harness buffers and, for rejection, the saved rollback-reference string; it is not session-only retention. Origins-mode sample buffers alone request 4,870,080 B in this case and are excluded from the retained-session baseline.

Origins headers and alignment add 5,699,138 B / 5,699,173 B at those maxima. The separate header-inclusive System-request peaks are 47,229,352 B / 49,326,752 B. None of these quantities measures OS RSS, allocator free lists, hidden allocator realloc peaks or stack memory. A successful realloc is attributed as a full logical replacement request in its current phase, including in-place realloc; this does not measure physical byte copying.

All 66 nonempty group peak comparisons and 24 retained-state comparisons have zero timing/origins deltas against their ordinary controls. This investigation added diagnostics, so unchanged requested heap is not a failed memory optimization. The rejected event's net +49 B at the observation boundary is its returned error string, not measured session growth.

The ordinary mutual case retains **9,252,154 B before release and 2,948,056 B after release and draining**. Saves are **1,290,931 B / 298,583 B**. Retired records fall from 5,998 to zero. Serialized archive output totals **3,228,566 B** across the run, including unrelated probes; it remains historical storage even though the harness drains it. The largest undrained buffer contains 11,996 items. These figures must remain separate from transient event peaks. The high-degree control emits **14,648,806 B** of archive output, illustrating that a small final save does not imply small historical storage.

## Why propose earlier boxing

At the frozen source's `runtime/src/reactive_departure.rs:949–1011`, entries are staged inline in a map, receive journal data, move into an inline vector for deterministic ordering, and are finally boxed into `ArchiveItem::Record`. The vector's allocation remains while its entries are boxed for publication. The proposed change allocates each entry's final box earlier, carries that box through the existing staging and ordering operations, and moves it into the archive. This targets representation and temporary coexistence, not the required historical payload.

A separate type-layout probe against the frozen build reports **648 B per `ArchiveEntry` versus 8 B per box handle**. For 5,998 entries, the logical ordering-vector payload is **3,886,704 B versus 47,984 B**, a **3,838,720 B difference**, about 14.0% of the measured additional success peak. This is size arithmetic, **not a measured reduction**. Actual vector capacity, map-node slack, sort scratch space and changed allocation lifetimes can alter the global peak. The diagnostic does not assign all 24.23 MB of archive-origin storage to this vector. Source, compiler/artifact hashes and output are in `layout-probe/`.

That concrete redundant representation supports the proposed **10% total-peak target** as a useful next experiment without promising it. On authorization, compare the feature-disabled ordinary before/after builds using the same fixtures, build settings, allocator definitions and drain schedule. Keep the high-degree, unrelated-mutation, bulk-release, rejected-event, archive reconstruction, restore and compatibility checks. Report retained heap, transient peak, archive bytes and absolute dispatch times separately. Use at least eight fresh alternating before/after process pairs for the timing guard: investigate any repeatable median regression above 5%, rather than claiming success from memory alone. If the memory target or correctness gate fails, report the miss and stop for review; do not redefine the target or automatically expand the implementation.

## Required-chain growth remains active work

| Reachable-chain cycles | Ordinary final-10% growth median, ms | Retained requested heap, B | Save, B | Retired records |
|---|---:|---:|---:|---:|
| 300 | 1.5100 | 791,426 | 118,670 | 598 |
| 1,000 | 4.8587 | 2,720,830 | 398,709 | 1,998 |
| 3,000 | 14.1688 | 7,811,422 | 1,238,709 | 5,998 |

These are single-process controls, not stable tail guarantees. The timing build attributes **57.727%, 61.335% and 64.301%** of the corresponding tail to collection. At 3,000 cycles the growth phase visits 8,997,000 vertices and 26,991,000 edges without collecting records. Draining occurs every event; growth creates no archive output here. Required live chains and repeated traversal, rather than an undrained archive, account for this separate concern. Evaluation copy-on-write work and remaining retained capacity also remain candidates for later investigation. The proposed archive change would not solve them.

The large ordinary release observations range from **57.8551–65.0242 ms**; rejected process medians range from **52.3712–57.8775 ms**. Those pauses do not fit a synchronous 16.7 ms frame budget on this host and fixture. Even the chain's approximately 14.2 ms growth median leaves little such budget for the host application. Smaller mutual releases were approximately 1.23 ms at 60 cycles and 6.85 ms at 300, but each smaller cell has only one control process. Four primary processes do not establish p99, unrestricted interactive suitability or any supported workload guarantee. An application with an explicit larger event budget may still find the collector useful; suitability needs that application's measured workload and resource budget.

Renewal removal remains a known quadratic scaling concern from the prior investigation. Its approximately 3 ms block does not explain the growth phases, which had no renewal-removal blocks. Deferral is a priority decision, not a claim that batching can never matter.

## Verification, source and review limits

All **14 local verification checks passed**: formatting; strict full/reactive/timing/origins Clippy; full native (883 passed, one ignored); reactive native (871 passed, one ignored); timing-feature tests (75); origins-feature tests (80, including five allocator tests); the prior withdrawal recorder (two); developer kit (335); Lean staging (44); security/capability staging (14); and the reused independent WASM matrix. Counts overlap. The ignored capacity stress test was not rerun. Staging tests are not a fresh Lean proof or advisory-database audit.

The WASM comparison against accepted PR #177 covers **22 traces, 4,096 steps, 396 rejected steps, 604 restored sessions and 906 expected-closure checks**. The fresh native comparisons cover save hashes, counts, ordered archive hashes and recorded collector work across all 12 cells. These are finite regression observations, not proof for all programs. The native candidate census stayed at 1,441 files; it is not a count of successful individual program checks. CAVEAT's local receipt audit accepted all 14 checks after capture. The independent replay validates raw streams, phase accounting, origin partitions, source/build identities and aggregation; it is not another independent timing run.

`REPRODUCE.md` gives source recovery, build and replay commands. `review-source.bundle` and `review-source.patch` preserve this exact commit; the runtime tree is `89f3a5bde145a3071ce6e41652a866e6b722f3b1`. `builds/` preserves selected build inputs, commands, artifacts and streams. `run-001/` preserves the frozen contract and all attempts. Private receipt stores and keys are excluded from the review ZIP; exported observations do not authenticate execution history.

`prior-677/` includes the **complete original `677ad136` bundle**, its exact source diff and directly accessible raw receipts. Replaying the twelve rejected samples now independently reconstructs **5.5160543649%**, closing the summary-table arithmetic gap without replacing the earlier evidence.

At the final recorded GitHub read, **PRs #174–#177 remain open, draft and unmerged at their unchanged reviewed heads**, each with 18 successful checks. This local commit has **no new GitHub CI run**. No PR was changed, merged, published or released. Accepting this profiling report would authorize neither the proposed optimization nor rc.16 readiness.
