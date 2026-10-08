# Collector index compaction: frozen native result

Candidate `f171d41e9beae588c480f09fc5c03afc7487efb4` satisfies both registered
memory targets in **all three** fresh matched 3,000-cycle reachable-chain trials.
All **99 native executions / 33 triples** completed; all registered save,
ordered archive, inventory and collected-count comparisons passed. No result was
rerun, erased, or substituted. This is the native measurement result; final-head
WASM equivalence, package/runtime gates and CI are recorded separately.

## Identity and method

| Variant | Revision | Executable SHA-256 |
| --- | --- | --- |
| Original F | `f5ec8294efe2be24705f234ef75e5f5459aa5e89` | `27cd7b3e68a21a0633e2abe8effc7568529fca1ac3555e005df41fcc4453b8c4` |
| Repaired B | `3d77aa27bcffa16f39818b2684e9afb3b102ed17` | `c0bf892c2cb618b377c38a14206bc56244d891c561f0684fceeb8b0aa8b0a9c5` |
| Compacted C | `f171d41e9beae588c480f09fc5c03afc7487efb4` | `c206c83292dc10e908246a3bace2d6d0dc3a1dc9b4a8b6c0c9126bc1edfe32fd` |

The original `collector_profile.rs` harness is unchanged, SHA-256
`1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357`.
Candidate runtime Git tree is `907b6ae90b73bf536bc78c9346d4fb2e8f8dcdd3`.
`candidate.json` records every frozen runtime/game/toolchain input hash, source
revision and command. The separate build observation records the successful clean
native release build. Preserved baseline hashes were checked before reuse.

Build settings were the original release profile, Rust 1.98.1, LTO and one
codegen unit, `--no-default-features`; B/C also use `collector-metrics`.
Original F lacks that ABI, so its zero work counters mean unavailable. The five
original fixtures retain their original hashes; the additional high-degree source
is separately hashed. Every process used drain interval 1, 100 unchanged probes
and 100 unrelated-renewal probes. Release modes add three rejected attempts and
one successful release. Trial orders rotate F/B/C, B/C/F, C/F/B.

The native processes ran serially after an explicit quiet-machine signal. No
local builds/tests ran during capture; OS load and CPU affinity were not otherwise
controlled. Raw receipts preserve argv, timestamps, executable/fixture identities
and every harness output. The registered matrix and definitions are unchanged.

## Both primary targets pass

For each metric and trial, the required inequality is
`C - F <= 0.5 * (B - F)`, with `B > F`. The denominator is the repaired
collector's added cost above original F, not its total heap. These are fresh
measurements, not the historical forecast values reused as denominators.

| Metric, bytes | F | B | C | B − F | C − F | Candidate ceiling | Added-cost reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Retained after growth/probes/drain | 6,004,653 | 13,413,126 | 7,811,422 | 7,408,473 | 1,806,769 | 9,708,889 | **75.6121%** |
| Maximum growth-dispatch additional heap | 6,139,810 | 14,176,145 | 8,574,441 | 8,036,335 | 2,434,631 | 10,157,977 | **69.7047%** |

Every byte value above was identical across all three matched trials:
trial 1 retained **PASS**, peak **PASS**; trial 2 retained **PASS**, peak **PASS**;
trial 3 retained **PASS**, peak **PASS**. Total B-to-C reduction is 5,601,704
bytes for each metric. The target percentages specifically use added cost.

The chain still retains all 5,998 required retired dynamic records and its exact
1,238,709-byte save. Growth collects none. Both implementations perform
8,997,000 vertex visits and 26,991,000 edge visits over 3,000 growth events;
compaction changes representation cost without changing this traversal work.
The 100 unrelated probes have the same 4,865,371-byte additional peak in F/B/C.

## Timings are descriptive, separately from allocation gates

Values below are native `apply` milliseconds, excluding snapshot serialization.
Each chain cell is that process's phase median; the final-decile phase contains
the last 300 growth dispatches. Allocator accounting and B/C counters are enabled.

| Trial | All growth B → C | Final decile B → C | Unrelated B → C |
| --- | --- | --- | --- |
| 1 | 8.7944 → 8.1895 | 19.7771 → 19.8704 | 3.0957 → 4.7689 |
| 2 | 7.8257 → 7.8393 | 17.0557 → 15.2257 | 3.1026 → 3.0247 |
| 3 | 8.3329 → 7.5147 | 19.9017 → 14.9786 | 3.3265 → 3.1138 |

Median of these per-process medians is 8.3329→7.8393 ms for all growth,
19.7771→15.2257 ms for its final decile, and 3.1026→3.1138 ms for unrelated
events. Three paired trials do not establish a stable population tail or a
universal speedup. In particular, the 3,000-cycle mutual workload's all-growth
median-of-medians increased **0.0201→0.0247 ms** (4.6 microseconds); its observed
trial ranges were 0.0194–0.0238 and 0.0235–0.0254 ms. Full phase medians, p99,
maxima, work counts and ranges for every registered size are in the summary.

## Successful and rejected final-root releases

At `release-mutual/3000`, successful additional peak is
**31,128,319→27,403,251 bytes**, and rejected peak is
**31,128,532→27,403,464 bytes**, identically in every trial.
These are whole-dispatch peaks, including transaction copies, collector work,
ordinary departure and archive construction; they do not isolate collector scratch.

| Trial | Successful latency B → C, ms | Median of three rejected attempts B → C, ms |
| --- | --- | --- |
| 1 | 78.8077 → 80.2518 | 70.7241 → 68.6603 |
| 2 | 95.0654 → 77.1321 | 90.3961 → 73.3768 |
| 3 | 93.5230 → 94.6491 | 89.0125 → 90.2335 |

Median successful latency is 93.5230→80.2518 ms, with observed ranges
78.8077–95.0654 and 77.1321–94.6491 ms. Median of rejected-attempt medians is
89.0125→73.3768 ms, with ranges 70.7241–90.3961 and 68.6603–90.2335 ms.
There are only three successful releases and nine rejected attempts per variant;
these are descriptive samples, not reliable p99 estimates.

Retained heap before release is 14,470,210→9,252,154 bytes; after successful
release and drain it is **2,949,924→2,948,056 bytes**, with zero retired dynamic
records and zero undrained items. Post-release retained heap is therefore almost
unchanged. Saves are exactly 1,290,931 bytes before and 298,583 after, identical
between B/C. Total emitted archive is exactly 3,228,566 bytes, 6,097 records and
6,097 provenance nodes. This includes the 99 unrelated departures; it is not
release-only archive size. Draining moves this output outside runtime retention,
and does not make the external archive disappear.

The separate one-trial `release-self/3000` diagnostic has successful peak
15,551,613→13,688,551 and rejected peak 15,551,826→13,688,764 bytes. Successful
latency is 30.0382→30.0970 ms; rejected median is 26.9411→28.6236 ms. Post-drain
heap is 1,488,288→1,487,354 bytes. No stable-tail claim is made for this pair.

## Fixed high-degree case and equality scope

The additional fixture has 16 subjects sharing one self-withdrawn reason and
16 overlapping state holders. Repeating it at 60/300/1,000 cycles is a fixed-shape
stress case, not a sweep of degree or an unbounded degree claim. At 1,000 cycles
all three trials report retained heap **238,099→232,511 bytes**, growth peak
**276,120→269,972**, and post-release/drain heap **136,271→136,271**.
Median all-growth latency is 0.8350→0.8008 ms; final-decile median is
0.7471→0.8555 ms, with overlapping observed trial ranges. Total ordered archive
is **14,648,806 bytes**, 34,082 records/nodes: runtime retention remains small
while external archive output grows. Before/after saves are 10,009/2,615 bytes
and retired dynamic counts are 17/0, equal between B/C.

Across all 33 triples, B/C have identical before/after save SHA-256 and byte
counts, ordered archive NDJSON SHA-256 and counts (including growth-only
subtotals), retired/withdrawal/undrained counts, phase sample counts and collected
counts. Reachable-chain saves also match original F. The unchanged harness
asserts exact save rollback and no archive publication for every rejected release.
The additional full-snapshot/view/outcome/save/ordered-archive WASM traces and
restore branches are a separate finite semantic check, not part of these native
allocation measurements; their final receipt must be consulted independently.

## Bounded recommendation and verification status

Accept this compaction as satisfying the registered memory objective, subject to
the separate final semantic and integration gates. It improves growing qualified
history storage and mutation peaks while preserving the native observable results
in this matrix. It also preserves the collector's bounded self/mutual behavior:
at 3,000 cycles candidate retained heap is 14,541/15,730 bytes respectively.
It does not establish browser frame budgets, every application workload, arbitrary
graph degree, or bounded external archives. Growing required chains still require
work proportional to the visited component; successful and rejected bulk release
still produce tens-of-milliseconds native samples and large transaction peaks.

The implementation's 130 focused tests, Clippy and formatting passed before the
freeze, as recorded by the implementation owner. This report independently records
99 successful native processes and their equality/target assertions. Full
final-head gates, actual-WASM equivalence, adapters/games, package checks and CI
are forthcoming separate receipts, and are not implied by the native pass.

`supplement-summary.json` contains every per-trial absolute measurement, timing
range and the raw-file/input/executable SHA-256 manifest; `supplement-summary.md`
contains all sizes. `contract.json`, `candidate.json`, `attempts.jsonl` and `runs/`
preserve registration, source freeze and all command/output receipts. The
registered `summary-2026-10-08T04-56-37.053Z.json` reports complete/pass true.
Requested Rust heap excludes allocator internals, stacks and RSS; the registered
B−F comparison is a cost delta, not direct isolation of every collector allocation.
