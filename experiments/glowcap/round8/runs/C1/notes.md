# Notes (C1)

## References read
- README.md, phases/*.md, scenarios/current.json.
- node_modules/caveat-lang: README-level docs AGENT_START.md, REFERENCE.md, WORKED_EXAMPLE.md, STARTER.md,
  docs/reference/docs/AI_AUTHORING.md; specs: typed-parameters, repetition, routed-repetition,
  reactive-0.1/0.2, decision-journal, reopening-triggers, explanations-0.1/0.2, define, view-0.1/0.2;
  lib/session.d.mts, lib/node.d.mts.

## Phase base
Design: `glowcap.cav` holds all rules. Mushrooms are entities of kind `mushroom`; a routed `for` block
gives each its evidence (`absorb_<id>`, `taste_<id>`), consumed/taste state, rejects, and HUD bindings.
`support`/`contradiction` counters get each absorption's evidence as grounds via `qualified(1, absorb_$m)`,
so `because support` on bindings yields the evidence lists (binding_explanations). `trust` is a decision
series committed once (`not committed(trust)`), reopened by an authored `reopen` per duskcap so each
duskcap is journaled. The adapter only renames payload fields (id->target, kind->sort) and projects
bindings, binding explanations and the decision journal (current revision) into the view.
Checked with `node run.mjs check` (no warnings), `caveat validate`, and `node run.mjs test` (14/14).
Ambiguity: absorbing a mushroom tasted sweet with kind duskcap is rejected ("contradicts a previous taste").

## Phase cr1
Added one `entity grove kind mushroom at level;` line; the routed block and the adapter (which reads the
entity list from the snapshot) pick it up. check clean, test 15/15.

## Phase cr2
Added `define $m_too_risky` (unknown and contradiction >= 2), folded into `$m_can_absorb` (so the same
define drives both the absorb refusal and canAbsorb) and a later-winning label binding citing
`contradiction`. check clean, test 17/17.

## Phase cr3
Moved the shared "observation bears on the belief" steps into `proc observe(e evidence, found, dark)`
(reveal stance on glow_safe, `qualify e with tasted_in_dark` when dark, count into support/contradiction,
commit/reopen trust), called by absorb and taste. Dark = `glow` is 0 at taste time. Caveats come from the
runtime: label/belief caveats are the binding explanations' caveats; decision caveats are the frozen
journal `committed` entry's caveats. Dark-tasted labels use `has_caveat($m_taste, tasted_in_dark)`.
Dropped the per-mushroom claim (taste now bears on glow_safe). check clean, test 19/19.

## Phase cr4
One-word change: the uncertain label binding cites `contradiction` only; caveats follow from the
explanation automatically. check clean, test 20/20.

## Phase cr5
Added `caveat taste_faded` and `on taste qualify taste_$m with taste_faded after 60;` (scheduled late
qualification, docs: late-qualification, renewal specs). Faded labels are later bindings guarded by
`has_caveat($m_taste, taste_faded)` so they win over the dark ones. The decision's caveats already come
from the frozen journal entry, so it is unaffected. check clean, test 22/22.

## Phase cr6
No program change: `decision.history` is the runtime's decision journal entries for `trust`, projected
to `{change, because}` in journal order by the adapter. test 24/24.

## Phase cr7
Added state `recovery` (grounds = glowcap observations counted while trust is reopened; reset to 0 by
each contradiction) and, in `observe`, a recommit `commit trust because enough using recovery` when it
reaches 2, so the new revision's grounds are exactly those two observations (journal gives them in
observation order). Raised the series limit to 256 so recommits cannot hit history_limit in practice.
Adapter unchanged (basis/reopenedBy/caveats already follow the current revision). check clean, test 26/26.

## Phase cr8
Heavy timer max raised to 30; duskcap sets `if(heaviness > 0, min(30, heaviness + 20), 20)`; new
binding `slime.heavy_seconds = ceil(heaviness)`, mapped to `heavySeconds`. check clean, test 27/27.

## Phase cr9
New event `witness` (adapter maps it like absorb/taste), per-mushroom evidence `witness_$m` with a
declared `secondhand qualifies witness_$m;`, and witness rules in the routed block: reject consumed /
contradicting-taste, `call observe(witness_$m, sort, 0)`, mark consumed; timers are only touched by the
top-level absorb rules. Generalised the taste-contradiction check into `define $m_contradicts_taste =
$m_taste != 0 and sort != $m_taste` (taste state holds the tasted sort), shared by absorb and witness.
Decision: witnessing a bitter-tasted mushroom as a duskcap is allowed (only contradiction is refused).
check clean, test 30/30.

## Phase cr10
Read the renewal spec. absorb/taste/witness evidence are `renewable … limit 64` (64 lives). Per mushroom:
`$m_regrow` (45 s, counted down by ticks once consumed), `$m_life`, and a per-tick flag `$m_sprouting`
that, when set, renews the three evidences, clears taste and consumed and increments life. Old taste
fades still apply via the scheduled qualification on the old occurrence. The adapter renames runtime
occurrence ids `absorb_cave@2` -> `absorb_cave_2` (naming only).
Bug found with my own replay (`node run.mjs try scratch_lives.jsonl`, 64 lives + resume): my first
version guarded each regrow step by a define that read `$m_life`, so the step incrementing life to 64
disabled the following "not consumed" step and life 64 never grew. Fixed by deciding once into
`$m_sprouting`. Verified: 64 absorptions succeed, the 65th is refused, no tick refused, resume gives the
same view. check clean, test 33/33.

## Phase cr11
`$m_consumed` is now set to `qualified(1, absorb_$m)` / `qualified(1, witness_$m)` so it is grounded on
what consumed the current life (regrowth sets it to plain 0). New bindings `$m.why_absorb` /
`$m.why_taste` with `because` citations (`$m_consumed`, `$m_taste`, `contradiction`), ordered so the
table's first match is the last-winning binding; the adapter maps them plus their explanations into
`why`. check clean, test 35/35.

## Phase cr12
First run: S36 save 4593 bytes > 4096. Investigated by temporarily dumping the runtime save from the
adapter (env var, removed afterwards). Two changes:
1. Program: removed the `$m_consumed` and `$m_sprouting` states. `$m_regrow` (set to
   `qualified(45, absorb_$m|witness_$m)`) is now the consumed marker (`define $m_consumed = $m_regrow > 0`)
   and carries the "Already eaten" grounds; a life sprouts when its timer is 0 while its absorb or
   witness occurrence is still observed (renewal makes the new occurrences unobserved), with the three
   renews in `proc new_life`. The last life's timer is never run down. (A variant that tested
   `regrow <= dt` instead of `observed()` produced a larger save, reverted.)
2. Adapter: `save()` returns the runtime save parsed as a JSON value instead of the string (stringifying
   a string escaped every quote, ~15% more), and `createPolicy(saved)` restores `JSON.stringify(saved)`.
Verified: test 38/38; a 400-event random game replayed with a save/resume after every event gives the
identical output to the uninterrupted run (scratch_mix_*.jsonl); 64-lives replay still OK.
Uncertainty: the save still grows with the number of observations (belief grounds, journal, renewals);
the 4096-byte bound is only checked on the given scenarios.

## Phase cr13
Read windows, departure and lineage-compaction specs and docs/HISTORY.md.
- Belief window: `support`/`contradiction` are now defines summing twelve slot states (g1..g6, d1..d6):
  each observation shifts the slots and puts `qualified(1, e)` in g6 or d6 (the other gets a plain 0),
  so the grounds of the sums are exactly the six remembered observations. Everything citing the belief
  (labels, too risky, why) follows automatically. The first-commit rule is evaluated after the shift, so
  an observation that pushes out the last remembered contradiction commits. CR7's `recovery` counts all
  accepted observations, as before.
- `renewable … window 2` replaces `limit 64` (lives are now unlimited; no `$m_life` state). Window 2 keeps
  the current and previous life live; a taste fades (60 s) before two regrowths (>= 90 s) can retire it.
- `journal window 6` bounds the journal; the adapter's history is the live journal; reopenedBy is the
  live journal's reopenings of the current commitment (always a suffix of the journal); basis/caveats now
  come from `commitment_grounds[current]` (exact and frozen even after its journal entry departs).
- Save size: S41's save was 4450 bytes as JSON. The adapter now stores the runtime's save text
  deflate-compressed and base64-encoded (node:zlib, built in) inside `{caveatSave}`. This is a host
  storage encoding, not a game decision, but it is a departure worth flagging: without it S41 fails.
- Verified: test 41/41; random 400-event game with save/resume after every event identical to the
  uninterrupted run; a 40-cycle long play (~30 min, ~30k events) resumes with the same view each time.
- Known limitation (runtime): a decision series cannot have a window (`decisions trust window 6` is
  refused at load: "a decision series has no window yet"), and its limit is at most 256. Every trust
  revision keeps its basis/reopening qualifications in the save, so the save still grows with the number
  of recommits: in my long play the compressed save went 3.7 KB -> 32 KB over 40 regrowth cycles
  (27 trust revisions). The 4096-byte bound therefore holds for the scenarios but NOT for play of any
  length, and after 256 recommits an observation would be refused as limit/history_limit. I found no
  way to express a bounded recommitting decision in this runtime.

## Phase cr14
New events `mark`/`unmark` (`target kind mushroom`; the adapter maps `id`), per-mushroom state
`$m_marked` (plain number, no evidence), folded into `$m_can_absorb` (so absorb is refused and canAbsorb
is false), cleared when a witness consumes it; mark refused on a consumed or marked mushroom, unmark on
an unmarked one. New `$m.marked` binding and a "Marked to avoid" why_absorb binding placed between
"Too risky untasted" and "Known duskcap" in last-wins order (`because nothing`). check clean, test 44/44.

## Phase cr15
Added `entity pit` and, in the routed block, `define $m_out_of_reach = $index == target.pit;` (true only
in pit's copy). It is folded into `$m_can_absorb`/`$m_can_taste` (so absorb and taste are refused and
the view flags are false), refuses `mark` (so `unmark` is refused as not marked), and adds "Out of reach"
why bindings just below "Already eaten" in last-wins order. Labels need no change (pit is never tasted).
Adapter unchanged (it reads entities from the program). check clean, test 47/47.

## Phase cr16
The journal lives in the program: six slot states j1..j6 shifted by `proc note(entry)`; an entry's
value is a text code and its grounds are what it cites, so its caveats are the cited evidence's current
caveats (a fade updates old entries; nothing else writes them). `observe` now also takes the action and
the mushroom's `$index` and notes `qualified(code, e)` first, then a trust entry where it commits
(`qualified(100, e)`, or `100 + 0 * recovery` for a recovery commit, the `0 *` being there only to ground
the entry on the new basis) or reopens (`qualified(200, e)`). Pure functions `mushroom_name`,
`observation_text` and `entry_text` give the texts (the mushroom names are written there once more, a
duplication of the entity list I accepted). Bindings `journal.textK` / `journal.citesK`; the adapter
lists the non-empty slots oldest first. check clean, test 50/50; random save/resume comparison identical.

## Phase cr17
`slime.glow_seconds = ceil(glow)`, `$m.regrows_in = ceil($m_regrow)` (the regrowth countdown already
exists and is 0 while present; with unlimited lives there is no "last life" case). For `fadesIn` the
runtime's scheduled qualification is not readable from source, so I added one per-mushroom state,
`$m_tasted_at = elapsed()` at taste time, and show `ceil(fade_seconds - (elapsed() - $m_tasted_at))`
while present, tasted and not yet faded (`has_caveat($m_taste, taste_faded)` false). This departs from
"nothing new is stored" in the request; it computes with the same binary64 subtraction the scheduler
uses, so it reaches 0 on the event the fade lands. The 60 s now lives in `define fade_seconds`, used by
both the `qualify … after` and the countdown. check clean, test 53/53; random save/resume comparison
identical.

## Phase cr18
- `doubt` (program: `act in absorb taste witness, target kind mushroom, life`): the adapter only splits
  the evidence id string (`witness_pool_2` -> witness/pool/2; no life suffix = 1); a string that does not
  split is passed through and the runtime refuses it as payload_invalid. The program decides the rest.
- To find a remembered witness, belief slots now hold an observation code instead of 1:
  `(life * 10 + who) * 10 + kind` (kinds 1-6 as in the journal texts), grounded on the evidence; counts
  are `min(1, slot)` sums. `$m_life` is back (unbounded) to give the life number. Journal codes use the
  same scheme, with kinds 9/0 (trusted, caused by a witness / otherwise) and 7/8 (stopped trusting).
- Doubt: refused unless act is witness and its code is in a slot (so "already doubted" = no longer in a
  slot), and refused if no undoubted remembered observation would be left. `proc undoubt` zeroes the
  slot (the doubted witness keeps its place and shifts out in turn).
- The caveat: if the doubted witness is its mushroom's current occurrence, `qualify witness_$m with
  doubted` (late qualification, so every current value citing it gains it; frozen decision grounds do
  not). If the mushroom has regrown since, `qualify` cannot name the old occurrence, so undoubt adds the
  caveat to the journal entries about that witness (its observation entry and a trust entry it caused)
  with `+ doubt_mark`, where `doubt_mark = if(examined(doubted), 0, 0)`: reading `examined(doubted)`
  makes the value carry the caveat. This is a workaround, and it leaves one gap: a "trust now" journal
  entry whose two-observation basis includes a doubted older-life witness that did not trigger it does
  not get `doubted`. Checked by hand with scratch_doubt.jsonl (doubt after regrowth: journal entries show
  doubted+secondhand, belief drops it, decision unchanged, repeat doubt refused, resume identical).
- check clean, test 56/56; random save/resume comparison identical.

## Phase cr19
`retaste` event (adapter maps like taste) and per-mushroom renewable `retaste_$m` (window 2, renewed in
`new_life`). A retaste is an `observe` call with act 4 (kinds 7/8, new journal texts); then
`$m_taste = qualified(sort, retaste_$m)`, so labels, because, caveats, why, contradiction checks and
the fade countdown follow the retaste with no other change; it gets its own `qualify … after
fade_seconds`. `$m_can_retaste` = present, tasted, `$m_faded`, `not observed(retaste_$m)` (one per
life, no extra state), reachable; it drives both the refusal and `canRetaste`. Codes widened to
`identity * 100 + kind` to fit kinds 1-8 and trust kinds 10-13. check clean, test 59/59; random
save/resume comparison identical; doubt scratch still shows doubted.

## Phase cr20
Re-read cr18/cr20 and scenarios S60-S62. A doubt must now be reversible, and a `qualify` cannot be
removed, so I replaced it: no doubt touches the evidence any more. Every value that cites a doubted
witness also cites `doubt_mark` (the cr18 `examined(doubted)` trick), controlled by 0/1 flags:
- `x1..x6` beside the belief slots (shifted with them, a new observation enters undoubted). The slot keeps
  the witness's code; `support`/`contradiction` add `if(xk == 1, 0, min(1, gk))`, so (lazy `if`
  provenance) a doubted slot cites nothing.
- `y1..y6` beside the journal slots (shifted in `note(entry, doubted)`); `journal.citesK` binds
  `jK + if(yK == 1, doubt_mark, 0)`.
- `$m_doubted_consumer` per mushroom: set by a doubt of this life's consuming witness, cleared by its
  undoubt and on sprouting; "Already eaten" cites `$m_regrow + if(flag, doubt_mark, 0)` (also its
  condition, for the lineage rule).
`proc set_doubt(wg, wd, from, to)` flips the slot flag, the journal flags of entries about that witness
(observation, trust change it caused, its doubt/undoubt entries: kinds 5/6/11/13/14/15), then writes
the journal entry (kind 14 "I doubt…", 15 "I believe … again"), grounded on the witness through
`0 * (gk + dk)` of its slot. `doubt` needs a matching slot with x = 0 (and the cr18 non-empty check);
`undoubt` (new event, adapter splits its evidence id like doubt's) needs one with x = 1, so a forgotten
doubt cannot be taken back. Neither touches recovery, timers or the decision.
- Consequence: this also removes cr18's distinction between the current occurrence and an older-life
  one; both now go through flags. The cr18 gap remains: a recovery "trust now" entry whose
  two-observation basis includes a doubted witness that did not trigger it does not show `doubted`.
- Caveat order within a list comes from the runtime's explanation (sometimes `doubted` before
  `secondhand`); the scenarios pass, so I left the adapter's order as given.
- check clean, test 62/62. New random game with doubts/undoubts (scratch_mix20_*.jsonl, 700 events,
  103 undoubts of which several accepted): resume after every event gives output identical to the
  uninterrupted run. scratch_cr20_lives.jsonl: doubting witness_pool (older life) and witness_pool_2,
  then undoubting witness_pool, shows the right caveats on each journal entry and on pool's `why`.
