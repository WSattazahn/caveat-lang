# Official MCP client fixture

The installed-package gate uses the lockfile-pinned official
`@modelcontextprotocol/sdk@1.31.0` client to start the actual installed CLI,
negotiate the documented stdio profile, discover and call all five tools, check
invalid source and virtual-path refusal, and make a fresh successful call.

The fixture is development-only and excluded from the published tarball.
Installs disable lifecycle scripts. Its dependency audit is a separate scope
from the dependency-free language package and is retained by the security gate.
This proves interoperability with this client, not installation in every host.
