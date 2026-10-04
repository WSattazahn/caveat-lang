# Notes (C2)

## References read
README.md, phases/, scenarios/. Package docs: README.md, docs/AGENT_START.md,
docs/REFERENCE.md, docs/WORKED_EXAMPLE.md, docs/README.md,
docs/reference/docs/AI_AUTHORING.md, specs: identifiers, repetition, routed
repetition, typed parameters, view, explanations 0.1/0.2, save (start),
procedure symbols / observation order (start), reactive 0.1 (start);
lib/session.d.mts, lib/node.d.mts.

## Design
- Mushrooms are `entity … kind mushroom`; per-mushroom rules in a
  `for mushroom as $m routed by target` block. Host `{id, kind}` maps to
  `{target, sort}` typed parameters (unknown id/kind -> input refusal).
- Evidence `absorb_<id>`/`taste_<id>`. Belief counts `supported`/`contradicted`
  add `qualified(1, absorb_<id>)`, so their grounds are the evidence lists;
  the adapter reads them from `binding_explanations` (bindings cite `because`).
- Labels/text/state/canAbsorb etc. are Caveat bindings; mushroom `because` is the
  label binding's explanation. Decision basis = commitment_grounds of trust@1;
  reopenedBy = trust journal "reopened" entries in order.
- Adapter: rename events, throw on any non-accepted outcome, project the view.

## Phase log
- base: 14/14. Checked with `caveat validate`, `caveat check` (no warnings),
  `run.mjs test`, and a `try` file (two duskcaps after commit + resume) confirming
  repeated reopening is journaled. Ambiguity: an unknown event field throws in
  the adapter (input validation only).
- cr1: added `entity grove`; one line, adapter reads mushrooms from the world. 15/15, check clean.
- cr2: `define twice_bitten = contradicted >= 2`; a reject for untasted absorbs, a label binding citing `contradicted`, and canAbsorb updated. Rejection and canAbsorb are written separately in the .cav (kept consistent by hand). 17/17, check clean.
- cr3: factored absorb/taste learning into `proc learn(e evidence, sort, dark)` (reveal, `qualify e with tasted_in_dark` when dark, counts, commit, reopen); taste stance now on `glowing_is_safe`. Caveats come from binding explanations (labels, `belief.observations`) and commitment_grounds (decision). Dark = glow timer 0 at the moment of tasting. 19/19, check clean.
- cr4: the uncertain label binding now says `because contradicted`; caveats follow automatically. One-line change. 20/20, check clean.
- cr5: `caveat taste_faded`; `on taste qualify taste_$m with taste_faded after 60` (scheduled late qualification on the tick clock) and two label bindings using `has_caveat($m_taste, taste_faded)`, placed after the dark labels so they win. Runtime keeps commitment grounds unchanged, so the decision keeps its original caveats with no extra code. 22/22, check clean.
- cr6: `decision.history` is the runtime decision journal for `trust`, projected as {change, because}; adapter-only change (3 lines). Checked with a try file that the committed entry cites the grounds (taste_ruin), not lineage. 24/24.
- cr7: `state recovery` adds qualified(1, e) per glowcap observation and resets to 0 on a contradiction; in `learn`, `when reopened(trust) and recovery >= 2 commit trust because enough using recovery`. Series limit raised 1 -> 16. Adapter: basis/caveats/reopenedBy now describe the latest committed journal entry (basis taken from the journal `because`, which is in acquisition order). 26/26, check clean.
- cr8: heavy bound raised to 30; `set heavy = if(heavy > 0, min(heavy + 20, 30), 20)`; `bind slime.heavy_seconds = ceil(heavy)`, mapped to heavySeconds. 27/27, check clean.
- cr9: `event witness`, per-mushroom `evidence witness_$m` with `secondhand qualifies witness_$m`; three routed rules (two rejects, consume) plus `call learn(witness_$m, sort, 0)` - the learning rules were reused unchanged. Timers stay on absorb only. Adapter: accept `witness` as an event type. 30/30, check clean.
- cr10: `renewable absorb_/taste_/witness_$m limit 64` (64 lives per mushroom); consuming sets `$m_grows_at` to elapsed()+45 (or 1e12, i.e. never, on the last life); on tick a regrowing mushroom renews its three evidence names, forgets its taste, counts a life and becomes present. Old tastes keep fading on their scheduled clock (runtime). Adapter renames occurrences `x@N` -> `x_N`. Raised count bounds to 100000 and the trust series limit to 256 so long play cannot hit them. Found and fixed a bug with a scratch run of 64 lives: guards re-evaluate per rule, so incrementing life before clearing `eaten` stopped the last regrowth; moved the cap into the growth time. Verified 64 lives then a rejected absorb, no tick refused, resume equal. 33/33, check clean.
- cr11: `$m_eaten` is now set to `qualified(1, <consuming evidence>)` (after `learn` reveals it) so it is grounded on what consumed the current life; `why_absorb`/`why_taste` bindings per mushroom, highest priority last, each citing the state that justifies it; adapter adds a 3-line `why` projection. 35/35, check clean.
- cr12: the runtime already saves and restores sessions (scheduled fades, renewals, journal included), and `save()` had been returning its text since base. Changed `save()` to return the parsed JSON value and `createPolicy(saved)` to stringify it back, because the size limit is measured on `JSON.stringify(save())` and a string doubles its escaping. Measured with a temporary stderr line in the adapter (removed): S36 resume points 2.5 KB / 3.5 KB raw (4.1 KB escaped as a string, just under the limit), S37 about 2.1 KB. The save grows with renewals and journal entries, so a much longer game with many regrowths would go over 4096 bytes; the visible scenarios stay under it. Parsing would lose a -0, but no state here can become -0. 38/38.

## Departure: files in the shared scratchpad (recorded at the coordinator's request)
While measuring save sizes (cr12 and cr13) I copied impl/adapter.mjs to
`adapter.bak` in the session scratchpad under /tmp/claude-0/..., added a temporary
debug line, and copied the backup back afterwards: three times in all (cr12, the cr13 S41
measurement, and a long-play measurement that I stopped before its restore ran).
That was the only thing I kept there. The coordinator said another author read
and overwrote a file of that name. After that notice I read the whole current
impl/adapter.mjs. Every line is my own; the only extra was my debug line, which I
removed by hand. Each earlier restore was followed by a test run that passed
with the code I expected. From then on all files stay in this directory, and I
do not read the scratchpad.
- cr13: belief memory is a shift register of six slots (`glow_k`/`dusk_k`, each `qualified(1, e)` or 0) in `proc remember`, called from `learn`; `supported`/`contradicted` are the slot sums, so their grounds are exactly the remembered evidence and every label, twice-bitten check and "Too risky untasted" reason follows unchanged. The first commit now uses `qualified(1, e)` (the one observation). Recovery counts only while trust is reopened and is reset otherwise, so its grounds stay bounded. Adapter: history and reopenedBy show only the last six entries (a display bound; the runtime journal keeps everything).
  Save: the runtime's save keeps all evidence history by design: 4.7 KB at S41 already, and about 395 KB (24 KB deflated) after a 64-cycle stress run. So `save()` now returns `{replay}`: the accepted events, deflated and base64-encoded, with runs of equal ticks counted. `createPolicy(saved)` replays them into a fresh session; the runtime is deterministic. Stress run of 64 regrowth cycles × 4 mushrooms (232 log entries): save 637 bytes, resumed view equal. Cost: resuming replays the whole game, so it takes time in proportion to play length (about 2.5 min for that stress run of about 47k events). Uncertainty: nothing in Caveat bounds its own history, so this bound comes from the host, not the language. 41/41, check clean.
- cr14: `event mark/unmark target kind mushroom`; per-mushroom `$m_marked` with routed rejects; absorb is rejected while marked, absorb and witness clear the mark, a `marked` binding, canAbsorb gains `$m_marked == 0`, and a "Marked to avoid" why binding (`because nothing`) placed between Too risky and Known duskcap. Adapter: translate mark/unmark and project `marked`. Marks go through the replay save automatically. 44/44, check clean.
- cr15: `entity pit`; inside the mushroom block `define $m_out_of_reach = $index == target.pit` (a constant per copy), used by three rejects (absorb, taste, mark; unmark is already refused because the pit can never be marked), by canAbsorb/canTaste, and by two "Out of reach" why bindings placed just before "Already eaten". No adapter change. 47/47, check clean.
- cr16: the journal is a second six-slot shift register (`journal_who_k`, `journal_cite_k`; `proc shift_journal`). `journal_cite` holds the entry kind as a value grounded on the evidence the entry cites: the observation, the new basis (`recovery` or the single observation), or the contradiction. `trust_change` records, in each commit/reopen step's guard, whether this observation changed trust. Text comes from pure `fn journal_text(kind, who)`; mushroom names are spelled out a second time in `fn mushroom_name` (no function turns an entity into text). First try failed: a binding citing a state its value did not read is refused as ungrounded_citation, so I folded the kind into the cited value. Lists inside the journal are compared in order. The adapter now lists evidence in observation order (position of its stance relation in `view.relations`) and caveats in the order their `qualifies` relations were added, instead of the explanation's sorted order. Uncertainty: this ordering is my reading of "in order" for caveats. 50/50, check clean.

## Summary
All 17 phases (base, cr1-cr16) were committed green. Final files: impl/glowcap.cav and impl/adapter.mjs.
The game rules, text, citations and caveats are in Caveat. The adapter renames events and payloads, projects bindings and
explanations, orders citations, maps `x@N` to `x_N`, shows only the last six journal entries for history and reopenedBy (`SHOWN`), and saves a
replay log, because the runtime save grows without bound. Scratch files were removed.
