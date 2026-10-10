# Glowcap: change requests CR17-CR20

Applied in order, each cumulative on CR1-CR16 and on the requests before it.
No existing scenario's expectation changes. New events are dispatched as
`{ type, ...payload }`, and, as before, a rejected event throws and leaves the
view exactly as it was. Rounding "up to a whole second" means the same as
CR8's `heavySeconds`.

# Phase cr17

- **CR17 — the HUD shows countdowns.** The player can see how long things
  last. Three new view fields, all whole seconds rounded up, and all read-only
  (no new event):

  | field | value | 0 when |
  | --- | --- | --- |
  | `slime.glowSeconds` | remaining glow time | the slime is not glowing |
  | `mushrooms.<id>.regrowsIn` | time until this mushroom regrows (CR10) | it is present, or it has used its last life and will not regrow |
  | `mushrooms.<id>.fadesIn` | time until the taste that identifies this mushroom's current life fades (CR5) | it is consumed, its current life has not been tasted, or that taste has already faded |

  So right after absorbing a glowcap `glowSeconds` is 30 and that mushroom's
  `regrowsIn` is 45; right after a taste that mushroom's `fadesIn` is 60; half
  a second before a timer ends the field shows 1, and at the moment it ends
  (the glow stops, the mushroom regrows, the taste fades) it shows 0. A
  mushroom that regrows starts untasted, so its `fadesIn` is 0. These are
  views of timers the game already keeps; nothing new is stored and the
  fields come back identical after a resume.

# Phase cr18

- **CR18 — doubt what you saw.** The player can tell the slime that a
  creature's reaction was misleading (perhaps the creature was already ill).
  New event `doubt`, payload `evidence` (a string: an evidence id such as
  `witness_pool` or `witness_pool_2`).

  | Event | Effect | Rejected when |
  | --- | --- | --- |
  | `doubt` | that witness becomes doubted | `evidence` is not the id of a witness observation (`witness_<id>` or `witness_<id>_<n>`) the slime currently remembers (CR13); it is already doubted; or every other remembered observation is doubted (the slime must keep at least one undoubted observation) |

  - A doubted witness carries the caveat `doubted`, in addition to
    `secondhand`, so every view that cites it shows that caveat: the journal
    entry for it, a consumed mushroom's `why` that cites it, and so on. The
    caveat stays on that evidence wherever it is still cited, including after
    the observation is forgotten.
  - **The belief does not count a doubted observation.** It is removed from
    `supportedBy` / `contradictedBy`, and the belief's `state`, `text`, `note`
    (N counts undoubted remembered entries), `caveats`, and everything that
    cites the belief (unknown mushrooms' `label`, `canAbsorb`, `because`,
    `caveats`, the CR2 twice-bitten check and CR11's `Too risky untasted`)
    follow. Because one undoubted observation always remains, the belief is
    still never `none` again once anything has been observed.
  - **A doubt is not an observation.** The doubted witness still occupies its
    place among the six remembered observations and is forgotten in its turn;
    the `doubt`
    event forgets nothing and changes no timer.
  - **A doubt never changes the decision.** `state`, `basis`, `caveats` (still
    frozen at commit), `reopenedBy` and `history` are untouched, even when they
    cite the doubted witness. CR7 still counts every accepted observation, and
    a doubted contradiction is still "the most recent contradiction" for CR7's
    count. While trust has never been committed, only an accepted observation
    can commit it (CR13), so if a doubt leaves the belief `probably_safe`, the
    next accepted observation after which it is still `probably_safe`
    commits, with that one observation as the basis.
  - A doubt writes nothing in the journal (CR16's list is unchanged).
  - Doubts are permanent in this request. Bound: only remembered witnesses
    can be doubted, so at most five observations are doubted at once; once
    forgotten, a doubted witness can only still be cited by the journal (at
    most six entries) or by its consumed mushroom's `why` (one per mushroom)
    as far as caveats are concerned; the decision may also cite it, but its
    caveats are frozen and `reopenedBy` / `history` show none, so it needs no
    doubt flag.

# Phase cr19

- **CR19 — taste again.** Once a taste has faded, the player can have the
  slime taste that mushroom a second time. New event `retaste`, payload `id`,
  `kind` (the mushroom's true kind, as for `taste`).

  | Event | Effect | Rejected when |
  | --- | --- | --- |
  | `retaste` | the slime tastes the mushroom again | unknown id or kind; the mushroom is `pit`; consumed; its current life has not been tasted; that taste has not faded yet; it has already been retasted in its current life; `kind` differs from the kind that taste found |

  - The evidence is `retaste_<id>` for a mushroom's first life and
    `retaste_<id>_<n>` for life n (as CR10 numbers lives). At most one retaste
    per life, so nothing grows.
  - A retaste is an observation, exactly like a taste: it is remembered and
    forgotten (CR13), a sweet one supports and a bitter one contradicts the
    belief, it can commit, reopen and recover trust (it counts for CR7), it
    carries `tasted_in_dark` when made while the slime is not glowing, and
    sixty seconds after it, it fades and carries `taste_faded`. A marked
    mushroom can be retasted.
  - From the retaste on, for the rest of that life, the retaste replaces the
    first taste as what identifies the mushroom: its `label` (`Glowcap`,
    `Duskcap — avoid`, or their `(tasted in the dark)` / `(taste has faded)`
    forms, decided by the retaste's own caveats), `because` = `[retaste id]`,
    `caveats`, CR11's `Known duskcap` and `Already tasted` rows (their
    `because` is the retaste) and CR17's `fadesIn` (counting down the retaste).
    `canAbsorb` and `canTaste` are as for the first taste. The first taste
    stays evidence on its own (still in the belief while remembered, keeping
    its own caveats).
  - Each mushroom view gains `canRetaste`: true exactly when a `retaste` of
    it, with the kind its current life was tasted as, would be accepted. A
    regrown mushroom starts untasted, so it cannot be retasted until it has
    been tasted and that taste has faded. `pit` is always `false`.
  - Journal (CR16) text for a retaste, with `because` = `[retaste id]` and
    followed by a trust entry when it changes the decision, as for any
    observation: `Tasted the <id> mushroom again: sweet.` /
    `Tasted the <id> mushroom again: bitter.`

# Phase cr20

- **CR20 — take back a doubt, and write doubts down.**
  - New event `undoubt`, payload `evidence`. Accepted when `evidence` is a
    witness the slime currently remembers and that is doubted; rejected
    otherwise (including a doubted witness that has since been forgotten:
    that doubt is permanent). The witness loses the `doubted` caveat
    everywhere it is cited and counts in the belief again, with everything
    that follows from the belief (so it can bring back "twice bitten").
    Like a doubt, it is not an observation, forgets nothing, changes no
    timer and never changes the decision; it does not reset CR7's count.
    A witness can be doubted again after being undoubted.
  - **The journal records doubts.** This replaces CR18's "a doubt writes
    nothing in the journal" and adds two events to CR16's list of what writes
    to the journal. Each accepted `doubt` adds one entry
    `I doubt what I saw at the <id> mushroom.` and each accepted `undoubt`
    adds one entry `I believe what I saw at the <id> mushroom again.`, where
    `<id>` is the mushroom's name without a life number, `because` =
    `[evidence]`, and `caveats` are, as for every entry, the current caveats
    of `because` (so a doubt's entry shows `doubted` until the witness is
    undoubted). The journal still keeps at most six entries, dropping the
    oldest first; nothing else changes about it.

## Scenarios

`scenarios-r8.mjs`, S51-S62: three per request (S51-S53 CR17, S54-S56 CR18,
S57-S59 CR19, S60-S62 CR20). New steps are written
`['doubt', { evidence }]`, `['undoubt', { evidence }]` and
`['retaste', { id, kind }]`. `journal` and `decision.history` are compared in
order, as before; other lists are sets.
