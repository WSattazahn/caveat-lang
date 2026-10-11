# CAVEAT 0.1.0-rc.18 development plan

Status: **approved by the owner on 2026-10-11 at 18:24 UTC**, with the four
scope answers recorded under "Decisions" below. This plan sorts what Glowcap
round 8 and the first agent pilot found into work for rc.18. Approving it
starts implementation threads. It does not approve any merge, tag, npm or
registry publication, deployment or outreach; each of those still waits for
the owner's words at its own step.

## Decisions

| Decision | Answer | Notes that bind the implementation | Source |
| --- | --- | --- | --- |
| Q1. Qualifications that can be taken back, and on an earlier occurrence | **B.** Taking back in rc.18; the earlier-occurrence form in 1.1 | The take-back aims at the **qualification record**, the `C qualifies E@k` edges a `qualify` added, not at an occurrence named in source. Glowcap's sequence is doubt (CR18), retaste (CR19, a `renew`), undoubt (CR20), and a new occurrence does not carry a late `qualify` ([Renewal 0.1](../../spec/caveat-renewal-0.1.md) L42–44), so a take-back that reached only the current occurrence would not solve CR20. If the spec cannot keep item 2's design question out, the 2026-10-24 gate moves Q1 to 1.1 rather than stretching. The take-back is a recorded event that marks the edge, never a deletion; frozen bases, the journal and archives never change. P3 says whether `proofs/lean/Caveat/Late.lean` gains a take-back step or leaves it unmodeled, as [the proofs README](../../proofs/lean/README.md) L251–255 does for renewal. The owner notes that B's 6 days is optimistic if the take-back's reach has to be settled in P3 | Owner, 2026-10-11 18:24 UTC |
| Q2. Observation order in binding explanations | **A.** Order them in View 0.2 before 1.0 | Caveats stay sorted (the 2026-10-10 decision stands). This extends [the design note](../design/view-0.2.md)'s grounds-only scope (L46–49) with the same first-observation mechanism on one more field. The delta gate reruns over the full discovered corpus; that is the standing rule, not optional. View 0.1 stays byte for byte | Owner, 2026-10-11 18:24 UTC |
| Q3. A decision with grounds and no value | **B.** Document the idiom now; the form in 1.1 | The guide says outright that a hold's value is not a verdict | Owner, 2026-10-11 18:24 UTC |
| Q4. `explain` names the event behind each grounds occurrence | **A.** In rc.18, kit only | An additive field on `caveat-explain/0.1`. In rc.17 a `renew` effect carries the new occurrence (`kit/lib/session.d.mts` L668–672) and a `reveal` effect names the evidence it observed (L636–642), and `explain` already replays each event with its payload; `kit/lib/explain.mjs` does not read effects yet, which is the whole job | Owner, 2026-10-11 18:24 UTC |
| Merge of this plan | Merge commit, after the status and decisions above are recorded | | Owner, 2026-10-11 18:24 UTC |

[Path to 1.0](path-to-1.0.md) stays the plan of record. This file covers the
cycle between rc.17 and the feature freeze it names (2026-11-16, an
estimate), and decides nothing that file decides.

Inspected revision: `main` = `a8470e120af46d649d945160859d08e4fc0c5eb6`
(merge of #195), 2026-10-11 UTC. Probes were run against the published
`caveat-lang@0.1.0-rc.17` from npm. Nothing here is a claim about any later
state.

## Starting point

- rc.17 is published and closed out
  ([record](v0.1.0-rc.17.md)): View 0.2 and its delta, the source digest in
  saves, the integration starter, the collector spec and the compatibility
  draft. `latest` and `next` name it, and so does the MCP Registry.
- Round 8 ran on rc.17 and is recorded in
  [the "Round 8" section of RESULTS.md](../../experiments/glowcap/RESULTS.md#round-8-fresh-authors-blind-requests-rc17).
  TypeScript won and neither rule was met. The section ends with an
  eight-item work list. Round 8 is not re-scored by anything below, and
  nothing in this cycle may claim that Caveat beats TypeScript.
- The first agent pilot (Path to 1.0, B3) filed four Caveat issues,
  [#198](https://github.com/WSattazahn/caveat-lang/issues/198) to
  [#201](https://github.com/WSattazahn/caveat-lang/issues/201). The pilot's
  subject is private; this plan and every PR from it describe only Caveat.
- So Path to 1.0's blockers B6 (round 8 run) and B7 (pilot run) are met as
  steps. Neither needed a favorable result.

## Already done

| Item | Where |
| --- | --- |
| Round 8 probe's four guide items: no records, lists or loops (use a `for` block); `caveat NAME consequence LEVEL`; no hand-built `because` lists or caveat unions; grounds and caveats are carried by values, not written by the author | `docs/AI_AUTHORING.md`, [#195](https://github.com/WSattazahn/caveat-lang/pull/195) (amendment 6 of the round 8 protocol) |
| Dogfooding round 3, D001, D002, D003 and D005: plain decisions in `explain`, the withdrawals a reason backs in `dependents`, "retaining" wording, support and opposition links | rc.16 (`b0d4c77`), regressions in `kit/test/dogfooding.test.mjs` |
| D004: skill step 2 says runtime lineage is what rules read, and a withdrawal guard is used only when the correction gated the decision | all three copies of the caveatist skill (`skills/`, `caveatism/skills/`, `plugins/caveat/skills/`), rc.16 |
| Withdrawal collector contract | [Withdrawal Collection 0.1](../../spec/caveat-withdrawal-collection-0.1.md), [#185](https://github.com/WSattazahn/caveat-lang/pull/185), merged 2026-10-10 |

Samples 002 and 003 of dogfooding round 3 stay paused until the owner says.

## How each item is sorted

The owner's 1.0 brief fixes the pre-1.0 feature scope at View 0.2 and the
starter. A benchmark result is not a blocker by itself, though a concrete
defect it finds may need repair. So:

- **Defect in a supported promise, or a documentation fix:** in rc.18.
- **New language or new output:** a scope decision for the owner, with a
  recommendation and its cost against the freeze. Where an item forks, the
  default is the brief's reading (no new scope), and the plan says so.

Two facts bear on every scope decision:

1. [COMPATIBILITY.md](../COMPATIBILITY.md) allows new syntax, optional fields
   and new outputs in a **1.x minor** release. Leaving a new construct out of
   1.0 does not close it off; it can arrive in 1.1 without breaking anyone.
2. A change to what the language means is its own spec PR with a corpus
   sweep, merged before the freeze. Anything not specced by its gate date
   below moves to 1.x.

## The round 8 work list

Numbers are the record's.

| # | Round 8 observed | Checked on rc.17 | Kind | rc.18 |
| --- | --- | --- | --- | --- |
| 1 | A qualification cannot be taken back (CR20) | `qualify` adds a permanent `CAVEAT qualifies EVIDENCE` edge ([Late Qualification 0.1](../../spec/caveat-late-qualification-0.1.md) "Meaning"). `withdraw` adds the `withdrawn` caveat; it does not remove another one ([Withdrawal 0.1](../../spec/caveat-withdrawal-0.1.md)) | New language | **Scope decision Q1** |
| 2 | `qualify` reaches only the current occurrence of renewable evidence (CR18) | As specified: a qualification names the current occurrence ([Renewal 0.1](../../spec/caveat-renewal-0.1.md) L35–45, L70) | New language | **Scope decision Q1** |
| 3 | Only commitment grounds are in observation order, and only in View 0.2; two programs built order from binding explanations, one never asked for 0.2 | View 0.2 leaves `binding_explanations` sorted and says "their order was not part of this decision" ([View 0.2](../../spec/caveat-view-0.2.md) L51–52) | Open point in agreed scope | **Scope decision Q2**; the doc fix below either way |
| 4 | A decision series cannot have a window, so recommits grow the save | Decided: series windows wait until after 1.0 | Deferred | Doc fix only (R8-4) |
| 5 | `caveat check` did not stop a `because observed(E)` the value never reads (M5) | **rc.17's check already warns on every one.** All 215 Caveat M5 mutants produced a C006 `citation-unreachable` warning, and the three unmutated programs produce none (`checkWarnings` in `experiments/glowcap/round8/results/mutation.json`; `caveat check` on `runs/C1–C3/impl/glowcap.cav`). The harness counts a static catch only when the program fails to load (`mutate.mjs` L68–70), so warnings were recorded but not counted. `caveat check --strict` exits 1 on a warning | Not a missing check; a reading to note | Post-hoc note and doc fix (R8-5) |
| 6 | An event parameter cannot name an evidence; adapters parse text ids into numbers | An event parameter is a number, a sort member or an `id` handle ([Identifiers 0.1](../../spec/caveat-identifiers-0.1.md)); naming a member's evidence through a parameter is the draft [Member Symbols 0.1](../../spec/caveat-member-symbols-0.1.md) (#107) | Draft feature | Assessment only (R8-6) |
| 7 | A host that never frees sessions traps with `unreachable` | `session.free()` exists (`kit/lib/session.d.mts` L223) and no kit guide mentions it; the trap does not say why | Defect: an error that does not name its cause | Fix (R8-7) |
| 8 | Countdowns from an accumulated binary64 clock are one tick late | Specified behavior (F267, [Elapsed 0.1](../../spec/caveat-elapsed-0.1.md) "Changes"); integer clocks are the remedy and the authoring guide describes them (L314–320) | Documented | Doc fix (R8-8) |

## The pilot's issues

| Issue | Checked on rc.17 | Kind | rc.18 |
| --- | --- | --- | --- |
| [#198](https://github.com/WSattazahn/caveat-lang/issues/198) "not ready" needs a value | Without `using`, a commit's basis is `null` and `explain` prints "based on nothing" and "committed because nothing": its grounds come only from `using` ([Reactive 0.3](../../spec/caveat-reactive-0.3.md) L138, L176; probe below) | New language | **Scope decision Q3**; idiom documented either way |
| [#199](https://github.com/WSattazahn/caveat-lang/issues/199) grounds don't show which item the evidence was about | Each accepted `renew` reports its occurrence in `effects` ([Renewal 0.1](../../spec/caveat-renewal-0.1.md) L57), and `explain` already replays the events with their payloads, so the link exists in data the kit holds | New output (tooling) | **Scope decision Q4** |
| [#200](https://github.com/WSattazahn/caveat-lang/issues/200) how `expect` matches arrays | The rule is in Scenarios 0.1; the kit guides don't state it | Doc fix | In rc.18 (P1) |
| [#201](https://github.com/WSattazahn/caveat-lang/issues/201) what a consequence level does | The guide says a caveat carries no logic; the kit reference lists levels without saying so | Doc fix | In rc.18 (P1), wording confirmed against the spec first |

Probe for #198, on published rc.17: the issue's program with
`on hold_release commit hold because enough;` beside its `using ci + build`
form. `explain` shows `hold@1 = null … based on nothing` for the first and
`hold2@1 = 1 … based on build_check, ci_run` for the second.

## Scope decisions as put to the owner

The options and costs below are as the owner saw them on the cards. The
answers, with their binding notes, are under "Decisions" at the top; each
chose the recommended option.

**Q1. Qualifications that can be taken back, and on an earlier occurrence
(items 1 and 2).** These two decided round 8's gap: CR18 and CR20 cost
Caveat 151 and 319 lines against TypeScript's 33 and 33, and caused all of
its regressions.

| Option | What happens | Cost against the freeze |
| --- | --- | --- |
| A. Both in rc.18 | One spec PR for both (they share the qualification record), then the runtime, save and View changes | About 10 days of one thread. The spec must merge by 2026-10-24 or both move to 1.1 |
| **B. Taking back in rc.18, earlier occurrence in 1.1 (recommended)** | A spec PR for a reversible qualification; the earlier-occurrence form follows as a 1.x addition | About 6 days. Same 2026-10-24 spec gate |
| C. Neither before 1.0 | Both arrive in 1.1 as additions; rc.18 documents the current limits | None |

Why B: taking back a caveat is the language's own claim ("withdraw rather
than erase"), it was the larger of the two costs (CR20), and it is
additive, so it changes no accepted program. The earlier-occurrence form
needs a way to name a past occurrence in source, which is a larger design
question and can wait for 1.1 without breaking anything. Whatever is chosen
keeps the decided rules: frozen bases, the journal and archives never
change; restore refuses only qualifications no source mechanism can produce,
and never authenticates history.

**Q2. Observation order in binding explanations (item 3).** View 0.2 orders
evidence only in commitment grounds.

| Option | What happens | Cost |
| --- | --- | --- |
| **A. Order them in View 0.2 before 1.0 (recommended)** | A View 0.2 spec PR orders each binding explanation's evidence by first observation, as grounds are; caveats stay sorted; View 0.1 is unchanged byte for byte; the delta gate reruns over the whole corpus | About 3 days |
| B. Leave them sorted | The guide tells hosts to build ordered lists from grounds only | None now. After 1.0, a change needs a new schema name beside 0.2 |

Why A: View 0.2 is in the agreed scope, has shipped only in a prerelease,
and becomes a promise at 1.0. Deciding its order now costs days; deciding it
after costs a new schema.

**Q3. A decision with grounds and no value (#198).**

| Option | What happens | Cost |
| --- | --- | --- |
| A. New form in rc.18 | A spec PR for a commit whose grounds come from what it cites, with no basis value | About 4 days; spec gate 2026-10-24 |
| **B. Document the idiom now, the form in 1.1 (recommended)** | The guide shows recording "not ready" with `using` over the evidence and says the value carries the grounds only | Half a day |

Why B: the idiom works today and loses nothing but a meaningless number,
the new form is additive in 1.1, and Q1 is the better use of the time
before the freeze.

**Q4. `explain` names the event behind each grounds occurrence (#199).**

| Option | What happens | Cost |
| --- | --- | --- |
| **A. In rc.18, kit only (recommended)** | `explain` prints, beside each occurrence in a decision's grounds, the event that observed it and its payload, and adds the same as an optional field in its JSON. No runtime, save or language change. If that turns out to need a runtime change, it moves to 1.1 | About 2 days |
| B. 1.1 | The guide says how to keep the link by hand | Half a day |

Why A: the data is already in the kit's hands, the change is additive to
`caveat-explain/0.1`, and it answers the question a release gate exists for.

## Ordered PRs

Effort is working days for one thread, plus review; dates are estimates.
Regression cases are named after their finding (`round8_item1_…`,
`issue_199_…`).

### Core thread

| PR | Content | Depends on | Effort | Proposed date |
| --- | --- | --- | --- | --- |
| P0 | This plan | — | 0.5 | 2026-10-11 |
| P2 | Q2 (A): View 0.2 spec and implementation for ordered binding explanations; delta gate over the whole discovered corpus; View 0.1 bytes unchanged | P0 | 3 | 2026-10-16 |
| P3 | Q1 (B) spec PR for a qualification that can be taken back, aimed at the qualification record (see "Decisions"): syntax, record, effects, save field, restore rule, View and delta, interaction with frozen bases, withdrawal, departure and the collector; corpus sweep showing no accepted program changes; whether `Late.lean` gains a take-back step | P0 | 2 | by 2026-10-24 (gate) |
| P4 | Q1 implementation: runtime, save, View 0.2 delta, kit types; tests named after round 8 item 1, including CR20's doubt, renew, take-back sequence; the departure gate and save sweep | P3 merged | 4 | 2026-10-31 |
| P5 | R8-7: find why an unfreed session traps, raise an error that names the cause, and document `free()` in the kit guides; regression `round8_item7_unfreed_sessions` | P0 | 1.5 | 2026-10-17 |

### Docs and kit thread (parallel)

| PR | Content | Effort | Proposed date |
| --- | --- | --- | --- |
| P1 | #200 and #201 in the kit reference; R8-4 (the series cap idiom and its save cost), R8-8 (countdowns: point to integer clocks from the kit reference), R8-5 (say that `caveat test` prints check warnings and `check --strict` fails on them), the #198 idiom (Q3 B), saying outright that a hold's value is not a verdict | 1 | 2026-10-15 |
| P6 | Q4 (A): `explain` text and an additive optional field in `caveat-explain/0.1` for the observing event of each grounds occurrence; tests on `examples/readiness` | 2 | 2026-10-21 |
| P7 | R8-5 post-hoc note in round 8's record: the C006 warning count for every M5 mutant, labelled post hoc like the other `fuzz-checks/`. It changes no figure or verdict | 0.5 | 2026-10-15 |
| P8 | R8-6 assessment: whether a round 8 program meets Member Symbols §11.6 (a second real program not written for it). Report only; #107 stays a draft unless the owner decides otherwise | 0.5 | 2026-10-17 |

### 1.0 plan items carried into this cycle

| PR | Content | Effort | Proposed date |
| --- | --- | --- | --- |
| P9 | Path to 1.0 baseline: B6 and B7 met, rc.17 published; link this plan | 0.5 | with P1 |
| P10 | COMPATIBILITY preparation named there: each spec's status line matches its stable list; each promise row names a check that runs in CI; a checked-in rc.17 save restored by the candidate, as the first per-release fixture | 2 | 2026-11-04 |
| P11 | rc.18 candidate: release notes and the release procedure. Tag and publication wait for the owner | 1 + owner | 2026-11-07 |

The owner notes that P4's estimate is optimistic if the take-back's reach
has to be settled in P3. If the P3 gate is missed, Q1 moves to 1.1, P11
ships without it, and the freeze date does not move.

## Not in rc.18

- Series windows (item 4): after 1.0, as decided.
- Member symbols (#107): draft until a second real program exists; P8 only
  checks round 8 against that bar.
- Making C006 an error at load: it would refuse programs that load today.
  The warning and `--strict` already exist.
- Round 9 (the agent-correction benchmark), and any rerun or re-scoring of
  round 8.
- Mr. Caveat's presentation pass (Path to 1.0, D4): its own review track.

## Answers

Q1 B, Q2 A, Q3 B, Q4 A (owner, 2026-10-11 18:24 UTC). None of them moves
the freeze date or changes the 1.x surface beyond additions that
COMPATIBILITY.md already allows.

## What this plan rests on

Inspected at `a8470e1`: the files cited above, round 8's record and results,
issues #198 to #201 and PR #185. Run on published rc.17 from npm:
`caveat check` on round 8's three Caveat programs and on C1's M5-S1 mutant,
and `explain` on #198's program in both forms. Read, not rerun:
`results/mutation.json`. Not run: builds, runtime tests or measurements.
Effort and dates are estimates.
