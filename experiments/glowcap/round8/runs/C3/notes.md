# C3 notes

## References read
README.md (package), docs/AGENT_START.md, docs/REFERENCE.md, docs/reference/docs/AI_AUTHORING.md,
docs/WORKED_EXAMPLE.md, docs/STARTER.md, templates/umbrella.cav, lib/session.d.mts, lib/node.d.mts,
specs: typed-parameters, repetition, routed-repetition (start), reactive-0.1, view-0.1,
decision-journal, explanations-0.1, save-0.1 (start).

## Phase base
Program: entities per mushroom, a routed `for mushroom` block with per-mushroom evidence
(`absorb_<id>`, `taste_<id>`), consumed/taste states, rejects and bindings. Counts `support` /
`contradiction` are built as `count + qualified(1, absorb_<id>)` so they carry the absorptions;
bindings cite them with `because`, and the adapter reads `binding_explanations` as the evidence
lists. `trust` is a decision series committed `using support` (frozen grounds); duskcap absorptions
reopen it (each new evidence adds a reopening edge). Adapter reads basis/reopenedBy from the
decision journal, state from a binding. Checked with `validate`, `check` and `run.mjs test` (14/14).
Ambiguity: the adapter renames payload fields id/kind to target/sort (a typed param named `kind`
seemed risky).

## Phase cr1
Added one `entity grove` line; the for block and the adapter (which reads members from the interface) cover it. 15/15.

## Phase cr2
Added `twice_bitten` (contradiction >= 2) and a per-mushroom `too_risky` define used by canAbsorb, a reject and a label citing `contradiction`. check clean, 17/17.

## Phase cr3
Tastes now reveal with a stance on the general claim and add to the counts; `on taste when glow == 0 qualify taste_<id> with tasted_in_dark` before the counts are updated, so values built from it carry the caveat. A bitter taste reopens trust; commit moved into a `proc consider_trust` called on absorb and taste. Adapter: caveats from binding_explanations (mushroom label; union of the two belief lists) and the committed journal entry (frozen at commit time). Ambiguity: decision caveats = caveats of the basis as recorded when committed (S19 confirms). 19/19.

## Phase cr4
One-word change: the uncertain label now cites only `contradiction`; caveats follow from the explanation. 20/20.

## Phase cr5
`on taste qualify taste_<id> with taste_faded after 60` (late qualification on the tick clock; 0.0625 steps are exact) plus two faded labels after the dark ones. Late qualification leaves commitment grounds and journal caveats alone, so the decision keeps its own. No adapter change. 22/22.

## Phase cr6
The runtime decision journal already is this history; the adapter maps the trust entries to {change, because}. 24/24.

## Phase cr7
Added state `recovery` (glowcap observations since the last contradiction, carrying their evidence, reset to 0 by a contradiction) and moved the count updates into procs `glowcap_seen`/`duskcap_seen` (numeric parameters keep provenance). `consider_trust` recommits `using recovery` while reopened and recovery >= 2. Series limit raised to 16 (4 mushrooms x 2 observations bounds the number of revisions well below it). 26/26.

## Phase cr8
Heavy max raised to 30; duskcap sets `if(heavy > 0, min(30, heavy + 20), 20)`; binding `slime.heavySeconds = ceil(heavy)` passed through by the adapter. 27/27.

## Phase cr9
New `witness` event and `witness_<id>` evidence declared with `secondhand qualifies`. Refactored the shared observation rules (reveal, counts, recovery, reopen) into `proc saw(e evidence, k)`, called from absorb, taste and witness; `$m_disagrees` define for the taste-contradiction reject. Taste dark qualification now follows the call (late qualification updates current values anyway, and precedes the commit). Adapter renames id/kind for any event that carries them. 30/30.

## Phase cr10
Renewable evidence (`renewable absorb_/taste_/witness_<id> limit 64`), a per-mushroom life counter (capacity 64 lives), `consumed_at = elapsed()` on consumption, and on tick a `regrowing` flag (computed once, so later rules see one decision) that renews the three evidence names, clears the taste and consumed state and bumps the life. Adapter renames the runtime occurrence `absorb_cave@2` to the view id `absorb_cave_2` (presentation only). Checked the capacity with scratch_lives.jsonl via `run.mjs try`: 64 witnessed lives of ruin, the 65th witness refused as consumed, no tick refused, resume gives the same view. 33/33.

## Phase cr11
`consumed` now set to `qualified(1, <evidence>)` so it carries what consumed the current life; `whyAbsorb`/`whyTaste` bindings in reverse priority order (last match wins) with `because` citations; adapter projects them with their explanations. 35/35.

## Phase cr12
The adapter already saved via `session.save()`; the first run failed S36 only on size (4336 bytes) because the save text was a JSON string re-escaped by JSON.stringify. `save()` now returns the parsed save object and resume stringifies it for `runtime.restore`. 38/38. Uncertainty: I did not measure the margin under 4096 bytes beyond the harness checks; long sessions with many regrowths grow the save (renewal occurrences, journal).

## Phase cr13
Read specs: windows-0.1, departure-0.1, lineage-compaction (series section), late-qualification, reactive-0.3 (provenance of guarded sets), reactive-0.7 (skipped calls).
Memory: six slot pairs `g1..g6`/`d1..d6`, shifted unconditionally on every observation in `proc saw`; the newest observation sets slot 6 to `qualified(1, e)` or a plain 0, so each slot carries exactly what it remembers and support/contradiction are sums (defines). First commit is `using g6` (the observation that made it safe). CR7 recommit uses the two newest slots (`g5 == 1 and g6 == 1` while reopened), which removed the `recovery` state. `journal window 6`; renewables now `window 2` (no renewal limit; I dropped the 64-life cap and the life counter, so the declared capacity is unlimited). Adapter: basis/caveats from `commitment_grounds` of `decision_series.trust.current` (minus `inherited`), reopenedBy/history from the (windowed) journal.
Save size: the first version failed S41 on size (4652, then 4407 bytes). Trimmed by deriving consumed/tasted from `observed(...)` of the current occurrences instead of states with lineage, a plain `$m_taste` code, an `eaten_at` sentinel for regrowth, a shorter claim name, and short-circuit guard ordering (`not committed(trust) and belief == 1`) so revisions inherit less. 41/41.
Checked long play with scratch/gen.py + `run.mjs try` (temporary C3_DUMP hook in the adapter, removed afterwards): play WITHOUT recommits stays flat, but every recommit of `trust` adds a series revision whose basis and reopening qualifications (with departure markers) cost ~1.5-3 KB, so random play with many recommits exceeds 4096 bytes (10 KB after 5 cycles, 51 KB after 20). The runtime refuses `decisions trust window N` ("a decision series has no window yet"), and a state-machine replacement would have its lineage polluted by skipped-guard provenance, so I kept the series. Known limits: (1) save bound not met for play with many recommits; (2) a reopened revision's grounds are not pinned, so after its committed journal entry leaves the window and its occurrences depart, `basis` could lose names; (3) `history_count(trust) < 64` stops recommitting after 64 revisions rather than refusing events.

## Phase cr14
`mark`/`unmark` events (target only) routed into the mushroom block; a plain `$m_marked` state (no evidence), cleared on absorb/witness; canAbsorb and an absorb reject read it; `Marked to avoid` why-row placed between Too risky and Known duskcap (last-match-wins order). Adapter now renames payload keys generically (the first attempt sent `sort: undefined`, which the kit refuses as non-JSON). 44/44.

## Phase cr15
Added `entity pit`; in the block `define $m_out_of_reach = $index == target.pit` (a source constant, no evidence) gates canAbsorb/canTaste, rejects absorb/taste/mark, and adds the `Out of reach` why-rows just below `Already eaten`. 47/47.

## Phase cr16
Journal kept in Caveat beside the memory slots: plain code states `j0..j6` (observation kind + mushroom position) shifted with the slots (`g0/d0` added for a recommit cited in slot 1), and one base-4 state `marks` recording which trust entry each observation caused. Text comes from pure fns (`observation_text`, `mushroom_name`, a positional name table that must follow the entity order); each entry binding cites the memory slot(s) (`because gN + dN`), so current caveats (fades) follow. The adapter flattens the six observation/trust pairs in order and keeps the last six entries (a presentation slice; the program holds at least six entries).
Save size again drove the design (CR12 bound failed S41 at 4187, then 4909/5132): lessons recorded from dumps: (a) a procedure call's arguments enter the lineage of every state the call sets, so the journal shift is a no-argument proc; (b) a skipped conditional `set` adds its guard's lineage to the target, so trust-entry markers live in one state (`marks`) rather than per-slot states; (c) CR5's fade is now an on-tick `qualify` at 60 s (`tasted_at`), with a scheduled `qualify ... after (remaining)` only when the mushroom regrows before its taste fades (scheduled entries cost ~120 bytes each). 50/50; S41's second resume measured ~3.9 KB with a temporary dump hook (removed).

## Phase cr17
Bindings only: `slime.glowSeconds = ceil(glow)`, per mushroom `regrowsIn` from `gone_at` and `fadesIn` from `tasted_at` read against `elapsed()` (both sentinels already existed, so nothing new is stored). 53/53.

## Phase cr18
Memory restructured: one evidence-carrying slot `e_N` per remembered observation plus plain codes `j_N` (life*100 + kind*10 + position, so a remembered witness can be matched by id), a bitmask `doubts` and the base-4 `marks`, all shifted together by a no-argument `shift_memory`. The belief sums `if(is_glow(j_N) and counts(j_N, doubts, 2^N), e_N, 0)`: the condition reads only plain codes, so it selects memories without citing them, and a doubted memory leaves the belief while the journal (which cites `e_N`) still shows it. New `doubt` event `via in absorb taste witness, target kind mushroom, life`: the adapter splits the host's evidence id `witness_pool_2` into those parts (other strings go through unchanged and the runtime refuses them). Rejections (not a witness, not remembered, already doubted, last undoubted) are program rules; `qualify witness_<id> with doubted` attaches the caveat. A per-mushroom `life` counter is back (for the codes). 56/56.
Known limit: `qualify` can name only the current occurrence, so doubting a remembered witness from a mushroom's earlier life (it has regrown since) removes it from the belief but cannot attach `doubted` to that older occurrence; views citing it would lack the caveat. No scenario exercises this.

## Phase cr19
`retaste` event and renewable `retaste_<id>` evidence (window 2, renewed at regrowth). It goes through `saw` like any observation (codes 7/8), takes `tasted_in_dark` in the dark, and reuses `tasted_at` for its own fade (the first taste must already have faded, so the timer is free); the fade rules qualify the retaste once there is one. Rejections and `canRetaste` share one define. Retaste-based label/why bindings follow the taste-based ones so they win (last match). 59/59.

## Phase cr20
`qualify ... with doubted` cannot be undone, so the doubted caveat is no longer attached to evidence. It is shown through an `examined(doubted)` citation instead, and the caveat appears only while a plain flag says the cited witness is doubted. Changes:
- **Doubt bits.** `doubts` now keeps one bit per slot from -4 to 6. A bit is frozen once its witness is forgotten, which makes a forgotten doubt permanent. A doubt entry still in the journal never refers further back than slot -4.
- **Journal records.** The journal is now six records, packed as 4-digit codes in two plain states `ra`/`rb`. Each code is observation number mod 100, record type and mushroom. Observation and trust records cite the memory slot they are about. A doubt or undoubt record keeps its own carrier `rd_N`, because its witness may be forgotten. The bindings are `diary.rN` (text), `diary.cN` (citations) and `diary.dN` (doubted). They were generated by `scratch/genjournal.py`.
- **Why rows.** Each mushroom has a plain `$m_doubted` flag for its current-life witness, used by the "Already eaten" rows.
- **Plain guards.** Guards now read plain mirrors: `plain_belief` from the codes, and `trust_code`, which packs the standing and the commit count. That keeps the record states free of lineage.
- **`undoubt`.** The adapter parses its evidence id the same way as `doubt`. It is rejected when the witness is not remembered or not doubted, and an id that is not a witness is also rejected.

Side effect: doubting a witness from an earlier life now does show `doubted`, which removes the cr18 known limit.

How it was checked: 62/62. S59's save measured about the same as in cr19 (raw text 4102 vs 4101 bytes, under the harness bound), using a temporary dump hook that has been removed. I also ran manual `try` runs covering doubt/undoubt/re-doubt, a commit citing two witnesses one of which is doubted, an older-life witness, and resume.

Departure: the recommit cap now counts commits (`trust_code`) instead of `history_count`, so it is still 64.
