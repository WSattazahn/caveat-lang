# Glowcap or duskcap — results

This benchmark is Caveat's scoreboard against a plain TypeScript
implementation of the same beat. The rule is fixed in
[PROTOCOL.md](PROTOCOL.md) (commit `cfcc881`): to win, Caveat must be better
on at least two of the four measurements.

**Round 1: TypeScript.** Caveat was not clearly better on any measurement.
The two tied on correctness, change cost and explanation accuracy. On size
Caveat had fewer lines but more bytes, and on a tie the incumbent wins. The
places where it lost became the language's work list.

**Round 2, after [Explanations 0.1](../../spec/caveat-explanations-0.1.md):
Caveat**, on change cost and explanations. Details and caveats are
[below](#rescore-after-explanations-01).

**Round 3, blind change requests: TypeScript.** Four new requests were
committed before either side was changed. Caveat was clearly cheaper on two of
them and far more expensive on one: 97 changed lines against 53. Details are
[below](#round-3-blind-change-requests).

**Round 4, after grounds, reject, define, typed parameters and the view:
level, Caveat slightly ahead.** Across all eight requests the change cost was
105 lines against TypeScript's 110. The program was 154 lines against 164,
dispatch took 97 µs (down from 202 µs), and the runtime shipped 333 KB
gzipped. This round was written knowing every request. Details are
[below](#round-4-the-language-after-rounds-1-3).

**Round 5, after late qualification and the decision journal: Caveat.**
Replaying CR5–CR8 from round 4's CR4 state cost 36 changed lines against
TypeScript's 53. Across all eight requests that makes 71 against 110. The
program is 134 lines against 164. Also written knowing every request. Details
are [below](#round-5-late-qualification-and-the-decision-journal).

## Measurements

All numbers come from `runs.jsonl`, `harness.mjs --measure`, `--bench`, and
`git diff` between the phase commits.

| | TypeScript | Caveat | Better |
| --- | --- | --- | --- |
| **1. Correctness**: phases green on first complete run | 5 of 5 | 5 of 5 | tie |
| Harness runs needed / failing runs | 5 / 0 | 5 / 0 | tie |
| **2. Size** (final, code lines) | 141 | 83 source + 40 adapter = 123 | Caveat, −13% |
| Size (final, bytes) | 6,747 | 6,424 + 2,543 = 8,967 | TypeScript, −25% |
| **3. Change cost**, code lines changed, CR1–CR4 | 57 | 55 | tie |
| Regressions during change requests | 0 | 0 | tie |
| **4. Explanation failures** | 0 | 0 | tie |
| **5. Dispatch + view**, median / p95 | 0.9 µs / 1.9 µs | 139.6 µs / 173.8 µs | under the 1 ms gate |
| Bytes a host ships (gzipped) | 2,210 | 396,971 | TypeScript, 180× |

Change cost by request, counting code lines only (blank and comment lines
excluded):

| Request | TypeScript | Caveat | Why |
| --- | --- | --- | --- |
| CR1: a fourth mushroom | 2 | 1 | One declaration each. Repetition (`for mushroom as $m`) made Caveat's cost O(1). |
| CR2: twice bitten | 5 | 13 | Caveat's "too risky" label cited everything its sibling guards read, so four existing label guards had to be reordered to short-circuit before `support`. |
| CR3: tasting teaches, darkness qualifies | 48 | 32 | Caveat's core strength. A caveat added once to a qualified value reached labels, the belief and the frozen decision basis with no bookkeeping. TypeScript had to add a caveat map, a union helper and a restructured label function. |
| CR4: cite the counterexample | 2 | 9 | Caveat's provenance holds that an uncertain label *does* depend on the supporting evidence, so it cannot express "cite only the counterexample". The source had to add a `cite` binding and the adapter an override. |

## Post-hoc check: differential fuzz

This check was not part of the pre-registered protocol. `differential.mjs`
replays seeded random sequences through both implementations: absorb and taste
on valid and invalid ids and kinds, in-range and out-of-range ticks, and bursts
long enough to cross both timers. It compares every accept or reject decision,
and the view after every step.

- 1,000 sequences, 1,727,134 events (seed 7): **0 divergences.** Another
  50 sequences with seed 1 also showed 0.
- **Sensitivity.** The fuzz caught one planted bug per implementation:
  - TypeScript: the dark-taste test changed from `glow <= 0` to `glow < 0`.
    89 of 100 sequences diverged.
  - Caveat: the twice-bitten threshold changed from `< 2` to `< 3`.
    22 of 100 sequences diverged.
  Both plants were reverted.

The two implementations reached the same behaviour through independent
mechanisms, which makes a shared misreading of the spec unlikely.

## What each language made easy or hard

**Caveat's advantage is real and narrow.** It shows when a new qualification has
to flow through derived values (CR3). The runtime refused to lose a dependency,
and every consumer inherited the caveat without being touched.

**Getting a *designed* explanation out of Caveat means steering its lineage.**
Its provenance is conservative: a binding cites everything its guards
evaluated. Three constructs exist only to make the cited evidence match what a
player should be told:

- a dummy `check` state, so validation reads do not enter a label's lineage;
- label guards ordered so `and` short-circuits before reading unrelated state;
- the CR4 `cite` binding and its adapter override.

An author has to reason about evaluation order to get the explanation right.
In TypeScript the explanation is an explicit list and needed no such reasoning.
This matches the open question in
[DEVELOPMENT_INSIGHTS.md](../../docs/DEVELOPMENT_INSIGHTS.md):
preserving a dependency is not the same as proving relevance.

**Learning cost.** The TypeScript side needed nothing beyond the protocol. The
Caveat side needed the reactive 0.1–0.7 specs and the repetition spec (about
1,000 lines), plus two exploratory CLI runs to learn the snapshot's shape.

## Threats to validity

- One author wrote both. TypeScript went first and so bore the cost of finding
  ambiguities in the spec; none needed an amendment.
- Caveat ran at its most capable revision. That includes repetition 0.1, which
  is unmerged (`feat/modules-0.5`). On `main` the CR1 cost would have been one
  copied block per mushroom.
- One beat is one data point. The result settles this adoption question for
  Vessel. It does not show that Caveat's ideas are unsound: the TypeScript
  implementation *is* those ideas (evidence, frozen basis, reopening, cited
  explanations) written without the language.

## Rescore after Explanations 0.1

[Explanations 0.1](../../spec/caveat-explanations-0.1.md) added `because` to
bindings. An author says what a binding cites, and the runtime rejects any
citation the binding never read. The Caveat side was then re-authored as
`caveat2/` and replayed through the same frozen harness, phase by phase, with
one commit per phase. The TypeScript side is unchanged.

| | TypeScript | Caveat (before) | Caveat + `because` |
| --- | --- | --- | --- |
| Phases green on first run | 5 / 5 | 5 / 5 | 5 / 5 |
| Final size, code lines | 141 | 123 | **113** (78 + 35) |
| Final size, bytes | **6,747** | 8,967 | 8,088 |
| Change cost CR1–CR4 | 57 | 55 | **40** |
| CR1 / CR2 / CR3 / CR4 | 2 / 5 / 48 / 2 | 1 / 13 / 32 / 9 | 1 / 5 / 32 / 2 |
| Explanation failures | 0 | 0 | 0 |
| Explanation bookkeeping | caveat map, union helper, lists by hand; nothing checks a citation | guard ordering, dummy `check` state, `cite` override | none; every citation is checked against lineage at runtime |
| Differential fuzz vs TypeScript | — | 0 of 1.73M events | 0 of 1.73M events |
| Dispatch + view, median | 0.8 µs | 196 µs | 168 µs |
| Shipped, gzipped | 2 KB | 402 KB | 402 KB |

**By the pre-registered rule, Caveat now wins.**
- It is better on change cost (−30%) and on explanations: no bookkeeping, and
  a guarantee TypeScript lacks, since a fabricated citation is rejected at
  runtime.
- It ties on correctness. Size is mixed: 20% fewer lines, 20% more bytes.
- It is no more than 2× worse on any of measures 1–4.
- Dispatch stays under the 1 ms gate.

The gain is exactly where the language was weak. CR2 fell from 13 to 5 lines
and CR4 from 9 to 2; CR1 and CR3 are unchanged. The dummy validation state,
the guard ordering and the adapter override are gone.

**Costs and caveats of this result:**
- **Authored with foresight.** The rescore was written knowing the four change
  requests, and the TypeScript side got no second pass with the same
  knowledge. The improvement is confined to the two requests the feature
  targets, but only a blind round can confirm it: new change requests,
  pre-registered before either side sees them.
- **Explanations made every dispatch slower.** They are published for every
  shown binding, so the unchanged Caveat program went from 140 µs to 196 µs
  per event. Serialising the whole snapshot on each dispatch is the remaining
  runtime cost, and the next target.
- **Shipped size is unchanged**, about 400 KB gzipped.

## Round 3: blind change requests

CR5–CR8 and scenarios S21–S27 were committed at `6409b13`, before either
implementation changed. Caveat was frozen at Explanations 0.1. Each side
applied the requests in order, one commit per request, and TypeScript went
first each time.

| | TypeScript | Caveat |
| --- | --- | --- |
| CR5: a taste fades | 22 | **15** |
| CR6: trust has a history | 15 | **6** |
| CR7: trust recovers | **11** | 71 |
| CR8: heaviness stacks | 5 | 5 |
| **Change cost, blind** | **53** | 97 |
| Phases green on first run | **4 / 4** | 3 / 4 |
| Final size, code lines | **164** | 174 (125 + 49) |
| Final size, bytes | **8,220** | 13,506 |
| Differential fuzz, 1.84M events | — | 0 divergences |

**TypeScript wins the blind round:** it was better on correctness, size and
change cost. Both are correct: the fuzz found no disagreement, including
fading, recovery, ordered history and stacking.

**Where Caveat was better.**
- CR5: a caveat attached to evidence after the fact reached every label and
  the belief by adding `qualified(0, taste, taste_faded)` to the values that
  counted it, and the decision's frozen basis kept its own caveats for free.
  TypeScript needed a timestamp map and a caveat freeze at commit time.
- CR6: the history already existed in the commitment's record.

**Where it broke down: CR7, 71 lines against 11.** The request was ordinary
("the second glowcap after a contradiction restores trust, based on those
two"). Four language facts turned it into shadow bookkeeping:

1. A singleton commitment cannot be made twice, so `trust` became a
   decision series.
2. A new revision's basis includes its predecessor's basis and reopening
   witness. That is correct lineage, but the designer's "based on those two"
   had nowhere to live except a separate `trust_basis` state.
3. A guard's lineage enters whatever its rule sets, including rules that are
   skipped. Guards that read qualified counts or `reopened(trust)` would have
   put unrelated evidence into `trust_basis`, `recovery` and `trust_state`.
   Keeping them clean took plain shadow copies: `trust_state`,
   `recovery_count`, `contradiction_count` and a plain `epoch`, which
   decides whether a fading taste is still in the recovery window.
4. Bindings are primitive and provenance sets are unordered, so the adapter
   rebuilt the per-revision history by subtracting inherited lineage and
   re-sorting by observation order.

The same thing that cost CR2 and CR4 in round 1 struck again one level
down: Caveat's lineage is the complete dependency, and the language had no
way to say *which part is the meaning*. Explanations 0.1 solved that for
bindings only. State and commitments still lack it.

**Also found:** a `for` block containing the word "for" in a comment is
rejected as a nested block. That caused the one failed Caveat run.
(Fixed in `72b7c32`.)

## Round 4: the language after rounds 1–3

Five additions came out of the rounds above:
- [Explanations 0.2](../../spec/caveat-explanations-0.2.md): grounds, what a
  value is *based on*, separate from lineage;
- [Reject 0.1](../../spec/caveat-reject-0.1.md);
- [Define 0.1](../../spec/caveat-define-0.1.md);
- [Typed Parameters 0.1](../../spec/caveat-typed-parameters-0.1.md);
- [View 0.1](../../spec/caveat-view-0.1.md), with a lean reactive-only
  runtime.

The Caveat side was re-authored as `caveat3/` and replayed from base to CR8,
one commit per phase. **Written with knowledge of every request**, so read it
as a measure of the language, not a blind trial.

| | TypeScript | Caveat, rounds 2–3 | Caveat, round 4 |
| --- | --- | --- | --- |
| CR1 / CR2 / CR3 / CR4 | 2 / 5 / 48 / 2 | 1 / 5 / 32 / 2 | 1 / 5 / **27** / 2 |
| CR5 / CR6 / CR7 / CR8 | 22 / 15 / **11** / 5 | 15 / 6 / 71 / 5 | **14** / 13 / 38 / 5 |
| **Change cost, all eight** | 110 | 137 | **105** |
| Final size, code lines | 164 | 174 | **154** (116 + 38) |
| Final size, bytes | **8,220** | 13,506 | 11,338 |
| Phases green on first run | 9 / 9 | 8 / 9 | 8 / 9 |
| Differential fuzz vs TypeScript | — | 0 of 1.84M | 0 of 1.83M |
| Dispatch + view, median | 1.4 µs | 202 µs | 97 µs |
| Shipped, gzipped | 2.7 KB | 413 KB | 333 KB |

**What changed in the source.**
- The label hacks are gone.
- Validation is plain `reject` rules.
- Conditions have names.
- The host sends `{"target": "pool", "sort": "duskcap"}`.
- The fade rules test qualified state directly, because a guard no longer
  leaks into grounds.
- A recovered decision is grounded on exactly its two observations, with no
  shadow state and no subtraction.

The failed run was a runtime bug: a `define` that names an event parameter
was rejected on its own. It is fixed in the runtime with a regression test.

**Where Caveat still lost: CR7, 38 lines against 11.** Two gaps remain, both
about Caveat's own subject, time and knowledge:
1. **A caveat that arrives late.** When a taste fades, everything built on it
   should carry `taste_faded`. Caveat attaches caveats at computation time,
   so the source pushes the caveat into each accumulator by hand. Keeping it
   on the right recovery window took a plain `epoch` counter.
2. **No decision journal.** The runtime knows every commitment and
   reopening in order. The host still rebuilds that history from revisions,
   relations and unordered grounds (CR6's 13 lines and 16 of CR7's).

## Round 5: late qualification and the decision journal

Round 4 left two gaps, and the language gained one feature for each:
- [Late Qualification 0.1](../../spec/caveat-late-qualification-0.1.md):
  `qualify EVIDENCE with CAVEAT`, a caveat learned after the fact that
  reaches every current value built on that evidence and leaves decisions
  already made alone.
- [Decision Journal 0.1](../../spec/caveat-decision-journal-0.1.md): every
  commitment and reopening, in order, with its evidence in observation order.

`caveat4/` starts from `caveat3/` as it stood after CR4. It replays CR5–CR8,
one commit per phase, written knowing the requests.

| Request | TypeScript | Caveat, round 3 | Caveat, round 4 | Caveat, round 5 |
| --- | --- | --- | --- | --- |
| CR5: a taste fades | 22 | 15 | 14 | **11** |
| CR6: trust has a history | 15 | 6 | 13 | **5** |
| CR7: trust recovers | **11** | 71 | 38 | 15 |
| CR8: heaviness stacks | 5 | 5 | 5 | 5 |
| **CR5–CR8** | 53 | 97 | 70 | **36** |
| **All eight requests** | 110 | 137 | 105 | **71** |
| Final size, code lines | 164 | 174 | 154 | **134** (107 + 27) |
| Final size, bytes | **8,220** | 13,506 | 11,338 | 10,153 |
| Phases green on first run | 9 / 9 | 8 / 9 | 8 / 9 | 5 / 5 |
| Differential fuzz vs TypeScript | — | 0 of 1.84M | 0 of 1.83M | 0 of 1.79M |
| Dispatch + view, median | 1.3 µs | 202 µs | 97 µs | 114 µs |
| Shipped, gzipped | 2.7 KB | 413 KB | 333 KB | 340 KB |

**What the features removed.**
- **CR5.** A fading taste is one fact: `qualify taste_$m with taste_faded`.
  The runtime finds every value that counted the taste, including the
  belief, the labels and the recovery window. The decision keeps its caveats
  because it was already made.
- **CR6.** The adapter filters the journal: one line.
- **CR7.** No epoch counter and no window test: when the recovery
  accumulator is reset, it no longer holds the taste, so a late caveat does
  not reach it. The history needed no change, because the journal already
  records every revision with its own grounds, in order.

TypeScript still wins CR7 on lines, 11 against 15. It keeps the recovery
window as a plain array. Caveat declares a decision series and two recommit
rules, in exchange for revisions whose grounds and journal entries are
correct without any code.

**What is left.**
- **Runtime cost.** Still two orders of magnitude slower than plain
  TypeScript, and about 340 KB to ship.
- **Foresight.** Rounds 4 and 5 were written knowing the requests. Only
  round 3 was blind. The next confirmation is another blind round on the
  current language.

## Reproduce

```sh
npm run build
node experiments/glowcap/harness.mjs --phase=cr4
node experiments/glowcap/harness.mjs --measure
node experiments/glowcap/harness.mjs --bench
node experiments/glowcap/differential.mjs --sequences=1000 --length=30 --seed=7
node experiments/glowcap/harness.mjs --phase=cr4 --impl=caveat2
node experiments/glowcap/differential.mjs --sequences=1000 --length=30 --seed=7 --against=caveat2
node experiments/glowcap/harness.mjs --phase=cr8 --impl=ts
node experiments/glowcap/harness.mjs --phase=cr8 --impl=caveat2
node experiments/glowcap/differential.mjs --sequences=1000 --length=30 --seed=11 --against=caveat2
node experiments/glowcap/harness.mjs --phase=cr8 --impl=caveat3
node experiments/glowcap/differential.mjs --sequences=1000 --length=30 --seed=13 --against=caveat3
node experiments/glowcap/harness.mjs --phase=cr8 --impl=caveat4
node experiments/glowcap/differential.mjs --sequences=1000 --length=30 --seed=17 --against=caveat4
```
