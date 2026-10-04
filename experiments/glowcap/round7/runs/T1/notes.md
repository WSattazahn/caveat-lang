# Notes (T1)

## base
Read README.md, phases/base.md, scenarios/current.json. State is a plain JSON object; each dispatch applies to a clone and swaps only on success (atomic rejection). Belief/mushroom views are derived from the ordered absorption list; decision is stored (frozen basis). Checked with `node run.mjs test` (14/14) and `node run.mjs check` (clean). Ambiguity: commit happens when belief transitions to probably_safe while decision is none; basis = supportedBy at that moment (always one absorption in base).

## cr1
Added `grove` to the level list. 15/15, tsc clean.

## cr2
Added a 'Too risky — taste first' row after the tasted rows and before the belief rows (unknown = present and untasted), triggered by contradictedBy >= 2. Absorb rejection follows from canAbsorb false. 17/17, tsc clean.

## cr3
Unified absorptions and tastes into one ordered evidence list ({id, kind, caveats}); belief and decision derive from it. A taste while glow timer is 0 records caveat tasted_in_dark (judged before the taste). caveats on mushroom/belief/decision = union of caveats of cited evidence (because / support∪contradiction / frozen basis). Ambiguity: a mushroom tasted then absorbed contributes both taste_ and absorb_ evidence (S11b confirms). 19/19, tsc clean.

## cr4
Uncertain-state unknown mushrooms now cite contradictedBy only; caveats follow automatically. 20/20, tsc clean.

## cr5
Taste evidence carries an age advanced by tick; at age >= 60 s (exact comparison, like the timers; float drift with dt such as 0.1 could shift the boundary by one tick) the evidence gains caveat taste_faded. Faded label wins over dark label. Decision caveats are now snapshotted at commit (the basis caveats then) and never updated. 22/22, tsc clean.

## cr6
decision.history appended on commit and on each reopen, stored in state. 24/24, tsc clean.

## cr7
State tracks glowcap observations since the most recent contradiction (reset by any duskcap evidence). While reopened, the second one recommits with basis = those two, reopenedBy reset to [], caveats snapshotted from the new basis, history appended. Ambiguity: counting includes observations made before the reopen only if after the latest contradiction, which is always the reopening one, so effectively counted from the reopen. 26/26, tsc clean.

## cr8
Duskcap absorption while heavy adds 20 s capped at 30; heavySeconds = ceil(timer) when heavy else 0. 27/27, tsc clean.

## cr9
New witness event: validates id/kind/consumed/taste agreement (not canAbsorb), consumes, records witness_<id> with caveat secondhand through the same record() path (commit/reopen/recover). 30/30, tsc clean.

## cr10
Replaced consumed/tasted with per-spot slots {life, consumed, sinceConsumed, taste}. Evidence ids get _<life> from life 2. Regrow at >= 45 s of ticks since consumption; capacity declared as exported LIVES_PER_MUSHROOM = 64 (last life stays consumed; events rejected as consumed). Checked with test (33/33), tsc, and a scratch replay (scratch-lives.jsonl) of 65 witness+regrow cycles: 64 accepted, 65th rejected. Old tastes keep fading since fade age lives on the evidence.

## cr11
Slots record consumedBy (evidence id of the consuming absorb/witness for the current life). mushroom.why computed per the table (first match), caveats via caveatsOf. A tick reset on regrowth clears consumedBy and taste. 35/35, tsc clean (one tsc type error fixed before commit).

## cr12
save() now returns a compact encoding (impl/save.ts): evidence as short strings <verb><kind><spot>.<life><caveat flags>, decision lists/history and slot references as evidence indices, ages kept only for unfaded tastes (faded tastes drop their clock). decodeSave rebuilds the full internal state. 38/38, tsc clean. Size check: could not measure bytes through the runner (try does not print save size); replayed 10 minutes of a taste+witness on every life of every spot with a resume after each cycle (scratch-size.jsonl, 104 evidence items, all resumes 'same view: true'); by hand estimate ~1.5 KB, under 4096. Uncertainty: save size grows linearly with evidence, so play far beyond ten minutes could exceed 4 KB.

## cr13
Added a remembered window (last 6 observation ids); belief derives from it only. Evidence store is garbage-collected after every accepted event to the window plus each spot's current-life taste and consumedBy, so state is bounded (<= 14 evidence entries, reopenedBy/history capped at 6 via pushBounded, CR7 count keeps at most 2 ids, faded tastes drop their clock, a spot at its last life stops counting). First commit restated: while decision is none, any observation after which belief is probably_safe commits with basis [that observation] and its caveats. Save format v2 references evidence by short id codes instead of indices (forgotten ids can still be cited by basis/history). 41/41, tsc clean; scratch replays (13 resumes over ~10 min, 64-life cap) still behave.

## cr14
Slots carry an optional marked flag; mark/unmark events validated per table; mushroom view adds marked and ANDs canAbsorb with !marked (absorb validation uses the full view); why.absorb gains 'Marked to avoid' after Known duskcap; consume clears the mark and regrowth starts unmarked. Save format v3 adds the mark per spot. 44/44, tsc clean.

## cr15
Added pit to the level plus an OUT_OF_REACH list: view forces canAbsorb/canTaste false; taste and mark reject it explicitly (absorb via canAbsorb; unmark via never marked); why gains 'Out of reach' after 'Already eaten'. Witness/regrowth/save unchanged (save indexes spots by LEVEL). 47/47, tsc clean.

## cr16
State gains journal (<= 6 {text, because}); record() adds the observation entry and journalTrust() adds the trust entry right after any history append. View computes current caveats per entry. Evidence GC also keeps ids cited by the journal (still bounded). Save v4 encodes journal as [code, refs] and rebuilds text. 50/50, tsc clean, long replay resumes still identical.

## Summary
All 17 phases (base, cr1–cr16) committed complete, each with every scenario passing and `node run.mjs check` clean before commit. References read: README.md, phases/*.md, scenarios/current.json only. Departures from rules: none. Scratch files written in this directory: scratch-lives.jsonl, scratch-size.jsonl and their output .txt files (used only via `node run.mjs try`). Remaining uncertainties: timer/fade/regrow thresholds use exact float comparisons (fine for the 0.0625 ticks in the scenarios; dt values like 0.1 accumulate rounding); save size was estimated, not measured byte-for-byte, since the runner does not report it.
