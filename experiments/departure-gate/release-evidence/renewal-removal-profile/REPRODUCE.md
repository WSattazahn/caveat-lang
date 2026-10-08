# Reproduce the local renewal-removal profiling study

This is diagnostic source and evidence, not a released runtime. The review commit is
`677ad1365398fe37bb24ff8fb6cfbae82d3ef5b5`, on local branch
`codex/renewal-removal-profile`, after diagnostic commit `1e2374a` and accepted PR177 commit
`055e2af0b411d8385941dd8522201e5afc73cb34`. Nothing was pushed.

## Recover the exact source

Use a Caveat checkout containing the accepted PR177 commit. The included Git bundle
requires that prerequisite; it does not include the entire repository history.
Verify it before fetching its local profiling ref:

```text
git bundle verify /absolute/path/review-source.bundle
git fetch /absolute/path/review-source.bundle refs/heads/codex/renewal-removal-profile
git switch --detach 677ad1365398fe37bb24ff8fb6cfbae82d3ef5b5
```

Do this in a suitable clean checkout. The task itself used the user's main Caveat
checkout and a local branch, with no worktree. Source snapshots accompanying the
build receipts are also included under `repaired-head/builds/*/source/`.

## Inspect or replay existing evidence

`run-002/contract.json` and `registration.json` bind the measured population before
execution. `attempts.jsonl` records each process before launch. All raw outputs,
observed command identities, pair comparisons and the original arithmetic summary
remain under `run-002/`. Do not run the capture command against that directory.
The earlier `run-001/` registered the same population but executed no native
processes. A kit scanner test failed on CRLF; the repair changed only that test.
Its evidence and first builds remain preserved. `verification-supersession.json`
records the transition before any native measurements.

The committed driver can recompute a newly named arithmetic summary without
executing the binaries:

```text
node experiments/departure-gate/renewal-removal-profile/profile.mjs summarize --root=/absolute/path/to/caveat --output=/absolute/path/to/run-002
```

The independent review script rechecks raw arithmetic and identity links separately:

```text
python independent-arithmetic.py /absolute/path/to/run-002 --output=/fresh/path/recomputed.json --summary=/absolute/path/to/original-summary.json
```

Original
command paths identify the actual Windows execution environment; they are evidence,
not portable filesystem locations.

## Execute fresh native measurements

The bundle includes `accepted-pr177-input/`, the exact minimal historical layout
required by registration. It preserves the original contract, candidate identity
and executable, build observation/streams, source manifest, four selected fixtures
and thirteen candidate rows with their raw streams. All fifty files retain their
original bytes and names. `copy-receipt.json` records the original paths and hashes;
`prepare-accepted-pr177-input.py` can verify the copy without running an executable:

```text
python prepare-accepted-pr177-input.py --verify=/absolute/path/to/bundle/accepted-pr177-input --protocol-contract=/absolute/path/to/bundle/run-002/contract.json
```

From the recovered checkout, register a wholly new capture directory using this
included input set; the original prior task directory is not required:

```text
node experiments/departure-gate/renewal-removal-profile/profile.mjs register --root=/absolute/path/to/caveat --output=/fresh/path/to/capture --accepted-run=/absolute/path/to/bundle/accepted-pr177-input
```

This input set is deliberately narrower than the full prior experiment. The
original contract and source manifest are preserved verbatim and mention other
historical files that are not needed by this registration path. Do not treat the
subset as a complete replay of the prior native study. Its accepted review revision
is `055e2af0`; its executable retains its actual `8a161dc` build identity.

The protocol README is the source of truth for registration, builds, freezing and
the fixed 26-process capture. Use a new output directory and new build observations.
The committed `profile.mjs` driver never builds executables or overwrites attempts.
The historical accepted references are used only for finite semantic comparisons.
Their timings must not enter the new plain/diagnostic comparison.

Pinned Rust 1.98.1, locked dependencies, release mode, no default features:

```text
cargo build --release --locked --manifest-path runtime/Cargo.toml --example collector_profile --no-default-features --features collector-metrics
cargo build --release --locked --manifest-path runtime/Cargo.toml --example renewal_removal_profile --no-default-features --features renewal-removal-profile
```

Preserve the plain executable before the diagnostic build. Both must come from
the same clean source revision. `build-profile.py` demonstrates the exact observed
build schema and source selection; its task-local Windows paths need adaptation
on another machine. It also builds ordinary WASM without the profiling feature.

The existing preserved Windows executables can be inspected or executed directly
for a fresh, separately identified exploratory result:

```text
plain.exe ABSOLUTE_FIXTURE_PATH CYCLES MODE 1 100
diagnostic.exe ABSOLUTE_FIXTURE_PATH CYCLES MODE 1 100
```

`MODE` is `release` for the release fixtures and `initialize` for reachable-chain.
Never replace captured raw files. Direct invocations do not automatically satisfy
the registered matrix or its receipt checks. Keep agent-controlled builds/tests
idle during timing; this does not isolate user or operating-system activity.

## Verification and limits

`repaired-head/verification.plan.json`, `repaired-head/verify-profile.mjs`, public
command observations/streams, and `repaired-head/verification.audit.json` specify
the current local correctness coverage. Root-level verification files preserve
the first attempt, including its failure; do not treat them as the final verdict.
Reissue receipts after transfer; private issuer keys and live receipt stores are
intentionally excluded. Public observations are not authenticated execution history.

Kit tests generated six additional `.cav` examples after the original native scan.
The original census guard detected this and its failed final-census record is
preserved. `repaired-head/corpus-refresh-v2.mjs run` records the diagnosed additions
and runs only the actual C006 repository scan in full/reactive configurations,
under its own supplemental two-check plan. Original suite results and censuses
remain intact. The first supplemental helper was refused before Cargo execution
because it declared an empty artifact list; its plan and error are preserved.
The v2 helper binds the same nonempty artifact set as the original verification.
Supplemental snapshots before/after both checks bind the final
candidate population; these are candidates, not counts of parseable programs.

For the original local checkout and private stores, the final entry point is
`node repaired-head/corpus-refresh-v2.mjs check`. It compares the current census with
the supplemental snapshots and issues fresh audits of both immutable plans.
The latest `corpus-refresh-v2-final-check-*.json` links the two actual audits and
census by hash. Compatibility audit aliases retain the first accepted audit;
use the linked latest final-check record for freshness. A transferred reviewer
must create new stores and receipts rather than reuse these local signatures.

Native corpus censuses record discovered `.cav` candidates with the C006 test's
directory exclusions. C006 skips unreadable sources and programs that do not check
as single files, so census totals are not counts of successfully checked programs.
The final census must match both native after-censuses before the audit runs.

`repaired-head/baseline-artifacts/` and `repaired-head/current-artifacts/` preserve the actual ordinary WASM
pair used by the existing 22-trace matrix. Build identities and copy manifests
bind those files. Native source snapshots and diagnostic executables do not stand
in for those WASM artifacts.

Timing attribution uses nested clocks and is perturbed by instrumentation. Requested
heap excludes allocator metadata, stack and RSS. Native apply excludes subsequent
draining/serialization, browser and WASM. Search probes count String equality
calls, and shifted-header bytes are an operation estimate, not allocated bytes.
No optimization benefit, performance target, general workload guarantee, or release
readiness follows automatically from these observations.
