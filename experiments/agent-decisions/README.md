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

Samples 001 and 002 run on rc.15 from this directory's `package.json`. Sample
003 and [`retest-rc19/`](retest-rc19/README.md) pin the published rc.19 in their
own `package.json`, so run `npm ci` in that directory.

Each sample directory `NNN-slug/` holds `decision.cav`, `events.jsonl`,
`decision.scenarios.json`, `explain.txt` (the checked-in output of `explain`),
`dependents.txt` where queried, and `NOTES.md` (step-3 pricing and findings).

## Samples

| Sample | Decision | Kind | Owner |
| --- | --- | --- | --- |
| [001](001-departure-reason-mechanism/NOTES.md) | #170's mechanism correction: a departed withdrawal reason strands a citation | retro-fit, written after the fact | Dogfooding round 3 thread |
| [002](002-departure-continuation/NOTES.md) | continuation of the departure PR; earlier private log unavailable | live continuation | continuation agent |
| [003](003-presentation-pass-deferral/NOTES.md) | the owner defers the game presentation pass to after 1.0 (03:21 UTC card, 2026-10-12) | retro-fit, written after the fact on rc.19 | Dogfooding round 3 thread |

## Findings

Numbered in order across samples. 002's PR adds its rows with the next free
numbers on main.

| Id | Sample | Class | What was needed | What happened | Spec line | What changes if fixed |
| --- | --- | --- | --- | --- | --- | --- |
| D001 | 001 | tooling | `explain` to show what a plain `commit` rests on | Without a `decisions` declaration, `explain` prints "none declared" and `dependents` lists it only under decision changes; `check` gives no warning | `kit/lib/explain.mjs` L121–122; Dependents 0.1 L19 | Authors following skill step 4 with a plain `commit` see the decision they made |
| D002 | 001 | tooling | "what changes if the correction is wrong?" for a withdrawal's reason | `dependents` on the reason omits the withdrawal it backs | Dependents 0.1 L34–35 | A disputed correction shows every withdrawal standing on it; the relation #170 pins becomes visible |
| D003 | 001 | tooling | to tell a retained caveat from one the grounds carry | `retaining ci_pending` is printed as "based on withdrawal_L93 (caveats: ci_pending)" and "based on evidence with ci_pending"; the carrying evidence appears nowhere | Explanations 0.2 L60–61; Dependents 0.1 L51–52 | Readers can find where a retained caveat came from; bears on 003 |
| D004 | 001 | practice gap (reviewed; filed as convention needed) | the refuted reading kept in the decision's lineage | Lineage is what rules read; a `withdrawn(E)` guard adds the reason, not `E`; only an artificial `observed(E)` guard adds `E` | Explanations 0.2 L60–63; Withdrawal 0.1 L90–93 | One agreed place to see that the decider weighed an objection; no runtime change implied |
| D005 | 001 | tooling (low) | `dependents` on a reading to list the claim it supported | Claims are outside the dependents report; `explain`'s Evidence line has the relation | Dependents 0.1 L19 | One command for "what did this support?" and "what rests on it?" |
| D006 | 003 | grammar gap | skill step 3's "record caveats you chose not to pursue as such, with the reason" | `retaining` takes names only; a reason on the caveat's declaration or on `retaining` does not load. The reason lands as the deferral's grounds, and a caveat not pursued looks like any other retained caveat | Permission 0.1 L27; Reactive 0.3 L140 | "What did we choose not to look at, and why?" is answerable from the record. The workaround goes in the authoring guide during the soak (owner, 2026-10-12 15:53 UTC); syntax is 1.1 at the earliest |
| D007 | 003 | tooling (spec question) | to tell a caveat the grounds carried from one the author retained | On rc.19 both print as `retaining:`; the commit record unions them | Reactive 0.3 L143–146; Dependents 0.1 L21–26, L98–100 | Readers see which caveats were a choice. Fixed in `explain`'s text during the soak (owner, 2026-10-12 15:53 UTC); the record itself is unchanged |
| D008 | 001 (rc.19 re-run) | runtime (spec question) | to commit a plain `commit` again after it was reopened | The second commit is fatal `unclassified` ("commitment close already exists"), so a host loses the session; the repository's departure sweep found it in `retest-rc19/d001-undeclared.cav`, which now refuses that event itself | Dispatch 0.1 L141–146 keeps unclassified errors fatal; `evaluation/decision_in_force` classified the series case | A history-dependent failure refuses the event instead of ending the session. No existing code's documented meaning fits, so the owner's "Refuse it" (2026-10-12 16:21 UTC) puts a new code in 1.1 (COMPATIBILITY.md L43–45); the authoring guide says how to avoid it meanwhile |
| D009 | reviewer study prep (rc.19, main `0b99573`) | tooling | to tell a revision's grounds caveats from those it holds only through its series history | A revision committed after a reopening holds its predecessor's reopening witness in `retained` but not in its grounds (Reactive 0.5 L127–134; Explanations 0.2 L60–66). rc.19 printed it under `retaining:`; main's D007 fix printed it beside the grounds as "based on check, scan (caveats: db_stale)" once a take-back had removed it from the scan | Explanations 0.2 L60–66 | The text shows grounds caveats beside the grounds or under `retaining:`, and the rest as "also carrying, through its lineage". Fixed with D007's text during the soak; the record is unchanged |
| D010 | reviewer study prep (rc.19) | tooling | to see a take-back in `explain` | After `unqualify`, the Evidence line still lists the caveat and the JSON report has no `taken_back`; only `replay`'s snapshot records it | Take-back 0.1 L99–102 | A reader of `explain` sees that a doubt was taken back. An additive report field, so 1.1 at the earliest (COMPATIBILITY.md "1.x minor") |
| D011 | reviewer study prep (rc.19) | tooling (low) | to find "observed at #N" in the Events list | `#N` is the session sequence, which a refused event does not advance, while the Events list numbers every line; after a refusal they differ | Dispatch 0.1 L149–152 | The Events list shows each accepted event's sequence. CLI prose, so it can change in any release; not taken during the soak |
| D012 | 1.1 trial program (main `dba0828`) | runtime | an `unqualify` to clear a doubt from values read after a guarded renewal | With `on report when observed(e) renew e`, the sequence report, flag, report, flag, unflag leaves `carries(occurrence(e, 1), flaky)` and `carries(occurrence(e, 2), flaky)` at 0 while `has_caveat(v, flaky)` stays 1: the guard copied the doubt into the new occurrence's record as kept | Take-back 0.1 L110–124 | Fixed in 1.1: a stored observation or predicate qualification keeps only what it did not take through a live relation (Take-back 0.1, "Kept caveats") |
| D013 | 1.1 trial program (main `dba0828`) | runtime | an `unqualify` to clear a flag from a value whose evidence was observed under a guard that carried a caveat of the same name | With `on setup qualify r with flaky`, `on setup when observed(r) set fz = 0` and `on report when fz == 0 reveal t`, the sequence setup, report, flag (`t`), unflag leaves `carries(t, flaky)` at 0 while `has_caveat(p, flaky)` for `p = qualified(1, t)` stays 1: the take-back tested the grounds by the lineage, which still held `r` | Take-back 0.1 L116–124 | Fixed in 1.1: grounds take the live test over their own evidence (Take-back 0.1 step 4) |
| D014 | 1.1 trial program (main `dba0828`) | runtime | a late `qualify` to reach every value built on the evidence | With `on see_e when observed(f) reveal e`, a value set to `qualified(1, e)` before `qualify f with flaky` gains `flaky` in its lineage, while one set after it from the same records does not: the observation's record listed `f` but kept the caveats it had when the guard ran | Late Qualification 0.1 step 2 | Fixed in 1.1: a `qualify` reaches stored observation and predicate qualifications that list its evidence (Late Qualification 0.1 step 2) |
| D015 | 1.1 randomized take-back check | spec | a late `qualify` to reach every value built on the evidence | A value set under `when committed(trust)` reads the decision's frozen basis, which lists `e`. Set before `qualify e with flaky`, it gains `flaky` (Late Qualification 0.1 step 2: its evidence includes `e`); set after it, from the same basis, it does not, because the basis keeps what it carried. 1.0.0 behaves the same | Late Qualification 0.1 steps 2–3 | Fixed: a later `qualify` reaches the value through the basis's evidence, whenever it was computed; the basis keeps what it recorded, and a take-back never reaches through it ([Caveat Sources 0.1](../../spec/caveat-caveat-sources-0.1.md)). Test: `d015_a_later_qualify_reaches_a_value_read_through_a_basis_whenever_it_was_set`; the randomized check asserts it |
| D016 | randomized departure check (main `c29b76f`) | runtime | departure to run on a current index of what each stored record references | In a windowed program: `see_e`, `see_f`, a commit guarded on `observed(e)`, a skipped reopening whose guard read `committed(d)`, then `withdraw e because f` under a guard reading `f`. The withdrawal's late `withdrawn` reached the reopening's stored record, which then listed `f`, but the withdrawal collector still counted only `e` for it; the debug consistency check failed at the end of the event, and a release build would run departure on the stale index. Introduced by D014's fix | [Departure 0.1](../../spec/caveat-departure-0.1.md) | Fixed in 1.1: each record a late `qualify` reaches is recounted before departure runs. Test: `a_late_qualification_reaching_a_stored_record_updates_the_collector` in `runtime/tests/departure.rs` |
| D017 | 1.1 randomized departure check (main) | spec | to know whether a reopened current revision pins its grounds | Lineage Compaction 0.1 ("What stays exact") and Departure 0.1 say the grounds of the revision *in force* pin; Dispatch 0.1's `decision_in_force` says a reopened revision is not in force. The runtime keeps the current revision's grounds exact whether or not it is open, so a retired reading held by nothing else stays. 1.0.0 behaves the same | Lineage Compaction 0.1 L63–L80; Departure 0.1 L27–L40 | Settled 2026-10-18 (owner: keep the runtime). Both specs now say the current revision, reopened or not; no runtime change. `d017_a_reopened_current_revision_keeps_its_grounds_pinned` asserts it |
| D018 | 1.1 randomized windowed take-back check (main `2b5eacc`) | runtime | a value read through a stored record after its evidence departed to keep the doubt it took through that evidence | D012 leaves a take-back caveat live, not kept, in a stored observation or predicate qualification while the evidence it came through has a live relation. When that evidence departs, its relation leaves with it, and departure kept the caveat on current values only. A value read afterwards through the record carried `flaky` with no live relation and nothing kept. Five events: `report`, `report`, `flag`, `report`, `report`, then a read of `qualified(1, e)` | Take-back 0.1 "Departure and collection" | Fixed in 1.1: every provenance that names a departing record keeps what its live relations gave, in `compact`. Tests: the D018 case in `runtime/tests/departure.rs`; `runtime/tests/take_back_windows.rs` |
| D019 | 1.1 randomized take-back check, with a guarded `examine` added (main `34960f1`) | runtime | an `unqualify` to clear a flag a value took through `examined(...)` | With `on scan when observed(e) examine stale cost 1` and `on look when examined(stale) set seen = 1`, the sequence report, flag, scan, unflag, look left `flaky` on `seen` though no occurrence carried it: `examine` stored its guard as the attention record with every take-back caveat frozen in as kept, and neither a take-back nor a late `qualify` reached that record. Present since 1.0.0 | Take-back 0.1, "Kept caveats" | Fixed in 1.1: the examine record keeps only what it did not take through a live relation, and a take-back or late `qualify` reaches it like any other guard record. Test: `d019_unflag_reaches_a_flag_an_examine_guard_read` |
| D020 | randomized take-back check (main `1a7a3fa`) | runtime | an `unqualify` to clear a doubt an unguarded `qualify` put on a stored record | With `on see_e when not observed(e) reveal e supports ok`, `on flag_e qualify e with flaky`, `on unflag_e unqualify e with flaky` and `on read set ev = qualified(1, e)`: see_e, see_e, flag_e, read, unflag_e left `flaky` on `ev`, and a read after it carried it too. The late `qualify` merged `flaky` into the stored predicate qualification frozen against its empty guard, so the record marked it kept. Introduced by D014's fix | Take-back 0.1, "Kept caveats" | Fixed in 1.0.3 and 1.1: the lineage merged into a record is frozen with the qualified evidence too. Test: `d020_a_doubt_raised_after_a_stored_guard_read_the_evidence_can_be_taken_back` in `runtime/tests/take_back.rs` |

### Review of 001's findings

The owner reproduced all five on the published rc.15 (wasm `29edda09…`) at
18:42 UTC on 2026-10-05; every receipt below was re-checked on main
`38d6193`. No finding is a card yet. Order when cards open: D002, then D001
and D003 as two kit PRs.

- **D001: confirmed, kit-only.** `kit/lib/explain.mjs` L61 builds Decisions
  from `snapshot.decision_series` alone. Confound: a program that binds
  `history_count(close_170)` without the declaration does not load ("close_170
  must name a declared reading or decision history"), so D001 bites only a
  program that never queries the history.
- **D002: confirmed, first in line for a card.** An additive Dependents 0.1
  field listing the withdrawals whose reason is the subject. It is the relation
  Departure 0.1 pins (#170 at `e655a87`, L37 and L59–60).
- **D003: confirmed and sharpened.** `retaining` accepts a declared caveat
  that nothing carries, by design (Borrowed Uncertainty 0.1 L9–10 retains
  `own_doubt`). So the fix is rendering only: the commit effect already reports
  `retained` (`runtime/src/reactive.rs` L1023; the JSON at Borrowed Uncertainty
  0.1 L30). Print "retaining: ci_pending" and drop "based on evidence with".
- **D004: split.** The runtime side is design (Explanations 0.2 L60–63). The
  finding is a practice gap in skill step 2, an amendment for the skill's next
  revision and not a card: runtime lineage is what the rule read, not
  everything that could have influenced the decider. Use the withdrawal guard
  only when the refutation actually gated the decision. In 001 it did not (the
  owner closed because L93 held), so the checked-in program without the guard
  is the truthful one.
- **D005: logged, low, no card.**

## Known, hit again

Design items from round 1 (agent-ledger items 1, 2 and 5) and round 2
(AI_AUTHORING's three "refuses to do for you" entries). Frequency is the
signal.

- Ledger item 1, identifiers known only at run time: 1 (001: #170's head
  `e655a87` lives in an evidence name and string).
- "A caveat carries no logic": 1 (001: `ci_pending` stays retained after CI
  goes green unless a rule says otherwise).
- "There is no fallback": 1 (003: "after 1.0" needs an explicit reopen rule).

## Earlier items re-tested

- D001 to D005 on the published rc.19 (2026-10-12): **all fixed**; 001 and
  002 scenarios still pass. Receipts in [retest-rc19](retest-rc19/README.md).

- Ledger item 4, withdrawing a wrong observation (`../agent-ledger/README.md`
  L100): **resolved on rc.15** by Withdrawal 0.1, for named evidence and for a
  reading. Receipts in [001's notes](001-departure-reason-mechanism/NOTES.md#ledger-item-4-re-tested).

## 2026-10-07 continuation

The owner authorized work on the departure handoff, plugin setup and the
dogfooding findings. D001, D002, D003 and D005 now have candidate kit fixes and
regressions in `kit/test/dogfooding.test.mjs`; D004 has a practice clarification
in all three skill copies. Publication and final verification are recorded in
`docs/releases/rc16-development.md`; a candidate fix here is not a publication.

## 2026-10-28 open PR recovery findings

The separate [PR recovery record](../../docs/reviews/2026-10-28-pr-recovery.md)
records D030 (unmerged graph-storage regression), D031 (failed replay process
accepted by the check), D032 (ledger patch-version display), and D033 (native
touch delivery racing the browser test clock). These are scoped local
observations and corrective candidates, not findings from the published-runtime
dogfooding round above or claims that all four affect published 1.4.1.
