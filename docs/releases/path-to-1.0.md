# Path to 1.0

Status: **approved by the owner on 2026-10-10 at 16:28 UTC.** This is the plan of record for the
work between rc.16 and 1.0.0. Earlier plans keep their history; where they
disagree with this file about scope after rc.16, this file decides. Approving
the plan does not approve any merge, tag, npm or registry publication, Pages
deployment, outreach or recurring automation. Each of those still waits for
the owner's words at its own step.

Inspected revision: `main` = `ed210d802566b671fe4c9deacf6ec88ddeea179a`
(merge of #180), 2026-10-10 UTC. The baseline below was checked against that
revision, the npm registry and the MCP Registry on 2026-10-10. It is not a
claim about any later state.

## Destination

1.0.0 is an owner-approved release with a written compatibility commitment.
Removing the `-rc` suffix is not enough. The work toward it runs as five
workstreams with different acceptance rules:

| Workstream | What it delivers | Gates 1.0? |
| --- | --- | --- |
| A. Core and compatibility | Collector spec, compatibility commitment, View 0.2, the Node and browser starter | Yes |
| B. Evidence before freeze | Glowcap round 8 and the outside engineering pilot, both run on the first candidate carrying View 0.2 and the starter | Yes, as steps that must have happened. A favorable verdict or a customer is not required |
| C. Technical introduction and distribution | Clearer entry path, proof visibility, related-work note, registry and channel upkeep, the frozen visibility protocol | Docs that state the 1.0 promise must be accurate at release; everything else runs in parallel |
| D. Mr. Caveat in the game | A likeness built from the concept sheet, then presentation polish | No. Separate review track with its own visual approval |
| E. Release | Feature freeze, bounded stabilization, exact-artifact verification, owner release decision | Yes |

## Decisions already made

| Decision | Answer | Source |
| --- | --- | --- |
| Policy identity of saves (D6) | Keep the refusal: restore under edited source stays refused as today, with no override. Saves add a SHA-256 of the exact source, and `explain` shows it beside past decisions. Saves made before that show "not recorded" | Owner card "Keep refusal", 2026-10-10 16:40:09 UTC (replaces the 16:19 pick, which rested on a wrong baseline) |
| Who implements | Claude threads implement; the brief's "implementer plus independent reviewer" applies within that | Owner card, 2026-10-10 16:12 UTC |
| Pilot timing | The outside pilot runs on the starter candidate **before** feature freeze | Owner card, 2026-10-10 16:12:57 UTC |
| Positioning | "A decision ledger for agents" stays the headline. "An embeddable decision runtime with a small rules language" is the technical sentence under it | Owner card, 2026-10-10 16:13:03 UTC |
| Round 8 timing | Before feature freeze, on the first published rc carrying View 0.2 | Owner card, 2026-10-10 16:13:13 UTC |
| View 0.2 caveat order | Caveats stay sorted. Only evidence moves to first-observed order (`docs/design/view-0.2.md`). This supersedes `docs/design/beat-typescript.md`'s line putting journal caveats in first-observed order; that note keeps its history | Owner card, 2026-10-10 16:13:19 UTC |
| rc.16 scope | View 0.2 and series windows moved out of rc.16 | Owner, 2026-10-08, [rc.16 scope amendment](rc16-development.md#scope-amendment-for-release-preparation-2026-10-08) |
| View 0.2 delta gate | Full view at N plus delta equals full view at N+1, over the whole corpus as the save sweep runs, not a sample | Owner, 2026-10-05 14:45 UTC, [rc.16 plan](rc16-development.md) |
| Refusal codes | Published codes are permanent | Standing rule |
| Restore and authenticity (A3) | Restore never authenticates history; no document may say or imply it verifies saves or detects forgery | Standing rule |
| Release mechanics | Merge commits, never squash; one branch per thread; the procedure in [CONSOLIDATION_PLAN.md](../CONSOLIDATION_PLAN.md#candidate-and-release-gates) | Standing rules |

## Verified baseline

| Item | State on 2026-10-10 | Receipt |
| --- | --- | --- |
| Published package | `caveat-lang@0.1.0-rc.16`, published 2026-10-09 00:13:49 UTC with provenance | [rc.16 record](v0.1.0-rc.16.md), tag `v0.1.0-rc.16` on `3cc7b0f` |
| npm channels | `latest` = rc.16, **`next` = rc.15** | registry `dist-tags`, 2026-10-10 07:07 and 16:15 UTC |
| MCP Registry | rc.15 is `isLatest`; **no rc.16 entry** | `registry.modelcontextprotocol.io/v0/servers`, same times |
| GitHub prerelease | rc.16, 2026-10-09 01:12 UTC | release-prerelease run 37868589454 |
| Socket for rc.16 | Not recorded | rc.16 record has no Socket section |
| CI on main | Green: runtime 37868561602, format, Lean, deploy-pages 37872251547 | GitHub Actions |
| History management | Departure, archive provenance and the withdrawal collector shipped in rc.16 | `runtime/src/reactive_departure.rs`, `reactive_collector.rs`, `reactive_archive.rs`; [HISTORY.md](../../kit/docs/HISTORY.md) |
| Collector contract | Owner-accepted experimental overlay, written in `docs/design/`, not `spec/` | [withdrawal-collector-draft.md](../design/withdrawal-collector-draft.md); [departure spec](../../spec/caveat-departure-0.1.md) |
| View 0.2 | Design note only; no code | [view-0.2.md](../design/view-0.2.md) |
| Starter | None. Pieces exist: `kit/examples/agent-evidence/` (Python over `caveat serve`), `kit/lib/session.mjs`, `kit/lib/archive.mjs`, `caveat-lang init` | kit tree at the inspected revision |
| Save and source identity | A save already binds its exact source: `source_id` is FNV-1a64 of the source bytes plus their length, and restore under any other source text refuses ("it belongs to a different program"). The id is not cryptographic and appears in no report. (Corrected 2026-10-10; the first draft of this plan said the save bound only the program name.) | `runtime/src/reactive.rs` `source_identity` L5653; `runtime/src/reactive_save.rs` L673; [save spec](../../spec/caveat-save-0.1.md) L43 |
| Proofs | Lean models with `late_qualification_preserves_basis_and_grounds`; conformance gates in CI | [proofs/lean/README.md](../../proofs/lean/README.md) L238, `theorems.json` |
| Game | `web/mr-caveat.html` still draws the brown antenna robot in exploration (L92) and chase (L119) | file at the inspected revision |
| #179 | Open draft, 18 checks green, based on `9a2d1dd` (behind main) | PR #179 |
| Visibility | Grok V1 report captured 2026-10-09T04:33:13Z (third-party; not reproduced by us) | to be archived unchanged, PR C3 below |

### What exists and will not be rebuilt

- Departure, lineage compaction, the archive and the collector: rc.16 runtime.
- `caveat serve`, the MCP authoring bridge, `doctor`, `demo agent`, `init`.
- The agent-evidence example and its Python lifecycle tests: the pilot and the
  starter build on them.
- The View 0.2 design: the spec PR adopts it with the decisions above.
- The Lean models and their conformance gates: the docs PR links them; no new
  formalization.
- Glowcap's registered protocol and historical verdicts: round 8 follows them.
- The release procedure and verification scripts: 1.0 uses them unchanged
  except for the channel check in PR C4.

## The 1.0 compatibility commitment (what PR A2 writes)

PR A2 adds `docs/COMPATIBILITY.md`. This section fixes its shape so the owner
can approve the direction now.

### Stable surface

| Surface | Stable in 1.x | Notes |
| --- | --- | --- |
| Language syntax and observable semantics | Yes | `spec/caveat-0.1.md` plus the implemented profiles listed in `spec/`; the `-draft` documents (`caveat-0.2-draft.md` to `0.5-draft.md`) are excluded |
| Refusal origins and codes | Yes, permanent | The published set, e.g. `evaluation/expression`, `evaluation/requirement_failed`, `evaluation/unobserved_evidence`, `evaluation/not_committed`, `limit/scheduled_limit`, `evaluation/bound_exceeded`, `input/payload_invalid`, `input/bound_exceeded`; A2 lists all of them from `spec/` |
| CLI | Yes | `test`, `explain`, `dependents`, `validate`, `check`, `types`, `replay`, `serve`, `init`, `doctor`, `demo agent`, `mcp`, `--version`; documented exit statuses and `--json` outputs |
| Library entry points | Yes | `kit/package.json` exports: `./session`, `./node`, `./scenarios`, `./explain`, `./serve`, `./check`, `./types`; `./runtime/*` is an implementation path, not a stable API |
| `serve` protocol | Yes | Operations in [serve spec](../../spec/caveat-serve-0.1.md), including `drainArchive` and `undrained` |
| MCP tools | Yes, as an authoring bridge | `caveat_validate`, `caveat_check`, `caveat_test`, `caveat_explain`, `caveat_dependents`. No persistent session is promised |
| Schemas | Yes, by schema name | `caveat-reactive-view/0.1` and `/0.2`, `caveat-reactive-save/0.1`, `caveat-explain/0.1`, `caveat-dependents/0.1`, `caveat-check/0.1`, `caveat-interface/0.1`, `caveat-archive-provenance/0.1`, `caveat-dispatch/0.1`. Package 1.0 does not rename schemas to 1.0 |
| Saves and archives | Yes, within the promise below | |
| Supported environments | Node 20+ (`engines`), current Chromium, Firefox and WebKit through the WASM build | A2 lists what CI actually runs |
| Games, `caveat3d`, `game-session`, presentation profiles | **Excluded** | Experimental. Sharing the repository does not put them under the promise |
| Host conformance and `origin: "host"` | **Excluded, future component** | Stays in the package README's "Not yet" list. No supported 1.0 integration depends on it |

### Versioning policy

- **1.x patch:** fixes that make behavior match a documented promise.
- **1.x minor:** additions only: new schemas or schema versions, new
  operations, new optional fields, new refusal codes for inputs that were
  already refused or newly accepted syntax. Existing outputs for existing
  inputs stay byte-identical except where a fix restores a documented promise.
- **2.0:** removing or renaming anything stable, changing an accepted
  program's outcomes or a published refusal code, or making a valid save
  unrestorable.
- Deprecation: announced in release notes at least one minor release before
  removal in 2.0.
- npm version and schema versions move independently.

### Save, archive and source promise

- With the same source text and a 1.x runtime, a save written by an earlier
  1.x runtime restores, and later events give the outcomes the earlier runtime
  would have given. A save is not a promise across edited source.
- Archives drained from a 1.x session stay readable by `explain` and
  `dependents` in later 1.x runtimes.
- Restore checks that a save is possible for the source. It does not
  authenticate history (A3).
- **Policy identity of historical assessments (D6):** restore already refuses a
  save under any source text other than the one it was made under, and that
  stays. Saves add a SHA-256 of the exact source, and `explain` shows it beside
  past decisions; saves made before then show "not recorded" (D6 below). The id identifies source text; it
  does not authenticate the save or its history (A3).

### Limits that are not defects

Stable does not mean unbounded or universal. The commitment states these as
limits, each with its measured fixture:

- Window sizes do not bound saves. Required reason chains can grow (rc.16's
  3,000-cycle control saved 1.24 MB).
- The 65,536 held-record threshold and its admission timing.
- No authentication of evidence, saves or archives.
- No crash recovery beyond what a starter implements and tests.
- Synchronous bulk work can exceed a frame budget (rc.16: 55–60 ms bulk
  release). Hosts set and test their own budgets.

## Named blockers

Each blocker names a missing contract, a defect or missing evidence for a
gate. Owners are the threads proposed under "Threads after approval".

| ID | Blocker | Kind | Owner | Closure |
| --- | --- | --- | --- | --- |
| B1 | The withdrawal collector and its capacity amendment are core behavior in every windowed program, but their contract lives in `docs/design/`. Departure's invariant now reads "ordinary departure preserves outcomes" | Missing contract | Core | `spec/caveat-withdrawal-collection-0.1.md` merged with a corpus sweep, as F268 was; departure spec points to it |
| B2 | No written compatibility commitment or 1.x policy | Missing contract | Core | `docs/COMPATIBILITY.md` merged; each stable row has a named check |
| B3 | Saves bind exact source, but reports do not show which source a historical decision was made under | Missing contract (D6 decided) | Core | PR A3 merged with its tests |
| B4 | View 0.2 not implemented | Missing feature in agreed scope | Core | Spec and implementation merged; the delta gate passes over the full discovered corpus with negative controls |
| B5 | No starter; archive persistence after write failure or restart is undefined | Missing feature in agreed scope | Integration | Node and browser starters pass fresh-install lifecycle tests including the failure and restart cases they claim |
| B6 | Round 8 not run | Required step (owner) | Benchmark | Round 8 record committed, whatever its verdict |
| B7 | Pilot not run | Required step (owner) | Integration | One outside developer, team or agent has attempted the starter candidate on a real task of its own, with the owner's approved outreach or setup, and its defects are triaged; or the owner waives it at the freeze review |
| B8 | Final artifact not verified | Release gate | Release | The release procedure passes on the exact 1.0.0 candidate, followed by the owner's approval |

Not blockers: series windows, further collector or dispatch optimization,
search position, a paying customer, a favorable round 8 verdict, game polish.

## Ordered PRs

Effort is working days for one implementing thread, plus review. Dates are
proposals, not promises. Each PR is its own reviewable change.

### A. Core and compatibility (one thread, PRs in order)

| PR | Content | Depends on | Effort | Proposed date |
| --- | --- | --- | --- | --- |
| A0 | This plan | — | 0.5 | 2026-10-10 |
| A1 | Collector spec (B1): move the accepted contract into `spec/`, restate departure's invariant with the capacity difference, run the corpus sweep. No runtime change expected; a sweep difference stops the PR | A0 | 2 | 2026-10-13 |
| A2 | `docs/COMPATIBILITY.md` (B2), package README "Not yet" update, the D6 contract text | A1, D6 | 2 | 2026-10-15 |
| A3 | D6 implementation (B3): an additive SHA-256 source field in saves; `explain` (and `dependents`) show it beside past decisions; saves without it show "not recorded", never a digest computed at restore; the existing refusal unchanged; tests for new saves, older saves and edited source | A2 | 2 | 2026-10-17 |
| A4 | `spec/caveat-view-0.2.md`: ordered evidence, sorted caveats, `reopened`, delta schema; departures, qualifications, journal windows, sequence mismatch, event-local cues and effects; restore and resync. View 0.1 and its bytes unchanged | A1 | 2 | 2026-10-19 |
| A5 | View 0.2 implementation and delta gate (B4): the save sweep's discovered population, exclusions named and counted, fail on an empty or shrunken population; an independently written host-side applier; negative controls that corrupt deltas; accepted, no-op, rejected, restore, mismatch, explanation-only, qualification, departure and journal cases; dispatch, view build, apply time and bytes measured separately | A4 | 6 | 2026-10-28 |
| A6 | rc.17 candidate: A1–A5 and B1 starter (below), release notes, the release procedure. Tag and publish wait for the owner | A5, B2 | 1 + owner | 2026-11-02 |

### B. Starter and pilot (one thread)

| PR | Content | Depends on | Effort | Proposed date |
| --- | --- | --- | --- | --- |
| B1 | Node starter and browser starter from `caveat-lang init`: typed event forwarding, outcome handling that keeps "handled", "accepted" and "permitted by the current assessment" separate, save/resume with source identity, view resync, archive persistence. Node: append-only archive file with fsync before acknowledging a drain, pending items kept on write failure. Browser: IndexedDB with the same rule. Tests cover write failure, retry and restart; no crash-safety claim beyond those tests. Fresh-install tests from the packed tarball | A2 (contract), A5 for 0.2 view use | 5 | 2026-10-31 |
| B2 | Pilot package: the readiness example (revision A approved on test result T; unrelated decision on U; T found to use the wrong configuration; qualification and the authored reopening policy; U's decision unchanged; original grounds and policy identity shown; repeated through save, resume and archive), a one-page brief, acceptance criteria and a scoped offer whose price is marked as a hypothesis. Sanitized inputs only | B1 | 3 | 2026-11-04 |
| B3 | Pilot run (B7): one named developer, team or agent, only after the owner approves the target and the message or setup. An agent pilot is a fresh session given only the published package and its docs, never this repository or a briefing (owner card, 2026-10-11 04:26 UTC). Measured separately: install, correct integration, useful behavior, repeat use, willingness to pay (for an agent, its operator's) | B2, rc.17 published, owner approval | elapsed, not effort | by 2026-11-13 |

### Round 8 (benchmark thread)

| Step | Content | Depends on | Proposed date |
| --- | --- | --- | --- |
| R1 | Pre-register: same beat, registered rules, fresh blind authors, numeric predictions; learnability probe first (a model from another family, never shown the syntax) | — | 2026-10-30 |
| R2 | Run on published rc.17 and commit the record (B6). No README claim before it exists | rc.17 | 2026-11-11 |

### C. Technical introduction and distribution (one thread, parallel)

| PR | Content | Effort | Proposed date |
| --- | --- | --- | --- |
| C1 | rc.16 closeout record once the owner has moved `next`, published the registry entry and sent Socket screenshots; reword the departure spec's "forged departure report" to a source-possibility phrase (A3) | 0.5 | after owner steps |
| C2 | Front door: headline plus the technical sentence; problem, install, one runnable correction example, verification coverage (model theorems vs sampled conformance vs mutation controls vs runtime tests vs unproved boundaries), limits; link `proofs/lean/README.md`; grounds vs lineage, live qualification vs frozen grounds. Aligned across README, kit README, about page, AGENTS.md and metadata without duplicating long text | 2 | 2026-10-20 |
| C3 | Archive the Grok V1 report unchanged with provenance and capture time, plus annotations (tool order is not rank, P4/P5 weak software intent, summarized results only). Write the frozen visibility protocol (P1–P6, C1–C2) | 0.5 | 2026-10-14 |
| C4 | Release-procedure addition: npm `latest`/`next` and MCP Registry check at closeout, with intentional lag recorded. Review the registry description against the shipped tools | 1 | 2026-10-16 |
| C5 | Related-work note from primary sources: Doyle truth maintenance, Datalog provenance, Rego, Cedar, differential dataflow, Scallop. Differences we have not verified are marked so | 1 | 2026-10-24 |
| C6 | Fresh-agent discovery study design (frozen prompts, no Caveat name or URL). Running it needs separate approval | 1 | 2026-10-27 |

Weekly repeats of the visibility protocol need the owner's approval for a
schedule; none is set up by this plan.

### D. Mr. Caveat in the game (one thread, own review track)

| Step | Content | Effort | Proposed date |
| --- | --- | --- | --- |
| D1 | Bring #179 up to main and finish it, so layout work is not duplicated | 0.5 | 2026-10-13 |
| D2 | Baseline captures, then a likeness built from the concept sheet in both exploration and chase: hat with bells, ivory face, red nose, collar, mechanical joints, fortune slot, wind-up key; idle, movement, thinking and reaction. Technique suited to the current DOM/CSS game, with no engine switch. Deliverables: reference, original and replacement side by side, desktop and mobile build screenshots, a short motion capture, an accessible preview page. If a needed asset tool is unavailable, the PR says so | 5 | 2026-10-21 |
| D3 | **Stop for the owner's visual approval** | — | — |
| D4 | Presentation pass after approval: framing, NPC and prop consistency, placeholder art, dialogue and controls; case, choices and outcomes unchanged; all game browser checks; emulated mobile is labelled as such | 5 | after D3 |

### E. Freeze, stabilization and release

| Step | Content | Proposed date |
| --- | --- | --- |
| E1 | Feature freeze review: B1–B7 closed or explicitly waived by the owner; surface frozen in `COMPATIBILITY.md` | 2026-11-16 |
| E2 | Stabilization: fixes, compatibility evidence and doc corrections only. Candidates on explicit channels (`next`), earlier releases untouched. A representative installed integration runs a predefined trial: decisions, corrections, refusals, save/resume, archive persistence, explicit resource budgets | 2026-11-16 to 2026-11-30 |
| E3 | Exact 1.0.0 candidate through the release procedure; owner approves tag and publication (B8) | 2026-11-30 target |

The two-week stabilization window is a proposal. If a blocker delays the
release, this file names it and moves the date; scope is not added silently.

## Decided: D6, policy identity of historical assessments

The owner's 16:19 UTC card ("Record and guard") was posted on this plan's first
draft, which wrongly said a save bound only the program name. Restore already
refuses a save under different source text. On a corrected card the owner chose
**Keep refusal** (2026-10-10 16:40:09 UTC): the refusal stays exactly as it is,
with no override; saves add a SHA-256 of the exact source, shown by `explain`
beside past decisions; older saves show "not recorded". The digest identifies
the source text; no document may say it authenticates a save or detects
forgery (A3). Options not taken: a named-replacement override; no change.

## Coverage of the brief

| Brief section | Where it is in this plan | Status |
| --- | --- | --- |
| 1. First deliverable and authorization | This file (A0); limits stated at the top | Draft PR |
| 2. Real baseline | Verified baseline; what exists | Inspected at `ed210d8` |
| 3. Compatibility commitment | Commitment section; A2; B2, B3; D6 | Proposed |
| 4A. View 0.2 | A4, A5; caveat-order decision recorded | Proposed |
| 4B. Node/browser starter | B1 | Proposed |
| 5. Stabilization, gates, timing | E1–E3; blockers table | Proposed |
| 6. Technical introduction and the Grok review | C2, C5; positioning decision recorded | Proposed |
| 7. Mr. Caveat game | D1–D4 with the visual stop | Proposed |
| 8. Engineering pilot | B2, B3; pilot before freeze recorded | Proposed |
| 9. Discovery and distribution | C1, C3, C4, C6; rc.16 closeout owner steps | Proposed; closeout waits for the owner |
| 10. Deferred work, order, reporting | Deferred list; ordered PRs; this table | Proposed |

## Deferred, not required for 1.0

- Series windows, with the `kit/examples/boss-stance` fixture, scoped separately.
- Further collector, retained-capacity, archive-growth or dispatch
  optimization unless a supported workload fails a stated budget.
- Package-size work not needed for the supported contract.
- Glowcap round 9 (the agent-correction benchmark).
- Lending or clinical pilots; any hosted service or general agent platform.

## Threads after approval

One branch per thread, PRs in the order above:

1. **Core and compatibility:** A1 to A6.
2. **Starter and pilot:** B1 to B3.
3. **Docs and distribution:** C1 to C6.
4. **Mr. Caveat game:** D1 to D4.
5. **Round 8:** R1 and R2, started when A5 is close to merging.

## What this plan rests on

Inspected: the files and records cited above at `ed210d8`; the npm registry
and MCP Registry responses at 2026-10-10 07:07 and 16:15 UTC; GitHub Actions
and PR state. Not run for this plan: builds, tests or measurements. The rc.16
figures are from its own record. Effort and dates are estimates.
