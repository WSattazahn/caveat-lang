# Sample 001: #170's mechanism correction

**Written after the fact.** This is the round's only retro-fit sample: the
program was written at about 18:25 UTC on 2026-10-05 from the record of
17:49–18:03 UTC, not while deciding. It is a different kind of sample from
002 and 003.

Run against the published `caveat-lang@0.1.0-rc.15` (doctor: build
`3a88ba0`, wasm sha256 `29edda09…`), installed by `../package.json`. Skill
text: `skills/caveatist/SKILL.md` at main `38d6193`, sha256 `dda34513…`,
identical in its three copies.

## What happened

- 17:49:55 UTC. The reviewer (the owner, pasting from their other chat) argued
  that a standing withdrawal's reason must pin under departure, because
  Withdrawal 0.1 L93 says each withdrawal predicate carries the reason as its
  evidence. Their mechanism: `reopen merge because recheck` is grounded only
  because the guard supplied `recheck@j`.
- 17:53–17:54 UTC. The rc.15 development thread checked L93 and the predicates
  (L78, L84, L86) and found the mechanism wrong: `reopen … because` resolves a
  selector at dispatch and cites no stored name. The citation a departed
  reason would strand is a `set`'s or binding's `because`, refused as
  `evaluation/ungrounded_citation` (Dispatch 0.1 L114). #170 at `e655a87`
  says so (`spec/caveat-departure-0.1.md` L55–60).
- 18:03 UTC. The owner accepted the correction ("mine was wrong in the
  mechanism"), said the conclusion stands, and closed #170 from their side.

## The program

| Name | Kind | Source |
| --- | --- | --- |
| `reason_must_pin` | claim | the conclusion everyone acted on |
| `reopen_strands` | claim | the reviewer's mechanism |
| `withdrawal_L93` | evidence, supports `reason_must_pin` | the spec line itself |
| `mechanism_reopen` | evidence, supports `reopen_strands`, caveat `proposed` | the reviewer's reading |
| `thread_correction` | evidence, opposes `reopen_strands` | the thread's check |
| `owner_acceptance` | evidence, neutral reveal, the grant | the owner's 18:03 message |
| `ci_e655a87` | evidence, neutral reveal, caveat `ci_pending` | #170's head at 18:03 |
| `close_170` | decision, `using qualified(1, withdrawal_L93)`, `permitted by owner_acceptance`, `retaining ci_pending` | |

The conclusion and the mechanism are two claims because they came apart: the
correction refuted the mechanism and left the conclusion standing. The
reviewer's 17:49 message supplied both the L93 citation and the reading, but
the citation is named by its own source, the spec line, so withdrawing the
reading leaves the citation untouched.

## Step 3: what changes if each caveat is true

- `proposed` on `mechanism_reopen`: the reading is an interpretation, not a
  check. If it is wrong (it was), what changes is one sentence of #170's
  wording, not the close and not the pin. Cost: a wrong mechanism in a spec
  the runtime PR tests against. Priced material; resolved by the withdrawal.
- `ci_pending` on `ci_e655a87`: #170's CI was still running at 18:03. If it
  fails, the merge waits; the review close stands. Cost: a red merge, not a
  wrong decision. Retained at commit, unresolved in the program (nothing
  marks CI green; see "Known, hit again" in the README).
- The built-in `withdrawn` on `mechanism_reopen`: changes nothing that rests
  on the close, which is what S02 and S03 check.

## What held, against the plan's expectations

- **`rests_on_withdrawn(close_170)` is false and nothing reopens.** Held, in
  both orders: the record's (S01, withdrawal before the close) and the plan's
  (S02, withdrawal after it). S03 withdraws L93 itself as a counterfactual and
  the close reopens, so the quiet in S02 is the rule working, not a dead rule.
- **`explain` shows the withdrawal with its reason and the decision
  unchanged.** Held (`explain.txt`): `mechanism_reopen … caveats: proposed,
  withdrawn  withdrawn at #3 because thread_correction`, and `close_170@1 in
  force, based on withdrawal_L93`.
- **L93 holds live.** The binding `reading.withdrawn = withdrawn(mechanism_reopen)`
  is displayed `because thread_correction`: the predicate carried the reason,
  not the withdrawn reading. This is the read #170's pin protects.
- **`dependents … mechanism_reopen` lists only the claim it supported.** Did
  not hold as worded: it lists the withdrawal and nothing else, and claims are
  not part of a dependents report at all (D005).
- **Did a refuted interpretation have a natural home?** Yes: evidence named by
  its source, a `proposed` caveat, and `withdraw … because`. No convention
  beyond the markers-as-caveats one was needed. But "the reading stays in
  lineage" did not hold: it never entered the decision's lineage (D004).

## Findings

**D001 — tooling. An undeclared commitment is missing from `explain` and
`dependents`.**
- Needed: `explain` to show what `close_170` rests on (SKILL.md L35).
- Wrote `on owner_closes commit close_170 because enough using … permitted by
  … retaining ci_pending;` with no `decisions close_170` declaration. `validate`
  and `check` accepted it with no warning. `explain` printed `Decisions  none
  declared`; `dependents … withdrawal_L93` printed `Decisions  nothing` and
  only `Decision changes  #4 owner_closes: close_170 committed because
  withdrawal_L93`. Grounds, grant and retained caveat were not shown.
- Concerns: `kit/lib/explain.mjs` L121–122 lists `report.decisions` only;
  Dependents 0.1 L19.
- Workaround in this sample: `decisions close_170 limit 1;`.
- If fixed (shown, or `check` warns): every author who follows skill step 4
  with a plain `commit` sees the decision they made. Today the skill never
  says to declare it.

**D002 — tooling. `dependents` on a correction does not show the withdrawal it
is the reason for.**
- Needed: "what changes if the correction is wrong?" for `thread_correction`.
- `dependents decision.cav thread_correction events.jsonl` lists only the
  displayed `reading.withdrawn`, which happens to read the predicate. The
  withdrawal of `mechanism_reopen` because `thread_correction` is not listed;
  `Withdrawals` lists withdrawn occurrences matching the subject only.
- Concerns: Dependents 0.1 L34–35.
- If fixed: a reviewer disputing a correction sees every withdrawal standing
  on it. It is the same relation #170 pins (the reason a predicate reads), so
  departure's users would get the answer to "why is this reason still held?"
  that `Retired` gives for retired records (Dependents 0.1 L64–68).

**D003 — tooling. A retained caveat is printed as if the grounds evidence
carried it.**
- Needed: the decision retains `ci_pending`, which `ci_e655a87` carries, and
  that evidence is not in its grounds.
- `explain`: `based on withdrawal_L93 (caveats: ci_pending)`. `dependents …
  ci_pending`: `close_170@1 … based on evidence with ci_pending`, JSON
  `"basis": "grounds", "via": ["ci_pending"]`. No evidence in the grounds
  carries `ci_pending`, and `ci_e655a87` appears nowhere in the decision.
- Concerns: Explanations 0.2 L60–61 (grounds include retained caveats, so the
  JSON is per spec); Dependents 0.1 L51–52 (no invented qualification edge,
  which the JSON keeps and the human text's "evidence with" does not).
- If fixed: a reader can tell a caveat the decision chose to carry from one
  its evidence brought in, and find the evidence the retained caveat came
  from. This bears on 003, where `retaining` carries names only.

**D004 — convention needed. A reading that was weighed and refuted is not in
the decision's lineage.**
- Needed: the plan's "the reading stays in lineage" (skill step 2: lineage is
  everything that could have influenced the conclusion).
- The commit rule reads only `withdrawal_L93` and the grant, so lineage is
  `owner_acceptance, withdrawal_L93`. Two variants, not checked in:
  - guard `when withdrawn(mechanism_reopen)`: lineage gains
    `thread_correction`, the reason, not the reading (Withdrawal 0.1 L93);
  - guard `when observed(mechanism_reopen)`: lineage gains `mechanism_reopen`,
    but the guard says nothing true about why the owner closed.
- Concerns: Explanations 0.2 L60–63; Withdrawal 0.1 L90–93.
- Runtime lineage is what a rule read; skill step 2's lineage is what could
  have influenced the decider. They differ for anything weighed in the
  decider's head. If a convention is chosen (for example, the withdrawal
  guard, so lineage names the correction and the correction's withdrawal
  record names the reading): reviewers asking "did the decider see the
  objection?" get one place to look. No runtime change is implied.

**D005 — tooling, low. `dependents` does not report claim relations.**
- Needed: the plan's expectation that `dependents … mechanism_reopen` lists
  the claim it supported.
- It reports decisions, changes, values and displays only (Dependents 0.1
  L19); the relation `supports reopen_strands` is in `explain`'s Evidence
  line instead.
- If fixed: one command answers both "what did this support?" and "what rests
  on it?". Low: `explain` already answers the first.

## Ledger item 4, re-tested

Agent-ledger item 4 (`experiments/agent-ledger/README.md` L100, "Withdrawing a
wrong observation: not possible") is **resolved on rc.15** by Withdrawal 0.1.
- Named evidence: this sample. `mechanism_reopen` keeps its record and
  relation, gains `withdrawn`, and the withdrawal records its reason.
- A reading, the ledger's own case: `ledger-item4.cav`, a decision grounded on
  `latest(checks)`, then `withdraw latest(checks) because misread_note`.
  `ledger-item4.explain.txt`: `merge@1 … based on checks@1 / checks@1 has
  since been withdrawn at #2 because misread_note / #2 correct: reopened
  because misread_note`. The decision still cites the reading, now marked.

## Citations checked

All of the plan's citations used here hold on main `38d6193`. Three from the
owner's 18:03 message point a line or two off on #170 at `e655a87`: the
`ungrounded_citation` sentence is L59–60 (cited L60), the invariant L65–68
(cited L65–66), the outcome sweep L256–260 (cited L259). None changes a
reading. Archive.cav L33–37 declares `mythological`, `analogy` and
`unfinished`; `proposed` extends that convention, it is not declared there.

## Reproduce

```sh
cd experiments/agent-decisions && npm ci
cd 001-departure-reason-mechanism
npx --no-install caveat-lang check decision.cav
npx --no-install caveat-lang test decision.scenarios.json
npx --no-install caveat-lang explain decision.cav events.jsonl      # = explain.txt
npx --no-install caveat-lang dependents decision.cav NAME events.jsonl  # see dependents.txt
```
