# Review of CR17–CR20 and scenarios S51–S62

I traced all twelve scenarios by hand against BEAT.md (CR1–CR16) and
REQUESTS.md (CR17–CR20). The new requests do not conflict with the existing
scenarios. Times below are seconds since the scenario started. "obs n" means
the n-th accepted observation.

## 1. Every expectation follows from the requests and BEAT.md

No problems found. Every expectation matches my trace. The checkpoints most
likely to go wrong:

- **S51.** At 0.5 s: glow 29.5 shows 30, cave regrowth 44.5 shows 45, pool fade 59.5 shows 60.
  At 29.5 s: glow 0.5 shows 1, cave 15.5 shows 16, pool 30.5 shows 31.
  At 30 s: glow 0 shows 0 and the slime is not glowing; cave shows 15 and pool shows 30.
  At 45 s cave regrows (S31 shows that regrowth happens exactly at 45 s); pool shows 15.
  At 60 s the pool taste fades. It was made in the light, so the label is `(taste has faded)` and `fadesIn` is 0.
- **S52.** At 10 s glow and both regrowths show 20/35/35, and the resume keeps them.
  After the pool duskcap and one tick: heavy 19.9375 shows 20, glow 19.9375 shows 20,
  pool regrowth 44.9375 shows 45, pit 34.9375 shows 35.
  After 35 s more (45.0625 s): pit and cave have regrown, glow and heavy are at 0, pool regrowth 9.9375 shows 10.
- **S53.** `taste_ruin` (in the dark) shows 60, then 40 at 20 s. The witness at 20 s matches the taste, so it is accepted.
  The ruin is consumed, so `fadesIn` is 0 and `regrowsIn` is 45. It regrows at 65 s. Two remembered contradictions
  (`taste_ruin`, `witness_ruin`) make it `Too risky — taste first`. `taste_ruin_2` (dark) shows 60, then 1 at 59.5 s and 0 at 60 s.
- **S54.** Doubting `witness_ruin` leaves supportedBy=[absorb_cave] and contradictedBy=[witness_pool].
  The belief is uncertain, the grove follows CR4 (`because` [witness_pool], caveats [secondhand]), and the decision is unchanged.
  Doubting `witness_pool` is accepted because `absorb_cave` stays undoubted; the belief becomes `probably_safe` with N=1.
  `absorb_grove` is the first glowcap since `witness_ruin`. A doubted contradiction still counts for CR7, so there is no recommit.
- **S55.** The first doubt is rejected because there is no other observation. After `witness_cave` is doubted, a doubt of `witness_pool` is rejected.
  The decision keeps its frozen caveats [secondhand]. All four mushrooms regrow at 45 s.
  Obs 7 forgets `witness_cave` and obs 8 forgets `witness_pool`; the remaining six absorptions match the expectation.
- **S56.** The doubt makes the belief `probably_safe`, but trust is not committed (CR18).
  `absorb_pool` commits it with basis [absorb_pool]. The pit regrows as `Probably a glowcap`.
  `witness_pit_2` reopens trust; the belief is uncertain, because `witness_pit` is still doubted.
- **S57.** `taste_cave` is in the dark and commits trust. It fades at 60 s.
  `absorb_ruin` at 60 s makes the slime glow, so `retaste_cave` is in the light: label `Glowcap`, caveats [], `fadesIn` 60.
  The belief caveats come from `taste_cave` (dark and faded). The journal has four entries, and the trust entry shows the current caveats.
  At 120 s the retaste fades.
- **S58.** `taste_pool` is in the light, then marked. It fades at 60 s; the glow ended at 30 s and cave regrew at 45 s.
  `retaste_pool` is in the dark. `Known duskcap` comes before `Marked to avoid`.
  Two remembered contradictions make the regrown cave `Too risky — taste first`.
  A second reopen appends to `reopenedBy` and to the history (as in S23). The journal has exactly six entries. The glowcap witness of pool is rejected.
- **S59.** `taste_ruin` is in the light at 0 s and fades at 60 s; the glow ended at 30 s, so `retaste_ruin` is in the dark.
  It is the second glowcap since `absorb_pool`, so trust is recommitted with basis [taste_ruin, retaste_ruin].
  The frozen caveats are [taste_faded, tasted_in_dark]. After `absorb_ruin` and its regrowth the belief is uncertain
  with one contradiction, so the label is `Could be a duskcap` with `because` [absorb_pool].
- **S60–S62.** I replayed the journals entry by entry, dropping the oldest entry when a seventh arrives.
  Every listed order and every current caveat set (`doubted` present or absent) matches.
  In S62, `witness_pit` is the second glowcap since `witness_ruin`. The undoubt neither reset nor added to the CR7 count,
  so trust is recommitted with basis [absorb_grove, witness_pit] and caveats [secondhand].

## 2. Ambiguity

1. **CR18, "A doubt is not an observation" bullet.** The bullet says "It still occupies its place among the six
   remembered observations and is forgotten in its turn". Grammatically, "It" refers to *the doubt*. Read that way,
   a `doubt` would take a slot in the six-observation window and push out the oldest observation. That contradicts
   "is not an observation" and "forgets nothing", but an implementer could still read it so. The intent is that the
   *doubted witness* keeps its slot. S55 (obs 7 forgets `witness_cave`) fixes the intent for the tested case.
   *Fix:* "The doubted witness still occupies its place among the six remembered observations and is forgotten in its turn."

## 3. Bounds

1. **CR18, closing bound statement.** It says that "once forgotten, a doubted witness can only still be cited by the
   journal … or by its consumed mushroom's `why`". This is not true: the decision's `basis`, `reopenedBy` and
   `history` can still cite it. S61 expects `basis: ['witness_cave']` after `witness_cave` is both doubted and forgotten.
   The bound itself still holds, because the decision's caveats are frozen and `reopenedBy`/`history` carry no caveats,
   so the decision never needs the doubt flag. The stated reasoning is wrong, though, and an implementer could rely on it.
   *Fix:* append "(the decision may also cite it, but shows no current caveats, so it needs no doubt flag)".

## 4. Conflicts with earlier requests or existing scenarios

No problems found. No existing scenario uses `doubt`, `undoubt`, `retaste`, `glowSeconds`, `regrowsIn`, `fadesIn`
or `canRetaste`, and every existing expectation is partial, so none of them changes.

CR20's replacement of CR18's "a doubt writes nothing in the journal" does not affect S54–S56: none of them checks the
journal, and they remain in force through cr20. CR20 also overrides CR18's "Doubts are permanent in this request"
explicitly. All of S51–S62 still hold at every later phase.

## 5. Format

No problems found:

- The module imports nothing and exports only `SCENARIOS`, as the r7 module does.
- Ids run S51–S62 with no gaps.
- `since` is cr17 for S51–S53, cr18 for S54–S56, cr19 for S57–S59 and cr20 for S60–S62.
- Steps use the documented forms: positional `absorb`/`taste`/`witness`/`tick`; `['mark', { id }]` from CR14;
  `['doubt', { evidence }]`, `['undoubt', { evidence }]` and `['retaste', { id, kind }]` from REQUESTS.md; and
  `['reject', { type, ... }]`.

## Not counted

- The S53 title says eating a tasted mushroom "stops its fade countdown". Only the displayed `fadesIn` stops: the taste
  itself keeps fading on its own clock (CR10). "hides its fade countdown" would be more accurate.
- These scenarios check nothing past CR10's 64-life minimum, where `regrowsIn` depends on the capacity each
  implementation declares. `present` already depends on that capacity, so this is not new.
