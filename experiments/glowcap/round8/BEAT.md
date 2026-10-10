# Glowcap: the beat and change requests CR1-CR16

# Phase base

## The beat

Vessel's slime finds glowing mushrooms that all look alike. Glowcaps make it
glow for 30 s. Duskcaps make it heavy for 20 s. The slime can absorb a
mushroom, which consumes it and reveals its kind through the effect, or taste
one, which identifies that mushroom without consuming it. What it has learned
changes what the HUD tells the player, what it will agree to absorb, and what
its journal says, and the journal explains why.

Base level: `cave`, `pool`, `ruin`. Scenarios report each mushroom's true kind
in their events; the policy never knows a kind until it is absorbed or tasted.

### Events (host → policy)

| Event | Payload | Rejected when |
| --- | --- | --- |
| `absorb` | `id`, `kind` (`glowcap`/`duskcap`) | unknown id or kind; mushroom consumed; `canAbsorb` false; kind contradicts a previous taste |
| `taste` | `id`, `kind` | unknown id or kind; mushroom consumed; mushroom already tasted |
| `tick` | `dt`, 0 ≤ dt ≤ 0.1 | dt out of range |

A rejected event throws and must leave the view exactly as it was.

### Effects

- Absorbing a glowcap sets the glow timer to 30 s; a duskcap sets the heavy
  timer to 20 s. `tick` lowers both timers by `dt`, not below 0.
- `slime.glowing` is glow timer > 0; `slime.heavy` is heavy timer > 0.

### Knowledge

- **Evidence ids** are `absorb_<id>` and `taste_<id>`.
- A mushroom is *known* once tasted (sweet = glowcap, bitter = duskcap).
- The **belief** about glowing mushrooms comes from absorptions only:
  `supportedBy` = glowcap absorptions, `contradictedBy` = duskcap absorptions.

| belief.state | when | text | note |
| --- | --- | --- | --- |
| `none` | nothing absorbed | `""` | `""` |
| `probably_safe` | support, no contradiction | `Glowing mushrooms give you light.` | `Based on one observation.` / `Based on N observations.` (N = supportedBy count) |
| `probably_unsafe` | contradiction, no support | `Glowing mushrooms make you heavy.` | same form, N = contradictedBy count |
| `uncertain` | both | `Not every glowing mushroom is safe. Taste before absorbing.` | `""` |

### Decision

`trust` = "absorb glowing mushrooms without tasting".

- **Committed** the first time the belief becomes `probably_safe`.
  `basis` = the evidence that made it so. The basis is frozen: later support
  does not change it.
- **Reopened** by every duskcap absorption after it was committed.
  `reopenedBy` lists them in order. It is never recommitted in this beat.
- `state`: `none`, `committed`, or `reopened`.

### Mushroom view (every mushroom in the level)

| Condition (first match) | label | canAbsorb | canTaste | because |
| --- | --- | --- | --- | --- |
| consumed | `""` | false | false | `[]` |
| tasted glowcap | `Glowcap` | true | false | `[taste_<id>]` |
| tasted duskcap | `Duskcap — avoid` | false | false | `[taste_<id>]` |
| belief `none` | `Glowing mushroom` | true | true | `[]` |
| belief `probably_safe` | `Probably a glowcap` | true | true | supportedBy |
| belief `probably_unsafe` | `Probably a duskcap` | true | true | contradictedBy |
| belief `uncertain` | `Could be a duskcap — taste first` | true | true | supportedBy ∪ contradictedBy |

`present` is false only when consumed. Evidence lists are compared as sets.

### View shape

```js
{
  slime: { glowing, heavy },
  mushrooms: { <id>: { present, label, canAbsorb, canTaste, because } },
  belief: { state, text, note, supportedBy, contradictedBy },
  decision: { state, basis, reopenedBy },
}
```

Each implementation exports `createPolicy()` returning `{ dispatch(event), view() }`.
Whatever glue an implementation needs to produce this shape counts as its code.

# Phase cr1

Applied after every earlier phase; cumulative.

- **CR1 — a fourth mushroom.** Add `grove`. Every rule applies to it.

# Phase cr2

Applied after every earlier phase; cumulative.

- **CR2 — twice bitten.** When `contradictedBy` has two or more entries, an
  unknown present mushroom is labelled `Too risky — taste first`,
  `canAbsorb` false, `canTaste` true, `because` = contradictedBy. Known
  mushrooms are unaffected.

# Phase cr3

Applied after every earlier phase; cumulative.

- **CR3 — tasting teaches, darkness qualifies.** A sweet taste also supports
  the belief and a bitter taste contradicts it, exactly like absorptions (so
  it can commit or reopen `trust`). A taste made while the slime is not glowing
  carries the caveat `tasted_in_dark`. Every mushroom view, the belief and the
  decision gain `caveats`: the union of the caveats on the evidence they cite
  (`because`; supportedBy ∪ contradictedBy; `basis`). Tasted mushrooms whose
  taste carries the caveat are labelled `Probably a glowcap (tasted in the
  dark)` / `Probably a duskcap (tasted in the dark)`, with the same
  `canAbsorb`/`canTaste` as their certain versions.

# Phase cr4

Applied after every earlier phase; cumulative.

- **CR4 — cite the counterexample.** In the `uncertain` state an unknown
  mushroom's `because` is contradictedBy only (its `caveats` follow `because`).

# Phase cr5

Applied after every earlier phase; cumulative.

- **CR5 — a taste fades.** Sixty seconds after a taste (counted in ticks since
  that taste), it has faded. The tasted mushroom is labelled
  `Probably a glowcap (taste has faded)` / `Probably a duskcap (taste has
  faded)`, which wins over the dark label, with the same `canAbsorb` and
  `canTaste`. The taste evidence now carries the caveat `taste_faded`, so
  every view citing it includes that caveat. The trust decision keeps the
  caveats its basis carried when it was committed; a later fade does not
  change them.

# Phase cr6

Applied after every earlier phase; cumulative.

- **CR6 — trust has a history.** `decision.history` lists each change to
  trust, in order: `{ change: 'committed', because: basis }` and
  `{ change: 'reopened', because: [evidence] }`. It is compared in order.

# Phase cr7

Applied after every earlier phase; cumulative.

- **CR7 — trust recovers.** While trust is reopened, the second glowcap
  observation (an absorption or a sweet taste) since the most recent
  contradiction commits it again. The new basis is those two observations,
  in order. `reopenedBy` and `caveats` describe the current commitment, and
  the history gains the entry. Belief and labels are unaffected.

# Phase cr8

Applied after every earlier phase; cumulative.

- **CR8 — heaviness stacks.** Absorbing a duskcap while heavy adds 20 s to
  the heavy timer, up to 30 s; otherwise it sets the timer to 20 s.
  `slime.heavySeconds` is the remaining heavy time rounded up to a whole
  second, and 0 when the slime is not heavy.

# Phase cr9

Applied after every earlier phase; cumulative.

- **CR9 — watching others eat.** A new event, `witness` (`id`, `kind`): the
  slime sees another creature eat that mushroom and what it did to the
  creature. The mushroom is consumed. The evidence is `witness_<id>`, and it
  always carries the caveat `secondhand`. A witnessed glowcap supports the
  belief and a witnessed duskcap contradicts it, exactly like an absorption
  (it can commit, reopen and recover trust). The slime's own timers do not
  change. Rejected when the id or kind is unknown, the mushroom is consumed,
  or the kind contradicts a previous taste. Being too risky for the slime to
  absorb does not stop another creature eating it.

# Phase cr10

Applied after every earlier phase; cumulative.

- **CR10 — mushrooms regrow.** Forty-five seconds after a mushroom is consumed
  (counted in ticks since, like a fade), a new mushroom grows in its place:
  present, untasted, of unknown kind, which may differ from the last. Evidence
  about the new mushroom takes a life number: the first life keeps
  `absorb_cave`, `taste_cave`, `witness_cave`; the second is `absorb_cave_2`,
  `taste_cave_2`, `witness_cave_2`; and so on, up to a capacity the implementation declares, of at least
  64 lives per mushroom. A mushroom that has used its last life stays consumed
  and does not regrow; no tick is rejected for this, and an event on that
  mushroom is rejected as on any consumed mushroom. Evidence from
  earlier lives stays in the belief and the decision, and an earlier taste
  still fades on its own clock.

# Phase cr11

Applied after every earlier phase; cumulative.

- **CR11 — say why not.** Each mushroom view gains
  `why: { absorb: { reason, because, caveats }, taste: { reason, because, caveats } }`.
  `reason` is `''` and `because` is `[]` when the action is allowed;
  otherwise the first match:

  | action | when | reason | because |
  | --- | --- | --- | --- |
  | absorb, taste | consumed | `Already eaten` | the evidence that consumed the current life |
  | absorb | tasted duskcap | `Known duskcap` | the taste |
  | absorb | unknown, twice bitten | `Too risky untasted` | contradictedBy |
  | taste | tasted | `Already tasted` | the taste |

  `caveats` is the union of the caveats on `because`, as for every other view.

# Phase cr12

Applied after every earlier phase; cumulative.

- **CR12 — save and resume.** Each policy gains `save()`, returning a JSON
  value, and `createPolicy(saved)` resumes from it. After resuming, the view is
  identical and every later event is accepted or rejected, and changes the
  view, exactly as it would have without the save. `JSON.stringify(save())` is
  at most 4096 bytes in every scenario, including ten minutes of continuous
  play. The harness step `['resume']` saves, round-trips the save through
  JSON, resumes, compares the complete views and continues on the resumed
  policy.

# Phase cr13

Applied after every earlier phase; cumulative.

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

Applied after every earlier phase; cumulative.

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

Applied after every earlier phase; cumulative.

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

Applied after every earlier phase; cumulative.

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
