# Structured dispatch implementation and audits

The reactive core and WASM bridge now expose the
[Dispatch Outcomes 0.1 contract](../../spec/caveat-dispatch-0.1.md), implemented
at `541e09d61f131dbd25e546f96600e12d845f3592`. Classification is carried from the
failure site through rule/procedure context. The old dispatch APIs retain their
behavior and diagnostic text. The new API returns accepted/rejected values;
unclassified errors and escaped exceptions are fatal.

Payload bounds are `input/bound_exceeded`; state bounds are
`evaluation/bound_exceeded`. Codes are additive within the schema; changing an
existing meaning or origin requires a new schema. The initial catalog is
deliberately narrow: arithmetic, `require`, and history-capacity failures remain
unclassified fatal errors for new callers. A bare expected rejection means
policy only. The future scenario runner must enforce that interpretation.

## Evidence

- [Author-log audit](AUTHOR_LOGS.md): no candidate reruns. Of 415 recorded
  rejected events, 160 are source-corroborated authored-policy text and 255 are
  other error text. This is retrospective inference, not structured attribution
  or a rescore. Invalid-input probes and history limits are among the other
  errors; their count is not a bug count.
- [Trail Rescue audit](TRAIL_RESCUE.md): a new supplemental run of the unchanged
  24 registered histories with the new runtime. Its 151 dispatches include 118
  acceptances, 20 policy rejections, 2 input rejections, and 11 adapter
  exceptions. No fatal outcomes occurred. Adapter exceptions are recorded
  separately and cannot establish a policy rejection. Restored shadows are
  counted separately; 11 explicit and 24 final restores passed.
- [WASM checks and build hashes](results-wasm.json): all 18 checks passed across
  the full and reactive-only bundles built from the clean committed runtime.
  The report preserves compiler/host metadata, JS/WASM hashes, fixture hashes,
  and test-script identity. This local Windows build is not a claim of matching
  Linux release bytes.

The original Glowcap, Trail Rescue, and v1/v2 study artifacts are unchanged.
These reports do not replace their harnesses or rewrite historical scores.
The scenario inventory and format remain separate work.

## Validation on 2026-09-23

| Check | Result |
| --- | --- |
| `cargo fmt --manifest-path runtime/Cargo.toml --all -- --check` | Passed |
| `cargo clippy --manifest-path runtime/Cargo.toml --all-targets -- -D warnings` | Passed |
| `cargo test --manifest-path runtime/Cargo.toml --all-targets` | 471 passed, 0 failed |
| `npm run build` | Both release WASM bundles compiled from clean `541e09d` |
| `npm run test:dispatch-outcomes` | 18 checks passed |
| `npm run test:elapsed-clock` | 7 checks passed |
| `npm run test:slime-glow` | 10,001 toggles; bounded graph checks passed |
| `npm run test:glowcap` | Explanations, decisions, late caveats, and rejection checks passed |
| `npm run test:trail-rescue` | Original 24 scenarios and 5 additional checks passed; seeded run: 16,000 events, 3,110 restores |
| `npm run test:glowcap-page` | Chromium playthrough passed |
| `npm run test:trail-rescue-page` | Chromium desktop/mobile playthroughs passed |
| `node --test experiments/dispatch-audit/author-logs.test.mjs experiments/dispatch-audit/trail-rescue.test.mjs` | Both audit fixtures passed |

Current browser screenshots were inspected after the tests. Browser checks use
the unchanged production adapters and legacy dispatch APIs; the new wire is
exercised by the dedicated WASM checks and the supplementary Trail audit.
The complete CI browser matrix and Linux reproducibility job have not been
rerun locally. The new WASM checks are included in runtime CI.

For reruns, build before executing the WASM checks or Trail audit. The author-log
audit needs the preserved study packets and Git history, and does not use the
current WASM. Audit scripts write only their own supplementary reports.

The [consolidation plan](../../docs/CONSOLIDATION_PLAN.md) inventories the
historical branch stack, preserves experiment commit identities, and defines
release gates. No main merge, release tag, or package publication is part of
this implementation.
