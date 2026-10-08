# Renewal occurrence-removal profiling — completed local investigation

The measurements do not establish renewal removal as the highest-value next
optimization. Its instrumented block takes about 3 ms in the largest registered
release case, roughly 5% of whole dispatch. The diagnostic/plain timing difference
is of similar magnitude. Keep stable per-history filtering as a possible follow-up,
but first attribute the larger transaction/departure costs with fewer timing calls.
No optimization or optimization acceptance target was introduced.

Exact review commit: `677ad1365398fe37bb24ff8fb6cfbae82d3ef5b5` on local branch
`codex/renewal-removal-profile`. Diagnostic implementation commit: `1e2374a52a3a0e103ca1342db36a665d6d896c80`.
Their runtime tree is `12547178224bdb4273bf51ae889c512b5cabba86`.
Accepted PR177 baseline remains `055e2af0b411d8385941dd8522201e5afc73cb34`.
This work adds native-only diagnostics and repairs a Windows-sensitive test scanner.
It does not implement renewal batching, change language syntax, or authorize release.

## Measured contribution

Four fresh matched process pairs, mutual cycles=3,000, drain every event. Each
process performs three rejected releases and one successful release after growth,
100 unchanged probes and 100 unrelated mutations. Rejected samples are reduced to
a median within each process, then across four processes; twelve rejections are
not treated as twelve independent sessions. Across-process medians average the
central two values. Within-process even-size summaries retain the original
harness's upper-middle convention.

| Release | Plain apply median, ms | Diagnostic apply median, ms | Renewal block median, ms | Block/apply median share | Block/departure median share |
|---|---:|---:|---:|---:|---:|
| Successful | 57.82955 | 60.75460 | 3.00005 | 4.942% | 6.033% |
| Rejected | 52.87970 | 54.33655 | 2.96290 | 5.516% | 6.222% |

These shares are medians of per-event ratios, not a ratio between unrelated sums
and medians. They are descriptive instrumented observations, not predicted gains.

| Pair | Execution order | Successful plain / diagnostic, ms | Rejected plain / diagnostic, ms | Successful / rejected renewal block, ms |
|---|---|---:|---:|---:|
| 1 | plain / diagnostic | 57.41950 / 60.68810 | 51.94260 / 53.09820 | 2.97700 / 2.92200 |
| 2 | diagnostic / plain | 53.70410 / 58.23660 | 48.61480 / 53.16400 | 2.98090 / 2.94930 |
| 3 | plain / diagnostic | 61.61230 / 63.91940 | 58.26220 / 56.58950 | 3.01920 / 2.97650 |
| 4 | diagnostic / plain | 58.23960 / 60.82110 | 53.81680 / 55.50910 | 3.02790 / 3.02440 |

Successful plain apply spans 53.70410–61.61230 ms; rejected per-process medians
span 48.61480–58.26220 ms. Diagnostic/plain paired-ratio medians are 1.05063 and
1.02685, with ranges 1.03745–1.08440 and 0.97129–1.09358 respectively. The negative
rejected difference in one pair illustrates that this ratio combines observer
effects and host/run variation. It is not a calibrated overhead to subtract.
Four process pairs establish neither stable tails nor a statistical speedup claim.

Every primary successful release and every attempted rejected release performed
5,998 renewal removals, 3,647,564 String equality probes and 5,361,432 shifted String
headers. Estimated shifted-header volume is 128,674,368 bytes per release; it is
not allocated memory or measured memory traffic and excludes String payloads.
One shared renewal-map detachment cloned 3 histories / 6,004 occurrences.
The original rejected collector-work object is a zero placeholder; these attempted
work counts come from the separate recorder outside transactional rollback.

Across the sampled mutual sizes, combined probes plus shifted headers match
N² + 3N − 4; the 1,000-to-3,000 comparison is about ninefold work. This makes
larger bulk populations a reason to revisit batching, not evidence of a measured
speedup. Growth phases in mutual, self and reachable-chain cases have zero
renewal-removal blocks, so this path does not explain their measured growth cost.

Nested successful medians are 1.04840 ms search, 1.36090 ms removal, and 0.21680 ms
shared-map copy; rejected medians are 1.04150, 1.37730 and 0.18670 ms. These clocks
are inside the block timer and include their own measurement effects. They must
not be added to the block again or treated as a precise partition of all work.
The block starts after history lookup and classification. Whole-departure time
includes other departure work and diagnostic bookkeeping; whole apply also
includes subsequent settlement and rollback/publication.

## Smaller and different cases

Each row below is one matched process pair, descriptive only. Block/share columns
refer to the successful diagnostic release; plain apply shows successful/rejected
medians. Complete rejected and all-phase observations remain in the raw files and
independent arithmetic result.

| Fixture | Cycles | Plain successful / rejected, ms | Renewal block, ms | Block/apply | Probes | Shifted headers |
|---|---:|---:|---:|---:|---:|---:|
| release-mutual | 1 | 0.00950 / 0.00930 | 0.00000 | 0.000% | 0 | 0 |
| release-mutual | 2 | 0.02160 / 0.02350 | 0.00070 | 2.682% | 4 | 2 |
| release-mutual | 60 | 1.07080 / 0.95720 | 0.02240 | 2.127% | 842 | 2,934 |
| release-mutual | 300 | 5.47970 / 4.83120 | 0.14100 | 2.711% | 36,866 | 54,030 |
| release-mutual | 1000 | 17.02060 / 16.09180 | 0.51040 | 2.726% | 94,008 | 908,988 |
| release-self | 3000 | 27.32330 / 24.90990 | 1.56570 | 5.240% | 1,823,782 | 2,680,716 |
| high-degree | 60 | 0.26910 / 0.25070 | 0.00640 | 1.924% | 34 | 17 |
| high-degree | 1000 | 0.22250 / 0.20970 | 0.00570 | 2.278% | 34 | 17 |

High-degree repeats a fixed N=16 graph; 60 versus 1,000 cycles is not an increasing
degree experiment. Reachable-chain=3,000 uses initialization and growth without
a final release. All reading-removal counters in this renewal corpus are zero;
this does not characterize reading-stream performance. Focused diagnostic tests
separately cover reading removal and earlier compaction detaching its map.

## Memory remains a separate problem

Every primary successful/rejected additional requested-heap peak is exactly
27,403,251 / 27,403,464 bytes in both fresh builds. This is an equality observation
for this profiling comparison, not a memory improvement. The phase summaries and
independent arithmetic report preserve all additional-heap and retained-heap
differences across the full population. All 76 phase-peak comparisons and 26
retained-heap comparisons have zero delta; no memory acceptance threshold was set.

| Fixture / cycles | Retained before, B | Retained after, B | Save before / after, B | Emitted archive NDJSON, B |
|---|---:|---:|---:|---:|
| release-mutual / 3000 | 9,252,154 | 2,948,056 | 1,290,931 / 298,583 | 3,228,566 |
| reachable-chain / 3000 | 7,811,422 | 7,811,422 | 1,238,709 / 1,238,709 | 45,413 |

The reachable-chain row has no final release; its two columns describe the same
retained end state rather than a failed attempt to collect required records.

Retained heap is measured above the pre-session harness baseline, with sample
buffers and report allocations excluded as documented. Additional dispatch peak
includes transaction copies, archive construction and temporary work; it does not
isolate collector-only temporary allocations. Requested container capacity is
included while allocated; allocator metadata/free lists, stack, OS reservations
and RSS are not measured by the requested-live-byte counter.
Archive bytes are total emitted NDJSON, not a retained archive buffer inside this
draining harness. Moving history to host storage does not eliminate its cost.

The primary save drops from 5,998 retired dynamic records to zero, but required
reachable chains still grow. This experiment changes neither required retention,
conservative pins, retained allocation capacity, nor archive growth. Those remain
active engineering issues. Passing earlier C3 gates was fixture-specific evidence,
not proof of a universal window-only bound.

## Practical recommendation

Do not implement renewal batching as the next substantial latency/memory project
on this evidence alone. Millions of probes and shifts show avoidable work, but
the measured block is a small portion of dispatch and its clocks/bookkeeping
materially perturb the observation. A replacement also has nonzero filtering and
membership costs; removing this entire measured interval is not a valid gain model.
The small fixtures do not establish a user-visible bottleneck in this path.

The smallest useful next task is a bounded phase-level profile of accepted and
rejected bulk departures: transaction preparation, collector/compaction work,
archive construction, final settlement and commit/rollback cleanup. Use a fixed
number of timing boundaries per event, compare fresh ordinary controls, report
nonoverlapping phase totals and any residual, and associate requested allocations
and peaks with those phases. Distinguish where the live-heap peak occurs from
earlier phases that allocated still-live objects; per-phase peaks cannot be summed.
Success means locating the dominant latency and explaining the live allocations
at the peak, with observed instrumentation effects,
before selecting a representation/copying change and registering its performance
target. This is a proposal, not an implementation started under this task.

`source-options.md` records a possible stable filtering change that retains current
ordered renewal vectors, exact identities/reasons, admission timing, saves and
rollback. Its value remains unproven. It does not require weakening retention or
introducing lossy history, and it would not solve required-chain storage.

All primary successful plain releases exceed a concrete 16.7 ms synchronous frame
budget. This 3,000-cycle bulk workload does not fit that budget. Smaller observations
are not a supported-workload or tail-latency guarantee. Native apply excludes later
archive drain/serialization, browser and WASM. Measurements were taken on Windows
11 x64 / Intel Core Ultra 9 275HX with Rust 1.98.1, release, locked dependencies and
the documented features. Agent-controlled builds/tests were idle during capture;
the host and user/background activity were not globally isolated.

## Verification and evidence history

The fresh local checks below passed on the exact review revision. The separate
two-test C006 refresh also passed against the final generated candidate population;
both current CAVEAT audit plans accepted with no problems. Suite counts overlap
and are not added as unique coverage. The ignored native capacity stress test was
not newly run by this profiling-only verification; no fresh release-readiness or
universal compatibility claim is made.

| Check | Result | Scope |
|---|---|---|
| fmt | passed | Formatting of the complete runtime and profiling example. |
| full-clippy | passed | Strict ordinary full-runtime lint; profiling feature disabled. |
| reactive-clippy | passed | Strict ordinary reactive-runtime lint; profiling feature disabled. |
| profile-clippy | passed | Strict lint of the native-only renewal-removal diagnostics, example and tests. |
| full-native | 883 passed, 1 ignored | Full ordinary native regression population with an explicit 16 MiB debug-test stack; diagnostic feature disabled. |
| reactive-native | 871 passed, 1 ignored | Reactive-only ordinary native regression population with an explicit 16 MiB debug-test stack; diagnostic feature disabled. |
| profile-native | 76 passed, 0 ignored | Native diagnostic counter boundaries, reset, reading/renewal removal and rejected-transaction recording, plus library regressions. No benchmark claim from debug tests. |
| withdrawal-profile | 2 passed, 0 ignored | The separate existing withdrawal diagnostic recorder remains usable. |
| lean-staging | 44/44 passed | Lean verification infrastructure and declared-example staging regressions; not a new full Lean proof run. |
| security-staging | 14/14 passed | Dependency-audit staging and security/capability regressions; not a new advisory database audit. |
| kit | 335/335 passed | Developer kit against the clean current ordinary WASM build. |
| independent-wasm | passed | Unchanged externally authored PR176 matrix reused against accepted PR177 and current feature-disabled WASM: finite outcomes/saves/archives/rollback/restore/expected retention, not new independent authorship or performance coverage. |

The unchanged externally authored WASM matrix was reused against actual accepted
PR177/current ordinary artifacts: 22 traces, 4,096 logical steps, 396 rejected
steps, 604 accepted restored-session dispatches and 906 expected-retention checks.
This is reused finite coverage, not newly independent authorship. All 13 native
pairs also match exact save/archive digests and counts plus successful collector
work, including their accepted historical references. Native hashes do not cover
every snapshot/view field; the separate WASM matrix provides that finite coverage.

The initial kit run had 333/334 passing. Its Rust declaration scanner required LF
after an opening brace and missed a CRLF file. Commit677ad13 fixes the scanner,
with explicit LF/CRLF field/serde/enum/duplicate checks, and the fresh kit run passes
335/335. Initial registration run-001 executed no native profiling processes.
It and the initial build/verification evidence are preserved; run-002 kept the same
protocol and population and was registered before its first measurement.

Kit tests then generated six new `.cav` examples after the broad native scan. The
final census guard correctly blocked audit. A focused C006 refresh covers the
1,435 final candidates under both native configurations without replacing the
earlier suites/censuses. Candidate count is not the number of sources that parse
as single-file programs. The first supplemental plan was refused before Cargo
because its artifact list was empty; v2 binds the original nonempty artifact set.
That setup error and its correction are retained too.

Raw arithmetic was recomputed from preserved stdout by a separate Python checker
and agrees with the driver summary. This recalculation is not another timing run.
The accepted-input subset was also exercised through the actual registration
driver, so fresh registration does not require the prior task workspace.

PR174–177 retain their exact heads and remain open, draft and unmerged. Live GitHub
metadata reports all 18 checks passing on each. Those are baseline PR checks;
the new profiling branch was not pushed and has no new CI run or PR. Nothing was
merged, published or released.

## Deliverables and replay

`REPRODUCE.md` explains source recovery through `review-source.bundle`, current
builds, raw arithmetic replay, fresh registration and measurement, and verification
boundaries. `run-002/` contains the frozen contract, actual binaries, all 26 attempts,
raw streams, 13 comparisons and the single original arithmetic summary. The exact
registered contract SHA-256 is
`375482ec4eb031232ca54fb740634b92788bc176697817ff76b52824bd9ed6aa`.

Current builds/public receipts/WASM artifacts are under `repaired-head/`; root
verification files preserve the initial attempt. Use the latest linked
`corpus-refresh-v2-final-check-*.json` for fresh census and both actual audit files.
Private issuer keys, signed stores and signed-receipt CLI logs are excluded from
the deliverable ZIP; unsigned observations and raw command streams are supplied.
Local receipts and matching archives are not authenticated execution history.
