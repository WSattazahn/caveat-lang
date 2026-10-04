# CAVEAT 0.1.0-rc.12 development plan

Status: development open, unpublished. The owner's scope proposal was updated
2026-10-03 at 20:40 PT and handed over on 2026-10-04. It was written from the
rc.11 state of `main`, Version Lab round 7 on the published rc.11, and the
rc.11 release record. Keep the "Progress" section factual, as
`rc11-development.md` does.

The owner wrote the line references against `main` at
`4500decab76aeb3aa1e055a17154be220f7d2c8b` (merge of PR #105). Recheck them
before editing, because they move.

## Starting point

- The cycle opens at `main` = `b3d34e0be9fc3b973733b16745651ed9bc5d2450`,
  the merge of PR #107. #106 (`0c818b8`) and #107 are both in.
- Tag `v0.1.0-rc.11` peels to `8e7805a57269c2084224df487fe186f0c6f6a4a9`
  (PR #103).
- The published npm rc.11 tarball has SHA256
  `13fd6e298731c46e024f10788e1f68834a1c02625fd74bb4715730b4a4887a8e`.
  `latest` and `next` both name rc.11. The GitHub pre-release is up.
- Only `kit/package.json` owns the candidate version. The cycle-opening
  commit bumps it to `0.1.0-rc.12`, and the bundled installation blocks are
  generated from it (`node scripts/kit-docs.mjs --write`).
- The ground rules are the same as rc.11:
  - one PR per item;
  - failing-then-passing tests first;
  - a spec `Changes` entry and a release-note line in the same PR;
  - no new syntax except where PR 6 is accepted.
- PRs merge as merge commits, never squash.

## First process item: the attested publish path

rc.11 was published by hand, without provenance, and `publish-npm.yml` has
never run (`docs/releases/v0.1.0-rc.11.md` L3–5). **rc.12 ships through
`publish-npm.yml`, or it does not ship.** PR 7 describes that path.

## Inputs

1. **rc.11's own record.** rc.11 was published by hand, without provenance,
   and `publish-npm.yml` has never run (`docs/releases/v0.1.0-rc.11.md`
   L3–5). The README copy on npm still says releases publish under `next`.
   The repository copy is fixed (#106), and the registry copy clears at the
   next publish.
2. **Version Lab round 7 on the published rc.11.** It found no regressions.
   It measured the npm tarball, whose SHA256 equals the CI candidate's:
   - 272 of 273 transplanted kit tests passed, against 254 of 273 on rc.10;
     the one test that doesn't apply needs the full runtime build;
   - every fixed finding was checked on the published bytes;
   - 27,313 corpus events in 371 scenarios across 36 programs were identical
     between rc.10 and rc.11, and every rc.10 save restores in rc.11.

   Its findings for rc.12 are F261–F266, and new findings start at F267.
   They are folded into the fixture lists of PR 2 and PR 3 when those reach
   the reviewer. Nothing below depends on them.
   - F261 (medium): `id_text` of a non-handle is still fatal `unclassified`.
     The identifiers spec still compares it to `1 / 0`, which refuses since
     rc.11.
   - F262 (medium): a negative `qualify … after` delay is fatal. A delay that
     divides by zero already refuses as `evaluation/expression`.
   - F263 (material): restore accepts a save with a qualification, or a
     withdrawal with its edge, removed once a later event has run.
   - F264 (material): restore accepts a pending scheduled qualification that
     has been removed or retimed.
   - F265 (low): the packaged README says releases publish under `next`.
     The repository copy is fixed by #106.
   - F266 (low): `REFERENCE.md`'s function list omits `atan2`.

   Round 7 recommends a design decision before any code for F263 and F264,
   as A3 had.

   The earlier Muse snapshot (250 findings, rc.3–rc.10) still identifies what
   rc.11 left open:
   - per-caveat attention states forged at restore (findings 123, 188–191);
   - qualification-table keys and the withdrawal record's `event`, both
     trusted at restore (96, 118–120, 86, 89).
3. **Already in motion, not scope.**
   - Glowcap blind round 7 against TypeScript on rc.11. Its protocol was
     pre-registered and approved at 20:17 PT, and it is running. Its result
     is recorded here whichever way it falls. No PR below waits on it or is
     written to anticipate it.
   - The member-symbols design decisions are made and recorded in
     `spec/caveat-member-symbols-0.1.md` section 12 (#107):
     - brackets, `TEMPLATE[Q]`;
     - no reference in `define` or `bind` for 0.1;
     - advisory C005;
     - `for`-block templates only.
4. **Repository's own open items.** The deferred adoption items stay
   deferred: public `whatif`, `abstain` and scenario string matchers
   (`docs/AGENT_ADOPTION_FOLLOWUP.md` L11–13).

## Accepted scope (proposed), in PR order

### PR 1 — Open the cycle

#106 and #107 merge first. Then comes the opening commit:
- this file, from the rc.11 template, with inputs 1–3 recorded;
- `kit/package.json` bumped to `0.1.0-rc.12`.

The rc.11 deviation is recorded as rc.12's first process item (above).

### PR 2 — Restore: a caveat's attention cannot be forged (findings 123, 188–191)

Today `graph.attention` entries restore with a kind check only
(`runtime/src/reactive_save.rs` L849–853). A save can mark any caveat
`examined` for free, `examined(c)` guards then pass, and decisions proceed as
if the examination happened. The budget check covers only totals (L1320) and
only the last event's `examine` effect (L1736–1745). The reactive runtime
refuses `examine` without a budget (`reactive.rs` L2386–2390) and only ever
sets `examined` (L4273–4295). `deferred` and `examining` belong to the game
session (`lib.rs` L36).

- **Checks.** All four are conservative: when uncertain, restore accepts.
  Genuine saves satisfy all four by construction.
  - (a) With no attention budget, every attention entry is refused.
  - (b) `deferred` and `examining` are refused in a reactive save.
  - (c) Each `examined` caveat must be examinable: some rule or procedure
    step reaches `Effect::Examine { caveat }`, found via
    `reached_effects(None)` (L1664).
  - (d) For each examined caveat, take the smallest declared cost among the
    rules that examine it. The sum of those costs must be at most
    `resources.spent`.
- **Fixtures.**
  - Genuine saves after 0, 1 and N examinations, on native, WASM and the
    installed package, each followed by an event and another save and
    restore.
  - Forged `examined` with `spent: 0`.
  - Forged attention on a caveat that no rule examines.
  - `deferred` in a reactive save.
  - The fuzz's insert-key operator, extended to attention values.
  - Whatever round 7 reported for this family.
- **Spec.** A sentence under restore validation and a `Changes` entry in
  `spec/caveat-save-0.1.md`. `RESTORE_TRUST_BOUNDARY.md` is updated the way
  #104 updated it.

### PR 3 — Restore: validate the keys the tables trust (96, 118–120, 86, 89)

This gap is inert today and cheap to close, and the PR has the same shape as
#99.
- **Table keys.** The `observation`, `examination` and `reopening` tables
  check only their values (`reactive_save.rs` L1281–1289). Their keys must
  be, respectively, observed evidence, caveats with examined attention, and
  commitments with a reopening record.
- **Predicate tables.** Their targets (L1290–) are checked by kind.
- **The withdrawal record's `event`.** It is checked only for being declared
  (L562–566). PR 3 requires that the event reaches a `withdraw` of that
  evidence, reusing the Withdraw arm of `last_event_can_make` (L1805–) with
  `reached_effects(Some(event))`.
- **Fixtures.** One genuine and one forged per table, plus round 7's for this
  family. The fuzz is unchanged.

### PR 4 — Docs: what agents hit in long sessions

The MCP bridge ends a connection after 4,096 request IDs (`kit/docs/MCP.md`
L65, `kit/lib/mcp.mjs` L14). This is documented and not a bug. The agent
quickstart (`kit/docs/AGENT_START.md`) never mentions it, nor that each tool
call is a fresh subprocess with no session handle (`MCP.md` L60–61). PR 4 adds
one paragraph to AGENT_START and one to the agent-evidence README. It also
folds the rc.11 refusal codes into `docs/AI_AUTHORING.md`, where that guide
lists the outcomes a program should expect.

### PR 5 — Release ledger experiment (dogfooding; non-contract)

`experiments/release-ledger/` holds a `release.cav` in which:
- "rc.12 is publishable" is a claim;
- each PR's CI run is evidence, carrying a caveat when it ran on a stale
  base;
- the `npm-publish` approval is the `permitted by` grant that the publish
  commitment needs;
- `verify-publication` confirms or withdraws the "published and attested"
  evidence;
- a merge to `main` reopens readiness for anything measured before it.

A small driver (`ledger.mjs`) turns GitHub check runs, merges, the tag and
the verification artifact into events through `caveat serve`. Scenarios
cover three cases:
- stale green;
- hand publish: the publish is permitted, but without provenance the
  attestation claim stays unsupported;
- withdrawal.

It runs live for rc.12's release, and its `explain --json` is attached to the
release record. It is patterned on `experiments/agent-ledger/`. It makes no
language change and claims nothing beyond what it measures.

### PR 6 — Member symbols 0.1 (owner decision: implement in rc.12, or keep as draft)

The design was settled in #107. Implementation is the cycle's largest item
and the first WASM-visible language feature since rc.10. It covers:
- the parser (brackets);
- the load-time checks (§5);
- dispatch resolution (§4);
- the VS Code grammar;
- kit types.

It must also meet the §11 bar:
- byte-identical Before the Rain snapshots, views, saves and journals in both
  forms, on its 12 scenarios and on the 3,000-history differential;
- cost at or below the hand-written form;
- every repository program loads byte for byte as before;
- each §5 error has a test with its exact text;
- each reachable refusal is identical to the one for the written name;
- a second program (§11.6).

Before PR 1, the owner decides whether §11.6 stays a merge gate or becomes a
release-note caveat. That decides whether member symbols ship in rc.12.

**Status: pending the owner's decision** (see "Owner decisions" below).

### PR 7 — Publish through the attested path

The rc.12 release is the first run of `publish-npm.yml`
(`docs/CONSOLIDATION_PLAN.md`, "npm publication from rc.11 on"):
1. The candidate job runs from the tag.
2. The publish is approved in the `npm-publish` environment.
3. `verify-publication` must pass, with `dist.attestations` present.

Dispatch with `latest`, then add `next` by hand afterwards. Provenance is
recorded in the release record only from the verification artifact. The
one-time setup of the npm trusted publisher and the `npm-publish` environment
must be done before the tag.

## Owner decisions

- Needed before PR 1: whether PR 6 is in or out of rc.12, and whether §11.6
  (a second program) is a merge gate. **Pending.**

## Explicitly out of scope for rc.12

- `abstain`, public `whatif` and scenario string matchers. They remain
  deferred adoption items, with no new evidence
  (`AGENT_ADOPTION_FOLLOWUP.md` L11–13).
- Any dispatch reclassification beyond rc.11's catalog. The still-fatal list
  ("a type error in an expression") stands.
- Authenticated host persistence and signed saves. These remain a host task,
  unchanged.
- Glowcap round 7. It is already running; its result is recorded here, not
  scoped.

## Order

1. PR 1.
2. PR 2.
3. PR 3.
4. PR 4.
5. PR 5, ready before the release so that it can run live.
6. PR 6, if accepted.
7. PR 7, which is the release.

Round 7's Glowcap result and the Version Lab's rc.12 findings land in this
record as they arrive.

## Acceptance and verification

The rc.11 rules carry over:
- Pin reproductions before changing behavior: the exact package, source,
  events and actual outcomes.
- Failing-then-passing tests for every restore check. They exercise native
  full and reactive-only Rust, built WASM, the kit, the CLI and serve, and
  save and restore.
- Run on the final review commit:
  - the gate list in `docs/CONSOLIDATION_PLAN.md`;
  - `npm run test:kit`, `npm run test:kit-package`,
    `npm run audit:kit-security`, `npm run check:kit-docs` and
    `npm run check:notices`;
  - Runtime CI and the Pages gate on the final `main`;
  - the existing Lean gates, unchanged (no new proofs are claimed).
- Identify the exact tested revision, and report failures and checks not run
  explicitly.
- Publication happens only through `publish-npm.yml` (PR 7). Record it in
  `docs/releases/v0.1.0-rc.12.md` and the `-npm-publication.json` file, built
  from the `npm-publication-verification` artifact.

## Progress

- Development opened 2026-10-04 at `b3d34e0`; `kit/package.json` names
  `0.1.0-rc.12`. The published preview remains rc.11.
- #106 (kit README no longer names the dist-tag a release uses; F265 in the
  repository copy) and #107 (member-symbols §12 decisions) merged before the
  opening commit.
- PR 6 is pending the owner's decision.
