# High-degree confirmation: frozen scope and criteria

This separately registered comparison asks whether the previously observed high-degree slowdown repeats. It does not redefine or rerun the completed mutual3000 peak-memory experiment. No runtime change, optimization, rebuild, checkout switch, push, merge, publication or release belongs to this task.

## Fixed artifacts and workload

Reuse the exact ordinary native executables and their preserved build identities from `early-boxing/run-001`:

| Variant | Source revision | Executable SHA256 |
|---|---|---|
| BEFORE | `11c2a6a741609d3b640f27224ab6c9943b07f72c` | `2286a87f01560aed8f12ee4f5a6f8f5116f5ea79b4d3bc8f72e6016afae9a95f` |
| AFTER | `590fae59aa4295ac5209f930a1e58ac152bd4603` | `64125fda7f7323469f9d31f4ac5f787bf3b817856a38d25af2b653044be643d2` |

Both were built with Rust 1.98.1, identical release/locked/no-default-features/collector-metrics settings and separate initially empty target directories. All event/renewal/withdrawal attribution features were disabled. Reuse, do not rebuild. Pin exact compiler, Cargo, normalized argv, feature tree, source snapshots, raw build streams and original harness hash `1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357`. The only changed native source input between the preserved builds is `runtime/src/reactive_departure.rs`. Verify the current clean tracked checkout remains the exact AFTER revision and matches its preserved native inputs; reading these files does not switch source or influence the executable contents.

Use only the existing fixed-N16 high-degree fixture, SHA256 `f3fd8fe258e926dab76a7793187215915037f67bc95317ec50556b1344eace4f`, plus the unchanged original harness's probe suffix. Registration computes and pins the augmented-source digest and verifies each output against it. The fixture repeats a bounded 16-subject graph; 1,000 cycles do not mean degree 1,000.

Every process argv is `<frozen before|after exe> <frozen high-degree fixture> 1000 release 1 100`. Drain after each growth event, then 100 steady and 100 unrelated mutation probes, three rejected release attempts and one successful release. No warmup, new fixture, extra process, extra sample or silent retry.

## Population and timing handoff

Exactly eight matched pairs / sixteen fresh processes. Odd-numbered pairs execute BEFORE then AFTER; even-numbered pairs execute AFTER then BEFORE. Run serially after the coordinator confirms no agent-controlled builds/tests/compression jobs are running. This is not a claim that the host is globally quiet or isolated from user/background activity. Clear the same recorded Rust/build override environment variables for both variants.

Freeze this protocol, driver, reused helper bytes, identities, fixture and complete contract before any capture. Registration and capture require the parent's explicit handoff after independent protocol review. Timeout remains 1,800 seconds per process. Every started attempt is appended to an immutable ledger; preserve stdout/stderr bytes, command, timestamps, exit/error/timeout, executable/contract hashes, source checks and pair comparisons. Refuse an existing registration directory or capture ledger. Do not overwrite, restart, extend or silently retry. Stop on process, provenance or semantic failure. A latency review trigger does not stop the registered population.

## Three separate review triggers

For each phase below, each process contributes its original harness `median_ns`. Compute eight paired ratios `AFTER process median / BEFORE process median`, then their ordinary median (average of the middle two). A median **strictly greater than 1.05** triggers review for that phase:

1. Full growth: all 1,000 cycle dispatches; original harness uses the upper-middle event time.
2. Successful release: the one release time in each process.
3. Rejected release: the median of the three rejected attempts within each process.

These are independent review triggers, not a new memory gate or a retroactive failure of the earlier experiment. Equality to 1.05 does not trigger. Do not replace the paired median with a ratio of aggregate medians. Preserve all eight ratios and absolute process times, paired differences, minima/medians/maxima, and counts above 1.0/1.05. Summarize both run orders descriptively to expose order sensitivity. No post-result threshold change, selective exclusion, confidence claim, automatic follow-up run or automatic optimization is allowed.

The unchanged harness emits event-group median/mean/max/p99 summaries, not individual rejected sample times. Three failed events inside one process are not three independent process observations. Last10% growth overlaps full growth and is descriptive only, as are steady and unrelated-change summaries. Short-sample p99 is not a reliable tail estimate.

## Allocation and equality

For all six reported phases, show whole-apply maximum additional requested heap in absolute bytes and paired deltas, alongside timing summaries. There is no newly selected byte ceiling: report whether the previously observed memory values repeat, including any increase. Include retained heap before/after release, save sizes and archive output separately. Requested heap includes live container capacity but excludes allocator internal overhead/free lists, stacks and OS reservation/RSS. It is neither archive-origin bytes nor isolated collector storage. Do not sum independent phase maxima.

Require exact equality within every pair for augmented-source digest, cycles/mode/drain, phase counts, save digest/bytes, retired/withdrawal/undrained counts, ordered archive digest/bytes/record/provenance and growth/queue counts, successful collector-work projections and retained requested heap. Also compare each variant's finite semantic/retention projection to its preserved earlier high-degree output; historical timing/allocation values are labeled reference observations and excluded from new aggregates. The original harness independently asserts exact rollback save equality and zero published archive after each failed event. Rejected collector-work values are zero placeholders, not attempted-work attribution.

These are finite native projections, not proof for every intermediate snapshot or every Caveat program. Previously completed correctness evidence and the primary memory result retain their own scopes. The new conclusion should say whether each observed slowdown repeats under this criterion and recommend retaining, repairing or restricting the candidate for this workload, while acknowledging measured uncertainty. A trigger prompts that review; it does not automatically authorize repair.

## Preserved inputs and commands

Registration copies the previous contract/registration and their pinned tool/fixture records, both identity records/pins, both executables and each identity's source/build records. It also preserves the two prior high-degree stdout/stderr/process/source-check records, their comparison and the historical attempt ledger for provenance. Only those historical process outputs are reused as semantic references. The complete earlier experiment is not rerun. `native_helpers.py` is an exact byte copy of the reviewed earlier driver; only its hash, source/identity verification, sample validation and comparison utilities are reused. Its old population, capture function and analyzer are never called.

Commands (prepare/review first; no implicit execution):

`python profile.py controls --out <fresh controls directory> --prior <early-boxing/run-001>`

`python profile.py register --out <fresh run directory> --prior <early-boxing/run-001> --controls <controls directory>/controls.json`

`python profile.py run --out <registered run directory>`

`python profile.py analyze --out <registered run directory>`

Controls are data-only checks against preserved stdout and deliberately altered in-memory copies; they run no native binary. Registration pins controls evidence as well as protocol/helper/driver hashes. Each later command verifies the complete contract hash, tools, input records and frozen identities. Analyze recomputes from raw stdout and validates ledger order/argv/hashes before producing fresh exclusive JSON and Markdown reports. Old thresholds are never imported into the new review triggers.

For portable reproduction, the new run's `prior/` directory preserves every transitive prior file required by this register step, including identity-referenced source snapshots and raw build streams; no original absolute input folder, private store or build target is required. First run data-only controls with `--prior <copied run>/prior`, then register using that same `--prior`, the fresh controls receipt, a fresh output path, and `--repo <clean checkout at 590fae59>`. The preserved Git bundle can materialize that checkout separately. Run/analyze use the copied bytes. Saved Windows argv is compared using Windows path syntax even when data-only analysis is run on another OS. Registering/running native Windows executables still requires an appropriate Windows environment and an explicit timing handoff; copying evidence does not execute them.
