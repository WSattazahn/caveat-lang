# CAVEAT Lean model and narrow executable bridge

This pinned rc.9 verification project contains core dependency and outcome
proofs plus a narrow executable branch, guard, and citation comparison model.
Since rc.13 it also models one late-qualification step over recorded
commitments, since rc.15 one reopening step, and since 1.2 the Caveat Sources
invariant over whole event histories. The runner executes the same constructors covered by the proofs.
Its accepted fragments are small and explicit; they are not a model of the
complete CAVEAT language.

## Build and inspect

Use Lean `leanprover/lean4:v4.34.1`, as pinned by `lean-toolchain`. The model imports bundled `Std`; the audit also uses bundled Lean metaprogramming APIs. No external Lean package is required. From this directory:

```text
lake build --wfail
lake env lean Audit.lean
lake env leanchecker --verbose Caveat
```

`Caveat.lean` imports the core model, laws, executable bridge, late-qualification model, reopening model, Caveat Sources model, and runner. `laws.json` records 144 authored theorem declarations, including six inclusion helper lemmas, 26 bridge laws, 12 late-qualification laws, 9 take-back laws, 17 reopening laws and 36 Caveat Sources laws. `theorems.json` inventories all 542 public `Caveat` theorems in the elaborated environment, including compiler-generated declarations. `Audit.lean` prints their transitive axioms, emits the actual environment inventory, and rejects forbidden axioms in every public `Caveat` declaration (including definitions and unused custom axioms).
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
`ungrounded_citation`. A late qualification of unobserved evidence maps to
origin `evaluation`, code `unobserved_evidence`, and so does a reopening
whose cause was not observed. Reopening a commitment that was never made maps
to origin `evaluation`, code `not_committed`. Provenance capacity overflow is a distinct modeled cause
but maps to fatal code `unclassified` under the current
[dispatch contract](../../spec/caveat-dispatch-0.1.md). It must not be treated as
`limit/identifier_limit`, which concerns the separate interned-identifier
resource.

The outcome equations do not prove that Rust detects overflow correctly,
that an event rolls back every runtime field, or that a caller actually discards
a fatal session. Those boundaries require targeted runtime and host tests.
The core does not compute provenance identifier counts or provenance UTF-8 byte limits, so its
unbounded union equations make no claim about an overflowing runtime union.

## Executable branch, guard, and citation fragment

`lake build --wfail` also builds `.lake/build/bin/caveat_compare` (`.exe` on
Windows). It reads one JSON request from stdin and writes one compact JSON
response. Invalid requests produce stderr and exit status 1; they are
infrastructure failures, not modeled CAVEAT rejections.

The protocol schema is `caveat-guard-citation/0.2`:

```json
{
  "schema": "caveat-guard-citation/0.2",
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
An eager body is an array of 0..4 state references. Alternatively, a body is
exactly `{"condition":"g","then":["a"],"else":["b"]}`: the condition is one state
reference and each branch is an array of 0..4 references. Nested conditional
bodies, missing or extra fields, and invalid branch references are rejected.
Citation lists also contain 0..4 references. A conservative magnitude bound
multiplies the largest seed magnitude (at least one) by each eager body length,
or the larger branch length for a conditional (each multiplier at least one),
regardless of guards or rejection, and must remain at most 10^9.
These resource restrictions admit a small exactly representable comparison
domain; no theorem proves the decoder or Rust's resource-bound implementation.

The first frame has five plain zero states. The next frame seeds `a`, `b`, and
`g` with evidence/caveat pairs `ea/ca`, `eb/cb`, and `eg/cg`. This fixed fixture
uses neutral reveals in chronological order `eg`, `ea`, `eb`; it models that
initialization directly, not arbitrary reveal semantics. Each later frame
represents one transaction containing the listed sequential actions.

An eager body sums its current referenced states, folding the existing
`Tracked.combine` from plain zero. `Bridge.evalBody` executes a conditional
through the proved `Tracked.select`, with each branch sum supplied as a thunk.
A nonzero condition selects `then`; zero selects `else`. Only the selected sum
contributes its value and dependencies. The condition contributes its lineage
and grounds to their respective channels, even when the selected sum is empty.
This expression condition differs from the external statement guard, which
contributes lineage only. A null guard is plain one; a reference reads that
state's current tracked value. `Tracked.guardedWrite` selects a successful
body with guard lineage or retains the previous value and grounds with added
guard lineage. A false guard skips body and citation evaluation. Null citations
retain the body's grounds. A citation list unions the current referenced
states' grounds before the target write, then invokes the existing `cite`
check against the candidate lineage. An empty list explicitly clears grounds.
Guard-only citations are valid when their grounds are included in the candidate
lineage. Self-citation reads the target's current pre-write grounds.

`Bridge.runAction`, `runActions`, and `runSessionStep` compose those operations.
The 26 bridge laws cover exact target selection, preservation of other states,
body-fold combination, true/false branch values and exact lineage/grounds,
nonempty branch witnesses and the distinction between expression conditions
and statement guards, skipped and accepted writes, invalid citations,
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

## Late qualification and recorded commitments

[Late.lean](Caveat/Late.lean) models one step of
`[when GUARD] qualify EVIDENCE with CAVEAT` from
[Late Qualification 0.1](../../spec/caveat-late-qualification-0.1.md) over a
`Ledger`: named current values, named commitments (each a frozen `using` basis
with an optional number, and grounds), observed evidence, and the step's
qualify effects. `Qualification` carries the evidence, the caveat, the caveats
that qualify it in the graph, and the evaluated guard.

`qualifyStep` is the step. A false guard changes nothing and reports no
effect. Unobserved evidence is refused as `evaluation/unobserved_evidence`.
Otherwise each current value that read the evidence gains the caveat, its
qualifiers, and the guard's lineage in its lineage, and the caveat and its
qualifiers in its grounds when its grounds include the evidence. Grounds stay
within lineage by construction. Values that never read the evidence, and every
commitment, are left as they were.

What is proved, for every ledger and qualification:

- An accepted step leaves the commitment list equal, so each commitment keeps
  exactly the basis and grounds it was made on
  (`late_qualification_preserves_commitments`,
  `late_qualification_preserves_basis_and_grounds`). The session that
  continues after any outcome, refusal included, has the same commitments
  (`late_qualification_session_preserves_commitments`), and so does any
  accepted sequence of qualifications (`late_qualifications_preserve_commitments`).
- The contrast with current values: a value that read the evidence gains the
  caveat and the guard's lineage and keeps its number; one grounded on the
  evidence gains the caveat in its grounds; the guard never enters grounds; a
  value that did not read the evidence is unchanged.
- Unobserved evidence is refused with that classification; a false guard
  changes no value. A nonempty witness, a value and a commitment both resting
  on `ea`, shows the value learning `late` and its qualifier while the
  commitment's basis and grounds stay `{ea; ca}`.

This holds of the model because `qualifyStep` maps only values; the theorems
make that a checked statement rather than a reading of the definition. How
commitments come to exist (`commit … using … retaining`, guards, series,
predecessors), reading archives, scheduled `qualify … after`, renewal and
occurrence identity, and the graph closure that supplies a caveat's qualifiers
are not modeled. Reopening is modeled separately, below.

The runner accepts a second schema, `caveat-late-qualification/0.1`: a
`before` ledger (`states`, `commitments` with `basis` `{value, evidence,
caveats}` and `grounds`, `observations`) and one `qualification`
(`evidence`, `caveat`, `qualifiers`, `guard` naming a state or null). Names are
lowercase identifiers; at most 8 states, 4 commitments, 8 qualifiers and 16
names per provenance or observation list;
numbers are integers within ±10^9; grounds outside a state's lineage are
refused. The response is the `after` frame: outcome, states, commitments,
observations, and, for an accepted step only, its qualify effects. A refused
step's effects are the previous event's and are checked by the rollback
comparison, not by the model.

The conformance gate runs nine registered programs
([lean-late-qualification-cases.mjs](../../scripts/lean-late-qualification-cases.mjs))
on the native and WebAssembly runtimes: seed, write, commit, then one event per
qualification. For each qualification event it gives the model the runtime's
own state before the event and compares the model's prediction with the
runtime's state after it, on both runtimes. Two compiled runtime mutants must
be caught by that comparison: a qualification that also reaches commitment
bases, and one that reaches bases, grounds and the journal together, so that
the runtime's own restore checks still accept its saves. This is sampled
agreement on the qualification step from states the runtime reached, not a
proof that the runtime's commit or qualify implementation refines the model.

### Take-back

The same file models one step of `[when GUARD] unqualify EVIDENCE with
CAVEAT` from [Take-back 0.1](../../spec/caveat-take-back-0.1.md).
`TakeBack` carries the caveat, the held occurrences whose live relation the
take-back marks, and the current values for which the caveat is neither live
nor kept. The model has no graph, so those two lists are supplied as input, as
a qualification's qualifiers are; the runtime computes them, and the runtime
tests in `runtime/tests/take_back.rs` check that computation. `takeBackStep`
never refuses: a false guard and an empty take-back change nothing.

What is proved, for every ledger and take-back: an accepted step leaves the
commitment list, and so every basis and grounds, and the observations equal
(`take_back_preserves_commitments`, `take_back_preserves_basis_and_grounds`,
`take_back_preserves_observations`); a value keeps its number and evidence
(`take_back_keeps_number`), loses at most the take-back caveat from lineage
and grounds alike (`take_back_loses_at_most_the_caveat`), keeps grounds within
lineage (`take_back_grounds_within_lineage`), and is unchanged when it holds
no marked occurrence (`take_back_ignores_unrelated_value`). The witness
`round8_take_back_spares_commitment` doubts `ea` and takes the doubt back: the
value loses `doubted`, the commitment keeps `{ea; ca}`. Renewal, retirement,
departure, kept caveats and the save are not modeled, and the take-back has
no runner schema or conformance cases.

## Reopening keeps what a commitment retained and records its cause

[Reopen.lean](Caveat/Reopen.lean) models one step of
`[when GUARD] reopen COMMITMENT because EVIDENCE` over a `Ledger`: named
commitments (each a frozen basis, grounds, the caveats it retains, and the
evidence that has reopened it, in order), observed evidence, the decision
journal's entries (commitment, change, `because`, caveats), and the step's
reopen effects. A `Reopening` carries the commitment, the evidence, the
evidence's caveats at that moment (which the graph supplies), and whether the
guard held.

`reopenStep` is the step. A false guard changes nothing. A commitment that was
never made refuses the event as `evaluation/not_committed`; unobserved
evidence then refuses it as `evaluation/unobserved_evidence`. Evidence that
already reopened the commitment adds nothing. Otherwise the commitment gains
the evidence as a cause, the journal gains one `reopened` entry naming the
evidence and its caveats, and the step reports one reopen effect.

What is proved, for every ledger and reopening:

- Every commitment keeps its basis, grounds and retained caveats through an
  accepted reopening, the reopened one included
  (`reopen_preserves_records`, `reopen_retains_caveats`). The session that
  continues after any outcome keeps them (`reopen_session_preserves_records`),
  and so does any accepted sequence of reopenings
  (`reopenings_preserve_records`).
- A new cause is recorded: the journal gains exactly the entry naming the
  evidence and its caveats, and the step reports the reopen
  (`reopen_records_cause`); the commitment lists the cause after its earlier
  ones (`reopen_marks_commitment`). An accepted reopening only appends to the
  journal (`reopen_keeps_history`).
- A repeated cause and a skipped guard change nothing; an uncommitted target
  and unobserved evidence are refused with their classifications. A nonempty
  witness, `plan` resting on `ea` and retaining `late`, reopened by `eb`,
  shows the record unchanged, `eb` recorded, and the journal entry carrying
  `cb`.

The model does not cover series (a reopening of a series reopens its current
revision; earlier revisions are other commitments, which the theorems show
are untouched), `latest(...)` or `caveated(...)` causes, reopening triggers,
the `reopened` predicate's dependency tracking, or the reopening
qualifications a later revision's basis receives.

The runner accepts a third schema, `caveat-reopening/0.1`: a `before` ledger
(`commitments` with `basis`, `grounds`, `retained` and `reopened_by`,
`observations`, and `journal` entries with `commitment`, `change`, `because`
and `caveats`) and one `reopening` (`commitment`, `evidence`, `caveats`,
`guard` as a boolean). At most 4 commitments, 16 journal entries and 16 names
per list. The response is the `after` frame, with effects for an accepted
step only.

The conformance gate runs nine registered programs
([lean-reopening-cases.mjs](../../scripts/lean-reopening-cases.mjs)) on the
native and WebAssembly runtimes: seed, commit, then one event per reopening.
For each reopening event it gives the model the runtime's own state before
the event, with the guard read from the runtime's `g` and the cause's caveats
from the program's declarations, and compares the model's prediction with the
runtime's state after it. Two compiled runtime mutants must be caught by that
comparison: a reopening that drops the commitment's retained caveats, and one
whose journal entry drops its cause's caveats. The runtime's own restore checks
accept both mutants' saves, so only the comparison with the model catches them.
This is sampled agreement on the reopening step from states the runtime
reached, not a proof that the runtime's reopen implementation refines the
model.

Evidence renewal and occurrence identity, arbitrary observations, save/restore,
complete runtime session rollback, source parsing, function evaluation, nested
conditional bodies, arbitrary expressions, graph semantics, commitment
creation, series revision, the rest of the decision journal, work budgets,
and general effect scheduling are not formalized here.
CAVEAT's evidence remains supplied evidence; the model does not authenticate it
or turn it into a truth guarantee.

## Caveat Sources: the invariant over any history

[Caveat Sources 0.1](../../spec/caveat-caveat-sources-0.1.md), "The invariant",
says that for every current value, its grounds and every stored observation or
predicate qualification, a caveat is carried exactly when it has a kept route,
or a route through an occurrence whose relation is live. The take-back and
late-qualification findings D012 to D020 were each a carrier that drifted from
this: a stored guard record a `qualify` missed (D014, D020), a basis reader a
later `qualify` missed (D015), a doubt left unexplained after its occurrence
departed (D018).

`Caveat/Sources.lean` keeps every route explicitly and updates the routes event
by event, as an implementation that pushes caveats to what holds them would:

- `observe`: a `reveal` or `renew` makes a new occurrence with its declared
  relations and stores what its guard read as the occurrence's observation.
- `predicate`: a `commit`, `reopen`, `withdraw` or `examine` guard stores what
  it read.
- `set`: a state takes its body's lineage and grounds; its guard joins the
  lineage only.
- `qualify`: refused as `unobserved_evidence` without a held current
  occurrence; otherwise a new live relation, and each carrier that lists the
  occurrence gains the route through it. A current value that gains it also
  takes the guard's lineage.
- `unqualify`: marks each live relation of the caveat on a held occurrence of
  the evidence and removes the routes through it. Never refused.
- `commit`: freezes a basis as a record, the evidence it lists and the caveats
  it was made with. Reading it gives those caveats as kept routes and the
  listed evidence read live.
- `depart`: the occurrence is no longer held, and every route through it
  becomes kept.

`step_preserves` proves each event keeps the invariant; `run_preserves` and
`caveat_sources_invariant` prove it holds after any sequence of events from
an empty session, with refused events leaving the session as it was. Further
laws prove that a take-back never removes a kept route and removes only its own
caveat, that a `qualify` only adds, that departure changes no caveat a carrier
shows, and that only `commit` changes recorded bases. The witnesses
`d012_decision_keeps_flag`, `d015_late_qualification_reaches_basis_reader`,
`d018_departed_doubt_is_kept` and `d020_stored_record_take_back` run each
finding's shape on the model.

The proofs are not vacuous. `npm run verify:lean` also compiles three altered
copies of the model and requires each to fail inside the theorem that covers
it: a `qualify` that skips stored records (the shape of D014 and D020) fails
`qualify_preserves`; a departure that leaves routes through the departed
occurrence (D018) fails `depart_preserves`; a take-back that reaches a record's
own caveat fails `take_back_keeps_kept_routes`.

What the model leaves out: values and guard truth (a rule whose guard is false
is no event); `D qualifies C` chains, where a caveat arrives with the one it
qualifies; citations narrowing grounds; reading archives, decision series and
departure of records; scheduled qualification; save and restore. Like the rest
of this project it is a model of the specification, not a proof about the Rust
runtime. Whether the runtime matches it is a separate, sampled check.
