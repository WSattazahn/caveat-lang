# C1 notes

## References read
README.md (author instructions); package README.md, docs/AGENT_START.md, docs/REFERENCE.md,
docs/reference/docs/AI_AUTHORING.md, docs/WORKED_EXAMPLE.md, docs/README.md; specs: identifiers,
view, repetition, routed repetition (first part), typed parameters, reactive 0.1-0.5,
explanations 0.1/0.2, decision journal, reopening triggers, save (start), check (headings);
lib/session.d.mts, lib/node.d.mts.

## Phase base
Program `impl/glowcap.cav`: mushrooms are `mushroom` entities; one routed `for` block per
mushroom holds its evidence (`absorb_<id>`, `taste_<id>`), state, rejects and label bindings.
Belief counts `support`/`contra` are sums of `qualified(1, absorb_<id>)`, so their grounds are
exactly the supporting/contradicting absorptions; `because` lists in the view come from the
runtime's `binding_explanations`. Decision `trust` is a decision series committed `using support`
(frozen grounds = basis); every duskcap absorption after commit reopens it (authored reopen);
reopenedBy = journal reopened entries of the current revision. Adapter only translates events
and projects bindings/explanations/grounds/journal. Checked with `run.mjs check` (no warnings)
and `run.mjs test` (14/14 on first run).
Decisions/ambiguities: `save()` returns the runtime's save text (resume already supported).

## Phase cr1
Added one `entity grove kind mushroom` line; the routed block and adapter (which reads the members from the program) cover it. 15/15 on first run.

## Phase cr2
Added `twice_bitten` (contra >= 2) and per-mushroom `unknown` defines; canAbsorb and the absorb reject share one define, and a "Too risky" label cites `contra`. Interpreted "unknown" as present and untasted. 17/17.

## Phase cr3
Absorb and taste now share `proc learn(e evidence, k)` (reveal stance, count, reopen) and `proc reconsider()` (first commit). A dark taste (glow == 0 before the taste) runs `qualify taste_<id> with tasted_in_dark` before the commit, so the caveat reaches counts, labels and a commitment made in the same event; earlier commitment grounds keep their own caveats. Dark labels use `has_caveat($m_taste, tasted_in_dark)`. Adapter adds `caveats` from binding explanations / commitment grounds. 19/19.

## Phase cr4
The mushroom "Could be a duskcap" label now cites `contra` only (caveats follow from the explanation). Belief bindings unchanged. 20/20.

## Phase cr5
Each taste schedules `qualify taste_<id> with taste_faded after 60` (session clock advanced by tick); faded labels test `has_caveat($m_taste, taste_faded)` and come after the dark labels so they win. Commitment grounds stay frozen. 22/22.

## Phase cr6
`decision.history` is the runtime decision journal for `trust`, projected to {change, because}; no program change. 24/24.

## Phase cr7
New state `recovery`: sum of qualified(1, e) over glowcap observations, reset to 0 by a contradiction; `reconsider` commits a new `trust` revision `using recovery` when reopened and recovery >= 2, so the new grounds are exactly those two observations. Interpretation: the count runs regardless of decision state, but only matters while reopened. 26/26.

## Phase cr8
Heavy timer bound raised to 30; a duskcap sets `heavy = if(heavy > 0, min(heavy + 20, 30), 20)`; `slime.heavySeconds = ceil(heavy)` is a binding. 27/27.

## Phase cr9
New `witness` event and per-mushroom `witness_<id>` evidence with declared `secondhand qualifies witness_<id>`; it consumes the mushroom, rejects on consumed or taste contradiction (not on risk), and calls the shared `learn`/`reconsider` procs; timers untouched. Adapter maps the event. 30/30.

## Phase cr10
absorb/taste/witness evidence per mushroom is `renewable ... limit 64` (64 lives incl. the first). Consumption records `grows_at = elapsed() + 45` (or 1e12 on the last life, so no renew past the limit and no tick is rejected); a tick at/after it renews the three evidences and resets taste/consumed and bumps the life counter. Adapter renames runtime occurrence ids `x@N` to the required `x_N`. Old tastes fade on their own scheduled qualification. Checked with tests (33/33) and a scratch replay (`scratch-lives.jsonl`) of 64 lives: the 65th absorb is rejected, ticks go on, resume gives the same view. Note: the renewable limit 64 is written as a literal in the declarations and repeated as `define lives = 64`.

## Phase cr11
New per-mushroom state `eaten` = qualified(1, the consuming absorb/witness evidence), reset on regrowth; `why.absorb` / `why.taste` bindings with first-match order written last-wins, each citing `eaten`, `$m_taste` or `contra`. The "Already eaten" guard reads `eaten` so the citation is grounded. Adapter projects reason/because/caveats from bindings and explanations. 35/35.

## Phase cr12
No change: `save()` already returned the runtime save text (a JSON string value) and `createPolicy(saved)` restores it with the same source. 38/38, including the harness size and resume checks. I did not measure save sizes separately (only through the runner).

## Phase cr13
Memory of six: slot states s1..s6 / c1..c6 shifted on every accepted observation inside `learn`;
a remembered glowcap sits in s_N as qualified(1, e), a duskcap in c_N, so `support`/`contra`
(now defines summing the slots) are grounded on exactly the remembered observations and labels,
twice-bitten, notes and caveats follow. First commit is `using s6` (the observation that made the
belief safe). CR7 recovery now keeps only the last two glowcaps since a contradiction (r1, r2),
recommit `using r1 + r2`. Decision series limit raised to 256 (max) so recommits are not refused.
reopenedBy/history: adapter shows the six most recent journal entries (display bound only; the
runtime journal itself is append-only and keeps everything).
Save size: the runtime save holds the full evidence/decision history and was ~4.8 KB after 7
observations (5.5 KB once JSON.stringify escaped it), so `save()` now returns the exact runtime
save text brotli-compressed and base64-encoded (`{brotli: ...}`, node:zlib builtin); restore
decompresses it. 41/41. UNRESOLVED: the "any length" bound does not hold in principle. A scratch
replay of maximal play (all 64 lives of all 4 mushrooms, `scratch-long.jsonl`) gave a 588 KB raw
save, ~21.8 KB compressed. Caveat keeps every observation, relation, revision and journal entry
(by design), and a commitment-series revision's lineage includes its predecessor's, so the save
grows with play; I found no language facility to drop history. Only the released scenarios are
within 4096 bytes.

## Departure / incident
While measuring save sizes I used a file `adapter.bak` in the session scratchpad
(/tmp/claude-0/.../scratchpad), which turned out to be shared with other agents: when I copied
it back into impl/adapter.mjs it contained another author's adapter (I saw its source and ran one
test with it, 10/41). I discarded it and rewrote my own adapter from my own earlier version; my
earlier `cp` may also have overwritten another author's `adapter.bak` there. Debug save dumps
(save.json, long.json) were also written to that scratchpad. The temporary save-dump hook in the
adapter was removed. Scratch event files (scratch-*.jsonl) are in this directory.

## Phase cr14
New `mark`/`unmark` events and per-mushroom `marked` state (plain number, no evidence): rejects for consumed/already marked/not marked, cleared on absorb/witness, part of the `can_absorb` define (so absorb is rejected and canAbsorb false), and a "Marked to avoid" why-row (`because nothing`) placed between Known duskcap and Too risky in first-match order. 44/44.

## Phase cr15
Added `entity pit` and a per-member `out_of_reach` define (`$index == target.pit`) that enters can_absorb/can_taste (so absorb/taste are rejected and greyed), rejects `mark` (so `unmark` is rejected as not marked), and adds "Out of reach" why-rows (`because nothing`) just before "Already eaten" in last-wins order. Witness, regrowth and labels need no change. 47/47.

## Phase cr16
Journal kept in Caveat: six slot states j1..j6 shifted by `proc write(entry)`; each entry is a numeric code (10 * mushroom index + observation kind 1..6, or 7/8 for trust changes) grounded on the evidence it cites (`qualified(code, e)`, or `s6 * 0 + 7` / `(r1 + r2) * 0 + 7` so a commit entry carries the new basis). `learn` writes the observation entry, then the reopen entry after a reopen; `reconsider` writes the commit entry under the same guard just before committing. Texts come from bindings (pure fns for prefix/suffix, the mushroom name from the routed block), `because`/`caveats` from explanations, so a later fade shows. Departure to note: pure functions cannot read defines, so `trust_text` repeats the literal 7. The journal is compared in order, including the caveats lists, which the runtime gives as sorted sets; the adapter orders caveats by the order of the `qualifies` relations on the cited evidence in the view (insertion order). 50/50.

## Summary
All 17 phases (base, cr1-cr16) committed as complete; every phase's scenarios passed (final run 50/50).
`run.mjs check` reported no warnings at every phase. Known limits: the 4096-byte save bound holds
only through compression and only for the released scenarios, not for play of any length (see cr13).
The journal/reopenedBy/history six-entry bounds are applied at display time (adapter slices the
runtime journal) or by slot states in the program (journal, memory); the runtime's own history keeps
everything.
