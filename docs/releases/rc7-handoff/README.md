# rc.7 manual publication handoff

The immutable rc.7 Linux release candidate is main commit
`1f3fc7a2208eec964399d6c14232690f411e48be`, not this moving handoff/development branch.
The tagged GitHub asset must be the installed-package tarball retained by
Runtime36970516513 and its same-byte security gate. No repack or build is allowed.

The cloud release workflow creates an annotated tag and published GitHub
**prerelease** only after exact Runtime/Pages/live gates, artifact/report hashes
and retained extra controls succeed. npm stays pending. The publication helper
contains exact hashes and version; it works from a temporary consumer regardless
of the owner's local checkout branch (including rc.8).

Requirements: Windows PowerShell5.1+ or PowerShell7, Node20+ with npm.cmd,
GitHub CLI gh.exe. No live desktop MCP host is installed. Missing tools fail
before publication. Authentication uses interactive GitHub/npm login and may
request browser approval or npm2FA; no token/password is embedded or printed.

Download the immutable release's Publish-Rc7.ps1, verify its SHA256 given in
release-manifest.json and in the cloud completion instructions, then execute it.
It checks exact tgz SHA256/SHA512, identity and Linux runtime, tests a fresh local
consumer, and only then publishes with --access public --tag next to the official
registry. It never changes latest. Existing rc.7 bytes must match, otherwise it
stops; unexpected dist-tags stop for explicit review rather than silently moving
channels. Registry failures other than explicit HTTP404 never mean absent.

After publication it verifies metadata/integrity and downloaded bytes, requires
fresh npm-lock integrity/origin, and exercises both aliases, doctor/demo and
umbrella scenarios. Only after all gates pass does it attach a unique GitHub
publication receipt and update the release's factual npm availability. If a
GitHub recording step fails after npm succeeds, retain the printed receipt,
authenticate and rerun with -VerifyOnly; identical registry bytes are required.

Return the final receipt to the cloud thread. The remaining explicit cloud step
is to independently verify publication, update public installation examples and
the dated publication record, and validate/deploy those documentation changes.
rc.8 development is opened only after rc.7's annotated tag and assets freeze;
its version/docs updates do not modify the frozen rc.7 artifact. No domains,
trademarks, renaming, stable/latest release or major features are part of this.

Recovery: failed/interrupted receipts remain retained. If tagging or an upload
fails, inspect the existing tag target, prerelease state and served asset hashes.
Never force a tag, overwrite a mismatched asset, or rerun blindly. The release
job validates same named assets byte-for-byte; a new live-evidence archive from
a different run can require explicit recovery using the original verified
archive. GITHUB_TOKEN permission or protection denials must be reported, not
bypassed.
