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

**Round 8, fresh authors on blind requests, rc.17: TypeScript.** Six new
agents built the beat from scratch under round 7's rules, three per side,
with no run excluded. Caveat kept pace on the inherited requests and was
cheaper on the countdown request, but two new requests needed a caveat that
can be taken back or attached to an earlier life, which rc.17 cannot do, and
every Caveat author rebuilt that with flags. Over the four new requests
Caveat changed 484 lines against 148, got 2 of 4 phases green on the first
run against 4, and caught 62% of mutants against 95%. Neither decision rule
is met, under any reading. Details are
[below](#round-8-fresh-authors-blind-requests-rc17).

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
assertion failed in some run; C1's 22 all came from the incident
snapshot at CR13 (below), and T1's two from CR15. The timings are the existing 10k-event replay; the shipped bytes are
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
and measure 4 no worse, and dispatch under 1 ms. Drift is higher
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
runtime behaviour, not any author's error. The Version Lab records it as
F267: correct binary64 arithmetic under the stated rounding, which rc.12
documents as a firing rule (a schedule may fire one clock event late, never
early) rather than changes.

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

### Learnability: why each Caveat run failed

The protocol asks which Caveat failures were glue, which were language and
which were documentation. `round7/fuzz-checks/first-failures.mjs` re-runs
every failing run's snapshot against its phase's scenarios and prints the
first failure of each failed scenario; the table classifies every failing
Caveat run that way. "First" marks a phase's first run, the one measure 1
counts.

| Author | Phase, run | Scenarios failed | Cause | Kind |
| --- | --- | --- | --- | --- |
| C2 | CR13, first | S41 | save 4,721 bytes over the 4,096 bound | language: the session save keeps all evidence history |
| C2 | CR16, first | 25 | `ungrounded_citation`: the journal binding cited evidence its value did not read; also S48's caveat order (below) | language rule, met by the author on the second run |
| C3 | CR12, first | S36 | save 4,226 bytes | language: the session save |
| C3 | CR12, second | 36 | `ungrounded_citation` in the why bindings after a restructuring | language rule |
| C3 | CR13, first, second, third | S41 | save 4,210, 5,542 and 4,164 bytes | language: the session save |
| C3 | CR16, first | 13 | `ungrounded_citation` in a journal binding; S48's caveat order | language rule; scenario |
| C3 | CR16, second | S48, S50 | caveat order inside journal entries only | scenario |
| C4 | CR12, first | S36 | save 4,333 bytes as wrapped; the runtime's own text was 3,764 | glue: the adapter's JSON wrapper pushed it over |
| C4 | CR16, first | S48, S49, S50 | S49 and S50: an entry cited earlier lives of renewed evidence (`absorb_cave` with `absorb_cave_2`) after the value went through a numeric procedure parameter; S48: caveat order | language (C4's notes say it did not verify why); scenario |
| C4 | CR16, second and third | S48, S50 | caveat order inside journal entries only | scenario |
| C1 | CR13, first | S41 | save 5,492 bytes | language: the session save |
| C1 | CR13, two runs | 31 | the incident snapshot `8cdfd394b1d2`, with C2's adapter | incident |
| C1 | CR16, first | 50 | load error: a function used an undeclared identifier | author error in source |
| C1 | CR16, second | S48, S50 | caveat order inside journal entries only | scenario |

**Language.** Most first-run failures are the session save. It keeps every
relation, renewal occurrence, journal entry and revision basis, so the
4,096-byte bound fails first at CR12 or CR13 and fails for good in long play
(above). The second is `ungrounded_citation`, the rule that a `because` may
cite only what its value reads. Authors met it while building the journal
and fixed it on the next run. It is the same rule that caught every M5
mutant statically.

**Glue.** One first-run failure is glue alone: C4's JSON wrapper took a save
under the bound over it. The adapters' larger cost shows in size, change
cost and drift, not in failed runs.

**Documentation.** None of the failures traces to a gap in the Caveat
documents that I could establish. The packet does not state the save's size
or growth. Authors found that by measuring.

**The round's own scenarios.** S48 and S50 expect each journal entry's
`caveats` in one order, `["tasted_in_dark", "taste_faded"]`, the order the
caveats arrived in. CR16 defines `caveats` as a union and says nothing of
order. The registered fuzz compares it as a set. The scenario comparator
compares nested lists in order, because its sort treats a list of objects as
already sorted. Every Caveat author got the order wrong at least once, since
the runtime lists caveats sorted by name, and fixed it in the adapter. This
changed no first-run outcome: each of those first runs failed for another
reason as well. It did add four failing runs (C1 one, C3 one, C4 two).
Without them, the primary median blind runs to all-green is 6 against
TypeScript's 4, rather than 7. It is a defect in this round's registration,
not in either language, and it is left as registered.

**F267, the late fade.** No scenario failure in any run comes from the
one-tick-late fade. Every `taste_faded` mismatch in the table is an order
difference with the caveat present on both sides. The scenarios' fades sit
on exact 0.0625-second clocks, where the rounding cannot occur, so the fade
appears only in the fuzz. The project now records it as F267, a binary64
firing rule that rc.12 documents rather than changes. One sequence per
Caveat program is all it cost, and only in the fuzz.

**On the TypeScript side**, T2 and T3 failed no run. T1's four failing blind
runs were a resume bug at CR13, and at CR14, CR15 and CR16 a test run before
the request's code existed ("unknown event type mark", no `pit`, no
`journal`). The protocol counts the first run whatever it was, so T1's 0 of
4 is partly how T1 worked rather than what TypeScript made hard.

The disclosed Glowcap fragments in the Caveat specifications favour the
Caveat side on CR1–CR12, so its inherited-phase figures flatter it, if
anything.

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
which rc.12 plans. One Caveat author was excluded for an isolation breach
and replaced, and the exclusion changes no verdict. The round's scenarios
compare caveat order inside journal entries, which CR16 leaves unspecified;
that added four Caveat failing runs and changed no first-run outcome. Mutation sites
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
3. **Timing to the tick.** F267's firing rule is now documented, not
   changed: a schedule fires on the first clock event at which the elapsed
   time reaches its delay in binary64, so it can be one event late. A program
   that needs exact ticks needs integer tick time, a later design question.
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

## Round 8: fresh authors, blind requests, rc.17

Round 8 repeats round 7 under the same registered rules, on the published
`caveat-lang@0.1.0-rc.17` (tarball SHA256 `03bc86cf…91b6`, commit
`304a16d`). It is step R2 of the "Round 8" table in
[the path to 1.0](../../docs/releases/path-to-1.0.md). The rules are
[round8/PROTOCOL.md](round8/PROTOCOL.md): round 7's protocol word for word,
except for the eleven rows it changes. It was registered in four hashed
stages before any author started: `registration.json` (`d42b70e`), `-2`
(`4fcbca7`), `-3` (`b23c0cc`, the requests CR17–CR20, their scenarios and the
per-request predictions) and `-4` (`57686de`, the rc.17 pin, packet, runner,
comparator, fuzz mode and prompts). CR1–CR16 are inherited phases. CR17–CR20
are new requests that a fresh agent wrote and a second agent reviewed, and
they decide the round. Each author implemented 21 phases.

**Authors.** Six fresh agents started together at 05:27 UTC on 2026-10-11
with the registered prompts: C1, C2 and C3 on rc.17, and T1, T2 and T3 in
TypeScript 7.0.2 (strict). Node was 22.22.0. All six committed every phase
with all of its scenarios passing; none used `--incomplete`. Tokens, tool
uses and durations are in `round8/launches.json`.

**Isolation.** No run is excluded under row 7. Every author's transcript was
searched for tool calls naming another author's directory, the private store,
the mutator, drift, writer or reviewer directories, the task output folder,
the session scratchpad, round 7, the repository, or the package's runtime,
`lib/*.mjs` or `examples/`. None did, apart from C2 reading the output of its
own background command. Three things are disclosed:

- C1, T2 and T3 read `run.mjs` in their own directory. `README-author.md`
  does not list it, but it holds no other author's work.
- C2 was shown harness notices that C1 and C3 had finished, naming their
  output files. It did not open them.
- In CR13 and CR16 C2 added a size-logging hook to its adapter and ran it
  through `run.mjs`, then removed it. Those runs are in its log.

So every figure below is the registered set, C1–C3 against T1–T3, and there
is no second reading. Each figure is the median of the side's three authors.
A margin under 10 percent is a tie, and a tie goes to TypeScript.

**The probe.** The learnability probe ran before the authors, on a Gemini
chat. Its program does not load, 3 of its 30 constructs are the language as
written, and it wrote the explanation bookkeeping by hand in the source.
Its diff is [round8/probe/DIFF.md](round8/probe/DIFF.md). The reply came after rc.17 was
published, so the authors read rc.17's docs without its four guide items;
those go to rc.18 (amendment 6).

### Measurements

Per author, blind phases CR17–CR20 unless stated:

| | C1 | C2 | C3 | T1 | T2 | T3 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Blind phases green on first run, of 4 | 3 | 2 | 2 | 4 | 4 | 4 |
| Blind runs to reach all-green | 6 | 13 | 9 | 4 | 5 | 6 |
| Blind failing runs | 1 | 6 | 4 | 0 | 0 | 0 |
| Inherited phases green on first run, of 17 | 15 | 15 | 12 | 16 | 16 | 17 |
| Code lines, end of CR20 (policy + glue) | 434 + 102 | 554 + 127 | 599 + 77 | 560 | 611 | 509 |
| Bytes, end of CR20 | 31,363 | 38,113 | 40,363 | 27,323 | 27,805 | 25,606 |
| Change cost, code lines, CR17–CR20 | 417 | 484 | 714 | 148 | 179 | 137 |
| Regressions during CR17–CR20 | 0 | 11 | 11 | 0 | 0 | 0 |
| Explanation failures (scenario ids) | 0 | 3 | 9 | 0 | 0 | 0 |
| Dispatch + view, median µs | 800 | 334 | 240 | 19.4 | 18.9 | 35.8 |
| Dispatch + view, p95 µs | 1,382 | 666 | 479 | 38.2 | 46.8 | 86.5 |
| Resume ten minutes of play, ms | 44.9 | 37.5 | 86.2 | 0.60 | 0.65 | 0.84 |
| Save after ten minutes, bytes | 989 | 1,528 | 1,583 | 284 | 226 | 200 |
| Gzipped bytes a host ships | 811,310 | 812,831 | 811,940 | 7,670 | 8,010 | 7,410 |
| Explanation drift, units, CR17–CR20 | 106 | 188 | 314 | 28 | 25 | 31 |
| Mutation score, overall | 86.5% | 62.1% | 56.4% | 93.1% | 96.7% | 95.2% |

Medians, and which side each favours:

| | Caveat C1–C3 | TypeScript T1–T3 | Better |
| --- | ---: | ---: | --- |
| **1. Correctness**: blind first-run green | 2 | 4 | TypeScript |
| Blind runs to all-green | 9 | 5 | TypeScript |
| **2. Size**, code lines | 676 | 560 | TypeScript, +21% |
| Size, bytes | 38,113 | 27,323 | TypeScript, +39% |
| **3. Change cost**, code lines | 484 | 148 | TypeScript, 3.3× |
| Regressions | 11 | 0 | TypeScript |
| **4. Explanation failures** | 3 | 0 | TypeScript |
| **5. Dispatch + view**, median µs | 334 | 19.4 | TypeScript; both under 1 ms |
| Gzipped bytes shipped | 811,940 | 7,670 | TypeScript |
| **6. Mutation score** | 62.1% | 95.2% | TypeScript |
| **7. Explanation drift** | 188 | 28 | TypeScript |

Per request, primary medians:

| | CR17 countdowns | CR18 doubt | CR19 retaste | CR20 undoubt |
| --- | ---: | ---: | ---: | ---: |
| Change cost, Caveat / TypeScript | 17 / 24 | 151 / 33 | 61 / 56 | 319 / 33 |
| Failing runs, Caveat / TypeScript | 0 / 0 | 1 / 0 | 0 / 0 | 2 / 0 |
| First-run green, Caveat / TypeScript, of 3 | 3 / 3 | 0 / 3 | 3 / 3 | 1 / 3 |
| Drift, Caveat / TypeScript | 3 / 3 | 38 / 8 | 26 / 15 | 123 / 4 |

The rules for counting are round 7's (`round8/score.mjs` is round 7's
`score.mjs` for round 8's authors and phases). Timings were taken with the
machine otherwise idle, after the fuzz and the mutation runs finished; a
first measurement taken while other agents were running gave the same order
(`results/score.json` holds the idle one). Raw numbers are in
`round8/results/`.

### Verdicts

**Rule A, adoption (unchanged): not met.** Caveat is better on none of
measures 1–4. Correctness, size, change cost and explanation failures all
favour TypeScript, and change cost is 3.3 times TypeScript's, beyond the 2×
the rule allows. Round 7's reading question (lines alone against lines with
regressions) does not arise: Caveat is worse on both.

**Rule B, the stated claim: not met.** Caveat's mutation score is lower,
its drift is higher, its blind change cost is higher, and first-run
correctness and explanation failures are both worse. Only dispatch is under
1 ms.

Row 7 excluded no run. Row 5's set comparator for journal caveats removed
the kind of failing run round 7 counted four of: none in this round is a
caveat-order failure.

### Where the Caveat cost came from

**CR17, countdowns: Caveat cheaper.** Three read-only numbers. Caveat
changed 17 lines against 24 and every author on both sides was green on the
first run. The prediction was the reverse.

**CR18 and CR20, doubt and take it back: the cost of the round.** Doubting a
witness adds a `doubted` caveat to every citation of it; CR20 takes it back.
All three Caveat authors found the same two limits in rc.17, wrote them in
their notes, and worked around both with state:

1. **A qualification cannot be taken back.** `qualify E with C` is permanent
   in the evidence's record. To make a doubt reversible each author stopped
   qualifying the witness and rebuilt `doubted` from plain flags: C1 through
   a `doubt_mark` value that carries the caveat, C2 through a new internal
   evidence `d` ("the player's doubt") cited as `qualified(…, d, doubted)`,
   C3 through `examined(doubted)` citations guarded by a flag per witness.
   CR20's median diff is 319 lines against TypeScript's 33, where the flip is
   one boolean.
2. **`qualify` names only the current occurrence of renewable evidence.** A
   remembered witness from a mushroom's earlier life cannot be given
   `doubted`, so at CR18 each author had a gap (C1 and C3 wrote "known
   limit", C2 "KNOWN GAP"). CR20's workaround closed it for C2 and C3.

The workaround is also where the regressions and explanation failures came
from: C2's 11 regressions and 3 explanation failures, and C3's 11 and 9, all
fall in CR18 and CR20, while the journal and memory were rebuilt around the
flags.

**CR19, retaste: a tie.** 61 lines against 56, every author green first.

### What the suite did not see

**Differential fuzz.** `round8/fuzz.mjs`, registered at stage 4, ran 300
seeded sequences (seed 7, 815,501 events) through all six programs against
T2:

| Program | Sequences that diverged | First differences |
| --- | ---: | --- |
| T1, T3 | 0 | |
| C1 | 40 | `journal` 29, countdowns (`fadesIn`) 11 |
| C2 | 29 | `journal` 29 |
| C3 | 184 | refused or failed events 144, `decision.basis` 11, countdowns 13, other 16 |

A post-hoc rerun with the same generator records every divergence
(`fuzz-checks/all-divergences.mjs`) and sorts them into classes
(`fuzz-checks/classify.mjs`, `results/fuzz-classes-seed7.json`). Three
classes cover all but three:

- **Basis order, 69.** Each is the CR7 recovery commitment's two
  observations in name order instead of the order they were observed. In
  C1 and C2 it is the trust entry in the journal: both read the basis from
  View 0.2's ordered grounds, but built the journal from binding
  explanations, which View 0.2 leaves in name order
  (`caveat-view-0.2.md`, "Ordered evidence in grounds"). C3 never asked for
  View 0.2, so its `decision.basis` itself is in name order. This is round
  7's work-list item 2 again: ordered grounds reach only a host that asks
  for 0.2, and only the grounds.
- **Countdowns one second long, 24 (C1 and C3).** Every case follows ticks
  of 0.05, 0.1, 0.2 or −0.01 s. C1 and C3 compute the time left from the
  session's accumulated clock (C1 keeps `$m_tasted_at`, C3 `tasted_at =
  elapsed()`); the TypeScript programs count down per timer. The cause is
  inferred from that, not traced in the runtime. This is the binary64 firing rule recorded
  as F267 in round 7, now visible in a field the view shows. C2 kept its
  own countdown state per mushroom and shows none.
- **Sessions, 157 (C3).** From sequence 145 onwards C3's policy could not be
  created. The fuzz runs synchronously and never frees a policy that has no
  `free()`, so the runtime's finalizers never run. In a standalone check
  (`fuzz-checks/session-reuse.mjs`) C3 traps with the WebAssembly error
  `unreachable` on the 236th round of opening, playing, saving and resuming
  in one process; C1 and C2 pass 400 rounds. A
  rerun that collects garbage between sequences (`--release-between`,
  `results/fuzz-classes-seed7-release-between.json`) removes the class:
  C3 then diverges in 57 sequences, 26 on basis order, 28 on countdowns and
  3 other.

The three other C3 differences are program errors that no scenario reached:
twice a mushroom's `why.absorb` shows `doubted` on a witness T2 shows
without it, and once a label has no `because` where T2 cites `absorb_pool`.
Their causes were not traced.

**Long play and the save bound.** `round8/longplay.mjs` plays 10, 30 and 60
regrowth cycles of every mushroom and saves:

| Cycles | C1 | C2 | C3 | T1 | T2 | T3 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 4,073 | 14,500 | 9,621 | 783 | 522 | 307 |
| 30 | 4,101 | 14,719 | 9,690 | 791 | 530 | 307 |
| 60 | 4,105 | 14,720 | 9,670 | 791 | 530 | 331 |
| Resume at 60, ms | 55.8 | 36.2 | 85.2 | 1.6 | 2.1 | 2.7 |

Every program resumed to the same view. Unlike round 7, every Caveat save
stops growing in this play, and resume no longer scales with play. All
three are still over CR13's 4,096-byte bound in long play, C1 by 9 bytes.
All three authors name one remaining source of growth that this play does
not reach: rc.17 refuses a `window` on a decision series, so each recommit
of `trust` adds a revision that stays in the save. C1 caps the series at
256 revisions and C3 at 64 commits; C2 stops recommitting after 256.

### Mutation score

A fresh mutator agent listed every applicable site per program before any
mutant ran (`round8/mutation/sites.json`, 485 sites, frozen at `d644609`),
then wrote one mutant per site, each recorded as a `CHANGE.md` and a diff in
`round8/mutation/mutants/`. `round8/mutate.mjs` (round 7's) ran each one
through the static check, the runtime and the registered suite, then fuzzed
every survivor against its original (100 sequences, seed 11). Results are in
`round8/results/mutation.json`. To finish in hours rather than a day the run
was split into four processes by program (`--programs`), which changes
nothing about how a mutant is judged.

| Program | M1 | M2 | M3 | M4 | M5 | Overall | Static / runtime / suite | Survivors, possibly equivalent |
| --- | --- | --- | --- | --- | --- | --- | --- | ---: |
| C1 | 16/16 | 1/1 | 1/1 | 2/4 | 25/30 | 45/52, 86.5% | 0 / 20 / 25 | 7, 4 |
| C2 | 18/27 | 1/1 | 1/1 | 2/5 | 32/53 | 54/87, 62.1% | 0 / 26 / 28 | 33, 22 |
| C3 | 23/26 | 1/1 | 2/2 | 2/4 | 65/132 | 93/165, 56.4% | 1 / 61 / 31 | 72, 65 |
| T1 | 9/10 | 2/2 | 2/2 | 11/13 | 30/31 | 54/58, 93.1% | 0 / 1 / 53 | 4, 2 |
| T2 | 16/16 | 2/2 | 2/2 | 12/13 | 26/27 | 58/60, 96.7% | 0 / 1 / 57 | 2, 1 |
| T3 | 15/15 | 2/2 | 2/2 | 11/13 | 30/31 | 60/63, 95.2% | 0 / 1 / 59 | 3, 1 |

**Where Caveat lost.** Two operators account for it.

- **M5, cite evidence never read.** In round 7 `caveat check` refused every
  Caveat M5 mutant before it ran. Here it refused none, because this
  mutator wrote the extra citation as `observed(absorb_cave)` where round 7's
  wrote the bare name. rc.17 refuses a bare evidence name in `because` at
  load ("a declared evidence, not a value citation"), but accepts
  `observed(E)` and refuses it only at run time, as `ungrounded_citation`,
  once `E` has been observed and the binding is active. 122 of the 215
  Caveat M5 mutants were caught that way or by the suite. Of the 93 that
  survived, the fuzz could not tell 81 from their originals; the likely
  reason, not traced, is that their bindings are inactive whenever the cave
  mushroom has been absorbed. A
  post-hoc rewrite of every Caveat M5 mutant to the bare name
  (`fuzz-checks/m5-bare-name.sh`, `results/m5-bare-name.txt`) makes all
  215 fail to load. So the score depends on which spelling a careless author
  reaches for, and `caveat check` does not catch the one this mutator chose.
- **M1 in C2, drop a caveat.** Nine of C2's 27 M1 mutants survived. Seven drop
  the `d, doubted` arguments from the recovery commitment, a why citation or
  journal pages 1 to 5, the CR20 workaround above, which the suite exercises
  on only some pages. The other two remove the retaste's fade.

**M4 on both sides.** Reversing `decision.basis` survives the suite in every
program, because the registered comparator compares the basis as a set; the
fuzz, which compares it in order, tells each one apart. Swapping the
operands of the recovery commitment in Caveat source changes nothing, as
predicted: View 0.2 orders grounds by observation, not by the source.

A survivor that the fuzz cannot tell apart is labelled possibly equivalent.
Because C3 can exhaust its sessions in one process (above), every survivor
was fuzzed again with sessions freed between sequences
(`fuzz-checks/survivors-release.sh`, `results/survivors-release-between.txt`).
No label changed.

### Explanation drift

A drift agent counted, after each blind request, the explanation-producing
units each diff touched (`round8/drift/DRIFT.md`, unit lists in
`DRIFT.json`). Medians: CR17 3 against 3, CR18 38 against 8, CR19 26 against
15, CR20 123 against 4. Like round 7's agent, it counted save code and also
rejection guards as feeding explanations, since an accepted event adds
evidence; it says the Caveat counts for CR18–CR20 would fall the most under
a reading that leaves guards out. TypeScript's CR20 total is 1 to 5 units
per program, against 50 to 206, so no reading of the guards closes that gap.

### Learnability: why each Caveat blind run failed

`round8/fuzz-checks/first-failures.mjs` (round 7's) re-runs every failing
run's snapshot against its phase's scenarios. "First" marks a phase's first
run.

| Author | Phase, run | Scenarios failed | Cause | Kind |
| --- | --- | --- | --- | --- |
| C1 | CR18, first | S54–S56 | the program refused three doubts the scenarios accept, citing its rule that one undoubted observation must remain | author: the rule applied where CR18 does not apply it |
| C2 | CR18, first | S54–S56 | a doubted witness still counted in the belief, label and note | author |
| C2 | CR20, first | 10 | four saves over 4,096 bytes; six `ungrounded_citation` refusals of a `doubted` citation the value did not read | language: the save; language rule |
| C2 | CR20, four more | 2 to 10 | an `undoubt` payload the adapter passed unparsed; an empty journal mid-rewrite; saves over the bound | glue; author; language: the save |
| C3 | CR18, first and second | S48, S49 (first only), S54–S56 | journal entries lost during a memory rewrite; `doubt` refused because an event parameter must be a number, and the doubted evidence is a text id | author; language: event parameters carry numbers and sorts, not text |
| C3 | CR20, first | 10 | `reopenedBy` and `history` kept only the first reopening after the rewrite; one save 35 bytes over | author; language: the save |

**Language.** Again the save, now only in CR20 and by tens to hundreds of
bytes. The citation rule (`ungrounded_citation`) caught C2 once, as it
caught round 7's authors, and was fixed on the next run. New this round: an
event cannot carry a text parameter, so a doubt naming `witness_pool_2` has
to be parsed into numbers in the adapter (all three adapters do this).

**Author.** The CR18 first-run failures are the authors' readings of the
request, made while redesigning around the two qualification limits above.
TypeScript authors met the same request with none.

**The registered comparator.** No failing run in this round is an order
failure of journal caveats; row 5 removed the four round 7 counted.

### Predictions

Round-level predictions were recorded before the protocol was registered
([round8/PREDICTIONS.md](round8/PREDICTIONS.md)); primary medians.

| Measure | Target | Expected, Caveat | Actual, Caveat | Expected, TypeScript | Actual, TypeScript | Outcome |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Blind first-run green, of 4 | 4 | 3 | 2 | 4 | 4 | **failed** for Caveat; held for TypeScript |
| Blind runs to all-green | — | 5 | 9 | 4 | 5 | **failed** both |
| Regressions | 0 | 0 | 11 | 0 | 0 | **failed** for Caveat |
| Change cost, lines | lower than TypeScript | 150 | 484 | 220 | 148 | **failed** both, and the target |
| Size, lines | lower than TypeScript | 380 | 676 | 560 | 560 | **failed** for Caveat; held for TypeScript; target missed |
| Explanation failures | 0 | 0 | 3 | 0 | 0 | **failed** for Caveat |
| Mutation score | 100% | 98% | 62.1% | 100% | 95.2% | **failed** both |
| Drift, units | at or below TypeScript | 35 | 188 | 30 | 28 | **failed** for Caveat; target missed |
| Dispatch, median µs | under 50 | 100 | 334 | 20 | 19.4 | **failed** for Caveat |
| Gzipped bytes shipped | under 150,000 | 760,000 | 811,940 | 7,000 | 7,670 | both within 10%; target missed |
| Rule A | met | met | not met | | | **failed** |
| Rule B | — | not met | not met | | | held |

Per request (recorded at stage 3):

| Prediction | Caveat | TypeScript | Outcome |
| --- | ---: | ---: | --- |
| CR17: TypeScript smaller diff and fewer failed runs | 17 lines, 0 failed runs | 24, 0 | **failed**: Caveat −29%; failed runs tie |
| CR18: Caveat smaller diff by more than 10% | 151 | 33 | **failed** |
| CR18: Caveat at least one failed run | 1 | 0 | held |
| CR19: tie | 61 | 56 | held (+9%) |
| CR20: TypeScript smaller diff and fewer failed runs | 319 lines, 2 failed runs | 33, 0 | held |
| Drift: Caveat lower on CR18 | 38 | 8 | **failed** |
| Drift: TypeScript lower on CR17 and CR20 | 3, 123 | 3, 4 | **failed** on CR17 (a tie); held on CR20 |
| Drift: tie on CR19 | 26 | 15 | **failed** |

The probe's predictions are scored in `probe/DIFF.md`: two held and the
third failed.

### What this round can and cannot show

It shows how rc.17 fares with six fresh agents of one model family on one
beat. On the inherited requests and on CR17 and CR19, Caveat authors kept
pace. On the two requests that needed a caveat to be taken back or attached
to an earlier life, they rebuilt that bookkeeping with flags in the source,
and that cost lines, regressions, drift and mutation score. TypeScript is the
default. The round cannot show how people would fare, or other kinds of
program. All agents, the request writer and reviewer, the mutator, the drift
counter and the drafter of the protocol are one model family; only the probe
was not. TypeScript is a language these agents know far better. The Caveat
packet's specifications contain Glowcap fragments, disclosed at stage 4; none
uses CR17–CR20's events or fields. One request pair, CR18 and CR20, decides
most of the gap, and a different set of blind requests could show a smaller
one. The mutation score turns on how the mutator spelled one operator (M5,
above), so the per-operator rows matter more than the overall figure.

No claim that Caveat is better than TypeScript goes into the README, AGENTS,
the site or other documents on the strength of this round.

**The work list this round leaves.** Each item names what the round
observed, not a design.

1. **Withdrawing a qualification.** Every Caveat author needed a caveat that
   can be taken back, and built it from flags (CR20).
2. **Qualifying an earlier occurrence.** `qualify` reaches only the current
   life of renewable evidence (CR18).
3. **Ordered evidence beyond grounds.** View 0.2 orders only commitment
   grounds, and only for a host that asks; two programs built the order wrong
   from binding explanations and one never asked (fuzz, 69 sequences).
4. **A window on a decision series.** All three authors capped recommits
   because the series cannot forget (long play).
5. **Static coverage of `observed(E)` citations.** `caveat check` accepts a
   `because` that cites `observed(E)` the value never reads; it is refused
   only at run time (mutation, M5).
6. **Text in events.** An event parameter cannot carry an evidence id;
   adapters parse it into numbers.
7. **Sessions a host never frees.** A host that creates many sessions in one
   synchronous run exhausts the runtime with the error `unreachable`, which
   does not say why (fuzz, C3).
8. **Countdowns to the tick.** F267's firing rule now shows in a view field
   (CR17's `fadesIn` and `regrowsIn`).

### Reproduce (round 8)

The authors' programs live outside the repository at run time; each one's
final source is in `round8/runs/<ID>/impl/`, with its phase commits in
`phases.bundle`, every test snapshot in `snapshots.tgz`, its runner log and
its notes.

```sh
cd experiments/glowcap/round8
node prepare.mjs authors --tarball=caveat-lang-0.1.0-rc.17.tgz   # checks its hash, installs it
# then copy each runs/<ID>/impl/ to /home/claude/glowcap-r8/authors/<ID>/impl/,
# runs/<ID>/{state.json,log.jsonl} to /home/claude/glowcap-r8/private/<ID>/,
# and clone runs/<ID>/phases.bundle to /home/claude/glowcap-r8/private/<ID>/repo
node score.mjs --bench > results/score.json        # measures 1–5
node longplay.mjs > results/longplay.json
A=/home/claude/glowcap-r8/authors
node fuzz.mjs --impl=$A/T2/impl/glowcap.ts --impl=$A/T1/impl/glowcap.ts --impl=$A/T3/impl/glowcap.ts \
  --impl=$A/C1/impl/adapter.mjs --impl=$A/C2/impl/adapter.mjs --impl=$A/C3/impl/adapter.mjs \
  --sequences=300 --seed=7 --dump=results/fuzz-seed7.json
# the same six --impl, recording every divergence, then with sessions freed between sequences
node fuzz-checks/all-divergences.mjs … --sequences=300 --seed=7 --dump=DUMP.json
node --expose-gc fuzz-checks/all-divergences.mjs … --sequences=300 --seed=7 --release-between --dump=DUMP-RB.json
node fuzz-checks/classify.mjs DUMP.json > results/fuzz-classes-seed7.json
# mutation: the mutator's directory holds sites.json and the full mutants
node mutate.mjs --programs=C1 > C1.json   # and C2,T1 / C3 / T2,T3; join in sites.json order
bash fuzz-checks/survivors-release.sh > results/survivors-release-between.txt
bash fuzz-checks/m5-bare-name.sh > results/m5-bare-name.txt
node fuzz-checks/first-failures.mjs C2   # each author
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
