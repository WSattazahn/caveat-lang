# Early archive boxing: measured findings

The registered native experiment passes both 10% memory targets and the primary latency guard. All 32 fresh processes completed in their fixed order, without failure, retry or target adjustment. Capture ran from 2026-10-08 19:44:26 to 19:50:53 UTC after the coordinator confirmed agent-controlled builds/tests were idle. This is not a claim that background host activity was isolated.

BEFORE: `11c2a6a741609d3b640f27224ab6c9943b07f72c`, executable `2286a87f01560aed8f12ee4f5a6f8f5116f5ea79b4d3bc8f72e6016afae9a95f`.

AFTER: `590fae59aa4295ac5209f930a1e58ac152bd4603`, runtime tree `a5d144827c0d3cfed859dc57b747bed91b33af04`, executable `64125fda7f7323469f9d31f4ac5f787bf3b817856a38d25af2b653044be643d2`.

Both fresh builds used the unchanged original counting-allocator harness, `--release --locked --no-default-features --features collector-metrics`, separate initially empty target directories, and the same pinned compiler. Attribution instrumentation was disabled. Freeze checks enforce `runtime/src/reactive_departure.rs` as the only changed native input. The preregistration contract SHA256 is `cf21d21e7af51394d626f87232def44884520908aec05f2b697e8bef398253f4`.

## Primary mutual3000 result

| Outcome | BEFORE additional peak | AFTER additional peak | Reduction | Fixed ceiling | All 8 peaks pass |
|---|---:|---:|---:|---:|---|
| Successful release | 27,403,251 B | 21,789,689 B | 5,613,562 B / 20.485022% | 24,662,925 B | Yes |
| Rejected release | 27,403,464 B | 21,789,902 B | 5,613,562 B / 20.484863% | 24,663,117 B | Yes |

Each outcome had identical peaks across all eight process pairs. Every fresh BEFORE peak exactly reproduced its preregistered historical value. Rejected peak is the original harness's maximum over three attempted releases per process; the target therefore bounds all three attempted-release peaks in each process.

| Outcome | Median BEFORE process time | Median AFTER process time | Paired ratio min / median / max | Pairs above 1.05 | Registered guard |
|---|---:|---:|---:|---:|---|
| Successful release | 65.91800 ms | 57.54685 ms | 0.78108 / 0.86218 / 0.98929 | 0/8 | Pass |
| Rejected release | 60.79735 ms | 51.56630 ms | 0.76468 / 0.83855 / 1.12034 | 1/8 | Pass |

The guard uses the median of eight paired AFTER/BEFORE ratios, not a ratio of the two aggregate medians. Each rejected process contributes its median of three attempts. The first rejected pair was 12.03% slower; the remaining seven were faster. The range and that outlier remain in the result. The unchanged harness emits phase median/mean/max/p99 summaries rather than individual failed-event times, and this experiment makes no stable-tail or noise-free speedup claim.

These peaks cover additional requested heap during the entire apply call, including temporary containers, transaction copies and undrained archive. They are not archive-origin-only savings or a change in RSS. Requested container capacity is included; allocator internal overhead/free lists and OS reservation are excluded.

## Supplemental tradeoffs and unresolved costs

All 90 nonempty reported phase-summary comparisons have AFTER additional peak <= BEFORE; full-growth and last10% summaries overlap. No broader workload guarantee follows from this finite set. Every retained-heap comparison is exactly equal.

| Supplemental release case | Successful peak BEFORE → AFTER | Rejected peak BEFORE → AFTER | Successful / rejected time ratio |
|---|---:|---:|---:|
| Mutual60 | 563,335 → 442,445 B | 563,548 → 442,658 B | 0.89083 / 0.87872 |
| Mutual300 | 2,746,957 → 2,245,443 B | 2,747,170 → 2,245,656 B | 0.90929 / 0.87923 |
| Mutual1000 | 9,316,611 → 7,307,337 B | 9,316,824 → 7,307,550 B | 0.94157 / 0.87252 |
| Self3000 | 13,688,551 → 10,892,902 B | 13,688,764 → 10,893,115 B | 0.75491 / 0.78421 |
| Fixed-N16 high-degree1000 | 114,024 → 103,456 B | 114,237 → 98,596 B | 1.09334 / 1.12037 |

The high-degree case was slower in its one pair: successful release 0.22070→0.24130 ms, rejected process median 0.17530→0.19640 ms, growth median 0.74360→0.88510 ms and last 10% median 0.70340→0.85800 ms. Its growth additional peak nevertheless fell 269,972→242,900 B. Its unrelated-change ratio was 1.19824 and its very short steady median 0.0007→0.0009 ms. These are observed single-pair changes, not proof of a repeatable regression or grounds to erase the observation as noise. The protocol sets a latency guard only for the eight-pair primary population; this smaller-case tradeoff remains visible for review. The fixed 16-subject graph repeats rather than increasing degree with cycle count.

| Required-chain cycles | Unchanged growth peak | Unchanged retained heap | Unchanged save bytes | Last10% time BEFORE → AFTER |
|---|---:|---:|---:|---:|
| 300 | 858,728 B | 791,426 B | 118,670 B | 1.2362 → 1.2495 ms |
| 1,000 | 2,966,209 B | 2,720,830 B | 398,709 B | 4.1199 → 4.3167 ms |
| 3,000 | 8,574,441 B | 7,811,422 B | 1,238,709 B | 15.9627 → 15.8632 ms |

Required-chain growth emits no archive and remains required-retention work, outside this optimization's benefit. The one-pair chain timings do not establish improved scaling. Unrelated-change peaks fall by 6,806 B in all 16 cells, but most growth and all steady peaks are unchanged. The full analysis reports every absolute phase peak and time, including mutual300 growth and unrelated timing increases and self3000 last 10% timing increase; these are not hidden by the primary result.

## Equality and retention

All 16 native pairs match augmented-source digest, phase counts, successful collector work, exact save digest/bytes, ordered archive digest/bytes/counts, retired/withdrawal/queue counts, and retained requested heap. Each original harness process also asserts exact save rollback and zero published archive after every failed release. Failed work counters are zero placeholders and are not attempted-work evidence. These checks do not claim every intermediate snapshot or arbitrary program equivalence; the parent's separate correctness gates cover their stated behavior.

Primary retained requested heap before release is 9,252,154 B and after release/drain is 2,948,056 B in both variants. Save bytes remain 1,290,931→298,583; retired records 5,998→0; withdrawals 6,000→2. Ordered output is the same 3,228,566 archive bytes, including unrelated probes, with 6,097 record entries and 6,097 provenance nodes. Growth archive is zero. This optimization changes temporary peak storage; it does not shrink required information, archive output or retained capacity.

The measured result supports retaining the scoped change for review under the registered criteria. It does not authorize merging/publishing, broader optimizations or replacing the still-open chain/retained-capacity work. The high-degree timing observation is a limitation, not a reason to expand or rerun this experiment automatically.

## Preserved evidence

- Full arithmetic and all-phase tables: `run-001/analysis-2026-10-08T19-51-04.703063+00-00.json` and `.md`.
- JSON SHA256: `c5107f616ce6c6b9fbd22235ea8c28bd699e0c912dbbe6fa09663ce2a69fc2dd`.
- Markdown SHA256: `8333472d917bf949a5df026d43430dc9967464ec6dddf5f5f61fe8e7af1f21a5`.
- Raw attempts ledger SHA256: `eec0e7005b1fcea021881c312a4a897f47655a2f53f6a2a12f8f1833d93b0975`.
- `run-001/capture/` retains 32 immutable stdout/stderr pairs, process observations, source checks and 16 equality comparisons; frozen tools, fixtures, executables and source/build receipts are beside them.

The analysis re-read and checked raw stdout, stream hashes, process order/argv, source identity observations and ledger completeness. No process was rerun and no target was redefined.
