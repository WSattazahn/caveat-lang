# CLI ownership collision fixture

This fixture installs the real `caveat-cli@0.19.13` package beside the packed
`caveat-lang` candidate. The regression deliberately gives each package
ownership of the ambiguous `caveat` executable and checks that `caveat-lang`
still launches CAVEAT Language. Third-party lifecycle scripts and the competing
CLI are never executed.

The competitor package is pinned with its registry integrity hash. Its
`smol-toml` dependency is overridden from the published exact `1.7.0` requirement
to `1.9.0`, which includes the malformed TOML denial-of-service fix from
`1.7.1` for [GHSA-7w5x-hrqm-74c2](https://github.com/advisories/GHSA-7w5x-hrqm-74c2)
and fixes the quadratic-time key parsing in
[GHSA-r4xh-jqrq-34v2](https://github.com/advisories/GHSA-r4xh-jqrq-34v2).
The latter affects versions through `1.8.0`; the pre-update audit reported both the
vulnerable dependency and its dependent `caveat-cli` package. Neither is
part of the published CAVEAT package or executed by this fixture.
This is a fixture with a patched dependency tree, not a reproduction of the
competitor's original dependency tree or a test of its application behavior.
The lockfile pins the replacement and its integrity, and the collision receipt
records `dependencyOverrides` explicitly.

Run `npm audit --json --audit-level=high` in this directory to inspect its locked
dependencies. The full installed collision test runs in `npm run test:kit-package`
from the repository root, after `npm run build`. Its consumer directories are
created under `test-results/kit-package/`; the fixture itself is never published.
