# Notes (T1)

## base
Read README.md, phases/base.md, scenarios/current.json. Implemented impl/glowcap.ts:
state is plain JSON; dispatch applies to a deep clone and swaps only on success (atomic rejection).
Checked with `node run.mjs check` (tsc clean) and `node run.mjs test` (14/14).
Decisions: absorbing a tasted glowcap still counts as absorption evidence (S11);
commit basis = supportedBy at the moment the belief first becomes probably_safe;
reopen only happens for duskcap absorptions after commit. Non-finite dt rejected.

## cr1
Added `grove` to the level list. check clean, test 15/15.

## cr2
Refactored per-mushroom rules into one function used by both view and absorb validation, so canAbsorb=false always rejects. Added "Too risky" row after the taste rows (unknown = untasted). check clean, test 17/17.

## cr3
Tastes now feed supportedBy/contradictedBy through the same record() as absorptions (commit/reopen). Each evidence id stores its caveats; taste while glow timer is 0 gets tasted_in_dark (decided at taste time, glow>0 counts as light). caveats = union over because / supportedBy+contradictedBy / frozen basis. Dark-tasted labels keep the certain rows' canAbsorb/canTaste. check clean, test 19/19.

## cr4
Uncertain unknown mushrooms cite contradictedBy only; caveats derive from because automatically. check clean, test 20/20.

## cr5
Track per-taste age (sum of tick dt); at >= 60 s (1e-9 tolerance, my choice for float sums) the taste evidence gains taste_faded. Faded label wins over dark. Decision caveats are now frozen at commit time and stored. Ambiguity: tastes of later-consumed mushrooms keep aging (evidence still cited by belief). check clean, test 22/22.

## cr6
Decision stores history entries appended on commit (because = basis) and each reopen (because = [evidence]). A first scripted edit failed (python str.replace hit two places, assertion aborted before writing); redone. check clean, test 24/24.

## cr7
State keeps glowcap observations since the latest contradiction (reset by every contradiction). While reopened, the second one recommits with those two as basis; reopenedBy cleared, caveats frozen from the new basis, history appended. Initial commit rule (state none) unchanged, so support after a contradiction with no prior commit still never commits. check clean, test 26/26.

## cr8
Duskcap absorption while heavy adds 20 s capped at 30; otherwise sets 20. heavySeconds = ceil(remaining - 1e-9), at least 1 while heavy, 0 otherwise (tolerance is my choice against float drift from non-binary dt). check clean, test 27/27.

## cr9
Added witness event: validates id/kind/consumed/taste agreement (not canAbsorb), consumes, records witness_<id> with caveat secondhand via the shared record() (commit/reopen/recover), timers untouched. check clean, test 30/30.

## cr10
Each mushroom has a life number and consumed-age; after >= 45 s of ticks (1e-9 tolerance) it regrows untasted with life+1. Evidence ids: life 1 unsuffixed, later lives `_<n>`. Declared capacity MAX_LIVES = 1024 (exported); on the last life it stays consumed, ticks still accepted. Taste ages are keyed by evidence id so old tastes fade independently. Capacity limit itself not exercised by a run (would need ~460k tick events); checked by reading the code. check clean, test 33/33.

## cr11
Mushroom state records consumedBy (evidence id of the current life's consumption, cleared on regrowth). why.absorb/why.taste follow the table; reason is empty exactly when canAbsorb/canTaste is true. check clean, test 35/35.

## cr12
save() returns a compact versioned encoding: one ordered evidence table (sign, source letter, mushroom index, life, caveat codes) and indices everywhere else; createPolicy(saved) decodes it to the identical internal state. Faded tastes drop their age entry to keep saves small (behaviour unchanged). Checked with scenarios (38/38) and `try stress.jsonl` (scratch file: ~10.5 min of maximal evidence, 112 evidence ids, resumes mid-way and at end report same view). I could not measure the byte size directly (only run.mjs may run the program and try does not print it); by estimate the worst case is about 3 KB (about 120 evidence entries at ~11 bytes plus history/reopenedBy indices), under 4096. Uncertainty: that estimate is not measured.

## cr13
Replaced supportedBy/contradictedBy with a memory of the 6 most recent observations (oldest dropped on the 7th); belief, labels, CR2/CR11 risk checks derive from it. First commit (decision none) now uses basis = the single observation after which belief is probably_safe. sinceContradiction keeps only its first two entries; reopenedBy and history capped at 6. After every accepted event, caveats and taste clocks of evidence nothing can cite any more (not in memory, not a current-life taste of a present mushroom, not a consumedBy) are pruned, so state and save are bounded. New save format v2 writes evidence as short codes. Backup of the cr12 version kept as glowcap.cr12.bak.ts.txt. check clean, test 41/41, stress try resumes report same view.

## cr14
Added per-mushroom `marked`; mark/unmark events with the listed rejections; marked forces canAbsorb false (so absorb rejects) without touching label/because/taste; witness ignores marks; absorb/witness clear the mark; why.absorb gains "Marked to avoid" between Known duskcap and Too risky. Save format v3 adds a marked flag per mushroom. check clean, test 44/44.

## cr15
Added `pit` to the level and an OUT_OF_REACH list: absorb/taste/mark rejected for it (unmark rejected because it is never marked), view forces canAbsorb/canTaste false, why gives "Out of reach" for both right after the consumed row. Witness and regrowth unchanged. Save version bumped to 4 (five mushroom rows). check clean, test 47/47.

## cr16
Journal (max 6, oldest dropped) stored as {text, because}; caveats derived at render so fades update entries. record()/commit() now return the history entry they added so observe() can append the trust entry after the observation entry. Journal-cited evidence is kept by prune (its caveats and taste clock). Save v5 adds the journal (texts stored verbatim; estimated total save well under 4096 bytes, not measured directly). check clean, test 50/50, stress try resumes same view.

## cr17
View-only countdowns: glowSeconds like heavySeconds (nonzero exactly while glowing); regrowsIn from consumedAge (0 if present or on the last life); fadesIn from the current life's unfaded taste clock (0 if consumed/untasted/faded, since faded clocks are already dropped). Nothing new stored. check clean, test 53/53.

## cr18
New doubt event; the `doubted` caveat on the evidence is the doubt flag (so it follows the evidence wherever it is still cited and survives saves via caveat code x). Belief and everything citing it use remembered undoubted observations; memory slots, CR7 count, decision and journal untouched. Rejections: not a remembered witness_ id, already doubted, or no other undoubted remembered observation. Save v6. check clean, test 56/56.

## cr19
Added retaste: per-life `retasted` flag, evidence retaste_<id>[_n] (save code r), dark caveat and fade clock like a taste, recorded via observe() (belief, CR7, journal text "again"). identifyingTaste() picks the retaste when present and drives label/because/caveats/why/fadesIn. canRetaste = reachable, present, tasted, not retasted, first taste faded. Save v7 adds the flag. check clean, test 59/59.

## cr20
Merged doubt/undoubt handling: undoubt needs a remembered, doubted witness and removes the doubted caveat (re-doubt appends it again). Both write a journal entry naming the mushroom without life number. No decision/CR7/timer change. check clean, test 62/62.

## Summary
All 21 phases (base, cr1-cr20) committed complete; no phase was marked incomplete. Each phase was checked with
`node run.mjs check` (tsc clean) and `node run.mjs test` (all scenarios passed). I read only README.md, phases/,
scenarios/ and impl/. I did not read run.mjs, git history or anything outside this directory, and I used no web or npm packages.
Scratch files in this directory: stress.jsonl (generated 10.5-minute replay used with `try`) and glowcap.cr12.bak.ts.txt (backup).
Open uncertainties: I never measured save byte size directly, only estimated it at under ~2 KB once state was bounded (cr13+).
I did not exercise the MAX_LIVES=1024 capacity in any run. Float tolerance EPS=1e-9 is my own choice for fade, regrowth
and the rounded-up countdowns.
