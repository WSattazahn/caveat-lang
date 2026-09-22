# Glowcap or duskcap — results

**Verdict: TypeScript**, by the rule fixed in [PROTOCOL.md](PROTOCOL.md) before
either implementation existed (commit `cfcc881`). Caveat had to be better on
at least two of the four measurements. It was not clearly better on any: the
two implementations tied on correctness, change cost and explanation accuracy.
On size Caveat had fewer lines but more bytes. On a tie the incumbent wins.

Caveat did not lose on quality. It matched TypeScript and was no better, and
that is not enough to justify a 1.3 MB runtime, a second repository and a
pinning pipeline.

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

## Reproduce

```sh
npm run build
node experiments/glowcap/harness.mjs --phase=cr4
node experiments/glowcap/harness.mjs --measure
node experiments/glowcap/harness.mjs --bench
node experiments/glowcap/differential.mjs --sequences=1000 --length=30 --seed=7
```
