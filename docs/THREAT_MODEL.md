# Threat model

This page says what Caveat protects, which inputs it treats as untrusted,
what stands between those inputs and the things it protects, and what it
leaves to the host. It covers the `caveat-lang` npm package (CLI, Node and
browser libraries, MCP server and WebAssembly runtime) and the hosted MCP
server. The browser games and the site are demonstrations and hold no user
data. Report a gap through the [security policy](../SECURITY.md).

Review this page whenever a change adds an input, a capability (filesystem,
subprocess, environment or network), a dependency, a hosted service or a
release channel. The release checklist asks for that review each release.

## What Caveat protects

1. **The meaning of a decision record.** A recorded decision keeps exactly the
   basis and grounds it was made on; later qualifications add doubt without
   rewriting it ([Late Qualification 0.1](../spec/caveat-late-qualification-0.1.md),
   [decision journal](../spec/caveat-decision-journal-0.1.md)).
2. **The host process.** Running a program, an event or a save must not
   execute host code, touch files, start processes or reach the network.
3. **The integrity of what is shipped.** A user installs the bytes that CI
   tested, built from the revision the release names.

Caveat does not protect the truth of the evidence it is given. It records
what a host tells it; it does not authenticate that evidence, the sender of
an event or the history behind a save, and it does not authorize actions
outside itself.

## Trust boundaries and untrusted inputs

| Input | Who controls it | Treated as |
| --- | --- | --- |
| Program source (`.cav`) | Author, agent or MCP client | Untrusted text: parsed and checked, never executed as host code |
| Events and scenario files | Host, agent or test author | Untrusted data: each event is accepted or refused as a whole |
| Agent named on an event (from 1.4.0) | Host | The host's word, not authenticated: the runtime checks that the program lets that agent send the event and records the name, not who really sent it ([Agents 0.1](../spec/caveat-agents-0.1.md)) |
| Saves | Whoever stored them | Untrusted data checked against the program, not authenticated |
| MCP requests (stdio or HTTP) | Any connected client | Untrusted JSON-RPC with size, depth and time limits |
| npm tarball and crates | The release workflow and crates.io | Verified by hash, provenance and audit before publishing |

## Threats and mitigations

### Hostile program source or events

- **Host code execution.** The language has no foreign calls, no `eval`, no
  file, process, host clock or network access (session time advances only by
  events); a program reaches the outside only
  through the values a host reads from it. The runtime is Rust compiled to
  WebAssembly, so a runtime fault traps inside its sandbox instead of
  corrupting host memory.
- **Resource exhaustion.** Each event and each procedure is bounded by a work
  limit (`MAX_EVENT_STEPS`, `MAX_PROCEDURE_STEPS` in
  [reactive.rs](../runtime/src/reactive.rs)) and refused when it exceeds it;
  from 1.4.0 an expression whose lineage or evaluation would outgrow its
  caps is refused as `limit/provenance_limit` or `limit/evaluation_limit`
  instead of ending the session, and from 1.4.1 a text value past 65,536
  bytes as `limit/text_limit`;
  streams and decision series have declared limits. Memory
  exhaustion traps the instance, and the kit reports it as `fatal` and loads a
  fresh one ([kit README](../kit/README.md)).
- **A partial update.** A refused event leaves the session exactly as it was
  (`runtime/tests/atomic_refusal.rs`), so a bad event cannot leave a
  half-applied decision.

### Tampered or forged saves

- Restore checks the save's schema, its source's SHA-256 fingerprint, and that
  its graph, values, journal and qualifications are ones the source's
  mechanisms can produce ([save contract](../spec/caveat-save-0.1.md#restore-validation),
  [restore trust boundary](RESTORE_TRUST_BOUNDARY.md)).
- The fingerprint identifies the source; it is not a signature. Restore does
  not prove a save came from a real event history. A host that needs that
  stores saves where they cannot be altered, or keeps the event log and
  replays it.

### The MCP server

- Tools take program source and scenario text as arguments, not file paths,
  and do their work in an isolated worker process with a timeout; messages
  are capped at 4 MiB, JSON nesting at 64 levels and request IDs per
  connection ([mcp.mjs](../kit/lib/mcp.mjs)).
- The hosted server runs the published package in a Cloudflare Worker,
  stores and logs nothing, and adds a per-address rate limit and a step cap
  on `caveat_test` ([hosted MCP README](../hosted-mcp/README.md)).
- A client that sends source can always learn what that source computes;
  that is the service, not a leak.

### Supply chain and release

- The package has no npm dependencies. Its compiled-in Rust crates are
  pinned by `runtime/Cargo.lock`, listed in
  [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md), and audited with
  RustSec before every candidate ([package security gate](PACKAGE_SECURITY.md)).
- The WebAssembly build is reproducible byte for byte (CI job
  "reproducible WebAssembly"), and the npm tarball's declared filesystem,
  subprocess, environment and network capabilities are checked against the
  files that use them.
- npm publishing uses trusted publishing from one workflow and environment,
  so no npm token is stored; it publishes only the tarball a Runtime run on
  `main` tested, waits for the owner's approval, and attaches npm provenance.
  The GitHub release refuses bytes whose SHA-256 differs from the tested
  tarball.

### Sensitive data in records

Caveat keeps every source a decision used, in plain text. Saves, event logs,
scenario files, `explain` and `dependents` output and the project's ledger hold
evidence and caveat names, reading values, decision bases and the event
history behind them. None of it is encrypted, redacted or access-controlled by
Caveat.

- Protect saves, event logs and ledgers as you protect the data their
  evidence came from: the same access rules, retention and storage.
- Do not put secrets, credentials or personal data in evidence, caveat or
  stream names, or in reading values. Use an opaque identifier and keep the
  mapping to the real record in the host, where its own controls apply.
- Before sharing `explain` output or a save, for example in a bug report,
  check what it names; the [security policy](../SECURITY.md) accepts a reduced
  program instead.
- The [PR ledger Action](../actions/pr-ledger/README.md) pushes its record to a
  branch of the same repository, so on a public repository that record (pull
  request numbers, check names, reviewers and authors) is public too.
- Proving what a decision rested on without revealing the evidence is not a
  1.x feature.

## Out of scope

- The truth of evidence, readings and event senders a host supplies.
- Keeping saves, event logs and ledgers confidential or unaltered at rest;
  see [Sensitive data in records](#sensitive-data-in-records).
- Authorizing the actions a host takes on a decision.
- Side channels such as timing differences between programs.
- Vulnerabilities in Node.js, browsers, WebAssembly engines or Cloudflare.
