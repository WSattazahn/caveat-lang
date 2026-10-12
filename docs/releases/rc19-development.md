# CAVEAT 0.1.0-rc.19 development plan

Status: **approved by the owner on 2026-10-12 at 04:39 UTC**, with both scope
answers recorded under "Decisions" below: rc.19 is the 1.0 feature-freeze
candidate. Approving the plan starts the work below. It does
not approve any merge, tag, npm or registry publication, deployment or
outreach; each of those still waits for the owner's words at its own step.

## Decisions

| Decision | Answer | Notes that bind the implementation | Source |
| --- | --- | --- | --- |
| Q1. Freeze at rc.19 | **A. Freeze now** | rc.19 holds only items 1 to 5 under "What 1.0 still needs". The freeze moves up from 2026-11-16 to this plan's approval; the freeze review (S2) records it in Path to 1.0. rc.19's npm channel is Q3. 1.0.0 follows after two weeks of fixes only, on or before 2026-11-30 | Owner card "Freeze rc.19 now as the 1.0 freeze candidate?", "Freeze now", 2026-10-12 03:21:04 UTC |
| Q2. Game presentation pass | **A. After 1.0** | No game thread this cycle; the game stays as #184 left it. D4 and draft #179 wait until after 1.0.0 | Owner card "Run the game presentation pass now or after 1.0?", "After 1.0", 2026-10-12 03:21:06 UTC |
| Q3. npm channel for rc.19 | **Latest** (recommended; owner's answer pending) | rc.19 publishes with `dist_tag` `latest`, and `next` moves to it afterwards as for rc.17 and rc.18. rc.18 has the restore defect rc.19 fixes, so `latest` should not stay on it. Earlier releases stay published and unchanged, as E2 asks | Owner card on rc.19's npm channel, 2026-10-12 05:19 UTC; work proceeds on the recommendation until the owner answers |
| Merge of this plan | Merge commit, after the status and decisions above are recorded | | Owner, 2026-10-12 04:39 UTC |

[Path to 1.0](path-to-1.0.md) stays the plan of record. This file covers the
cycle between rc.18 and the 1.0.0 candidate, and decides nothing that file
decides except the freeze date, which the owner moved on Q1.

Inspected revision: `main` = `113bc9b47d0c3c1a83a6bcc41f8ec5f6b2b5a59c`
(merge of #213), 2026-10-12 UTC, and the repository's open issues and pull
requests at that time. Nothing here is a claim about any later state.

## Starting point

- rc.18 is published and closed out ([record](v0.1.0-rc.18.md)): take-back,
  ordered binding explanations, `explain`'s observing events, the memory trap
  message and the compatibility preparation. Every item of the
  [rc.18 plan](rc18-development.md) merged (#203 to #211).
- Every feature and documentation item in Path to 1.0 has shipped:

| Path to 1.0 | Blocker | Where it closed |
| --- | --- | --- |
| A1 collector spec | B1 | [#185](https://github.com/WSattazahn/caveat-lang/pull/185), [Withdrawal Collection 0.1](../../spec/caveat-withdrawal-collection-0.1.md) |
| A2 compatibility commitment | B2 | [#187](https://github.com/WSattazahn/caveat-lang/pull/187), prepared in [#208](https://github.com/WSattazahn/caveat-lang/pull/208); [COMPATIBILITY.md](../COMPATIBILITY.md) |
| A3 source digest in saves | B3 | [#189](https://github.com/WSattazahn/caveat-lang/pull/189), rc.17 |
| A4, A5 View 0.2 and its delta gate | B4 | [#190](https://github.com/WSattazahn/caveat-lang/pull/190), [#192](https://github.com/WSattazahn/caveat-lang/pull/192), rc.17; rerun over the corpus in [#205](https://github.com/WSattazahn/caveat-lang/pull/205) |
| A6 rc.17 candidate | — | [#194](https://github.com/WSattazahn/caveat-lang/pull/194), [#197](https://github.com/WSattazahn/caveat-lang/pull/197) |
| B1 starter | B5 | [#183](https://github.com/WSattazahn/caveat-lang/pull/183), `kit/lib/starter.mjs`, `kit/lib/starter-node.mjs` |
| B2 pilot package | — | [#188](https://github.com/WSattazahn/caveat-lang/pull/188), [ENGINEERING_READINESS.md](../pilot/ENGINEERING_READINESS.md) |
| B3 pilot run | B7 | Met; issues [#198](https://github.com/WSattazahn/caveat-lang/issues/198) to [#201](https://github.com/WSattazahn/caveat-lang/issues/201), triaged in rc.18 ([Path to 1.0](path-to-1.0.md) L81) |
| R1, R2 round 8 | B6 | Met; [#195](https://github.com/WSattazahn/caveat-lang/pull/195) ([Path to 1.0](path-to-1.0.md) L80) |
| C1 to C6 | — | [#182](https://github.com/WSattazahn/caveat-lang/pull/182), [#186](https://github.com/WSattazahn/caveat-lang/pull/186), [#191](https://github.com/WSattazahn/caveat-lang/pull/191), [#209](https://github.com/WSattazahn/caveat-lang/pull/209) |
| D2 Mr. Caveat likeness | — (not gating) | [#184](https://github.com/WSattazahn/caveat-lang/pull/184), approved by the owner on 2026-10-10 |

What is left is the release track, E1 to E3 ([Path to 1.0](path-to-1.0.md)
L237–239), and blocker B8. E1 asks for B1 to B7 closed or waived and the
surface frozen in COMPATIBILITY.md; the table above closes B1 to B7.

## What 1.0 still needs

| # | Item | Receipt | Kind | rc.19 |
| --- | --- | --- | --- | --- |
| 1 | The rc.18 save fixture. COMPATIBILITY.md says each release from rc.17 on adds its fixture once it is published; only `0.1.0-rc.17` exists | [COMPATIBILITY.md](../COMPATIBILITY.md) L170–171; `kit/test/fixtures/saves/` | A promise without its check | S1 |
| 2 | The freeze review: B1 to B7 recorded as closed, and the stable surface frozen | [Path to 1.0](path-to-1.0.md) L237 | Release gate E1 | S2 |
| 3 | The stabilization trial: "a representative installed integration runs a predefined trial: decisions, corrections, refusals, save/resume, archive persistence, explicit resource budgets". No trial is defined in the repository yet | [Path to 1.0](path-to-1.0.md) L238 | Release gate E2 | S3 |
| 4 | Issues #200 and #201 are still open although [#204](https://github.com/WSattazahn/caveat-lang/pull/204) answered both and rc.18 shipped it | [rc.18 record](v0.1.0-rc.18.md), "Documentation" | Housekeeping | S2 |
| 5 | The candidate on an explicit npm channel, earlier releases untouched (E2). E2 names `next`; rc.17 and rc.18 went to `latest`, and Q3 puts rc.19 there too | [Path to 1.0](path-to-1.0.md) L238 | Release gate E2 | S4 |

Left for the 1.0.0 candidate itself, not rc.19: COMPATIBILITY.md's status
line ("proposed for 1.0.0", L3) becomes the promise, and the kit README's
"Not yet" list drops "A stable release" (`kit/README.md` L471–472). Both are
true only of 1.0.0.

## Scope decisions as put to the owner

The options below are as the owner saw them on the cards. Each answer chose
the recommended option; see "Decisions" at the top.

**Q1. Make rc.19 the 1.0 freeze candidate.**

| Option | What happens | Cost |
| --- | --- | --- |
| **A. Freeze now (recommended)** | rc.19 holds only items 1 to 5. The freeze review (E1) happens with this plan, and stabilization (E2) runs on rc.19 as published (channel per Q3). 1.0.0 follows after two weeks of fixes only, on or before 2026-11-30 | About 4 days to rc.19's candidate |
| B. Keep the 2026-11-16 freeze | rc.19 stays a pre-freeze release. The only work left to fill it is the 1.1 additions (a caveat on an earlier occurrence, the no-value `using` form), which reverses the rc.18 decisions Q1 and Q3 | The same 4 days, plus whatever goes in; 1.0 stays at 2026-11-30 |

Why A: nothing in the 1.0 scope is left to build, and the owner's brief asks
that the finish line not move. An open window before a fixed freeze is where
unplanned scope arrives. A moves the freeze earlier, not the target later.

**Q2. The game's presentation pass (Path to 1.0, D4) and draft #179.**

| Option | What happens | Cost |
| --- | --- | --- |
| **A. After 1.0 (recommended)** | The game stays as #184 left it, with the likeness the owner approved | None now |
| B. Alongside rc.19 | One more thread, on `web/` and `game/` only, with the owner's visual approval stop; #179 brought up to main first (D1) | About 5 days of one more thread |

Why A: the game does not gate 1.0, the 1.0 promise does not cover it, and one
thread keeps the cycle bounded. Under B it would not affect the freeze,
because COMPATIBILITY.md lists `web/` and `game/` as not stable.

## Ordered PRs

Effort is working days for one thread, plus review; dates are estimates.

| PR | Content | Depends on | Effort | Proposed date |
| --- | --- | --- | --- | --- |
| S0 | This plan | — | 0.5 | 2026-10-12 |
| S1 | rc.18 save fixtures: `scripts/save-fixture.mjs --package=DIR` against the published `caveat-lang@0.1.0-rc.18` writes `kit/test/fixtures/saves/0.1.0-rc.18/`; `kit/test/save-fixtures.test.mjs` restores them. No fixture is edited by hand | S0 | 0.5 | 2026-10-13 |
| S2 | Freeze review (E1) in Path to 1.0: a dated baseline, B1 to B7 with their closing PRs as above, the frozen surface named as COMPATIBILITY.md at that revision, and the rule that from here until 1.0.0 only fixes, compatibility evidence and doc corrections merge. Close #200 and #201 as completed by #204 | S0 | 0.5 | 2026-10-13 |
| S3 | The stabilization trial (E2): one script, run against the packed tarball from a fresh install, driving the integration starter over `kit/examples/readiness` and the agent-evidence example. It covers decisions, a correction (withdrawal and take-back), each rejection origin, save and resume under the same source, a refused restore under edited source, archive drain and restart, and a recorded resource budget per step (event time, save and archive size). Budgets are recorded as measured on the candidate; they are host budgets, not new promises. Runs in CI on every PR from then on | S1 | 2 | 2026-10-15 |
| S4 | rc.19 candidate: release notes and the release procedure, with `dist_tag` `latest` (Q3). Tag and publication wait for the owner | S1–S3 | 1 + owner | 2026-10-16 |

After rc.19 is published, its own save fixture follows (the same S1 step), and
the trial runs against the published package. Defects found then are fixed on
main with regressions named after the trial step. Stabilization ends with the
1.0.0 candidate through the release procedure and the owner's approval (E3,
B8).

## Not in rc.19

Each of these stays out unless the owner decides otherwise:

- A caveat on an earlier occurrence, and the no-value `using` form
  ([#198](https://github.com/WSattazahn/caveat-lang/issues/198), which stays
  open for it): 1.1 additions, as decided for rc.18.
- Series windows: after 1.0.
- Member symbols (#107, [Member Symbols 0.1](../../spec/caveat-member-symbols-0.1.md)):
  a draft until a second real program exists.
- Narrowing the collector's conservative vetoes: post-1.0, and only if a real
  program hits a save-size problem
  ([Withdrawal Collection 0.1](../../spec/caveat-withdrawal-collection-0.1.md)
  L174).
- Glowcap round 9, and any rerun or re-scoring of round 8.
- Dogfooding round 3 samples 002 and 003, paused until the owner says.
- Mr. Caveat's presentation pass and #179: after 1.0 (Q2).
- W3C PROV export and OpenTelemetry span events: post-1.0 ideas.
- A weekly visibility protocol run: needs the owner's approval of a schedule.

## What this plan rests on

Inspected at `113bc9b`: the files cited above, the rc.18 record, the save
fixture directory, and the repository's open issues (#198, #200, #201) and
open pull requests (#179, #212). Not run: builds, tests or measurements.
Effort and dates are estimates.
