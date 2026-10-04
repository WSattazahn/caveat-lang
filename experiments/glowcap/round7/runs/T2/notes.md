# T2 notes

References read: README.md, phases/*.md, scenarios/current.json for each phase. Nothing outside this directory.
Checking: `node run.mjs test` and `node run.mjs check` (tsc strict) each phase.

## base
State object (timers, per-mushroom consumed/tasted, supportedBy/contradictedBy, decision); each dispatch applies to a structuredClone and swaps only on success, so rejected events leave no trace. Ambiguity: decision basis = supportedBy at the moment of commit (always the first glowcap absorption here). save() returns the state already (not yet required).

## cr1
Added `grove` to the level list; no other change.

## cr2
Moved per-mushroom rules into one `mushroomView` function that both the view and absorb validation use; added the "Too risky" row (contradictedBy >= 2) after the tasted rows and before the belief rows. Absorb is rejected via canAbsorb false.

## cr3
Absorb and taste both go through one `observe(evidenceId, kind, caveats)` that updates supportedBy/contradictedBy and the decision; per-evidence caveats stored in a map, and every view/belief/decision derives `caveats` as the union over its cited evidence. Uncertainties: a dark taste still makes the absorb-kind-contradicts-taste rejection apply (the host reports true kinds, so I kept it); `note` counts tastes as observations since N = supportedBy count; the decision basis stays frozen and only reports caveats of its own basis.

## cr4
Uncertain-state unknown mushrooms cite contradictedBy only; caveats already follow `because`.

## cr5
Each taste gets an age counter advanced by tick; at >= 60 s (with 1e-9 tolerance for float accumulation) the evidence gains `taste_faded` and the counter is dropped. Decision caveats are now snapshotted at commit time. Faded label wins over dark label. Ambiguity: a taste keeps aging after its mushroom is absorbed (evidence still cited by the belief), so the belief can gain `taste_faded` from a consumed mushroom.

## cr6
Decision keeps a `history` array appended at commit and at each reopen.

## cr7
Track glowcap observations since the latest contradiction (reset on any contradiction); while reopened, the second one recommits with basis = those two, reopenedBy cleared, caveats re-snapshotted, history appended. Ambiguity: the counter also runs before any commitment, but only matters while reopened; the never-trusted state still never commits once contradicted (S08 rule kept).

## cr8
Duskcap absorb while heavy adds 20 s capped at 30; `heavySeconds` = ceil(remaining) (with 1e-9 tolerance), 0 when not heavy. Timers now snap residues <= 1e-9 to 0 so float dt sums like 0.1 do not leave a phantom effect (a slight departure from strict "dt lowers by dt", affecting only sub-nanosecond residue).

## cr9
Added `witness` event: same validation as absorb minus canAbsorb, consumes the mushroom, observes `witness_<id>` with caveat `secondhand`, no timer change. Witnessing a tasted-duskcap mushroom with kind duskcap is allowed (only contradiction with the taste is rejected).

## cr10
Each mushroom has a life number and a since-consumed counter; ticks regrow it at >= 45 s as untasted, life+1, unless it is on its last life (declared capacity `MAX_LIVES` = 1024). Evidence ids use `_<life>` from life 2. Taste fades stay keyed by evidence id, so old tastes fade independently. Not exercised by scenarios: the capacity limit itself (reasoned only).

## cr11
Mushrooms record `consumedBy` (evidence id of the consuming absorb/witness for the current life); `whyNot` derives absorb/taste reasons from the same conditions as canAbsorb/canTaste, with caveats from the shared `caveatsOf`. A dark/faded duskcap taste still gives "Known duskcap".

## cr12
`save()` encodes state compactly (evidence ids as tokens like `a0`, `t3_3`; caveats as letters; witness `secondhand` implied); `createPolicy(saved)` decodes with basic shape validation (throws on a malformed save). Checked with `node run.mjs try stress.jsonl` (ten minutes of random absorb/taste/witness with 13 resumes: all "same view: true"); estimated save size from the final view of that run ~1.1 KB, well under 4096 (estimate made in Python from the printed view, not by running the policy outside the runner). Floats round-trip exactly through JSON.

## cr13
State keeps `memory` (last 6 accepted observations) with supportedBy/contradictedBy as its kind-split subsets; forgetting happens inside `observe` before the decision update. Never-committed trust commits with basis = the single observation after which the belief is probably_safe (base rule restated). CR7 counter keeps only the first two glowcaps since the last contradiction. reopenedBy and history are capped at 6. After every accepted event, caveats and fade clocks of evidence no longer citable (not remembered, not the current taste of a present mushroom) are pruned; `secondhand` is now derived from the witness id so consumed-by citations keep it. Save format v2 stores the ordered memory with +/- kinds; everything in it is bounded (lives capped at MAX_LIVES = 1024, so the life number is bounded too). Re-ran the stress replay: 13/13 resumes identical. Uncertainty: after MAX_LIVES (~12.8 h per mushroom) mushrooms stop regrowing, as CR10 allows.

## cr14
`mark`/`unmark` events toggle a per-mushroom boolean (rejected per table; unmark of a consumed mushroom is rejected as "not marked"). The view ANDs canAbsorb with !marked, so absorb is rejected through the same canAbsorb check; why.absorb gains "Marked to avoid" after "Known duskcap". `consume` clears the mark. Save format v3 adds the mark flag per mushroom.

## cr15
Added `pit` to the level and an OUT_OF_REACH list. The view forces canAbsorb/canTaste false for it; absorb and taste are both now rejected via the view (taste validation switched from "already tasted" to `!canTaste`, equivalent for the other four); mark rejects it. why gains "Out of reach" right after "Already eaten". Witness/regrow unchanged. Save format v4 (five mushroom rows).

## cr16
State keeps a bounded `journal` of structured entries (+/- observation, C commit, R reopen, each with evidence ids); text is rendered from the evidence id (prefix + mushroom name, life number dropped) and caveats are computed live from current evidence caveats. Entries are pushed inside `observe` (observation first) and in the commit/reopen paths, so only accepted absorb/taste/witness write. Pruning now also keeps caveats and fade clocks for evidence the journal cites, so journal entries see later fades. Save v5 adds the journal (bounded: 6 entries x <= 2 tokens). Stress replay re-run: 13/13 resumes identical.

## Summary
All 17 phases (base, cr1-cr16) committed complete; every phase passed all scenarios on its first measured `test` run, and `node run.mjs check` (tsc strict) was clean each time. Scratch file: stress.jsonl (random ten-minute replay with resumes, used via `run.mjs try`). No departures from the rules beyond the notes above (save-size estimate computed in Python from a printed view).
