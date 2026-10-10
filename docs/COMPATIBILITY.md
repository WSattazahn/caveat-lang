# Caveat compatibility commitment

Status: **proposed for 1.0.0**, written under [Path to 1.0](releases/path-to-1.0.md)
(PR A2, blocker B2). Until 1.0.0 is released, prereleases may still change any
of this. From 1.0.0 on, this file is the promise, and changing it follows the
policy below.

The promise covers the `caveat-lang` npm package and the runtime it carries.
It does not cover the games, the 3D profiles, the experiments, the Caveatist
material or anything else that shares the repository.

## What is stable

| Surface | What 1.x keeps | Where it is defined |
| --- | --- | --- |
| Language | Syntax, and what an accepted program computes, decides, refuses and explains | [`spec/caveat-0.1.md`](../spec/caveat-0.1.md) and every implemented profile in `spec/`, listed under "Specifications" below |
| Dispatch outcomes | `accepted` or `rejected`; every rejection's `origin` and `code`; a fatal error is neither | [Dispatch 0.1](../spec/caveat-dispatch-0.1.md), "Origins and codes" |
| CLI | The commands, their arguments, exit statuses and `--json` output | `caveat-lang --help`; [kit README](../kit/README.md) |
| Library | The package's exports: `./session`, `./node`, `./scenarios`, `./explain`, `./serve`, `./check`, `./types`, with their TypeScript declarations | `kit/package.json`, `kit/lib/*.d.mts` |
| `serve` | Its request and response lines and operations | [Serve 0.1](../spec/caveat-serve-0.1.md) |
| MCP tools | `caveat_validate`, `caveat_check`, `caveat_test`, `caveat_explain`, `caveat_dependents`, as an authoring bridge | [MCP tools](../kit/docs/MCP.md) |
| Schemas | Every schema named below, by its name | the spec that defines it |
| Saves and archives | As stated under "Saves, archives and source" | [Save 0.1](../spec/caveat-save-0.1.md), [Departure 0.1](../spec/caveat-departure-0.1.md) |

### Schemas

`caveat-dispatch/0.1`, `caveat-reactive-view/0.1` (and `/0.2` once it ships),
`caveat-reactive-save/0.1`, `caveat-explain/0.1`, `caveat-dependents/0.1`,
`caveat-check/0.1`, `caveat-interface/0.1`, `caveat-archive-provenance/0.1`.
Package 1.0 does not rename them. A schema changes its version only by a new
name beside the old one.

### Rejection origins and codes

The origins are `policy`, `input`, `evaluation` and `limit`. The codes in
[Dispatch 0.1](../spec/caveat-dispatch-0.1.md) are permanent: `reject`,
`not_permitted`, `unknown_event`, `payload_invalid`, `bound_exceeded`,
`decision_in_force`, `ungrounded_citation`, `empty_caveated_selection`,
`unobserved_evidence`, `not_committed`, `expression`, `requirement_failed`,
`attention_limit`, `work_limit`, `depth_limit`, `history_limit`,
`identifier_limit`, `renewal_limit`, `scheduled_limit`. A 1.x release never
renames or removes one, and never moves an input from one code to another. It
may add a code only for an input that was refused or fatal before, or for new
syntax. Hosts should treat an unknown code as a refusal.

### Specifications

Stable: every file in `spec/` whose name has no `-draft`, except the profiles
listed under "What is not stable" below. The drafts `caveat-0.2-draft.md` to
`caveat-0.5-draft.md` are not promises, and neither is
`caveat-member-symbols-0.1.md`, whose status is draft and which nothing
implements. A spec's own status line is updated to match this list before
1.0.0.

## What is not stable

- **Games and presentation.** `game-session-0.1`, `caveat3d-0.1`,
  `caveat3d-0.2`, `presentation.md`, the `games` Cargo feature, `web/` and
  `game/`. They may change in any release.
- **`./runtime/*` paths** in the package. They are how the library loads its
  WebAssembly, not an API.
- **Host conformance and `origin: "host"`.** These are future components. No
  1.0 integration depends on them, and they stay in the package README's
  "Not yet" list.
- **Native Rust API.** The `runtime/` crate is not published. Its public items
  are not a promise, and neither are the profiles that exist only there:
  `caveat-rs-0.1` (a bootstrap experiment) and `source-library-0.1`.
- **Diagnostics and messages.** A rejection's `message`, CLI prose, `check`
  warning text and fatal-error text can change. Their codes and statuses
  cannot.
- **Timing, memory and sizes.** See "Limits".

## Versioning policy

- **1.x patch:** a fix that makes behavior match a documented promise.
- **1.x minor:** additions only. New syntax, schemas, operations, optional
  fields, CLI flags or exports. A program, input, save or archive that worked
  before works the same way, with the same output bytes, unless a fix
  restores a documented promise.
- **2.0:** removing or renaming anything stable, changing what an accepted
  program computes, decides or refuses, changing a code, or making a valid
  1.x save unrestorable.
- **Deprecation:** announced in the release notes at least one minor release
  before removal in 2.0.
- The npm version and schema versions move independently.

## Saves, archives and source

- A save binds the exact source it was made under. Restore refuses a save made
  under any other source text, including one that differs only in a comment
  ("it belongs to a different program"). This stays as it is in 1.x.
- With that same source and any later 1.x runtime, a save written by an
  earlier 1.x runtime restores, and later events give the outcomes the earlier
  runtime would have given.
- A save is not a promise across edited source. A host that changes its
  program starts new sessions, or migrates its own data outside Caveat.
- Archives drained from a 1.x session stay readable by `explain` and
  `dependents` in later 1.x runtimes.
- Saves record a SHA-256 of their source, and `explain` shows it beside past
  decisions, so a reader can tell which source a historical decision was made
  under. A save made before that field exists shows the source as "not
  recorded". (Owner decision D6, 2026-10-10; implemented by PR A3.)
- Restore checks that a save is possible for its source. It does not
  authenticate a save, an archive or their history, and a matching digest is
  not proof that a save is genuine.

## Limits

Stable does not mean unbounded. These are limits, not defects, and 1.x may
change their numbers only upward:

- **Held records.** A windowed program refuses an event that would hold more
  than 65,536 windowed records ([Withdrawal Collection 0.1](../spec/caveat-withdrawal-collection-0.1.md)).
  Other histories refuse past their declared `limit`.
- **Saves grow.** Windows and collection do not bound a save. A reachable
  reason chain is required and kept (rc.16: 1.24 MB after 3,000 cycles of a
  required chain).
- **Event time.** No constant or bounded event time. A synchronous release of
  3,000 mutual withdrawal cycles took 55–60 ms in rc.16, which is more than
  one 60 Hz frame. Hosts set and test their own budgets.
- **Archives grow.** Draining moves history to the host; the host stores it.
- **Authentication.** Caveat records the evidence supplied to it. It does not
  authenticate evidence, saves or archives.
- **Crash recovery.** None beyond what a host or starter implements and tests.

## Supported environments

- Node 20 or later (`engines` in `kit/package.json`). CI runs Node 22 on Linux,
  and the MCP worker also on Windows.
- Browsers through the WebAssembly build: CI runs current Chromium and WebKit.
  Firefox is expected to work but is not run in CI.

## How each promise is checked

| Promise | Check |
| --- | --- |
| Language and dispatch outcomes | `cargo test` in `runtime/`; the departure gate's no-window sweep against published rc.15 (`experiments/departure-gate/run.mjs`) |
| Rejection codes | the dispatch table tests in `runtime/src/reactive_outcome.rs` and `runtime/tests/` |
| CLI, library, `serve`, MCP | `npm run test:kit` and `npm run test:kit-package` against the packed tarball |
| Saves across versions | the published rc.15 C3 save restore (`experiments/departure-gate/cross-restore.mjs`); from 1.0, a fixture save per release |
| Browsers | the runtime workflow's browser jobs |

A promise without a check is not yet a promise. Before 1.0.0, each row must
name a check that runs in CI on the release candidate.
