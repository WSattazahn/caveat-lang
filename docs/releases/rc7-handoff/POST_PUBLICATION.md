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
