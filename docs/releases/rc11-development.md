# CAVEAT 0.1.0-rc.11 development plan

Status: development open, unpublished. Written 2026-10-03 from the rc.10 state
of `main`, the Meta Muse Version Lab snapshot of the same day, and Socket's
analysis of the published package. Keep the "Progress" section factual, as
`rc10-development.md` does.

All line references are to `main` at `00be10da2bdbf631e1a2c6a3a240c8c0cc2a6c0e`
(merge of PR #95). Recheck them before editing; they move.

## Starting point

- `main` = `00be10da2bdbf631e1a2c6a3a240c8c0cc2a6c0e`. Tag `v0.1.0-rc.10` =
  `03955daf6ed64914d3c21835d8779b286f856042` (PR #94). Published npm rc.10
  tarball SHA256 `19f82b19c6eed94f9191cc45611d61917e8d1d52e1b834b41e5230aaa751378f`;
  `next` names rc.10, `latest` still names rc.5 (`npm view caveat-lang dist-tags`).
- Only `kit/package.json` owns the candidate version (see `rc10-development.md`,
  "Starting point"). Bump it to `0.1.0-rc.11` in the cycle-opening commit; bundled
  installation blocks are generated from it.
- One open PR predates this cycle: #68 "name a second member by its parameter in
  a routed rule (`$Q`)", branch `feat/evidence-by-member`, opened 2026-09-29,
  draft. Its author's own closing note recommends against adoption: the
  per-pair text expansion costs +57% per `confront`, lowers the loadable
  exhibit count from 61 to 44, and refuses one existing pattern. **Close it
  as superseded by the draft in PR 6 below, unmerged, and keep the branch**
  as the measurement record. Do this before rc.11 work starts. Its game and
  scenarios are salvaged by PR 5. No open issues.

## Inputs and how to treat them

1. **Meta Muse Version Lab snapshot, 2026-10-03** (rc.3–rc.10; 6 distinct WASM
   builds; 276 probes, 187 pins / 89 flips; 250 findings; 355 spec sentences:
   308 pinned, 22 contradicted, 2 split, 2 flip, 18 unmeasurable).
   External evidence, same rule as rc.10: reproduce a reported outcome against
   the exact published package before changing behavior, keep reported
   measurements separate from reproduced ones, and accept none of its broad
   conclusions or proposed syntax (`abstain`) as a contract. Findings are cited
   below as F<n>.
2. **Socket, rc.10**: Supply Chain 80, Vulnerability 100, Quality 100,
   Maintenance 93, License 100. Zero dependencies, no dependency alerts. One
   visible alert ("Unpopular package", time-based). The rc.9→rc.10 diff added
   one low supply-chain alert, "Filesystem access", on the new
   `kit/examples/agent-evidence/save-text.mjs` (`node:fs/promises`, L6, L29–30),
   raising the low-alert count 19→20. `npm view caveat-lang@0.1.0-rc.10 dist`
   has no `attestations`: the package carries no npm provenance.
3. **Repository's own open items**: the bounded restore-validation step in
   `docs/RESTORE_TRUST_BOUNDARY.md` L46–73; "another blind round is needed" in
   `experiments/glowcap/RESULTS.md` L574; deferred adoption items in
   `docs/AGENT_ADOPTION_FOLLOWUP.md` L10–11.

## Why this scope

Of Muse's 26 contradicted/split/flipped register rows, most fall into two
families. rc.10 closed two sites of the first family and documented the second
without changing it. rc.11 closes the rest of the first family, takes the cheap
half of the second, fixes the spec sentences Muse still catches against current
text, and moves publication onto a provenance-attested path. It also settles
PR #68: the `$Q` branch is closed as superseded, its game is kept as a corpus
program, and the design that replaces `$Q` is recorded as a draft
specification. No new syntax, no new WASM-visible language feature, no
reclassification beyond the sites named.

## Accepted scope (proposed), in PR order

Each PR is one kind of change. Do not mix refactors with behavior changes.
Every behavior change ships with a failing-then-passing test written first.

### PR 1 — Classify the remaining recoverable evaluation failures

Spec: `spec/caveat-dispatch-0.1.md`. The catalog (L105–121) says adding a
documented code for a previously unclassified site is a compatible change
(L180–187). Classification must happen at the failure site, never by matching
error strings (`runtime/src/reactive_outcome.rs` L1–2). Template: the rc.10
`AttentionLimit` site at `runtime/src/reactive.rs` L4146–4150, and the
`RejectionCode` enum at `reactive_outcome.rs` L18–34.

Six sites remain fatal `unclassified` where the spec says the event fails or
is rejected (Muse's "spec-says-reject / impl-says-fatal" family):

| Proposed code | Site | Receipts | Muse |
| --- | --- | --- | --- |
| `evaluation/unobserved_evidence` | late `qualify EVIDENCE with CAVEAT` on unobserved evidence | `reactive.rs` L3941; spec `caveat-late-qualification-0.1.md` L32 | F154 |
| `evaluation/unobserved_evidence` | `withdraw` / `withdraw … because` on unobserved evidence, and a stream with no reading | `reactive.rs` L3973–3980; spec `caveat-withdrawal-0.1.md` L42 | F78 |
| `evaluation/unobserved_evidence` | `reopen ACTION because EVIDENCE` on unobserved evidence; today it fails inside `qualify_core` with the wrong message "cannot qualify a value with unobserved evidence" | `reactive.rs` L4320–4336 calling `qualify_core` L3628–3634; spec `caveat-reactive-0.1.md` L167 | F158 |
| `evaluation/expression` | division by zero; history index out of range; nonfinite result — in rule effects **and** bindings (bindings already roll back in the same transaction: `reactive.rs` L3227–3232) | `runtime/src/reactive_expr.rs` L1095, L1260; `reactive.rs` L2767, L2787; spec `caveat-reactive-0.2.md` L88 | F104, F212 |
| `evaluation/requirement_failed` (name confirmed by the owner) | `require(COND, VALUE)` false in an expression | `reactive_expr.rs` `Node::Require`; `caveat-dispatch-0.1.md` L134–139 lists it as unclassified | — |
| `limit/scheduled_limit` | the 4,097th pending `qualify … after` | `reactive.rs` L3955–3959, `MAX_SCHEDULED_QUALIFICATIONS` L29; spec `caveat-renewal-0.1.md` L85 | F184 |

Design note for `evaluation/expression`: `reactive_expr.rs` returns
`Result<_, String>`. Introduce a small typed failure in `reactive_expr.rs`
(e.g. an enum with `DivisionByZero`, `NonFinite`, `HistoryIndex`,
`Requirement`, `Other(String)`) and map the classified variants to
`DispatchFailure::rejected` where expression errors enter dispatch: the rule
path through `execute_guarded_effect` and the binding path through
`evaluate_bindings` (`reactive.rs` L2488). `Other` stays fatal. Do not sniff
message text.

Keep fatal: everything not in the table. The catalog's "every other existing
string error is fatal" rule (L134–139) stands; narrow it only by the six rows.

Diagnostic text is unchanged except the F158 mislabel, which should name the
reopen site; record that one change in the release note.

Consumers to update in the same PR:
- `kit/lib/session.d.mts` L27 union of codes; `kit/test/types.test.mjs`
  declaration/runtime parity; `kit/docs/REFERENCE.md` where codes are listed;
  the parity list in `reactive_outcome.rs` test
  `a_failure_is_the_same_refusal_or_fatal_on_both_paths` (L237–260).
- `spec/caveat-dispatch-0.1.md`: catalog rows, the examples sentence at
  L134–139, and a "Recovering from…" paragraph modelled on L160–178.
- The five spec sentences cited in the table, reworded to name their code as
  rc.10 did for `caveat-state-caveats-0.1.md` L49 and `caveat-reactive-0.2.md` L188.

Acceptance (per site, mirroring `rc10-development.md` L49–60):
- Pin the reproduction first: source, events, exact rc.10 published outcome
  and message, under `test-results/rc11-development/` (untracked).
- Failing-then-passing tests in `runtime/tests/dispatch_outcomes.rs` and
  `kit/test/recovery.test.mjs`: classified `rejected` outcome on `dispatch`,
  `dispatch_outcome`, `dispatch_view_outcome`; whole-event rollback (snapshot,
  save, view, sequence, cues, journal, scheduled table, identifiers); a later
  valid event; save/restore and continuation; serve and CLI replay; the scenario
  runner matching the explicit origin/code.
- Native full and `--no-default-features` Rust, built WASM through the kit,
  installed-package gate.
- No evidence, attention, occurrence or history is invented to recover.

### PR 2 — Spec and documentation corrections only

No runtime change. Extend `kit/test/spec-docs.test.mjs` (added in rc.10) where
a sentence can be checked mechanically.

- `spec/caveat-scenarios-0.1.md` L165: the `size` example uses
  `"since": "start"`, which the spec's own name rules (L137–146) make an invalid
  file. Use `initial`. Add a test that loads the spec's example scenario (F143).
- `spec/caveat-elapsed-0.1.md` L43: "an accumulation that becomes nonfinite
  rejects the event" is unreachable from dispatch because `dt` is bounded; the
  real guarantee is that restore refuses a nonfinite `elapsed`
  (`runtime/src/reactive_save.rs` L487–491). Say that (F151, F172).
- `spec/caveat-save-0.1.md` L76: "has a field this schema lacks" is enforced at
  the top level only (F247). Either state the boundary here, or implement PR 3c
  and leave the sentence.
- Sweep the kit docs and `AGENTS.md` for the new PR 1 codes once PR 1 lands.

### PR 3 — Restore hardening, the cheap half

Reference: `docs/RESTORE_TRUST_BOUNDARY.md` L41–44 (update fixture and contract
together) and L62–71 (acceptance cases). One commit per check, each with
failing-then-passing fixtures in `kit/test/restore-contract.test.mjs` and
`runtime/tests/save_restore.rs`, plus the installed-package and WASM paths.
Genuine saves from each published rc must still restore; only edited saves
may newly fail.

- **3a Cues (F115):** restore validates cue membership only
  (`reactive_save.rs` L1213–1224, "unknown cue"). Also require that some rule
  of the saved `last_event` can emit the cue, using the compiled rules, as the
  existing effect-reachability check does. A declared cue the event never
  emitted must refuse.
- **3b Effect kinds (F248):** restore checks only `EffectReport::Reveal`
  semantics (`reactive_save.rs` L1229–1260). Check every effect's kind against
  the rule at its place (examine, qualify, withdraw, renew, …) with the same
  compiled rule set.
- **3c Nested unknown fields (F247):** `ReactiveSave`, `SavedState`,
  `SavedGraph`, `SavedNode`, `SavedResources` carry
  `#[serde(deny_unknown_fields)]` (`reactive_save.rs` L24, L100, L162, L181,
  L188); `CommitmentBasis` (`reactive.rs` L536–541) and `JournalEntry`
  (`reactive.rs` L1008–) do not, so `"zz": 1` inside `decision_journal[0]` or
  `commitment_bases["plan@1"]` restores silently and is dropped on resave.
  Audit every type reachable from `ReactiveSave` (withdrawals, permission
  records, scheduled qualifications, reading streams, decision series, effect
  reports, `Compact`) and refuse unknown fields on each save-only type. Leave
  the `dependents` report formatter's older-object tolerance alone; that is a
  different schema (`caveat-dependents/0.1`).
- **3d Elapsed:** no runtime change; restore already refuses a nonfinite value
  (L487–491). Covered by PR 2.
- Update the rc.10 characterization fixtures whose expectation changes, with
  the contract text, in the same commit.

The first plan deferred to rc.12 the source-capability validator for injected
`qualifies` edges and pending scheduled qualifications (F95, F122, F123),
scoped at `RESTORE_TRUST_BOUNDARY.md` L46–73. It needs the compiled effect
graph plus occurrence/template analysis and an explicit compatibility decision
for schema-0.1 saves; too large to ride with PR 1. The digest's A3 brings it
into rc.11 after PR 3, under the gate recorded in the digest table below.

### PR 4 — Release engineering

- **4a Provenance-attested publication.** Add a `workflow_dispatch` workflow
  (inputs: tag, expected tarball SHA256) that checks out the exact tag,
  verifies the Runtime workflow succeeded on that revision, obtains the retained
  clean Linux tarball and compares its SHA256 to the input, then runs
  `npm publish <tarball> --provenance --access public --tag next` with
  `permissions: id-token: write` (already used by `.github/workflows/pages.yml`
  L106). Gate the job on a GitHub environment with the owner as required
  reviewer: that approval is the "release authorization" of
  `docs/CONSOLIDATION_PLAN.md` L318–324. Configure npm trusted publishing for
  `caveat-lang` to this repository/workflow. Extend the publication verification
  (`docs/releases/v0.1.0-rc.<n>-npm-publication.json` producer) to fail if
  `dist.attestations` is absent from the registry metadata. This replaces the
  manual Windows publish (`rc10-development.md` L137–141;
  `AGENT_ADOPTION_FOLLOWUP.md` L146–147) for rc.11 onward.
- **4b Declared capabilities.** Add a table to `docs/PACKAGE_SECURITY.md`
  listing which published files legitimately use the filesystem, subprocesses
  and the environment, and a check (in `scripts/audit-kit-security.mjs` or a
  sibling test) that scans the packed tarball for `node:fs`, `fs/promises`,
  `child_process`, `process.env`, `node:http(s)`, `node:net`, `fetch(` and fails
  on any file outside the list. Current list at rc.10:
  filesystem — `bin/caveat.mjs`, `lib/node.mjs`, `lib/doctor.mjs`,
  `lib/demo.mjs`, `lib/mcp.mjs`, `examples/agent-evidence/save-text.mjs`;
  subprocess — `lib/mcp.mjs`; environment — `lib/doctor.mjs`; network — none.
  A future Socket capability alert is then either expected and listed, or a
  regression.
- **4c `latest` dist-tag.** Owner decision (2026-10-03: promote): promote rc.11 to `latest` at
  publication (`npm dist-tag add caveat-lang@0.1.0-rc.11 latest`), or document
  `next` as the only channel in the README and `kit/docs/AGENT_START.md`.
  Recommendation: promote; every quickstart already pins an exact version, and
  `npm install caveat-lang` currently resolves to rc.5.
- **4d Examples in the tarball.** Keep `examples/` in `files` (14 files,
  ~154 KB; the adoption study depends on agents reading the installed package
  offline), and review new examples against 4b.

### PR 5 — Salvage from PR #68: Before the Rain as a corpus program

No language change. The game and its scenarios exist only on
`feat/evidence-by-member`; they are a real program with withdrawal,
`rests_on_withdrawn`, state-caveat reopening and a decision series, and the
repository keeps asking for more such programs.

- Add `game/before_the_rain.cav` and `game/before_the_rain.scenarios.json`
  from commit `b0c763b` on that branch, **in the hand-written form**: take the
  branch's file and replace its two `$shown` blocks (the block routed by
  `shown` and the block routed by `about`, lines 133–144 on the branch) with
  the seven-rule "before" text quoted verbatim in that branch's
  `spec/caveat-routed-repetition-0.1.md` section 10 (a plain block of five
  rules over `$s` and a routed block of four `withdraw` rules over `$w`).
  Everything else in the file is unchanged. The result must load on current
  `main` with no `$shown` anywhere.
- Add the root script `"test:before-the-rain": "node kit/bin/caveat.mjs test
  game/before_the_rain.scenarios.json"` and the `runtime.yml` step from the
  same commit, with the step named "Before the Rain scenarios" (drop "with a
  member named by its parameter").
- Expect 12/12. If a scenario's outcome differs from the branch's, record the
  difference as a finding in this document; do not edit a scenario to pass.
- `caveat check` on the program: record its report; add allow comments only
  where section 6 of the routed-repetition spec already prescribes them.
- Do **not** take from the branch: `runtime/src/repeat.rs`,
  `runtime/src/reactive_check.rs`, the `$Q` tests, the `$Q` sections of
  `spec/caveat-routed-repetition-0.1.md`, `spec/caveat-repetition-0.1.md` and
  `spec/caveat-check-0.1.md`, the `docs/AI_AUTHORING.md` addition, or the
  `kit/docs` lines that mention `$Q`.
- Editor fixes on the branch (`cfd6a7e`, `5710883`: test-only; `979ad76`,
  `9cdbdf6`, `1680ce5`: entangled with `$Q`): cherry-pick only hunks that do
  not mention `$Q` or "names Q", and only if `npm run test:vscode` passes on
  the result. Otherwise leave them on the branch.

### PR 6 — Draft specification: member symbols (docs only)

Add `spec/caveat-member-symbols-0.1.md`, status **draft, not implemented**,
from the drafted text that accompanies this plan. It records the design that
supersedes `$Q`: a rule names the symbol belonging to the member one of its
event's `kind` parameters names, resolved at dispatch by family lookup, with
every check at load. It needs no copy of a rule per pair of members, so the
cost that sank #68 does not arise, and it writes the `hold` rule that `$Q`
could not.

- Link it from `spec/caveat-routed-repetition-0.1.md` section 9 ("A rule
  about another member's event") as the proposed answer, one sentence.
- Record the decisions it asks of the owner (its section 12) when they are
  made; the sigil `[Q]` is a placeholder.
- **No implementation in rc.11.** Adoption follows the bar #68 set for itself
  and the draft's section 11: a prototype that is byte-equivalent to the
  hand-written Before the Rain on its 12 scenarios and the 3,000-history
  differential, at or below hand-written cost, with every existing program
  loading unchanged, and a second real program that is simpler with it.

## Version Lab digest additions

The owner's second rc.11 plan (2026-10-03), from the Version Lab digest
(findings through F260, measured on published rc.3 to rc.10), adds to the scope
above. It was re-checked against `00be10d`, so the lab's rc.10 measurements
describe the starting point. The same evidence rule applies: reproduce each item
in a repository regression before changing behavior, and name regression cases
after the finding they reproduce.

Already resolved at the starting point, no work: F169 (view fields), F249 and
F250 (the two rc.10 classifications), and F105 with the prose half of F95 (the
save contract already states these). The lab's stability pins stay green; per
F259, new `dependents` fields are additive keys only.

| Digest item | Finding | Where it lands |
| --- | --- | --- |
| A1 nested unknown save fields | F247 | Restore hardening (PR 3 above, 3c) |
| A2 kind and reachability checks for last-event effects and cues | F248, F115 | Restore hardening (PR 3, 3a and 3b) |
| A3 source-capability check for restored `qualifies` edges and schedules | F95, F260 | In rc.11, the restore thread's PR after PR 3 (owner, 2026-10-03). Policy: refuse a pair only when no source mechanism can create it; accept when uncertain. Gate: before the forgery refusals enter the contract, the genuine fixtures for all six creation paths and the whole existing save/restore corpus (the 3,000-round fuzz, every scenario with a `resume` step, kit `restore-contract`) pass with the check on. If a genuine path cannot pass conservatively by the time PR 4's publish path is ready, A3 moves to rc.12; `spec/caveat-save-0.1.md` already documents the gap |
| B1 the 4,097th pending qualification | F184 | PR 1, as `limit/scheduled_limit` (the digest's `schedule_limit` predates PR 1) |
| B2 unobserved qualify and reopen, uncommitted reopen, mislabelled diagnostic | F154, F158 | PR 1: `evaluation/unobserved_evidence` and `evaluation/not_committed` |
| B3 expression failures at dispatch | F212 | PR 1, as `evaluation/expression`; the owner chose to keep this code (2026-10-03) and asked that a negative `sqrt` and `latest` of an empty stream or series refuse under it too, so the line falls where a host can predict |
| B4 guard the elapsed accumulator | F151 | Its own PR; owns `spec/caveat-elapsed-0.1.md` (moves out of PR 2) |
| C1 the function limit counts the prelude | F257, F258 | Its own PR |
| C2 the scenarios spec's `size` example | F143 | PR 2, with an executable check of the spec's step examples |
| C3 say what "strict subset" applies to | R54 | PR 2, one sentence in `spec/caveat-save-0.1.md` |
| C4 keep the dispatch still-fatal list true | — | PR 1 |

Out of scope, from the digest: `abstain` (a feature request, if the lab posts
one, goes through the `AGENTS.md` process and is not implemented in rc.11; none
was open on 2026-10-03), authenticated host persistence or signed saves (A3
narrows acceptance and does not authenticate history), and any reclassification
beyond B1 to B3.

## Explicitly out of scope for rc.11

- `abstain` syntax, public `whatif`, scenario string matchers — still deferred
  (`AGENT_ADOPTION_FOLLOWUP.md` L10–11); Muse's `abstain` proposal was not
  accepted in rc.10 (`rc10-development.md` L42–47).
- Performance work (51.6 µs/event vs 2.5 µs TypeScript; ~483 KB gzipped). The
  measured `dispatchView` win is already shipped
  (`experiments/performance-opt-0.1/RESULTS.md`).
- Glowcap blind round 7 (`experiments/glowcap/RESULTS.md` L574). Run it against
  the tagged rc.11, as an experiment, after publication.
- Source-capability restore validation (rc.12, above).
- Implementing member symbols (PR 6 is a draft specification only) or merging
  any `$Q` runtime code from `feat/evidence-by-member`.
- Any new language feature, any blanket fatal→refusal reclassification, any
  change to registered experiment inputs or results.
- Muse's own open threads (its backfills, fn-limit probes, "on all six" sweep).

## Acceptance and verification

- Pin reproductions before changing behavior (PR 1, PR 3): exact package,
  source, events, actual outcomes.
- Failing-then-passing tests for every classification and every restore check,
  exercising native full and reactive-only Rust, built WASM, kit/CLI/serve,
  structured outcomes, save/restore, and a successful event after each refusal.
- PR 5: `npm run test:before-the-rain` passes 12/12 on the hand-written
  program, the file contains no `$shown`, and `caveat check`'s report on it is
  recorded here.
- Run on the final review commit: the gate list in
  `docs/CONSOLIDATION_PLAN.md` L150–165, plus `npm run test:kit`,
  `npm run test:kit-package`, `npm run audit:kit-security`,
  `npm run check:kit-docs`, `npm run check:notices`; Runtime CI and the Pages
  gate on final main; existing Lean gates unchanged (no new proofs claimed).
- Keep verification receipts outside tracked source; identify the exact tested
  revision; report failures and checks not run explicitly.
- Publication only through PR 4a after release authorization; record the
  attested publication in `docs/releases/v0.1.0-rc.11.md` and the
  `-npm-publication.json` file as rc.10 did.

External expectations, not gates: on Muse's next run, six probes flip at the
rc.11 column (`attention_budget`-style patterns gain a new final `X`), the
F115/F247/F248 save-mutation probes flip to refusals, and the contradicted
count drops from 22 to roughly 8. On Socket, no capability alert outside the
4b list, and a provenance attestation on the rc.11 version.

## Verification boundaries

Unchanged from rc.10: the root `npm run typecheck|lint|test|audit:all` aliases
do not exist and are not gates; the owner's Windows checkout is not a clean
reproducible release build; Lean checks cover their stated fragment only.
Muse measurements remain external evidence until reproduced here.

## Working notes for the implementing agent

- Read `AGENTS.md`, `docs/CAVEAT_ESSENCE.md` and the relevant `spec/` file
  before touching runtime semantics. Preserve evidence history, qualifications,
  frozen decision bases and atomic rollback.
- `npm run build` before `npm run test:kit` or `npm run test:kit-package`;
  runtime changes also need the workflow's Rust and affected browser checks
  (`.github/workflows/runtime.yml`, `core`, `reproducible`, `kit-security`).
- One PR per item above; small reversible commits; no new `eslint-disable`
  or `#[allow]` without saying why; no refactors folded into behavior changes.
- Do not change diagnostic text except where this plan says so.
- Do not run formatters over, re-register, or re-hash anything under
  `experiments/`.
- Report the exact revision, commands and outcomes, including failures and
  checks not run.

## Progress

- Development opened 2026-10-03; `kit/package.json` names `0.1.0-rc.11`.
  The published preview remains rc.10.
- PR #68 closed unmerged as superseded; `feat/evidence-by-member` kept.
- Owner decisions: the `require(false)` code is
  `evaluation/requirement_failed`; arithmetic failures at dispatch stay refusals
  as `evaluation/expression` (digest B3); rc.11 is promoted to `latest` at
  publication, through the PR 4a workflow only.
- PR 1 (#96): the six sites above plus the digest's uncommitted reopen
  (F158) refuse with classified codes. At the owner's review, a negative
  `sqrt` and `latest` of an empty stream or series also refuse as
  `evaluation/expression`, and `reopen ACTION because latest(STREAM)` on an
  empty stream as `evaluation/unobserved_evidence`, matching `withdraw`.
  Still-fatal lists in the dispatch spec and `REFERENCE.md` updated.
- A3 accepted for rc.11 with the gate in the digest table (owner, 2026-10-03).
