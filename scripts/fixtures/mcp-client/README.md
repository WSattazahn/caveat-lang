# Official MCP client fixture

The installed-package gate uses the lockfile-pinned official
`@modelcontextprotocol/sdk@1.32.1` client to start the actual installed CLI,
negotiate the documented stdio profile, discover and call all five tools, check
invalid source and virtual-path refusal, and make a fresh successful call.

The fixture is development-only and excluded from the published tarball.
Installs disable lifecycle scripts. Its dependency audit is a separate scope
from the dependency-free language package and is retained by the security gate.
This proves interoperability with this client, not installation in every host.

The 1.32.1 pin replaces 1.31.0 after the fixture audit reported
GHSA-22jm-h49p-29qw, GHSA-6prh-2h8m-c8cw, GHSA-46jr-4wx8-v453 and
GHSA-rvq5-wwqv-78pq against the earlier SDK. The patched SDK range is
1.32.0 or later. Keep auditing this development fixture independently of the
published language package.
