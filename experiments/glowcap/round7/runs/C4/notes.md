# Notes (C4)

## References read
README.md (package), docs/AGENT_START.md, docs/reference/docs/AI_AUTHORING.md, docs/REFERENCE.md,
docs/WORKED_EXAMPLE.md, lib/session.d.mts, and specs: identifiers, view, procedure-symbols,
explanations 0.1/0.2, observation-order, typed-parameters, define, repetition, routed-repetition
(first part), reactive 0.1 and 0.3 (part), runtime/prelude.cav.

## Phase base
Program `impl/glowcap.cav`: one routed `for mushroom` block with per-mushroom evidence
(`absorb_<id>`, `taste_<id>`), consumed/known states, rejects and label bindings. Belief counts
`support`/`contradiction` are built with `qualified(1, absorb_<id>)`, so their grounds are the
evidence lists; the view's `supportedBy`/`contradictedBy`/`because` are binding explanations.
`trust` is a decision series committed `using support` (frozen grounds = basis); duskcaps reopen it;
`reopenedBy` comes from the decision journal. Adapter only renames payload fields and projects.
Checked with `run.mjs check` (no warnings), `run.mjs test` (14/14), and a `try` file with two
reopenings plus resume (scratch1.jsonl).
Ambiguity: a taste never adds to the belief; absorbing a sweet-tasted mushroom counts as support (S11).

## Phase cr1
Added one `entity grove` line; the routed block and the adapter (which reads mushroom ids from the snapshot world) cover it. check clean, test 15/15.

## Phase cr2
Added `define twice_bitten = contradiction >= 2`, a label binding citing `contradiction`, a routed reject for absorbing an untasted mushroom when twice bitten, and folded it into `can_absorb`. check clean, test 17/17.

## Phase cr3
Taste now also reveals for/against `glowing_is_safe`, adds `qualified(1, taste_<id>)` to support/contradiction, can commit and reopen `trust`. `qualify taste_<id> with tasted_in_dark` when glow == 0 at the taste (before the value is read, so the commit made in the same event keeps it). Dark labels use `carries(...)`. Caveats come from binding explanations (mushroom label, a new `belief.evidence` binding citing both counts) and from frozen commitment grounds. Decision: "not glowing" is read from the glow timer before the taste. check clean, test 19/19.

## Phase cr4
One citation changed: the uncertain label is `because contradiction`; caveats follow from the explanation. check clean, test 20/20.

## Phase cr5
`qualify taste_<id> with taste_faded after 60` on every taste (session clock = sum of tick dt); faded labels placed after the dark ones so they win. Frozen commitment grounds keep their caveats automatically (late qualification leaves them alone). check clean, test 22/22.

## Phase cr6
`decision.history` is a projection of the runtime decision journal for `trust` (change + because; committed entries carry grounds evidence per the journal spec). No program change. test 24/24.

## Phase cr7
New state `recovery` (+ qualified(1, evidence) per glowcap observation, reset to 0 on any contradiction); while `reopened(trust)` and recovery >= 2 a new revision is committed `using recovery`. Series limit raised to 16. Adapter: basis = the current revision's committed journal entry (observation order), reopenedBy filtered to the current revision. check clean, test 26/26.

## Phase cr8
Heavy timer bound raised to 30; duskcap absorb sets `if(heavy > 0, min(heavy + 20, 30), 20)`; new binding `slime.heavy_seconds = ceil(heavy)` (0 when not heavy since the timer floors at 0). check clean, test 27/27.

## Phase cr9
Refactored the shared belief/trust rules into `proc learn(e evidence, sort)` (reveal, counts, recovery reset, reopen, both commits), called by absorb, taste and witness. Added `event witness`, `witness_<id>` evidence qualified by the declared caveat `secondhand`, and witness rejects (consumed, contradicts taste) in the routed block; witness does not touch timers or the too-risky rule. check clean, test 30/30.

## Phase cr10
Each mushroom's absorb/taste/witness evidence is `renewable ... limit 64` (capacity: 64 lives). Consumption records `regrows_at = elapsed() + 45`; on tick a per-mushroom flag is computed once, then `proc regrow` renews the three evidence names and known/consumed/life reset. Old occurrences keep their caveats and scheduled fades. The adapter renames runtime occurrence ids `absorb_cave@2` to the required `absorb_cave_2` (pure naming). Bug found by my own 64-life try file: guarding each regrow rule with the same define re-read state changed by an earlier rule (life reached 64, so `consumed = 0` was skipped); fixed with the flag. Verified: 64 lives then rejection as consumed, no tick rejected, resume same view (scratch_lives.jsonl); scaled-down debug copy scratch_dbg.cav via `caveat explain`. check clean, test 33/33.

## Phase cr11
`consumed` is now `qualified(1, <evidence of this life>)`, set after `learn` reveals it, so "Already eaten" cites what consumed the current life. Two bindings per mushroom, `why_absorb` / `why_taste`, ordered so the table's first match is the last winning binding, each with its `because`; caveats from the explanations. check clean, test 35/35.

## Phase cr12
`save()` returns `{caveat: base64(deflateRaw(session.save()))}` and `createPolicy(saved)` restores with the same source (node:zlib is built in, not an npm package). The plain save text in S36 measured 3764 bytes (4333 once JSON-escaped inside my wrapper, which failed the 4096 limit on the first test run); compressed it is ~1.3 KB, leaving room for longer play. To measure it I temporarily added a debug write of the save text to the adapter (removed afterwards); `caveat serve` via run.mjs gets no stdin. Uncertainty: compression is host-side storage, not game logic; the save is still the runtime's own. test 38/38.

## Phase cr13
Belief rebuilt on a six-slot memory window: `glow_1..6` / `dusk_1..6` states each hold `qualified(1, e)` or 0; `learn` shifts them on every observation, so `support`/`contradiction` (now defines summing the slots) have exactly the remembered evidence as grounds, and every label, twice-bitten check and why-reason follows. First commit is now `using qualified(1, e)` (the one observation). `recovery` counts every observation since the last contradiction, capped at 2. Taste identity, consumption and frozen bases are untouched. Series limit 256 (the runtime maximum); with 64 lives x 4 mushrooms there are at most ~170 revisions. Adapter shows the last 6 journal entries and last 6 reopenings of the current revision (view bounding; the runtime journal itself is append-only). check clean, test 41/41.

Known departure from CR13 "the 4096-byte save bound holds for play of any length": it holds in every scenario, but not for long play. The runtime save keeps the whole evidence graph (relations, observation/predicate/reopening qualifications, journal, commitment bases), which grows with every observation by design (Caveat preserves evidence history). Measured with a temporary debug hook (removed) on a seeded heavy-play file (scratch_heavy.jsonl, every mushroom tasted and witnessed each life): after 64 observations the raw save is 85 KB, 7.9 KB compressed; after 512 observations 2.6 MB raw / 128 KB compressed. Resume stayed exact throughout (8/8 same view). I found no language facility to drop forgotten evidence from the session; bounding it would need replacing the runtime save with an adapter-side state encoding, which would move game state out of the program. This would be a candidate for an agent feature request (bounded/compactable history); not filed.

## Phase cr14
New events `mark`/`unmark` (target kind mushroom) with routed rejects; per-mushroom `marked` state set to plain numbers (no evidence), cleared on absorb/witness and so false for a regrown mushroom; absorb rejected when marked; `can_absorb` and a "Marked to avoid" why-binding (`because nothing`) placed between "Too risky" and "Known duskcap" in last-wins order. Adapter maps `{id}` to `{target}` and exposes `marked`. check clean, test 44/44.

## Phase cr15
Added `entity pit` and, in the routed block, `define $m_out_of_reach = $index == target.pit` (true only in pit's copy). It rejects absorb/taste/mark/unmark, is folded into can_absorb/can_taste, and gives an "Out of reach" why-binding just before "Already eaten" (last wins). Witness and regrowth unchanged. No adapter change. check clean, test 47/47.

## Phase cr16
Journal as six numeric slot states in the program (`journal_1..6`), shifted by `proc shift_journal()`; each entry is a code (act*100 + sort*10 + mushroom, 800 reopened, 900 trusting) whose value is `qualified(code, e)` (or `900 + 0 * recovery` for a recovered trust), so its grounds are the cited evidence and late caveats (fades) reach it. Pure functions `mushroom_name`/`observation_text`/`journal_text` render the text; bindings `journal.text_k` cite the slot. Pitfall found: passing the qualified value through a numeric proc parameter gave the slot grounds that included earlier occurrences of a renewed evidence (e.g. witness_pit and witness_pit@2); setting the slot directly in `learn` fixed it (unverified why; possibly parameter lineage counted as grounds). Since journal lists are compared in order and the runtime reports sets sorted by name, the adapter orders a journal entry's evidence by the snapshot's `observations` (refreshed only after events with reveal effects) and caveats by their declaration order in the source (the snapshot sorts symbols by name). Uncertainty: the expected caveat order across different evidence in one entry is not specified; declaration order is my choice. check clean, test 50/50.

## Wrap-up
All 17 phases (base, cr1-cr16) were committed complete, with every scenario passing at its commit. Scratch files (scratch*.jsonl/.out/.cav/.json) are my own `try`, `explain` and `replay` inputs and debug outputs. Departures: the adapter was temporarily modified three times to write save text or symbols to scratch files for measurement (each change was reverted, which I checked with grep). `caveat replay`/`serve` through run.mjs gave truncated or no output for large inputs, so I used `try` with those hooks instead. Open limitation: CR13's save bound for unbounded play (see cr13).
