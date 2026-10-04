# Review of CR13–CR16 and scenarios S39–S50

Read: README.md, BEAT.md (base, CR1–CR12), existing-scenarios.mjs, REQUESTS.md,
scenarios-r7.mjs. Every r7 scenario was traced by hand against the
cumulative rules. No code was run.

## 1. Expectations follow from the requests and BEAT.md

No problems found.

All twelve scenarios trace to the stated expectations. Key traces (t = seconds
of ticks, "obsN" = Nth accepted observation):

- **S39.** obs1–2 `absorb_pool`, `absorb_ruin` (duskcap; twice bitten, so the
  untasted `absorb cave` is rejected). obs3–4 `taste_cave`, `taste_grove` in the
  dark. obs5–6 absorbs of the tasted cave and grove (allowed because the
  mushrooms are known); glow starts at t=0. At t=45, cave, grove, pool and ruin
  regrow and glow has ended. obs7 `taste_pool_2` (dark) forgets `absorb_pool`.
  That leaves 1 contradiction, so the belief is uncertain, cave's `because` is
  `[absorb_ruin]` (CR4) and absorb is allowed. obs8 `taste_ruin_2` forgets
  `absorb_ruin`, leaving 6 supports. The belief becomes probably_safe ("Based on
  6 observations.") and first commit has basis `[taste_ruin_2]` with caveat
  `tasted_in_dark`. No taste has faded by t=45.
- **S40.** Commit at obs1. obs2–6 are 5 contradictions; tastes made while glowing
  carry no caveat, so `reopenedBy` has 5 entries and `history` has 6. At t=45,
  cave, pool and ruin regrow (grove was never consumed). obs7 `witness_cave_2`
  forgets `absorb_cave`; `history` reaches 7 and drops the commit. obs8
  `witness_pool_2` forgets `taste_grove`; `reopenedBy` and `history` drop their
  oldest entries. The window holds 6 contradictions, so the belief is
  probably_unsafe with caveat `secondhand`. grove is still `Duskcap — avoid`
  with `[taste_grove]` and no caveat. Both rejects hold.
- **S41.** Commit on `taste_cave`; reopen on `taste_grove`. `absorb_cave` and
  `absorb_pool` are the 1st and 2nd glowcaps since then, so trust recommits.
  obs7 forgets `taste_cave`. obs8 `witness_grove` forgets `taste_pool` and
  reopens. The sets and the 4-entry history match.
- **S42–S44 (CR14).** The marked rows, `why` priorities, mark clearing on
  witness and survival across resume all match.
- **S45–S47 (CR15).** The pit's out-of-reach `why`, the consumed row, regrowth
  at 45 s (including across a resume at t=30), the CR4 counterexample on cave,
  and `witness_pit` + `taste_grove` (in the light) recovering trust with caveat
  `secondhand` all match.
- **S48.** Taste at t=31 in the dark, fade at t=91, cave regrows at 45. The
  journal's 4 entries pick up `taste_faded`; `decision.caveats` stays `[]`.
- **S49.** The 7th entry drops `I trust…`. After regrowth, `absorb_cave_2` drops
  one entry, and `absorb_pool_2` plus the recommit drops two. That gives the
  listed 6 in order.
- **S50.** Only 3 observations, all uncertain or unsafe, so no commit. Marks,
  regrowth and the rejected absorb add no entries. `taste_cave` (t=45) fades at
  t=105.

## 2. Ambiguity

No problems found beyond item 3.1. CR13 states when commit and forgetting are
checked ("after which the belief (with any forgetting it caused)"). It also says
CR7 counts accepted observations, remembered or not. CR14 and CR15 place their
`why` rows exactly and say how each interacts with consumption and witnessing.
CR16 states the entry order, which events write entries, and what is dropped
when an event adds two entries.

## 3. Bounds

1. **CR13 (last bullet) relies on CR10's life capacity, whose behaviour at the
   limit is ambiguous.** CR13 says the 4096-byte save bound "now holds for play
   of any length". Evidence ids carry life numbers (up to the declared capacity
   of at least 64 lives per mushroom, now including `pit`). CR10 says "an event
   that would exceed it is rejected". It does not say which event that is once
   a mushroom's last life is consumed:
   - (a) the consuming `absorb`/`witness` of the last life, or
   - (b) the `tick` in which life N+1 would regrow.

   Reading (b) rejects every later tick, which freezes the glow, heavy, fade and
   regrowth timers for good. Reading (a) leaves time running. Two implementations
   would accept different events in long play. CR13's "any length" claim is the
   first to depend on this, and no scenario reaches the limit.
   *Smallest fix:* add one sentence to CR13 (or CR10): "At capacity the
   mushroom does not regrow; ticks are never rejected for this reason, and an
   `absorb`/`witness` of a mushroom with no remaining lives is rejected as
   consumed."

## 4. Conflicts with earlier requests or scenarios still in force

No problems found. I checked every existing scenario that applies through cr12
(no `until`) against CR13–CR16:

- None has more than six observations. S33 has exactly six, and S25, S26 and
  S29 have five. So forgetting never triggers.
- None has more than six `history` or `reopenedBy` entries.
- All mushroom expectations are partial, so the added `pit` and `marked` fields
  do not break them.
- None uses `mark`, `unmark` or `journal`.

So REQUESTS.md's claim "No existing scenario's expectation changes" holds. The
overrides of CR1 ("every rule applies") for `pit` and of CR2's `canTaste` for
`pit` are explicit.

## 5. Format

1. **`PHASES` does not include cr13–cr16.** `existing-scenarios.mjs` exports
   `PHASES` ending at `'cr12'`, and `applies()` uses `PHASES.indexOf(since)`.
   For `since: 'cr13'` … `'cr16'`, that returns -1, so `applies()` returns true
   for every phase from `base` onward. S39–S50 would then run against the base
   policy. The phases cr13–cr16 also cannot be selected at all. scenarios-r7.mjs
   neither exports nor extends the phase list.
   *Smallest fix:* add `'cr13', 'cr14', 'cr15', 'cr16'` to `PHASES` (or export
   the extended list from the new module), and have the harness treat an unknown
   `since` as an error.

These passed: the module imports nothing; ids run S39–S50 continuing from S38;
every `since` is cr13–cr16; existing steps use the documented forms.

Note (not counted): `['mark', { id }]` and `['unmark', { id }]` are documented
only in the r7 file header. They take an object payload, unlike the positional
`['absorb', id, kind]` and `['witness', id, kind]`. In-order comparison of
`journal` is likewise documented only there; the existing header says "lists are
sets". The harness needs both the new step dispatch and ordered comparison for
`journal`. `['mark', id]` would match the existing convention.

## Totals

| Check | Problems |
| --- | --- |
| 1. Expectations | 0 |
| 2. Ambiguity | 0 |
| 3. Bounds | 1 |
| 4. Conflicts | 0 |
| 5. Format | 1 |
