# CAVEAT Language 0.1.0-rc.7

GitHub candidate available; npm publication pending.

This experimental prerelease freezes commit `1f3fc7a2208eec964399d6c14232690f411e48be`. At GitHub freeze, npm published
rc.6 under `next` and rc.5 under `latest`. Neither channel denotes stable software.
The owner-console handoff publishes the exact tested tgz using the attached
hash-pinned PowerShell helper; this cloud freeze performs no npm publication
or dist-tag action. The verified publication section below, when present,
records the subsequent registry state.

Changes include neutral evidence observation, clearer citation/named-expression
diagnostics, the unambiguous `caveat-lang` CLI alias (with `caveat` retained),
doctor, deterministic agent demo, onboarding, a bounded inline-only MCP bridge,
and same-artifact dependency auditing. Public whatif, abstain and scenario string
matchers remain deferred. No runtime npm dependencies or desktop-host installation.

## Frozen Linux artifact

- Tarball: `caveat-lang-0.1.0-rc.7.tgz`, 810,289 bytes.
- SHA256: `0e441f896c58ba41f2741a82af40a17e74247a0c765169abda10b9ddffad5bbc`.
- SHA512: `0b2c6dba702e372a3593b28b421a2f12f330ad2fae58fd94e152d3ba465343035170a25bbc8f04e407d85c681df02e2d382d684a2c47dd80f6279125ecb8a6f2`.
- npm integrity: `sha512-CyxtunAuNyo1k7KLQhovEvMwrS+uWP2U4VLTukZTQwNRcKJbvI8E5AfYXGgd8C4tOC1oSixH3YD2J5El7Lim8g==`.
- Reactive WASM SHA256: `32139e7b59306e4da799f6977ec9356f68add27c7f7159a443d7a1f56cae8977`.
- Package report SHA256: `7e39237a02ca1fe3e961a00d52e192a64483753b62c8cd9068672482af15fcf9`.
- Security report SHA256: `77ff1e7bdc42360131678d7adf9399d3a2b58c6d24476362e21428dfb533125d`.
- Clean compiled Linux: Rust1.98.1, wasm-bindgen0.2.104,
  `x86_64-unknown-linux-gnu`; build-info and complete tool/lock identities retained.

## Validation and preserved failures

[Final-main Runtime36970516513](https://github.com/WSattazahn/caveat-lang/actions/runs/36970516513)
attempt1 and [format36970516523](https://github.com/WSattazahn/caveat-lang/actions/runs/36970516523)
pass. Runtime includes full/reactive Rust, 229 kit tests, installed package,
MCP interoperability, Python guides, editor checks, both Light the Way browsers,
other games including conditional Legacy Door, and reproducible WASM.
[Pages36972072386](https://github.com/WSattazahn/caveat-lang/actions/runs/36972072386)
attempt1 deployment and live evidence/Moon/Mr.Caveat/LegacyDoor QA pass.
An additional pinned cloud release job checks live full/reactive JS+WASM hashes
and Last Beacon/Light the Way gameplay before creating this tag; its receipts
and screenshots are attached.

Same-byte security passed at 2026-10-02T06:05:05.359Z: all five npm scopes have
zero findings; the complete Rust lockfile has zero vulnerabilities/informational
warnings. All 56 stdout/stderr receipt hashes matched. Rust yanked status is
excluded; advisory scans do not prove absence of unknown vulnerabilities.

Independent extra checks pass: 29 author/Glowcap pure controls; registered v1/v2
source identities and retained run receipts; all 335 files listed by the adoption
archive manifest; five saved divergences (16,011 events/13 restores); 4,000
seeded differential sequences (14,443,471 events), zero divergence.
No registered trials were rerun or rewritten, and Windows packets were not
reconstructed as Linux evidence.

PR81's earlier first WebKit attempt was interrupted/cancelled, not a successful
first pass; its second attempt passed. Final-main WebKit passed on attempt1.
The added release-evidence workflow's first run36972762671 failed on a
Response.ok API mistake; commit8e615cf corrected it. Its failed log remains
retained. Neither failure is silently counted as success.

The adoption study is descriptive and qualified by Windows working-directory
leakage/filename enumeration; all three final programs pass the registered
15 semantic/10 host cases with scores16/16,16/16,15/16. No adoption gain or clean
isolation claim. Naming research, including the cloud USPTO/TSDR/TMview addendum,
is not legal clearance; broader markets/similar-mark/phrase review and WIPO
access remain pre-1.0 limits, not an rc.7 gate.

## Manual npm handoff

Use `Publish-Rc7.ps1` on Windows with Node20+ and GitHub CLI available. It uses a
fresh temporary directory, immutable release tgz, hard-coded SHA256/SHA512 and
runtime/package identity; lifecycle scripts stay disabled. Login/2FA is
interactive. It publishes only that tgz to https://registry.npmjs.org with
`--access public --tag next`, retains `latest`, then verifies registry integrity,
served bytes, both aliases, doctor/demo and real umbrella scenarios in a fresh
exact-version install. A same-version retry must prove identical bytes.

Only after these gates pass does it attach a publication receipt and update this
GitHub release description. Return its receipt to the cloud thread for the
remaining factual public-install documentation update. rc.8 development starts
after this tag/assets freeze and does not change these frozen release bytes.
