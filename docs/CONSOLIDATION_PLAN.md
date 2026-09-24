# Consolidation and first release plan

Status: proposed operations only. This document does not merge branches, change
PRs, create tags, publish packages, or certify a release. The inventory below was
verified on 2026-09-23 against local Git objects, live `git ls-remote` output and
read-only GitHub PR metadata. Its baseline is `3980c3d`, before any subsequent
cleanup commits. Recheck moving refs before execution.

## Verified state

- Repository: `https://github.com/WSattazahn/caveat-lang.git`.
- Shared checkout at historical inspection: `C:/Dev/caveat-lang`, branch
  `codex/elapsed-clock`. Follow-up implementation uses `codex/dispatch-outcomes`.
- `main` and `origin/main`: `859db9a72a9ea2bead114095b66d04cbeeb198cb`.
- `HEAD` and `origin/codex/elapsed-clock` at inspection:
  `3980c3de2589f126fde237d9941300e3d2d94c09`.
- `main` is an ancestor of that head. There are **111 commits** reachable from
  the head but not from main, including **12 merge commits**. These are a stack
  with shared ancestry, not 111 independent changes or independent branches.
- Local and live origin tips matched for every branch in the inclusion table.
- No local or origin tags were returned. The root npm package is private
  (`caveat-lang-games`, version `0.1.0`); it is not a publishable runtime kit.
  The Rust crate also declares `0.1.0`, which does not establish a release tag.
- The checkout was initially clean. At evidence-file creation, another task had
  modified `runtime/src/reactive.rs` and added `runtime/src/reactive_outcome.rs`.
  Those working changes are outside the historical inventory and were untouched
  by this planning task. They must enter the eventual reviewed candidate.

Follow-up: commit `541e09d` adds structured dispatch outcomes on
`codex/dispatch-outcomes`, directly after the historical baseline. Its contract
is [Dispatch Outcomes 0.1](../spec/caveat-dispatch-0.1.md). Include that commit
and the supplementary audit/plan commits in the final review; the historical
111-commit count above deliberately excludes this follow-up work.

[Machine-readable evidence](consolidation-evidence-2026-09-23.json) records full
commit IDs, all origin heads, every stage's exact commit list, pre-write status
and the PR check URLs. Counts below exclude main and every preceding stage, so
each of the 111 commits is counted exactly once.

| Review stage | Included tip | New commits | Resulting coverage |
| --- | --- | ---: | --- |
| Modules, repetition, authorship, borrowed uncertainty | `feat/modules-0.5` @ `72b7c32` | 14 | Module/linker pipeline and fixes |
| Grounds, explanations, typed parameters, view, late qualification, journal, lean WASM, explainer | `feat/grounded-explanations` @ `70d4c7b` | 13 | First 27 commits; existing draft PR #11 |
| Round-six runtime response | `feat/round6-language` @ `6c4b504` | 8 | Incremental evaluation, symbol parameters, renewal, save/restore, exact numeric parsing |
| Glowcap experiment history | `experiment/glowcap` @ `d8a8ff5` | 65 | Registered rounds, adapters, measurements, corrections and replay evidence |
| Trail Rescue | `codex/trail-rescue` @ `04f72dc` | 3 | Registered mechanic, state-ground reopening, game and QA |
| Fresh-author study v1 | `codex/agent-authoring-study` @ `156bdc2` | 4 | Registration, scorer correction, six trial archives, guidance |
| Elapsed clock and study v2 | `codex/elapsed-clock` @ `3980c3d` | 4 | Runtime clock, signed-zero check, follow-up registration and four trial archives |
| **Total** | | **111** | |

Every tip in the table is an ancestor of the next one. There is one important
base nuance: current main is **not** an ancestor of the modules tip. Modules
lacks `eec557c` and merge `859db9a`, the reproducible-WASM changes later merged
into grounded explanations. Therefore `modules..grounded` alone counts 15,
including those two already-on-main commits; the table correctly counts 13.
Use PR #11 for the combined modules/grounded integration rather than trying to
fast-forward current main directly to the modules tip.

Three other origin branches are already ancestors of main and need no merge:
`codex/last-beacon-game` @ `1f56d70`, `fix/enforce-lf-checkout` @ `5c97020`, and
`fix/reproducible-wasm` @ `eec557c`.

The following origin tips are **not** ancestors of either main or the inspected
elapsed head, and are excluded from this plan:

| Origin branch | Tip |
| --- | --- |
| `caveat-action-plans` | `98c1f00` |
| `cleanup/stabilize-caveat` | `038ee58` |
| `feature/caveat3d-moon-garden` | `8a497e4` |
| `feature/caveat3d-semantic-actions` | `8e1a219` |
| `feature/moon-garden` | `108b467` |
| `feature/mr-caveat-feel` | `556b83b` |
| `feature/mr-caveat-missing-dumpling` | `fb0eef8` |
| `feature/mr-caveat-real-3d` | `0436056` |

Ancestry alone does not determine whether their content was superseded or
integrated with different commit IDs. Audit them separately before deleting,
merging, or claiming that every repository branch has been consolidated.

## Existing PR and recommended integration

[PR #11](https://github.com/WSattazahn/caveat-lang/pull/11), **Caveat: code that
knows why (explanations, grounds, late caveats, decision journal, glowcap
explainer)**, was OPEN, draft, MERGEABLE and CLEAN at inspection. Its base is
main and its exact head is `70d4c7b3e67f8af8e1dcbcf7b72f0fac860966f2`.
It covers the first 27 commits, including modules. It does not include the
remaining 84 commits: round-six runtime work, the experiment branch's history,
Trail Rescue, either authoring study, or `elapsed()`.

All 12 checks reported success at that PR head: rustfmt, regression scope, core
semantics, reproducible WebAssembly, Glowcap explainer browser QA, Last Beacon,
Light the Way Chromium/WebKit, Mr. Caveat iPhone, Moon Garden 2D/3D, and Legacy
Door. These checks apply to that PR revision. They do not validate the latest
stack, current uncommitted changes, or a future merged candidate.

Recommended sequence, to execute only when integration is authorized:

1. Finish and commit the current cleanup independently. Record its new full
   head and update this inventory if ancestry or scope changes. Do not switch
   or reset the shared checkout while another task is using it.
2. Review the seven logical stages above. Keep #11 as the review of the initial
   27 commits; update its stale validation notes only after current checks.
3. Integrate #11 with a **merge commit**, preserving its original parents and
   commits. Then review and merge the later tips in dependency order: round-six
   language, Glowcap history, Trail Rescue, v1, and elapsed/v2 plus cleanup.
   Subsequent PR diffs must be checked against the actual updated main; historical
counts are not promises about future diff sizes.
4. Use merge commits for the later integrations too. If policy requires the
   head to contain current main, merge main into the review branch; do not rebase
   the experiment stack. Conflict resolutions require renewed semantic and
   browser validation.
5. A single consolidation PR from the final descendant is also structurally
   valid if the reviewer prefers one merge: it contains the full stack and
   preserves its original commits. In that case mark #11 as superseded only
   after the consolidation is accepted, and retain the seven-stage review map.
6. Verify all included historical tips are ancestors of final main before
   removing any branch refs. Keep excluded legacy branches outside cleanup.

Do not squash, rebase, or cherry-pick this stack as a substitute for integration.
Those operations create different commit identities and can strand the
historical objects used by `git show`, preregistration records, manifests and
change-cost measurements. Retaining original commits through merge ancestry
lets ordinary full clones reproduce the studies after feature refs are removed.

Read-only checks to repeat before approving an integration, from the repo root:

```powershell
git status --short
git ls-remote --heads origin
git rev-parse HEAD main origin/main
git rev-list --count main..HEAD
git log --graph --oneline --decorate main..HEAD
git log --reverse --format="%h %s" feat/grounded-explanations --not main feat/modules-0.5
gh pr view 11 --json state,isDraft,baseRefName,headRefOid,mergeable,mergeStateStatus,statusCheckRollup
```

After authorized merges and refreshing local refs, `git merge-base
--is-ancestor <full-historical-tip> origin/main` must exit zero for all included
tips. Use the immutable IDs from the evidence file, not just moving names.

## Candidate and release gates

The following are proposed acceptance gates. This document records no new gate
results. Use the actual final committed revision, a clean build, Rust **1.98.1**,
wasm-bindgen CLI **0.2.104**, lockfiles and Node **22** as current CI does. Keep
full and reactive-only runtimes under test. Do not use `npm run assemble` to
claim a release was compiled from its declared revision.

```powershell
npm ci
cargo fmt --manifest-path runtime/Cargo.toml --all -- --check
cargo clippy --manifest-path runtime/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path runtime/Cargo.toml --all-targets
cargo clippy --manifest-path runtime/Cargo.toml --all-targets --no-default-features -- -D warnings
cargo test --manifest-path runtime/Cargo.toml --all-targets --no-default-features
npm run build
npm run test:slime-glow
npm run test:elapsed-clock
npm run test:dispatch-outcomes
npm run test:glowcap
npm run test:trail-rescue
node --test experiments/glowcap/compare-views.test.mjs experiments/agent-authoring/v1/checks.test.mjs experiments/agent-authoring/v2/checks.test.mjs experiments/agent-authoring/v2/infra.test.mjs
node experiments/glowcap/replay-divergences.mjs
```

Root `package.json` has no `typecheck`, `lint`, `test`, or `audit:all` scripts at
the inspected head. The Rust/Node commands above are the repository's relevant
checks; missing scripts must not be reported as passing. Current runtime CI
does not run the extra no-default-features lint/test pair or author-study pure
tests shown above. Treat those as added release expectations unless the workflow
is separately updated and reviewed.

Run `.github/workflows/runtime.yml` at the final review head and final main.
Besides the commands above it contains CLI/Map playthroughs and the browser
matrix. Its reproducibility job builds on Ubuntu 24.04 from another checkout
path and cargo home and compares byte-for-byte:

- `pkg/caveat_runtime_bg.wasm` and `pkg/caveat_runtime.js`;
- `pkg-reactive/caveat_runtime_bg.wasm` and `pkg-reactive/caveat_runtime.js`;
- `build-info.json`, plus a check against builder paths in the full WASM.

Local browser equivalents for the scripted game checks:

```powershell
npx playwright install chromium webkit
npm run test:glowcap-page
npm run test:trail-rescue-page
npm run test:beacon
npm run test:rescue
npm run package:game
npm run test:launch
```

On Ubuntu runners use `npx playwright install --with-deps chromium webkit`.
The remaining Mr. Caveat, Moon Garden 2D/3D and conditional Legacy Door tests
are embedded in the workflow; do not omit them because they lack npm aliases.
Runtime changes require Legacy Door's regression scope. Inspect fresh game
screenshots and actually exercise desktop/mobile input, explanations,
reopening, reset/restore, and launch failure/retry. The user retains final
interactive visual judgment.

For the Glowcap replay, use the retained float-divergence fixture and both
registered replay seeds, including ordered-decision comparison:

```powershell
node experiments/glowcap/differential.mjs --round=6 --sequences=2000 --seed=6 --against=caveat5 --ordered-decisions
node experiments/glowcap/differential.mjs --round=6 --sequences=2000 --seed=7 --against=caveat5 --ordered-decisions
```

These are current-runtime compatibility checks, not a rewrite of historical
scores. If rerunning the full historical harness/benchmarks, use an ordinary
separate clone and [the recorded commands](../experiments/glowcap/RESULTS.md):
`harness.mjs` appends to tracked `runs.jsonl`. Do not accidentally include new
measurement logs as original trial evidence. Run timings after fuzz finishes.

### Pages deployment gap

The original consolidation workflow rebuilt and deployed Pages independently
of the full runtime/browser checks. The follow-up [Pages deployment
gate](PAGES_DEPLOYMENT.md) consumes the exact tested distribution after the
complete runtime workflow succeeds, checks the current main revision again
before deployment, and adds live Glowcap and Trail Rescue smoke checks.
It retains the existing Mr. Caveat, Moon Garden 2D/3D and conditional Legacy
Door live checks.

The gate's first run on the default branch must pass before tagging a release
candidate; a pull request cannot prove the `workflow_run` deployment trigger.
A release is not complete until both final-main workflows succeed and the
deployed revision/build hashes match. Also run live smoke checks for the
retained Beacon/rescue games, including desktop/mobile screenshots. Local
success is not evidence about the public URL.

## Preserve and reproduce registered studies

Preserve protocols, original source versions and line endings, prompts,
dispatch metadata, scorer corrections, manifests, results and disclosures.
Do not run registration again, change expected hashes to accept a new build,
or run a formatter over archived author submissions.

| Evidence | Commit dependency and constraint |
| --- | --- |
| Glowcap | Initial registration `cfcc881`, blind CR5–8 `6409b13`, blind CR9–12 `653bf24`; round-six language freeze `70d4c7b`; replay registration `9f8c346`; final retained evidence `d8a8ff5`. Preserve per-phase commits because change-cost scoring uses their diffs. |
| Trail Rescue | Requirements commit `a03af45` precedes implementation; state-ground runtime change `c5c0183`, game/results `04f72dc`. Current harness success does not turn this into a comparative language study. |
| Authoring v1 | Registration `fba9c78` pins source/reference base `04f72dc` (full ID in the evidence file); its runtime change is `c5c0183`. Preserve the documented null-payload scorer correction `383b49b`, trial records `608d0de`, and later explanatory notes. Runtime/reference bytes must match `v1/packet/manifest.json`. |
| Authoring v2 | Full runtime/documentation pin `c4b25e18ffdab96f328c45da4266069255c32ecf`; registration `b44a019`; results `3980c3d`. `study.json`, registration, and packet manifest agree on that full runtime ID. |

V2 records a clean compiled build using `rustc 1.98.1
(48a229cea 2026-09-01)`, host **`x86_64-pc-windows-msvc`**, and wasm-bindgen
`0.2.104`. The build script explicitly does not promise Windows/Linux WASM byte
equality; Linux release reproducibility does not reproduce this Windows packet.
V1's manifest freezes bytes without v2's build-host provenance. Verify its
actual hashes rather than inventing equivalent provenance.

For full study reproduction, use separate ordinary clones, not worktrees or a
branch switch in this shared checkout. Build the historical runtime in one
clean clone. Keep the study at its historical result/registration revision in
another, copy that build's `dist/` there, and run its packet preparation script.
V2 preparation also checks that the study checkout's runtime/build inputs still
match its pin. A later cleanup that changes those inputs requires reproducing
at `3980c3d` (or the registration revision), not blindly preparing on final main.
An already-preserved packet is usable if every frozen hash matches.

After a valid v2 packet exists, write fresh outputs outside preserved results:

```powershell
node --test experiments/agent-authoring/v2/checks.test.mjs experiments/agent-authoring/v2/infra.test.mjs
node experiments/agent-authoring/v2/audit.mjs --output test-results/agent-authoring-v2-integrity.json
node experiments/agent-authoring/v2/verify.mjs --output test-results/agent-authoring-v2-scored.json
```

V2's default result paths use exclusive creation; explicit `--output` paths
are overwritten and must point outside preserved results. V1's
`audit.mjs` and `verify.mjs` instead overwrite their tracked results; run them
only in the reproduction clone and keep the original checkout/evidence intact.
V1 preparation copies generated packet files before its final manifest check,
so do not try an arbitrary new runtime against a valuable preserved packet.

A scorer's exit zero means it completed; candidate failures can still be
recorded. A mismatched manifest is an infrastructure/pin failure, not candidate
failure. Keep the established limitations: v1's corpus passes include known
clock-horizon violations; v2 repeats two known tasks and changes runtime and
documentation together. Fresh contexts share a filesystem and do not establish
a security sandbox or independent model families. Do not upgrade those studies
into general reliability or cost claims during release writing.

## Tag and npm-kit sequencing

Kit design, implementation and unpublished-tarball tests can proceed while the
stack is reviewed. The sequence below gates release and the planned v3 trial;
v3 must install the tested, tagged tarball with its retained content hash.

1. **Consolidate and validate first.** Final main must contain all approved
   historical tips and cleanup, with the release gates above complete. Keep a
   manifest of final Git SHA, lockfiles, compiler/bindgen versions, build host,
   full/lean JS and WASM hashes, and tested policy/source hashes.
2. **Decide the public artifact boundary.** Create a separate runtime-kit
   package in a future packaging change; keep the private game-site package
   private. Select its package name/scope and version deliberately, define the
   supported import/init API and browser/Node/WASM asset loading, include
   licenses, source/spec references and one runnable policy example, and
   document the exact-source restore compatibility contract. No npm name or
   version is reserved by this plan.
3. **Test the packed artifact.** Build it from a clean committed candidate using
   the verified Linux runtime artifacts. Run `npm pack --dry-run --json` in that
   package, then `npm pack --json`; install the resulting tarball into a fresh
   ordinary consumer directory and exercise initialization, dispatch/view,
   malformed-input rollback, save/restore, `elapsed()`, and browser loading.
   Exclude study private tooling, trial archives and site-only assets unless
   explicitly part of the package contract. Archive the tarball and its hash.
4. **Freeze and tag the tested candidate.** Choose the release version after the
   package contract is settled. Commit version/release metadata, rebuild and
   retest that exact commit if bytes or metadata changed. Create an annotated
   tag at that exact SHA, never at an unverified moving branch head. Preserve
   original historical commits; a new tag supplements their identities.
5. **Publish only with release authorization.** Push the reviewed tag, attach
   the verified runtime/kit artifacts and manifests to a release, then publish
   the exact tested npm tarball. A prerelease tag/channel is appropriate while
   the API is experimental. Verify registry integrity and a fresh install,
   and only then update downstream consumer pins. Keep the prior pins available
   for rollback. The Pages artifact currently expires after 90 days; archive
   release bytes durably before relying on them as long-term pins.

Illustrative future commands, with values selected during release review:

```powershell
# In the new kit package directory, after its implementation and review:
npm pack --dry-run --json
npm pack --json
# After exact-candidate validation and release authorization:
git tag -a <release-tag> <verified-commit> -m "Release <version>"
git push origin <release-tag>
npm publish <tested-tarball> --access public --tag next
```

These placeholders are intentionally not runnable release instructions today.
There is no implemented npm-kit package or tag-driven publishing workflow at
the inspected head. Shipping a kit is a separate deliverable after this
consolidation, not a side effect of merging the language stack.
