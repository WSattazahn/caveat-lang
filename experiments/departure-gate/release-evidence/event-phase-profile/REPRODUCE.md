# Reproduce and inspect the event-phase investigation

Source revision: `11c2a6a741609d3b640f27224ab6c9943b07f72c`.
Runtime Git tree: `89f3a5bde145a3071ce6e41652a866e6b722f3b1`.
Accepted language/runtime baseline: PR #177 at
`055e2af0b411d8385941dd8522201e5afc73cb34`.
The accepted native reference was built at `8a161dcdfdc8a2a5b3ca43b4183552401002cca7`,
whose runtime tree matches that baseline; it is not mislabeled as a new build.

## Start with saved evidence

Read `REVIEW.md`, `measured-findings.md`, `allocation-methods.md`, and
`phase-boundaries.md`. `run-001/contract.json` fixes the population and definitions;
`attempts.jsonl` plus `capture/*.json`, `*.stdout`, and `*.stderr` preserve each
attempt. All times are observations from one Windows host, not promised budgets.
The ordinary controls retain their original allocator counters and emit summary
statistics. The timing and origin builds additionally emit all individual events.

The ZIP's `SHA256SUMS` covers its members. Its `prior-677/` directory contains the
full earlier bundle, exact 055-to-677 patch, directly extracted raw rejected
samples, and a replay of the previously uncheckable 5.516% share. Follow
`prior-677/README.md` for that evidence. Do not confuse the earlier measurements
with the fresh controls in this investigation.

The two data-only scripts can be run with Python 3, without Rust or the native
executables. Use new output names; they refuse overwrites:

```
python -B validate_event_profile.py --input run-001/capture/release-mutual-3000-t1-timing.stdout --mode timing
python -B validate_event_profile.py --input run-001/capture/release-mutual-3000-t1-origins.stdout --mode origins
python -B analyze_event_phases.py --capture run-001 --json replay-findings.json --markdown replay-findings.md
```

`measured-findings.json` records hashes of its raw inputs. Analysis timestamps and
absolute locations can differ on replay; measured values and their aggregation
must agree. Arithmetic consistency does not authenticate the recorded executions.

`final-independent-review.py --check --output final-independent-review-v2.json` additionally replays the recorded audit,
source and layout-probe identities against the original live checkout and earlier
local evidence paths. It is read-only, but is a local closeout check, not a portable
bundle-only verifier. `final-independent-review-v2.json` records those boundaries.
The later `data-verification-v2` receipt records this check over the final report and
evidence. Its acceptance does not establish any proposed optimization's outcome.

`layout-probe/` includes a source-only type-layout experiment and every attempt.
The successful `attempt-003` pins Rust 1.98.1 and links the frozen runtime rlib.
It creates no session and dispatches no events. To reproduce on another host,
rebuild that revision and adapt the explicit dependency/library paths in `run.py`;
type sizes can differ with compiler and target. The printed vector-size arithmetic
is not an observed peak-memory reduction.

## Inspect or rebuild source

`review-source.bundle` is a local Git bundle with accepted PR177 as prerequisite.
In a separate ordinary clone that contains that baseline (no worktree is needed),
inspect the bundle, fetch its named branch, and check out the review commit:

```
git bundle verify PATH/TO/review-source.bundle
git fetch PATH/TO/review-source.bundle refs/heads/codex/event-phase-profile
git switch --detach 11c2a6a741609d3b640f27224ab6c9943b07f72c
```

`builds/{plain,diagnostic,origins,wasm}/` contains the actual build command,
compiler identity, observed streams, and selected source-byte snapshots. Native
builds also preserve their executable. These native binaries are Windows x64
artifacts. `current-artifacts/` preserves the ordinary WASM build and build-info.
The source snapshot captures local line endings as compiled; Git can normalize
line endings in a fresh checkout. A fresh build must record its own file hashes.

The repository's `experiments/departure-gate/event-phase-profile/README.md` gives
the three locked release build commands. The supplied `build-profile.py` records
and snapshots each build, using the original `C:/Dev/caveat-lang` checkout path;
review/adapt that path for another host. It refuses an existing build destination.
Rust 1.98.1/MSVC and Node 24.11.1 were used here. Cargo dependencies must already be
available or fetched normally; no hidden runtime dependency is bundled with the
native command itself.

## Rerun the fixed population

Use fresh build/capture directories. The original accepted reference material is
inside `prior-677/Caveat_Renewal_Profile_677ad136.zip` under
`accepted-pr177-input/`. Its executable identity is verified during registration.
Pass that extracted directory to the driver. A fresh reference process is run
for each cell solely for finite semantic comparison; its times are not included
in the control/diagnostic comparisons.

```
python -B experiments/departure-gate/event-phase-profile/profile.py freeze --root=CHECKOUT --output=NEW_CAPTURE --builds=NEW_BUILDS --accepted-input=EXTRACTED_ACCEPTED_INPUT
python -B experiments/departure-gate/event-phase-profile/profile.py run --root=CHECKOUT --output=NEW_CAPTURE --accepted-input=EXTRACTED_ACCEPTED_INPUT
python -B experiments/departure-gate/event-phase-profile/profile.py validate --root=CHECKOUT --output=NEW_CAPTURE --accepted-input=EXTRACTED_ACCEPTED_INPUT
```

Do not run builds/tests during native capture. The driver preserves every attempt
and stops on failed provenance, execution, semantic, or accounting checks. Do not
erase or silently retry a failed capture. A new registration is a distinct run.

## Correctness receipts and limits

`verification.plan.json`, `verification-summary.json`, `public-verification/`, and
the final audit preserve exact local checks, command streams and bounded coverage.
The independent WASM comparison uses the accepted ordinary artifact from the
prior ZIP's `baseline-artifacts/pkg-reactive` directory. The verification helper
records the original local paths; review its explicit root/runtime/tool paths
before reissuing receipts on another host. Private issuer keys, signed live
stores, and signed run logs are intentionally excluded. Unsigned observations
are inspectable evidence, not transferable trusted receipts.

The tracked repository has no general `typecheck`, `lint`, `test`, or `audit:all`
npm scripts. The recorded checks use this Caveat repository's actual native/kit/
WASM commands, not unrelated game-project commands from the surrounding workspace.
This local diagnostic commit has no new GitHub CI run. Preserved PR metadata
reports earlier PR heads and checks only. Nothing here authorizes an optimization,
merge, publication, release, or a universal bounded-memory/workload claim.

## Closeout correction

The first closeout report said preparation began with shallow cloning. Final
source review corrected this to include the preceding input validation and
clarified inline archive staging in the source inventory. No source, raw run,
measurement or numerical recommendation changed. The earlier report/helper copies
remain in `superseded-closeout-v1/`; original closeout receipts remain historical.
Use `final-independent-review-v2.json` and `data-verification-v2.audit.json` for
the corrected report. The runtime verification audit still covers the same source.
