# Renewal occurrence-removal profiling

This experiment profiles the algorithm accepted at
`055e2af0b411d8385941dd8522201e5afc73cb34`. It contains no optimization, speedup
requirement, memory acceptance threshold, or assumed 10% goal. Freeze the first
contract before capture. Never discard an attempt, replace its outputs, or adjust
the population after seeing measurements.

The accepted runtime tree is `aec511bf3ee147a79391f90674446eff9c7f393e`, also used
by measured `8a161dcdfdc8a2a5b3ca43b4183552401002cca7`. The preserved accepted
plain executable has SHA-256
`9f5a0a5af2b4073849d30fd7070380eb1341ea4ae9206828aa7fd755cb207909` and remains
identified by its actual `8a161dc` build. The new profiling head adds native-only
instrumentation. Fresh plain and diagnostic executables must be built from that
same clean profiling head, with only the attribution feature differing.

## Fixed population and ordering

Exactly thirteen matched pairs / twenty-six fresh processes are registered:

| Existing fixture | Cycles | Pairs | Purpose |
|---|---:|---:|---|
| `release-mutual` | 3000 | 4 | Repeated bulk successful/rejected attribution and perturbation |
| `release-mutual` | 1, 2, 60, 300, 1000 | 1 each | Tiny controls and increasing size |
| `release-self` | 3000 | 1 | Different withdrawal reason shape |
| `high-degree` | 60, 1000 | 1 each | Repetition of a fixed N=16 graph; not increasing degree |
| `reachable-chain` | 3000 | 1 | Required-history retention without a final release |

Pairs run in table order. The first pair runs plain then diagnostic; the next
runs diagnostic then plain, continuing that alternation. The four primary pairs
therefore have balanced order. The coordinator confirms no concurrent local task
builds/tests during capture. This coordinates agent-controlled work; user/background
processes and the host are not claimed to be globally isolated. Processes run
serially. There is no discarded warmup
or automatic repetition; all observed trials count.

Every process uses the original `DRAIN_EVERY=1`, `PROBES=100`. The harness includes
100 unchanged and 100 unrelated mutation dispatches. Release-mode processes run
three rejected releases followed by one successful release. Four primary pairs
give four successful and twelve rejected diagnostic dispatches, with matched
plain controls. Four process pairs support a descriptive repeat/range check, not
stable tails or a statistical performance claim. All one-pair supplemental
timings are descriptive.

The exact fixture bytes are copied from the prior frozen corpus at registration;
their hashes must also equal the current tracked files. No new `.cav` source is
added to the repository for this experiment.

## Commands and immutable capture

Run these from the repository root. Supply absolute paths on Windows; replace
the uppercase values below with the actual directories and exact revision.
The driver never invokes Cargo builds or edits runtime source.

```text
node experiments/departure-gate/renewal-removal-profile/profile.mjs register --output=NEW_CAPTURE_DIRECTORY --accepted-run=PRIOR_WITHDRAWAL_RUN_002
```

Registration refuses any existing output directory. Its separate registration
receipt binds the contract hash, which later phases, frozen identities and raw
attempts must match. This detects accidental drift, not a malicious editor who
rewrites both records. It freezes the driver hash,
fixture hashes and modes, command flags, compiler/host identity, matrix, timing
scope and aggregation. It also freezes the accepted executable, original build
observation/streams and source identity, and thirteen matching historical rows
with their raw stdout/stderr. These old rows provide finite semantic references
only. Their timing values never enter the new timing comparison.

Build the two current executables separately and preserve each before the next
build. Builds and focused validation finish before the coordinator starts timing.

```text
cargo build --release --locked --manifest-path runtime/Cargo.toml --example collector_profile --no-default-features --features collector-metrics
cargo build --release --locked --manifest-path runtime/Cargo.toml --example renewal_removal_profile --no-default-features --features renewal-removal-profile
```

The diagnostic feature includes collector metrics; do not also enable
`withdrawal-extraction-profile`. The ordinary harness remains byte-identical,
SHA-256 `1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357`.
Preserve actual build stdout/stderr, compiler identity, source snapshot and
command observation outside selected source. The build-observation JSON supplied
to `freeze` uses the coordinator's `build-profile.py` schema: `revision`, exact
`argv`, `status: 0`, `error: null`, `inputsStable: true`, `headAfter`, empty
`trackedDiffAfter`, `rustc`, `executableSha256`, `inputs`, `stdoutSha256` and
`stderrSha256`. Its sibling raw `build.stdout`/`build.stderr` are hash-checked and
copied with the verbatim observation. Those fields are observed build records,
not a compiler-authenticated attestation.

```text
node experiments/departure-gate/renewal-removal-profile/profile.mjs freeze --output=CAPTURE_DIRECTORY --kind=plain --exe=PRESERVED_PLAIN_EXE --revision=EXACT_PROFILING_SHA --build-observation=PLAIN_BUILD_JSON
node experiments/departure-gate/renewal-removal-profile/profile.mjs freeze --output=CAPTURE_DIRECTORY --kind=diagnostic --exe=PRESERVED_DIAGNOSTIC_EXE --revision=EXACT_PROFILING_SHA --build-observation=DIAGNOSTIC_BUILD_JSON
node experiments/departure-gate/renewal-removal-profile/profile.mjs run --output=CAPTURE_DIRECTORY
node experiments/departure-gate/renewal-removal-profile/profile.mjs summarize --output=CAPTURE_DIRECTORY
```

`freeze` requires a clean exact tracked head and records native source input
hashes: tracked runtime, game, embedded web source, toolchain, scripts, package
manifests and protocol files, plus existing runtime build script and root/runtime
Cargo configuration files. The build observation must
contain that same selected input manifest. Both executables must use the same head/input manifest. The coordinator
also preserves those raw source files. `run` verifies frozen source and artifact
hashes before each launch, appends to the attempt ledger before launching, and
writes exclusive raw stdout, stderr, metadata and result files. Failures and parse
errors remain recorded; the driver stops rather than silently rerunning them.
`summarize` runs no executable and creates a newly named arithmetic summary. It
requires all twenty-six attempts and rechecks raw streams and semantic equality.

## Harness schema and equality boundaries

Both native programs accept exactly:

```text
EXE SOURCE CYCLES MODE DRAIN_EVERY PROBES
```

`CYCLES` is 1..4000, drain interval positive, probes at most10000. These fixtures
use `release`, or `initialize` for reachable-chain. The source is augmented by
the unchanged `profile_probe` and renewable `profile_side`/`profile_mutate` text;
reported `source_sha256` hashes that augmented text. The registration separately
records the unmodified fixture hash.

Plain schema: `caveat-collector-native-profile/1`. Diagnostic schema:
`caveat-renewal-removal-diagnostic/1`, with `diagnostic_only: true`. Both expose
`growth`, `growth_last_10_percent`, `steady`, `unrelated_changes`, `failed_release`,
`release`, `before_release`, `after_release` and `archive`.

Each fresh plain row is checked against its preserved accepted reference. Each
diagnostic row is checked against its fresh plain pair. Equality covers augmented
source hash, phase sample counts, exact saved-string hashes and lengths, retired
record/withdrawal/undrained counts, complete ordered archive NDJSON hashes and
counts, and full successful `collector_work` counters. Hash comparisons are
finite observations; the native harness does not capture full snapshot/view text
or inspect error codes. It asserts the expected accepted/rejected apply outcome,
exact save rollback and no published archive after each rejected attempt.

Rejected `collector_work` is deliberately a zero placeholder in the original
harness. Its equality is not attempted-work evidence. The separate diagnostic
TLS capture supplies attempted renewal/readings work even when dispatch rolls
back.

## Timing, work and memory boundaries

Each diagnostic phase contains `removal.renewal_removal`, `removal.readings`,
`removal.departure_ns` and `removal.small_phase_samples`. Per-path fields are
`blocks`, `searches`, `removes`, `matches`, `misses`, `probes`, `shifted_elements`,
`estimated_shifted_header_bytes`, `shared_cow_detaches`, `cow_cloned_histories`,
`cow_cloned_occurrences`, `block_ns`, `search_ns`, `cow_ns`, and `remove_ns`.
Raw small samples expose `ns`, heap peak, existing work and the same `removal`
object. All successful/rejected release samples are retained, not reconstructed.

The renewal timer begins inside the already identified renewal branch, before
the occurrence-vector position search, and ends after removal. It excludes the
immutable history-map lookup and journal/reading classification. The shared
`Arc<BTreeMap<String, Renewal>>` detachment clones the whole renewal map and its
occurrence vectors, not merely the current vector. COW inventory counts use
vector lengths after the timed block; those diagnostic scans remain within
whole apply/departure time. COW counters/timing cover detachments at this removal
boundary only: earlier compaction can already detach a map, so zero here does not
mean zero COW during the dispatch. Inventory counts describe history/occurrence
populations, not every cloned payload or byte. Exact search probes come from found index+1 or the
missed vector length, without per-predicate counter increments. Shift count is
length-index-1; byte estimates cover moved String headers, not string payloads or
measured allocator traffic. Reading-stream accounting stays separate; this
corpus does not establish reading-stream performance.

Search, COW and removal timers are nested portions of the block timer. Do not add
them to block time again or assume their sum partitions every cost exactly.
Whole-departure time is an enclosing interval. Whole native apply includes the
transaction and rejected rollback, allocator accounting and collector counters;
it excludes later diagnostic reads, save serialization, archive drain/NDJSON
hashing, snapshot serialization, browser and WASM. Rejected rollback follows the
departure interval, so its contribution remains in apply time alone.

The plain control disables the new attribution feature but retains the original
allocator and collector metrics. Diagnostic clocks, bookkeeping, inventory scans
and larger fixed sample working sets perturb timings. Compare fresh paired apply
times to expose combined perturbation; it is not a calibrated quantity to
subtract. Existing sample vectors are allocated before `base_heap`, and the one
release buffer is separately corrected. Preserve that order. Report every phase's
plain/diagnostic peak and retained heap difference without a memory pass/fail
threshold. Requested heap excludes allocator metadata, stacks and RSS.

## Arithmetic and interpretation

For each primary process show all successful/rejected samples, block/apply and
block/departure ratios, nested COW cost, probes, shifts and clone populations.
Reduce the three rejected samples to a median within each process, then show
min/median/max across the four process values. Do not pool twelve failures as
twelve independent fresh sessions. Also show matched diagnostic/plain apply
ratios and absolute heap differences for all phases and all pairs.

Across processes the median averages the two central values. The unchanged
harness uses upper-middle within even-sized phase samples. Never divide a sum
of phase block times by a single median apply time. `summarize` keeps aggregate
phase work distinct from ratios calculated on the raw small samples.

The result should answer whether the observed absolute cost and scaling make
renewal occurrence removal a plausible next engineering priority. A small
contribution or excessive diagnostic perturbation is an informative outcome.
Measured block time is not a prediction of the benefit from any replacement.
No implementation, extra timing population or performance goal follows
automatically from this profiling-only study.
