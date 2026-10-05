# Agent decisions: dogfooding round 3

Threads working on this repository write their own live decisions as Caveat
programs while deciding, under the caveatist skill's steps 2–5
(`skills/caveatist/SKILL.md` L21–27) and its runtime section (L31–39). Each
program runs against the published `caveat-lang@0.1.0-rc.15`, not the working
tree, so the findings are about what users get.

Round 1 was [`../agent-ledger/`](../agent-ledger/README.md) (2026-09-24, "What
the language made hard", L72–128). Round 2 was the rc.14 dogfooding side PR
(`docs/releases/rc15-development.md` L206–214), which produced
`docs/AI_AUTHORING.md`'s "What the language refuses to do for you" and
`kit/examples/reviewer-decision/`.

This round changes nothing in `spec/`, `runtime/`, `kit/` or the skill. It is
not adoption or benchmark evidence: one model, under a skill that tells it to
use the language. A finding is a candidate; it becomes an owner card only after
the owner and the reviewer have gone over it.

## Setup

```sh
cd experiments/agent-decisions
npm ci                                   # caveat-lang@0.1.0-rc.15, from package-lock.json
npx --no-install caveat-lang doctor
```

Each sample directory `NNN-slug/` holds `decision.cav`, `events.jsonl`,
`decision.scenarios.json`, `explain.txt` (the checked-in output of `explain`),
`dependents.txt` where queried, and `NOTES.md` (step-3 pricing and findings).

## Samples

| Sample | Decision | Kind | Owner |
| --- | --- | --- | --- |
| [001](001-departure-reason-mechanism/NOTES.md) | #170's mechanism correction: a departed withdrawal reason strands a citation | retro-fit, written after the fact | Dogfooding round 3 thread |
| 002 | the departure PR's own decision | live | rc.15 development thread, its own PR |
| 003 | the next owner card that drops or defers something | live | Dogfooding round 3 thread |

## Findings

Numbered in order across samples. 002's PR adds its rows with the next free
numbers on main.

| Id | Sample | Class | What was needed | What happened | Spec line | What changes if fixed |
| --- | --- | --- | --- | --- | --- | --- |
| D001 | 001 | tooling | `explain` to show what a plain `commit` rests on | Without a `decisions` declaration, `explain` prints "none declared" and `dependents` lists it only under decision changes; `check` gives no warning | `kit/lib/explain.mjs` L121–122; Dependents 0.1 L19 | Authors following skill step 4 with a plain `commit` see the decision they made |
| D002 | 001 | tooling | "what changes if the correction is wrong?" for a withdrawal's reason | `dependents` on the reason omits the withdrawal it backs | Dependents 0.1 L34–35 | A disputed correction shows every withdrawal standing on it; the relation #170 pins becomes visible |
| D003 | 001 | tooling | to tell a retained caveat from one the grounds carry | `retaining ci_pending` is printed as "based on withdrawal_L93 (caveats: ci_pending)" and "based on evidence with ci_pending"; the carrying evidence appears nowhere | Explanations 0.2 L60–61; Dependents 0.1 L51–52 | Readers can find where a retained caveat came from; bears on 003 |
| D004 | 001 | convention needed | the refuted reading kept in the decision's lineage | Lineage is what rules read; a `withdrawn(E)` guard adds the reason, not `E`; only an artificial `observed(E)` guard adds `E` | Explanations 0.2 L60–63; Withdrawal 0.1 L90–93 | One agreed place to see that the decider weighed an objection; no runtime change implied |
| D005 | 001 | tooling (low) | `dependents` on a reading to list the claim it supported | Claims are outside the dependents report; `explain`'s Evidence line has the relation | Dependents 0.1 L19 | One command for "what did this support?" and "what rests on it?" |

## Known, hit again

Design items from round 1 (agent-ledger items 1, 2 and 5) and round 2
(AI_AUTHORING's three "refuses to do for you" entries). Frequency is the
signal.

- Ledger item 1, identifiers known only at run time: 1 (001: #170's head
  `e655a87` lives in an evidence name and string).
- "A caveat carries no logic": 1 (001: `ci_pending` stays retained after CI
  goes green unless a rule says otherwise).

## Earlier items re-tested

- Ledger item 4, withdrawing a wrong observation (`../agent-ledger/README.md`
  L100): **resolved on rc.15** by Withdrawal 0.1, for named evidence and for a
  reading. Receipts in [001's notes](001-departure-reason-mechanism/NOTES.md#ledger-item-4-re-tested).
