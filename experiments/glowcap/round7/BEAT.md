# Glowcap: the beat and change requests CR1-CR12

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
  64 lives per mushroom; an event that would exceed it is rejected and leaves
  the view unchanged. Evidence from
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
