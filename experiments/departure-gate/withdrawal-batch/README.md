# Withdrawal batch extraction: registered measurement protocol

This is a separate representation/removal follow-up from experimental baseline
`c1d8fea1164e9d89edc8ff33af5a3bb608bc37d3`. It changes no withdrawal reason,
eligibility rule, survivor order, archive order, transaction rule or public release
status. The [scope and invariants](../../../docs/design/withdrawal-batch-extraction.md)
govern the implementation. PRs 174–176 remain preserved; merge/release is not
authorized by this experiment.

## Attribution before optimization

At baseline `reactive_departure.rs` searches `withdrawals` with `position` and
removes one matching element for every departing record. The first actual removal
also detaches the shared Arc-backed vector. The transaction retains the old
snapshot. Departure runs before final binding evaluation, so the authored `fail`
event performs the removal work and then rolls it back.

`withdrawal-extraction-profile` enables a separate native-only recorder. A const
thread-local Cell, outside transactional/session/save state, counts attempts,
matches/misses, inferred predicate probes, shifted elements and estimated shifted
header bytes, shared COW detaches and their vector lengths. `position`'s result
implies `index + 1` probes, or the prior vector length on a miss. There is no new
counter in the search predicate. Shift count is `len - index - 1`; estimated bytes
multiply by `size_of::<Withdrawal>()`, excluding String payloads and not claiming
measured memory traffic. The timed original block includes lookup, first COW and
remove; the broader departure timer includes other departure/collector work.

The separate `withdrawal_extraction_profile` example copies the existing event
sequence and allocator accounting, reads the recorder after apply, and retains
accepted and rejected diagnostic samples. Per-block clocks, TLS stores and larger
diagnostic sample records perturb the instrumented process. Diagnostic timings
attribute observed work; they never satisfy the final latency or memory target.
The uninstrumented target harness `collector_profile.rs` stays byte-identical at
SHA-256 `1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357`.

Stage A runs six diagnostic processes: release-mutual at 60/300/1,000 cycles once
and 3,000 cycles three times. Three fresh plain baseline processes pair with the
3,000-cycle diagnostic processes, rotating plain/diagnostic, diagnostic/plain,
plain/diagnostic. Nine processes total. Report complete apply time, withdrawal
block time/share, departure time, exact counts, estimated shift bytes and the
plain/instrumented difference. Do not subtract a guessed clock overhead or treat
an isolated synthetic Vec benchmark as actual dispatch attribution. Production
optimization begins only after this baseline profile is reviewed.

## Candidate diagnostic scope

The frozen Stage A recorder above times one search/removal block per departing
subject. The candidate recorder times one batch: the read-only first-match scan,
any shared-vector COW, stable extraction and sorting of moved withdrawals. The
lexical merge attaching those records to archive entries remains outside
`block_ns`, but inside `departure_ns` and whole apply time. `blocks` now counts
batches; `batch_extractions` identifies that scope, and `input_withdrawals` totals
the original vector lengths. Both added counters are exposed in phase summaries
and small-phase samples. Frozen baseline receipts remain unchanged.

Candidate `probes` directly counts membership decisions, once per original
withdrawal; increments exist only in the native diagnostic feature. This does not
count every BTreeMap/string comparison, COW visit or sorting move. Matches/misses
still count departing subjects with/without withdrawals. Estimated shifted headers
count survivors after the first removal, excluding COW, sorting and String
payloads. Diagnostic block timings include probe-counter and sorting overhead;
the unchanged target harness disables this diagnostic feature. For W original
withdrawals, D departing records and K removed withdrawals, the candidate performs
O(W log D) membership work, O(K log K) sorting and uses an O(K) temporary record
buffer. These counts alone establish neither a latency gain nor a lower peak.

## Frozen target: eight balanced pairs

The final target is the unchanged `release-mutual.cav`, 3,000 cycles, drain interval
1, 100 unchanged and 100 unrelated probes. Each original-harness process performs
three final-binding rejections, then one successful release. Eight fresh baseline
and eight fresh candidate processes run serially on a quiet machine. Odd pairs
run baseline/candidate, even pairs candidate/baseline: four pairs in each order.

Successful latency is each process's `release.median_ns` (one observation).
Rejected latency is each process's `failed_release.median_ns` (median of its three
attempts). For each phase separately, both requirements must hold:

- `median(C) / median(B) <= 0.8` across the eight process values;
- `median(C_i / B_i) <= 0.8` across the eight matched pair ratios.

Even-sized medians use the arithmetic mean of the two central values. No pooled
median of 24 rejected observations is reconstructed from eight summary medians.
Report every pair, both ratios, ranges and sample counts. These are finite native
timings with allocator/counter overhead, not reliable population-tail estimates or
browser/WASM frame-budget evidence.

In **every primary pair**, require candidate successful peak no greater than its
matched baseline successful peak, and candidate rejected peak no greater than
its matched baseline rejected peak. Fields are
`max_dispatch_additional_heap_bytes` in the respective phase. No tolerance,
averaging, or maximum-over-all-trials substitution is allowed. This no-increase
gate is scoped to successful/rejected release-mutual at 3,000 cycles. All other
phase/size absolute peaks must still be reported, and any growth/small-case
increase prominently identified; a pass is not a universal no-memory-regression
claim. Requested heap excludes allocator internals, stacks and RSS. Whole-dispatch
peaks include transaction copies, departure, collector work and archive construction.

## Supplementary coverage: 40 target processes total

In addition to the 16 primary processes, run one matched pair for each row/size:

| Existing fixture | Cycles | Purpose |
| --- | --- | --- |
| release-mutual | 1, 2, 60, 300, 1,000 | empty/tiny batch and increasing sizes |
| release-self | 60, 3,000 | self reason and large one-subject release |
| high-degree (fixed N=16) | 60, 1,000 | shared reason/holders; not a degree sweep |
| reachable-chain | 3,000 | required history retained; unrelated miss path |
| self, mutual | 60 | ordinary frequent small departures |

All processes retain the original 100+100 probes; report their latency and peak
separately. Few departures from a large withdrawal vector can have a different
cost from bulk release. There are 20 target pairs / 40 processes, plus the nine
separate Stage A diagnostic/control processes. No additional repetition or changed
acceptance rule is chosen after viewing results. Preserve every attempt, failure
and raw output. A revised implementation uses a new result directory; old failures
are never erased or rebased to a favorable baseline.

## Identity, equality and commands

Baseline executable SHA-256 is
`c206c83292dc10e908246a3bace2d6d0dc3a1dc9b4a8b6c0c9126bc1edfe32fd`.
It was built at f171d41 and its runtime tree
`907b6ae90b73bf536bc78c9346d4fb2e8f8dcdd3` is identical at the frozen c1d8fea
baseline. Both target builds use release, locked Rust 1.98.1, no default features,
and `collector-metrics`; the diagnostic build additionally selects its separate
example/feature. Freeze executable SHA, clean revision, runtime tree, selected
runtime/game/toolchain file hashes and exact build argv before execution.

The driver verifies existing fixture hashes and freezes copies only in scratch or
the external bundle; this work adds no duplicate `.cav` source fixtures. B/C must
match exact save SHA/bytes, ordered archive NDJSON SHA/bytes/counts, before/after
inventory and phase collected counts. The original harness asserts each rejected
release preserves the exact save and publishes no archive. Full snapshots, views,
refusal details, survivors/withdrawal order and restored following behavior are
checked independently by regressions and actual-WASM traces, not inferred from
performance results.

```text
node experiments/departure-gate/withdrawal-batch/native-profile.mjs register --output=FRESH_OUTPUT --baseline-exe=PRESERVED_EXE
node experiments/departure-gate/withdrawal-batch/native-profile.mjs freeze --output=OUTPUT --kind=diagnostic --exe=DIAGNOSTIC_EXE --revision=CLEAN_DIAGNOSTIC_REVISION
node experiments/departure-gate/withdrawal-batch/native-profile.mjs run-diagnostic --output=OUTPUT
node experiments/departure-gate/withdrawal-batch/native-profile.mjs freeze --output=OUTPUT --kind=candidate --exe=CANDIDATE_EXE --revision=CLEAN_CANDIDATE_REVISION
node experiments/departure-gate/withdrawal-batch/native-profile.mjs run-target --output=OUTPUT
node experiments/departure-gate/withdrawal-batch/native-profile.mjs summarize --output=OUTPUT
```

The driver records registration and refusal-to-overwrite output, not build or
release approval. Run after the coordinator confirms no concurrent task builds/tests; the user has
already authorized these measurements. Successful and failed stdout/stderr are
preserved as exclusive sidecar files with hashes and exit status. Registration
requires an entirely new directory, even after a partial setup. Stage A results, target results, full semantic checks and final CI are
separate receipts with separate scopes.

## Recorded native result

The [frozen result](results/windows-8a161dc/REPORT.md) records clean candidate
`8a161dcdfdc8a2a5b3ca43b4183552401002cca7` against the accepted c1d8fea baseline.
Both registered latency criteria pass: ratio-of-process-medians reductions are
22.0580% successful and 22.9386% rejected; paired-median reductions are 22.0731%
and 23.1418%. Every primary additional requested-heap peak is unchanged at
27,403,251 / 27,403,464 bytes. All 20 native matched save/archive comparisons pass.

All 49 attribution/control/target attempts and their raw streams are retained.
Small-case slower timings, attribution overhead, absolute retained/archive bytes
and finite measurement limits are included, not excluded from the report.
The supplementary analysis scripts retain their original task paths; the portable
registered driver above reproduces fresh runs in a new output directory.
The native result is separate from final-head semantic, integration and CI checks.
The evidence-only review commit does not change the measured runtime tree
`aec511bf3ee147a79391f90674446eff9c7f393e`.
