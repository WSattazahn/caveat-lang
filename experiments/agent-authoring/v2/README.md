# Fresh-agent clock-task follow-up

Four new author contexts repeat v1's two clock tasks with updated documentation
and a runtime exposing `elapsed()`. This measures the joint change on repeated
tasks, not isolated causality or performance on unseen tasks. v1 is preserved.
The [protocol](PROTOCOL.md) fixes four runs, three versions and twelve checks
per run. [study.json](study.json) pins the runtime/documentation revision.

## Prepare and register before authors

1. Commit the updated runtime, guide and new elapsed specification. Put its full
   commit ID in `study.json` and the new spec path in its reference list.
2. With that revision checked out and tracked files clean, run `npm ci` and
   `npm run build`. Do not use `--assemble-only`. The build metadata must identify
   the exact pinned revision and a clean compiled build. Untracked v2 preparation
   files do not alter runtime provenance.
3. Run the commands below. Review the public tool and private oracle/scorer.
   A root-only `runs/SMOKE` may validate the tool; it is never an author result.
4. Commit all v2 registered files before dispatch. Save that registration commit,
   actual dispatch timestamps, fresh agent IDs and `forkTurns: "none"` in
   `launches.json`. Dispatch exact prompts from `prompts/` in waves A1/B1, A2/B2.
   Never send private outcomes during authoring.

```sh
node experiments/agent-authoring/v2/prepare-packet.mjs
node --test experiments/agent-authoring/v2/checks.test.mjs experiments/agent-authoring/v2/infra.test.mjs
node experiments/agent-authoring/v2/register.mjs
```

Registration refuses to overwrite a prior registration or run after any author
directory exists. It verifies byte-identical v1 contracts/oracle, 114/116 exact
case histories, packet hashes and build provenance. It hashes the protocol,
guide transformation, corpus, tooling, tests and prompts. Generated references
and runtime are ignored in Git; the manifest freezes their exact bytes.

The guide's final `## Evidence from fresh authors` section is removed by the
registered [transformation](packet-transform.mjs). Its original and resulting
hashes are recorded in the manifest; all preceding bytes stay intact. Other
references are copied byte-for-byte from the runtime commit.

## Score only after all four finish

```sh
node experiments/agent-authoring/v2/audit.mjs
node experiments/agent-authoring/v2/verify.mjs
```

The scorer first checks registered inputs and every finished source archive.
It then evaluates both first/final sources against exact policy, admission,
grounds, atomicity, restore, snapshot elapsed and sequence. The default output
files use exclusive creation to preserve the original results. Candidate
failures are outcomes, not verifier crashes. The source review and interpretation
rules are in the protocol. No result is implied by the existence of this setup.

## Reproduce after completion

Use Node 20 or later, the Rust version in the pinned commit's
`rust-toolchain.toml` and wasm-bindgen CLI 0.2.104. Build the pinned revision in
a separate ordinary checkout (no Git worktree required), using the registered
build-host platform shown in `packet/manifest.json`. Preserve the study checkout;
do not switch a dirty checkout to an older revision. Copy that build's `dist/`
into the study checkout, then run `prepare-packet.mjs`. The study checkout's
runtime/build inputs must also still match the pinned commit; if the repository
has advanced, reproduce at the v2 registration/results revision instead.

Windows and Linux builds are not promised to have identical WASM bytes. A
different-host build failing the manifest check is a pin mismatch, not a
candidate regression. Do not replace the manifest. A previously preserved
packet with matching hashes is also usable directly.

```sh
node --test experiments/agent-authoring/v2/checks.test.mjs experiments/agent-authoring/v2/infra.test.mjs
node experiments/agent-authoring/v2/audit.mjs --output test-results/agent-authoring-v2-integrity.json
node experiments/agent-authoring/v2/verify.mjs --output test-results/agent-authoring-v2-scored.json
```

CI can always run the pure oracle/infra tests. Full study reproduction must use
the exact frozen packet/build platform; the existing Linux current-HEAD build
must not overwrite this packet or be passed off as the registered runtime.
The integrity audit cannot prove what files an agent read. Preserve each
author's disclosure and any observed isolation breach alongside scores.
