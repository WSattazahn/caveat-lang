# T3 notes

## base
Read README.md, phases/base.md, scenarios/current.json. State is plain JSON (timers, per-mushroom consumed/tasted, ordered absorption log, stored decision); view is derived. Each dispatch runs on a deep copy and swaps in only on success, so rejected events leave no trace. Checked with `run.mjs check` (clean) and `run.mjs test` (14/14).
Decisions: unknown event types are rejected; basis = supportedBy at the moment of first commit; absorb of a tasted duskcap is rejected via canAbsorb.

## cr1
Added `grove` to the level list; nothing else changed. check clean, test all pass.

## cr2
Added the 'Too risky — taste first' rule after the tasted checks and before the belief-based labels (only unknown present mushrooms). Absorb rejection follows from canAbsorb false. check clean, 17/17.

## cr3
Replaced the absorption log with a general evidence log (absorb_ and taste_ entries, each with caveats). Tastes now feed belief and decision via one observe() path; a taste while glow timer is 0 carries tasted_in_dark. caveats on mushrooms/belief/decision = union over cited evidence (decision: basis only, reopenedBy not included, per S19). Ambiguity: 'not glowing' read as glow timer == 0 at the moment of tasting. Absorbing a duskcap that was tasted in the dark stays rejected (same canAbsorb as certain version). 19/19.

## cr4
Uncertain-state unknown mushrooms now cite contradictedBy only; caveats derive from because automatically. 20/20.

## cr5
Taste evidence carries an age advanced by tick; at age >= 60 s (plain float sum, no epsilon) it gains caveat taste_faded. Faded label wins over dark. Decision caveats are now stored at commit time (frozen) instead of derived. Uncertainty: fade threshold with non-binary-exact dt values may be off by float rounding. 22/22.

## cr6
Decision state gains an ordered history, appended at commit and at each reopen. Ambiguity: history entries carry no caveats (spec lists only change/because). 24/24.

## cr7
While reopened, a glowcap observation that makes the trailing run of glowcap evidence (since the last duskcap evidence) exactly 2 recommits: basis = that run in order, reopenedBy cleared, caveats re-frozen from the new basis, history entry appended. 26/26.

## cr8
Duskcap absorption while heavy adds 20 s capped at 30; otherwise sets 20. heavySeconds = ceil(timer) when heavy, else 0. Ceil on a float timer may round up a tiny residue for non-exact dt. 27/27.

## cr9
New witness event: same id/kind/consumed/taste-agreement checks as absorb but no canAbsorb gate; consumes the mushroom, records witness_<id> with caveat secondhand through the shared observe() path (commit/reopen/recover), timers untouched. 30/30.

## cr10
Mushrooms carry a life number and a since-consumed clock advanced by tick; at >= 45 s a consumed mushroom regrows (life+1, untasted) unless life == MAX_LIVES (declared 1024, exported). Evidence ids: life 1 unsuffixed, later lives _<n>. Old evidence untouched, old tastes keep fading. Not exercised: reaching the life capacity (no scenario; not tried by hand). 33/33.

## cr11
Mushrooms remember consumedBy (evidence id of the current life's consumption, cleared on regrowth). whyNot() computes absorb/taste reasons by the table; it mirrors canAbsorb/canTaste conditions. Tasted duskcap gives 'Known duskcap' whatever its dark/faded qualifier. 35/35.

## cr12
save() now returns a compact encoding (v1): timers, per-mushroom tuples, evidence as short codes like "t1.2dD@12.5", decision referencing evidence by log index; createPolicy(saved) decodes it (throws on malformed saves). Faded tastes drop their clock. Checked: 38/38, plus scratch_stress.jsonl (10 min of dt=0.03 with a taste and absorb/witness of every mushroom every 1.5 s, 6 resumes) via `run.mjs try`: every resume 'same view: true'. Size not printed by try; by count (~86 evidence codes of ~10 bytes, worst case ~112 plus history) it is well under 4096 bytes, but I did not measure it directly. Save size still grows with evidence over unbounded play beyond ten minutes.

## cr13
Restructured: the belief reads a 6-entry memory window; each mushroom keeps its own taste evidence (fading on its own copy of the clock) and the evidence that consumed it, so labels/why survive forgetting. First commit basis is now the single observation after which the belief is probably_safe. CR7 counts via a stored recovery list (glowcap ids since the last contradiction), independent of memory. reopenedBy and history capped at 6. Save format v2: decision lists use bare references (e.g. "a0.2") because they may cite forgotten evidence; every part is bounded (estimated well under 1 KB; not measured directly). Checked: 41/41, and scratch_stress.jsonl replay with 6 resumes all 'same view: true'. Uncertainty: recovery basis caveats are taken from the current (possibly faded) copies of the two observations, consistent with commit-time freezing.

## cr14
Added marked flag per mushroom; mark/unmark events with the listed rejections; canAbsorb masked by the mark (so absorb is rejected through the existing canAbsorb check), witness ignores marks; consuming or regrowth clears it. why.absorb gains 'Marked to avoid' between Known duskcap and Too risky. Save v3 adds the mark (v2 still decodes as unmarked). Extra payload fields on mark/unmark are ignored. 44/44.

## cr15
Added pit (appended to LEVEL so save indices stay stable) and an OUT_OF_REACH list. View masks canAbsorb/canTaste for it; why gives 'Out of reach' after 'Already eaten'; absorb rejected via canAbsorb, taste and mark rejected explicitly, unmark rejected as not marked; witness/regrowth unchanged. Decoder treats a missing mushroom entry (older save) as fresh. 47/47.

## cr16
Journal: entries stored as {type, evidence copies}; text is derived from the evidence id/kind at render time. Each holder of evidence (memory, mushroom taste/consumedBy, journal entries) keeps its own copy and all copies fade on the same clock, so caveats stay current even for evidence the belief forgot (checked with scratch_cr16.jsonl: a commit entry citing a forgotten, consumed taste shows taste_faded after 60 s; resumes 'same view: true'). Capped at 6, an event adding two entries can drop two. Save v4 adds the journal (bounded: 6 entries x <=2 codes). 50/50.

## Summary
All 17 phases (base, cr1-cr16) committed complete; every `test` run before each commit passed. References read: README.md, phases/*.md, scenarios/*.json (I diffed each phase's current.json against the previous phase's file to find new scenarios). Scratch files: scratch_stress.jsonl/.out, scratch_cr16.jsonl/.out, all in this directory and replayed only through `run.mjs try`. No departures from the rules that I know of. Open uncertainties: float accumulation for dt values that are not exact in binary (fade/regrow thresholds, heavySeconds ceil); I never measured the save size directly (estimated from the bounded structure); the MAX_LIVES capacity (1024) was never exercised.
