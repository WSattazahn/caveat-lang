# CAVEAT Latest 0.1 — qualifying and taking back the latest reading or occurrence

**Status:** stable from 1.1.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); drafted for 1.1. It adds to
[Late Qualification 0.1](caveat-late-qualification-0.1.md),
[Take-back 0.1](caveat-take-back-0.1.md) and
[Earlier Occurrence 0.1](caveat-earlier-occurrence-0.1.md); it changes no
program that loads today.

A reading stream records each check as a new reading. When a check turns out
to have run against the wrong configuration, the program wants to doubt that
reading, and later, if the configuration was right after all, to take the
doubt back. Before this profile neither loaded:

```text
does not load: line 13, column 1: invalid reactive identifier latest(checks)
```

(`caveat-lang@1.0.2`, `caveat check`, on the program below.) `withdraw
latest(STREAM)` and `reopen … because latest(STREAM)` name a reading, but
`qualify` and `unqualify` could not, and a stream's template cannot be
qualified for one reading, since every later reading copies its caveats.
An author who starts on a reading stream has to rewrite it as renewable
evidence to doubt one reading. Separately, `unqualify E with C` reaches every
held occurrence of `E`, where a task often wants only the latest. This
profile adds one target to both effects:

```caveat
claim passing;
caveat flaky consequence material;
evidence runner from "the test runner";
readings checks from runner limit 8;
decisions ship limit 8;
state last = 0;
event report v min 0 max 1;
event flag;
event unflag;
event decide;
on report sample checks = v supports passing;
on report set last = latest(checks);
on flag qualify latest(checks) with flaky;
on unflag unqualify latest(checks) with flaky;
on decide commit ship because enough using latest(checks);
```

After `report, flag, decide, report, flag, unflag`, the first reading,
`checks@1`, carries `flaky`, and `ship@1` keeps it in its basis. The second,
`checks@2`, carried it until `unflag` took it back, so `last` no longer
carries it. `unflag` marked only `checks@2`.

## Syntax

```text
qualify latest(NAME) with CAVEAT [after SECONDS];
unqualify latest(NAME) with CAVEAT;
carries(latest(NAME), CAVEAT)
```

`NAME` is a reading stream (`readings NAME from TEMPLATE …`) or declared
evidence. Anything else does not load:

```text
latest(NAME) must name a reading stream or declared evidence
```

The effects stand wherever `qualify` and `unqualify` do: in a rule or as a
procedure step. `carries(latest(NAME), CAVEAT)` is a boolean expression and
stands wherever one does; `CAVEAT` must name a declared caveat.
`has_caveat(…)` asks about a state, so `has_caveat(latest(NAME), CAVEAT)` does
not load, and its error names the `carries` form. `latest` keeps its other meanings: `latest(H)` in an
expression, `withdrawn(latest(STREAM))`, `withdraw latest(STREAM)` and
`reopen … because latest(STREAM)` are unchanged. A stream is still named only through
`latest(…)`: `qualify STREAM with C` and `unqualify STREAM with C` do not
load.

## Meaning

When the effect runs, `latest(NAME)` resolves to one concrete occurrence:

- for a reading stream, its current reading, the one `latest(NAME)` reads and
  `withdraw latest(NAME)` withdraws;
- for evidence, its current occurrence, which is the evidence itself unless
  it is renewable.

Then the effect does exactly what it does to that occurrence when the
occurrence is named directly ([Earlier Occurrence 0.1](caveat-earlier-occurrence-0.1.md)):

- `qualify` needs the occurrence observed. A stream with no reading yet is
  refused as `evaluation/unobserved_evidence`, as `withdraw latest(STREAM)`
  is. A reading is observed when it is taken. The relation is added once per
  live relation; with `after`, it is scheduled against the reading current
  when the effect runs, as `qualify … after` is scheduled against an
  occurrence ([Renewal 0.1](caveat-renewal-0.1.md)).
- `unqualify` marks that occurrence's live relation and no other. A stream
  with no reading, or an occurrence with no live relation, records nothing
  and reports nothing. It never refuses.

So `qualify latest(E) with C` means what `qualify E with C` means, and
`unqualify latest(E) with C` is `unqualify occurrence(E, I) with C` for the
current `I`, without a host or a state to track `I`.

`carries(latest(NAME), CAVEAT)` resolves `latest(NAME)` the same way when the
expression is evaluated, then asks `carries` of that occurrence, as
`carries(occurrence(E, I), CAVEAT)` and `carries(reading(STREAM, I), CAVEAT)`
do ([Earlier Occurrence 0.1](caveat-earlier-occurrence-0.1.md),
[Reading Index 0.1](caveat-reading-index-0.1.md)). A stream with no reading
yet makes it false, and nothing is refused. So a guard can ask whether the
current reading was already doubted:

```caveat
on flag when carries(latest(checks), flaky) reject "already flagged";
on flag qualify latest(checks) with flaky;
```

**What follows.** A reading is ordinary evidence once named, so everything
downstream works on it as it works for withdrawal:

1. Every current value whose lineage or grounds hold the reading gains the
   caveat ([Late Qualification 0.1](caveat-late-qualification-0.1.md)), and
   loses it when it is taken back ([Take-back 0.1](caveat-take-back-0.1.md)).
   Later reads through `latest`, `history_at` or `fold_history` get the
   reading's value with the caveats now live on it.
2. A later reading is a new observation and does not carry the caveat. Only
   the template's caveats are copied into a reading when it is taken.
3. Commitment bases and grounds, reading archives, the decision journal, and
   withdrawal and take-back records already recorded are left as they are. A
   decision keeps what it was made on.

The effects are reported in `effects` as today, with the concrete occurrence;
a take-back reports the stream as its evidence, as one of an earlier
occurrence reports the evidence:

```json
{"kind": "qualify", "evidence": "checks@2", "caveat": "flaky"}
{"kind": "unqualify", "evidence": "checks", "caveat": "flaky", "occurrences": ["checks@2"]}
```

## A caveat taken back from readings

Take-back 0.1 keeps a take-back exact by refusing any other way a take-back
caveat could reach a value. A reading copies its template's caveats when it is
taken, as relations of its own, which no `qualify` of the reading added. So a
program that takes a caveat back with `unqualify latest(STREAM)` may not also
put that caveat on the stream's template:

```text
unqualify latest(checks) with flaky: qualify runner with flaky puts it on the template, whose caveats every reading copies
```

Take-back 0.1's other rules apply unchanged: the caveat may not be
`withdrawn`, declared to qualify anything, qualified by a declared caveat, or
named in `qualified(…)`, `retaining` or `examine`. A stream's template still
cannot be taken back.

## Snapshot, save and views

The forms add no field and no refusal code.

- `caveat-reactive-save/0.1` is unchanged, and every 1.0 save restores as
  before. A save from a program that uses the forms holds relations and
  take-back records on readings, which restore accepts when an `unqualify
  latest(STREAM)` or `qualify latest(STREAM)` of the program could make them,
  the same reach checks it applies to an occurrence of renewable evidence.
- `caveat-reactive-view/0.1` and `/0.2` and `caveat-explain/0.1` are
  unchanged. Their `effects` and take-back members show the concrete
  occurrence.

## Departure

A relation or a take-back record on a reading pins it, or departs with it,
exactly as the same record on an occurrence of renewable evidence would
([Departure 0.1](caveat-departure-0.1.md)). An effect never names a retired
reading, since the current reading never retires.

## Not in this profile

- Naming an earlier reading of a stream by index. A reading is a value in a
  history, read by `history_at`; naming one as an effect target is a
  separate design.
- `latest(E)` for evidence in `withdraw` or `reopen … because`. `withdraw E`
  and `reopen … because E` already reach the current occurrence.
- `latest(…)` in a predicate other than `withdrawn(…)` and `carries(…)`.

## Corpus sweep

No tracked program writes `qualify latest(`, `unqualify latest(` or
`carries(latest(`; before this profile none loaded. The save sweep
(`runtime/examples/save_sweep.rs`, 8 runs of 150 seeded events) over all
tracked programs gives the same digest for every program on the runtime
before this change and after it: no program that loads today changes an
outcome, a save or a restored save. The departure gate in CI holds the same
programs to rc.15.

## Implementation

`qualify` and `unqualify` carry a `latest` flag. The session resolves the
target when the effect runs: a stream to its current reading, evidence to its
current occurrence. A take-back given one occurrence marks only it. Restore
counts a reading of a stream as a source of take-back records and accepts a
take-back report that names the stream. `carries(latest(NAME), CAVEAT)` reaches
the host as predicate kind `latest:carries:CAVEAT`, resolved with the target
the effects use. Tests: `runtime/tests/latest_qualify.rs` and
`runtime/tests/carries_latest.rs`.

## Changes

- 2026-10-18: drafted and implemented for 1.1.
- 2026-10-19: `carries(latest(NAME), CAVEAT)` added for 1.1; before it,
  `caveat-lang@1.0.3` and `main` refused it with "expected ',' before the
  carried caveat".
