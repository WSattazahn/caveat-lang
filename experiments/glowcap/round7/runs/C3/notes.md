# C3 notes

## References read
README.md (author instructions); package README.md, docs/AGENT_START.md, docs/REFERENCE.md,
docs/reference/docs/AI_AUTHORING.md, docs/WORKED_EXAMPLE.md, docs/README.md; specs: identifiers,
view, decision journal, repetition, routed repetition (first sections), typed parameters,
reactive 0.1/0.2/0.3, explanations 0.1/0.2; runtime/prelude.cav; lib/session.d.mts.

## Phase base
Program: one `for mushroom as $m routed by target` block holds per-mushroom evidence
(`absorb_<id>`, `taste_<id>`), states (consumed, taste result), rejects and bindings.
`support`/`contradiction` counts are built from `qualified(1, absorb_<id>)` so their grounds are
exactly the absorptions; `because` on bindings yields the `because`/supportedBy/contradictedBy
lists (adapter reads `binding_explanations`). Trust is a `decisions trust` series committed
`using support` (frozen grounds); each duskcap absorption reopens it. The adapter reads basis and
ordered reopenedBy from `decision_journal`. Checked with validate, check (no warnings), test 14/14,
and a scratch replay (scratch-base.jsonl) of two duskcap reopenings plus resume.
Ambiguity: "kind contradicts a previous taste" only matters for a sweet taste then duskcap
absorb (a bitter taste already forbids absorbing).

## Phase cr1
Added `entity grove kind mushroom at level;` — one line; the routed block and the adapter (which
reads the members from the absorb event's signature) pick it up. Test 15/15.

## Phase cr2
`define twice_bitten = contradiction >= 2`; an untasted mushroom gets the "Too risky" label
(because contradiction), `can_absorb` false, and absorbing it is rejected in the program. Known
(tasted) mushrooms keep their labels since those bindings come later. Check clean, test 17/17.

## Phase cr3
Absorb and taste now share `proc learn(e evidence, s, tasted)`: stance reveal on the general
claim, `qualify e with tasted_in_dark` for a taste while `glow == 0` (late qualification, before
the value is counted), count update, reopen on contradiction, first commit when probably_safe.
Dark labels use `has_caveat($m_taste, tasted_in_dark)`. Caveats come from binding explanations
(mushroom label; a new `belief.cited` binding citing both counts) and, for the decision, from
the journal's committed entry (frozen; later qualifications don't touch it). Dropped the unused
per-mushroom claim. Check clean, test 19/19, scratch replay with resume (scratch-cr3.jsonl).
Assumption: decision.caveats covers only the basis, not reopenedBy, per the request's wording.

## Phase cr4
The uncertain label's `because` now cites `contradiction` only; caveats follow automatically from
the binding explanation. Check clean, test 20/20.

## Phase cr5
Every taste schedules `qualify e with taste_faded after 60` (Renewal 0.1 scheduled qualification,
timed by `tick`). Faded labels test `has_caveat($m_taste, taste_faded)` and come after the dark
labels so they win. Decision caveats stay frozen because they come from the journal entry. Check
clean, test 22/22.

## Phase cr6
`decision.history` is the program's `decision_journal` entries for `trust`, projected to
`{change, because}` in journal order. Adapter-only change; no program change needed. Test 24/24.

## Phase cr7
New state `recovery`: each glowcap observation adds `qualified(1, e)`, each contradiction sets it
to 0 (a guard is control, so the grounds empty). While `reopened(trust)` and `recovery >= 2`,
`commit trust because enough using recovery` makes the next revision; its grounds, and so the
journal entry, are exactly the two observations in observation order. The adapter already took
basis/caveats/reopenedBy from the latest committed journal entry. Check clean, test 26/26.

## Phase cr8
Heavy timer bound raised to 30; a duskcap sets `heavy = if(heavy > 0, min(30, heavy + 20), 20)`.
`bind slime.heavy_seconds = ceil(heavy)` (0 when not heavy since the timer floors at 0). Adapter
maps it to `heavySeconds`. Check clean, test 27/27.

## Phase cr9
New `witness` event and per-mushroom `witness_<id>` evidence with a declared
`secondhand qualifies witness_$m`. Witness rejects only when consumed or when the kind differs
from a previous taste (`sort != $m_taste`; taste values share sort's positions), then consumes
the mushroom and calls the same `learn` procedure, so commit/reopen/recover follow. Timers are only
set by absorb rules. Adapter forwards `witness` like absorb/taste. Check clean, test 30/30.

## Phase cr10
Per mushroom: `renewable absorb_/taste_/witness_$m limit 64`, a `$m_life` counter (1..64) and a
45 s `$m_regrow` countdown set on consumption. A tick that brings it to 0 while lives remain calls
a per-mushroom `proc regrow_$m()` (declared inside the routed block) that renews all three
evidence names, forgets the taste and marks the mushroom present. Earlier occurrences stay in the
counts/grounds/journal and their scheduled fades still apply. The runtime names occurrences
`absorb_cave@2`; the adapter renames `@N` to `_N` (display only). Capacity: 64 lives per mushroom.
Checked: test 33/33; scratch-lives.jsonl replays 64 lives of `ruin` (all accepted, all ticks
accepted, the 65th absorb rejected as consumed, resume agrees).
Uncertainty: whether `renewable … limit N` counts occurrences or renewals — the replay shows 64
occurrences fit with limit 64, and my own `$m_life < 64` guard prevents any renewal_limit refusal.

## Phase cr11
`$m_gone` is now set to `qualified(1, absorb_$m|witness_$m)` after `learn` reveals it, so its
grounds are the evidence that consumed the current life (regrowth resets it to 0, ungrounded).
New bindings `$m.why_absorb` / `$m.why_taste` list the reasons in reverse priority (last match
wins) with `because` citing `$m_gone`, `$m_taste` or `contradiction`; the adapter reads value and
explanation into `why`. Check clean, test 35/35.

## Phase cr12
save/resume already existed (adapter returns the runtime's save text, a JSON string, and
`runtime.restore`s it). The first run failed one size check (S36: 4226 bytes > 4096): `$m_gone`
carried evidence, so every skipped tick guard reading it spread that lineage into other states,
which the save records. Split it into a plain `$m_gone` flag and a grounded `$m_eaten_by` that only
the why-bindings read. Test 38/38. To see the save I temporarily added a stderr print of the save
to the adapter for one `try` run (scratch-size.jsonl: 2609 and 3723 bytes), then restored the
adapter exactly; most of what remains is graph history (relations, renewal occurrences, journal),
which grows with observations and regrowths, so the 4096-byte bound holds for the scenarios given
but is not guaranteed for arbitrarily long eventful play. A `node run.mjs caveat serve` attempt to
read a save got no stdin through the runner (only the ready line); no effect.

## Phase cr13
Belief window: six `memory` entities and a plain `for memory as $k` block of slot states
(`$k_support`, `$k_contradiction`, each holding `qualified(1, e)` of the remembered observation or
0). `support`/`contradiction` became `define`s summing the slots, so every existing label, why,
twice-bitten and note rule follows the window unchanged. Per event: the routed mushroom block's
`learn` reveals/qualifies and sets `observation`/`observation_sort`; the memory block writes slot
`next_memory`; a second routed mushroom block calls `decide`, which advances the ring, counts
recovery (capped at the two it needs; counts remembered or not), reopens on a contradiction,
commits the first time the belief is probably_safe *using that one observation*, and recommits
on recovery. Taste/consumption state is per mushroom and untouched by forgetting. Trust series
limit raised to 256 (the runtime's maximum). The adapter shows the last six reopenings and
journal entries (presentation slice; the runtime journal itself is append-only).

Save size: the functional scenarios passed at once, but S41's raw runtime save reached
4210, then 4773 bytes. I tried plain shadow state for rule guards (to stop guard lineage
spreading into commitment bases); it helped little, because a decision-series revision's basis
includes its predecessor's basis and the runtime keeps every relation, renewal occurrence and
journal entry. I reverted that and instead **compress the save in the adapter** (raw deflate +
base64, node:zlib, one JSON string). All 41 scenarios pass. A long scratch run
(scratch-long.jsonl: 480 observations over ~46 minutes, resume every 10 cycles, all accepted,
every resume identical) measured raw/compressed saves of 69 KB/7.4 KB after 80 observations
growing to 1.16 MB/79 KB after 480 — roughly quadratic, since each trust revision's basis
(and its relies_on edges) includes all earlier ones. So the requirement that the 4096-byte bound
hold "for play of any length" is NOT met; it holds for the scenarios only. As far as I can tell
from the docs, Caveat's runtime has no way to forget archived evidence (by design: evidence
history is preserved), so a bounded save would need a capability the language does not
offer (or a host-side compact save, which would move game state out of the program). Other
long-play limits: 64 lives per mushroom; the trust series' 256 revisions (a 257th recommit would
be refused as history_limit, rejecting the observation).

## Phase cr14
`mark`/`unmark` events (routed by the mushroom) and a plain `$m_marked` flag per mushroom: mark
rejected when consumed or already marked, unmark when not marked; absorb rejected while marked;
witness clears the mark (absorb never reaches a marked mushroom); regrowth starts unmarked
because consumption cleared it. `can_absorb` and a new `marked` binding read it; the why-absorb
row "Marked to avoid" (`because nothing`) sits between "Too risky untasted" and "Known duskcap"
in the last-wins order. A mark touches no evidence, belief, decision or timer. Adapter forwards
`{id}` as `{target}`. Check clean, test 44/44.

## Phase cr15
Added `entity pit kind mushroom`; inside the routed block, `$index == target.pit` (the typed
parameter's member constant) marks the out-of-reach member: absorb, taste and mark of it are
rejected (unmark already fails as it is never marked), `can_absorb`/`can_taste` are false, and
"Out of reach" (`because nothing`) sits just below "Already eaten" in both why lists. Witness,
belief, regrowth and labels apply unchanged. Check clean, test 47/47.

## Phase cr16
Journal as a six-slot shift register in Caveat (`entity j1..j6 kind entry`, states `$j_code`,
`$j_cite`): `proc note(code)` shifts entries up and writes the newest into j6 citing
`note_value` (`qualified(1, e)` for observations and reopenings, `qualified(1, e)` or `recovery`
for commits, i.e. the new basis). `decide` now takes the action and mushroom index and notes the
observation, then a trust entry under the same guards as the reopen/commit it reports. Text is
composed in bindings from the code (mushroom name from `target.<name>` constants); the binding
condition also reads `$j_cite` so the citation is grounded (first run: ungrounded_citation refusals
for entries whose code carried no evidence). Because `cites $j_cite`, current caveats (e.g. a later
fade) show on old entries. Adapter lists shown slots j1..j6.
Ordering: the journal is compared in order, nested caveat lists included; the runtime reports
caveats sorted by name. Snapshot `symbols` turned out not to be in declaration order, so the
adapter now orders every caveats list by the position of the first `qualifies` relation (insertion
order) between that caveat and the cited evidence — acquisition order from the runtime's own
record. Check clean, test 50/50. Save bound: same situation as cr13 (passes the scenarios via the
compressed save; not bounded for arbitrary play).

## Wrap-up
All 17 phases (base, cr1–cr16) committed with every scenario passing (final: 50/50); none
committed incomplete. `caveat check` reported no warnings at every phase.
Known shortfalls: (1) the CR12/CR13/CR16 requirement that the 4096-byte save bound hold for play
of *any* length is not met: the runtime save keeps the whole evidence archive (relations, renewal
occurrences, journal, commitment bases, each series revision including its predecessor's basis),
so it grows roughly quadratically; compressing it in the adapter only lets the given scenarios
pass. (2) Hard caps: 64 lives per mushroom, 256 trust revisions (runtime maximum).
Process notes / departures: to measure save size I temporarily instrumented `impl/adapter.mjs`
(stderr print of the save) for a few `try` runs and restored it each time. One long `try` run was
moved to the background by the tool, which put its console output in a task file outside this
directory; I did not read that file (the read was refused) and re-ran the measurement in the
foreground with output inside this directory. Scratch files: scratch-*.jsonl.
