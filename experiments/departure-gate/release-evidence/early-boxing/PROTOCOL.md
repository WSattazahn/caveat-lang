# Early archive boxing: preregistered comparison

This is the user-authorized experiment in earlier boxing of archive entries. Baseline source is `11c2a6a741609d3b640f27224ab6c9943b07f72c`, runtime tree `89f3a5bde145a3071ce6e41652a866e6b722f3b1`. Freeze this protocol, driver and registration before the implementation. No change to eligibility, reason information, journal attachment, numeric archive ordering, effects, save format or rollback is permitted. The parent owns the source change, separate correctness checks and builds; this driver only freezes artifacts, runs the finite comparison and recomputes results.

## Builds and provenance

Both executables are freshly built from clean tracked source with the same pinned compiler and exact command:

`cargo build --release --locked --manifest-path runtime/Cargo.toml --example collector_profile --no-default-features --features collector-metrics --target-dir <OUT/private-build-target/before|after>`

All attribution features are off. The original `runtime/examples/collector_profile.rs` must retain SHA256 `1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357`. BEFORE must be the exact baseline revision; AFTER must descend from it. Preserve compiler/Cargo identity, argv, all declared source inputs and snapshots, clean-source checks before/after build, raw build streams and executable hashes. Runtime flags are identical. Registration copies the four existing fixtures and records their hashes, plus the prior event-profile contract as historical reference. No new `.cav` fixture is added to the repository.

The build observation interface is `build.json`, `build.stdout`, `build.stderr`, `<kind>.exe`, and `source/<repository-relative input>` for `kind=before|after`. JSON fields: `kind`, `revision`, `runtimeGitTree`, `argv`, `cwd`, `startedAt`, `finishedAt`, `status`, `error`, `rustc`, `cargo`, `cleared_environment`, `inputs:[{file,sha256}]`, `inputsStable`, `headAfter`, `trackedDiffAfter`, `stdoutSha256`, `stderrSha256`, `executableSha256`, `targetDir`, and `featureTree` (recorded `cargo tree -e features` output). Each build uses a separate initially empty target directory; normalize only its path when comparing argv. Freeze checks the source inputs against the clean checkout and preserved snapshots, then copies immutable evidence into the capture directory. Native-only inputs are all tracked runtime files, `rust-toolchain.toml`, and any present root/runtime Cargo configs. Fixtures are frozen separately. Games/WASM build-script/package inputs are outside this feature-disabled native build boundary. The two source sets/compiler/build settings must agree; the **only allowed changed native input is `runtime/src/reactive_departure.rs`**. No new runtime tests are included in this candidate. External correctness adapters, protocol and provenance may be committed under `experiments/departure-gate/early-boxing/`; they do not compile into either native executable. Registration also pins the parent's `build.py` helper bytes.

## Fixed population and execution

Exactly 16 cells / 32 fresh processes, executed serially after the coordinator confirms all agent-controlled builds/tests have finished. This is not a claim of a globally quiet or isolated host.

1. Primary: eight `release-mutual` / 3,000-cycle pairs. Odd pairs run BEFORE then AFTER; even pairs AFTER then BEFORE.
2. Supplemental, one pair each in this fixed order: mutual60, mutual300, mutual1000, release-self3000, fixed-N16 high-degree1000, required-chain300, required-chain1000, required-chain3000. Continue the same alternating order across all 16 cells.

Each argv is `<frozen exe> <frozen fixture> <cycles> <mode> 1 100`; mode is `release` for mutual/self/high-degree and `initialize` for required-chain. Drain after each growth event; 100 steady probes and 100 unrelated mutation probes. Every release-mode process attempts exactly three rejected final releases and then one successful release. Chains have no final release. No warmup run, extra control population, adaptive sample count or silent rerun is allowed. Timeout is 1,800 seconds per process. Clear recorded Rust/build environment overrides identically for both variants.

Preserve every started command in an append-only attempt ledger, all stdout/stderr bytes, exit/error/timeout observations and timestamps, before/after source checks, parsed comparisons and hashes. Never overwrite or reset an existing run directory/ledger. A process, provenance or semantic invariant failure stops further capture and remains in evidence; any follow-up requires a separately named attempt and explicit explanation. A memory or latency target miss does **not** stop the registered matrix.

## Fixed memory criterion

Measure total **additional requested Rust heap bytes during the whole apply call**, using the ordinary counting allocator. Include container capacity, transactional copies and undrained archive; exclude allocator internal overhead/free lists, stack and OS reservation/RSS. This is neither archive-origin bytes nor collector-only memory, and independent phase peaks are never summed. BEFORE and AFTER use identical event baselines and report-buffer handling.

Historical primary reference peaks are 27,403,251 B for success and 27,403,464 B for rejection. Every fresh primary BEFORE process must reproduce those respective values; otherwise flag a provenance/measurement difference and mark fixed-target comparability unresolved. Do not change the targets or denominator after seeing results.

The fixed 10% target requires **all eight** primary AFTER success peaks to be at most **24,662,925 B**, and all eight primary AFTER rejected-release maxima (maximum over the three attempts in each process) at most **24,663,117 B**. Report every paired baseline/candidate absolute peak, reduction and pass/miss, plus repeatability. A pass requires comparability as well as both outcome targets. Supplemental and growth/steady/unrelated peaks are reported in absolute bytes with paired deltas; increases are explicit, not hidden by the bulk result. This target is an experimental criterion, not a predicted saving.

## Latency guard and arithmetic

For each primary process, success contributes its single apply time; rejection contributes the original harness median of its three attempts. For each outcome calculate the eight paired ratios `AFTER process median / BEFORE process median`; the guard is the ordinary median of those eight ratios (average middle two) **<= 1.05**. Greater than 1.05 is a guard failure, independently of memory. Show all eight ratios, their min/median/max, both sets of process medians and paired absolute differences. Ratios of aggregate medians are secondary descriptive values, not substitutes for the registered paired guard.

The unchanged harness preserves summary median/mean/max/p99 for every phase in stdout, **not individual rejected sample times**; do not pretend its three attempts are three independent processes. Its even-count phase median selects the upper middle sample. Supplemental one-pair timings and short-sample p99 are descriptive. Report last-10% growth separately from full growth; the populations overlap and are not additive. No confidence interval or noise-free speedup claim is implied by eight pairs. No latency target was set for the supplemental cells, but regressions there remain visible.

## Meaning and equality checks

Each pair must match augmented-source digest, cycles/mode/drain, before/after save digest and bytes, retired/withdrawal/undrained counts, ordered archive NDJSON digest/bytes/record and provenance counts, growth archive and queue counts, successful collector-work projections and phase sample counts. Retained requested heap before and after release/drain must match exactly. The original harness itself asserts rejection, exact rollback save equality and zero leaked archive after each rejected event. Failed collector-work fields are deliberate zero placeholders, not attempted-work evidence.

This native harness establishes those finite digest/count/work projections, not every intermediate snapshot or arbitrary program equivalence. Any stronger exact snapshot/restore/journal/order coverage comes from separately preserved correctness gates and is reported separately. `max_heap_freed_by_drain` is a diagnostic observation, not archive semantics. A changed retained heap, save or ordered archive is not silently accepted to meet the peak target. Preserve all required evidence and archive information.

## Commands

`python profile.py register --out <fresh run directory> --reference <event-phase-profile/run-001>`

`python profile.py freeze --out <run directory> --kind before --build <before build directory>`

`python profile.py freeze --out <run directory> --kind after --build <after build directory>`

`python profile.py run --out <run directory>`

`python profile.py analyze --out <run directory>`

Registration pins the complete contract and tools by hash. Each later action verifies that pin, frozen inputs and artifacts; analysis reads preserved stdout and verifies the completed ledger and streams before computing results. All generated reports use fresh exclusive filenames. Root authorizes the timing handoff; preparing or freezing these tools does not start capture.
