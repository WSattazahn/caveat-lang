# Caveat compatibility commitment

Status: **the promise for 1.0.0 and every 1.x release**, written under
[Path to 1.0](releases/path-to-1.0.md) (PR A2, blocker B2). Changing it follows
the policy below. The 0.1.0 release candidates came before this promise.

The promise covers the `caveat-lang` npm package and the runtime it carries.
It does not cover the games, the 3D profiles, the experiments, the Caveatist
material or anything else that shares the repository.

## What is stable

| Surface | What 1.x keeps | Where it is defined |
| --- | --- | --- |
| Language | Syntax, and what an accepted program computes, decides, refuses and explains | [`spec/caveat-0.1.md`](../spec/caveat-0.1.md) and every implemented profile in `spec/`, listed under "Specifications" below |
| Dispatch outcomes | `accepted` or `rejected`; every rejection's `origin` and `code`; a fatal error is neither | [Dispatch 0.1](../spec/caveat-dispatch-0.1.md), "Origins and codes" |
| CLI | The commands, their arguments, exit statuses and `--json` output. `check` reads a single-file program; it refuses a bundle (`#caveat-bundle 1`) with exit status 2, which `validate`, `test` and the library accept | `caveat-lang --help`; [kit README](../kit/README.md) |
| Library | The package's exports: `./session`, `./node`, `./scenarios`, `./explain`, `./serve`, `./check`, `./types`, `./handoff` (sessions scoped to some events, and replay between sessions), `./saves` (from 1.4.0: a replay save's events read back by name, and replayed onto a changed program), and the integration starter's `./starter` and `./starter/node`, with their TypeScript declarations | `kit/package.json`, `kit/lib/*.d.mts` |
| `serve` | Its request and response lines and operations | [Serve 0.1](../spec/caveat-serve-0.1.md) |
| MCP tools | `caveat_validate`, `caveat_check`, `caveat_test`, `caveat_explain`, `caveat_dependents`, as an authoring bridge, and from 1.5.0 `caveat_dispatch`, which continues one ledger through the save it returns; `caveat_find` is a 1.5 draft ([Find 0.1](../spec/caveat-find-0.1.md)) until a release ships it | [MCP tools](../kit/docs/MCP.md) |
| Schemas | Every schema named below, by its name | the spec that defines it |
| Saves and archives | As stated under "Saves, archives and source" | [Save 0.1](../spec/caveat-save-0.1.md), [Replay Save 0.1](../spec/caveat-replay-save-0.1.md) (from 1.1.0), [Replay Save 0.2](../spec/caveat-replay-save-0.2.md) (from 1.2.0), [Departure 0.1](../spec/caveat-departure-0.1.md) |

### Schemas

`caveat-dispatch/0.1`, `caveat-reactive-view/0.1`, `caveat-reactive-view/0.2`,
`caveat-reactive-view-delta/0.2`,
`caveat-reactive-save/0.1`, `caveat-replay-save/0.1` (from 1.1.0),
`caveat-replay-save/0.2` (from 1.2.0), `caveat-explain/0.1`, `caveat-dependents/0.1`,
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
`identifier_limit`, `renewal_limit`, `scheduled_limit`, from 1.1.0
`already_committed`, from 1.2.0 `foreclosed` and
`caveated_irreversible`, from 1.3.0 `already_resolved` and
`not_retained`, and from 1.4.0 `reason_spent`, `provenance_limit`,
`evaluation_limit` and `wrong_agent`, and from 1.4.1 `text_limit`. A 1.x
release never renames or removes
one, and never moves an input from one code to another. It may add a code only
for an input that was refused or fatal before, or for new syntax. Hosts should
treat an unknown code as a refusal.

### Specifications

Stable: every file in `spec/` except those listed here. Each spec's status
line, under its title, says which it is, and `kit/test/compatibility.test.mjs`
checks that the two agree.

- Drafts: `caveat-0.2-draft.md`, `caveat-0.3-draft.md`, `caveat-0.4-draft.md`,
  `caveat-0.5-draft.md` and `caveat-member-symbols-0.1.md`, which nothing
  implements yet.
- The 1.5 drafts `caveat-across-members-0.1.md`,
  `caveat-reading-caveats-0.1.md`, `caveat-thresholds-0.1.md` and
  `caveat-find-0.1.md`, which become stable when a 1.x release ships them.
- Games and presentation: `game-session-0.1.md`, `caveat3d-0.1.md`,
  `caveat3d-0.2.md` and `presentation.md`.
- Native only: `caveat-rs-0.1.md`, `source-library-0.1.md` and
  `caveat-map-0.1.md`, which the npm package does not ship.
- Timing: `caveat-incremental-evaluation-0.1.md`, a performance design whose
  observable behavior the stable profiles state (see "Limits").
- Repository process: `caveat-verification-0.1.md`, a repository process CI
  runs; it promises nothing to a host.

`caveat-0.1.md` is the founding vocabulary. It is stable, and the reactive
profiles define what the package does with it.

## What is not stable

- **Games and presentation.** `game-session-0.1`, `caveat3d-0.1`,
  `caveat3d-0.2`, `presentation.md`, the `games` Cargo feature, `web/` and
  `game/`. They may change in any release.
- **The specifications listed as not stable** under "Specifications" above.
- **`./runtime/*` paths** in the package. They are how the library loads its
  WebAssembly, not an API.
- **Host conformance and `origin: "host"`.** These are future components. No
  1.0 integration depends on them, and they stay in the package README's
  "Not yet" list.
- **Native Rust API.** The `runtime/` crate is not published. Its public items
  are not a promise, and neither are the profiles that exist only there:
  `caveat-rs-0.1` (a bootstrap experiment), `source-library-0.1` and the
  semantic map `caveat-map-0.1` with its `caveat-map` command.
- **Diagnostics and messages.** A rejection's `message`, CLI prose, `check`
  warning text and fatal-error text can change. Their codes and statuses
  cannot.
- **Timing, memory and sizes.** See "Limits". This includes
  `caveat-incremental-evaluation-0.1`, whose observable behavior the stable
  profiles state.
- **The verification record.** `caveat-verification-0.1` is a repository
  process CI runs; it promises nothing to a host.

## Visible but not promised

With enough users, every behavior a host can observe will be depended on by
somebody, promised or not. This section names behaviors a host can see today
that 1.x does not promise, and what to depend on instead. It narrows nothing
above: where "What is stable", the versioning policy or a stable spec keeps
something, that promise stands.

- **Text written for people.** The text form of a report, printed without
  `--json`, has no schema. `explain`'s text, and `formatExplanation` in
  `caveat-lang/explain`, list each refused or fatal event with its `message`,
  which "Diagnostics and messages" above leaves free to change, so those lines
  change when a message does. Read `--json`, or the report the library
  returns. The one message a host may match is a `policy` rejection's, which is
  exactly the program's quoted literal ([Dispatch 0.1](../spec/caveat-dispatch-0.1.md),
  "The view path"); a `check` diagnostic's `message` and `suggestion` are text
  for people as well ([Check 0.1](../spec/caveat-check-0.1.md)).
- **MCP tool descriptions.** The descriptions of the MCP tools and their
  parameters are text for models and people, and a release may reword them.
  The tools' names, arguments and results are what [MCP tools](../kit/docs/MCP.md)
  defines.
- **An input staying fatal.** A minor may give a code to an input that was
  fatal before (see "Rejection origins and codes"), so a fatal outcome can
  become a rejection. A host should treat a rejection whose code it does not
  know as a refusal, as said above, and should not rely on an input staying
  fatal to stop its own work.
- **A limit refusing.** Refusals at the limits listed under "Limits" may turn
  into acceptance when a 1.x raises the number. A host that needs a bound
  writes it into the program, as a declared `limit` or a rule, where the
  program keeps it.
- **What is not there yet.** An unknown CLI option exits 2 today, and a minor
  may give that option a meaning; minors also add exports, schemas, codes and
  optional fields. Find out what an installed package can do from its
  version (`caveat-lang --version`), not by provoking an error, and ignore
  fields you do not know rather than refusing output that has them.
- **Members the declarations do not name.** The TypeScript declarations in
  `kit/lib/*.d.mts` are the library's shape. A property or method a returned
  object happens to have, and the declarations do not name, may change or go.
- **Starting material.** What `caveat-lang init` writes, and the files under
  the package's `templates/` and `examples/`, are material to copy.
  A release may change them; a project keeps the copy it made.
- **Hosted services.** The hosted MCP server runs a published package under its
  own settings, such as the scenario steps `caveat_test` may run per call and
  which version it runs ([hosted MCP README](../hosted-mcp/README.md)). Those
  are the service's settings, not the package's promise, and they can change
  between releases.

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
- A compact save (`save({ compact: true })`, from 1.1.0) is a replay save
  while the session's accepted events fit in 4,096 bytes and that is smaller,
  and otherwise the same state save `save()` writes. Later 1.x runtimes
  restore it like any save. A runtime before 1.1.0 does not, so a host that
  may move saves back to one keeps using `save()`, which is unchanged.
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
- Browsers through the WebAssembly build: CI runs the kit in current Chromium.
  The games' QA also runs WebKit, which does not test the kit. Firefox is
  expected to work but is not run in CI.

## How each promise is checked

Each row names the steps of the runtime workflow
(`.github/workflows/runtime.yml`) that run its checks on every pull request
and on the release candidate; `kit/test/compatibility.test.mjs` checks that
each named step exists.

| Promise | Check | CI steps |
| --- | --- | --- |
| Language and dispatch outcomes | `cargo test` in `runtime/`; the departure gate's no-window sweep against published rc.15 (`experiments/departure-gate/run.mjs`) | "Test the reactive-only runtime"; "Departure save bound, exact rc.15 compatibility, and restore continuation" |
| Rejection codes | the dispatch outcome tests in `runtime/tests/dispatch_outcomes.rs`; `kit/test/compatibility.test.mjs`, which holds the list above to Dispatch 0.1's catalog and the kit's `KnownRejectionCode` | "Test the reactive-only runtime"; "Developer kit session library and scenario runner" |
| CLI, library, `serve`, MCP | `npm run test:kit`, and `npm run test:kit-package` against the packed tarball; the MCP worker on Linux and Windows | "Developer kit session library and scenario runner"; "Pack the developer kit and use it from a fresh install"; "Exercise isolated authoring workers and lifecycle failures" |
| Schemas | `kit/test/compatibility.test.mjs`: each schema named above is written by the kit or the runtime | "Developer kit session library and scenario runner" |
| View 0.2 and its delta | the delta gate over every tracked program | "View 0.2 delta gate over the whole corpus" |
| Saves across versions | the published rc.15 C3 save restore (`experiments/departure-gate/cross-restore.mjs`); a fixture per release from rc.17 (`kit/test/fixtures/saves/`, written by `scripts/save-fixture.mjs` from the published package): this runtime restores each save, and the events after it give the outcomes, View 0.1 and View 0.2 that release gave, except that rc.18 lists binding-explanation evidence in first-observed order, which is held instead to the same events played live, and that 1.1 adds the caveats Caveat Sources 0.1 (D015) brings to `ordered.cav` and `readiness.cav`, whose views may differ only by those added caveats and are held to the same events played live; with each fixture's drained archive, `dependents` gives the reports the same events played live give (`kit/test/save-fixtures.test.mjs`) | "Departure save bound, exact rc.15 compatibility, and restore continuation"; "Developer kit session library and scenario runner" |
| Replay saves | `runtime/tests/replay_save.rs`, and the kit's compact-save tests in `kit/test/session.test.mjs`: a compact save restores the session that wrote it, with the same snapshot, state save and later outcomes, and each malformed or unreplayable save is refused | "Test the reactive-only runtime"; "Developer kit session library and scenario runner" |
| Browsers | the kit's session library and runner in Chromium | "Developer kit session library and runner in Chromium" |

A promise without a check is not yet a promise. Each release from rc.17 on
adds its save fixture once it is published.
