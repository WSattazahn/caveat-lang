# Packed candidate security gate

The gate checks one tested npm tarball and the Rust dependency lockfile belonging
to its recorded build revision. It separately reports dependencies used by the
site, editor and verification fixtures. A clean npm report alone says nothing
about the Rust crates compiled into the packaged WebAssembly runtime.

## Run the gate

After the regular build and packed-package tests, pass the exact package report:

```sh
node scripts/audit-kit-security.mjs --package-report test-results/kit-package/RUN/report.json
```

Replace `RUN` with the tested run directory. The gate accepts no implicit
"latest" artifact. It requires Node/npm, Git, `tar`, Cargo, the locked crates in
the pinned auditor below, and registry access to fetch the recorded locked crates. It returns 0 only when every
required scope passes, 1 for a recorded failed/incomplete gate, and 2 for invalid
command arguments or inability to create its receipt. It writes a new directory
under `test-results/kit-security/`, including failed runs.

Install the auditor locally; no global tool replacement is needed:

```sh
cargo install cargo-audit --version 0.22.2 --locked --root test-results/security-tools
git clone --depth 1 https://github.com/RustSec/advisory-db.git test-results/security-tools/advisory-db
```

`cargo-audit` 0.22.2 requires Rust 1.88 or later according to its
[crate manifest](https://github.com/RustSec/rustsec/blob/main/cargo-audit/Cargo.toml).
The [RustSec tool documentation](https://github.com/RustSec/rustsec/tree/main/cargo-audit)
describes the auditor. `--audit-bin FILE` and `--advisory-db DIRECTORY` can select
other local locations, but the auditor version and official database origin are
still checked. The database must have no local changes. Each run fetches the
current official HEAD, records its time and commit, and refuses an older checkout.
Update that clean advisory clone to the fetched commit and rerun when needed.

## What is checked

- **Actual packed npm artifact.** The tarball must match the package-test report's
  SHA256, size and exact file list. The gate retains a copy, inspects its manifest
  and runtime, and requires a clean, compiled build with matching revision and
  WASM hash. It installs that exact copy into a fresh consumer with scripts
  disabled. The current artifact contract has no npm runtime, optional, peer or
  bundled dependencies; adding one requires an explicit contract review.
- **Rust dependencies.** `Cargo.toml` and `Cargo.lock` are read from the packaged
  build's Git revision, hashed and retained. RustSec scans the entire lockfile,
  including build dependencies. A separate reactive-only Cargo tree identifies
  normal linked crates without labeling procedural macros as shipped code. An
  isolated project configuration forbids advisory ignores and platform/severity
  filters. The database commit and raw response are retained.
- **Development and site dependencies.** Separate npm audits cover the root
  project (including three.js and Playwright), the editor, the CLI collision
  fixture and the official MCP client fixture. All dependency categories are
  included explicitly. Their findings never masquerade as npm dependencies of
  the shipped language package. Fixture overrides appear in the receipt.

The npm scans use the public registry and
[`npm audit --json --audit-level=high`](https://docs.npmjs.com/cli/v11/commands/npm-audit/).
Each report includes every severity; the flag sets the failure threshold.
No dependency is updated automatically by the gate and no package scripts run.

## Declared capabilities

Supply-chain scanners such as Socket report published files that use the
filesystem, subprocesses, the environment or the network. These are the files
of the packed `caveat-lang` tarball that legitimately do so:

| Capability | Files | Why |
| --- | --- | --- |
| filesystem | `bin/caveat.mjs`, `examples/agent-evidence/caller.py`, `examples/agent-evidence/save-text.mjs`, `examples/agent-evidence/test_branching.py`, `examples/agent-evidence/test_caller.py`, `examples/agent-evidence/test_grounds.py`, `examples/agent-evidence/test_lifecycle.py`, `examples/agent-evidence/test_qualification.py`, `lib/demo.mjs`, `lib/doctor.mjs`, `lib/mcp.mjs`, `lib/node.mjs`, `lib/starter-node.mjs` | The CLI and Node library read programs, events and scenarios and write saves and `init` files; the starter's Node store writes its checkpoint and archive to a directory; the examples store save text and their tests use temporary directories. |
| subprocess | `examples/agent-evidence/caller.py`, `examples/agent-evidence/test_caller.py`, `lib/mcp.d.mts`, `lib/mcp.mjs` | The MCP server runs authoring work in an isolated worker process; the Python example starts `caveat serve`. The `.d.mts` file only declares the type. |
| environment | `examples/agent-evidence/caller.py`, `lib/doctor.mjs` | `doctor` reports the runtime environment; the Python example reads `CAVEAT_COMMAND`. |
| network | `runtime/caveat_runtime.js` | wasm-bindgen's loader fetches the `.wasm` file when given a URL, as in a browser. The kit's Node entry points pass the bytes instead. |

`scripts/kit-capabilities.mjs` holds the same table. The security gate scans
every JavaScript, TypeScript and Python file in the tested tarball (patterns:
`fs`, `fs/promises`, `child_process`, `process.env`, `http`, `https`, `net`,
`tls`, `fetch(`, `WebSocket`, and their Python counterparts) and fails when a
file uses a capability it is not listed for, or a listed file no longer uses
it. The match is textual, like the scanners it anticipates, so comments count.
A new scanner alert outside this table is a regression; a new example or
library file that needs a capability updates the table and this section in the
same change. Run the scan alone with:

```sh
node scripts/kit-capabilities.mjs --tarball test-results/kit-package/RUN/caveat-lang-VERSION.tgz
```

The plan's rc.10 inventory (`docs/releases/rc11-development.md`, 4b) listed
only the JavaScript files and no network use. The first scan of the packed
tarball also found the Python example files above, the type declaration
`lib/mcp.d.mts` and the wasm-bindgen loader's `fetch(`, so they are listed.

## Findings and release decisions

npm High or Critical findings block the gate. Lower severity findings remain in
the receipt for review; passing the threshold does not mean none were reported.
Every Rust vulnerability blocks, including findings without a severity score.
Unresolved Rust informational warnings also block this first gate because they
can include unsoundness without a CVSS score. There are no ignore lists.

For a blocking finding, record the exact advisory, affected locked version and
scope, then update the dependency or remove its use and repeat relevant behavior
tests plus this gate. If a patch is unavailable, keep the candidate blocked and
record the unresolved issue for the owner. Changing this policy is a separate
reviewable decision, not an automatic exception or a successful scan.

The initial inventory found and repaired two development-only issues:

- Playwright 1.55.0 was updated to 1.55.1 for
  [GHSA-7mvr-c777-76hp](https://github.com/advisories/GHSA-7mvr-c777-76hp), which
  affects certificate verification in browser installer scripts.
- The collision fixture keeps the real `caveat-cli@0.19.13` package but overrides
  its exact `smol-toml@1.7.0` dependency with 1.7.1 for
  [GHSA-7w5x-hrqm-74c2](https://github.com/squirrelchat/smol-toml/security/advisories/GHSA-7w5x-hrqm-74c2).
  Its [fixture note](../scripts/fixtures/cli-collision/README.md) records that
  patched dependency tree. The competitor's CLI and scripts are never executed.

## Evidence and limits

The receipt records tool versions, command arguments, timestamps and exit codes;
input manifest/lockfile hashes; the artifact and build identity; the RustSec
commit; and the hash of every retained raw command response. npm exposes no
advisory database commit, so its response and timestamp preserve what this run
reported. A future live audit can change when new advisories are published.

RustSec yanked-crate checks are explicitly disabled to keep the advisory scan
tied to its recorded database; yanking is a separate live registry signal, not
a vulnerability advisory. This gate does not inspect application source for
security defects, authenticate supplied evidence, detect all malicious code,
scan the browser executable or Node installation, or prove that unknown
vulnerabilities do not exist. The normal package/runtime tests remain required.
