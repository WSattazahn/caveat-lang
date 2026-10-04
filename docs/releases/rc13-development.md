# CAVEAT 0.1.0-rc.13 development plan

Status: development open, unpublished. The owner's scope proposal (v2) was
written 2026-10-04 at 10:40 PT and handed over the same day. It was written
from rc.12's published state of `main`, the Version Lab on published rc.12,
Glowcap round 7 and rc.12's own record. Keep the "Progress" section factual,
as `rc12-development.md` does.

The owner wrote the line references against `main` at `2ae3bb1` (merge of
PR #121); #122 (`8b1d2f3`) did not move them. They were rechecked at
`8b1d2f3` when this file was committed (see "Receipts checked" below).
Recheck them before editing, because they move.

## Starting point

- The cycle opens at `main` = `8b1d2f3b99884c15c231f56fe9911c50558ed9f3`,
  the merge of PR #122. #120, #121 and #122 are all in.
- Tag `v0.1.0-rc.12` peels to `9e360afab3ff8ff0a92dd31af62cefda3cb102b5`
  (merge of PR #119).
- The published npm rc.12 tarball has SHA256
  `2a22cc7a711418ed32f508a23de6c19aed66f1bd52c2c61548a79183eab20d15`.
  `latest` and `next` both name rc.12. The GitHub pre-release is up.
- Only `kit/package.json` owns the candidate version. The cycle-opening
  commit bumps it to `0.1.0-rc.13`, and the bundled installation blocks are
  generated from it (`node scripts/kit-docs.mjs --write`).
- The ground rules are the same as rc.11 and rc.12:
  - one PR per item;
  - failing-then-passing tests first;
  - a spec `Changes` entry and a release-note line in the same PR;
  - no new syntax except behind the member-symbols gate (below).
- PRs merge as merge commits, never squash. Claude merges its own rc.13 PRs
  as merge commits once CI is green, without waiting for the owner's review
  (the owner's plan, line 3, and their message of 2026-10-04 17:00 UTC: "each
  merging its own PR").

## First process item: the attested publish path, again

rc.12 was the first release published through `publish-npm.yml`: npm
provenance, the `npm-publish` approval, and `verify-publication` passing with
`dist.attestations` present ([release record](v0.1.0-rc.12.md)). **rc.13
ships the same way, or it does not ship.** PR 11 describes that path. The
trusted publisher is already configured, so the first attempt should publish.

## Pacing

Scope is set by the evidence, not by capacity. PR 1–7 are the items the
record requires; PR 8–10 are the items rc.12 explicitly deferred "unless
capacity appears" (`docs/releases/rc12-development.md: L349–350`) and round
7's largest finding. Threads run in parallel where files do not overlap
(noted per PR), and each merges its own PR once CI is green. Every PR is
independently shippable. rc.12's "stop after any PR" clause does not carry
over.

## Inputs

1. **Version Lab on published rc.12** (dashboard of 2026-10-04): 10 builds, 402 probes, 379 findings; F377 records rc.12 as measured, F378/F379 flip the attention-naming row to rc.12; the register is unchanged (392 pinned, 23 contradicted, 15 flip, 7 split, 20 unmeasurable) and no new contradiction appeared. What stays open from the register, with the findings that name it: the `id_text` residual fatal (F309–F312, F338–F339); forged qualification-record *content* that restores live (F330, F333, F335–F337); the membership rewrite whose graph skew nothing notices (F317–F322).
2. **Glowcap round 7** (`experiments/glowcap/RESULTS.md: L54–64, L893–931`): TypeScript; neither decision rule met; "No claim that Caveat is better than TypeScript goes into the README, AGENTS, the site or other documents on the strength of this round" (L912). Its work list (L915–931): a save that forgets what the program forgot; ordered grounds; timing to the tick; the adapter glue (95–113 lines per program). The README's evidence table still stops at round 6 (`README.md: L155–161`); round 7 appears nowhere in README, AGENTS, `web/` or `docs/WHY_CAVEAT.md`.
3. **Member symbols** (owner decision 2026-10-04, `docs/releases/rc12-development.md: L312–322`): draft stays; the implementation branch may start after rc.12; §11.6 (a second program, `spec/caveat-member-symbols-0.1.md: L324`) stays the merge gate; the second program is searched for in what exists, not written to order.
4. **rc.12's own record** (`docs/releases/rc12-development.md: L324–372`): direction items (MCP session handles, Lean theorems, register as a program, the narrow claim) and the out-of-scope list (`abstain`, `whatif`, string matchers, further dispatch reclassification, signed saves) carry forward unchanged except where PR 4 below reclassifies one named site.
5. **The package README's links 404 on Socket** (owner, 2026-10-04): `kit/README.md` links to `docs/AGENT_START.md`, `docs/GETTING_STARTED.md`, `docs/REFERENCE.md`, `docs/MCP.md`, `docs/reference/spec/…`, `examples/agent-evidence/…` (15 relative links, lines 34–407). Those paths are right inside the installed package (every one is in the rc.12 tarball) and on GitHub's view of `kit/`, but Socket, and npm's own README renderer, resolve them against the repository root as `blob/HEAD/docs/…`, ignoring `repository.directory: "kit"` (`kit/package.json`), so they 404. The rc.12 README on npm is frozen; the fix lands with rc.13's publish.

## Accepted scope, in PR order

### PR 1 — Open the cycle
`docs/releases/rc13-development.md` from the rc.12 template with inputs 1–4 recorded; `kit/package.json` bumped to `0.1.0-rc.13` (only that file owns the version). Record that rc.12 was the first attested publish and that rc.13 ships the same way or not at all.

### PR 2 — Package README: links that resolve on npm and Socket (input 5)
`kit/README.md` only. Replace each of the 15 relative links with the absolute `https://github.com/WSattazahn/caveat-lang/blob/main/kit/<path>` form (the hero image already uses the raw URL for the same reason), and add one sentence near the top: "These links open the current documentation on GitHub; the copies that shipped with this version are in the package's own `docs/` and `examples/` directories." Add a check to `scripts/kit-docs.mjs --check` (`check:kit-docs`, root `package.json: L41`) that every `github.com/WSattazahn/caveat-lang/blob/main/` link in `kit/README.md` names a path that exists in the checkout, and that no relative link remains, so the next 404 fails CI instead of appearing on Socket. `npm pack --dry-run` unchanged; `pack-docs.json` unchanged. Not pinned to the release tag: a tag-pinned link 404s on GitHub for the whole cycle until the tag exists, and the exact shipped docs are in the tarball already.

### PR 3 — Docs: round 7 enters the evidence, and the claim stays narrow
Docs only. Add round 7 to the README's benchmark table (`README.md: L155–161`): "7 · fresh authors, blind requests, rc.11 | 186 vs 282 (CR13–16) | TypeScript. Two of four phases on the first run against four; more regressions; no Caveat save stayed bounded with a fast resume in long play. Neither decision rule met." Add one sentence after the table linking the round 7 work list, and the sentence from `RESULTS.md: L912` verbatim. `docs/WHY_CAVEAT.md: L13–16` keeps "does not establish a limit on what TypeScript or its libraries can express" and gains "changed sixteen times across seven rounds; round 7 went to TypeScript". If `web/` carries the table, the same row; pages gate decides. The first screen's claim ("keeps evidence and caveats with computed values, freezes a decision's grounds, and records why it was reopened") is already the narrow one; do not broaden it. Nothing else changes.

### PR 4 — Dispatch: `id_text` on a non-handle refuses (F309–F312, F338–F339)
The last surveyed native-function error path that is still fatal. `id_text(h)` with `h` not a handle the session holds returns the host's `Err(String)` (`runtime/src/reactive.rs: L3500–3506`), and the `?` at `runtime/src/reactive_expr.rs: L1405` converts it through `From<String>` into `EvalFailure::Other` (`reactive_expr.rs: L71–79`), which `DispatchFailure` keeps fatal (`runtime/src/reactive_outcome.rs: L215–229`). The failure is a function argument outside its domain, exactly `EvalFailure::Domain` (`reactive_expr.rs: L37–38`), so classify it there: `identifiers(arguments[0]).map_err(|m| EvalError::new(EvalFailure::Domain, m))?`, and the outcome is `evaluation/expression` with the existing message. `h == 0` stays the empty string (L1402–1404). The closure type (`L1136`) and the serve host's `id` parameter check (never session-fatal, F339) are unchanged.
- Tests: `id_text(7)` with no such handle in guard position, rule-body position and a binding, each on native, WASM and `caveat serve`: outcome `rejected`, origin `evaluation`, code `evaluation/expression`, nothing changed, the session accepts the next event; the stdlib `require` path still reports `evaluation/requirement_failed`; a dispatch-outcomes scenario in `scripts/test-dispatch-outcomes.mjs`.
- Spec: `spec/caveat-dispatch-0.1.md` gains the site under "Recovering from unobserved evidence, expression failures and the scheduling limit" (L185) and a `Changes` entry (L222); the still-fatal paragraph (L139–144) keeps "a type error in an expression" as its example. Release-note line: one more fatal becomes a refusal; no accepted event changes.

### PR 5 — Restore: a record's caveats must be ones the source can attach (F330, F333, F335–F337)
Same shape as rc.11's A3 check for `qualifies` edges and rc.12's #110/#118. Today `check_provenance` (`runtime/src/reactive_save.rs: L1132–1151`) accepts any *declared* caveat in a restored provenance, so a forged caveat in `observation_qualifications`, `examination_qualifications` or `reopening_qualifications` restores live through the commit `using` read path and the `examined()`, `reopened()` and `committed()` guards (F333, F335–F337). The graph already knows which (caveat, evidence) pairs the source can produce: `qualification_sources()` (`L693–741`), used for edges at `L776`.
- Check, conservative: for each restored provenance, every caveat it carries must pair with some evidence of that provenance under `possible` — `(caveat, source_evidence(e))` for some `e` in the provenance's evidence, or `withdrawn` where the program withdraws. For a grounds provenance, pair against the lineage's evidence (grounds ⊆ lineage). A caveat that could attach to any evidence present is accepted without proof it did; a caveat no mechanism attaches to any of them is refused with `{what}: {caveat} cannot qualify any of its evidence`. Apply through `check_provenance` so the three tables (`L1361–1400`), predicate guards (`L1402–1430`), states, bases and journal entries all get it in one place.
- Gate: the rc.12 sweep (17,280 genuine saves from 100 repository programs, `docs/releases/rc12-development.md: L508–510`) plus round 7's Caveat programs and the caveatism canon restore with zero newly refused. If any genuine save fails the check, the check is wrong, not the save: narrow it and record the case.
- Fixtures: forged caveat on an observation record that no mechanism attaches (refused); the same caveat where a `qualify` in an unreached guard could attach it (accepted, documented as the boundary); analogues for examination and reopening records, a predicate guard, a state's grounds and a commitment basis; the fuzz's insert-key operator extended to provenance caveats. Spec: one sentence under restore validation and a `Changes` entry in `spec/caveat-save-0.1.md` (L70, L245); `docs/RESTORE_TRUST_BOUNDARY.md` updated the way #110 and #118 did.

### PR 6 — Bounded investigation: the membership rewrite's permanent skew (F317–F322)
A fully consistent rewrite of the nine membership records restores, later genuine events heal the tables but not the graph, and the `reopens`/`relies_on` skew is permanent with no surface noticing. The thread's deliverable is one of two, decided by the corpus run, within a day of work: (a) the graph→table direction of the invariant as a restore check — every `reopens` edge has a reopening record, every examined caveat an examination record, every observed-and-qualified evidence an observation record, whichever of these every genuine save satisfies (same sweep as PR 5; zero newly refused), with fixtures and a save-spec sentence; or (b) if no such invariant holds for all genuine saves, a one-page design note `docs/design/MEMBERSHIP_CONSISTENCY.md` stating the invariant the runtime does maintain, why restore cannot check it conservatively, and what a future save format would need. Either way the record says which. No speculative redesign.

### PR 7 — Lean: one more invariant in the proven fragment
The fragment (`proofs/lean/README.md`; 68 theorems in `laws.json`) proves provenance union laws, grounds ⊆ lineage through the core combinators, citation acceptance and the accepted/rejected/fatal boundary; it does not model commitments or late qualification. Add the first of rc.12's listed next theorems: a commitment's basis and grounds are unchanged by a later qualification of its evidence (Late Qualification §3, `docs/CAVEAT_ESSENCE.md: L5–14`). Model extension, the theorem, a conformance case in `scripts/verify-lean-conformance.mjs` that runs the same scenario on the runtime, and the README's scope paragraph updated to say exactly what is now proved and what still is not. No claim beyond the theorem. Independent of every other PR.

### PR 8 — Runtime: a program's interface as JSON (`caveat-interface/0.1`)
The first half of round 7's glue item. A new export beside `check` (`runtime/src/web.rs: L33`): `interface(source)` returns, for a loaded program, its events with their typed parameters (`target kind mushroom`, `sort in glowcap duskcap` — Typed Parameters 0.1) and payload fields, its states with ranges, its bindings by `TARGET.PROPERTY` with value type (number or text, as `check` already infers at `reactive_expr.rs: L1004`), its cue names, its decision series, and its evidence, caveat and claim names. Load-time facts only; nothing evaluated; stable field order; schema `caveat-interface/0.1` in a new `spec/caveat-interface-0.1.md`. Tests: byte-stable output for every repository program, native and WASM; a program that fails to load returns the load error, not a partial interface. Runs in parallel with PR 4–6 (different modules).

### PR 9 — Kit: `caveat types PROGRAM` emits TypeScript declarations
The second half. From PR 8's JSON, generate a `.d.ts` with a discriminated union of events (name and payload), the binding map (target → property → type), cue and decision name unions, and typed `dispatch`/`view` wrappers over the existing session API. Bar, measured and recorded, not asserted: round 7's four Caveat adapters are plain JavaScript (`experiments/glowcap/round7/runs/C1–C4/impl/adapter.mjs`, 115–136 lines each), so for each one add `// @ts-check` and a JSDoc import of the generated declarations, run `tsc --checkJs --noEmit`, and record (a) every program name or payload shape the adapter spells by hand that the types now carry, (b) every mismatch the check finds between the adapter and the program, and (c) whether any of the round's recorded blind-phase failures would have been caught at type-check time. Small numbers are the finding and the command still ships as a kit utility; no README claim either way. Kit docs: one section in `kit/docs/AGENT_START.md` and `REFERENCE.md`. Follows PR 8.

### PR 10 — Design notes for two round 7 items, with owner cards
Docs only, in parallel with everything. One page each under `docs/design/`: (a) **a save that forgets what the program forgot** — what "forgotten" can mean for the record given that evidence history is preserved (compaction of observed-but-dropped readings versus a bounded stream with a declared window), what each does to `explain`, `dependents` and restore validation, and the migration for existing saves; (b) **integer tick time** — a clock whose readings are integers in a declared unit so schedules fire exactly, what changes in `caveat-renewal-0.1` and the elapsed spec, and how an existing float-clock program opts in. Each ends in a card with two or three options and a recommendation. Neither is implemented in rc.13.

### PR 11 — Publish through the attested path
As rc.12: tag → `publish-npm.yml` dispatched with `latest` → owner approves `npm-publish` → `verify-publication` passes with `dist.attestations` present → owner moves `next` by hand → release record from the verification artifact, with Socket's result as #122 did. The trusted publisher is configured now, so attempt 1 should publish.

## Member symbols (parallel, gated, not in the PR order)
The §11.6 search runs first and alone: look for the second program in the agent ledger and round 7's Caveat programs; report whether any is simpler with `ev_[shown]`. If one is found, the implementation branch opens after PR 5 and merges under the full §11 bar; if none, the finding goes in the rc.13 record and the draft stands. No implementation work before the search reports.

## Owner decisions

- The plan's line 3: Claude merges its own rc.13 PRs as merge commits once
  CI is green, without waiting for the owner's review.
- Member symbols (2026-10-04, recorded in `rc12-development.md`): the draft
  stays; the implementation branch may start after rc.12; §11.6 stays the
  merge gate; the second program is searched for in what exists, not written
  to order.
- PR 10's two design notes each end in a card for the owner; neither is
  implemented in rc.13.

## Direction: rc.14 and after (recorded, not scoped)
- **The delta view and unchanged-guard skipping.** The rest of round 7's glue item once PR 9's numbers are in; a delta view is a view 0.2 decision because view bytes are a contract.
- **Ordered grounds in the view.** The journal already lists `because` in first-observed order (`spec/caveat-decision-journal-0.1.md: L31–32`); `commitment_grounds` are sets. Every round 7 author rebuilt the order. Same view 0.2 decision as above; PR 8's interface does not change the view.
- **MCP session handles** (owner boundary decision), **the remaining Lean theorems** (reopening retains the earlier revision's caveats; a rejected step preserves every field), **the register as a program**: unchanged from rc.12's direction list.

## Out of scope
- `abstain`, public `whatif`, scenario string matchers, signed saves: unchanged.
- Any dispatch reclassification beyond PR 4; "a type error in an expression" stays the fatal example.
- Member-symbols implementation without the §11.6 program.
- Any claim about TypeScript beyond what PR 3 records.

## Order
PR 1 first. Then in parallel: PR 2 and PR 3 (docs, disjoint files); PR 4 (`reactive_expr.rs`); PR 5 then PR 6 (both in `reactive_save.rs`, so in sequence); PR 7 (`proofs/`); PR 8 then PR 9; PR 10 any time; the §11.6 search alongside. PR 11 is the release once everything merged is green on `main`. Each thread rebases on `main` before merging and adds its progress line to `rc13-development.md` in its own PR.

## Receipts checked

Rechecked at `8b1d2f3` when this file was committed. They hold, with these
notes for the owning threads:
- `README.md: L155–161` is the table's header through its last row; the
  bullet list above it starts at L150.
- `docs/WHY_CAVEAT.md: L13–16` says the benchmark "changed eight times"; PR 3
  replaces that count.
- The round 7 Caveat adapters (`experiments/glowcap/round7/runs/C1–C4/impl/adapter.mjs`)
  are 115, 136, 129 and 127 lines. Input 2's "95–113 lines per program" is
  RESULTS.md's count of adapter glue, not of those files; PR 9 records which
  count it measures against.
- `kit/README.md` has 16 lines carrying a relative `docs/` or `examples/`
  link; PR 2 counts the links themselves.
- `id_text`: `runtime/src/reactive.rs: L3500–3506`, `runtime/src/reactive_expr.rs: L1402–1405`
  and `L71` (`From<String>`), `EvalFailure::Domain` at `L38`, the `IdText`
  type at `L1004`; `qualification_sources()` at `runtime/src/reactive_save.rs: L693`
  and `check_provenance` at `L1132`; `interface` would sit beside `check` at
  `runtime/src/web.rs: L33`; `proofs/lean/laws.json` lists 68 laws.

## Acceptance and verification

The rc.12 rules carry over:
- Pin reproductions before changing behavior: the exact package, source,
  events and actual outcomes.
- Failing-then-passing tests for every runtime change. Restore checks
  exercise native full and reactive-only Rust, built WASM, the kit, the CLI
  and serve, and save and restore.
- Run on the final review commit:
  - the gate list in `docs/CONSOLIDATION_PLAN.md`;
  - `npm run test:kit`, `npm run test:kit-package`,
    `npm run audit:kit-security`, `npm run check:kit-docs` and
    `npm run check:notices`;
  - Runtime CI and the Pages gate on the final `main`;
  - the Lean gates, with PR 7's theorem and conformance case added.
- Identify the exact tested revision, and report failures and checks not run
  explicitly.
- Publication happens only through `publish-npm.yml` (PR 11). Record it in
  `docs/releases/v0.1.0-rc.13.md` and the `-npm-publication.json` file, built
  from the `npm-publication-verification` artifact. After the release, every
  public page names rc.13: README, AGENTS, the release notes,
  `web/about.html`, the GitHub release and the npm page (the packed kit
  README).

## Progress

- Development opened 2026-10-04 at `8b1d2f3`; `kit/package.json` names
  `0.1.0-rc.13`. The published preview remains rc.12.
- The scope recorded here is the owner's v2 of 2026-10-04 at 10:40 PT. It
  adds PR 7 (one Lean theorem), PR 8–9 (the program interface and
  `caveat types`) and PR 10 (design notes with cards), lets threads run in
  parallel where files do not overlap, and renumbers the publish to PR 11.
- PR 1 (#123) merged 2026-10-04 as `592b492`.
- Member symbols §11.6: the search for a second program ran first and alone,
  as the plan asks, and was re-run at `8b1d2f3`. Across all 132 tracked
  `.cav` files, including the agent ledger's four programs and every version
  of round 7's four Caveat authors (C1–C4, final sources and each commit of
  their phase bundles), the only events with two or more `kind` parameters
  are still Before the Rain's `ask`, `confront` and `hold`
  (`game/before_the_rain.cav`). The two files new since rc.12's search,
  `caveatism/canon/archive.cav` and the F268 regression fixture, take no
  event parameters. The ledger's one cross-member rule (`release.cav`, `on
  merged when target != $index`) reaches every other member, which a
  reference naming one member cannot express. Round 7's C3 memory block
  could be written with references, in fewer lines but with twice the rules
  on each observation event (against §11.2), and it needs a grounded value,
  which a state already carries, rather than a name. So no second program
  exists; §11.6 is unmet, the draft stands, and no implementation branch
  opens in rc.13. This is a finding about the feature: outside Before the
  Rain, no program in the repository has the shape a reference serves.
- PR 2 (#126): all 16 relative links in `kit/README.md` now open the file
  on GitHub's `main`, and a sentence near the top says the shipped copies are
  in the package's `docs/` and `examples/`. The six `docs/reference/` links
  point at their repository source paths, because `kit/docs/reference/` is
  generated and untracked (`.gitignore`), so `blob/main/kit/docs/reference/`
  would 404. `scripts/kit-docs.mjs` refuses a relative README link and any
  `blob/main/` link to a file `git ls-files` does not list; the check runs in
  `kit/test/package-docs.test.mjs` under `npm run test:kit`. The npm page and
  Socket show the change only after rc.13 publishes.

Sources: the repository at `2ae3bb1`; the Version Lab dashboard snapshot of 2026-10-04 (Muse, measured on published npm artifacts; credibility 8/10 — it reads the published tarballs directly, but its probe code is not in this repository).
