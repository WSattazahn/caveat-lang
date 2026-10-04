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

**Round 6, blind change requests on the current language: TypeScript.**
Four new requests, committed before either side changed. Caveat changed
fewer lines, 92 against 103, but got two of four phases green on the first
run against TypeScript's four. It could not express unbounded regrowth, and
the workaround, with the round's other additions, made every event about ten
times slower: 1.1 ms, over the 1 ms limit the protocol sets. Its save and
resume is a replay of events, so resuming ten minutes of play takes 8
seconds. The gaps it found are the next language work list. Details are
[below](#round-6-blind-change-requests-on-the-current-language).

**Round 6 replay, after the language work list: lower change cost, with
remaining limits.** Written knowing CR9–CR12, it initially cost 86 changed
lines, plus two for a later adapter correction, against TypeScript's 103,
and 168 program lines against 233. All 38 scenarios
pass after fixes to save size and numeric parsing. Regrowth now uses dynamic
evidence but still stops at a declared 1,024-life limit. Size is mixed in
lines and bytes, so this is not a win under the unchanged adoption rule.
Details are [below](#round-6-replay-the-language-the-work-list-produced).

**Round 7, fresh authors on blind requests, rc.11: TypeScript.** Seven new
agents built the beat from scratch, three per side counted. Caveat changed a
third fewer lines on the four new requests (186 against 282) and was a
quarter smaller in lines, but got 2 of 4 new phases green on the first run
against 4, broke more earlier scenarios, touched more explanation code per
change, and caught 98% of mutants against 100%. No Caveat program kept its
save bounded with a fast resume in long play. Neither decision rule is met,
with C4 in place of the excluded C1 or with C1 included. Adoption turns on
one reading: counting change cost by lines alone, without its regressions,
would meet Rule A. Details are
[below](#round-7-fresh-authors-blind-requests-rc11).

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

## Round 6: blind change requests on the current language

Rounds 4 and 5 were written knowing the requests. This round asks whether the
improvements hold on requests the language was not shaped for. CR9–CR12 and
scenarios S28–S38 were committed in `653bf24`, with a prediction for each,
before either implementation changed. `caveat4/` continued from its CR8 state
with the runtime frozen at `feat/grounded-explanations` 70d4c7b. TypeScript
went first in every phase.

| | TypeScript | Caveat | Better |
| --- | --- | --- | --- |
| **1. Correctness**: phases green on first run | 4 of 4 | 2 of 4 | TypeScript |
| Harness runs / failing runs | 4 / 0 | 7 / 3 | TypeScript |
| Meets every request | yes | no: regrowth stops after eight lives | TypeScript |
| **2. Size** (final, code lines) | 233 | 162 + 46 = 208 | Caveat, −11% |
| Size (final, bytes) | 11,586 | 11,778 + 3,525 = 15,303 | TypeScript, −24% |
| **3. Change cost**, code lines, CR9–CR12 | 103 | 92 | Caveat, −11% |
| Runs that broke earlier scenarios | 0 | 2 | TypeScript |
| **4. Explanation failures** | 0 | 0 | tie |
| **5. Dispatch + view**, median / p95 | 2.6 µs / 4.8 µs | 1,090 µs / 1,467 µs | Caveat **over** the 1 ms gate |
| Resume ten minutes of play | under 1 ms | 8.0 s | TypeScript |
| Save after ten minutes | 624 bytes | 97 bytes | Caveat |
| Differential fuzz, round-6 mode | — | 0 of 1,109,220 events (300 sequences) | |

By the decision rule, Caveat is better on at most two measures (size in
lines, change cost), and each of those is offset by a worse number in the same
row group (bytes, regressions). Its dispatch cost is over the 1 ms gate as
well (median 1,090 µs and 1,115 µs on two runs), which fails the rule on its
own. TypeScript wins the round.

| Request | TypeScript | Caveat | Predicted | Outcome |
| --- | --- | --- | --- | --- |
| CR9: watching others eat | **12** | 16 | TypeScript | as predicted |
| CR10: mushrooms regrow | **28** | 50 | TypeScript | as predicted, and worse than predicted |
| CR11: say why not | 24 | **17** | Caveat | as predicted on lines; needed a second run |
| CR12: save and resume | 39 | **9** | TypeScript | **failed** on lines; right on runtime cost |

**CR9.** A second way to observe a mushroom meant restating the twelve
absorb rules for `witness`, because procedures take numbers and the rules
differ only in which evidence they name. TypeScript called its existing
`learn(evidence, kind)`. Explanations cost nothing on either side: the
`secondhand` caveat reached labels, the belief and the decision through a
qualified value in Caveat and through the caveat map in TypeScript.

**CR10.** This one breaks the frozen language. Evidence is declared, and a
mushroom that regrows is a new mushroom with new evidence, without limit.
Caveat's only route is to declare the lives: 28 entities (`cave_2` …
`grove_8`), so `for mushroom as $m` generates their evidence, with the
adapter routing each event to the current life. It passes every scenario,
which go up to six lives, but a ninth life is refused (checked directly:
TypeScript regrows a ninth time, Caveat reports the mushroom still eaten).
The runtime re-evaluated every binding of every declared entity after every
event, so eight times the entities made the median event about seven times
slower: 114 µs at CR8, 828 µs after CR10, and 1,090 µs after CR11 added seven
bindings per life. (This paragraph first blamed the rules. A profile made
afterwards found binding evaluation was 510 of 611 µs per event and the 1,600
rule guards about 30 µs.)
Two of the three failing runs were adapter bugs in this
phase: a name collision and a regular expression that lost its backslash.

**CR11.** The reasons are ordinary bindings with `because`, and the evidence
that consumed a mushroom is simply the grounds of its `consumed` value, so
"Already eaten, because another creature ate it (secondhand)" needed no
bookkeeping. TypeScript added a `consumedBy` field and a function that
restates the rule order. The failing run was an ordering trap: `qualified(1,
absorb_$m)` placed before the rule that reveals `absorb_$m` in the same event
is an error. Moving it after the reveal fixed every scenario.

**CR12.** The runtime has no way to restore a session, so the adapter saves
the accepted events, with runs of equal ticks folded together, and replays
them on resume. That is 9 lines against TypeScript's 39, and a 97-byte save.
But resume costs as much as replaying the whole session: 8.0 seconds for ten
minutes of play, growing linearly. The prediction was wrong about lines and
right about the thing that matters in a game.

**The work list this round leaves.** Each item is a gap a real game hits:

1. **Identity that is born at runtime.** Regrowth, spawning, a second copy of
   an item: evidence and entities need a declared family whose members are
   created by events, not declarations.
2. **Save and restore in the runtime.** A session snapshot of values,
   provenance, grounds, relations, commitments and the journal, restored in
   time proportional to the state rather than the history.
3. **Cost that follows the event, not the program.** Declaring more entities
   should not slow every event. (As first written, this item proposed indexing
   rules by target; the profile above showed the cost was binding evaluation.)
4. **Procedures over evidence.** `proc learn(e)` taking an evidence name
   would have made CR9 three lines.
5. **The ordering trap.** Qualifying with evidence that the same event
   reveals later should either work or be caught when the program loads.

## Round 6 replay: the language the work list produced

The language now has features addressing the five work-list items:

- [incremental evaluation](../../spec/caveat-incremental-evaluation-0.1.md);
- the [observation order](../../spec/caveat-observation-order-0.1.md) check;
- [procedure symbols](../../spec/caveat-procedure-symbols-0.1.md);
- [renewal](../../spec/caveat-renewal-0.1.md), with scheduled qualification
  and `carries`;
- [save](../../spec/caveat-save-0.1.md).

`caveat5/` starts from `caveat4/` exactly as it stood after CR8 and replays
CR9–CR12 on that language, one commit per phase. **This replay was written
knowing the requests**, like rounds 4 and 5: it shows what the new language
costs on these requests, not how it does on requests it was not shaped for.

The replay verification and timing outputs are preserved in
[`evidence/round6-replay.json`](evidence/round6-replay.json), including the
runtime revision, WebAssembly hashes, host, commands and both fuzz seeds.
The blind column retains its original measurements. The TypeScript and
replay timing columns use the current build, measured after the fuzz finished.
The replay ships 482,716 bytes gzipped (policy, adapter and runtime), against
TypeScript's 3,470. The raw current-build benchmark also includes `caveat4`;
that is the old program on the new runtime and does not replace the historical
blind result.

| | TypeScript | Caveat, round 6 (blind) | Caveat, replay |
| --- | --- | --- | --- |
| CR9: watching others eat | **12** | 16 | 30 |
| CR10: mushrooms regrow | **28** | 50 | 32 |
| CR11: say why not | 24 | **17** | 18 |
| CR12: save and resume | 39 | 9 | **6** |
| **CR9–CR12, code lines changed** | 103 | 92 | **86** |
| Later adapter correction, code lines changed | — | — | 2 |
| Total including that correction | 103 | 92 | **88** |
| Final size, code lines | 233 | 208 | **168** (135 + 33) |
| Final size, bytes | **11,586** | 15,303 | 12,701 |
| Phases green on first run | **4 / 4** | 2 / 4 | 3 / 4 |
| Meets every request | yes | no: eight lives | no: 1,024-life limit |
| Dispatch + view, median / p95 | 2.5 / 4.1 µs | 1,090 / 1,467 µs | 51.6 / 59.1 µs |
| Resume ten minutes of play | 0.071 ms | 8.0 s | 3.24 ms |
| Save after ten minutes | 624 bytes | 97 bytes | 2,045 bytes |
| Differential fuzz, round-6 mode | — | 0 of 1.11M events | 0 of 14.44M events (ordered) |

**CR9.** `proc learn(e evidence, sort)` holds what an observation teaches.
Absorbing and witnessing each call it with their own evidence, and
`secondhand qualifies witness_$m` states the difference once. The count, 30,
is the price of moving the nine absorb rules into the procedure: every moved
line counts as removed and added. After it, a new way of observing costs a
call.

**CR10.** Each mushroom's three evidence names are `renewable`. Regrowth
renews them and resets the mushroom's per-life states; the source and glue
spell `absorb_cave@2` where the protocol says `absorb_cave_2`, a one-line
rename. The fade became `qualify taste_$m with taste_faded after 60`, bound to
the taste it is about, and the faded labels ask `carries(taste_$m,
taste_faded)`. That removed two timer states and two tick rules. Regrowing the
cave mushroom twelve times now matches TypeScript: twelve absorptions, twelve
supporting observations. Round 6's Caveat stopped at eight. The bound is now
1,024 lives per mushroom, declared in the source. This removes the static
eight-life workaround, but does **not** meet CR10's literal requirement of
regrowth "with no limit". Exhausting the declared renewal limit rejects the
event; the finite scenarios and fuzz do not establish unlimited regrowth.

**CR11.** The same bindings as round 6. The consumption rule is written after
the call that reveals the evidence. The [observation order](../../spec/caveat-observation-order-0.1.md)
check now rejects round 6's first attempt when the program loads, naming both
rules.

**CR12.** The adapter saves with the runtime's own save and resumes with
`WebReactiveSession.restore`. Its first run failed one scenario: S36's second
save was 4,257 bytes, over the 4 KB limit, because the save format wrote every
state and its grounds twice. The format was then made compact (unchanged
states left out, grounds stored once, empty lists omitted), and the phase
passed. That failed run counts against the replay's correctness, and the
format change is part of the language, not the adapter (`61146b4`). The
unchanged policy/glue hash in `runs.jsonl` does not identify the runtime;
runtime changes can therefore turn a failing run into a passing run with
the same recorded hash.

**The differential failure and repair.** The first 2,000-sequence replay fuzz
(seed 6) found five divergences in 7,276,500 events: two missing faded-taste
caveats and three different mushroom explanation lists. The default
`serde_json` parser changed some floating-point numbers by one unit in the
last place. A changed saved clock or host-sent number could move a `>= 45`
or `>= 60` boundary to a different tick. Enabling `float_roundtrip` in
`6c4b504` fixes that shared parse path. The two regression tests check exact
bits after save/restore and after host event parsing; both were shown to fail
without the feature and pass with it during the original investigation.

The five captured failures are retained in
[`fixtures/round6-float-divergences.json`](fixtures/round6-float-divergences.json),
with identical adjacent ticks compressed by a `repeat` count.
`replay-divergences.mjs` checks accept/reject and the view after **each**
event, including each tick: 16,011 events and 13 resumes agree on the final
adapter and runtime.
The full fuzz checks accept/reject for every event and views at each step or
tick-burst endpoint. After the precision fix, both seeds passed in the original
comparison mode: seed 6 ran 7,280,140 events and seed 7 ran 7,163,331, each
across 2,000 sequences with zero divergences. A failing sequence stops early
and thus changes later random-number consumption: rerunning the same seed
after a fix does not replay every later sequence identically. The saved-case
replay above is the exact regression check for all five original failures.

**A comparison gap found during final review.** The original fuzz sorted
every top-level evidence array, including `decision.basis` and
`decision.reopenedBy`. CR7 requires the basis in observation order, and the
decision contract requires ordered reopening witnesses. A stricter replay
of the saved failures exposed an adapter bug at sequence 1510, event 1820:
after tasting pool then grove, its basis was `[taste_grove, taste_pool]`.
The runtime's grounds are a set, while its decision journal already had the
correct `[taste_pool, taste_grove]` order. The adapter now reads that
revision's committed journal entry. This costs two changed code lines after
the original 86-line replay; the failed order-sensitive check remains in the
evidence record.

The new shared comparator preserves the order of the basis, reopening
witnesses, journal entries and their evidence. Focused tests show that it
catches reordered lists while accepting set-valued evidence in either order.
`replay-divergences.mjs` uses it, and `differential.mjs --ordered-decisions`
enables it without changing the original comparison mode. Both seeds also
passed after the adapter correction with this stricter mode: 4,000 sequences,
14,443,471 events, zero divergences.

**Other verification.** All 418 Rust tests pass, as do formatting, Clippy
with warnings denied, the seven comparator tests, and all 38 scenarios on
each implementation. The extended altered-save test completed 30,000
mutations without a crash. The Glowcap and Slime Glow WebAssembly checks
pass. The real Chromium explainer test also passes locally, including its
labels, trust journal, late caveats and absence of browser errors; the saved
screenshot was inspected. Feature-branch pushes alone do not run browser CI
(the workflow runs on main or pull requests), so this browser result is local.

**By the unchanged decision rule** change cost is a clear Caveat win
(88 against 103, including the later correction). Size is mixed: 168 lines
against 233, but 12,701 bytes against 11,586, just as fewer lines but more bytes
was mixed in round 1. Explanations are level, and first-run correctness is
behind (3 of 4 phases against 4 of 4). Its dispatch is under the 1 ms gate at
51.6 µs median. This does not supply the two clear wins required for adoption,
and the unlimited
regrowth request remains unmet. Round 6 itself stays a TypeScript win. The
replay measures progress on known requests; another blind round is needed
to test whether that progress generalizes.

## Round 7: fresh authors, blind requests, rc.11

Rounds 3 and 6 were blind to the requests but had one author per side, who
already knew the beat. This round asks how the language fares with authors
who have never seen it. Seven fresh agents built Glowcap from scratch, phase
by phase, on the published `caveat-lang@0.1.0-rc.11` (tarball SHA256
`13fd6e29…a8e`) or in plain TypeScript 7.0.2 (strict). The rules were fixed
before any agent started, in [round7/PROTOCOL.md](round7/PROTOCOL.md), with
hashes in `registration.json` (`df3dde3`), `registration-2.json` (`01d4352`,
the new requests, scenarios S39–S50, fuzz mode and predictions) and
`registration-3.json` (the wave-2 prompts). CR1–CR12 are inherited phases;
CR13–CR16 are new requests a separate agent wrote and a second agent
reviewed, and they decide the round.

**Authors and the incident.** C1, C2, T1 and T2 ran in wave 1; C3, T3 and C4
in wave 2. At CR13 C1 restored a backup from the session scratchpad, which
every agent shares, that C2 had overwritten with its own adapter, and ran one
test with it before rewriting its own. C1 disclosed it. Under amendment 2 C1
is out of the primary counts and kept in the record; under amendment 3, the
owner's decision, C4 replaced it. C2's snapshots contain none of C1's code,
so C2 stays in. Every measure below is reported three ways: **primary**
(C2, C3, C4), **registered** (C1, C2, C3, the authors the protocol first
named) and **all four** Caveat authors. TypeScript is T1, T2 and T3 in each.
Each figure is the median of the side's authors; a margin under 10 percent
is a tie, and a tie goes to TypeScript.

### Measurements

Per author (blind phases CR13–CR16 unless stated):

| | C1 | C2 | C3 | C4 | T1 | T2 | T3 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Blind phases green on first run, of 4 | 2 | 2 | 2 | 3 | 0 | 4 | 4 |
| Blind runs to reach all-green | 9 | 6 | 9 | 7 | 8 | 4 | 4 |
| Blind failing runs | 5 | 2 | 5 | 3 | 4 | 0 | 0 |
| Inherited phases green on first run, of 13 | 13 | 13 | 12 | 12 | 13 | 13 | 13 |
| Code lines, end of CR16 (policy + glue) | 228 + 95 | 256 + 110 | 275 + 106 | 254 + 113 | 487 | 500 | 496 |
| Bytes, end of CR16 | 19,390 | 21,700 | 21,480 | 19,387 | 20,982 | 22,395 | 22,520 |
| Change cost, code lines, CR13–CR16 | 153 | 194 | 186 | 173 | 282 | 148 | 477 |
| Regressions during CR13–CR16 | 75 | 22 | 11 | 0 | 3 | 0 | 0 |
| Explanation failures (scenario ids) | 22 | 0 | 0 | 0 | 2 | 0 | 0 |
| Dispatch + view, median µs | 108 | 152 | 162 | 145 | 16.5 | 27.8 | 17.9 |
| Dispatch + view, p95 µs | 148 | 204 | 217 | 193 | 32.8 | 47.2 | 33.9 |
| Resume ten minutes of play, ms | 19.4 | 744 | 15.9 | 13.8 | 0.60 | 0.75 | 0.59 |
| Save after ten minutes, bytes | 673 | 113 | 834 | 877 | 220 | 178 | 209 |
| Gzipped bytes a host ships | 614,587 | 615,245 | 614,996 | 614,483 | 6,592 | 6,727 | 6,973 |
| Explanation drift, units, CR13–CR16 | 70 | 34 | 53 | 51 | 27 | 28 | 40 |
| Mutation score, overall | 98.5% | 98.4% | 97.9% | 97.7% | 100% | 100% | 100% |

Medians, and which side each favours:

| | Primary C2–C4 | Registered C1–C3 | All four | TypeScript | Better (all three readings) |
| --- | ---: | ---: | ---: | ---: | --- |
| **1. Correctness**: blind first-run green | 2 | 2 | 2 | 4 | TypeScript |
| Blind runs to all-green | 7 | 9 | 8 | 4 | TypeScript |
| **2. Size**, code lines | 367 | 366 | 366.5 | 496 | Caveat, −26% |
| Size, bytes | 21,480 | 21,480 | 20,435 | 22,395 | tie (−4%, −4%, −9%) |
| **3. Change cost**, code lines | 186 | 186 | 179.5 | 282 | Caveat, −34% (−36% for all four) |
| Regressions | 11 | 22 | 16.5 | 0 | TypeScript |
| **4. Explanation failures** | 0 | 0 | 0 | 0 | tie |
| **5. Dispatch + view**, median µs | 152 | 152 | 148 | 17.9 | TypeScript; both under 1 ms |
| Gzipped bytes shipped | 614,996 | 614,996 | 614,792 | 6,727 | TypeScript |
| **6. Mutation score** | 97.9% | 98.4% | 98.2% | 100% | tie under the margin; TypeScript caught all |
| **7. Explanation drift** | 51 | 53 | 52 | 28 | TypeScript |

Size counts policy plus glue by `harness.mjs --measure`'s rule. Change cost is
`git diff --numstat` code lines per blind request. A regression is a
previously passing scenario that failed in some run during the request. An
explanation failure is a scenario whose `because`, `supportedBy`,
`contradictedBy`, `basis`, `reopenedBy`, `caveats`, `why` or history
assertion failed in some run; C1's 22 came from one CR13 run, T1's two from
CR15. The timings are the existing 10k-event replay; the shipped bytes are
the rc.11 runtime the adapter loads against a TypeScript file with no
dependencies. Raw numbers are in `round7/results/score.json`.

### Verdicts

**Rule A, adoption (unchanged): not met, under all three readings.** Caveat
must be better on at least two of measures 1–4, no more than 2× worse on any,
with median dispatch under 1 ms. It is better on size (lines better, bytes a
tie). Correctness favours TypeScript (2 of 4 blind phases green on the first
run against 4). Explanations tie. Change cost is the reading that matters:
Caveat changed a third fewer lines, but its authors broke earlier scenarios
along the way (median 11 regressions in the primary count against 0). This
report reads a measure as better only when no number in its group is worse,
as round 6 read change cost and size ("each of those is offset by a worse
number in the same row group", above). Read on lines alone, change cost would
count for Caveat, and Caveat would be better on two measures; correctness
is within 2× (first-run green exactly 2×, runs to green 1.75×) and dispatch
is under 1 ms, so Rule A would then be met. The recorded reading was not
chosen for this round, but the verdict turns on it, and that is stated here
rather than left for a reader to find.

**Rule B, the stated claim: not met, under all three readings.** It needs all
of: a higher mutation score, lower drift, lower blind change cost, measure 1
and measure 4 no worse, and dispatch under 1 ms. 98.4%ULEB Drift is higher
for Caveat (51 against 28), and blind first-run correctness is worse (2
against 4). Change cost in lines is lower and dispatch is under 1 ms.

**The C1 exclusion is inert.** With C4 in place of C1, with C1 in place of
C4, and with all four, every measure favours the same side and both verdicts
are the same.

### What the suite did not see

The registered scenarios are the suite. Three post-hoc checks, all on the
frozen CR16 programs, found what it missed.

**Long play and the save bound.** CR13 says the 4,096-byte save bound "now
holds for play of any length", and CR16 repeats it. `round7/longplay.mjs`
plays 10, 30 and 60 regrowth cycles of every mushroom and saves:

| Cycles | C1 | C2 | C3 | C4 | T1 | T2 | T3 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 2,349 | 225 | 3,714 | 5,869 | 506 | 338 | 395 |
| 30 | 4,277 | 265 | 8,790 | 18,933 | 514 | 345 | 402 |
| 60 | 6,753 | 321 | 16,310 | 40,661 | 514 | 345 | 402 |
| Resume at 60, ms | 60 | 112,277 | 53 | 150 | 1.2 | 1.5 | 1.5 |

Every program resumed to the same view. The three TypeScript saves stop
growing. Three Caveat saves pass the 4,096-byte bound only in the scenarios:
the rc.11 session save keeps the whole evidence history (relations, renewal
occurrences, the decision journal, every revision's basis), as the language
intends, and C1, C3 and C4 compress it in the adapter, which slows its
growth without bounding it. C2 instead saves the accepted events and replays
them, which keeps the save small and makes resume proportional to play: 744
ms after ten minutes, 112 seconds after 60 cycles. C3's and C4's notes name
this as a known departure from CR13; C4 calls it a candidate feature request
(bounded or compactable history). No Caveat author met CR13's "any length"
with a fast resume. Every TypeScript author did.

**Differential fuzz.** `round7/fuzz.mjs`, registered before any author
started, ran 300 seeded sequences (seed 7, 846,913 events) through all seven
programs against T2, stopping a program's sequence at its first difference:

| Program | Sequences that diverged | First differences |
| --- | ---: | --- |
| T1, T3 | 0 | |
| C1 | 40 | `decision.basis` 39, `belief.caveats` 1 |
| C2 | 1 | `belief.caveats` 1 |
| C3, C4 | 40 each | `journal` 39, `belief.caveats` 1 |

The 39 basis and journal differences are one bug class: when trust is
committed again under CR7, the basis must be the two observations "in order".
C1's `decision.basis`, and the "I trust glowing mushrooms now." journal
entry in C3 and C4, list them in the wrong order when a taste precedes an
absorption (in the first example, `[absorb_pool, taste_cave_2]` for
`[taste_cave_2, absorb_pool]`). The rc.11 runtime's grounds are a set, and
round 6's replay hit the same trap (above). The suite has no case that
reaches it.

The single `belief.caveats` difference is the same event in all four Caveat
programs: a taste fades one 0.0625 s tick late. CR5 counts sixty seconds in
ticks since the taste; in the example exactly 960 ticks of 0.0625 s follow
the taste, so all three TypeScript programs report `taste_faded` and all four
Caveat programs report it one tick later. It reproduces without any resume
(`round7/fuzz-checks/fade-boundary.mjs`). The likely cause, inferred and not
traced in the runtime, is that `qualify … after 60` is scheduled on the
session's accumulated clock, where earlier inexact ticks (0.05, 0.1) leave the
sum one unit in the last place short of the taste time plus 60. It is a
runtime behaviour, not any author's error, and it is reported to the Version
Lab register rather than given a number here.

### Mutation score

A separate mutator agent listed every applicable site per program before
any mutant ran (`round7/mutation/sites.json`, 318 sites, frozen), then wrote
one mutant per site (`round7/mutation/mutants/<program>/<operator>-<site>/`,
each a `CHANGE.md` and a one-line diff). `round7/mutate.mjs` ran each one
through the static check (`caveat check` or `tsc`), the runtime (a load
error, or an event the original accepts being refused) and the registered
suite, then fuzzed every survivor against its original (100 sequences, seed
11). Results are in `round7/results/mutation.json`.

| Program | M1 | M2 | M3 | M4 | M5 | Overall | Static / runtime / suite |
| --- | --- | --- | --- | --- | --- | --- | --- |
| C1 | 9/9 | 2/2 | 1/1 | 0/1 | 54/54 | 66/67, 98.5% | 54 / 0 / 12 |
| C2 | 10/10 | 2/2 | 1/1 | 0/1 | 47/47 | 60/61, 98.4% | 47 / 0 / 13 |
| C3 | 10/10 | 2/2 | 1/1 | 0/1 | 34/34 | 47/48, 97.9% | 34 / 0 / 13 |
| C4 | 9/9 | 2/2 | 1/1 | 0/1 | 31/31 | 43/44, 97.7% | 31 / 0 / 12 |
| T1 | 5/5 | 1/1 | 2/2 | 1/1 | 25/25 | 34/34, 100% | 0 / 1 / 33 |
| T2 | 5/5 | 1/1 | 2/2 | 1/1 | 24/24 | 33/33, 100% | 0 / 0 / 33 |
| T3 | 3/3 | 1/1 | 2/2 | 1/1 | 24/24 | 31/31, 100% | 0 / 0 / 31 |

Both sides caught every dropped caveat (M1), unfrozen basis (M2), skipped
reopen (M3) and citation of unread evidence (M5). They differ in where.
Every Caveat M5 mutant was refused by `caveat check` before it ran, because
a `because` must cite what the value reads. Every TypeScript M5 mutant
compiled and was caught only because a scenario asserted that list. With a
weaker suite, TypeScript's M5 catches would fall and Caveat's would not.

The only survivors are the four Caveat M4 mutants, which swap the order in
which the two observations enter the recovery commitment. The fuzz could not
tell any of them from its original, so each is labelled possibly
equivalent: the runtime holds a commitment's grounds as a set, so the order
in the source has no effect. That is the same property behind the CR7 order
bugs the fuzz found in C1, C3 and C4 above. A Caveat author cannot reorder a
basis by editing the source, and cannot fix its order there either.

The overall scores differ by about 2 percentage points, a tie under the 10
percent margin, so TypeScript keeps it. A Caveat program has about one and
a half times as many sites as a TypeScript one, mostly M5 sites, because
each `because` is a site.

### Explanation drift

A drift agent counted, after each blind request, the explanation-producing
units each diff touched (`round7/drift/DRIFT.md`, unit lists in
`DRIFT.json`). Primary medians: CR13 18 against 10, CR14 8 against 7, CR15 3
against 3, CR16 22 against 7. The agent counted save and restore code as
feeding explanations, since after a resume every `because` comes from the
restored data. Without it, CR13 is C2 8, C3 19, C4 18 (median 18) against T1
6, T2 8, T3 14 (median 8), and the totals still favour TypeScript.

### Learnability

Caveat authors learned the language from the rc.11 package's documents and
type declarations alone. Two of the three primary Caveat authors failed one
inherited phase on its first run, both at CR12 and both on S36, the
4,096-byte save bound: the runtime's own save was over the bound. C3
restructured its program to stop evidence lineage spreading into the save;
C4 compressed the save in the adapter. These are language failures (the
save's size), not glue or documentation. The disclosed Glowcap fragments in
the Caveat specifications favour the Caveat side on CR1–CR12, so the
inherited-phase figures flatter it, if anything.

In the blind phases the Caveat failures had two main causes. At CR13 (S41)
it was the save bound again: C2 moved to an event-replay save, C3 to a
compressed one. At CR16 it was the journal. C2 and C3 first wrote bindings
that cited a journal slot their value did not read, which the runtime
refuses as `ungrounded_citation`, a rule of the language working as
designed. C4 found that passing a qualified value through a numeric
procedure parameter gave a slot the grounds of earlier lives of the same
evidence, and set the slot directly instead (its notes say it did not
verify why). All three then had to rebuild observation order in the
adapter, because lists are "compared in order" and the runtime reports
grounds as sorted sets. C1's 22 explanation failures and its CR13
regressions all come from the two runs of snapshot `8cdfd394b1d2`, the
incident run with C2's adapter; its 0/50 first CR16 run accounts for the
rest of its regressions.

### Predictions

Recorded by Claude for the owner before any author started
([round7/PREDICTIONS.md](round7/PREDICTIONS.md)); primary medians.

| Prediction | Caveat | TypeScript | Outcome |
| --- | ---: | ---: | --- |
| CR13: TypeScript smaller diff and fewer failed runs | 77 lines, 1 failed run | 163, 0 | **failed** on the diff; held on failed runs |
| CR14: tie | 22 | 41 | **failed**: Caveat −46% |
| CR15: Caveat smaller diff | 12 | 13 | **failed**: a tie (−8%) |
| CR16: TypeScript clearly better | 76 lines, 2 failed runs | 41, 0 | held |
| Mutation: Caveat higher on M5 | 100% | 100% | **failed**: both caught every M5 mutant (Caveat statically, TypeScript in the suite) |
| Drift: TypeScript lower on CR13 and CR16 | 18, 22 | 10, 7 | held |
| Drift: Caveat lower on CR15 | 3 | 3 | **failed**: a tie |

### What this round can and cannot show

It shows how rc.11 fares with seven fresh agents of one model family on one
beat: fewer lines and a cheaper diff on the Caveat side, and more failed
runs, more regressions, more drift, a slower and larger runtime, and a save
that does not stay bounded on the Caveat side. TypeScript is the default.
It cannot show how people would fare, or other kinds of program. All agents,
the request writer and reviewer, the mutator, the drift counter and the
drafter of the protocol are the same model family. TypeScript is a language
these agents know far better. The Caveat packet's specifications contain
Glowcap fragments (disclosed before the round). Caveat glue was hand-written,
because rc.11 has no `caveat types`, delta view or unchanged-guard skipping,
which rc.12 plans. One author on each side was replaced or excluded for an
isolation breach, and the exclusion changes no verdict. Mutation sites
differ by language (a Caveat program has more places to drop a caveat), so
the per-operator scores matter more than the overall one.

No claim that Caveat is better than TypeScript goes into the README, AGENTS,
the site or other documents on the strength of this round.

**The work list this round leaves.**

1. **A save that forgets what the program forgot.** CR13's window drops
   evidence from the belief, but nothing drops it from the session. A bounded
   or compacting save, under the rule that evidence history is preserved,
   needs a design decision about what "forgotten" means for the record.
2. **Ordered grounds.** Every Caveat author had to rebuild observation order
   in the adapter, and three got the CR7 basis or its journal entry wrong in
   ways the suite did not catch.
3. **A fade that lands on its tick.** Scheduled qualifications should fire on
   the tick where the elapsed ticks reach the delay, independent of how the
   session clock was accumulated.
4. **The glue.** Between 95 and 113 lines per Caveat program are adapter, and
   most blind-phase failures were there.

### Reproduce (round 7)

The authors' programs live outside the repository at run time; each one's
final source is in `round7/runs/<ID>/impl/`, with its phase commits in
`phases.bundle` and every test snapshot in `snapshots.tgz`.

```sh
cd experiments/glowcap/round7
node prepare.mjs authors --tarball=caveat-lang-0.1.0-rc.11.tgz   # checks its hash, installs it
# then copy each runs/<ID>/impl/ to /home/claude/glowcap-r7/authors/<ID>/impl/
node score.mjs --bench            # measures 1–5 → results/score.json
node longplay.mjs                 # → results/longplay.json
node fuzz.mjs --impl=…/T2/impl/glowcap.ts --impl=… --sequences=300 --seed=7 --dump=results/fuzz-seed7.json
node mutate.mjs > results/mutation.json
node fuzz-checks/fade-boundary.mjs
```

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
node experiments/glowcap/harness.mjs --phase=cr12 --impl=ts
node experiments/glowcap/harness.mjs --phase=cr12 --impl=caveat4
node experiments/glowcap/differential.mjs --round=6 --sequences=300 --seed=6 --against=caveat4
node experiments/glowcap/harness.mjs --phase=cr12 --impl=caveat5
node experiments/glowcap/replay-divergences.mjs
node experiments/glowcap/differential.mjs --round=6 --sequences=2000 --seed=6 --against=caveat5
node experiments/glowcap/differential.mjs --round=6 --sequences=2000 --seed=7 --against=caveat5
node --test experiments/glowcap/compare-views.test.mjs
node experiments/glowcap/differential.mjs --round=6 --sequences=2000 --seed=6 --against=caveat5 --ordered-decisions
node experiments/glowcap/differential.mjs --round=6 --sequences=2000 --seed=7 --against=caveat5 --ordered-decisions
# Run timings after both fuzz processes finish.
node experiments/glowcap/harness.mjs --bench
node experiments/glowcap/resume-bench.mjs
npm run test:glowcap
npm run test:glowcap-page
```
