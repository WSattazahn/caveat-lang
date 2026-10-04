# CAVEAT 0.1.0-rc.12 development plan

Status: development open, unpublished. The owner's scope proposal was updated
2026-10-03 at 21:05 PT and handed over on 2026-10-04. It was written from the
rc.11 state of `main`, Version Lab round 7 on the published rc.11, the rc.11
release record and the MCP `2026-07-28` specification. Keep the "Progress" section factual, as
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
  - no new syntax except where PR 7 is accepted.
- PRs merge as merge commits, never squash.

## First process item: the attested publish path

rc.11 was published by hand, without provenance, and `publish-npm.yml` has
never run (`docs/releases/v0.1.0-rc.11.md` L3–5). **rc.12 ships through
`publish-npm.yml`, or it does not ship.** PR 8 describes that path.

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

   Its findings for rc.12 are F261–F268, and new findings start at F269.
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
   - F267 (medium, added 2026-10-04 from Glowcap round 7): a
     `qualify … after` delay can fire one time event late. It is due when the
     clock reading minus the reading stored at scheduling reaches the delay,
     and that binary64 difference can fall just short of the steps' own sum.
     The behaviour is the same on rc.10 and rc.11.
   - F268 (medium, added 2026-10-04 from Glowcap round 7): a numeric `proc`
     parameter turns its argument's lineage into grounds. `set slot =
     qualified(7, w)` gives grounds `[w]`, but passing the same value through
     `proc store(v)` gives `[w, x]`, where `x` was only `w`'s reveal guard.
     A commit through a `proc` freezes the wider grounds the same way. The
     arguments are evaluated for lineage (`runtime/src/reactive.rs` L3940 at `8e7805a`),
     and a grounds read of the parameter returns that lineage (L3512). The
     behaviour is the same on rc.10 and rc.11. The owner chose (card,
     2026-10-04) to fix it in rc.12; the fix PR records the details.

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
     Result (#113, merged as `4dd2b66`; `experiments/glowcap/RESULTS.md`,
     "Round 7: fresh authors, blind requests, rc.11"): TypeScript wins, and
     Caveat meets neither decision rule.
     - Caveat is smaller (median 367 code lines against 496) and changed a
       third fewer lines on the blind requests (186 against 282). Its authors
       were green on the first run in 2 of 4 blind phases against 4, broke a
       median 11 earlier scenarios against 0, and drifted more in their
       explanations (51 units against 28).
     - Rule A (adoption) is not met. It would be met if change cost were read
       on lines alone; the report reads a measure as better only when no
       number in its group is worse, as round 6 did, and says so.
     - Rule B (the stated claim) is not met: drift is higher and blind
       first-run correctness is worse.
     - Excluding C1 for the isolation incident changes no verdict: C4 in
       place of C1, C1 in place of C4, and all four authors give the same
       verdicts.
     - Four of the seven recorded predictions failed.
     - No claim that Caveat is better than TypeScript goes into the README,
       AGENTS, the site or other documents on the strength of this round.
     - The work list it leaves: a save that forgets what the program forgot,
       ordered grounds, timing to the tick (F267, documented by #114) and
       the hand-written glue.
   - The member-symbols design decisions are made and recorded in
     `spec/caveat-member-symbols-0.1.md` section 12 (#107):
     - brackets, `TEMPLATE[Q]`;
     - no reference in `define` or `bind` for 0.1;
     - advisory C005;
     - `for`-block templates only.
4. **MCP specification `2026-07-28`** (the official changelog at
   modelcontextprotocol.io, a primary source). In this revision:
   - the protocol is stateless, and the `initialize` /
     `notifications/initialized` handshake is removed;
   - every request carries `io.modelcontextprotocol/protocolVersion` and
     `clientCapabilities` in `_meta`;
   - `server/discover` is a server MUST, and on stdio it is the client's
     backward-compatibility probe;
   - every result carries `resultType`;
   - `ping` and the Logging, Roots and Sampling features are removed or
     deprecated;
   - cross-call state is "explicit, server-minted handles passed as ordinary
     tool arguments";
   - a twelve-month deprecation window applies.

   The bridge speaks only `2025-11-25` (`kit/lib/mcp.mjs` L10, L215–236).
   `docs/MCP_AUTHORING_DESIGN.md` L16–19 says the new profile "is not
   advertised".
5. **Repository's own open items.** The deferred adoption items stay
   deferred: public `whatif`, `abstain` and scenario string matchers
   (`docs/AGENT_ADOPTION_FOLLOWUP.md` L11–13). The Lean fragment
   (`proofs/lean/README.md`; 68 theorems in `laws.json`) proves:
   - the provenance union laws;
   - grounds ⊆ lineage through the core combinators;
   - citation acceptance;
   - the accepted/rejected/fatal session boundary.

   It does not model commitments, reopening, restore validation or
   full-field rollback, and its README says so.

## Accepted scope (proposed), in PR order

### PR 1 — Open the cycle

#106 and #107 merge first. Then comes the opening commit:
- this file, from the rc.11 template, with inputs 1–4 recorded;
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

### PR 5 — MCP: speak the `2026-07-28` revision, keep the old one through its deprecation window

This PR is kit-only and additive, with no language change. In
`kit/lib/mcp.mjs` the bridge:
- answers `server/discover` with its supported versions (`2026-07-28` and
  `2025-11-25`), capabilities and identity;
- accepts requests that carry `io.modelcontextprotocol/protocolVersion` and
  `clientCapabilities` in `_meta` without a prior `initialize`;
- returns `UnsupportedProtocolVersionError` for a version it does not speak;
- puts `resultType: "complete"` and `io.modelcontextprotocol/serverInfo` on
  every result;
- returns tools from `tools/list` in a deterministic order, with the
  required `ttlMs` and `cacheScope` fields;
- keeps `initialize` and `ping` for `2025-11-25` clients.

It updates `docs/MCP_AUTHORING_DESIGN.md` L16–19 and `kit/docs/MCP.md`. Tests:
- `kit/test/mcp.test.mjs` for both profiles;
- the existing official-client checks;
- a probe that a client calling `server/discover` first reaches
  `tools/call`.

The bridge's boundary does not change: no sessions, no file paths, one call
at a time and 10 seconds per call.

### PR 6 — Release ledger, as an extension of the agent ledger (dogfooding; non-contract)

`experiments/agent-ledger/` already models merge decisions on changing
grounds, built from the agent's own mistakes of 2026-09-24
(`experiments/agent-ledger/README.md` L1–14). PR 6 extends it with the release
gates rather than starting a new experiment:
- "rc.12 is publishable" is a claim;
- a PR's CI run is evidence, carrying a caveat when it ran on a stale base;
- the `npm-publish` approval is the `permitted by` grant that the publish
  commitment needs;
- `verify-publication` confirms or withdraws the "published and attested"
  evidence;
- a merge to `main` reopens readiness for anything measured before it.

Scenarios cover three cases:
- stale green;
- hand publish: the publish is permitted, but without provenance the
  attestation claim stays unsupported;
- withdrawal.

A small driver turns GitHub check runs, merges, the tag and the
verification artifact into events through `caveat serve`. It runs live for
rc.12's release, and its `explain --json` is attached to the release record.
It claims nothing beyond what it measures.

### PR 7 — Member symbols 0.1 (owner decision: implement in rc.12, or keep as draft)

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

**Status: kept as a draft for rc.12** (see "Owner decisions" below).

### PR 8 — Publish through the attested path

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

- PR 7 is out of rc.12 (owner, card answered 2026-10-04): member symbols
  stay a draft spec through rc.12. The implementation branch starts after
  rc.12 publishes, and §11.6 (a second program) stays its merge gate. The
  reasons given: the bar is the draft's own, set by #68 ("pending
  independent evidence from 2–3 additional real programs requiring
  cross-member naming"); rc.12 is coherent without a parser change, editor
  grammar and kit types; and this cycle's effort goes to finishing. The
  search for a second program looks in what already exists (the agent
  ledger and round 7's Caveat programs) rather than writing one to order;
  if none is simpler with `ev_[shown]`, that is a finding about the
  feature.

## Direction: rc.13 and after (recorded now, not scoped)

- **A live session over MCP.** The `2026-07-28` pattern, server-minted
  handles passed as tool arguments, is how a Caveat session could be exposed
  to an agent host:
  - `open` returns a handle;
  - `dispatch`, `dispatchView`, `explain`, `dependents`, `save`, `restore` and
    `close` take it;
  - each wraps the existing serve protocol (`spec/caveat-serve-0.1.md`).

  It needs its own profile (an "MCP sessions 0.1"). The bridge's boundary
  deliberately excludes persistent sessions and save handles
  (`docs/MCP_AUTHORING_DESIGN.md` L10–15). Its single-flight and 10-second
  limits were set for stateless authoring calls. Whether the boundary moves
  is the owner's decision. If it does, this is the adoption path for "the
  decision ledger inside your agent framework."
- **Grow the proven fragment toward the invariants.** The next theorems map
  onto `docs/CAVEAT_ESSENCE.md` L5–14:
  - a commitment's basis and grounds are unchanged by later qualification
    (Late Qualification §3);
  - reopening retains the earlier revision's caveats and records its cause;
  - a rejected step preserves *every* runtime field, not only the modeled
    ones (`proofs/lean/README.md`, "Outcome and session boundary").

  Each is a model extension plus a conformance case in
  `scripts/verify-lean-conformance.mjs`. None is rc.12 work unless capacity
  appears.
- **The register as a program.** The Version Lab's register (the spec
  sentences pinned or contradicted per version) could be written as a Caveat
  program. `dependents` would then answer which sentences rest on which
  probe, and a flip would reopen a pin with grounds. Round 7's output is the
  first dataset.
- **The claim.** `docs/WHY_CAVEAT.md` L14–16 already states that the
  benchmark "does not establish a limit on what TypeScript or its libraries
  can express." Keep it that way. After Glowcap round 7, the README's first
  screen should make the narrow claim the record supports, and nothing
  broader: decisions with frozen reasons that a refactor cannot silently
  drop.

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
- MCP session handles. They are direction, not rc.12 work (above).

## Order

1. PR 1.
2. PR 2.
3. PR 3.
4. PR 4.
5. PR 5.
6. PR 6, ready before the release so that it can run live.
7. PR 7 is not in rc.12 (kept as a draft).
8. PR 8, which is the release.

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
- Publication happens only through `publish-npm.yml` (PR 8). Record it in
  `docs/releases/v0.1.0-rc.12.md` and the `-npm-publication.json` file, built
  from the `npm-publication-verification` artifact.

## Progress

- Development opened 2026-10-04 at `b3d34e0`; `kit/package.json` names
  `0.1.0-rc.12`. The published preview remains rc.11.
- #106 (kit README no longer names the dist-tag a release uses; F265 in the
  repository copy) and #107 (member-symbols §12 decisions) merged before the
  opening commit.
- The scope recorded here is the owner's revision of 2026-10-03 at 21:05 PT.
  It adds the MCP `2026-07-28` input and PR 5, makes the release ledger an
  extension of the agent ledger (PR 6), renumbers member symbols to PR 7 and
  the publish to PR 8, and records the Direction section.
- PR 6 (#109): `experiments/agent-ledger/release.cav` records the release
  gates, with 8 scenarios covering stale green, hand publish and withdrawal.
  `release-ledger.mjs` drives it from facts through `caveat serve`. Replayed
  from rc.11's facts, it shows a hand publish whose attestation stays
  unsupported, and refuses to record provenance. The live run for rc.12's
  release has not happened yet.
- PR 7 (member symbols) is kept as a draft for rc.12 (owner decision,
  2026-10-04); implementation follows rc.12's publication, gated on §11.6.
- PR 1 (#108) merged 2026-10-04 as `81bb39a`.
- PR 4: `kit/docs/AGENT_START.md` and the agent-evidence README say that a
  `caveat serve` session lasts as long as its process, that each MCP tool call
  is a fresh subprocess with no session handle, and that one MCP connection
  accepts 4,096 request IDs. `docs/AI_AUTHORING.md` names the rc.11 outcome
  codes it did not list yet: `evaluation/bound_exceeded` for a clock that
  would become nonfinite, and `limit/scheduled_limit`. Merged as #112
  (`63e775e`).
- F267: the owner chose (card, 2026-10-04) to document the firing rule, with
  no runtime change. `spec/caveat-renewal-0.1.md` states that a scheduled
  qualification is due once the current clock reading minus the reading
  stored at scheduling, both binary64, reaches the delay, so it can apply an
  event later or earlier than exact arithmetic would. The regression case
  `f267_a_delay_is_due_when_the_difference_of_clock_readings_reaches_it`
  (`runtime/tests/renewal.rs`) pins the lab's numbers (scheduled at
  `2.800000000000001`, steps of `0.0625`, applied on the 961st), through
  dispatch and through a save and restore taken while the schedule waits, and
  the exact case. The elapsed spec states the same rule, and
  `docs/AI_AUTHORING.md` advises power-of-two steps for timing exact to the
  event. Release note: the renewal and elapsed specs now state when a
  scheduled qualification is due; no program's behaviour changes.
- F267 alternative considered and declined: counting time per schedule. It
  would change which event existing schedules apply on, including schedules
  in saved sessions (`scheduled_at` and `after` are save fields), and it still
  rounds for steps that are not powers of two. If a program needs timing exact
  to the event, integer tick time is the design to consider for rc.13 or
  later, as a spec-level change.
- F267's tolerance: the owner asked for "at most one clock event late, never
  early". That holds only while rounding stays small. Measured on the rule as
  implemented, a schedule can apply an event early relative to the exact sum of
  the supplied `dt` values. At large readings it can be several events off
  either way: scheduled at `1e11`, a delay of 1 with steps of `0.001` applies on
  the 993rd step. The specs state the rule and these bounds instead.
- PR 5 (#111): `caveat-lang mcp` speaks MCP `2026-07-28` and keeps
  `2025-11-25`. Release note: a client may call `server/discover`, then
  `tools/list` and `tools/call` with `io.modelcontextprotocol/protocolVersion`
  and `io.modelcontextprotocol/clientCapabilities` in `_meta` and no
  `initialize`; those results carry `resultType: "complete"` and
  `io.modelcontextprotocol/serverInfo`, and an unsupported version returns
  -32022. `2025-11-25` responses are unchanged, and so is the boundary (no
  sessions, no file paths, single-flight, 10 s). Where the plan's summary
  differs from the specification (the `io.modelcontextprotocol/`
  capabilities key, code -32022, identity in `_meta`), the specification is
  followed. No official `2026-07-28` client is on npm yet, so the new path is
  checked by the package gate's own probe.
- PR 2 (#110): restore refuses a caveat's attention that no examination of
  the loaded source can leave (findings 123, 188–191): `deferred` and
  `examining`; `examined` without a budget or on a caveat no `examine`
  reaches; and examined caveats whose least costs exceed what the budget
  spent. Release note: a save that marks a caveat examined without its
  program's examination and spending is now refused at restore; attention
  the program could have left still restores, so saves the runtime writes
  are unaffected. One deviation from check (a): an explicit `unexamined`
  entry, which claims no examination, is still accepted. Round 7's F263 and
  F264 concern qualifications, not attention, and add no fixture here.
- F267's documentation and regression merged as #114 (`8b0cc85`).
- Glowcap round 7 merged as #113 (`4dd2b66`); its result is recorded under
  input 3.
- F268 (owner's decision: fix in rc.12): a procedure parameter now supplies
  its argument's grounds, so `call store(qualified(7, w))` grounds `slot` on
  `[w]` as `set slot = qualified(7, w)` does, and the guard that only revealed
  `w` stays in lineage. The call's guard and its eagerly frozen arguments
  still take full lineage. Release note: values and commitments passed
  through numeric `proc` parameters may report narrower grounds, and
  `rests_on_withdrawn` no longer holds for a withdrawal of evidence that only
  gated an argument; lineage, outcomes and values are unchanged, and saves
  written by rc.11 restore with their wider grounds kept. Re-running the
  corpus before and after the fix (54 scenario files, 406 scenarios, 27,531
  events; Glowcap round 7's four Caveat finals, 50 scenarios each) changed no
  outcome, snapshot or grounds: no program in it passes a guarded reveal's
  evidence through a parameter. The Lean conformance gate passes (72 cases).
- PR 3: restore refuses a qualification record keyed by a name its record
  cannot belong to (an observation of unobserved, unrenewed evidence; an
  examination of an unexamined caveat; a reopening of a commitment nothing
  reopens), a predicate guard on a name of the wrong kind, and a withdrawal
  record whose event reaches no `withdraw` of that evidence for that reason
  (findings 86, 89, 96, 118–120). Release note: these edits are now refused
  at restore; records the program could have left still restore, and a save
  with a withdrawal removed together with its relation (round 7's F263) still
  restores. 17,280 genuine saves from 100 repository programs, driven with
  seeded events, restore as before. Two PR 2 fixtures that left an
  examination record behind a removed attention entry now expect a refusal.
- Caveatism: `caveatism/` holds the culture around Mr. Caveat (the Caveatist
  Archive v1.1 and Atlas v1.0 as written, an optional practice for agents and
  its skill, and the artwork). It is fiction outside the language's contracts,
  and no gate depends on it except its own canon. `caveatism/canon/` keeps the
  Archive as a program; it loads, checks clean and passes its 7 scenarios on
  published rc.11 and on this checkout (`npm run test:caveatism`, run in
  `runtime.yml`), and it joins the programs member symbols' compatibility
  check (§11.3) must load. The artwork is AI-generated, licensed CC BY-SA 4.0
  as `caveatism/LICENSE` scopes it, with provenance in
  `caveatism/character/PROVENANCE.md`. The repository README, `AGENTS.md`,
  `web/about.html` and the kit README link it; nothing from it enters the
  package except the kit README's image URL.
- PR 8 (the publish) opened: `docs/releases/v0.1.0-rc.12.md` holds the
  release notes, marked unpublished, and the README points to them. The kit
  README and guides already name `0.1.0-rc.12`. Publication goes through
  `publish-npm.yml` from the annotated tag `v0.1.0-rc.12` at this pull
  request's merge commit, merged after the last plan PR, dispatched with `latest`; `next` is added by hand
  afterwards. The tag, artifact, provenance and registry verification are
  recorded only from the `npm-publication-verification` artifact.
- Published 2026-10-04: tag `v0.1.0-rc.12` at `9e360af` (merge of #119), the
  tarball from [Runtime 37185306547](https://github.com/WSattazahn/caveat-lang/actions/runs/37185306547)
  (SHA256 `2a22cc7a711418ed32f508a23de6c19aed66f1bd52c2c61548a79183eab20d15`),
  published by [publish-npm.yml run 37211390333](https://github.com/WSattazahn/caveat-lang/actions/runs/37211390333)
  under `latest` with npm provenance, after the owner approved it in
  `npm-publish`. Its first attempt was refused by npm because the trusted
  publisher was not yet configured; nothing was published until the re-run.
  `verify-publication` passed, and the owner then moved `next` to rc.12. See
  the [release record](v0.1.0-rc.12.md).

Sources: MCP specification changelog for `2026-07-28`,
<https://modelcontextprotocol.io/specification/2026-07-28/changelog> (a
primary source). Everything else is the repository at `4500dec`.
