# CAVEAT Reading Index 0.1 (draft) — naming an earlier reading of a stream

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.1: implemented on `main`, not yet in a release. It becomes stable when a 1.x release ships it. It adds to
[Earlier Occurrence 0.1](caveat-earlier-occurrence-0.1.md) and
[Latest 0.1](caveat-latest-0.1.md); it changes no program that loads today.

[Latest 0.1](caveat-latest-0.1.md) lets a program qualify, take back and
withdraw a stream's **current** reading. A report found wrong after later
reports arrived is an earlier reading, and nothing could name it. To withdraw
the second of five test reports, an author had to rewrite the stream as
renewable evidence, one declaration and one rule per report. Before this
profile the form did not load:

```text
does not load: line 17, column 1: invalid reactive identifier reading(checks, i)
```

(`caveat-lang@1.0.2`, `caveat check`, on the program below.) This profile names a stream's reading by
its position, wherever `occurrence(EVIDENCE, INDEX)` names an occurrence:

```caveat
claim passing;
caveat flaky consequence material;
evidence runner from "the test runner";
evidence recheck from "a re-run";
readings checks from runner limit 8;
decisions ship limit 8;
event report v min 0 max 1;
event see_recheck;
event retract i min 1 max 8;
event flag i min 1 max 8;
event unflag i min 1 max 8;
event decide;
event revisit i min 1 max 8;
on report sample checks = v supports passing;
on see_recheck reveal recheck;
on decide commit ship because enough using latest(checks);
on retract withdraw reading(checks, i) because recheck;
on flag qualify reading(checks, i) with flaky;
on unflag unqualify reading(checks, i) with flaky;
on revisit when committed(ship) reopen ship because reading(checks, i);
```

After `report, report, see_recheck, retract {"i": 1}`, the first reading,
`checks@1`, is withdrawn and the second is not.

## Syntax

```text
reading(STREAM, INDEX)
```

`STREAM` is a reading stream and `INDEX` a numeric expression. The form
stands wherever `occurrence(EVIDENCE, INDEX)` does:

- `withdraw reading(STREAM, INDEX) because EVIDENCE`;
- `qualify reading(STREAM, INDEX) with CAVEAT` and `unqualify reading(STREAM, INDEX) with CAVEAT`;
- `reopen DECISION because reading(STREAM, INDEX)`;
- `observed(reading(STREAM, INDEX))`, `withdrawn(reading(STREAM, INDEX))` and
  `carries(reading(STREAM, INDEX), CAVEAT)`.

`occurrence(STREAM, INDEX)` means the same: a reading is an occurrence of its
stream. A name that is neither renewable evidence nor a reading stream does
not load, with Earlier Occurrence 0.1's message.

## Meaning

`INDEX` is counted from 1: `reading(checks, 1)` is `checks@1`, the first
reading the stream took, and `reading(checks, k)` is `checks@k`. It is the
reading `history_at(checks, k - 1)` reads while the stream holds every reading
since the first.

The index is evaluated when the effect or predicate runs. It names a reading
the stream holds now. One that was never taken, or that has retired (a
`window` departs it), names none:

- in an effect, the event is refused as `evaluation/expression`, as
  `occurrence(EVIDENCE, INDEX)` is;
- in a predicate, the answer is false.

Once named, a reading is ordinary evidence, and each effect does to it exactly
what it does to an occurrence named by `occurrence(EVIDENCE, INDEX)`, or to the
current reading named by `latest(STREAM)`. In particular:

- a `qualify` needs the reading observed, which it is from when it was taken;
- an `unqualify` marks that reading's live relation and no other;
- a withdrawal or a take-back reaches every current value whose lineage or
  grounds hold the reading, and leaves commitment bases, grounds, journals and
  records already made as they are.

The index joins the effect's lineage, as an occurrence's index does.

Latest 0.1's template rule holds for every reading: a program that takes a
caveat back from a stream's reading may not also put that caveat on the
stream's template.

## Snapshot, save and views

The form adds no field and no refusal code, and changes no snapshot, view or
save of a program that does not use it. Restore accepts withdrawal records,
relations and take-back records on any reading, which Latest 0.1 already
counts as their own source.

## Corpus sweep

No tracked program writes `reading(` or names a reading stream in
`occurrence(`; before this profile neither loaded. The save sweep
(`runtime/examples/save_sweep.rs`, 8 runs of 150 seeded events) over all
tracked programs gives the same digest for every program as `main` before this
change.

## Implementation

The selector parses `reading(` as `occurrence(`. A stream's index resolves to
`STREAM@INDEX` while the stream holds it and it has not retired. Tests:
`runtime/tests/reading_index.rs`.

## Changes

- 2026-10-19: drafted and implemented for 1.1.
