# Registry-gated rc.7 documentation finalization

This is a one-time, owner-triggered follow-up, not recurring automation.
`Publish-Rc7.ps1` first verifies the immutable GitHub tag/asset, publishes only
its retained bytes with next, verifies registry SHA256/SHA512 and a fresh
exact-version install, and records that owner-console receipt in the prerelease.
It does not update moving public documentation or publish rc.8.

After that helper completes successfully, run:

```powershell
gh workflow run record-rc7-publication.yml --repo WSattazahn/caveat-lang --ref main
if ($LASTEXITCODE -ne 0) { throw 'Publication documentation follow-up did not start.' }
```

The cloud workflow independently requires official npm identity/integrity,
downloaded tarball hash, next rc.7/latest rc.5 and a new installed exact rc.7
consumer. It checks both executable aliases, clean compiled Linux build revision
1f3fc7a2208eec964399d6c14232690f411e48be, WASM hash, doctor, unchanged five-step
demo and real umbrella scenarios. No npm publication or channel write occurs.
HTTP404 or mismatches refuse all documentation writes. `--check-only` has no
GitHub mutation path and is used by PR validation.

Only then does it create branch codex/rc7-npm-publication and a publication-docs
PR, recording the dated verification and complete hashed command streams in git.
It changes exact npm examples/status in root/kit README, repository agent guidance, agent quickstart,
getting-started, naming/MCP/example docs and About metadata/text, and appends the
verified npm receipt to the rc.7 record. rc.8 stays unpublished development;
frozen rc.7 bytes and registered studies remain unchanged. About style/script
bytes must match the baseline. No existing publication branch is overwritten.

**A final cloud review, normal Runtime/format checks, authorized merge and
exact-revision Pages/live deployment verification remain necessary.** Auto-merge
is disabled in this repository. Return the workflow run/PR URL (and the helper
receipt) to this cloud thread; the cloud follow-up will review and merge under the
existing authorization, then verify Pages. The laptop is not needed for that work. If GITHUB_TOKEN-created PRs do not trigger normal CI,
the cloud reviewer must append its dated review receipt using the authorized
GitHub connection, then require the resulting checks before merge. Do not count
absence of triggered CI as a pass.

If GitHub's Actions PR setting or protection denies a write, retain the run's
receipt artifact. A successfully created branch plus prepared-pr.json contains
the exact reviewed next PR step; report the permission error and use the
owner-authorized GitHub connection without changing protection/settings. Never
silently mark this follow-up complete or claim npm availability on a failed gate.

## 2026-10-02 recovery amendment

The immutable release asset Publish-Rc7.ps1 (SHA256
860d75b726097e481b7e9d0206ffa4f401de9bfc89e63162a106f2e4a1f4e88e)
is preserved. It checked registry visibility only once after npm accepted the
publication, and reported a misleading identity/integrity mismatch on HTTP404.
The owner observed npm's processing message and + caveat-lang@0.1.0-rc.7;
the initial version endpoint returned HTTP404. This is accepted/pending evidence,
not evidence of different bytes. Subsequent registry download and fresh install
succeeded before Windows PowerShell5.1 refused the lockfile's empty packages key.
No authentication URLs, codes or tokens are retained here.

The moving-source helper now polls at most 61 reads with 10-second intervals
(10 minutes plus request time). HTTP404 and the known old next=rc.6 state may
propagate; actual identity/integrity mismatch, unexpected channels, and every
non404 request error refuse immediately. latest must remain rc.5. Timeout says
accepted/pending/unverified and retains a public recovery-context JSON with the
exact artifact identity. Use the reviewed source helper with **-VerifyOnly** for
this already accepted version; do not publish again. VerifyOnly cannot call login
or publish even while the registry version is absent. Node selects the installed
package lock entry before PowerShell parses JSON, supporting PowerShell5.1.
The frozen release helper has neither fix; do not overwrite that asset or assert
its original hash identifies this amended source. A successful recovery still
requires exact registry bytes, a fresh install and the publication-record workflow.
