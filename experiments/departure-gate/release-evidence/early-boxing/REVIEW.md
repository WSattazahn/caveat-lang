# Early archive boxing: scoped experiment result

**Recommendation: accept this as a scoped experimental peak-memory improvement for continued review. The frozen target passed. Keep it local and unmerged; it does not establish release readiness or uniformly faster dispatch.**

Exact candidate: `590fae59aa4295ac5209f930a1e58ac152bd4603`, on `codex/archive-early-boxing`. Baseline: `11c2a6a741609d3b640f27224ab6c9943b07f72c`. Candidate runtime tree: `a5d144827c0d3cfed859dc57b747bed91b33af04`.

## Change and scope

The only production change is in `runtime/src/reactive_departure.rs`: allocate each archive record's final `Box` at construction, retain that box during journal attachment and numeric ordering, and publish the existing box. The runtime diff is five inserted lines and three removed lines, including the explanatory comment. Payload construction, comparator, effect order, settlement, serialization, eligibility and transaction code are unchanged in the source comparison. The commit also contains the frozen protocol and a portable adapter/provenance record for the reused PR177 matrix.

No retention changes, renewal batching, new syntax, save-schema change or other optimization were added. Consumer tracing is in `semantics/source-review.md`; current execution evidence is separate below.

## Frozen primary result

Registration occurred before implementation. Both native executables were freshly built in separate initially empty target directories with the same pinned Rust 1.98.1/MSVC compiler, release settings and `collector-metrics` feature. Attribution instrumentation was disabled. The original counting allocator and work counters remain enabled in both variants: these are ordinary comparison builds, not completely uninstrumented binaries.

All **32 registered processes / 16 pairs** completed without failures, retries or dropped samples. Eight alternating primary pairs used the 3,000-cycle mutual-release fixture; eight single-pair controls covered smaller mutual cases, self cycles, high degree and required chains. Every fresh primary baseline peak reproduced the recorded reference exactly.

| Whole-dispatch additional requested heap | Before, every pair | After, every pair | Reduction | Frozen ceiling | Result |
|---|---:|---:|---:|---:|---|
| Successful release | 27,403,251 B | 21,789,689 B | 5,613,562 B / 20.485022% | 24,662,925 B | All 8 pass |
| Rejected release, maximum of 3 attempts per process | 27,403,464 B | 21,789,902 B | 5,613,562 B / 20.484863% | 24,663,117 B | All 8 pass |

This is additional caller-requested Rust heap during the entire apply call, including temporary transactional and archive allocations. It includes requested container capacity; it excludes allocator bookkeeping/free lists, stack and RSS. The result is not an archive-origin bucket or an estimate from a vector's layout. No phase attribution was collected here, so the measured saving is not assigned entirely to one container.

| Primary timing | Median of before process values | Median of after process values | Median of 8 paired after/before ratios | Guard <= 1.05 |
|---|---:|---:|---:|---|
| Successful release | 65.91800 ms | 57.54685 ms | 0.862183 | Pass |
| Rejected release | 60.79735 ms | 51.56630 ms | 0.838547 | Pass |

The registered guard uses the median of paired ratios, not the ratio of these aggregate medians. Each successful process supplies one release time; each rejected process supplies the harness median of three attempts. The original harness does not emit the three individual times, so their within-process medians cannot be independently reconstructed. Successful paired ratios ranged 0.781075–0.989288; rejected ratios ranged 0.764685–1.120340. The first rejected pair was slower by 12.03%; the other seven were faster. No outlier was removed.

Controlled builds/tests/compression were idle during capture. Other host activity was not exclusively controlled. Eight pairs support the stated comparison; they do not establish a noise-free speedup or a general workload guarantee.

## Controls and material timing caveat

All 90 nonempty phase comparisons are preserved in the timestamped analysis. **No measured requested-heap peak increased.** Growth peaks were unchanged for the mutual, self and chain cases. High-degree growth peak fell from 269,972 to 242,900 B. Unrelated-mutation peaks fell by 6,806 B in every pair; steady peaks were unchanged.

| Single-pair control | Success peak before -> after | Rejected peak before -> after | Success / rejected latency ratio |
|---|---:|---:|---:|
| Mutual 60 | 563,335 -> 442,445 B | 563,548 -> 442,658 B | 0.89083 / 0.87872 |
| Mutual 300 | 2,746,957 -> 2,245,443 B | 2,747,170 -> 2,245,656 B | 0.90929 / 0.87923 |
| Mutual 1,000 | 9,316,611 -> 7,307,337 B | 9,316,824 -> 7,307,550 B | 0.94157 / 0.87252 |
| Self 3,000 | 13,688,551 -> 10,892,902 B | 13,688,764 -> 10,893,115 B | 0.75491 / 0.78421 |
| High degree, N=16, 1,000 cycles | 114,024 -> 103,456 B | 114,237 -> 98,596 B | **1.09334 / 1.12037** |

The high-degree slowdown extends beyond release: growth median 0.7436 -> 0.8851 ms (+19.03%), last-10% growth median 0.7034 -> 0.8580 ms (+21.98%), and unrelated mutation 0.0227 -> 0.0272 ms (+19.82%). Successful release rose 0.2207 -> 0.2413 ms; rejection rose 0.1753 -> 0.1964 ms. These differences matter despite their small absolute times. One process pair cannot establish repeatability or causality, and there was no registered supplemental latency gate. They are not grounds to call this uniformly faster or to silently redefine the primary result.

Other increases remain visible in the complete table: mutual-300 unrelated mutation +6.89%, self-3000 growth tail +5.31%, some primary growth/tail pairs, and submicrosecond steady timings. Supplemental timings and short-sample p99 are descriptive. Full growth and the final 10% overlap and must not be added.

The smallest next performance check, before adopting this candidate for a high-degree workload, is a separately fixed repeated/counterbalanced high-degree comparison with an explicit latency budget and regression criterion. No additional population or optimization was started in this experiment.

## Live storage, capacity and archive remain distinct

Retained requested heap, exact save digests/lengths, ordered archive digests/counts and the registered native work/count projections matched in every pair. This is a temporary-peak improvement; it did not reduce retained session information or host archive output.

For the primary fixture, requested retained heap is 9,252,154 B before release and 2,948,056 B after release/drain in both variants. Exact save lengths are 1,290,931 and 298,583 B. Cumulative host-drained archive is 3,228,566 serialized bytes, comprising 6,097 records and 6,097 provenance nodes; these totals include the harness's unrelated probes. Archive bytes are serialized output, not resident heap, and must not be added to the dispatch peak as if independent. The high-degree case emits 14,648,806 cumulative archive bytes, including 14,593,848 during growth. Moving evidence into that archive does not eliminate its storage cost.

| Required-chain size | Retained requested heap, both variants | Exact save bytes, both variants | Growth additional peak, both variants | Candidate last-10% median |
|---|---:|---:|---:|---:|
| 300 | 791,426 B | 118,670 B | 858,728 B | 1.2495 ms |
| 1,000 | 2,720,830 B | 398,709 B | 2,966,209 B | 4.3167 ms |
| 3,000 | 7,811,422 B | 1,238,709 B | 8,574,441 B | 15.8632 ms |

These chain cases have no final release. Required reasons remain retained; early boxing does not solve their growth. Retained allocation capacity, required-chain representation/copying, conservative retention, bulk departure cost and archive growth remain active engineering problems. This experiment offers no universal bound and does not attribute the entire residual retained heap to capacity alone.

Successful primary candidate releases ranged 54.98–59.90 ms; rejected process medians ranged 49.77–63.97 ms. These releases need about 21.79 MB of additional requested heap beyond the starting state. It does not meet a 16.7 ms synchronous frame budget for this fixture. Its lower peak is practically useful, but an application must specify acceptable pause and memory budgets before claiming support for this workload. “Experimental” alone is not such a guarantee.

## Current correctness evidence

All **14 verification checks passed** on the exact clean candidate, with the final CAVEAT source/artifact receipt audit returning `accepted: true`, `problems: []`. Public commands and raw stdout/stderr are included. Counts overlap and must not be summed as unique test coverage.

- Full kit: **335/335**. Full native: **883 passed**; reactive native: **871 passed**. Each general native suite has one ignored capacity test, which was explicitly executed and passed separately in release mode at the real 65,536-record threshold. Debug suites use the recorded 16 MiB stack; optimized-depth/capacity checks use the default stack.
- Focused collector regressions: **24 passed**. Optimized maximum-depth: **1 passed**. Withdrawal diagnostic recorder: **2 passed**. Formatting and both strict Clippy configurations passed.
- Reused PR176/177 WASM matrices: **38 traces, 4,958 logical steps, 540 logical rejected steps** checked across duplicated baseline/candidate sessions, and **748 accepted restored-session calls**. PR176's 604 restored calls cover paired noop continuation; PR177's 144 cover paired actual release. The matrices check outcomes, snapshots/views, exact saves, ordered archives, rollback and their independent retention/order oracles. They are reused independently authored cases, not new test authorship.
- Registered departure suite: C3 passed with **2,646-byte compressed adapter saves at both 30 and 60 cycles**; published rc.15 save continuation covers 61 state fields. No-window comparison executed 116 of 146 discovered candidates (30 skipped), with 8 seeded runs and 150 events each. Windowed comparison covers 22 programs x 32 runs x 400 events and compares outcomes, not complete windowed state equivalence.
- Browser integration passed all scripted checks using **Chrome 154.0.8037.98**. Native `.cav` candidate census remained **1,447** before/after the native suites and final audit; that is a discovered population, not 1,447 independently successful program checks.

The WASM baseline is a preserved copy of the previously verified ordinary 11c2 build; candidate WASM is freshly built. Both native comparison builds are fresh. Reading/journal consumers rely on existing native and kit cases, not additional coverage invented for this small representation change. Finite checks do not prove arbitrary-program equivalence, OOM/panic equivalence, cross-platform allocator behavior or authenticated historical truth. Source and coverage reviews are in `semantics/`.

## Reviewable evidence and disposition

`run-001/analysis-2026-10-08T19-51-04.703063+00-00.json` and its Markdown companion contain every primary pair, all phase deltas, retained/storage results and fixed criteria. `review/verify-capture.py` independently recomputes the saved arithmetic without importing the driver; full-local and portable saved-only modes state their different source-verification limits. The supplied reviewer ZIP was not provided; its claims are attributed in intake rather than represented as independently rerun here.

The bundle includes complete reachable Git history, the exact patch, source snapshots, executables, WASM artifacts, raw measurement streams, verification outputs and reproduction commands. The initial source-bundle validation helper omitted an explicit clone branch; its failure is preserved. Validation was repaired by cloning the same unchanged bundle with the explicit branch into a fresh validation directory, then checking the baseline object and exact diff. No runtime measurement was repeated. Private receipt keys/stores and build caches are excluded. Hashes and local receipts establish inspectable consistency, not authenticated execution history.

At the recorded read-only GitHub check, PRs **#174–#177 remained unchanged, open, draft and unmerged**, each with 18 successful checks. Those checks belong to their earlier heads; there is no new CI run for this local candidate. Nothing was pushed, merged, published or released.

The registered experiment is complete and successful. The high-degree timing observation needs confirmation before a broader performance claim; the retained-memory and archive issues remain separate unfinished engineering work. No further optimization has been started automatically.