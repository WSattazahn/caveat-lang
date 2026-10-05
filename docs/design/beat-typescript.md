# Beating TypeScript on the record (2026-10-04)

The rules are already registered: Rule A (adoption) and Rule B (the stated claim), `experiments/glowcap/round7/PROTOCOL.md: L237–256`. A win means meeting them in a fresh blind round under the same protocol, with the same 10 percent margins, judged by the same "no number in the group worse" reading that cost us round 7's change cost. Nothing below re-scores round 7, drops a measure, or weakens the suite; the Archive Rule applies to the benchmark too. Receipts are `path: Lx–Ly` at `main @ fa0b125`.

## Where round 7 was lost, and why

Round 7's measurements (`RESULTS.md: L615–662`) and the per-run failure table (`L805–876`) decompose the loss into four causes. Three are already scheduled; the fourth is a kit change.

| Measure | Round 7 | Why | Fix |
| --- | --- | --- | --- |
| **1. Correctness**, blind first-run green | 2 of 4 vs 4 of 4 | (a) the session save passed the 4,096-byte bound at CR12/CR13 for C1, C2, C3; (b) `ungrounded_citation` on the first journal attempt for C2, C3; (c) the round's caveat-order registration defect hit every author; (d) C4's JSON wrapper took a save under the bound over it; (e) C4's renewed-evidence citation through a procedure parameter (F268) | (a) windows, rc.15 PR 4–6; (b) check-first `caveat test` and a reachability diagnostic, below; (c) journal caveat order defined by the spec and the registration compares as the spec says; (d) the adapter scaffold ships a correct wrapper; (e) fixed in rc.12 (#115) |
| **3. Change cost**, blind lines | 186 vs 282, Caveat | counted against Caveat because of regressions: median 11 vs 0 | the same causes as measure 1 produce the regressions (each failing run during a request counts), so the fixes above take the median toward 0; C1's 75 were an incident |
| **2. Size**, lines | 366 vs 496, Caveat | glue is 95–113 lines per program | the scaffold and ordered grounds cut the glue; already a win |
| **4. Explanation failures** | 0 vs 0, tie | | keep |
| **6. Mutation score** | 98.2% vs 100% | the only survivors are four M4 mutants that swap the order of two observations in a basis, equivalent because grounds are sets (`L785–791`) | ordered grounds (view 0.2) make the order observable, so the suite catches M4 and the score is 100% |
| **7. Explanation drift** | 51 vs 28 units | every `because` is a unit; save/restore adapter code counted as feeding explanations (`L795–803`) | windows remove the adapters' compression code; ordered grounds and a delta view remove the order-rebuilding code; `caveat types` removes hand-kept names |
| **5. Dispatch + view** | 148 µs vs 18 µs, both under the 1 ms gate | the view is re-serialized as JSON on every event | a delta view and a cheaper view path; target under 50 µs, still a gate not a score |
| Gzipped bytes a host ships | 615 KB vs 7 KB | the WASM bundles the game session, graphics, 3D world and map modules beside the reactive runtime (`runtime/src/lib.rs: L1–21`; `caveat_runtime_bg.wasm` is 2.15 MB raw, 602 KB gzipped in the rc.12 package) | a reactive-only build for hosts; target under 150 KB gzipped; not in either rule, but it is the number a host reads first |

Rule A needs two of measures 1–4 better with none more than 2× worse: size and change cost are the two, and change cost counts only with regressions at zero, so **Rule A is the regressions**. Rule B needs 100% mutation, lower drift, lower change cost, measures 1 and 4 no worse, dispatch under 1 ms: **Rule B is ordered grounds, drift, and 4 of 4 first-run**. Correctness at 4 of 4 is the hard one; it needs every cause in row 1 gone.

## The work, by cycle

### rc.15 (planned)
- Windows: retirement and departure (`rc15-plan.md` PR 4–6). Removes cause (a), the save-bound failures, and the adapters' compression code (drift).
- Integer clock (PR 2–3). Not a round 7 cause, but removes F267 from the fuzz.
- View 0.2 design note (PR 7): ordered grounds and the delta view, decided for rc.16.

### rc.16
- **View 0.2**: `commitment_grounds` and journal `caveats` in first-observed order (the journal's `because` already is, `spec/caveat-decision-journal-0.1.md: L31–32`), and a delta view. Closes M4 (mutation to 100%), removes the order glue (drift, size), and ends the caveat-order class of failures.
- **Check first.** `caveat test` runs `check` on the program before any scenario and reports its diagnostics with the run; an error-severity diagnostic stops the suite. A new diagnostic, C006 `citation-unreachable`: a `because` names evidence that no read in the binding's expression can reach on any path. Round 7's `ungrounded_citation` first-run failures (C2, C3) were caught at dispatch; the TypeScript authors got the equivalent from `tsc` before running. Same information, earlier.
- **Adapter scaffold.** `caveat init --host node|browser PROGRAM` writes the adapter the round 7 authors each wrote by hand: typed session from `caveat types`, the save wrapper that keeps the runtime's bytes as the measured size, resume from a save, event forwarding. The scaffold is the glue; the author writes policy.
- **Reactive-only build.** A `caveat-lang/reactive` entry whose WASM carries only the reactive runtime; the kit's loaders pick it unless a game module is requested. Measured in the package gate.
- **Dispatch path.** Profile `dispatch_view` on the round 7 programs; the first target is not serializing an unchanged view.

### Round 8: the same game, played to the registered rules, on rc.16
- Same protocol, same beat, same measures and both rules; fresh authors on both sides, blind change requests written before either side changes, same model family, the C1-style exclusion rule kept.
- Registration fixes stated up front: journal `caveats` compared as the spec defines them (ordered under view 0.2, so the comparator and the spec agree); the packet states the save's size behaviour under windows.
- Predictions pre-registered: regressions median 0; first-run green 4 of 4; mutation 100%; drift at or below TypeScript's; change cost lower; size lower; dispatch under 50 µs; shipped bytes under 150 KB. Each is a falsifiable number, and the record keeps the misses.
- A learnability probe before the round: a model from a family other than the authors', never shown the syntax, writes the program it expects for the beat. Diff it against a real program; each mismatch becomes a packet or guide item, and the diff goes in the record.
- If Rule A is met, the README's table gets the row and the claim gets exactly one sentence wider: "under these rules, on this beat, with fresh authors". If it isn't, the failure table says which cause survived.

### Round 9: Caveat's own ground
A second benchmark where provenance is the task, not a side effect: a long-running agent that acts on tool results, has some of them corrected or withdrawn mid-task, and must reopen exactly the decisions that rested on them, explain each, and resist a forged save. Measures: correctness of reopening under correction (mutants that drop a dependency, widen grounds, or skip a reopen); explanation accuracy after correction; forgery resistance (edited saves that restore live); lines and change cost when the correction rules change; dispatch under the gate. TypeScript implements provenance by hand, as the round 7 authors did for the CR7 basis and got wrong three times out of three (`L692–752`). Protocol written and pre-registered the way round 7's was, with the TypeScript side's rules drafted by an author who wants TypeScript to win. This is the round that tests the thing the language is for; round 8 tests whether it stopped losing at someone else's.

## What not to do
- No re-reading of round 7. Its verdicts stand as registered, including the line that says what reading would have changed them.
- No measure dropped, no margin moved, no author excluded after the fact, no suite weakened. A win under softer rules is not a win.
- No README claim before round 8's record exists, and then only the sentence the record supports.

## One docs fix now
`RESULTS.md: L684` carries a stray token ("98.4%ULEB") in the Rule B verdict; correct the typo in a docs PR, with the verdict unchanged.
