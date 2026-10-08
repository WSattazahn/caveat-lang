# Collector index compaction: registered measurement plan

This experiment compares original departure baseline
`f5ec8294efe2be24705f234ef75e5f5459aa5e89`, repaired collector baseline
`3d77aa27bcffa16f39818b2684e9afb3b102ed17`, and one explicitly frozen candidate.
The owner requires **both** at least 50% reductions in collector-added retained
requested heap and collector-added growth-dispatch peak at 3,000 reachable-chain
cycles. Current semantics, required reasons, save bytes, snapshots and ordered
archive contents must remain unchanged from the repaired collector baseline.

Preparation is not a measurement or a passing result. The runtime implementation,
tests, clean source freeze, native build and quiet-machine signal are separate.
The driver does not build code or relax a failed target.

## Exact targets

For each of three fresh matched trials at `reachable-chain`, 3,000 cycles:

- Retained heap is `before_release.retained_rust_heap_bytes`: after growth,
  draining and the existing 100 unchanged plus 100 unrelated-renewal probes.
- Growth peak is `growth.max_dispatch_additional_heap_bytes`: maximum additional
  requested heap during any growth dispatch. It excludes the separate unchanged,
  unrelated, failed-release and successful-release phases.
- For either measurement, let **F** be freshly measured original f5, **B** freshly
  measured repaired 3d, and **C** the compacted candidate. The denominator is
  **B − F**, not B. Require B > F and **C − F ≤ 0.5 × (B − F)**. Report F, B, C,
  denominator, candidate-added bytes, reduction and integer ceiling separately.
- Both inequalities must hold in all three trials. No averaging a failing trial
  into a passing result; no substitution of saved bytes, small fixtures or timings.

Historical anchors are F=6,004,653 and B=13,413,126 retained bytes (ceiling
9,708,889), and F=6,139,810 and B=14,176,145 growth-peak bytes (ceiling
10,157,977). These anchors are predictions from prior receipts; **fresh paired
measurements set the actual denominators**.

These differences are a registered cost comparison, not isolated instrumentation
of every collector allocation. The counting allocator measures requested Rust
heap, excluding allocator internals, stacks and RSS. Whole-dispatch peaks include
transaction copies, ordinary departure, collector work and archive construction.
Native `apply` omits snapshot serialization and is not browser/WASM latency.

## Frozen inputs and builds

The original `runtime/examples/collector_profile.rs` remains byte-identical,
SHA-256 `1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357`.
The five original fixtures are checked against the frozen Windows f5 receipt's
raw-byte hashes. The new `high-degree.cav` is separately hashed; it is never
substituted for a target fixture. Preparation copies inputs and executables into
a fresh output directory and writes an immutable contract with a driver hash.

Preserved executables:

| Variant | SHA-256 | Features |
| --- | --- | --- |
| f5 | `27cd7b3e68a21a0633e2abe8effc7568529fca1ac3555e005df41fcc4453b8c4` | `--no-default-features`; original baseline has no collector metrics ABI |
| repaired 3d | `c0bf892c2cb618b377c38a14206bc56244d891c561f0684fceeb8b0aa8b0a9c5` | `--no-default-features --features collector-metrics` |
| candidate | recorded only after clean source freeze and build | same settings as 3d |

Original and candidate builds use the checked Rust 1.98.1 toolchain and the
runtime's release profile (`lto=true`, `codegen-units=1`). The shared build command
is `cargo build --release --locked --manifest-path runtime/Cargo.toml --example
collector_profile --no-default-features`; add `--features collector-metrics` for
3d/candidate only. f5 zero work counters mean unavailable, not zero work. Allocation
and timing instrumentation overhead is retained rather than subtracted selectively.

Rebuilding old source requires an ordinary scratch extraction from the pinned
Git revision, plus the identical frozen harness, using the same commands. Do not
silently substitute a different executable for a preserved hash. A distinct build
must receive its own reviewed identity and output directory.

## Finite matrix: 33 triples, 99 native executions

| Workload | Cycles | Trials |
| --- | --- | --- |
| self, mutual, reachable-chain, release-mutual | 60, 300, 1,000 | one per size |
| self, mutual, reachable-chain, release-mutual | 3,000 | three per size |
| release-self | 60, 300, 1,000, 3,000 | one per size; diagnostic timings only |
| high-degree | 60, 300 | one per size |
| high-degree | 1,000 | three |

Every execution uses the original harness with drain interval 1 and 100 probes:
100 unchanged events and 100 unrelated renewable-source events are included in
each trace. Release workloads additionally perform three final-binding failures
and one successful root release; the original harness asserts unchanged saves
and no archive publication on every failed attempt. Nine rejected attempts and
three successful releases per variant at the repeated release-mutual size still
do not estimate a reliable population p99.

Triples execute serially: trial 1 f5/3d/candidate, trial 2 3d/candidate/f5,
trial 3 candidate/f5/3d. Other local builds/tests must be quiet. No assumption of
CPU affinity, universal hardware performance or perfect OS-load control is made.
Report each timing trial and its range; do not make a single-trial speedup claim.

The new high-degree fixture has 16 subjects sharing one self-withdrawn reason and
16 overlapping state holders. `same`, `trim`, `expand`, `clear`, `refuse`, `fail`
and staged release exercise shared references and degree transitions. Its repeated
cycle creates, holds and retires this fixed-size shape; it does **not** increase
the degree with cycle count. The separate regression exercises staged release of
15 subjects, preservation of `s00` and its exact reason, then final release.

## Equivalence checks

The native comparison requires byte-exact save hashes and complete ordered
archive NDJSON hashes/counts for 3d versus candidate, before and after release.
The reachable-chain save also remains identical to f5. Collected counts agree;
other work counters are exposed as diagnostics because representation changes
can legitimately alter traversal accounting. f5's intentionally different
departed inventory is not treated as a failure of collector equivalence.

`index-compaction-equivalence.mjs` is a **separate**, finite actual-WASM check.
It compares full snapshots, views and dispatch outcomes, exact save strings,
and every ordered archive item for 3d versus candidate. Only object-key order is
canonicalized; array order, types and negative zero remain exact. There is no
projection or omitted provenance field. Six 60-cycle traces plus a staged
high-degree trace run with per-attempt and end-only drains. Exact-save restore
branches at initial/midpoint/final checkpoints compare a following accepted event
without replacing the main session or dropping its undrained history. All failed
attempts must preserve save text and queued archive count. This adds no timing or
allocation evidence and proves only these finite traces.

The preserved 3d reactive WASM is
`81c048ac5197f2ba85a27dcb077e930ac7a03663d69b803971017d47a13ea3eb`;
its glue is `a0fff7c2bc9df767502cb66f978bcc944fc65701cdeea14d7e37171c75118056`.
Both runtimes must agree with clean compiled build-info identities before use and
remain byte-stable afterward.

## Commands and receipts

From the repository root, with an authorized new output directory and preserved
executables:

```text
node experiments/departure-gate/index-compaction.mjs prepare --output=OUTPUT --f5-exe=F5_EXE --before-exe=THREE_D_EXE
```

After the clean candidate source freeze and authorized native build:

```text
node experiments/departure-gate/index-compaction.mjs freeze --output=OUTPUT --candidate-exe=NEW_EXE --revision=FULL_REVISION
```

Only after the quiet signal, run either the entire registered matrix once or
individual registered cells, then summarize:

```text
node experiments/departure-gate/index-compaction.mjs run --output=OUTPUT --all=true
node experiments/departure-gate/index-compaction.mjs run --output=OUTPUT --workload=reachable-chain --cycles=3000 --trial=1
node experiments/departure-gate/index-compaction.mjs summarize --output=OUTPUT
node experiments/departure-gate/index-compaction-equivalence.mjs --before=THREE_D_PKG_REACTIVE --candidate=NEW_PKG_REACTIVE --candidate-revision=FULL_REVISION --native-contract=OUTPUT/contract.json --output=OUTPUT/equivalence.json
```

Choose the all-at-once command or individual cells; the driver refuses duplicate
runs. Every command, source/fixture/executable identity, timestamp and raw output
is retained. Native allocation targets are evaluated immediately after each
registered 3,000-chain triple. A target failure remains in its result and gives
exit code 2. A command/equality failure retains a failed comparison receipt and
the completed raw runs. Summaries report incomplete or failed populations without
discarding them. A revised implementation uses a new candidate directory; do not
erase failures, alter the matrix or reset the comparison to a favorable baseline.

Capacity-after-release checks and layout/degree diagnostics, if added, must remain
separate from this unchanged primary harness and be labeled with their own inputs.
Use observed diagnostics to explain costs; do not infer representation gains from
the number of records collected or add unmeasured optimizations to satisfy a target.
