# CLI ownership collision fixture

This fixture installs the real `caveat-cli@0.19.13` package beside the packed
`caveat-lang` candidate. The regression deliberately gives each package
ownership of the ambiguous `caveat` executable and checks that `caveat-lang`
still launches CAVEAT Language. Third-party lifecycle scripts and the competing
CLI are never executed.

The competitor package is pinned with its registry integrity hash. Its
`smol-toml` dependency is overridden from the published exact `1.7.0` requirement
to `1.7.1`, which fixes the malformed TOML denial of service in
[GHSA-7w5x-hrqm-74c2](https://github.com/advisories/GHSA-7w5x-hrqm-74c2).
This is a fixture with a patched dependency tree, not a reproduction of the
competitor's original dependency tree or a test of its application behavior.
The lockfile pins the replacement and its integrity, and the collision receipt
records `dependencyOverrides` explicitly.

Run `npm audit --json --audit-level=high` in this directory to inspect its locked
dependencies. The full installed collision test runs in `npm run test:kit-package`
from the repository root, after `npm run build`. Its consumer directories are
created under `test-results/kit-package/`; the fixture itself is never published.
