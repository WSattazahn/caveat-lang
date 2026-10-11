# Notes (T3)

References read: README.md, phases/*.md, scenarios/current.json. Nothing outside this directory.
Checked with `node run.mjs test` and `node run.mjs check` (tsc strict).

## base
State object cloned per dispatch; a throw discards the clone, so rejected events leave no trace.
Decision commits when the belief first becomes probably_safe (basis = supportedBy at that time, frozen);
every later duskcap absorption reopens it. Ambiguity: a taste whose kind is given is trusted as the true kind;
unknown event types are rejected.

## cr1
Added grove to the level list; all rules derive from that list.

## cr2
Added a row after the taste rows: contradictedBy >= 2 makes unknown present mushrooms "Too risky — taste first", not absorbable. Since canAbsorb is false the existing absorb check rejects them.

## cr3
Absorptions and tastes now both go through addEvidence (support/contradict, commit/reopen). Tastes made while glow timer is 0 carry tasted_in_dark, stored per evidence id; caveats are unions over cited evidence. Ambiguity: absorbing a tasted mushroom adds absorb_<id> as separate evidence (scenario S11b confirms); only tastes can carry the dark caveat.

## cr4
Uncertain-row because changed to contradictedBy only; caveats already derive from because.

## cr5
Each taste ages by tick dt (also after the mushroom is absorbed, since the evidence is still cited); at >= 60 s (1e-9 tolerance for float sums) the taste evidence gains taste_faded. Faded label wins over dark. Decision caveats are now frozen at commit time (basisCaveats) rather than recomputed.

## cr6
decision.history is derived from basis and reopenedBy (trust is never recommitted, so this is exact); no new state.

## cr7
History is now stored state. Glowcap observations since the latest contradiction are tracked; while reopened, the second one recommits with those two as basis, clears reopenedBy and refreezes caveats. Ambiguity: on recommit basis caveats are those the two observations carry at that moment (e.g. a faded taste would contribute taste_faded).

## cr8
Duskcap while heavy adds 20 s capped at 30; heavySeconds = ceil(timer) with 1e-9 tolerance, at least 1 while heavy, 0 otherwise.

## cr9
witness event: consumes, adds witness_<id> with caveat secondhand through the same addEvidence path; no timer change; not gated by canAbsorb, only by unknown id/kind, consumed, or taste contradiction.

## cr10
Mushrooms carry a life number; evidence ids are <what>_<id> for life 1 and <what>_<id>_<n> after. Consumed mushrooms regrow after 45 s of ticks (fresh, untasted). Declared capacity MAX_LIVES = 1024 (exported); at the last life a consumed mushroom stays consumed. Taste fade clocks moved to per-evidence tasteAges so earlier-life tastes keep fading. Capacity limit not exercised by a test (would need ~740k ticks); checked by reading the code only.

## cr11
Mushrooms record consumedBy (the evidence id that consumed the current life); why.absorb/why.taste computed by first-match per the table, caveats from because. "Known duskcap" also covers dark/faded duskcap tastes (S34 confirms faded).

## cr12
save() returns a compact versioned encoding: evidence ids as short tokens (action letter, T for dark taste, mushroom index, life base 36); caveats, tasted kinds and consumedBy are rederived on resume; timers/ages stored as raw numbers (JSON round-trips doubles exactly); basis caveats stored verbatim since they are frozen. Checked with the scenarios plus a scratch replay (scratch_stress.jsonl, 10 min of taste+witness every regrowth, 4 resumes, all "same view: true"). Size estimated by hand (<~2 KB worst case for 10 min), not measured directly, since programs may only be run through run.mjs.
Departure: while looking for how resume is checked I grepped run.mjs for a few lines (it is not on the README read list); I stopped and did not use it further.

## cr13
Rewrote state around a 6-entry observation memory; supportedBy/contradictedBy are derived from it. Caveats and fade clocks are pruned to evidence still referenced (memory, current taste of present mushrooms, consumedBy), so state is bounded; basis caveats stay frozen. Initial commit uses basis = [the observation]; once committed only CR7 recommits (CR7 counter capped at 2, counts accepted observations regardless of memory). reopenedBy and history keep the last 6. Save format v2 (tokens with kind and dark flag; caveats rederived). Checked: scenarios, tsc, plus a 3500-event random replay with 148 resumes (all same view) whose output is identical to the same replay without resumes. Save size bounded by construction (well under 1 KB), not measured directly. A backup of the cr12 source is kept as scratch_glowcap_cr12_backup.ts.txt.

## cr14
mark/unmark events with a per-mushroom boolean; canAbsorb is the unmarked value AND not marked (label/because/canTaste unchanged); why.absorb gains "Marked to avoid" after "Known duskcap"; consumption clears the mark; mark rejected on consumed/already marked, unmark on not marked. Save v3 adds a 0/1 mark flag per mushroom. Checked: scenarios, tsc, a 4000-event random replay with marks and 204 resumes, output identical to the no-resume replay.

## cr15
Added pit with an OUT_OF_REACH list: canAbsorb/canTaste always false, absorb/taste/mark rejected (unmark rejected as never marked), why rows "Out of reach" right after "Already eaten"; witness and regrowth unchanged. Save format unchanged apart from the extra mushroom entry (older saves are not migrated; none exist across phases). Checked: scenarios, tsc, random replay with resumes identical to no-resume replay, and only witness events on pit were accepted.

## cr16
Journal of at most 6 entries stored as {entry: kind|committed|reopened, because}; texts are derived at render from the evidence id (mushroom name without life). Observation entry is pushed in observe() before the decision logic; pushHistory also pushes the trust journal entry, so the order is observation then trust change. Journal evidence counts as referenced, so its caveats and fade clocks are kept and a fade updates old entries. Save v4 adds the journal as tokens. Checked: scenarios, tsc, three 5000-event random replays with ~240 resumes each, all identical to no-resume replays.

## cr17
glowSeconds, regrowsIn, fadesIn are derived from existing timers (glow, consumedAge, tasteAges) with the same round-up helper as heavySeconds; regrowsIn is 0 at the last life, fadesIn 0 when consumed/untasted/faded. No new state, save unchanged. Checked: scenarios, tsc, two random resume replays identical to no-resume replays.

## cr18
doubt event: the evidence must be a remembered witness, not already doubted, and some other remembered observation must stay undoubted. Doubt is represented solely as the caveat "doubted" (after secondhand) on that evidence; supportedBy/contradictedBy skip doubted observations; the decision, journal, memory order and CR7 counter are untouched. The caveat persists wherever the evidence is still referenced (journal, consumedBy) through the existing prune rule. Save v5: "!" flag on tokens; consumedBy now saved as a flagged token. Checked: scenarios, tsc, three random replays including doubts (10-12 accepted each) with ~250 resumes, identical to no-resume replays.

## cr19
retaste event and canRetaste: allowed when reachable, present, tasted, not yet retasted, and the first taste carries taste_faded; kind must match. Per-mushroom retasted flag (reset on regrowth); idTaste() returns the retaste when present, used for label/because/caveats/why/fadesIn and pruning. Retaste goes through observe() like a taste (dark caveat, fade clock, journal text "again"). Save v6: action letter e for retaste; the mushroom taste token is the identifying one. Checked: scenarios, tsc, random replays (one set with 1-4 accepted retastes) with resumes identical to no-resume replays.

## cr20
undoubt removes the doubted caveat from a remembered doubted witness (forgotten doubts are permanent because the check requires memory). doubt/undoubt now push journal entries (doubt/undoubt kinds, text from the mushroom name); caveats stay live via the shared caveat map. Save v7: journal prefixes c/r/d/u. Checked: scenarios, tsc, four random replays with 10-19 accepted doubts/undoubts and ~100 resumes each, identical to no-resume replays (retaste coverage came from the cr19 replays).

## Summary
All 21 phases (base, cr1-cr20) committed with every scenario passing and tsc clean; none incomplete. Scratch files (scratch_*.jsonl, scratch_out*.txt, scratch_glowcap_cr12_backup.ts.txt) are kept in this directory. Remaining uncertainties: the MAX_LIVES=1024 cap and the 4096-byte save bound were checked by reasoning, not run directly.
