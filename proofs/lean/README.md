# CAVEAT Lean model and narrow executable bridge

This pinned rc.8 verification project contains core dependency and outcome
proofs plus a narrow executable guard-and-citation comparison model. The runner
executes the same constructors covered by the proofs. Its accepted fragment is
small and explicit; it is not a model of the complete CAVEAT language.

## Build and inspect

Use Lean `leanprover/lean4:v4.34.1`, as pinned by `lean-toolchain`. The model imports bundled `Std`; the audit also uses bundled Lean metaprogramming APIs. No external Lean package is required. From this directory:

```text
lake build --wfail
lake env lean Audit.lean
lake env leanchecker --verbose Caveat
```

`Caveat.lean` imports the core model, laws, executable bridge, and runner. `laws.json` records 58 authored theorem declarations, including four inclusion helper lemmas and 16 bridge laws. `theorems.json` inventories all 128 public `Caveat` theorems in the elaborated environment, including compiler-generated declarations. `Audit.lean` prints their transitive axioms, emits the actual environment inventory, and rejects forbidden axioms in every public `Caveat` declaration (including definitions and unused custom axioms).
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
of every core, law, bridge, and runner source module. Actual incomplete-proof and indirect custom
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
Lean `Int`. The executable decoder admits a bounded integer fragment; the
decoder, floating-point behavior, and arithmetic failure are not formally proved.

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
The core does not compute provenance identifier counts or provenance UTF-8 byte limits, so its
unbounded union equations make no claim about an overflowing runtime union.

## Executable guard-and-citation fragment

`lake build --wfail` also builds `.lake/build/bin/caveat_compare` (`.exe` on
Windows). It reads one JSON request from stdin and writes one compact JSON
response. Invalid requests produce stderr and exit status 1; they are
infrastructure failures, not modeled CAVEAT rejections.

The protocol schema is `caveat-guard-citation/0.1`:

```json
{
  "schema": "caveat-guard-citation/0.1",
  "id": "guard-citation-example",
  "seed": {"a": 3, "b": 5, "g": 1},
  "steps": [{"actions": [{
    "target": "x", "body": ["a"], "guard": "g", "citations": ["g"]
  }]}]
}
```

All shown fields are required, including nullable `guard` and `citations`.
Unknown fields, wrong types, and unknown state references are rejected.
Repeated keys in any one JSON object are rejected after decoding escapes, so
an escaped spelling cannot hide a duplicate. Separate objects have separate
key scopes; bundled Json.parse remains responsible for JSON syntax.
The five state names are `a`, `b`, `g`, `x`, and `y`. The request id has 1..64
ASCII characters, begins with a lowercase letter, and otherwise contains only
lowercase letters, digits, or hyphens. Input is valid UTF-8 and at most 65,536
bytes. The input loader reads at most 65,537 bytes (the limit plus one sentinel)
and rejects overflow without waiting for EOF or buffering the entire stream.
Seeds `a` and `b` are integers in [-1000,1000]; `g` is 0 or 1. Numeric tokens
use integer spelling, excluding fractions, exponents, and negative zero.
There are 1..8 steps, 1..4 actions per step, and at most eight actions in total.
Body and citation lists contain 0..4 references. A conservative magnitude bound
multiplies the largest seed magnitude (at least one) by each body length (at
least one), regardless of guards or rejection, and must remain at most 10^9.
These resource restrictions admit a small exactly representable comparison
domain; no theorem proves the decoder or Rust's resource-bound implementation.

The first frame has five plain zero states. The next frame seeds `a`, `b`, and
`g` with evidence/caveat pairs `ea/ca`, `eb/cb`, and `eg/cg`. This fixed fixture
uses neutral reveals in chronological order `eg`, `ea`, `eb`; it models that
initialization directly, not arbitrary reveal semantics. Each later frame
represents one transaction containing the listed sequential actions.

A body eagerly sums its current referenced states, folding the existing
`Tracked.combine` from plain zero. A null guard is plain one; a reference reads
that state's current tracked value. `Tracked.guardedWrite` selects a successful
body with guard lineage or retains the previous value and grounds with added
guard lineage. A false guard skips body and citation evaluation. Null citations
retain the body's grounds. A citation list unions the current referenced
states' grounds before the target write, then invokes the existing `cite`
check against the candidate lineage. An empty list explicitly clears grounds.
Guard-only citations are valid when their grounds are included in the candidate
lineage. Self-citation reads the target's current pre-write grounds.

`Bridge.runAction`, `runActions`, and `runSessionStep` compose those operations.
The 16 bridge laws cover exact target selection, preservation of other states,
body-fold combination, skipped and accepted writes, invalid citations,
sequential reads of updated state, rejection after earlier successful writes,
and modeled-session rollback. A rejected step retains values, observations,
and effects from before the entire step. Acceptance clears effects and retains
observations. Seed observation/effect ordering has explicit equations. The
fragment has no fatal-producing operation; fatal results remain distinct in
the core outcome type and are exercised by separate runtime boundary tests.

Responses contain `schema`, `id`, and `frames` (initial, accepted seed, then
one per step). Each frame contains `outcome`, all five `states`, `observations`,
`effects`, and an empty `decision_journal`. Each state exposes its numeric
`value` and separate `lineage` and `grounds` objects with `evidence` and `caveats`
arrays. Only provenance arrays are deduplicated and sorted. Observation and
effect order is preserved. The seed emits three reveal effects; accepted steps
emit none, and rejected steps retain the prior effects. The fragment contains
no commitments, so the empty decision journal is an explicit restriction.

`Runner.lean` is an unproved boundary adapter for JSON admission, serialization,
and IO. Kernel replay checks its Lean declarations, but does not independently
check the generated executable, C backend, or machine execution. It does not prove
that its parser or projection corresponds to the source program or runtime.
The comparison harness validates and renders shared cases, executes this Lean
model and the production native/WASM runtimes, and compares the registered
projection. Run `npm run test:lean-conformance` and
`npm run verify:lean-conformance` from the repository root after its required
build. Consult that run's receipt for actual comparisons and mutation controls.
Sampled executable agreement is conformance evidence, not a proof of
Rust-to-Lean refinement or correctness for all admitted programs.

Evidence renewal and occurrence identity, arbitrary observations, save/restore,
complete runtime session rollback, source parsing, function evaluation, graph
semantics, work budgets, and general effect scheduling are not formalized here.
CAVEAT's evidence remains supplied evidence; the model does not authenticate it
or turn it into a truth guarantee.
