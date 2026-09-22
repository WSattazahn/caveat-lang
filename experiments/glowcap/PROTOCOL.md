# Glowcap or duskcap — a decisive Caveat vs TypeScript test

Status: **pre-registered**. This file, `scenarios.mjs` and `harness.mjs` are
committed before either implementation exists. Changes after that commit are
listed under *Amendments* with their reason.

## Question

For gameplay logic where the character's knowledge is the mechanic, is Caveat
a better way to write it than plain TypeScript? "Better" is decided by the rule
at the end, on measurements taken by the harness, not by impression.

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

## Change requests

Applied in order after both implementations pass the base phase. Each is
cumulative. Its scenarios are already written in `scenarios.mjs`.

- **CR1 — a fourth mushroom.** Add `grove`. Every rule applies to it.
- **CR2 — twice bitten.** When `contradictedBy` has two or more entries, an
  unknown present mushroom is labelled `Too risky — taste first`,
  `canAbsorb` false, `canTaste` true, `because` = contradictedBy. Known
  mushrooms are unaffected.
- **CR3 — tasting teaches, darkness qualifies.** A sweet taste also supports
  the belief and a bitter taste contradicts it, exactly like absorptions (so
  it can commit or reopen `trust`). A taste made while the slime is not glowing
  carries the caveat `tasted_in_dark`. Every mushroom view, the belief and the
  decision gain `caveats`: the union of the caveats on the evidence they cite
  (`because`; supportedBy ∪ contradictedBy; `basis`). Tasted mushrooms whose
  taste carries the caveat are labelled `Probably a glowcap (tasted in the
  dark)` / `Probably a duskcap (tasted in the dark)`, with the same
  `canAbsorb`/`canTaste` as their certain versions.
- **CR4 — cite the counterexample.** In the `uncertain` state an unknown
  mushroom's `because` is contradictedBy only (its `caveats` follow `because`).

## Measurements

Every harness run appends a record to `runs.jsonl`: time, phase,
implementation, content hash of its files, and pass/fail per scenario.

1. **Correctness.** Scenario failures on the first complete run of each phase,
   and runs needed to reach all-green.
2. **Size.** Non-blank, non-comment lines and bytes: policy source, plus glue.
3. **Change cost.** Lines added + removed per CR (`git diff --numstat` of that
   implementation's files), and regressions: previously passing scenarios that
   failed during the CR.
4. **Explanation.** Failures on `because`/`supportedBy`/`contradictedBy`/
   `basis`/`reopenedBy`/`caveats` assertions, counted separately, and whether
   each implementation produced explanations from its own model or needed
   hand-written bookkeeping.
5. **Cost to run and ship.** Median µs per dispatch over a 10k-event replay,
   and bytes a host must ship.

## Decision rule

Caveat is adopted for this kind of gameplay logic in Vessel only if it is
**better on at least two of measurements 1–4 and no more than 2× worse on
any**, and its dispatch cost stays under 1 ms. Otherwise TypeScript is the
default: on a tie the incumbent wins, because Caveat alone brings a WASM
runtime, a second repository and a pinning pipeline.

## Conditions and threats to validity

- One author writes both, and knows TypeScript far better than a five-day-old
  language. TypeScript is implemented **first**, so it bears the cost of
  discovering any ambiguity in this spec; the Caveat side benefits.
- Caveat is tested at its most capable revision: `feat/modules-0.5`, including
  the unmerged repetition 0.1 (`for KIND as $x { … }`).
- CR3 exercises what Caveat is designed for (qualifications carried through
  derived values); CR4 exercises authored control over explanations. Neither
  was chosen after seeing an implementation.
- One beat is one data point. The result decides *this* adoption question, not
  whether Caveat's ideas are sound.

## Amendments

None yet.
