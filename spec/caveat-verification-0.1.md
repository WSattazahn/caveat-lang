# Caveat Verification Pilot 0.1 — semantics and proof boundary

The first slice provides the semantics contract, targeted boundary tests, and a
pinned Lean core with provenance-constructor laws. The second adds an executable
model for a bounded guarded-assignment/citation fragment and a comparison gate
against production native Rust and WebAssembly. The scope of that fragment and
the gate's acceptance requirements are registered below. Their implementation
is distinct from a successful execution receipt.

A Lean proof establishes a property of its stated model and assumptions.
Testing that model against Caveat provides evidence of correspondence, not
a proof that the Rust implementation refines the model. Native Rust and
WebAssembly execute the same Rust interpreter: their agreement checks build and
host boundaries but is not agreement between independent language semantics.
Neither result authenticates evidence or establishes an authored policy's truth,
relevance, sufficiency, or authority to act.

## Authoritative semantics and extension precedence

Existing specifications remain authoritative. An explicit extension applies to
its earlier profile; the verifier must not prove obsolete wording from an older
profile. A discrepancy requires a small witness and resolution against the
contract, not changing expected answers to match whichever runner was inspected.

| Question | Contract | Required interpretation |
| --- | --- | --- |
| Which dependencies survive computation? | [Reactive 0.3](caveat-reactive-0.3.md) | All dependencies actually evaluated, including ignored eager arguments; not untaken operands. |
| Which dependencies are grounds? | [Explanations 0.2](caveat-explanations-0.2.md) | Grounds are content; lineage also records control. Both retain their evidence and caveat categories. |
| Can citations choose less? | [Explanations 0.2](caveat-explanations-0.2.md) | Citations read grounds, replace the assignment's grounds, and must be included in its new lineage. |
| Does observation require a stance? | [Neutral Reveal 0.1](caveat-neutral-reveal-0.1.md) | No. Neutral observation enables `qualified` without support or opposition, extending Reactive 0.3's earlier wording. |
| Which observation order matters? | [Neutral Reveal 0.1](caveat-neutral-reveal-0.1.md) | First observation of each occurrence, including a neutral observation before its later stance. |
| May evidence be used before observation? | [Observation Order 0.1](caveat-observation-order-0.1.md) | Declaring evidence is insufficient; static checks and reached runtime checks have distinct boundaries. |
| What is a refusal versus a failure? | [Dispatch Outcomes 0.1](caveat-dispatch-0.1.md) | Accepted, classified Rejected, and Fatal are different results. |
| What may a restored session assume? | [Save 0.1](caveat-save-0.1.md) | Validation must establish state invariants before restored state is admitted. Unsigned consistency is not historical authenticity. |

## Required dependency laws

`L(v)` denotes a value's lineage and `G(v)` its grounds. Each consists of an
evidence set and a caveat set; inclusion and union operate on each category
separately. The invariant `G(v)` included in `L(v)` is necessary but insufficient:
an implementation that erases both channels everywhere would satisfy it. A
model and its comparison suite must also establish the actual dependency flow.

- Plain literals have empty channels. State reads obtain that state's current
  value and channels. They must not reconstruct dependencies from the numeric
  result or consult an unrelated current evidence alias.
- Eager arithmetic and comparisons union every evaluated operand's respective
  channels. Cancellation or multiplication by zero does not erase a read.
  An ignored eager function argument still contributes its dependencies.
- Lazy `and`, `or`, and `if` retain the condition and evaluated operands only.
  An untaken branch neither contributes provenance nor requires its evidence
  to be observed. A condition inside `if` or `require` is content, so its
  grounds contribute; it is not an external rule guard.
- `qualified(v, e, ...)` preserves the numeric result and existing channels,
  adds observed evidence and its inherited/explicit qualifications, and retains
  the observation's control lineage where the current profiles require it.
  Grounds omit the revealing guard. The exact host provenance rules must be
  represented or explicitly excluded, never replaced by an empty default.
- A successful assignment's new lineage is the expression lineage union the
  evaluated external guard's lineage. Without `because`, its grounds are the
  expression's grounds. With `because`, they are exactly the union of the
  citations' grounds, after checking inclusion in the new lineage.
  `because nothing` therefore yields empty grounds and leaves lineage intact.
- A skipped assignment retains the old value and grounds and unions the
  evaluated guard into the old lineage. It does not evaluate the assignment's
  value or citations. A later independent successful assignment can replace
  earlier lineage; global provenance monotonicity is not a language invariant.
- Grounded citations may omit dependencies but may never invent them. The
  inclusion check compares both evidence and caveat categories. A later change
  in a cited state can make a citation invalid at the event where it is reached.

An initial proof about provenance constructors must state that limited scope.
Giving every modeled value a proof field `grounds included in lineage` can make
constructor safety explicit, but an accepted-state theorem that only projects
that field does not by itself verify evaluation, citation behavior, source
translation, or Rust correspondence. Include nontrivial success and failure
witnesses and exact-flow properties; do not count impossible assumptions or
all-refusal executions as evidence of correspondence.

## Ordered observations and frozen history

A first neutral reveal observes the current occurrence once without creating a
claim, stance, reading value, or new occurrence. Repetition does not reorder it.
Revealing guards and inherited call dependencies are observation lineage, not
content grounds. A skipped reveal records only the specified control dependency;
it must not fabricate observation. Renewal alone leaves the new occurrence
unobserved, and old dependencies continue naming their original occurrences.

Provenance membership can be compared as sets. Observation order, event order,
reading order, revision order, and journal order must be compared as sequences.
For example, neutral B followed by supporting A then supporting B keeps
observation order `[B, A]`, despite the different stance-edge order. A generic
sort or set comparison of every array would erase this distinction.

A missing optional observation ledger in a stance-only snapshot does not mean
nothing was observed. A later comparison must either implement the documented
representation rule or explicitly restrict its admitted corpus to initially
unobserved programs that use neutral reveal exclusively.

Current state can acquire a later caveat without rewriting a frozen commitment
basis/grounds, reading archive, or decision journal. Reading historical content
and updating an immutable historical record are different operations. A model
that has not implemented these temporal rules must exclude them from its claims.

## Outcomes and bounded failure

Models and adapters must distinguish these result constructors:

- **Accepted**: the event publishes the complete resulting modeled state.
- **Rejected**: the runtime returns a classified refusal; the full observable
  session remains unchanged and can receive another event.
- **Fatal**: dispatch returns an unclassified fatal report or the runtime traps;
  the caller discards the session. No follow-up snapshot, save, or event may be
  used to pretend that a fatal session is a reusable rejection.

Load failure is not a dispatched rejection. Harness/process/JSON failures are
also not domain rejections or valid model results. Native and WebAssembly
adapters must use the structured outcome API, not classify diagnostic strings.
Expected rejections must name the expected origin/code when classification
matters; `false` alone is not an adequate comparison.

The current relevant mappings are:

| Reached condition | Dispatch result |
| --- | --- |
| Source `reject` | Rejected, `policy/reject` |
| Ungrounded authored citation | Rejected, `evaluation/ungrounded_citation` |
| Numeric assignment outside its declared range | Rejected, `evaluation/bound_exceeded` |
| Payload outside its declared bound | Rejected, `input/bound_exceeded` |
| Provenance identifier-count or name-byte overflow | Fatal, `unclassified` |
| Division by zero, a nonfinite result, a negative square root or an unavailable history read | Rejected, `evaluation/expression` |
| A false `require` | Rejected, `evaluation/requirement_failed` |
| `id_text` of a number that is no identifier's handle | Rejected, `evaluation/expression` (fatal `unclassified` through rc.12) |

Provenance currently allows at most 1,024 evidence-plus-caveat identifiers and
65,536 UTF-8 name bytes. Deduplicate within each category before accounting;
count bytes, not characters. `Provenance::merge` checks the complete result
before extending either set, so overflow leaves that receiver unchanged and
never truncates dependencies. This local merge property is distinct from the
host rule requiring a session to be discarded after fatal dispatch. A later
bounded model must match the actual overflow classification and limit values;
it may not turn every exhausted bound into ordinary Rejected.

[verification_outcomes.rs](../runtime/tests/verification_outcomes.rs) exercises
both production provenance limits through dispatch: an exactly full provenance is accepted,
while adding the next dependency returns `Fatal/unclassified`. The tests discard
fatal sessions instead of querying or reusing them. These boundary witnesses
record the current mapping; they do not change it or prove the full evaluator.

A first algebra model may omit limits only if it says so and does not claim the
bounded runtime's failure behavior. Integer or real arithmetic likewise cannot
stand in for Rust binary64 in a numeric-equivalence claim. Signed zero is
observable. Any restricted exact-integer comparison corpus must register and
check its domain, including intermediate results, rather than assume it.

## Restoration boundary repaired in the first slice

The grounds-inclusion contract applies to admitted restored state, not only to
values constructed by events. The targeted witnesses in
[grounds_restore.rs](../runtime/tests/grounds_restore.rs) reproduce separately
an observed but unrelated evidence name and a declared but unrelated caveat in
saved state grounds and commitment grounds. For commitments, the witnesses also
change the journal to agree with the false grounds, so journal agreement alone
cannot satisfy the invariant. The four negative witnesses were accepted without
the new boundary check.

[reactive_save.rs](../runtime/src/reactive_save.rs) now checks saved state
grounds against saved lineage after the existing name/observation/value checks.
Omitted state grounds still default to lineage. Commitment grounds are checked
against the frozen commitment basis after the existing journal and graph/basis
consistency checks, preserving those diagnostics.

Positive witnesses cover neutral observation with empty authored grounds and
continued original/restored sessions, omitted/equal state grounds, and a later
caveat that leaves frozen commitment grounds and basis unchanged. These are Rust
regression tests for an implemented boundary; they do not constitute a Lean
proof of arbitrary save validation or proof of historical reachability.

## Core model and proof coverage

[Model.lean](../proofs/lean/Caveat/Model.lean) represents provenance as two
unbounded lists with set-membership inclusion, and numeric payloads as integers.
Its tracked constructors combine channels, retain ignored arguments, distinguish
external control from selected content, and check citation inclusion. A tracked
value carries a proof of its own grounds inclusion. Its qualification constructor
is an algebra operation; it does not establish that a host observed evidence.

[Laws.lean](../proofs/lean/Caveat/Laws.lean) states exact channel flow for those
constructors and includes nonempty witnesses for eager arguments, selected
content, skipped guards, narrowed citations, and foreign-citation refusal. Union
idempotence is about membership, not canonical list equality. These laws do not
verify resource accounting, arbitrary source evaluation, or the Rust mapping.

The scaffold distinguishes Accepted, Rejected, and Fatal. Its generic
`resumeSession` describes the host rule: accepted uses the supplied new state,
rejected retains the old state, and fatal supplies no reusable session. The
provenance-overflow constructor records Fatal/unclassified. These are explicit
model definitions and laws about them, not a proof that Rust detects every
overflow or rolls back an arbitrary event. The targeted Rust boundary tests
supply separate implementation evidence. The second-slice bridge below executes
these same constructors on a restricted event model; it adds no formal save
model, general observation semantics, or Rust refinement proof.

## Second slice: bounded executable comparison

The input schema is `caveat-guard-citation/0.2`. The fixture module
[lean-conformance-cases.mjs](../scripts/lean-conformance-cases.mjs) validates and
renders this fragment; it does not calculate expected semantic outputs.

- The only state names are `a`, `b`, `g`, `x`, and `y`, initially plain zero.
  Seeds `a` and `b` are integers in `-1000..1000`; `g` is zero or one. One
  `seed` event neutrally reveals `eg`, `ea`, and `eb` in that order, then sets
  `a`, `b`, and `g` using their respective evidence and declared caveats
  `ea/ca`, `eb/cb`, and `eg/cg`.
- A request contains one to eight steps, each with one to four sequential
  actions, with at most eight actions in total. Each action requires `target`,
  `body`, `guard`, and `citations`. Unknown fields or state names are invalid.
  `body` is either a list of zero to four state reads added in order, or an
  exact object `{condition, then, else}`. The condition is one state reference
  tested with `!= 0`; each branch is a list of zero to four state reads. Empty
  sums are zero. Repeated reads retain numeric multiplicity. Conditional bodies
  render `if(condition != 0, then-sum, else-sum)` and evaluate only the selected
  branch. The condition and selected branch contribute both lineage and grounds;
  the untaken branch contributes neither. Nested conditional bodies are excluded.
- `guard` is null or a state name tested with `!= 0`. A false guard evaluates
  neither the body nor citations; it adds guard lineage to the old target
  while retaining the old value and grounds. A true guard permits assignment
  and contributes lineage only unless the author explicitly cites its grounds.
- `citations` is null for default body grounds, or a list of zero to four
  current-state grounds to union. The empty list renders `because nothing`.
  Every citation sees the state after earlier actions in the same event but
  before its own target write, including a self-citation. Citation inclusion
  is checked against the new body-plus-guard lineage. Failure rejects the
  whole step as `evaluation/ungrounded_citation`, including earlier writes.
- Inputs are bounded to 64 KiB, IDs to 64 lowercase ASCII identifier characters,
  and numeric tokens to canonical integers without negative zero. Starting
  with the largest absolute seed or one, multiplying by each action's maximum
  branch length (eager body length for a sum), or one if larger, gives a
  conservative intermediate magnitude bound no larger than `1e9`. Rendered states use bounds `-1e9..1e9`. The gate rejects numeric
  outputs outside this exact-integer domain. This checked restriction does
  not prove general Lean integer/Rust binary64 equivalence.

[Bridge.lean](../proofs/lean/Caveat/Bridge.lean) folds sum reads through the
proved `Tracked.combine`. Its `evalBody` invokes the proved `Tracked.select`
with branch thunks for conditional bodies. It uses `guardedWrite` and `cite`
directly and applies `resumeSession` at the complete-step transaction boundary. Bridge equations
state the exact target and other-state behavior, body fold, skipped/cited/
uncited action transitions, current-state sequencing, and later rejection of
a successful prefix. Its modeled session also records the fixed seed's ordered
observations and reveal effects, clears effects after an accepted ordinary step,
and preserves all modeled fields on rejection.
[Runner.lean](../proofs/lean/Caveat/Runner.lean) decodes input and executes those
same bridge definitions; it is not a second semantic implementation. JSON
parsing/encoding and source rendering remain unproved boundaries.

The registered corpus contains 40 fixed witnesses (the original 21 and 19
conditional-body witnesses) and 32 cases generated by xorshift32 with seed
`0x05eedca7`. All 72 execute; a rejected event remains a case result and does
not remove a case. Fixed witnesses include cancellation
with nonempty dependencies, duplicate reads, true/false/negative guards,
control-only lineage, explicit empty and narrowed grounds, current-state and
self-citations, guard-only citations, fresh assignment, invalid citation,
and late rejection followed by another event. Conditional witnesses exercise
both selections with distinct condition/then/else evidence and caveats, equal
branch values, selected zero, negative conditions, empty selected sums, repeated
reads, narrowed condition/branch grounds on both selections, external guards,
condition/selected/untaken citations, transactional rollback, and sequential
changes in selection.
Exact-flow laws cover both outcomes, with successful nonempty witnesses and a
witness distinguishing expression conditions from external guards. These prevent
empty metadata or refusal-only agreement from standing in for the intended behavior.

The [comparison gate](../scripts/verify-lean-conformance.mjs) first requires the
proof audit, then runs the compiled Lean runner, native
[lean_conformance.rs](../runtime/examples/lean_conformance.rs), and the freshly
built kit/WebAssembly runtime. The native adapter opens production
`ReactiveSession::from_source` and dispatches through the structured outcome API;
the kit uses the production WebAssembly session API. No shadow Rust evaluator
supplies comparison results. The gate verifies the WebAssembly build-input
fingerprint and output-byte hashes before loading its artifacts, then checks
both again after execution. Source hashes must remain unchanged throughout the
gate. These are local build receipts, not authenticated attestations.

The semantic projection contains each state's number, both provenance channels,
exact outcome origin/code, ordered observations, effects, and decision journal.
The journal is always empty in this fragment; comparing it is not coverage of
commitments or historical journal semantics. Only evidence/caveat membership
is canonicalized as sets. Observation and effect arrays retain their order.
Native and WebAssembly complete snapshots, exact save text, and views are also
compared. Every classified rejection must leave those three captures unchanged.
Each pre-event prefix is restored into a twin, which receives the next event;
its outcome and complete captures must agree with the original session. A final
save roundtrip is checked separately. This is one-step continuation at every
pre-event prefix, not one restored twin executing each entire remaining suffix.

33 permanent raw-input controls exercise the compiled Lean decoder.
Each must return exactly exit code 1, no stdout, and its registered diagnostic.
They include missing/unknown fields, numeric spellings outside the domain,
size limits, conditional-object fields and branch shapes, rejection of the old
schema, and duplicate keys including escaped spellings. Duplicate keys are
compared after decoding; stdin is bounded before parsing. These protocol
refusals are reported separately from semantic mutation detection.

Eight semantic control families must be detected in 15 executions for their
intended differences. Five compilation entries covering four distinct source
mutations of the production expression evaluator supply seven executions: the
original eager cancellation drops state
`a` metadata from both channels; condition loss drops `g` for both selections;
selected-branch loss drops `a` or `b` for the corresponding selection; untaken
branch injection evaluates both branches while retaining the selected number.
Each must complete an accepted event with the same number and exactly the
registered dependency difference in both channels, still satisfying grounds
inclusion. The remaining four source variants drop a skipped guard, move guard
metadata into grounds, bypass an invalid citation, or reorder seed observations.
Each source variant runs through native and WebAssembly (eight executions).
These are source-translation controls, not independently mutated interpreters.
Mutated evaluator sources and executable hashes are retained in the receipt.
A compiled rejected-state-leak mutation remains outside this registered gate.
Every rejected event is checked for complete rollback, but that comparison is
not mutation coverage of the transaction implementation.
A build failure, fatal outcome, crash, malformed trace, unrelated difference,
or timeout is an infrastructure failure, never a successful semantic kill.

The receipt records source/model-input hashes, generator identity, events,
commands, proof audit, artifact identity, comparisons, and control discrepancies.
A named-case replay is diagnostic and does not claim the full mutation gate.
Only a completed full-gate receipt establishes a passing execution; this
contract does not record one by itself.

This fragment excludes arbitrary expressions, functions, nested conditional
bodies, other lazy expression forms, strings, fractional/exceptional arithmetic,
provenance/work overflow,
graph predicates, guarded observation, repeated reveal/renewal, late caveats,
withdrawal, readings, commitments, and historical occurrence or journal rules.
The general core equations and separate Rust boundary tests retain their own
scopes; they do not extend the executable corpus to those features. Restore
checks are runtime conformance evidence, not a formal save-validation proof.
Fatal remains an explicit outcome in the model, but no admitted case is meant
to produce it; a fatal result fails this comparison instead of matching a refusal.

## Third slice: late qualification and recorded commitments

[Late Qualification 0.1](caveat-late-qualification-0.1.md) §3 says a late
caveat leaves commitment bases and grounds as they were.
[Late.lean](../proofs/lean/Caveat/Late.lean) models one
`[when GUARD] qualify EVIDENCE with CAVEAT` step over named current values,
named commitments (frozen `using` basis with optional number, and grounds),
observed evidence and the step's qualify effects. A false guard changes nothing;
unobserved evidence is refused as `evaluation/unobserved_evidence`; otherwise a
value that read the evidence gains the caveat, the caveats that qualify it and
the guard's lineage in its lineage, and the caveat and its qualifiers in its
grounds when its grounds include the evidence. Twelve registered laws prove
that an accepted step, any continuing session and any accepted sequence of
steps leave every commitment's basis and grounds unchanged, together with the
contrast on current values, the refusal classification and a nonempty witness.
How commitments are created, reopening, the decision journal, reading archives,
scheduled qualification, renewal and the graph closure of qualifiers are not
modeled.

The runner's `caveat-late-qualification/0.1` schema takes a `before` ledger and
one qualification and returns the `after` frame. Its comparison differs from
the second slice: the model does not execute the program from its start. The
gate runs nine registered programs
([lean-late-qualification-cases.mjs](../scripts/lean-late-qualification-cases.mjs))
through the same native adapter and kit WebAssembly session, with the same
rollback and prefix-restore checks, and for each of their 14 qualification
events gives the model the runtime's state before the event. The model's
prediction must equal the runtime's projected state after it on both runtimes:
each state's number and both channels, each commitment's basis (number,
evidence, caveats) and grounds, observations, the outcome, and an accepted
step's effects. The corpus covers a guarded qualification of evidence in a
commitment's basis and grounds, evidence in a value's lineage but not its
grounds, evidence reaching values only as a guard, a skipped guard, unobserved
evidence, repetition, a chained qualifier, an unrelated commitment, a retained
caveat equal to the late one, and three commitments with three qualifications.
Eleven more decoder controls cover this schema.

Two further semantic control families compile mutants of
`apply_qualification` in `runtime/src/reactive.rs` and must be detected on two
witnesses each: the late caveat also reaches commitment bases; and it reaches
bases, grounds and the journal entry together, so restore validation still
accepts the saves. Each must produce an accepted step whose only projected
difference is the late caveat and its qualifiers in exactly the commitment
records that include the evidence. This is sampled agreement on the
qualification step from states the runtime reached; it is not a proof that the
runtime's commit or qualify implementation refines the model.

## Requirements for extending the model and comparison

1. Register the precise fragment, semantics matrix, assumptions, theorem targets,
   and required witnesses before treating the model as an oracle. Every modeled
   syntax constructor and relevant branch needs a reached witness, including
   successful changes with nonempty, distinct provenance channels.
2. Execute the same Lean definitions whose properties are proved. A separate
   unproved reimplementation in the Lean CLI would reopen the model gap. Keep
   JSON decoding, source rendering, and any extraction/adapter assumptions
   explicit. Generated expected results must not come from Rust.
3. Drive production entrypoints: native `ReactiveSession::from_source` and
   `dispatch_outcome_json`, and the freshly built `WebReactiveSession` structured
   outcome interface through the kit. Do not substitute a shadow evaluator.
   Compare exact acceptance/rejection/fatal classification and the modeled
   values, channels, effects, and ordered observations after every event.
4. On classified rejection compare the complete before/after runtime snapshot,
   save, and view, not just selected fields. Restore genuine saves at registered
   prefixes, compare full state, and state exactly how much subsequent history
   runs on original and restored sessions. Treat save numeric text as authoritative rather
   than parsing/reserializing away signed zero.
5. Retain deterministic case identity, source and model-input hashes, seed,
   exact event stream, build/toolchain identity, first differing path, and a
   replay command. A reduced witness must independently reproduce. Unknown
   fields, unsupported constructs, silent case skips, fatal-as-rejection
   normalization, and missing runtime artifacts fail the run.
6. Check semantic negative controls: dropped eager dependency, invented untaken
   dependency, control moved into grounds, accepted invalid citation, leaked
   rejected mutation, and reordered observations. A control counts as detected
   only when a completed run reaches the intended semantic discrepancy.
   Compilation errors, tool errors, crashes, timeouts, and malformed output do
   not count as successful semantic-mutant detection.

The receipt must separately report theorem checks, native comparisons,
WebAssembly comparisons, restoration/rollback checks, mutation controls, and
omissions. Fixed acceptance/refusal witnesses and constructor/branch coverage
must prevent a pair of always-empty or always-rejecting runners from passing.
A large case count is not a substitute for those requirements.

The scaffold uses the [pinned Lean toolchain](../proofs/lean/lean-toolchain).
The [proof gate](../scripts/verify-lean.mjs) checks the exhaustive theorem
inventory, authored-law registry, transitive axiom allowlist, and explicit
kernel replay. It also exercises incomplete-proof and custom-axiom refusal
controls; a successful build alone is insufficient. See the
[scaffold guide](../proofs/lean/README.md) for commands, trust boundary, and
execution receipts. This verifies the stated Lean definitions and laws only.

The comparison gate must run its manifest without silent skips, preserve
failing inputs, and detect semantic mutations for the reasons specified above.
These gates supplement existing Rust, kit, and affected WebAssembly/browser
checks. This contract is not authorization to publish a release.

## Changes

- rc.13: the third slice, late qualification and recorded commitments: twelve
  Lean laws, the `caveat-late-qualification/0.1` runner schema, nine
  conformance programs, eleven decoder controls and two mutation families.
