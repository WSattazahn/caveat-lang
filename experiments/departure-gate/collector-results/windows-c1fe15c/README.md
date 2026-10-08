# Windows collector draft evidence — c1fe15c

This directory records the separate, unpublished withdrawal collector draft at
`c1fe15cfcb1cac6c069ba5b00f1c7238df7e9853`, compared with preserved baseline
`f5ec8294efe2be24705f234ef75e5f5459aa5e89`. Both native profile records identify
clean source. Full and reactive WASM were also built from clean `c1fe15c`;
[build-info.json](build-info.json) records their hashes, source fingerprint,
compiler and Windows host. The later commit adding this README and its receipts
is documentation/evidence delivery, not the measured runtime revision.

The [draft contract and C01–C20 evidence map](../../../../docs/design/withdrawal-collector-draft.md)
state the provisional semantics and remaining gates. Draft PR #175 is stacked on
unchanged PR #174. These records authorize neither merge nor release and do not
establish universal boundedness.

## Provenance and limits

The native profiler uses the same checked harness on both revisions. Harness
SHA-256 is `1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357`;
baseline executable SHA-256 is `27cd7b3e68a21a0633e2abe8effc7568529fca1ac3555e005df41fcc4453b8c4`,
candidate executable SHA-256 is `b6f6dc591a29840ddd054ecd9c7f9002741833a358ab398464c4ccd93b1748da`.
The reusable baseline extraction is verified byte-for-byte against the pinned
Git blobs before building. No executable is included in this evidence directory.
Each report retains source, fixture and executable hashes and measurement times.

Native counts are requested Rust heap bytes, excluding allocator metadata,
stacks and RSS. Dispatch peaks include transaction copies, ordinary departure,
archive construction and collection; they are an upper bound, not an isolated
measurement of collector-only temporary bytes. Work-set counts are actual entry
counts, not byte estimates. Counter/allocator overhead is included.

Timing comparisons are descriptive Windows single-host observations. Other
builds/tests ran during parts of collection, so concurrent CPU/disk load can
increase noise and differs between trials. Native `apply` excludes snapshot
serialization and is not a browser/WASM latency claim. Each final release has
one sample per trial: the three samples below are not a reliable population p99.
Do not turn these measurements into a constant-time or universal memory claim.

## Population and comparison

Five workloads (self, mutual, reachable-chain, release-self, release-mutual)
run at 60/300/1,000/3,000 cycles, with three trials each and 100 unchanged plus
100 unrelated single-window-renewal probes. Two extra self/mutual 1,000-cycle
runs delay draining until growth ends. [comparison.json](comparison.json)
records **62 passing paired comparisons**: source/harness agreement, exact
reachable-chain save agreement, zero remaining retired dynamic records for
isolated/released groups, and drained queues. It is not a substitute for the
registered no-window/outcome/C3 population gates.

Retired dynamic counts exclude bare declaration records; “zero” here does not
claim an empty snapshot or zero allocated capacity. The native harness injects
probe events and an unrelated renewable source, so its save sizes must not be
mixed with the bare-source WASM supplement below. Heap/save entries are equal
across the three trials for these rows; timing data stays per trial in raw JSON.

| Workload | Cycles | Candidate retired dynamic records | Save bytes baseline / candidate | Requested retained heap bytes baseline / candidate |
| --- | ---: | ---: | ---: | ---: |
| self | 60 | 0 | 12,076 / 947 | 70,950 / 15,445 |
| self | 300 | 0 | 59,693 / 955 | 314,177 / 15,460 |
| self | 1,000 | 0 | 199,712 / 965 | 1,067,416 / 15,475 |
| self | 3,000 | 0 | 619,712 / 965 | 2,998,952 / 15,475 |
| mutual | 60 | 0 | 23,401 / 1,143 | 135,996 / 17,538 |
| mutual | 300 | 0 | 118,635 / 1,159 | 615,618 / 17,568 |
| mutual | 1,000 | 0 | 398,671 / 1,177 | 2,120,664 / 17,598 |
| mutual | 3,000 | 0 | 1,238,671 / 1,177 | 6,003,600 / 17,598 |
| reachable-chain | 60 | 118 | 23,433 / 23,433 | 139,399 / 287,238 |
| reachable-chain | 300 | 598 | 118,670 / 118,670 | 618,650 / 1,350,926 |
| reachable-chain | 1,000 | 1,998 | 398,709 / 398,709 | 2,127,581 / 4,586,798 |
| reachable-chain | 3,000 | 5,998 | 1,238,709 / 1,238,709 | 6,004,653 / 13,413,126 |

The reachable chain retains every required true reason. At 3,000 cycles its
save remains exactly 1,238,709 bytes in both versions, while candidate requested
retained heap is 13,413,126 versus baseline 6,004,653 bytes. Indexing adds a real
cost where collection cannot reclaim anything.

Beside that chain, 100 unrelated renewal events visit zero collector region
vertices/edges. Across the three trials their candidate median is
6.052–7.949ms (baseline 4.019–6.739ms), with 7,652,584 additional requested heap
bytes at peak (baseline 4,865,371). This exposes whole-event/index-copy costs
that region counters do not capture. Unchanged-event candidate medians were
0.9–1.1 microseconds; that observation does not describe mutation cost.

## Final root release costs

Each release case retains groups through an independent root, attempts three
final-binding failures while asserting an unchanged save and no new archive,
then releases the root in one accepted event. Original reasons remain archived.
No collector work is deferred and no new work-limit refusal is added.

| Workload | Cycles | Three release samples (ms) | Maximum additional requested heap bytes | Collected records | Peak work-set entries |
| --- | ---: | --- | ---: | ---: | ---: |
| release-self | 60 | 0.709, 0.607, 0.683 | 313,541 | 59 | 118 |
| release-self | 300 | 3.834, 3.763, 3.686 | 1,575,437 | 299 | 598 |
| release-self | 1,000 | 14.532, 12.377, 12.626 | 5,299,213 | 999 | 1,998 |
| release-self | 3,000 | 44.742, 41.496, 30.879 | 15,551,613 | 2,999 | 5,998 |
| release-mutual | 60 | 0.917, 1.105, 0.896 | 638,495 | 118 | 296 |
| release-mutual | 300 | 4.874, 5.969, 5.319 | 3,120,487 | 598 | 1,496 |
| release-mutual | 1,000 | 26.930, 21.412, 25.369 | 10,559,151 | 1,998 | 4,996 |
| release-mutual | 3,000 | 97.888, 89.876, 115.684 | 31,128,319 | 5,998 | 14,996 |

At 3,000 mutual cycles the same-event release takes 89.876–115.684ms and peaks
at 31,128,319 additional requested heap bytes. That is an explicit contract
tradeoff, not a constant-dispatch result. The precise whole-event counts and
failed-release costs remain in [candidate.json](candidate.json).

## Archive storage and last-event effects

The native profile streams drained archive items into a byte-count/hash sink
and discards them. The following growth-only NDJSON sizes exclude the later
side-history probes. Retaining exact host history still requires growing
external storage even when a live save stays small.

| Workload | Cycles | Departed records | Provenance nodes | Growth archive NDJSON bytes |
| --- | ---: | ---: | ---: | ---: |
| self | 60 | 59 | 59 | 30,060 |
| self | 300 | 299 | 299 | 154,910 |
| self | 1,000 | 999 | 999 | 520,320 |
| self | 3,000 | 2,999 | 2,999 | 1,586,320 |
| mutual | 60 | 118 | 118 | 60,120 |
| mutual | 300 | 598 | 598 | 309,820 |
| mutual | 1,000 | 1,998 | 1,998 | 1,040,640 |
| mutual | 3,000 | 5,998 | 5,998 | 3,172,640 |

For both self and mutual at 1,000 cycles, the per-event and late-drain profile
rows have identical final save and archive NDJSON hashes. [release-effects.json](release-effects.json)
separately records eight actual-WASM release/drain/exact-restore/following-no-op
rows using the clean reactive artifact. These use the bare release fixtures,
not the native profiler's added events/source, and report JSON serialization,
not requested heap.

| Bare-source workload | Cycles | Immediate save bytes | Departure effects | Save bytes after accepted no-op | Released archive JSON bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| self | 60 | 3,303 | 59 | 479 | 30,136 |
| self | 300 | 15,034 | 299 | 489 | 155,422 |
| self | 1,000 | 49,345 | 999 | 499 | 523,528 |
| self | 3,000 | 149,345 | 2,999 | 499 | 1,591,528 |
| mutual | 60 | 6,323 | 118 | 675 | 60,271 |
| mutual | 300 | 29,783 | 598 | 693 | 310,843 |
| mutual | 1,000 | 98,403 | 1,998 | 711 | 1,047,055 |
| mutual | 3,000 | 298,403 | 5,998 | 711 | 3,183,055 |

In the 3,000 mutual case, zero retired dynamic records coexist with a
298,403-byte immediate save because 5,998 departure effects describe the last
accepted event. The next accepted no-op leaves 711 serialized bytes. The
native profiler's corresponding immediate save is 298,583 bytes because of its
extra source/state. Clearing effects earlier would change observable/saveable
last-event behavior. [release-memory.json](release-memory.json) now supplies eight native allocation-state
checks after effects clear and after restoring the identical save. Every archive
buffer is dropped before checkpoints; checkpoint structs retain no heap. The
runtime/build inputs match `c1fe15c` before and after the run, with the same
standard runtime build fingerprint as [build-info.json](build-info.json).
The supplemental harness/driver are later diagnostic additions, separately
hashed in the report; they do not change the runtime source or claim that the
whole later documentation/harness commit was measured as `c1fe15c`.

| Native supplemental workload | Cycles | Requested heap after no-op/drain | After identical-save restore and dropping original | Identical logical save bytes |
| --- | ---: | ---: | ---: | ---: |
| release-self | 60 | 70,794 | 40,039 | 481 |
| release-self | 300 | 181,401 | 40,055 | 491 |
| release-self | 1,000 | 544,744 | 40,071 | 501 |
| release-self | 3,000 | 1,353,448 | 40,071 | 501 |
| release-mutual | 60 | 104,279 | 42,954 | 677 |
| release-mutual | 300 | 325,493 | 42,986 | 695 |
| release-mutual | 1,000 | 1,052,179 | 43,018 | 713 |
| release-mutual | 3,000 | 2,669,587 | 43,018 | 713 |

Each row has zero retired dynamic records and zero effects at both checkpoints,
identical save hashes across restore, and zero residual requested bytes after
dropping the final session. Nevertheless the 3,000 mutual case retains
2,669,587 requested bytes before restore versus 43,018 after it; self retains
1,353,448 versus 40,071 bytes. Source inspection identifies retained vector
capacities in the departure paths; this is resident allocation cost, not retained
logical records or evidence of a session allocation surviving destruction. The
probe does not isolate each container's contribution and does not recommend or
implement automatic save/restore. Capacity management is a separate design choice.

This harness injects `collector_memory_noop`; the WASM supplement injects
`collector_following`. Accordingly the native mutual save is 713 bytes and the
WASM save 711 bytes at 3,000 cycles. These distinct source identities must not be
combined as a byte-compatibility comparison. The corrected diagnostic counts an
omitted effects field as zero bytes; its earlier four-byte `null` diagnostic
remains only in private initial receipts and is not used here.

## Native gates and capacity boundary

[native-summary.json](native-summary.json) records **869 full-profile and 857
reactive-only tests passed**, zero failed, one ignored large capacity test per
profile, and 82 binaries per profile. Strict Clippy for both profiles and
formatting passed. Debug test threads used the documented 16MiB Windows stack;
these ordinary runs do not enable `collector-metrics`.

[capacity.log](capacity.log) is the separate final release-mode gate: a
synthesized source-possible 14,306,948-byte save with 65,536 held records passes
the normal restore validator; the next allocation is atomically refused with
`limit/renewal_limit`; an accepted no-op then collects eligible records, and a
later allocation succeeds. It passed 1/1 in 17.33s. The large save is explicitly
synthetic, not an authenticated old run. The storage-capacity outcome difference
remains part of the owner's semantic review.

## Registered departure gates

`node experiments/departure-gate/run.mjs` passed on the identified clean
collector artifact. The public gate JSON files freeze the consistent second
recorder batch from `run.mjs --skip-sweeps`; the native sweeps were run separately
and protected as recorder inputs. [departure-gates.json](departure-gates.json) and
[compatibility-summary.json](compatibility-summary.json) retain exact populations,
digests and the pinned published rc.15 baseline identity. The copied native
sweep text and population files make omissions/skips visible.

- No-window: 146 delivered programs, 116 executed and 30 explicitly skipped;
  eight seeds of 150 events per program. Dispatch/save/restored-save digests
  match the pinned rc.15 baseline byte-for-byte for this population.
- Windowed: 15 fixtures × 32 seeds × 400 events; all 480 outcome rows match
  rc.15. These compare outcome classifications, not identical raw retirement,
  effect, archive or historical-report representations.
- Registered C3: 44,460 accepted outcomes match baseline. Adapter save is
  2,646 bytes at both 30 and 60 cycles, with identical normalized structure;
  raw saves are 25,682/25,683 bytes. Cycles 51–60 have observed median
  115.1µs and p99 252.1µs. [c3-reviewed.json](c3-reviewed.json) retains the
  complete checkpoints. This scoped gate remains distinct from universal bounds.
- C3 archive: 2,303 items (1,011 records + 1,292 provenance nodes), 722,174
  serialized JSON bytes, zero undrained after drain. The collector draft emits
  record anchors even without live holders, so archive counts must not be
  confused with the earlier baseline's 1,578 items.
- [cross-restore-reviewed.json](cross-restore-reviewed.json) restores the exact
  published rc.15 save and reaches all 61 state-write paths after 742 accepted
  events. The path/guard check is not a runtime statement-coverage trace.
- [pinned-reviewed.json](pinned-reviewed.json) records the five registered
  programs at eight seeds × 4,000 events with pin/restore checkpoints, including
  the forever-skipped commitment case. It reports pins rather than assuming
  every program's retained graph is bounded.

## Paired current computation

[current-computation.json](current-computation.json) compares the preserved
clean f5 reactive WASM (`daab2c71023777a3a9e5dc72f7882c653147c4acb445470368cecd76ed30bc59`)
with the clean c1 reactive WASM above. All 15 registered fixtures × 32 seeded
runs × 400 event attempts pass: **480 full initial snapshot comparisons and
192,000 current-computation/outcome comparisons**, with 142,393 accepted and
49,607 refused attempts. All 480 per-run accepted/refused counts independently
match the registered native sweep. The baseline departs 45,247 records and the collector
113,856; equality therefore covers the declared computational projection, not
raw historical inventory.

The projection preserves complete state/binding values and their caveats; own
grounds use the separate grounds fields and the runtime's evidence-minus-
inherited definition. It includes commitment graph fields and derived status,
current series/standalone commitment bases/grounds, permission records, live
reading members/current renewals, cues, scheduled scalar fields and guard
caveats. Object keys are canonicalized; array order, types and negative zero
remain exact. The report enumerates every path and exclusion. Full historical
lineage, raw qualification evidence, retired/departed records, archive buffers
and historical effects are deliberately not an equality claim. Archive/restore
assertions live in their separate receipts. No finite sweep proves arbitrary
programs or every future trace equivalent.

The tracked diagnostic
[current-computation.mjs](../../current-computation.mjs) has SHA-256
`b1a82524a8d80f94b611f749da034b293922d5d6481a5770afb3af935b9bcc1a`.
The report retains all 15 source hashes, per-run input/result digests and
before/after artifact/source/harness stability checks. This later diagnostic
does not change either pinned runtime artifact.

## Installed package, Chrome and bounded audit

[kit-verification.json](kit-verification.json) records the 56 focused and 329
full kit tests, strict TypeScript, and successful installed-package checks.
[package-report.json](package-report.json) records CLI/library/guides/Python and
Chrome execution from the private 1,011,990-byte tarball (SHA-256
`8aa06bcd179dfaebfac45a70f28566c0af451fb468b0468f874d9edd3fb89f83`).
The 57-test Python caller includes one POSIX-only child skip; the separate
21-test surface also passes. No package was published.

[installed-smoke.json](installed-smoke.json) checks the actual installed collector
in Node 24.11.1 and Chrome 154.0.8037.98: mutual true reasons, nested reading
references, restore, conservative missing archives and unchanged live lists.
Both produce identical results with no browser errors. This demonstrates runtime
report behavior, not interactive visual-design acceptance. An initial ephemeral
smoke harness import error and initial sandbox permission failures remain named
in the verification receipt; successful reruns did not alter runtime source.

[caveat-audit-summary.json](caveat-audit-summary.json) is a redacted summary of
the private local audit, whose actual `accepted` field is boolean `true` at
2026-10-08T00:58:29.849Z. The checked private ledger sequence was 2.
[audit-plan.json](audit-plan.json) preserves its declared inputs and commands.
The scope is finite local current-WASM/archive-consumer tests and registered
gates with protected sweep receipts; it excludes merge/release readiness,
universal boundedness, authenticated history and exhaustive coverage. Output
hashes and the original audit hash are supplied, but keys, raw outputs and
store payloads are excluded. This summary is **not a standalone authenticated
proof**; rerun the plan after transfer. Native profiling and full suites,
package/browser checks and remote CI are separately observed evidence.

All recorded local gates passed. Remote c1 CI core semantics, runtime proofs,
reproducible WASM and both worker checks pass. Legacy Door 3D QA failed
(run 37708806805, job 113095624904): c1 had accidentally changed the workflow
assertion's correctly encoded expected badge into a misencoded string; the
actual DOM was correct. A one-line ASCII JavaScript `\u00b7` escape correction
preserves the exact assertion. [door3d-qa.json](door3d-qa.json) records the
extracted corrected workflow running all seven routes in WebKit 26.0 on
Windows, with ten screenshots and rejection of the corrupted badge value.
Screenshots remain in the local QA directory. Workflow/script hashes identify
this later workflow-only repair; the measured c1 runtime inputs are unchanged.
The one-line repair is commit `440ae92`; the old c1 CI head finished 17/18
checks with this sole failure. Corrected final-head CI remains pending.
Semantic/API/cost approval still
belongs to the owner.

## Reproduction

After normal dependency setup, from the repository root, use a new output
folder because the profiler refuses to overwrite measured reports:

```sh
node experiments/departure-gate/collector-profile.mjs --output=test-results/collector-profile-reproduction
cargo test --release --locked --manifest-path runtime/Cargo.toml --no-default-features --features collector-metrics --test withdrawal_capacity -- --ignored --nocapture
node experiments/departure-gate/run.mjs
node experiments/departure-gate/collector-release-effects.mjs
node experiments/departure-gate/collector-memory.mjs --output test-results/collector-release-memory-reproduction
```

For the computation comparison, supply the preserved clean f5 package directory:

```sh
node experiments/departure-gate/current-computation.mjs --baseline PATH_TO_CLEAN_F5_PKG_REACTIVE --output test-results/collector-current-computation-reproduction.json
```

The driver requires the recorded f5/c1 revision and WASM hashes; a rebuild with
different artifact bytes needs an explicit reviewed identity update. Its report
retains the exact local recorded command.

The recorded profile invocation ran `--only=baseline` and `--only=candidate`
separately with `--baseline-root=test-results/collector-profile-smoke/baseline-source`
and `--output=test-results/collector-profile-final`. The default sizes, three
trials and 100 probes were unchanged. Final native commands/environment are in
[native-summary.json](native-summary.json); WASM identity is in
[build-info.json](build-info.json). The registered C3/sweep checks are recorded separately above; package
validation and actual Chrome execution are recorded above; they are separate
from this directory's passing profile checks.

## Immutable raw records

These hashes identify copied receipts and the redacted derived audit summary;
they do not authenticate arbitrary external reports or a unique session branch.

| Record | Bytes | SHA-256 |
| --- | ---: | --- |
| [audit-plan.json](audit-plan.json) | 12,669 | `a9d31f385d9fe47e605dc6c2c74555a05a476aaaae850a9525de01937a727c21` |
| [baseline.json](baseline.json) | 260,943 | `9af841551059591a136e6f334b4d113efc314acd3b88023d9c1e1888349b2090` |
| [build-info.json](build-info.json) | 754 | `446ca8f17ec7264b150e04dc8bb65c349b441cbcb482af13db2bfa207491b7c8` |
| [c3-reviewed.json](c3-reviewed.json) | 8,690 | `c7bc502eb16bdbeef9a0cb45ad1a39c943d7f12a05f7ba1db3e703ce1ec3d333` |
| [candidate.json](candidate.json) | 263,825 | `233f11b3c04e4e8e0703addbdb8760e6ffdf4f70ec338430ef2035c875353797` |
| [capacity.log](capacity.log) | 511 | `216f867639cfe8e691fa5635d220d0adfa23b9dd4105eb8da9cd6dd150d8d647` |
| [caveat-audit-summary.json](caveat-audit-summary.json) | 4,458 | `452770d7ab94c5f35957616fbea584551c0d19b7ee5c8331852baba631b497c4` |
| [comparison.json](comparison.json) | 122,816 | `44f649e3ecbfab67b0e3ad7ec365a387b8f29701886d59b95cb60c2523d3c331` |
| [compatibility-summary.json](compatibility-summary.json) | 1,069 | `ba010cd587601a28a78c69b86f4e8db9398e969968de6268218526dae8e40589` |
| [cross-restore-reviewed.json](cross-restore-reviewed.json) | 2,906 | `3b4d34819965d56753836ec8431f88ec6efd393ac294b336833f9b6cc66e1bbe` |
| [current-computation.json](current-computation.json) | 246,345 | `b865a6b46cd6e9981898079f82718706c04030653c6fdac8cfc31695fd500c5e` |
| [departure-gates.json](departure-gates.json) | 18,745 | `a5a43f1840b125e86d4de13b84e6fe005ac5e06e6ee22b6684321566bfd50534` |
| [door3d-qa.json](door3d-qa.json) | 895 | `cc7ddfb831c891057d85930ac8e93839a2e1f71d9541d75bb61de42f7e7a84f1` |
| [installed-smoke.json](installed-smoke.json) | 1,416 | `f75caae91fa2b776da92f4104a96a1b5e882bebf45709059a6ea7d0c5f34dac7` |
| [kit-verification.json](kit-verification.json) | 3,594 | `16aea26c4d455c56d24db19fa47272a2353c918fbf54387e6211077eb26f462c` |
| [native-summary.json](native-summary.json) | 2,979 | `998b6277ee80280b4482e89c54d137b2584212c57d97e95ef98d763b1dec81a8` |
| [package-report.json](package-report.json) | 12,300 | `44c01b9c7b8aafe08dd6487a4f7d7752b94a82efbb20517f0b734a91a3d42ccd` |
| [pinned-reviewed.json](pinned-reviewed.json) | 11,088 | `0d33889138d195b1e0049dd5aea9eb3b2173bbe5adc03e6a00e5956f9cd750f2` |
| [rc15-outcome-sweep.txt](rc15-outcome-sweep.txt) | 56,791 | `324b6edec6f033310a3d3282069fe606e705f8158fe959666a3099830f2f6519` |
| [rc15-save-sweep.txt](rc15-save-sweep.txt) | 13,038 | `9ca78bbe1dfc44097092d7956d0d7b43a953c467715b3b962df2ddcccbaf58f9` |
| [rc16-nonwindow-population.txt](rc16-nonwindow-population.txt) | 7,219 | `64cbf364e882c3a4d5c904f49c3d13fde004c97afd08f3a8c965a61ffa3395fe` |
| [rc16-outcome-sweep.txt](rc16-outcome-sweep.txt) | 56,791 | `324b6edec6f033310a3d3282069fe606e705f8158fe959666a3099830f2f6519` |
| [rc16-save-sweep.txt](rc16-save-sweep.txt) | 13,038 | `9ca78bbe1dfc44097092d7956d0d7b43a953c467715b3b962df2ddcccbaf58f9` |
| [rc16-window-population.txt](rc16-window-population.txt) | 878 | `a318f8819d767ce79ffed716fa13b3defe5f573644e5d054006c71b256968331` |
| [release-effects.json](release-effects.json) | 7,290 | `a03b65517a7b4eef56562b45d8286c04faf6919d560f47132f974c27463ea148` |
| [release-memory.json](release-memory.json) | 40,510 | `7665dc398d4bca636585609f7c54d5c7cde99d62df86420f3c2dec23a688b527` |
