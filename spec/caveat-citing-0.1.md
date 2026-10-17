# CAVEAT Citing 0.1 (draft) — a decision with grounds and no value

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.1 ([#198](https://github.com/WSattazahn/caveat-lang/issues/198)): implemented on `main`, not yet in a release. It becomes stable when a 1.x release ships it. It adds to
[Reactive 0.3](caveat-reactive-0.3.md) and
[Permission 0.1](caveat-permission-0.1.md); it changes no program that loads
today.

A release gate has three answers: ready, not ready, and can't tell. "Not
ready" is worth keeping with its reasons, so a program commits it as a
decision of its own. But a commitment's grounds come only from its `using`
value ([Reactive 0.3](caveat-reactive-0.3.md), "Committing with a qualified
basis"), and `using` needs a number. The [guide's idiom](../docs/AI_AUTHORING.md#record-not-ready-with-its-reasons)
sums the readings only to read them:

```caveat
on assess when not (ci == 1 and build == 1) commit hold because enough using ci + build;
```

`explain` then prints `hold@1 = 1  based on build_check, ci_run`, and the
guide has to tell readers that the `1` is not a verdict. Without `using`, the
decision has no grounds at all: `hold@1 = null  based on nothing`. This
profile adds a clause that gives a decision grounds without a value:

```caveat
on assess when not (ci == 1 and build == 1) commit hold because enough citing ci, build;
```

`explain` prints `hold@1 = null  in force` and `based on build_check, ci_run`.
The same clause written today does not load:

```text
does not load: line 15, column 1: commit expects [using NUMERIC_EXPRESSION] [permitted by GRANT [for EXPRESSION]] [retaining CAVEAT, CAVEAT]
```

(`caveat-lang@1.0.0`, `caveat check`, on #198's program with its last line
written as `commit hold because enough citing ci, build`.)

## Syntax

```text
commit D because REASON citing CITATION {, CITATION} [permitted by GRANT [for X]] [retaining C, …]
```

`citing` takes the place of `using`; a commit has one or the other, or
neither. Each `CITATION` is an ordinary expression of any type, as a binding's
citations are ([Explanations 0.1](caveat-explanations-0.1.md)): a state, a
`qualified(…)` reading, `latest(STREAM)`, or anything computed from them.
Only what each one reads is used. To cite evidence directly, write
`qualified(1, EVIDENCE)`, as `using` already allows.

`citing` is recognised only as the first word after `because REASON`, where
today nothing but `using`, `permitted` or `retaining` may stand. It is not
reserved: a state, evidence or `fn` may still be named `citing`, and
`using citing` still reads a state called `citing`. An empty citation, an
empty item between commas, or an unknown name does not load, as for
`using`.

## Meaning

When the commit runs, each citation is evaluated in order, against the same
state, after the commit's guard and permission. Then:

- **Grounds** are the union of what each citation's value reads, plus the
  retained caveats, exactly as `using`'s grounds are for its one expression.
- **Lineage** is the guard's, the permission grant's, every citation's
  whole lineage and the retained caveats, as for `using`.
- **The basis value** is `null`, as for a commit without `using`.
- The frozen basis, `relies_on` edges, pins, reopening and the withdrawal
  collector treat the grounds and lineage as they treat a `using` commit's.
  `rests_on_withdrawn(D)` reads the cited grounds.

The commit refuses on the same codes as a `using` commit: an evaluation
failure in a citation as `evaluation/expression`, a cited reading of evidence
no event has observed as `evaluation/unobserved_evidence`, and so on. No new
refusal code is added.

A decision made with `citing` has no value to read, so `latest(D)` and
`history_at(D, …)` refuse it as `evaluation/expression`, as they refuse a
decision made without `using` today
([Dispatch 0.1](caveat-dispatch-0.1.md)). Read the verdict with
`committed(D)`, and the reasons with `explain`.

## Snapshot, save and views

The clause adds no field. A commitment made with `citing` has the shape a
commitment without `using` already has, with grounds:

- `commitment_bases` holds `{"value": null, "provenance": …}`, which
  [Reactive 0.3](caveat-reactive-0.3.md) already allows.
- `caveat-reactive-save/0.1` is unchanged, and every 1.0 save restores as
  before. Restore reads a commit's citations wherever it reads a `using`
  expression; it evaluates neither.
- `caveat-reactive-view/0.1` and `/0.2`, and `caveat-explain/0.1`, are
  unchanged. Their decision value is `null` and their grounds list what the
  citations read.

## Compatibility with other 1.1 additions

- [Earlier Occurrence 0.1](caveat-earlier-occurrence-0.1.md) does not add
  `occurrence(…)` as a value, so a citation cannot name an earlier occurrence.
- A caveat kept on purpose (`retaining C because E`, D006) extends the
  `retaining` clause, which follows `citing` as it follows `using`.
- Small saves (#236) store events, not commitments, and are unaffected.

## Not in this profile

- Citing a bare evidence name. Evidence is not an expression today, and
  `qualified(1, EVIDENCE)` already cites it.
- `citing nothing`. A commit without `using` or `citing` already has no
  grounds.
- A grounding check like a binding's. A binding's citations must be a subset
  of a larger lineage; a commit's grounds are what its clause reads, so there
  is nothing larger to check against.
- `using` and `citing` together. A decision whose value is meaningful
  already gets its grounds from `using`.

## Proofs

`proofs/lean/Caveat/Late.lean` already models a commitment's frozen basis
with an optional value ("the value is absent when the commit had no
`using`") beside its grounds. A citing commitment is one such basis whose
grounds are the union of its citations'. The theorems about frozen bases do
not depend on how the grounds were gathered, so they cover it without
change. The implementation adds runtime tests.

## Corpus sweep

`node experiments/citing-sweep/sweep.mjs` reads every tracked Caveat program
and writes [`RESULTS.json`](../experiments/citing-sweep/RESULTS.json). It
reads source text only and runs no program. At this profile's revision:

- 191 programs; none uses the word `citing`, so no commit writes it after
  its reason and no program reaches the new form;
- 146 programs make 204 commits: 191 with `using` and 13 without, which
  have no grounds today;
- 117 of the `using` commits, in 78 decisions, are never read by `latest(…)`
  or `history_at(…)`: their value reaches only `explain` and views. These are
  where `citing` could replace a value that means nothing. The sweep cannot
  tell which values mean something to a reader, so it changes none.

No accepted program changes.

## Implementation

- The runtime parses `citing` into the commit effect beside `using`, and
  evaluates each citation's value and grounds where it evaluates `using`'s.
  Restore, the check that a decision series is read, and procedure expansion
  read citations wherever they read `using`.
- `runtime/tests/citing.rs` covers #198's gate, withdrawal and reopening
  through cited grounds, the refusal to read a citing decision's value,
  refusals inside a citation, a save and restore round trip, citations of any
  type with `permitted by` and `retaining`, the load rules, and `citing` as a
  state name.
- The language reference and the authoring guide use `citing` for "not
  ready", the package ships this profile, and the editor grammar highlights
  `citing`.

## Changes

- 2026-10-17: specified for 1.1 (#198).
- 2026-10-17: implemented.
