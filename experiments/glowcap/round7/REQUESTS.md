# Glowcap: change requests CR13-CR16

Applied in order after CR12; each is cumulative on everything before it. No
existing scenario's expectation changes.

# Phase cr13

- **CR13 — the slime forgets.** The slime only remembers its **six most
  recent observations**. An observation is an accepted `absorb`, `taste` or
  `witness`, ordered by when its event was accepted. When a seventh is
  accepted, the oldest remembered observation is forgotten. Rejected events
  and ticks never forget anything.
  - **The belief** is built from remembered observations only:
    `supportedBy` and `contradictedBy` drop a forgotten observation, and the
    belief's `state`, `text`, `note` (N counts remembered entries only) and
    `caveats` follow. So does everything that cites the belief: an unknown
    mushroom's `label`, `canAbsorb`, `because` and `caveats`, the CR2
    twice-bitten check (two or more *remembered* contradictions), and CR11's
    `Too risky untasted` reason and its `because`. Forgetting can therefore lift
    "twice bitten". The belief is never `none` again once anything has been
    observed.
  - **What forgetting does not touch.** A taste still identifies its mushroom
    for the rest of that mushroom's life after the belief has forgotten it:
    that mushroom's label, `canAbsorb`, `canTaste`, `because`, `caveats` and
    `why` are unchanged, and an `absorb` or `witness` of the other kind is
    still rejected. A consumed mushroom still cites what consumed it in
    `why`. The decision's `basis` stays frozen and keeps the caveats it had
    when committed, even when its evidence is forgotten. CR7's count ("the
    second glowcap observation since the most recent contradiction") counts
    accepted observations, remembered or not.
  - **Committing.** While trust has never been committed, the accepted
    observation after which the belief (with any forgetting it caused) is
    `probably_safe` commits trust; the basis is that one observation. This is
    the base rule restated, and it now also applies when the observation made
    the belief safe by pushing the last remembered contradiction out. Once
    trust has been committed, only CR7 commits it again; forgetting never
    commits, reopens or recommits trust by itself.
  - **The decision is bounded too.** `decision.reopenedBy` keeps at most the
    six most recent contradictions of the current commitment, and
    `decision.history` keeps at most the six most recent entries (still
    compared in order). Adding a seventh drops the oldest.
  - Nothing else grows; CR12's 4096-byte save bound now holds for play of any
    length.

# Phase cr14

- **CR14 — the player can mark a mushroom.** Two new events, both with
  payload `id`:

  | Event | Effect | Rejected when |
  | --- | --- | --- |
  | `mark` | the mushroom becomes marked | unknown id; mushroom consumed; already marked |
  | `unmark` | the mushroom is no longer marked | unknown id; not marked |

  Every mushroom view gains `marked` (boolean, initially false). A marked
  mushroom has `canAbsorb` false and an `absorb` of it is rejected; its
  `label`, `because`, `caveats`, `canTaste` and `why.taste` are as they would
  be unmarked, and it can be tasted. Another creature does not care: a
  `witness` of a marked mushroom is accepted as before. Consuming a mushroom
  (by absorb or witness) clears its mark, so a consumed mushroom is never
  marked and a regrown mushroom starts unmarked. A mark is not evidence: it
  never appears in any `because`, belief or decision, and changes no timer.
  CR11's table gains one row for `absorb`, after `Known duskcap` and before
  `Too risky untasted`: when marked, reason `Marked to avoid`, `because` `[]`,
  `caveats` `[]`. The rows are still first match, so a marked tasted duskcap
  says `Known duskcap` and a marked consumed mushroom cannot exist. A mark is
  one boolean per mushroom and needs no further bound.

# Phase cr15

- **CR15 — a mushroom out of reach.** Add a fifth mushroom, `pit`. It grows
  where the slime cannot get to it, so only other creatures can eat it.
  - `absorb` and `taste` of `pit` are always rejected, and so is `mark`
    (and therefore `unmark`) of `pit`. `witness` of `pit` follows the usual
    rules and teaches the belief like any witness (`witness_pit`,
    `witness_pit_2`, …), and `pit` regrows like any mushroom (CR10).
  - Its view: `canAbsorb` false, `canTaste` false, `marked` false always.
    `present`, `label`, `because` and `caveats` follow the existing rules (it
    can never be tasted, so while present it is labelled as an unknown
    mushroom, including `Too risky — taste first` when twice bitten; when
    consumed it uses the consumed row).
  - CR11's table gains a row for both `absorb` and `taste`, directly after
    `Already eaten`: when the mushroom is `pit`, reason `Out of reach`,
    `because` `[]`, `caveats` `[]`. So a consumed `pit` says `Already eaten`
    and a present `pit` always says `Out of reach` for both actions.
  - The other four mushrooms are unaffected.

# Phase cr16

- **CR16 — the journal.** The view gains `journal`: a list of at most **six**
  entries, oldest first, **compared in order**. Each entry is
  `{ text, because, caveats }`, where `caveats` is the union of the *current*
  caveats on `because` (so a fade updates older entries, as it does every
  view except the decision).
  - Each accepted `absorb`, `taste` or `witness` adds one entry, with
    `because` = `[that event's evidence id]` and this `text`, where `<id>` is
    the mushroom's name (`cave`, `pool`, `ruin`, `grove`, `pit`) with no life
    number:

    | event | kind | text |
    | --- | --- | --- |
    | `absorb` | glowcap | `Absorbed the <id> mushroom: it gave light.` |
    | `absorb` | duskcap | `Absorbed the <id> mushroom: it made me heavy.` |
    | `taste` | glowcap | `Tasted the <id> mushroom: sweet.` |
    | `taste` | duskcap | `Tasted the <id> mushroom: bitter.` |
    | `witness` | glowcap | `Saw a creature eat the <id> mushroom: it glowed.` |
    | `witness` | duskcap | `Saw a creature eat the <id> mushroom: it grew heavy.` |

  - If that event also added an entry to `decision.history`, a second journal
    entry follows it: for `committed`, text `I trust glowing mushrooms now.`
    with `because` = the new basis; for `reopened`, text
    `I no longer trust glowing mushrooms.` with `because` = `[that event's
    evidence id]`.
  - Nothing else writes to the journal: not ticks, fades, regrowth, forgetting
    (CR13), `mark`/`unmark`, or rejected events.
  - When an entry would make seven, the oldest entry is dropped first (an
    event adding two entries can drop two). The journal starts empty and is
    saved and resumed like the rest of the view; CR12's 4096-byte save bound
    still holds.
