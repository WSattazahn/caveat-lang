# CAVEAT Lean scaffold

This is the pinned formal scaffold for the first rc.8 verification slice. It
contains core dependency and outcome equations, actual proofs, and a complete
named theorem audit. It does not contain an event interpreter or an executable
Rust/WASM comparison runner.

## Build and inspect

Use Lean `leanprover/lean4:v4.34.1`, as pinned by `lean-toolchain`. The model imports bundled `Std`; the audit also uses bundled Lean metaprogramming APIs. No external Lean package is required. From this directory:

```text
lake build --wfail
lake env lean Audit.lean
lake env leanchecker --verbose Caveat
```

`Caveat.lean` imports all model and law modules. `laws.json` records 42 authored theorem declarations, including four inclusion helper lemmas. `theorems.json` inventories all 81 public `Caveat` theorems in the elaborated environment, including compiler-generated declarations. `Audit.lean` prints their transitive axioms, emits the actual environment inventory, and rejects forbidden axioms in every public `Caveat` declaration (including definitions and unused custom axioms).
A successful build alone does not enforce an axiom allowlist; the repository
verification gate checks the registry and audit output. The allowed standard
axioms are `propext`, `Classical.choice`, and `Quot.sound`. No custom,
incomplete-proof, or native-evaluation axiom is permitted.

The explicit `Caveat` replay target is important: the pinned checker does not
implement a conventional `--help` option. This replay checks this package's
compiled declarations using Lean's kernel while trusting imported library
declarations. It is not a separately implemented checker, a fresh replay of
all `Std`, or a Comparator challenge comparison.

## Repository verification gate

From the repository root, run:

    npm run test:lean-gate
    npm run verify:lean

Lean and Lake must be available on PATH at the pinned version. Alternatively,
set CAVEAT_LEAN_BIN to that release's bin directory; the gate changes only its
child-process environment. The GitHub workflow downloads the exact Linux
release and verifies its SHA-256 before running these commands. It installs no
npm packages. Lean is a development verification tool, not a dependency of the
published caveat-lang kit.

The gate compares the elaborated theorem inventory with theorems.json, checks
the authored-law registry, audits transitive axioms, and requires kernel replay
of every model/law source module. Actual incomplete-proof and indirect custom
axiom controls must be refused for their intended reasons. Parser tests also
reject missing/extra theorem reports and unapproved native-evaluation axioms.

test-results/lean-verification/report.json records the source and gate hashes,
Git revision and working-tree status, actual compiler identity, exact commands,
axioms, replay scope, and control results. Command logs are stored beside it.
A failed gate overwrites the receipt with failure rather than leaving a prior
success as the current result. Inspect the receipt for a particular execution;
these instructions alone do not certify a run.

## What the model means

`Provenance` has distinct evidence and caveat lists. Union is append, interpreted
extensionally through membership: duplicates and order are not additional
evidence. The membership theorems prove both preservation and absence of
injected dependencies. These are unbounded mathematical finite-list laws,
not a proof of Rust's bounded `BTreeSet` implementation or serialization.

`Tracked` carries a number, full lineage, selected grounds, and a proof that
grounds are included in lineage. The constructor proofs establish that the
core operations can preserve this invariant. The numeric representation is
Lean `Int`; numeric parsing, floating-point behavior, and arithmetic failure
are outside this slice.

The exact flow equations go beyond the subset invariant:

- Eager combination retains both inputs' lineage and grounds, even when its
  chosen numeric result is zero.
- Ignoring an already evaluated argument keeps both of its metadata channels.
- A lazy branch retains the condition and the selected content in both
  channels. Its equation is independent of the unselected branch. This is a
  core combinator, not a proof about the Rust evaluator's control flow.
- An external guard joins lineage only. A successful guarded write takes the
  body's value and grounds, replaces prior dependencies, and joins only the
  guard's lineage. A skipped guarded write keeps its old
  value and grounds and adds the evaluated guard's lineage.
- Citations may replace grounds only with a subset of existing lineage.
  Accepted citations retain the exact requested grounds, existing lineage,
  and number. An unsupported evidence identity or caveat is rejected.
- Nonempty witness theorems exercise these rules with distinct reading, guard,
  and unused evidence and their caveats. They cannot pass by erasing all
  metadata.

## Outcome and session boundary

`Outcome` distinguishes `accepted`, `rejected`, and `fatal`.
`resumeSession` models the host contract: acceptance makes the supplied new
state available, a classified rejection retains the supplied prior state, and
fatal outcomes provide no usable session.

Citation failure maps to rejection origin `evaluation`, code
`ungrounded_citation`. Provenance capacity overflow is a distinct modeled cause
but maps to fatal code `unclassified` under the current
[dispatch contract](../../spec/caveat-dispatch-0.1.md). It must not be treated as
`limit/identifier_limit`, which concerns the separate interned-identifier
resource.

The outcome equations do not prove that Rust detects overflow correctly,
that an event rolls back every runtime field, or that a caller actually discards
a fatal session. Those boundaries require targeted runtime and host tests.
The scaffold does not compute identifier counts or UTF-8 byte limits, so its
unbounded union equations make no claim about an overflowing runtime union.

## Deferred bridge

A later slice may execute these same modeled operations on shared cases and
compare their projections against Rust and WASM. Until that bridge exists and
runs, these proofs establish properties of this Lean core only. Sampled
executable agreement would still be conformance evidence, not a proof of
Rust-to-Lean refinement.

Evidence observation, renewal and occurrence identity, save/restore, complete
session rollback, source parsing, function evaluation, graph semantics, work
budgets, and effect scheduling are not formalized here. CAVEAT's evidence
remains supplied evidence; the model does not authenticate it or turn it into
a truth guarantee.
