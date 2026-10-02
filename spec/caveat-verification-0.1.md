# Caveat Verification Pilot 0.1 — semantics and proof boundary

This first slice provides a semantics contract, targeted boundary tests, and a
pinned Lean scaffold with small provenance-constructor laws. Those laws concern
the scaffold's definitions only. This slice does not provide a source parser,
event interpreter, cross-runtime conformance harness, or verified-runtime result.
The subsequent interpreter model and comparison requirements below are not
implemented capabilities or release gates delivered by this slice.

A Lean proof would establish a property of its stated model and assumptions.
Testing that model against Caveat would provide evidence of correspondence, not
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

The subsequent model must distinguish these result constructors:

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
| Arithmetic failure such as division by zero | Fatal, `unclassified` |

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

## Restoration boundary repaired in this slice

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

## Present scaffold coverage

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
supply separate implementation evidence. There is no expression/event evaluator,
JSON runner, save model, observation ledger, or native/WASM comparison in the
scaffold delivered by this slice.

## Protocol for a later model and comparison

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
   prefixes, compare full state, and continue the remaining events on original
   and restored sessions. Treat the save's numeric text as authoritative rather
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
execution receipts. This verifies the stated Lean primitives only.

A later comparison gate must run its manifest without silent skips, preserve
failing inputs, and detect semantic mutations for the reasons specified above.
These gates supplement existing Rust, kit, and affected WebAssembly/browser
checks. This contract is not authorization to publish a release.
