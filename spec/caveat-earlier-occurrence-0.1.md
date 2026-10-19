# CAVEAT Earlier Occurrence 0.1 — naming an earlier occurrence of renewable evidence

**Status:** stable from 1.1.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); drafted for 1.1 ([#238](https://github.com/WSattazahn/caveat-lang/issues/238)). It adds to
[Renewal 0.1](caveat-renewal-0.1.md),
[Withdrawal 0.1](caveat-withdrawal-0.1.md),
[Late Qualification 0.1](caveat-late-qualification-0.1.md) and
[Take-back 0.1](caveat-take-back-0.1.md); it changes no program that loads
today.

A check reports twice, so its renewable evidence moves to a second
occurrence. Then the first report turns out to be wrong: it read a stale
artifact. A decision made on the first report still rests on it, which is
the case `rests_on_withdrawn(…)` and frozen grounds exist for. But a rule can
name only the current occurrence ([Renewal 0.1](caveat-renewal-0.1.md)), so
before this profile nothing could withdraw or qualify the first one:

```text
does not load: line 10, column 1: invalid reactive identifier occurrence(tests, 1)
```

(`caveat-lang@1.0.0`, `caveat check`, on the issue's program with its line 10 written as `withdraw occurrence(tests, 1) because operator`.) The workaround is
hand-built state, a bitmask per check and the basis encoded into a
decision's value. That works, but it moves provenance out of the language.
This profile adds one selector:

```caveat
claim passing;
evidence tests from "test runner";
renewable tests limit 8;
evidence operator from "the operator's correction";
decisions ship limit 8;
event report;
event decide;
event correct n min 1 max 8;
on report when observed(tests) renew tests;
on report reveal tests supports passing;
on decide commit ship because enough using qualified(1, tests);
on correct when not observed(occurrence(tests, n)) reject "no such report";
on correct when withdrawn(occurrence(tests, n)) reject "already withdrawn";
on correct reveal operator;
on correct withdraw occurrence(tests, n) because operator;
on correct when committed(ship) and not reopened(ship) and rests_on_withdrawn(ship)
    reopen ship because occurrence(tests, n);
```

## Syntax

```text
occurrence(EVIDENCE, INDEX)
```

`EVIDENCE` is renewable evidence, or a procedure's `evidence` parameter that
every call binds to renewable evidence. `INDEX` is a numeric expression: a
literal, a state, an event parameter or anything computed from them.
Occurrence 1 is the declared evidence itself, and occurrence `k` for `k ≥ 2`
is the one the `(k − 1)`th `renew` made, the one a snapshot names
`EVIDENCE@k`. Counting from 1 makes the index the number a host already
shows.

The selector is not an expression. It may stand only where a rule names one
evidence occurrence:

| Form | Meaning |
| --- | --- |
| `withdraw occurrence(E, I) because R` | [Withdrawal 0.1](caveat-withdrawal-0.1.md), with that occurrence as the target. The reason `R` still names its current occurrence. |
| `qualify occurrence(E, I) with C [after SECONDS]` | [Late qualification](caveat-late-qualification-0.1.md) of that occurrence; with `after`, scheduled against it as [Renewal 0.1](caveat-renewal-0.1.md) says. |
| `unqualify occurrence(E, I) with C` | A [take-back](caveat-take-back-0.1.md) that marks only that occurrence's live relation. |
| `reopen SERIES because occurrence(E, I)` | Reopens with that occurrence as the reason. |
| `observed(occurrence(E, I))`, `withdrawn(occurrence(E, I))`, `carries(occurrence(E, I), C)` | The existing predicates, asked of that occurrence. |

Every one of these positions refuses to load today ("invalid reactive
identifier", "invalid reactive effect", "expected ')' after predicate
target"), so no program that loads changes meaning. `occurrence` is not
reserved: evidence, a state or a `fn` may still be named `occurrence`, and
`withdraw occurrence because R` still names evidence called `occurrence`.
Only `occurrence(` in one of the positions above is the selector.

The load rules of each effect apply unchanged, with the selector's evidence
in place of a name: a take-back caveat in `unqualify occurrence(…)` meets
Take-back 0.1's rules, and `EVIDENCE` may not be a reading stream's
template, which is never renewable.

## Meaning

When a rule runs, `INDEX` is evaluated first and the selector resolves to
one concrete occurrence. It names a **held** occurrence when `INDEX` is a
whole number from 1 to the current occurrence's number and that occurrence
has not retired ([Windows 0.1](caveat-windows-0.1.md)).

**In an effect,** an index that names no held occurrence refuses the event
as `evaluation/expression`, as `history_at` does for an index out of range
([Dispatch 0.1](caveat-dispatch-0.1.md)). Otherwise the effect does exactly
what it does today to a current occurrence, and refuses on the same codes:

- `withdraw` and `qualify` need the occurrence observed, or refuse as
  `evaluation/unobserved_evidence`. An occurrence that `renew` made and no
  `reveal` observed before the next `renew` stays unobserved for good.
- `withdraw` of an occurrence already withdrawn keeps the first record and
  does nothing else.
- `qualify` adds a relation only when the occurrence has no live one for that
  caveat.
- `unqualify` never needs the occurrence observed. When the occurrence has no
  live relation a `qualify` added, it records nothing and reports nothing.
- `reopen` needs the occurrence observed, as `reopen … because EVIDENCE` does.

No new refusal code is added.

**In a predicate,** an index that names no held occurrence makes the
predicate false; it never refuses. That lets a rule check a host's index
with its own `reject`, as the example does. Otherwise each predicate answers
for that occurrence as it answers today for the current one.

**Lineage.** In an effect, `INDEX`'s lineage joins the effect's lineage, as
its guard's does, and never its grounds: which report a rule picked is not
what a decision rests on. In a predicate, the index is read like any other
operand of the expression, as `history_at`'s index is.

**What follows.** An earlier occurrence is ordinary evidence once named, so
everything downstream already works on it:

1. Every current value whose lineage or grounds hold the occurrence gains
   `withdrawn` or the caveat, as Withdrawal 0.1 and Late Qualification 0.1
   say for any target, and loses a taken-back caveat as Take-back 0.1 says.
   Values that hold only other occurrences are untouched: a value made from
   `tests@2` alone does not gain a caveat put on `tests`. A `renew` guarded
   by `observed(tests)` puts `tests` into `tests@2`'s lineage
   ([Renewal 0.1](caveat-renewal-0.1.md)), so there a value made from
   `tests@2` holds `tests` too, and does gain it.
2. `rests_on_withdrawn(D)` is true when the decision's current revision is
   grounded on the withdrawn occurrence, read from its frozen grounds as
   today.
3. Commitment bases and grounds, the decision journal, reading archives,
   withdrawal and take-back records already recorded are left as they are.
   A decision keeps what it was made on.
4. `withdrawn(tests)` still asks the current occurrence. Withdrawing an
   earlier occurrence does not withdraw the current one, as withdrawing the
   current one never withdrew a later one.

The effects are part of their event's transaction and are reported in
`effects` as today, with the concrete occurrence:

```json
{"kind": "withdraw", "evidence": "tests", "because": "operator"}
{"kind": "qualify", "evidence": "tests@2", "caveat": "flaky"}
{"kind": "unqualify", "evidence": "tests", "caveat": "flaky", "occurrences": ["tests"]}
```

### `unqualify` with and without the selector

`unqualify E with C` keeps its Take-back 0.1 meaning: it reaches every held
occurrence. `unqualify occurrence(E, I) with C` reaches only the one. This
closes the limit Take-back 0.1 left ("Choosing one occurrence needs the same
past-occurrence naming as doubting one").

## Snapshot, save and views

The selector adds no field. Withdrawal, qualification and take-back records
already name concrete occurrences, and a save made by a program that names
an earlier occurrence holds the same kinds of records a current-only
program's could, dated later. So:

- `caveat-reactive-save/0.1` is unchanged, and every 1.0 save restores as
  before.
- Restore keeps its rules. Where a rule asks whether an event reaches a
  `withdraw`, `qualify` or `unqualify` of some evidence, an effect on
  `occurrence(E, …)` counts as one of `E`, as an effect on `E` already counts
  for every occurrence of `E`. Conditions and indexes are not evaluated. As
  with every restore check, a save that passes holds records the program
  could hold, not records events are shown to have produced. Restore never
  authenticates history.
- `caveat-reactive-view/0.1` and `/0.2` are unchanged. Their `effects`,
  withdrawals and take-back members show the concrete occurrence.
- `caveat-explain/0.1` is unchanged; its effects already name occurrences.

## Departure

An effect never names a retired occurrence, so departure and
[Withdrawal Collection 0.1](caveat-withdrawal-collection-0.1.md) see only
records they already handle. A withdrawal or relation on an earlier held
occurrence pins it, or departs with it, exactly as the same record on the
current occurrence would.

## Compatibility with other 1.1 additions

- `unqualify` keeps recording a take-back and never deletes; the selector
  only narrows which relation it marks.
- A caveat kept on purpose (`retaining C because E`, D006) names a caveat on
  a decision, not an occurrence, and is unaffected. If it allows `E` to be
  evidence, it may allow `occurrence(E, I)` there by the same rules.
- The no-value `using` form (#198) is unaffected.

## Not in this profile

- Naming an earlier reading of a stream. `withdraw latest(STREAM)` still
  reaches only the latest reading, and a stream template still cannot be
  qualified. A reading is a value in a history, read by `history_at`; naming
  one as an effect target is a separate design.
- `occurrence(…)` as a value, in `qualified(…)`, in `reveal` or in `renew`:
  an earlier occurrence is never observed again or renewed.
- A count of occurrences. `observed(occurrence(E, I))` answers whether an
  index names an observed held occurrence, which is what a guard needs.
- Naming the occurrence a decision rests on (`grounds of D`).

## Proofs

`proofs/lean/Caveat` leaves renewal, retirement and departure unmodeled
([proofs README](../proofs/lean/README.md)), and resolving a selector is
renewal bookkeeping. The withdrawal, late-qualification and take-back steps
already take the target occurrence as input, so their theorems (frozen bases
and grounds are unchanged, a value keeps its number) cover an earlier
occurrence without change. The implementation adds runtime tests for
resolution and refusals.

## Corpus sweep

`node experiments/earlier-occurrence-sweep/sweep.mjs` reads every tracked
Caveat program and writes
[`RESULTS.json`](../experiments/earlier-occurrence-sweep/RESULTS.json). It
reads source text only and runs no program. At this profile's revision:

- 191 programs; no program writes `occurrence(` anywhere, so none is in a
  selector position and none declares a `fn occurrence`;
- no program uses the word `occurrence` at all;
- no effect target or predicate argument contains a parenthesis other than
  `latest(…)` and `reopen … because caveated(…)`, so no accepted program
  reaches the new forms;
- 72 programs declare 167 renewable evidence names, and 67 of those programs
  withdraw, qualify, take back, reopen on or ask a predicate of one of those
  names: the programs whose authors could now reach an earlier occurrence.

No accepted program changes. The implementation reruns the save sweep, the
departure gate and the View 0.2 delta gate over the whole corpus.

## Implementation

- The runtime resolves the selector when an effect runs
  (`EvidenceSelector::Occurrence`, and an index on `qualify` and
  `unqualify`), and a predicate on it through `occurrence:KIND` with the
  target `EVIDENCE#INDEX`. Restore's reach checks count the selector as its
  evidence.
- `runtime/tests/earlier_occurrence.rs` covers each form, each refusal, an
  unobserved earlier occurrence, a retired one, the index's lineage, a
  procedure's evidence parameter, a take-back of one occurrence and a save
  and restore round trip.
- The language reference lists the selector, and the editor grammar
  highlights `occurrence`.

## Changes

- 2026-10-17: specified for 1.1 (#238).
- 2026-10-17: implemented. "Lineage" now separates an effect's index from a
  predicate's, and "What follows" notes that a guarded `renew` puts an
  earlier occurrence into a later one's lineage.
