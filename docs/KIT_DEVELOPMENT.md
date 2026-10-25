# Developing the kit

These sections used to be in the kit README. They describe how the npm package
is tested and packed from a repository checkout.

## Tests

These commands run from a repository checkout, after `npm run build`.
`npm run test:kit` covers:

- validation and matching rules, including repeated elements in `$set` and
  `$includes`, and `before` defined only after the first `send`;
- the session library: typed outcomes, payload refusal, fatal and trapped
  sessions, and a runtime that predates `dispatch_outcome`;
- `dispatchView` against `dispatch` and `view`, event by event: the same
  outcome, view, save and snapshot for every refusal code, fatal outcomes,
  every tracked program on a seeded random stream, and the Glowcap, agent
  ledger and Trail Rescue streams;
- injected runtime faults: a rejection that changes state, a restore that
  loses history, a resumed session that diverges or disagrees, grounds outside
  lineage, fatal reports, unrecognised outcomes and traps. Each is caught and
  the misbehaving session is named;
- the [spec evidence](https://github.com/WSattazahn/caveat-lang/blob/ab3b0d3/experiments/scenario-format/README.md) fixtures and
  the CLI's exit codes;
- `explain`: the report matches the snapshot decision by decision, and the
  command lists refusals, stops at a fatal event and refuses bad input;
- `dependents`: the answer is checked against the snapshot's grounds and
  lineage for evidence, reading streams and caveats, and an unknown name is
  refused;
- `serve`: dispatch, explain, dependents, save and restore over the protocol;
  malformed requests answered without effect; a fatal outcome ending the
  server; and the process itself on standard input and output;
- the agent-evidence example: its program loads with no warnings, and its
  Python caller's tests pass against this checkout's `caveat serve`. They need
  Python 3.9 or later;
- `validate`, `replay` and `init`: exit statuses, line numbers, typed
  parameters, and `init`'s files staying identical to the guide's;
- the TypeScript declarations, by name: every export, method and member,
  every field of the snapshot, view, outcomes and reports, every option and
  runtime call, and every code and kind, against the modules, real sessions,
  the runtime's Rust structs and the documents that list them. Parameter and
  return types are not compared;
- the packaged documentation: every source exists, every link in the kit's
  own documents resolves inside the package, every link a copied document
  keeps resolves inside it and every link it rewrites names a repository
  file, the authoring guide's script runs, and the getting-started guide,
  followed step by step, prints what it shows. The package test follows the
  guide again from the installed tarball.

`npm run test:scenario-conversion` runs the
[converted Trail Rescue and Glowcap suites](https://github.com/WSattazahn/caveat-lang/blob/ab3b0d3/experiments/scenario-conversion/README.md).

`npm run test:kit-browser` runs the session library and the scenario runner
in Chromium: the runtime loaded from URLs, typed outcomes, refused input that
leaves state unchanged, save and restore, `elapsed()`, and scenario files read
with `fetch`, including a failing one. Set `PLAYWRIGHT_CHANNEL=chrome` or
`msedge` to use an installed browser.

## Packing

`npm run test:kit-package` copies the reactive runtime and its build info into
`kit/runtime/`, packs the kit, and removes `kit/runtime/` again so development
never uses a stale copy. It installs the tarball offline into a fresh consumer
directory under `test-results/kit-package/` and uses it only through the
install: the `caveat` command (`test` with exit statuses 0, 1 and 2,
`explain`, `dependents`, `validate`, `replay`, `serve` and `init`), the
library imported as `caveat-lang/node`, `caveat-lang/session`,
`caveat-lang/scenarios`, `caveat-lang/explain` and `caveat-lang/serve`, a
starter host resumed in a second process from `caveat-lang/starter` and
`caveat-lang/starter/node`, and the browser check above. It also copies the agent-evidence example out of the
install and runs its Python tests against the installed command. The tarball
holds the command, the public library files, bounded authoring bridge and their declarations, `init`'s
templates, the example, the runtime and its declarations, the documentation,
the license and the notices, and nothing else. In the documents copied from
the repository, links that leave the package point to GitHub at the build's
revision.

The package is MIT licensed. Packing copies the repository's `LICENSE` and
`THIRD_PARTY_NOTICES.md` (the crates compiled into the runtime) into the
tarball, and the test checks both arrive unchanged.

Exact installation examples and the package identity are generated from
`kit/package.json`. After changing that manifest, run
`node scripts/kit-docs.mjs --write` from the repository root.
`node scripts/kit-docs.mjs --check` and the packaging gate refuse stale blocks
or moving publication notices in the kit's own guides. The installed-package
test runs the README commands against the same tarball, including doctor, demo
and the starter.

CI already runs this packaging test against the runtime built by its Linux
core job. It retains the exact tested `.tgz`, the test report with its SHA-256,
`build-info.json` and `SHA256SUMS` together as an artifact. A local Windows run
checks packaging locally; the release candidate must use the tested Linux
artifact from the intended release commit. Before publication, the
[packed dependency security gate](https://github.com/WSattazahn/caveat-lang/blob/main/docs/PACKAGE_SECURITY.md) must
pass against that exact tarball and retain its npm/Rust advisory receipts.
A release publishes that one tested
tarball twice, unchanged: to npm and attached to the GitHub release. No
second build is made for the registry. Each release record names the npm
dist-tags its version was published under. See the
[consolidation plan](https://github.com/WSattazahn/caveat-lang/blob/main/docs/CONSOLIDATION_PLAN.md#candidate-and-release-gates)
for the release gates.
