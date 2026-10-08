# Reproduce the early-boxing experiment

Baseline: `11c2a6a741609d3b640f27224ab6c9943b07f72c`.
Candidate: `590fae59aa4295ac5209f930a1e58ac152bd4603`.
The experiment is local. It does not authorize a merge, publication or release.

## Read and inspect

Start with `REVIEW.md`, the frozen `run-001/PROTOCOL.md`, the timestamped
`run-001/analysis-*.md`, and `review/README.md`. `REQUEST.md` preserves the request;
`intake/` distinguishes supplied review statements from work executed here.

`review-source.bundle` includes the complete history reachable from the candidate
branch, with no external prerequisite objects. `source-preservation.json` records
a fresh clone without checkout and exact baseline-to-candidate diff validation.
It includes no unrelated local branches, stashes, ignored files or worktrees.
Inspect it in an ordinary review clone:

```
git clone --branch codex/archive-early-boxing review-source.bundle review-clone
git -C review-clone diff 11c2a6a741609d3b640f27224ab6c9943b07f72c 590fae59aa4295ac5209f930a1e58ac152bd4603
```

`review-source.patch` contains that same comparison. `builds/before` and
`builds/after` preserve actual native source-byte snapshots, commands, compiler
identity, raw build streams and executables. `before-wasm/` is the previously
verified ordinary 11c2 build used by the behavioral matrices; `after-wasm/` is the
fresh ordinary 590fae5 build. Their build identities and artifact hashes are
preserved. Native before/after comparison executables are both fresh builds.

## Recompute the recorded measurements

`run-001/` holds both frozen executables, build observations, fixtures, protocol,
append-only attempt ledger and every process stdout/stderr. Measurements are
ordinary native harness summaries; individual rejected-event timings are not
available. Do not interpret three rejected events within a process as independent
process replicates. All 32 processes and all 16 pairs belong to the result.

The frozen driver reconstructs the recorded Windows argv using Windows paths.
Run its arithmetic replay on Windows from the extracted bundle; this command
does not execute a native binary and writes a fresh timestamped analysis:

```
python -B profile.py analyze --out run-001
```

For an independent, portable saved-data check, use the explicit saved-only mode
documented in `review/README.md`. That mode must report that it did not inspect a
live checkout. The default independent check additionally checks the original
live source. Preserved byte snapshots and Git-normalized source can differ in
line endings; do not silently replace the compiled-input hashes with new ones.
Arithmetic agreement and matching hashes do not authenticate execution history.

## Rebuild and remeasure

Use an ordinary checkout, two new target directories and a new output directory.
Do not overwrite the supplied run or silently replace a failed attempt. Review
the explicit Windows paths in `build.py` before adapting it to another host.
Use the same build helper and settings for both variants. At the baseline head,
register the new protocol and preserve the clean before build; then switch to
the candidate and preserve the clean after build. `profile.py --help` and the
frozen protocol give register/freeze/run/analyze arguments.


The supplied `registration-reference/` directory is directly usable as
`--reference <bundle>/registration-reference`: it preserves the original
contract/registration bytes and the four exact fixtures required by registration.
`COPY-PROVENANCE.json` lists their hashes. It is a minimal registration input,
not the earlier full profiling run. At the clean baseline, use:

```
python profile.py register --repo <review-clone> --out <new-run> --reference <bundle>/registration-reference
```

Copy `profile.py`, `PROTOCOL.md` and `build.py` together to a new helper directory
before preparing a new run. The build helper derives its output directories from
its own location and refuses existing build attempts; do not run it in the
supplied evidence directory. Adapt its explicit checkout path before registering
(the new registration will pin those helper bytes). The before build also checks
and copies the baseline ordinary WASM from `dist/build-info.json`, `dist/pkg` and
`dist/pkg-reactive`. Prepare those at the baseline with its ordinary build, or
restore exactly those files from the preserved `before-wasm/` with matching
build-info/artifact hashes, before invoking `build.py before <baseline>`.
This dependency is for the companion comparison artifacts, not the native Cargo
compilation. Freeze the before identity at the baseline, then switch the same
review clone to the candidate, build/freeze after, and run the fixed population.

The exact native command for each variant is:

```
cargo build --release --locked --manifest-path runtime/Cargo.toml --example collector_profile --no-default-features --features collector-metrics --target-dir NEW_TARGET_DIRECTORY
```

The original counting allocator and collector-work counters are enabled in both
variants; phase clocks, per-record attribution and origin headers are disabled.
Rust 1.98.1/MSVC is pinned for the recorded comparison. A different platform or
compiler is a separate experiment, not proof of these exact absolute bytes.
Registration and artifact freezing verify source, fixture, harness and compiler
identities. Run the fixed 32-process population serially only after controlled
build/test activity is idle. Other host activity was not exclusively controlled.

The fixed target is at least 10% less whole-apply additional caller-requested heap
for both primary outcomes. Fresh before peaks must reproduce the recorded
references; all 8 after peaks must meet their fixed ceilings. A mismatching
baseline makes comparability unresolved. No target, denominator or sample count
may be changed after seeing results. The separate latency guard uses the median
of 8 paired process ratios, not a ratio of pooled event medians.

## Correctness and receipt boundaries

`verification.plan.json`, `verification-summary.json`, `public-verification/`
and `gate-products/` preserve exact commands, bounded coverage and raw outputs.
`verify.mjs` records the original machine paths. Review those paths before reuse.
It does not prepare the ordinary WASM required before kit gates; build or restore that artifact first. Cargo checks/tests and the departure compatibility gate do compile their own native artifacts. The kits,
native suites, real capacity test, registered departure gates and both reused
independent matrices have distinct coverage. The browser check uses installed
Chrome with `PLAYWRIGHT_CHANNEL=chrome`.

This Caveat repository does not define the surrounding game workspace's generic
`npm run typecheck`, `lint`, `test` or `audit:all` scripts. The recorded plan uses
Caveat's actual compiler, Clippy, native, kit, compatibility and browser commands.
No unrelated game checks or complete release/CI run is implied.

Native C006 scans a dynamic population of repository `.cav` candidates. Its
before/after censuses are retained separately; the count is not a successful
individual-program-check count. Private receipt stores, issuer keys, signed run
logs and build caches are excluded from the review ZIP. Public observations are
inspectable evidence, not transferable trusted receipts. Reissue receipts after
moving or modifying the checkout. The final saved-data check supports this
experiment's reported measurements; it does not establish a workload guarantee.
