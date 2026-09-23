# Pages deployment

Pages publishes the `browser-dist` artifact from the latest `runtime.yml`
push run for the current `main` commit, provided that run succeeds. It does
not rebuild the site. The complete runtime workflow must finish successfully, including its
browser jobs and reproducibility check, before an automatic deployment starts.

The gate checks the workflow identity, repository, push event, branch, current
`main` SHA and run result using fresh GitHub API responses. It selects one
unexpired artifact by ID, verifies its source SHA and clean compiled Linux
build metadata, and checks the included license and notices against that
source revision. Pull request runs cannot deploy. A delayed successful result
cannot replace a newer runtime run or deploy an older `main` revision.

Only the deployment job holds the `pages` concurrency lock. Failed or
ineligible runtime completions cannot cancel an active deployment. After
acquiring the lock, the job rechecks `main` and the runtime result immediately
before calling GitHub Pages. Build outputs retain the Pages artifact name, so
retrying a failed deploy can use the successful build from an earlier attempt.

The retained host artifact is named `caveat-runtime-<source SHA>` and includes
`LICENSE`, `THIRD_PARTY_NOTICES.md` and `build-info.json`. Both it and the
runtime workflow's tested `browser-dist` from pushes to `main` are retained
for 90 days. Other `browser-dist` artifacts, including pull request builds,
are retained for 7 days. The SHA is
the tested source revision, not the default-branch SHA attached to a delayed
`workflow_run` event.

Rerunning runtime CI replaces its named artifacts within that same workflow
run. Artifacts from other runs remain retained independently. The gate records
the selected runtime attempt and refuses deployment if a later attempt has
replaced it, even if that later attempt has already passed.

After deployment, live tests verify the build metadata and play Glowcap and
Trail Rescue on desktop and mobile. The existing Mr. Caveat, Moon Garden
2D/3D and conditional Legacy Door checks remain. The Door condition comes
from the selected runtime run's job result, preserving the scope of its
original push, including pushes containing several commits.

## Redeploy and recovery

Run `deploy-pages` manually on `main` to redeploy the tested current revision.
There is no arbitrary SHA input. The latest runtime push run for current
`main` must already be successful, and its `browser-dist` must still exist.
A pending or failed newer run is refused rather than hidden by an older pass.

If the artifact has expired or was deleted, rerun the current main commit's
runtime workflow and let its successful completion trigger Pages. If runtime
CI fails, fix that failure; the gate does not rebuild or bypass the checks.
If `main` advances while a deployment waits, the old deployment stops and the
new revision's successful runtime run supplies its replacement.

Live QA runs after deployment. A failure leaves that deployment live; there
is no automatic rollback. Investigate the failure before retrying the live
checks. A manual redeploy can only publish the tested current `main`, not an
older good revision. To restore earlier behavior, merge a revert or fix to
`main`, then let its successful runtime CI trigger a new deployment. Keep
release tagging blocked until that deployment and its live checks pass.

## Verification before tagging

`node --test scripts/pages-gate.test.mjs` exercises successful selection and
the failed, pending, stale, foreign, missing-artifact and manual-redeploy
refusal paths. Runtime CI runs these checks on pull requests as well.

GitHub loads a `workflow_run` workflow from the default branch. Its end-to-end
trigger, artifact transfer and deployment therefore must be verified on the
first run after this gate lands on `main`. Before tagging a release candidate,
require that run and its live checks to pass, and verify the retained runtime
artifact and deployed `build-info.json` name that exact merge commit. Unit
tests and a green pull request are not that deployment evidence.
