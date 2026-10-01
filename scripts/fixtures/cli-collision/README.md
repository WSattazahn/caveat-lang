# CLI collision fixture

`npm run test:kit-package` installs this locked `caveat-cli@0.19.13` and the
newly packed `caveat-lang` in two fresh directories. It needs npm registry
access or a cache containing the lockfile's exact dependencies. All third-party
installs and rebuilds use `--ignore-scripts`; the competing executable is never
run, and no hooks or memory store are initialized.

The fixture verifies npm's `caveat` shim target for each package in turn. npm
install ordering differs across versions, so it removes only the generated
`caveat` shims and asks npm to relink the selected owner with `npm rebuild`.
It then invokes the real `caveat-lang` shim and npm exec, checking CAVEAT
Language help and the packed package version in both ownership cases. This is
about executable selection, not compatibility with the other tool's runtime.

The fixture package and transitive dependencies are pinned by the checked-in
lockfile, with registry integrity hashes. Refresh them deliberately when
updating this regression; do not replace the real package with a local stub.
The package report records the selected version, integrity and both cases.
The fixture is not included in the published language package.
