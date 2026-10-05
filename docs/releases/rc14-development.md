# CAVEAT 0.1.0-rc.14 development plan

Status: published 2026-10-05; see the [release record](v0.1.0-rc.14.md). rc.14 is a one-item release: it
corrects the package's MCP server name so the MCP Registry can list it. The
feature release planned after rc.13 becomes rc.15. Keep the "Progress"
section factual, as `rc13-development.md` does.

## Why

On 2026-10-05 at 01:06 UTC the owner ran the MCP Registry publish for rc.13.
`mcp-publisher validate` passed, and the GitHub login succeeded, but the
publish was refused with 403: the login may publish `io.github.WSattazahn/*`,
and `server.json` named `io.github.wsattazahn/caveat-lang`. Nothing was
listed.

- The registry's namespace check is a case-sensitive prefix match on the
  GitHub login as GitHub returns it (`internal/auth/jwt.go`, `strings.HasPrefix`).
- Its npm ownership check compares the published package's `mcpName` with
  the server name exactly (`internal/validators/registries/npm.go`,
  `npmResp.MCPName != serverName`).
- The published `caveat-lang@0.1.0-rc.13` carries
  `"mcpName": "io.github.wsattazahn/caveat-lang"`, so rc.13 cannot be listed
  under either spelling, and npm versions cannot be changed after publication.
- DNS and HTTP namespace authentication refuse `github.io` domains, so there is
  no other route to the lowercase namespace.

The owner chose to call this release 0.1.0-rc.14 rather than 0.1.0-rc.13.1
(card, 2026-10-05 01:11 UTC), because the release tooling accepts only
`rc.N` versions and changing the release gates is not in scope.

## Scope

- `kit/package.json`: version `0.1.0-rc.14`; `mcpName`
  `io.github.WSattazahn/caveat-lang`. The bundled installation blocks are
  regenerated from it (`node scripts/kit-docs.mjs --write`).
- `server.json`: `name` `io.github.WSattazahn/caveat-lang`; version and package
  version `0.1.0-rc.14`.
- No runtime, language, kit library or specification change. The release goes
  through `publish-npm.yml` unchanged, as rc.12 and rc.13 did.

## After publication

The owner reruns the MCP Registry block from 2026-10-05 01:06 unchanged; it
reads `server.json` from `origin/main`. rc.13's release record stays as
written: it says what rc.13 shipped.

## Progress

- Release PR: the version bump, the name correction, this plan and the
  release notes in `docs/releases/v0.1.0-rc.14.md`, marked unpublished.
- Published 2026-10-05: tag `v0.1.0-rc.14` at `51590b4` (merge of #150), the
  tarball from
  [Runtime 37252199187](https://github.com/WSattazahn/caveat-lang/actions/runs/37252199187)
  (SHA256 `9aab7b811666e5b43b6f7c96ad5116235241b08fbed395698476cc035bfd57bf`),
  published by [publish-npm.yml run 37254432886](https://github.com/WSattazahn/caveat-lang/actions/runs/37254432886)
  under `latest` with npm provenance, after the owner approved it in
  `npm-publish`. `verify-publication` failed once on an attestation not yet
  served by the registry and passed when re-run alone. The owner then moved
  `next` to rc.14 and published `server.json` to the MCP Registry, which lists
  `io.github.WSattazahn/caveat-lang` 0.1.0-rc.14. See the
  [release record](v0.1.0-rc.14.md).
