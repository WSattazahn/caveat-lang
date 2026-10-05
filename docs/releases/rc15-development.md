# CAVEAT 0.1.0-rc.15 development plan

Status: development open, unpublished. The owner's scope proposal
(`rc15-plan.md`, dated 2026-10-04 18:40 PT) was handed over on 2026-10-05 at
01:16 UTC with rc.14's cause, and the owner settled its two open decisions at
01:18 UTC. The cycle opened at 02:25 UTC with the owner's merge line. At
02:38 UTC the owner sent v2 (19:45 PT), which adds PR 7b and PR 7c and records
[`beat-typescript.md`](../design/beat-typescript.md) as the direction for
rc.16 and Glowcap rounds 8 and 9; this file follows v2. Keep the
"Progress" section factual, as `rc13-development.md` does.

The owner wrote the line references against `main` at `7755a4b` (merge of
PR #148). They were rechecked at `51590b4` when this file was committed (see
"Receipts checked" below). Recheck them before editing, because they move.

## Starting point

- The cycle opens at `main` = `51590b4c3cd9ee09ee3603abaa147ee61df87d95`, the
  merge of PR #150 (the rc.14 release). #149 (the OpenAI plugin manifest) is in.
- Tag `v0.1.0-rc.14` peels to `51590b4`. rc.14 was published on 2026-10-05 at
  02:15 UTC through `publish-npm.yml` with provenance; `latest` and `next` both
  name it. The MCP Registry lists `io.github.WSattazahn/caveat-lang`
  `0.1.0-rc.14` as active and latest. The rc.14 publication record is written
  in its own pull request, outside this cycle.
- Only `kit/package.json` owns the candidate version. The cycle-opening commit
  bumps it to `0.1.0-rc.15`, regenerates the bundled installation blocks
  (`node scripts/kit-docs.mjs --write`) and moves `server.json`'s version and
  package version with it.
- The ground rules are the same as rc.13:
  - one PR per item;
  - failing-then-passing tests first;
  - a spec `Changes` entry and a release-note line in the same PR.
- Two items add syntax: an opt-in clock declaration and history windows. Each
  is spec-first, and the spec PR is reviewed before any runtime PR opens.
- PRs merge as merge commits, never squash. Claude merges its own rc.15 PRs as
  merge commits once CI is green, without waiting for the owner's review (the
  owner's message of 2026-10-05 02:25 UTC: "Claude merges its own rc.15 PRs as
  merge commits once CI is green, without waiting for my review.").

## First process item: rc.14's cause

rc.13 shipped the MCP server name `io.github.wsattazahn/caveat-lang` in
`kit/package.json` (`mcpName`) and `server.json`. The MCP Registry grants a
GitHub login the namespace `io.github.<login>` in the login's exact case and
lists an npm package only when the published package's `mcpName` equals the
server name exactly, so it refused rc.13. The lowercase name came from the
owner's plan, and nothing compared it with the login. Published npm versions
cannot change, so the fix was a release: rc.14 changed only the name
([rc.14 development record](rc14-development.md)).

PR 1 makes this impossible to repeat: `scripts/kit-docs.mjs --check` and the
kit's tests refuse a `mcpName` or `server.json` name other than
`io.github.WSattazahn/caveat-lang`, and a `server.json` version or npm package
version other than the kit version. The release procedure gains one line: the
registry name is checked against the GitHub login's exact case before the tag.

## Inputs

1. **rc.14's cause**, above.
2. **Owner decisions of 2026-10-04** on the two design notes
   (`docs/design/integer-tick-time.md`, `docs/design/save-forgetting.md`):
   **A, an opt-in integer clock**, and **A, declared windows with an archive
   hand-off**. The reviewer's riders: the clock needs a named dispatch-time
   refusal for a computed non-integer `after` and the host-side rounding rule
   in the authoring guide; the windows need the basis-chain answer (a commit
   whose `using` reads `committed(x)` inherits that commitment's grounds, so
   in-window bases can pin every occurrence ever observed and nothing departs)
   and a boundedness fixture on round 7's long-play harness, and are built in
   two halves (retire, then depart).
3. **Round 7's remaining work list** (`experiments/glowcap/RESULTS.md: L917–931`):
   ordered grounds (every author rebuilt observation order; three got the CR7
   basis wrong) and the glue. rc.13's `caveat types` measurement
   ([rc.13 release record](v0.1.0-rc.13.md), interface section): 11–12 lines
   added per adapter, 15–21 names checked, no mismatch found, and none of the
   round's recorded failures would have been caught at type-check time. The
   glue's remaining cost is semantic (order, deltas), not names, which points
   at the view, and view bytes are a contract.
4. **Member symbols**: the §11.6 search found no second program
   (`rc13-development.md`, Progress, "§11.6"); the draft stands; not in this
   cycle.
5. **Version Lab**: rc.13 was not yet measured at the last snapshot (402 → 405
   probes on rc.12, register unchanged). When it measures rc.13 and rc.14, the
   `id_text`, caveat-source and membership-record families should flip;
   anything new folds into the record, not the scope.
6. **Caveatism**: the `caveat` plugin is live in Anthropic's directory; the
   OpenAI manifest is on `main` (#149) and its follow-up and Amendment 2 land
   as docs PRs whenever they land; the first `[UNRESOLVED]` item from use
   (attested records) stays open until a second case.

## Owner decisions

Made 2026-10-05 at 01:18 UTC, both as the plan recommended:

- **The basis chain is an authoring rule, not a language rule (A).** A
  language rule would change what grounds mean for every program to solve a
  problem only windowed series have. Rider: PR 4 adds an advisory
  `caveat check` diagnostic (like C005) that fires when a windowed series'
  commit reads `committed()` or `reopened()` of its own series into `using`,
  and PR 6's harness shows the save flat only when the rule is followed.
  **If authors keep tripping the diagnostic, that is the evidence for a
  language rule in rc.16.**
- **Departure (PR 6) may slip to rc.16 (A).** The fixture decides, not the
  calendar. Rider: if the fixture fails, the departure branch stays open and
  this record carries the measured sizes, so rc.16 starts from the numbers.
- **View 0.2**: PR 7's card decides rc.16 or deferral, not rc.15.

## Accepted scope, in PR order

### PR 1 — Open the cycle, and make rc.14's cause impossible to repeat
This file; `kit/package.json` and `server.json` to `0.1.0-rc.15`; the registry
name and version check in `scripts/kit-docs.mjs` (run by `npm run
check:kit-docs` and by the kit's `package-docs` tests in Runtime CI); one line
in the release procedure (`docs/CONSOLIDATION_PLAN.md`, candidate and release
gates).

### PR 2 — Spec: the opt-in integer clock
Docs only, reviewed before PR 3 opens. Changes to Elapsed 0.1 (clock contract,
accumulator), Renewal 0.1 (scheduled qualification), Reactive 0.2
(`clock … integer` declaration), Dispatch 0.1 (two refusals: a fractional `dt`
on an integer clock is `input/payload_invalid`; a reading leaving ±(2^53−1) is
`evaluation/bound_exceeded`), Save 0.1 (the restore check that `elapsed` and
`scheduled_at` are whole and in range for an integer clock). A computed `after`
that is not a whole number of units refuses at dispatch as
`evaluation/expression`, named in the Dispatch catalog; a constant one is a
load error. `docs/AI_AUTHORING.md` gets the opt-in recipe (unit choice, source
conversions, the host rounds once at its boundary and carries the remainder).
Programs without the declaration are unchanged, and the spec says so in one
sentence.

### PR 3 — Runtime: the integer clock
Parser and load-time checks for the declaration and constant `after`; the two
refusals; `elapsed()` and schedule arithmetic on whole units; restore checks.
Fixtures: F267's case on a `step` event in milliseconds (`after 60000` with
steps of 62 and 63 fires on the exact event where the sum reaches 60000); a
schedule across save and restore; the 2^53 bound reachable in about 9,000
events of `1e12`; a fractional `dt` refused with nothing changed; a computed
fractional `after` refused. Gate: every existing program's snapshots, saves,
journals and scenario outcomes byte-identical (rc.13's sweep: 79,855 saves from
151 programs), plus Glowcap's round 7 C-programs converted to the integer clock
as a worked example under `experiments/glowcap/round7/clock/`, with the fade
exact at 60 s.

### PR 4 — Spec: declared windows and retirement
Docs only, reviewed before PR 5. `window N` on a reading stream, renewable
evidence, decision series or the journal; when an event would pass the window,
the oldest record *retires* instead of the event being refused. A retired
record leaves the live graph (`observed(...)` false, relations inert for new
reads, no new citation); its records stay in the session and the save; each
retirement is an effect of the event that caused it; refused events retire
nothing; numbering never reuses a name. `explain` and `dependents` show retired
records with the sequence at which they retired. The spec states the basis
chain as an authoring rule (owner decision above) and specifies the advisory
`caveat check` diagnostic for a windowed series' commit that reads
`committed()` or `reopened()` of its own series into `using` (C007; C005 is
reserved for member symbols). It is specified here and reports from PR 5,
because no windowed series loads before PR 5 adds the declaration. Restore
validation for retired records: a retired record has no live relation; the schema stays `caveat-reactive-save/0.1` if the fields
are additive and old saves restore unchanged, otherwise the note's F247
argument applies and the spec says which.

### PR 5 — Runtime: retirement
The declaration, the retire step inside dispatch, the effects, the read paths,
`explain`/`dependents`, the restore check. Fixtures: each history kind at its
window; a retired occurrence cited by an in-window basis stays explainable; a
refused event retires nothing (atomic rollback, the rc.11 catalog unchanged);
save/restore across a retirement on native, WASM and the installed package.
Gate: the full sweep byte-identical for programs without a window.

### PR 6 — Runtime: departure and the archive, conditional
Retired records that nothing pins (no state lineage or grounds, no in-window
basis, no in-window journal entry, no pending schedule, no withdrawal record)
leave the session into an append-only archive the host drains
(`session.drain_archive()`; kit `explain` reads a drained archive). The save
carries the retired count per windowed history (`STREAM@(retired+N)`
numbering); new refusals for a count past the declaration, a gap or reuse in
numbering, and a citation below the count that no pinned record holds. The
fixture decides whether this PR merges: round 7's `longplay.mjs` on a windowed
Glowcap program that follows the authoring rule, 60 regrowth cycles, save size
flat (C4 reached 40,661 bytes; TypeScript stayed at 345–514). If the save still
grows, PR 6 does not merge, rc.15 ships retirement only, the branch stays open,
and this record carries the measured sizes and what grew.

### PR 7 — View 0.2 design note, with an owner card
Docs only. Ordered grounds (the journal already lists `because` in
first-observed order, `spec/caveat-decision-journal-0.1.md: L31–33`;
`commitment_grounds` are sets) and a delta view (what changed since the last
accepted event, for hosts that re-render) as one `caveat-reactive-view/0.2`
proposal: what it adds, that 0.1 bytes stay available, and the migration for
hosts. Card: implement in rc.16, or defer. Not implemented in rc.15.
Added by the owner at 03:49 UTC: a commitment's `open: true` means reopened and
awaiting a decision, which reads as "active" to a host engineer. The note
proposes a rename for 0.2. The 0.1 line is clarified now, in the side PR
below.

### Side PR — Docs and examples from the rc.14 dogfooding
Added by the owner at 03:49 UTC. Docs and examples only, with no runtime
change. It comes from using the caveat plugin with rc.14, then asking a model
that had never seen the syntax to write the program it expected.
`docs/AI_AUTHORING.md` gains "What the language refuses to do for you", "What
grows the save" and the one-series-per-exclusive-choice pattern. The kit gains
`kit/examples/reviewer-decision/` and `kit/examples/boss-stance/`, each with
scenarios. The View 0.1 line for `open` is clarified, and
`beat-typescript.md`'s round 8 section gains a learnability probe.

### PR 7b — Kit and check: `caveat test` checks first, and C006 `citation-unreachable`
From `beat-typescript.md`: the cause that put `ungrounded_citation` in round
7's first runs. `caveat test` runs `check` on the program before the first
scenario, prints the report with the run, and stops on an error-severity
diagnostic (warnings run on). New advisory diagnostic C006
`citation-unreachable` in `runtime/src/reactive_check.rs`, beside C001–C004
(`L48–51`): a `because` names evidence that no read in the binding's
expression can reach on any path; the usual `allow` comment silences it.
Fixtures: round 7's C2 and C3 CR16 first-run programs
(`experiments/glowcap/round7/runs/C2`, `C3`) must report C006 at the site that
failed with `ungrounded_citation` at dispatch; every repository program checks
clean or is listed. Kit docs: one paragraph in `kit/docs/AGENT_START.md`.
Independent of every other PR.

### PR 7c — Docs: the stray token in the round 7 verdict
`experiments/glowcap/RESULTS.md: L684` reads "98.4%ULEB Drift…"; correct the
typo. The verdict is otherwise unchanged; a one-line PR.

### PR 8 — Lean: reopening retains the earlier revision's caveats and records its cause
The second of the rc.12 list; model extension, theorem, conformance case in
`scripts/verify-lean-conformance.mjs`, README scope paragraph updated.
Independent of everything.

### PR 9 — Publish through the attested path
As rc.13 and rc.14, plus: the owner's MCP Registry publish of `0.1.0-rc.15`
after verification (an update of the rc.14 listing), "Check for new commits"
on the plugin directory page, and the OpenAI plugin upload if a manifest change
merged this cycle. The release record names the registry listing by URL only
after it exists.

## rc.16 and the benchmark: direction, not scope

[`beat-typescript.md`](../design/beat-typescript.md) (owner, 2026-10-04) is
the program that spans rc.15 and rc.16. It decomposes Glowcap round 7's loss
under the registered Rules A and B
(`experiments/glowcap/round7/PROTOCOL.md: L237–256`) into causes, and assigns
each a fix. rc.15 carries the windows (the save-bound failures), the integer
clock, the view 0.2 note (PR 7) and the check-first run (PR 7b). **Nothing
from rc.16 starts in this cycle**: view 0.2 (ordered grounds and a delta
view), the adapter scaffold (`caveat init --host`), the reactive-only WASM
build and the dispatch-path work.

- **Round 8** is the same beat under the registered rules, with fresh blind
  authors on published rc.16 and predictions pre-registered as numbers.
- **Round 9** is the agent-correction benchmark on Caveat's own ground, with a
  protocol pre-registered the way round 7's was.
- No re-reading of round 7, no measure dropped, no margin moved, and no README
  claim before round 8's record exists.

## Direction (recorded, not scoped)

- **MCP session handles.** Still the owner's boundary decision. The plugin is
  live, so the first outside request for a live session over MCP is the signal
  to take it up; until then the stdio bridge stays stateless.
- **A language rule for the basis chain**, if PR 4's diagnostic shows authors
  tripping the authoring rule (owner, 2026-10-05).
- **The register as a program**, **the third Lean theorem** (a rejected step
  preserves every field), **an OpenAI-hosted MCP endpoint** (needs a verified
  domain; not before a host asks).

## Out of scope

Member symbols (no second program; the draft stands), `abstain`, public
`whatif`, scenario string matchers, signed saves, any dispatch reclassification
beyond the rc.11–rc.13 catalog, view 0.2 implementation.

## Order

PR 1 first. Then PR 2 → PR 3 (clock) and PR 4 → PR 5 → PR 6 (windows) as two
sequences that may run side by side (different modules until PR 5 touches
`reactive_save.rs`; PR 5 rebases on PR 3). PR 7, PR 7b, PR 7c and PR 8 any time. PR 9 is the
release once everything merged is green on `main`. Each PR adds its progress
line to this file.

## Receipts checked

Rechecked at `51590b4` when this file was committed:
- `experiments/glowcap/RESULTS.md`: the work list is L917–931 (the plan said
  L915–931; L915 is its heading).
- `spec/caveat-decision-journal-0.1.md`: the first-observed-order sentence is
  L31–33.
- Root `package.json`: `check:kit-docs` is L42 (the plan said L41).
- `docs/design/integer-tick-time.md` and `docs/design/save-forgetting.md` exist.

## Acceptance and verification

The rc.13 rules carry over:
- Pin reproductions before changing behavior: the exact package, source,
  events and actual outcomes.
- Failing-then-passing tests for every runtime change. Restore checks exercise
  native full and reactive-only Rust, built WASM, the kit, the CLI and serve,
  and save and restore.
- Run on the final review commit: the gate list in
  `docs/CONSOLIDATION_PLAN.md`; `npm run test:kit`, `npm run test:kit-package`,
  `npm run audit:kit-security`, `npm run check:kit-docs` and
  `npm run check:notices`; Runtime CI and the Pages gate on the final `main`;
  the Lean gates, with PR 8's theorem and conformance case added.
- Identify the exact tested revision, and report failures and checks not run
  explicitly.
- Publication happens only through `publish-npm.yml` (PR 9). After the
  release, every public page names rc.15: README, AGENTS, the release notes,
  `web/about.html`, the GitHub release and the npm page.

## Progress

- Development opened 2026-10-05 at `51590b4`; `kit/package.json` and
  `server.json` name `0.1.0-rc.15`. The published preview remains rc.14.
- PR 1: the registry name check. Against `7755a4b` (rc.13's `kit/package.json`
  and `server.json`) it refuses with "kit/package.json: mcpName must be
  io.github.WSattazahn/caveat-lang, the GitHub login's exact case"; against
  this commit it passes.
- PR 1 (#152) merged 2026-10-05 as `9ede60e`, after the rc.14 record (#153),
  Amendment 2 (#154) and the Codex manifest fix (#151).
- PR 2: the integer clock specification, docs only. Elapsed 0.1 gains an
  "Integer clocks" section; Renewal 0.1, Reactive 0.2, Dispatch 0.1 and Save
  0.1 each gain the matching rule and a `Changes` entry marked "specified";
  `docs/AI_AUTHORING.md` gains the opt-in recipe and the host's
  round-once-and-carry rule. The three refusals reuse existing codes
  (`input/payload_invalid`, `evaluation/bound_exceeded`,
  `evaluation/expression`); no code is added. Reviewed before PR 3 opens.
  Found while writing it, not changed: a computed negative `after` on today's
  binary64 clock fails as a fatal unclassified error ("qualify after requires
  a nonnegative number of seconds", `runtime/src/reactive.rs: L4119–4122`),
  though the event is rolled back; it goes to the Version Lab to register.
- PR 2 (#157) merged 2026-10-05 as `e539223`, after the owner's review
  approved it; the review's one sentence, that the bounds are checked first,
  is in PR 3 with a fixture for each case.
- PR 3: the integer clock in the runtime. `clock EVENT every STEP integer`
  parses and loads with whole `STEP` and `dt` bounds and not on `tick`; a
  literal `after` outside whole `0..2^53 − 1` is a load error; a fractional
  `dt` inside the bounds is `input/payload_invalid`, and the bounds are
  checked first, so any `dt` outside them is `input/bound_exceeded`; a
  reading leaving `±(2^53 − 1)` is `evaluation/bound_exceeded` (9,007 steps of `1e12` fit, the 9,008th is
  refused); a computed delay outside whole `0..2^53 − 1` is
  `evaluation/expression`; restore refuses fractional or out-of-range clock
  times; the clock metadata adds `integer: true` only when declared; `integer`
  is a clock clause word the module linker never substitutes. Tests:
  `runtime/tests/integer_clock.rs` (9), and a module linker test. Round 7's
  four Caveat programs, converted to milliseconds under
  `experiments/glowcap/round7/clock/`, each fade a taste on the exact step at
  which 60,000 ms have passed. Gate: a new seeded sweep
  (`runtime/examples/save_sweep.rs`, 8 runs of 150 events per program) over
  the 103 repository programs that load as reactive programs, every outcome,
  save and restored save hashed: 123,600 saves, digests identical between
  `main` (`51590b4`) and this branch, no fatal outcome.
- PR 3 (#158) merged 2026-10-05 as `2c3e8b0`, with the VS Code grammar
  highlighting `integer` after a clock's step.
- Side PR: docs and examples. Both examples load and check clean on the
  published `caveat-lang@0.1.0-rc.14`, installed from npm. On that version,
  reviewer-decision's three scenarios pass, and boss-stance's events replay
  with `parry_again` refused as `evaluation/decision_in_force`. On the switch
  back, `stance@2` is reopened because of `attack_rhythm@2` and `stance@3`
  (parry) is committed. The new boss-stance scenarios state those outcomes.
  Both join the corpus the repository's sweeps read (`git ls-files '*.cav'`,
  as `test:interface` does), and a kit test checks, tests and replays each.
- PR 4: the windows specification, docs only. New
  `spec/caveat-windows-0.1.md` (declarations, retirement, the `retired`
  snapshot and save field, restore checks, the basis-chain authoring rule);
  C007 `windowed-basis-chain` in Check 0.1; `Changes` entries in Reactive 0.5,
  the decision journal, Renewal 0.1 and Save 0.1. Defaults chosen where the
  plan left them open: a windowed history holds at most 65,536 records, live
  and retired, until departure (refused with the existing `limit/history_limit`
  or `limit/renewal_limit`); a scheduled qualification on an occurrence that
  retires still applies, inertly; the schema stays
  `caveat-reactive-save/0.1`, since `retired` is additive and old saves
  restore unchanged.
