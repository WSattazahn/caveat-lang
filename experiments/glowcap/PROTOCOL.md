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

## Round 3: blind change requests

These four requests and their scenarios (S21–S27) were committed before either
implementation was changed for them. They apply to `ts/` and to `caveat2/`,
which is Caveat with [Explanations 0.1](../../spec/caveat-explanations-0.1.md)
and nothing newer; the language is frozen for this round. The measurements and
the decision rule are the same.

- **CR5 — a taste fades.** Sixty seconds after a taste (counted in ticks since
  that taste), it has faded. The tasted mushroom is labelled
  `Probably a glowcap (taste has faded)` / `Probably a duskcap (taste has
  faded)`, which wins over the dark label, with the same `canAbsorb` and
  `canTaste`. The taste evidence now carries the caveat `taste_faded`, so
  every view citing it includes that caveat. The trust decision keeps the
  caveats its basis carried when it was committed; a later fade does not
  change them.
- **CR6 — trust has a history.** `decision.history` lists each change to
  trust, in order: `{ change: 'committed', because: basis }` and
  `{ change: 'reopened', because: [evidence] }`. It is compared in order.
- **CR7 — trust recovers.** While trust is reopened, the second glowcap
  observation (an absorption or a sweet taste) since the most recent
  contradiction commits it again. The new basis is those two observations,
  in order. `reopenedBy` and `caveats` describe the current commitment, and
  the history gains the entry. Belief and labels are unaffected.
- **CR8 — heaviness stacks.** Absorbing a duskcap while heavy adds 20 s to
  the heavy timer, up to 30 s; otherwise it sets the timer to 20 s.
  `slime.heavySeconds` is the remaining heavy time rounded up to a whole
  second, and 0 when the slime is not heavy.

## Round 6: blind change requests

These four requests and their scenarios (S28–S38) were committed before either
implementation was changed for them. They apply to `ts/` and to `caveat4/`
(Caveat as of `feat/grounded-explanations` 70d4c7b: explanations 0.2, reject,
define, typed parameters, view, late qualification, decision journal). The
language and runtime are frozen for this round: nothing in `runtime/` changes
until the round is scored. The measurements and the decision rule are the same.

- **CR9 — watching others eat.** A new event, `witness` (`id`, `kind`): the
  slime sees another creature eat that mushroom and what it did to the
  creature. The mushroom is consumed. The evidence is `witness_<id>`, and it
  always carries the caveat `secondhand`. A witnessed glowcap supports the
  belief and a witnessed duskcap contradicts it, exactly like an absorption
  (it can commit, reopen and recover trust). The slime's own timers do not
  change. Rejected when the id or kind is unknown, the mushroom is consumed,
  or the kind contradicts a previous taste. Being too risky for the slime to
  absorb does not stop another creature eating it.
- **CR10 — mushrooms regrow.** Forty-five seconds after a mushroom is consumed
  (counted in ticks since, like a fade), a new mushroom grows in its place:
  present, untasted, of unknown kind, which may differ from the last. Evidence
  about the new mushroom takes a life number: the first life keeps
  `absorb_cave`, `taste_cave`, `witness_cave`; the second is `absorb_cave_2`,
  `taste_cave_2`, `witness_cave_2`; and so on, with no limit. Evidence from
  earlier lives stays in the belief and the decision, and an earlier taste
  still fades on its own clock.
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
- **CR12 — save and resume.** Each policy gains `save()`, returning a JSON
  value, and `createPolicy(saved)` resumes from it. After resuming, the view is
  identical and every later event is accepted or rejected, and changes the
  view, exactly as it would have without the save. `JSON.stringify(save())` is
  at most 4096 bytes in every scenario, including ten minutes of continuous
  play. The harness step `['resume']` saves, round-trips the save through
  JSON, resumes, compares the complete views and continues on the resumed
  policy.

### Predictions, registered with the requests

The author wrote both implementations and has now seen them, so these requests
cannot be chosen blind to their fit. To keep that visible, each carries a
prediction made before any implementation work:

| CR | predicted | why |
| --- | --- | --- |
| CR9 | TypeScript smaller diff; explanations free on both sides | Caveat has no way to share the absorb rules with a new event (procedures take numbers, not evidence), so the rules are repeated |
| CR10 | TypeScript clearly better | Caveat evidence is declared statically; unbounded lives are dynamic identity, which the frozen language does not have |
| CR11 | Caveat better | reasons with cited, qualified evidence are bindings with `because`; TypeScript needs the bookkeeping of each rule twice |
| CR12 | TypeScript better | the runtime has no restore; the Caveat side must replay events from the host |

A prediction that fails is reported as failed.

### Fuzz for round 6

`differential.mjs --round=6` adds `witness` events, 45-second tick bursts,
and a `resume` step, in which both implementations save and resume at the
same point. It is run after CR12 with `--against=caveat4`.

### Round 6 replay: the language the work list produced

Not blind. After round 6 was scored, its work list was built into the
language: incremental evaluation, a load-time check on observation order,
procedures over symbols, renewable evidence with scheduled qualification and
`carries`, and save and restore. `caveat5/` starts from `caveat4/` as it stood
after CR8 (byte for byte) and replays CR9–CR12 on that language, one commit
per phase, knowing the requests and scenarios. The scenarios, TypeScript side,
measurements and fuzz (`--against=caveat5`) are unchanged. Like rounds 4 and 5,
it shows what the new language costs on these requests, not how it does on
requests it was not shaped for.

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

None to the behaviour, scenarios, measurements or decision rule after
`cfcc881`. One check was added after the fact and is labelled as such in
[RESULTS.md](RESULTS.md): `differential.mjs`, a seeded differential fuzz of
the two implementations.

Round 6: the round-6 fuzz mode committed in `653bf24` crashed on its first
run because a new local variable (`events`) shadowed the event counter. It
was renamed; what the fuzz generates and compares did not change. The round-6
fuzz ran 300 sequences rather than the default 2000, because the Caveat side
had become slow enough (see RESULTS) that 2000 would take about five hours.

Round 6 replay, final verification: the original differential comparator
sorted `decision.basis` and `decision.reopenedBy`, hiding order differences
despite the ordering requirements above. A new opt-in `--ordered-decisions`
mode preserves those lists, the journal and each journal entry's evidence
order, while comparing the specified evidence sets without order. The
original mode remains available so its published results are reproducible.
The saved-divergence replay uses the stricter comparison after every event.
This changes verification, not the requested behavior or scoring rule; the
adapter error it exposed and its correction are recorded in RESULTS.md.
