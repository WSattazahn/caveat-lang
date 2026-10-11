# Notes (T2)

References read: README.md, phases/*.md, scenarios/current.json. Checked with `node run.mjs check` (tsc) and `node run.mjs test`.

## base
State is plain JSON (timers, per-mushroom consumed/tasted, ordered absorption log, decision); view is derived. Rejection: events apply to a deep clone that replaces state only on success. Ambiguities: "uncertain" because-list uses absorption order; commit happens only on the transition into probably_safe while decision is none; unknown event types are rejected.

## cr1
Added `grove` to the level list; no other change.

## cr2
Moved per-mushroom view into one `mushroomRow` function used both by the view and by absorb validation; added the twice-bitten row (checked after consumed/tasted, before belief rows).

## cr3
Replaced the absorption log with an ordered evidence log (absorb and taste entries, each with caveats); an `observe` step updates trust for both. Caveats are unions over cited evidence. Absorbing a tasted mushroom adds a second evidence entry (absorb_<id>) alongside taste_<id>. One test run of cr3 happened against the unchanged cr2 code (a failed file write), 15/19. Ambiguity: "dark" = glow timer is 0 at taste time.

## cr4
Uncertain unknown mushrooms now cite contradictedBy only; caveats follow because automatically.

## cr5
Taste evidence carries an `age` advanced by ticks; at age >= 60 s it gains the `taste_faded` caveat (appended, so dark then faded). Faded label wins over dark. Decision now stores its caveats at commit time (frozen). Uncertainty: no float epsilon on the 60 s threshold (consistent with timers); tastes of later-absorbed mushrooms still fade.

## cr6
Decision stores an explicit ordered `history` appended on commit and on each reopen (not derived from basis/reopenedBy, so it survives future rule changes).

## cr7
Factored a `commit` helper. While reopened, a glowcap observation recommits when the evidence log has two glowcap entries after the latest duskcap entry; basis = those two in order, reopenedBy reset, caveats snapshotted at that moment. Ambiguity: absorbing a mushroom earlier tasted sweet counts as a second observation (S25 confirms).

## cr8
Duskcap absorb while heavy adds 20 s capped at 30; `heavySeconds` = ceil(timer) or 0. Uncertainty: no epsilon in ceil, so accumulated float error with non-dyadic dt could round up one extra second.

## cr9
Added `witness`: consumes, adds `witness_<id>` evidence with `secondhand`, goes through the same `observe` (commit/reopen/recover); skips the canAbsorb check, keeps the taste-agreement check. Ambiguity: witnessing a mushroom tasted bitter (kind duskcap) is allowed.

## cr10
Each mushroom spot has a `life` number and a `consumedFor` clock; ticks regrow it (untasted, unknown) at >= 45 s while life < MAX_LIVES (declared 1024, exported). Evidence ids carry `_<life>` from life 2. Old evidence and taste ages are untouched. Capacity exhaustion not exercised by scenarios; checked by reading the code only.

## cr11
Rows are now built from two refusals (absorb, taste); canAbsorb/canTaste derive from them, so `why` cannot disagree with the flags. Mushrooms record `consumedBy` (the evidence id that consumed the current life), cleared on regrowth.

## cr12
`save()` returns a compact encoding of the state (evidence as short codes like `tg1d.3`, decision/consumedBy as indices into the evidence log, taste ages only for unfaded tastes; faded tastes now drop their clock in play too, so decoded state equals live state). Checked with scenarios plus a scratch 10-minute stress replay (`stress.jsonl`, dt 0.1, regrowth, tastes, witnesses, 8 resumes, all "same view: true"; 73 evidence entries, est. well under 1 KB of evidence codes). Save size was estimated by reasoning, not measured directly, since the runner does not print it. Resume trusts its input (only a version check).

## cr13
State now keeps a bounded evidence registry (only what is cited: the 6-entry remembered window, each untasted-consumed mushroom's consumedBy, each present mushroom's current taste), a window of ids, and a CR7 `recovery` list (glowcaps since last contradiction, max 2), so CR7 counts accepted observations regardless of forgetting. Base commit now uses basis = the single committing observation (equivalent to before for earlier phases). reopenedBy/history capped at 6. Save format v2 encodes ids as short codes since basis/history ids may refer to pruned evidence. Worst case: <=14 evidence codes + 6 window + 2 recovery + <=6 basis/history entries, so well under 4096 bytes for any play length (estimated, not measured). Stress replay still gives "same view: true" at all 8 resumes.

## cr14
Added `marked` per mushroom, `mark`/`unmark` events, and a `Marked to avoid` absorb refusal placed after Known duskcap and before Too risky untasted; canAbsorb follows from it, so absorb is rejected. Consuming (absorb/witness) and regrowth clear the mark. Save format v3 adds one 0/1 per mushroom. Ambiguity: `mark` of a known (tasted) mushroom is allowed; marking a tasted duskcap is accepted (S43 confirms).

## cr15
Added `pit` with an OUT_OF_REACH list. Unknown-mushroom rows now compute label/because first, then refusals; out-of-reach overrides both refusals right after Already eaten. `taste` now also checks canTaste (only changes behaviour for pit). `mark` rejects pit. Save v4 (5 mushrooms). Second stress replay (`stress2.jsonl`, marks + pit, 11 resumes) all same view.

## cr16
Journal stored as {entry: kind|change, because}; text derived in the view from the evidence id verb and mushroom name (life stripped), caveats computed live so fades show. `observe` split into `decide` (returns the history change, if any) plus journal writes. Pruning also keeps evidence cited by the journal, so fades of journal-cited tastes keep ticking. Save v5 adds `j`. Both stress replays still resume to the same view.

## cr17
Added glowSeconds, regrowsIn (ceil(45 - consumedFor) while consumed and lives remain) and fadesIn (ceil(60 - age) of the current-life taste while unfaded and present), all derived from existing timers; no new state, save format unchanged. Each countdown reaches 0 at exactly the tick where its threshold check fires, since both use the same timer values.

## cr18
Added `doubt`: validated against the remembered window (witness ids only, not already doubted, at least one other undoubted remembered observation); it appends the `doubted` caveat to the evidence and nothing else. The belief (split) skips doubted evidence; belief caveats now come from the counted (undoubted) entries. Decision/journal/recovery untouched. Save v6 adds an `x` flag. Third stress replay (`stress3.jsonl`, 15 accepted doubts, 15 resumes) all same view.

## cr19
Added `retaste` (verb `retaste`, code `r`), a per-life `retasted` flag reset on regrowth, an `identifyingTaste` helper used by the tasted row, fadesIn and pruning, and `canRetaste` (present, reachable, tasted, not retasted, first taste carries taste_faded); retaste validation uses canRetaste then the kind check. Retaste goes through the normal `observe`, so it counts for belief, commit/reopen/CR7 and journal. Save v7 adds a retasted flag. Fourth stress replay (`stress4.jsonl`, 8 accepted retastes, 15 resumes) all same view.

## cr20
Added `undoubt` (remembered doubted witness only; removes the caveat) and journal entries for doubt/undoubt (entry types `doubt`/`undoubt`, codes x/u, save v8). Shared `rememberedWitness` validation and a `journal` helper that also prunes. Fifth stress replay (`stress5.jsonl`, doubts + undoubts + retastes, 15 resumes) and all earlier stress files still resume to the same view.

## Wrap-up
All 21 phases (base, cr1-cr20) committed complete; every phase's scenarios passed before commit. One deviation in process: early in cr3 a `test` run executed against unchanged cr2 code because a file write failed (recorded, 15/19). Scratch files kept in this directory: stress*.jsonl, stress.out, stress2.out. No rules departures: only README, phases/, scenarios/, impl/, run.mjs (read to learn what `try` prints) and my own files were read; program run only through `node run.mjs`.
Residual uncertainties: no float epsilon on timers (non-dyadic dt could shift a threshold by one tick); the save size bound was established by reasoning about bounded lists, not measured; resume does only a version check on its input.
