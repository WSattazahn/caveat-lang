# Reproduce the high-degree confirmation

Start with an intact extracted bundle. Python 3 is required for saved-data checks; Windows is required to execute the included native artifacts. Keep `profile.py`, `native_helpers.py` and `PROTOCOL.md` together. Never overwrite the registered run or append samples to it.

## Check the recorded result without running Caveat

From the bundle root:

```text
python -B review/verify-capture.py --self-check
python -B review/verify-capture.py --check --output review/independent-capture-review.json
```

The first command exercises 15 data-only checker controls. The second recomputes the frozen population, independently checks the official arithmetic, verifies preserved input identities and checks the exact report binding. A successful-release trigger is an expected finding, not checker failure. The checker does not import the measurement driver, execute Git or access the original checkout. It was executed on Windows; no Linux replay run is claimed.

To create a separate receipt, choose an unused filename:

```text
python -B review/verify-capture.py --output review/my-replay.json
python -B review/verify-capture.py --check --output review/my-replay.json
```

Existing `analysis-*.json` and raw streams remain authoritative inputs. Running `python -B profile.py analyze --out run-001` creates an additional timestamped analysis; that is a new derivative file, not another measured process. Prefer the read-only checker when preserving the shipped manifest.

## Recover exact source without a network fetch

Choose a fresh destination outside any live checkout:

```text
git clone --branch codex/archive-early-boxing prior-context/review-source.bundle review-clone
git -C review-clone rev-parse HEAD
git -C review-clone diff 11c2a6a741609d3b640f27224ab6c9943b07f72c 590fae59aa4295ac5209f930a1e58ac152bd4603 -- runtime/src/reactive_departure.rs
```

Expected HEAD is `590fae59aa4295ac5209f930a1e58ac152bd4603`. The bundle has complete reachable history under the explicit branch ref; do not rely on a bundle HEAD ref. The preserved full baseline-to-candidate patch also contains the earlier experiment's diagnostics/documentation. The new confirmation itself makes no source change.

Fresh registration additionally requires the raw bytes of all 135 candidate source inputs to match their preserved build manifest. Git's checkout line-ending conversion can produce the right commit but different bytes. If that happens, copy the files under `run-001/prior/build-identity/after/source/` to the same repository-relative locations **only in this fresh review clone**. Keep the evidence snapshots untouched. Require `git -C review-clone diff --quiet` to succeed; registration then verifies every raw input hash, the tracked-clean state and exact revision before permitting capture. Do not bypass that check or apply this restoration to a live working checkout. Saved-data replay does not require a clone or any line-ending conversion.

## Make a distinct fresh measurement, only when authorized

This is a reproduction recipe, not a request to execute extra samples. Ensure no controlled builds, tests or compression jobs compete with timing. Use fresh absolute output paths and a clean tracked checkout at the candidate revision. Substitute concrete paths for the angle-bracket placeholders:

```text
python -B profile.py controls --out <fresh-controls> --prior <bundle-root>/run-001/prior
python -B profile.py register --out <fresh-run> --prior <bundle-root>/run-001/prior --controls <fresh-controls>/controls.json --repo <candidate-checkout>
python -B profile.py run --out <fresh-run> --repo <candidate-checkout>
python -B profile.py analyze --out <fresh-run>
```

The copied `run-001/prior/` contains every transitive input needed by registration, including both exact executables, their 135-input snapshots and raw build streams, original fixture/harness, identities and the earlier high-degree reference outputs. No private store, original absolute evidence directory or Rust rebuild is required. Saved absolute Windows argv is checked as historical data, not executed to locate files. Each new capture runs the included exact executables at their new registered locations.

Fresh capture must still preserve every attempt, alternating order and eight-pair population. Source/artifact/control mismatches refuse capture. A failed or interrupted run is evidence to preserve, not a reason to silently reuse its output directory. Newly authored future comparisons require their own registration and interpretation.

## Identities

- BEFORE executable SHA256: `2286a87f01560aed8f12ee4f5a6f8f5116f5ea79b4d3bc8f72e6016afae9a95f`.
- AFTER executable SHA256: `64125fda7f7323469f9d31f4ac5f787bf3b817856a38d25af2b653044be643d2`.
- Registered contract SHA256: `7783052f1d2df037aec2c28087fedb93f4af341cff74cd2fa5f106fa48a2a495`.
- Attempts ledger SHA256: `223c210244c015b67e27b771620adfa9e7cbbf0ef75fc38bac1b1fb5e540f586`.
- Source bundle SHA256: `4104c0c7638e91301caa87e271ca1878784192ebbb4c47590aa9dc19ef308d4b`.

`SHA256SUMS` covers all other regular bundle members. Hash consistency is not authenticated execution history. CAVEAT public receipt exports describe observed local data checks; private issuer state is intentionally absent, so they are not portable signed proof.
