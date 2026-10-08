# Withdrawal batch extraction: frozen target result

Candidate `8a161dcdfdc8a2a5b3ca43b4183552401002cca7` passes the registered
3,000-cycle mutual-release latency and peak-memory gates against frozen baseline
`c1d8fea1164e9d89edc8ff33af5a3bb608bc37d3`. All 40 target processes / 20 matched
pairs completed. All native equality checks passed. No attempt was rerun,
discarded, or retuned. This report records measured native performance; final-head
semantic, integration, package and CI receipts are separate requirements.

## Profile first, then change only extraction

Before production optimization, clean diagnostic revision
`cb5faadfa2a9b40a11764c303f475fffb981a9f8` ran six actual-dispatch diagnostic
processes and three plain controls. At 3,000 cycles, each successful or rejected
release found 5,998 withdrawals through **8,137,067 search probes**, shifted
**9,865,930 Withdrawal headers** (789,274,400 estimated header bytes), and detached
one shared vector containing 6,000 withdrawals. The original block's median
diagnostic share was **20.91% of successful apply** and **22.22% of rejected
apply**. Full attribution and individual rejected samples are in `ATTRIBUTION.md`
and `attribution-summary.json`.

The diagnostic clocks, TLS stores and sample representation affect timing:
diagnostic process medians were 2.37%/3.22% higher than the plain controls for
success/rejection. Those differences are not a pure clock-cost calibration.
Diagnostic timings were never used for target acceptance. The measurements
identified repeated search/movement as a scoped cost; they did not promise a 20%
full-dispatch gain or provide an exact additive decomposition of later speedup.

The candidate performs one stable extraction of matching withdrawals, sorts only
the removed batch by evidence identity, and attaches records during the existing
ordered departure pass. A no-match precheck preserves sharing. Survivor order,
first reasons, eligibility and final archive order remain governed by the same
runtime rules. The implementation and independent semantic review have their own
test receipts; performance equality here covers the registered traces.

## Frozen identity and unchanged target harness

| Artifact | SHA-256 |
| --- | --- |
| Plain baseline executable | `c206c83292dc10e908246a3bace2d6d0dc3a1dc9b4a8b6c0c9126bc1edfe32fd` |
| Diagnostic executable | `4c7791a31239195acd17075993c1acec97ae29ecf35a1205d344a64acbcc8254` |
| Candidate target executable | `9f5a0a5af2b4073849d30fd7070380eb1341ea4ae9206828aa7fd755cb207909` |
| Original `collector_profile.rs` | `1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357` |
| Registered contract | `53044ddfe5bab62cfd28ac484a27412a0c68f3e521f9cff7c640b1f7a28f83aa` |

The baseline executable was built at f171d41; its runtime tree
`907b6ae90b73bf536bc78c9346d4fb2e8f8dcdd3` is identical at c1d8fea. Candidate
runtime tree is `aec511bf3ee147a79391f90674446eff9c7f393e`.
The target build completed from clean tracked source with rustc 1.98.1, using
`cargo build --release --locked --manifest-path runtime/Cargo.toml --example
collector_profile --no-default-features --features collector-metrics`.
It **does not enable `withdrawal-extraction-profile`**. Exact command, compiler,
stdout/stderr, source hashes and executable identity are in `candidate-build.json`;
141 raw selected source files are preserved in the external `candidate-source/`
snapshot with a hash manifest.

## Both latency criteria and every primary peak gate pass

Eight fresh pairs ran serially after the coordinator confirmed no concurrent task
builds/tests. Four pairs used baseline/candidate order and four candidate/baseline.
The original harness used drain interval 1, 100 unchanged probes and 100 unrelated
renewal probes, followed by three rejected releases and one successful release.

For each phase, both `median(C) / median(B) <= 0.8` and
`median(C_i / B_i) <= 0.8` were registered in advance. Successful process values
are single releases. Rejected process values are medians of three attempts;
no pooled median of the 24 rejected attempts is reconstructed from summaries.
Even-sized medians average the two central values.

| Phase | Baseline median ms | Candidate median ms | Ratio of medians / reduction | Median paired ratio / reduction | Result |
| --- | ---: | ---: | --- | --- | --- |
| Successful release | 73.00265 | 56.89975 | 0.779420 / **22.0580%** | 0.779269 / **22.0731%** | PASS |
| Rejected release | 68.76400 | 52.99050 | 0.770614 / **22.9386%** | 0.768582 / **23.1418%** | PASS |

| Pair | Successful B → C, ms | Rejected process median B → C, ms |
| --- | --- | --- |
| 1 | 76.2470 → 52.4589 | 70.3056 → 50.3948 |
| 2 | 74.7056 → 57.1339 | 73.6199 → 54.0882 |
| 3 | 72.0271 → 58.6703 | 70.3898 → 55.9294 |
| 4 | 70.7410 → 56.0666 | 67.0018 → 53.8841 |
| 5 | 73.9782 → 56.6656 | 73.4196 → 51.3695 |
| 6 | 70.0496 → 57.9388 | 67.2224 → 52.3292 |
| 7 | 70.3319 → 58.4387 | 65.8238 → 53.6518 |
| 8 | 76.3057 → 56.6214 | 66.8051 → 50.6860 |

Every one of the eight pairs has exactly unchanged additional dispatch peaks:
**27,403,251 bytes successful** and **27,403,464 bytes rejected**. Each phase in
each pair independently passes `C <= B`; no averaging or aggregate maximum hid
an increase. These are whole-dispatch peaks, including transaction copies,
departure, collector work and archive construction, not isolated collector scratch.

Successful observations ranged 70.0496–76.3057 ms baseline and 52.4589–58.6703 ms
candidate. Rejected process medians ranged 65.8238–73.6199 and 50.3948–55.9294 ms.
Some individual pair reductions were below 20%; all pairs remain in both aggregate
criteria. The result is the registered finite median comparison, not a claim about
every attempt, a population p99, or universal hardware speedup.

## Supplementary sizes, small cases and memory

The other 24 processes cover release-mutual at 1/2/60/300/1,000 cycles;
release-self at 60/3,000; fixed N16 high-degree at 60/1,000; required reachable
chain at 3,000; and ordinary self/mutual departure at 60. Every process includes
the unchanged and unrelated probes. There were **no phase peak increases** in
the registered matched processes: all reported phase peaks are exactly equal.
Before/after retained requested heap is also unchanged in every matched case.
This is finite coverage, not a universal memory-regression proof.

Small cases do not show a universal latency gain. The one-pair mutual-release
observations at 60 cycles were 0.9217→0.9775 ms successful and
0.8517→0.9188 ms rejected; at 300 they were 5.0182→5.0704 and
4.7235→4.8232 ms. Ordinary self/mutual growth medians at 60 cycles were
0.0114→0.0123 and 0.0213→0.0236 ms. Fixed N16 high-degree release at 1,000
cycles was 0.2051→0.2510 ms. These slower observations are retained; they are
single-pair diagnostics and do not establish stable small-case regression rates.

The 1,000-cycle mutual-release pair improved 18.7846→17.4124 ms successful and
16.7376→15.7108 ms rejected; the 3,000-cycle self-release pair improved
29.3925→25.5849 and 26.9366→23.1352 ms. Required-chain 3,000-cycle growth peak
remained 8,574,441 bytes and retained heap 7,811,422 bytes. Its one-pair final-decile
growth median was 13.8907→13.8021 ms. All absolute phase peaks, medians, p99/max,
observed ranges and retained values are in `TARGET-SUPPLEMENT.md` and its JSON.

## Equality, retention and external archive

All 20 pairs have exact before/after save SHA-256 and byte counts, ordered archive
NDJSON SHA-256/bytes/counts (including growth subtotals), retired/withdrawal/
undrained counts, phase sample counts and collected counts. Each rejected release
also passes the unchanged harness's exact-save rollback and no-archive-publication
assertions. Complete snapshots, views, detailed refusal shape, staged survivor
order and restored following behavior require the separate native/WASM regressions.

At the primary size, retained heap remains **9,252,154 bytes before** and
**2,948,056 after release/drain** in both versions, with zero retired dynamic
records and zero undrained items afterward. Save sizes remain 1,290,931/298,583
bytes. Emitted archive remains 3,228,566 bytes, 6,097 records and 6,097 provenance
nodes; totals include the 99 unrelated departures. Draining moves output outside
runtime retention and does not erase external archive storage.

At fixed N16/1,000, post-release heap is unchanged at 136,271 bytes while total
emitted archive is 14,648,806 bytes. Repeating this fixed shape is not a sweep of
degree. The batch change meets its latency goal without reducing retained capacity
or external archive size; growing required histories keep their required records.

## Bounded conclusion and receipts

The frozen candidate meets the registered native optimization target. Final-head
semantic, integration, package and CI checks remain separate receipts; this result
does not authorize merge or release or change the status of PRs 174–176. Native
apply excludes snapshot serialization and browser/WASM work. Requested Rust heap
excludes allocator internals, stacks and RSS. No other local task builds/tests ran
during capture, but OS load, CPU frequency and affinity were not fully controlled.

`contract.json`, `candidate.json`, build receipts, `attempts.jsonl`, all 40 raw
target JSON results, 80 stdout/stderr sidecars and 20 comparisons are preserved.
`summary-2026-10-08T07-00-12.480Z.json` and the independently computed
`target-supplement.json` agree on all target values. The latter also contains every
supplementary value and a raw-result/input hash manifest. The superseded run-001
registration predates measurements and remains untouched with its stated reason;
all nine attribution/control and 40 target attempts are retained under run-002.
