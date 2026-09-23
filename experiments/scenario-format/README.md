# Scenario format evidence (experimental)

This directory preserves the evidence behind
[Scenarios 0.1](../../spec/caveat-scenarios-0.1.md) as first drafted. It is
not the developer kit. `prototype-runner.mjs` was a throwaway used to execute
the spec's examples; the kit's runner replaces it and keeps these files as
fixtures.

The prototype predates the review corrections in `f8b289d`. It does not refuse
`before` ahead of the first `send`, does not count repeats in `$includes`, and
does not refuse non-finite payload numbers. Its results below are what it
reported; they are not a conformance claim for the corrected spec.

## Contents

| Path | What it is |
| --- | --- |
| `prototype-runner.mjs` | The throwaway runner. `node prototype-runner.mjs <pkg-reactive dir> <file>` |
| `examples/` | The spec's two examples, pointing at `examples/thermostat_history.cav` and `game/trail_rescue.cav`. |
| `faults/` | Altered copies and hand-written probes, `faults.cav`, and `make-fixtures.mjs`, which regenerates the altered copies. |
| `results-prototype.json` | Every file's exit status and output, with the runtime identity. |
| `glowcap-origin-audit.mjs` | Replays Glowcap's `cr12` histories on `caveat5/glowcap.cav` through `dispatch_outcome`. |
| `results-glowcap-origins.json` | That audit's output, with runtime and source hashes. |

## Results

Recorded on 2026-09-23 against the clean compiled reactive build of `541e09d`
(`x86_64-pc-windows-msvc`; reactive WASM SHA-256
`0b03a2419ead6d4f53c3d89d1dd071a8eed0de31ce81a2045f56db227ec2fa6b`).

| Exit | Files |
| ---: | --- |
| 0 | Both examples; `neg-pass-lineage-as-set`, `neg-evaluation-bound`, `neg-repeat` |
| 1 | `neg-wrong-value`, `neg-journal-order`, `neg-lineage-order-matters`, `neg-bare-rejected-on-input`, `neg-wrong-policy-message`, `neg-expected-accept-got-reject`, `neg-same-as-before-changed`, `neg-fatal-not-a-rejection`, `neg-size-bound` |
| 2 | `neg-invalid-unknown-field`, `neg-invalid-host-origin`, `neg-invalid-input-message`, `neg-invalid-absent-false`, `neg-invalid-mixed-matcher` |

The Glowcap audit found 21 expected rejections: 15 `policy/reject`, 2
`input/bound_exceeded` and 4 `input/payload_invalid`. All left the save and
view unchanged. The replay accepted 24,315 events with no fatal outcome. It is
a supplementary run on a newer runtime, not a rescore of Glowcap's recorded
rounds; Glowcap's own harness and scoreboard are unchanged.

## Reproduce

After `npm run build`, from this directory:

```sh
node faults/make-fixtures.mjs
node prototype-runner.mjs ../../dist/pkg-reactive examples/trail_rescue.scenarios.json
node glowcap-origin-audit.mjs
```

A different build host or runtime revision gives different hashes. Compare
outcomes, not bytes, unless the build identity matches.
