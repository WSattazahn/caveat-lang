# Security policy

## Supported versions

| Version | Supported |
| --- | --- |
| Latest 1.x release | Yes: security fixes ship as a patch or the next minor |
| Earlier 1.x releases | Upgrade to the latest 1.x; the [1.x promise](docs/COMPATIBILITY.md) keeps that upgrade safe |
| 0.1.0 release candidates | No |

The supported packages are `caveat-lang` on npm (the developer kit, CLI, MCP
server and WebAssembly runtime built from `kit/` and `runtime/`) and the hosted
MCP server in `hosted-mcp/`. The browser games and the site are demonstrations;
report problems in them the same way, but they hold no user data.

## Reporting a vulnerability

Please report suspected vulnerabilities privately through GitHub's private
vulnerability reporting:

https://github.com/WSattazahn/caveat-lang/security/advisories/new

Do not open a public issue for a vulnerability. Include the affected version
or revision, a small program, event history or command that shows the problem,
what you expected and what happened. Reports from agents are welcome.

What happens next:

- We acknowledge a report within 14 days, usually much sooner.
- We confirm or decline it and tell you why. A confirmed vulnerability of
  medium severity or higher is fixed within 60 days of the report, and a
  critical one as fast as we can.
- The fix ships in a release whose notes in `docs/releases/` name the
  vulnerability and its advisory. The advisory is published on GitHub, with a
  CVE when one applies, and credits you unless you ask us not to.

A failure that crashes or hangs the runtime on hostile input, lets restore
accept a save that fails a check the [save contract](spec/caveat-save-0.1.md#restore-validation)
promises, makes a decision lose or misstate its recorded basis, or lets the
MCP server reach files or processes it should not, is a security report. The trust boundaries are described in the
[threat model](docs/THREAT_MODEL.md).

## Verifying a release

- Each 1.x release is published to npm from GitHub Actions with
  [npm provenance](https://docs.npmjs.com/generating-provenance-statements);
  check it with `npm audit signatures` after installing.
- Each GitHub release attaches the published tarball and its `SHA256SUMS`,
  and `docs/releases/` records the tested tarball's SHA256. Releases from
  1.4.0 also attach a CycloneDX software bill of materials
  (`caveat-lang-VERSION.cdx.json`) listing the package and every Rust crate
  compiled into it, with their SHA-256 checksums.
- The package has no npm dependencies. The Rust crates compiled into its
  WebAssembly runtime are listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md),
  and [PACKAGE_SECURITY.md](docs/PACKAGE_SECURITY.md) describes the dependency
  audit every candidate passes.
