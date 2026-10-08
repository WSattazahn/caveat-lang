# Native compatibility verification, 2026-10-07

The final release runtime matches the pinned rc.15 reference for the complete delivered corpus: 146 no-window source rows (116 reactive programs executed, 30 sources skipped by the reactive loader), and 13 windowed programs producing 416 matching run rows. Every windowed run reports `fatal=false`; no-window executable rows report `fatal=0`.

The no-window sweep uses 8 seeded runs of 150 events per program. Its digest covers dispatch outcome JSON, saved JSON, and restored saved JSON. The windowed sweep uses 32 seeded runs of 400 events per program and compares accepted/refused outcomes. Both binaries read the same source files and relative paths from the current checkout.

Baseline: `v0.1.0-rc.15`, peeled commit `3a88ba0f80d563b4493840dc7bd7e195329b8302`.

## Reproduce on Windows or Linux

From the repository root:

```sh
node experiments/departure-gate/sweep.mjs
```

The runner discovers tracked `.cav` files plus new `.cav` files only in the owned departure-gate and 002-decision directories, classifies actual history window declarations, and records both populations. It uses a Git source archive of the pinned baseline in a fresh temporary directory; it does not create a Git worktree or switch branches. The current sweep examples are copied into that archive, and both runtimes are built with:

```sh
cargo build --locked --release --manifest-path runtime/Cargo.toml --example save_sweep --example outcome_sweep
```

For each runtime, with the repository root as the process working directory, the sweep invocations are:

```sh
<runtime>/target/release/examples/save_sweep 8 150 < test-results/rc16-nonwindow-population.txt
<runtime>/target/release/examples/outcome_sweep 32 400 < test-results/rc16-window-population.txt
```

On Windows the executable suffix is `.exe`; the portable Node runner supplies stdin directly. It writes `rc15-save-sweep.txt`, `rc16-save-sweep.txt`, `rc15-outcome-sweep.txt`, and `rc16-outcome-sweep.txt` under `test-results/`, preserving stderr departure counts separately. `compatibility-summary.json` records hashes and counts after a successful run. Historical saved digest files from another OS are not the comparison baseline because checkout newline choices can affect source identity.

## Native workflow gates

Completed on the final runtime source:

- Full native all-targets tests: 849 passed, 79 result groups, exit 0. Receipt: `rc16-archive-final-native-tests-16mb.log`.
- Reactive-only native all-targets tests: 837 passed, 79 result groups, exit 0. Receipt: `rc16-final-reactive-only-tests-16mb.log`.
- Full and reactive-only strict all-targets Clippy: passed with `-D warnings`; final reactive-only receipt: `rc16-final-reactive-only-clippy.log`.
- Formatting check passed. Sweep example sources were unchanged by the continuation.

The two native test runs set `RUST_MIN_STACK=16777216`. The default Windows thread stack had previously overflowed in `the_maximum_valid_procedure_depth_still_dispatches`; these receipts establish the larger-stack result, not a default-stack pass.

Reactive-only commands:

```sh
RUST_MIN_STACK=16777216 cargo test --locked --manifest-path runtime/Cargo.toml --all-targets --no-default-features
cargo clippy --locked --manifest-path runtime/Cargo.toml --all-targets --no-default-features -- -D warnings
```

In PowerShell, set `$env:RUST_MIN_STACK='16777216'` before invoking Cargo.

## Default Windows stack spot check

The current optimized release runtime passes `the_maximum_valid_procedure_depth_still_dispatches` with `RUST_MIN_STACK` unset (1 passed, 28 filtered out). Command:

```sh
cargo test --locked --release --manifest-path runtime/Cargo.toml --test dispatch_outcomes the_maximum_valid_procedure_depth_still_dispatches -- --exact
```

Receipt: `rc16-release-default-stack-depth.log`. The already-started pinned rc.15 debug test reproduced the same default-stack overflow (`0xc00000fd`); receipt: `rc15-native-default-stack-depth.log`. The larger-stack debug-suite requirement therefore also occurs on that baseline.
