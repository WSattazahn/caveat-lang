# CAVEAT Grounds Of 0.1 — a decision's value on its grounds alone

**Status:** stable from 1.5.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); drafted for 1.5. It adds to
[Reactive 0.5](caveat-reactive-0.5.md) ("A revision is a new commitment"),
[Withdrawal 0.1](caveat-withdrawal-0.1.md) and
[In Basis 0.1](caveat-in-basis-0.1.md); it changes no program that loads
today.

A revision of a decision series keeps two records of what it read. Its
**grounds** are what the commit rested on: the evidence its `using` or
`citing` expression read, and the caveats it retained. Its **basis** is
everything that could have influenced it: the grounds, the rule's guard, a
`permitted by` grant, and, because a revision may only follow a reopened
predecessor, the predecessor's basis and the reopening's witness
(Reactive 0.5, "A revision is a new commitment"). `rests_on_withdrawn(D)`
reads the grounds. `latest(D)` returns the value on the basis.

That is right for lineage and awkward for one common decision. The
pharma-safety demo decides a liver signal on a case report, withdraws the
case when its review fails, reopens the signal and decides it again on a
second case. The second revision rests on the second case alone, and
`rests_on_withdrawn(liver_signal)` says so. But `using latest(liver_signal)`
in a later decision carries the withdrawn first case, because the revision's
basis holds it through the reopening, so the later decision carries
`withdrawn` though nothing it rests on is withdrawn. The author's only way
out was to decide on the evidence again instead of on the decision. Before
this profile the direct form did not load:

```text
signal.cav does not load: unknown source function grounds_of
```

(`caveat-lang@1.4.0`, `caveat validate`.)

## Syntax

```text
grounds_of(SERIES)
```

`SERIES` is a declared decision series. The form is a number, written
wherever `latest(SERIES)` may be: a value, a guard, a `bind`, a `using` or
`citing` expression, a procedure step.

These do not load:

| Source | Message |
| --- | --- |
| `grounds_of(x)` where `x` is not a decision series (a reading stream, a state, evidence) | `grounds_of names a decision series; x is not one` |
| any other form, such as `grounds_of()` or `grounds_of(latest(d))`, in a program without its own `fn grounds_of` | `unknown source function grounds_of` |
| `grounds_of(SERIES)` inside a pure source function | the function's load error, as for `latest` |

**A program that defines `fn grounds_of`.** Before this profile, a source
function could be named `grounds_of`. A program that defines one keeps it:
every `grounds_of(…)` in that program calls the function, with the
arguments as written, and the form is not available to it. The word reserves
nothing: a state, event, stream or series may be named `grounds_of`.

## Meaning

`grounds_of(D)` is the current revision's frozen numeric `using` value, the
number `latest(D)` gives, on the revision's frozen **grounds** instead of its
basis:

- the evidence and caveats of `commitment_grounds[D@K]`, where `D@K` is the
  current revision: what the commit's expression read, and what its
  `retaining` clause named;
- plus the series' current head-selection dependencies, as `latest` adds
  them (Reactive 0.5, "Selecting a current value without rewriting history");
- plus `withdrawn` for every ground since withdrawn, as every read of a
  withdrawn occurrence carries it (Withdrawal 0.1).

It refuses as `latest(D)` refuses, with the same messages: a series with no
revision, or a current revision whose commit had no numeric `using` value,
is `evaluation/expression`, and the event is refused.

So a value computed from `grounds_of(D)` carries `withdrawn` exactly when
`rests_on_withdrawn(D)` is true, and `in_basis(D, E)` is true for exactly
the evidence it carries. What it leaves out is the rest of the basis: the
guard's evidence, a grant, and the predecessor's basis with the reopening's
witness. A decision that reads it rests on what `D` rested on, and nothing
`D` only saw.

`latest(D)` is unchanged. A program that wants the whole lineage keeps
writing it.

## Not in this profile

- `grounds_of(revision(SERIES, K))`, a past revision's value on its grounds.
  `rests_on_withdrawn(revision(…))` and `in_basis(revision(…), E)` already
  ask about a past revision's grounds; its value is for a later profile if a
  program needs it.
- A grounds form of a reading stream. A reading's grounds are its
  occurrence, which is its basis.
- `withdrawn(grounds_of(D))` or `carries(grounds_of(D), C)`. A guard asks
  `rests_on_withdrawn(D)` and `carries(latest(D), C)`.

## Snapshot, save, explain and views

Nothing new. The form adds no field, no schema member and no refusal code.
`caveat-reactive/0.1`, the saves, `caveat-explain/0.1`,
`caveat-dependents/0.1` and the views are unchanged, and a save restores as
it did. `caveat check`'s C008 `record-reads-late-caveat` counts
`grounds_of(D)` as a read of `D`'s record, as it counts `latest(D)`.

## Invariants kept

From [Caveat's invariants](../docs/CAVEAT_ESSENCE.md):

- A read changes nothing in the graph: the revision's basis and grounds stay
  as they were frozen.
- It discharges no caveat: a value it feeds carries every caveat the grounds
  carry, `withdrawn` included.
- A refused event publishes nothing; this profile adds no refusal.

## Proofs

`runtime/tests/grounds_of.rs`: the value beside `latest` on the demo's shape,
with one revision and after a reopening and a second commit; a withdrawn
ground reaching it exactly when `rests_on_withdrawn` turns true; a skipped
commit's selection guard inherited as `latest` inherits it; the refusals; the
load rules; a program's own `fn grounds_of`; and the save round trip.

## Corpus sweep

`node experiments/grounds-of-sweep/sweep.mjs` reads every tracked Caveat
program and writes
[`RESULTS.json`](../experiments/grounds-of-sweep/RESULTS.json). It reads
source text only and runs no program. At this profile's revision, no tracked
program writes the word `grounds_of` or defines a function of that name, so
no program reaches the new form and none changes meaning. The sweep also
lists the programs that read a series with `latest`, where the form would
stand.

The save sweep (`runtime/examples/save_sweep.rs`, 8 runs of 150 seeded
events) over every tracked program gives the same digest for every program
on `main` before this change and after it.

## Implementation

- `runtime/src/reactive_expr.rs`: `Node::GroundsOf` and
  `HistoryRead::Grounds`; the parser reads the form, and hands a program's
  own `fn grounds_of` its calls; C008 sees the read as `grounds_of`.
- `runtime/src/reactive.rs`: `grounds_read`, beside `latest`; the load check
  that the name is a decision series.
- `runtime/tests/grounds_of.rs`.
- The language reference, the authoring guide and the VS Code grammar name
  the form.

## Changes

- 2026-10-26: drafted and implemented for 1.5, from the pharma-safety
  demo's friction item 2 (triaged by the bug hunt as intended behavior of
  `latest`, with this form as the add-only answer). The AI change review
  demo's friction item 1 is the same shape after a take-back: a decision
  reopened by a review and made again still carries the taken-back caveat
  through `latest`, so an irreversible release reading it stays refused;
  `grounds_of` lets it rest on the reviewed revision's grounds
  ([Take-back 0.1](caveat-take-back-0.1.md), "Withdrawal, decisions and
  permission"). Test: `runtime/tests/grounds_of.rs`,
  `a_reviewed_revision_releases_on_its_grounds_not_its_lineage`.
