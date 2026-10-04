# CAVEAT Dispatch Outcomes 0.1

This is the outcome contract for the reactive developer kit and its scenario
runner. It adds a structured API to the existing interpreter. The existing
`dispatch`, `dispatch_view`, native `dispatch_json` and `apply` keep their
signatures, behavior and diagnostic text.

## Entry points

```rust
let outcome = session.dispatch_outcome_json("absorb", "{}");
// Result<DispatchOutcome, DispatchFatal>
```

```js
const outcome = JSON.parse(session.dispatch_outcome("absorb", "{}"));
// WebReactiveSession; returns JSON for accepted/rejected, throws on fatal.
```

For a host that redraws from the view, the view path returns the same outcome
with the view in place of the snapshot (see [The view path](#the-view-path)):

```rust
let outcome = session.dispatch_view_outcome_json("absorb", "{}");
// Result<DispatchViewOutcome, DispatchFatal>
```

```js
const outcome = JSON.parse(session.dispatch_view_outcome("absorb", "{}"));
```

All of them use the same payload resolver, rules and atomic transaction as the
legacy API. They do not replay events, pre-execute a second interpreter, or
classify failure messages by matching text. Normal input classification happens
in the core before rules run; evaluation classification happens at the failing
effect.

## Wire outcomes

An accepted result contains the complete runtime snapshot after the event:

```json
{"schema":"caveat-dispatch/0.1","outcome":"accepted","snapshot":{}}
```

The empty snapshot above is an abbreviated placeholder. Actual output is the
same full snapshot as a successful legacy `dispatch`.

A classified refusal is a returned value:

```json
{"schema":"caveat-dispatch/0.1","outcome":"rejected","origin":"policy","code":"reject","message":"already absorbed"}
```

An unclassified core error is `Err(DispatchFatal)` in Rust. The WASM bridge
throws its serialized fatal report:

```json
{"schema":"caveat-dispatch/0.1","outcome":"fatal","code":"unclassified","message":"diagnostic text"}
```

Every escaped exception is fatal, whether or not it contains a serialized
report. This includes a WASM trap, a host bug, and a returned-JSON parsing or
output failure. A host must fail the operation and discard the session after
a fatal outcome or exception. It must not call snapshot/save to try to prove
that a trapped session is still usable. Panics are not caught and converted
into ordinary rejections by the interpreter.

## The view path

`dispatch_view_outcome` (natively `dispatch_view_outcome_json`; in the
developer kit `session.dispatchView(event, payload)`) is `dispatch_outcome`
with one difference: an accepted result carries the view after the event
([View 0.1](caveat-view-0.1.md)), exactly as `view()` then returns it, in
place of the snapshot, which it does not build:

```json
{"schema":"caveat-dispatch/0.1","outcome":"accepted","view":{}}
```

The empty view above is an abbreviated placeholder. A refusal is the same
returned value as `dispatch_outcome`'s, byte for byte, and a fatal error the
same thrown report: both entry points run one transaction and take refusals
and fatal errors from one classification. Every rule of this contract applies
to it unchanged, atomicity included. After the same events, a session driven
through either entry point has the same save, snapshot and view.

It is an additional entry point. `dispatch_outcome`, `dispatch`, the legacy
`dispatch_view`, which still throws on a refusal, and their outputs are
unchanged, and so is the kit's `dispatch()`. The schema is unchanged too: an
accepted outcome carries `snapshot` from `dispatch_outcome` and `view` from
`dispatch_view_outcome`, so each caller receives the shape it asked for. A
consumer that validates outcomes must require the field of the entry point it
called.

The schema, outcome, origin and code are machine-readable contract fields.
`message` is human text and is not stable, except that a `policy/reject` message
is exactly the program's quoted literal. Rule and procedure prefixes remain in
legacy diagnostics; they are not part of the structured policy message.
The runtime emits no `stage` field. Host transport failures may identify their
I/O stage; a stage must never be used to guess a runtime rejection origin.

## Origins and the initial code catalog

| Origin | Code | Classified failure site |
| --- | --- | --- |
| `policy` | `reject` | An executed source `reject` effect, including inside a procedure. |
| `policy` | `not_permitted` | A commit's `permitted by` clause found its grant missing, withdrawn, or for another value ([Permission 0.1](caveat-permission-0.1.md)). |
| `input` | `unknown_event` | A well-formed payload reaches admission for an undeclared event. |
| `input` | `payload_invalid` | Invalid payload JSON/type, duplicate fields, wrong parameter set, or invalid named parameter member/type, including a fraction for a typed parameter inside its range. |
| `input` | `bound_exceeded` | A supplied numeric event parameter fails its declared finite range. |
| `evaluation` | `bound_exceeded` | An executed state assignment fails the state's finite range, or a clock event would make the session clock nonfinite ([Elapsed 0.1](caveat-elapsed-0.1.md)). |
| `evaluation` | `decision_in_force` | An executed `commit` would revise a decision series whose current revision is still in force: committed and not explicitly reopened ([Reactive 0.5](caveat-reactive-0.5.md)). |
| `evaluation` | `ungrounded_citation` | A `because` citation cites evidence or a caveat that what it explains never read: a shown binding's value and conditions ([Explanations 0.1](caveat-explanations-0.1.md)), or an executed `set`'s new value and guard ([Explanations 0.2](caveat-explanations-0.2.md)). |
| `evaluation` | `empty_caveated_selection` | An executed `reopen ACTION because caveated(STATE, CAVEAT)` finds no observed evidence in the state's grounds currently carrying that caveat ([State Caveats 0.1](caveat-state-caveats-0.1.md)). |
| `evaluation` | `unobserved_evidence` | An expression evaluated by the event reads `qualified(VALUE, EVIDENCE)` with evidence that is not observed; an executed `qualify EVIDENCE with CAVEAT` (immediate or `after`), `withdraw TARGET because REASON` or `reopen ACTION because EVIDENCE` names evidence that is not observed, or a `withdraw latest(STREAM)` or `reopen ACTION because latest(STREAM)` finds no reading ([Late Qualification 0.1](caveat-late-qualification-0.1.md), [Withdrawal 0.1](caveat-withdrawal-0.1.md), [Reactive 0.1](caveat-reactive-0.1.md)). |
| `evaluation` | `not_committed` | An executed `reopen ACTION because …` names a decision series that has no commitment to reopen ([Reactive 0.1](caveat-reactive-0.1.md)). |
| `evaluation` | `expression` | An expression evaluated by the event, in a rule guard, an effect, a procedure argument or an active binding, divides by zero, produces a nonfinite number, takes the square root of a negative number, reads a history index that is not an integer in `0..256` or is past the history's records, or reads `latest` of a reading stream with no reading or a decision series with no commitment, or reads a decision committed without a numeric `using` value ([Reactive 0.2](caveat-reactive-0.2.md)), or reads `id_text` of a number other than 0 that is not the handle of an identifier the session holds ([Identifiers 0.1](caveat-identifiers-0.1.md)). |
| `evaluation` | `requirement_failed` | An expression evaluated by the event reaches `require(CONDITION, VALUE)` with a false condition, including the prelude's `clamp` with reversed bounds and `wrap` with a period that is not positive. |
| `limit` | `attention_limit` | An executed `examine` would spend more than the declared attention budget currently has remaining ([Reactive 0.2](caveat-reactive-0.2.md)). |
| `limit` | `work_limit` | The event's execution-step budget is exhausted, including skipped procedure effects. |
| `limit` | `depth_limit` | The defensive procedure execution depth guard is reached. |
| `limit` | `history_limit` | A `sample` or `commit` would add a record to a reading stream or decision series that already holds its declared limit. |
| `limit` | `identifier_limit` | The event would add an identifier past the program's `identifiers` limit or 1 MiB of identifier text ([Identifiers 0.1](caveat-identifiers-0.1.md)). |
| `limit` | `renewal_limit` | A `renew` would give a renewable evidence more occurrences than its declared limit ([Renewal 0.1](caveat-renewal-0.1.md)). |
| `limit` | `scheduled_limit` | A `qualify … after` would schedule a qualification while 4,096 are already waiting ([Late Qualification 0.1](caveat-late-qualification-0.1.md)). |

Payload bounds are `input`; state bounds are `evaluation`. This choice depends
on where the failure occurs, not on the wording of the shared range diagnostic.
Payload parsing precedes event admission, preserving legacy failure precedence:
an unknown event with malformed JSON reports `payload_invalid` first.
Source loading already rejects excessively deep procedure chains; `depth_limit`
also classifies the runtime's defensive guard if it is reached.

`host` is reserved for explicitly classified refusals made by a future host
library before core execution. The core never emits it. An arbitrary caught
exception must not be labelled `host` and accepted as an ordinary rejection.

This catalog deliberately classifies specific sites only. Every other existing
string error is fatal `unclassified`, even if its message contains
`rejected: `. For example, a type error in an expression is not a classified
rejection. Expression failures are classified where they occur, by kind,
never by their message. New callers must not downgrade these errors to pass an
expected-rejection assertion. Their legacy behavior remains unchanged.

## Atomicity and scenario interpretation

Every `rejected` outcome preserves the complete save, view and snapshot,
including sequence, clock, pending qualifications, state, graph, effects and
decision history. Successful work before the refusal does not survive.
The scenario runner must check this for every rejected dispatch.

A bare expected rejection matches only `origin: policy`. Other origins must
be named explicitly. Assertions may also require a code; only policy rejection
messages may be matched as stable authored output. In particular, a host input
check cannot satisfy an assertion that a Caveat rule rejected an event.
Fatal outcomes and exceptions always fail the scenario. Missing/unknown schema,
outcome or origin fields also fail closed; consumers must never interpret an
unrecognized report as a successful rejection.

These rules are a dependency of the scenario format. They do not change or
retroactively rescore frozen experiment harnesses. Supplementary audits must
record the runtime/source versions they actually examine.

## Recovering from attention exhaustion and empty witness selection

An insufficient examination budget returns `limit/attention_limit`, and an
empty `caveated` reopening selection returns
`evaluation/empty_caveated_selection`. Both leave the session usable: the host
can read it, save and restore it, or dispatch a different event. Every change
from the refused event rolls back, including earlier examination costs,
scheduled qualifications, occurrence IDs and decision journal entries.

A true `has_caveat(STATE, CAVEAT)` does not guarantee a reopening witness: an
authored extra caveat can qualify the state's value without qualifying any of
its evidence. The refusal neither invents evidence nor reopens a decision.
The program can supply a legitimate witness in a later event or choose another
authored course. Budget exhaustion similarly grants no additional attention.

These two sites were fatal `unclassified` in rc.9. This classification adds
recoverable outcomes without changing their legacy diagnostic text, rollback
or successful effects. rc.11 classified further sites (next section);
uncatalogued failures remain fatal on outcome entry points.

## Recovering from unobserved evidence, expression failures and the scheduling limit

`evaluation/unobserved_evidence`, `evaluation/not_committed`,
`evaluation/expression`, `evaluation/requirement_failed` and
`limit/scheduled_limit` leave the session
usable in the same way: the host can read it, save and restore it, or dispatch
a different event, and every change from the refused event rolls back,
including earlier effects, scheduled qualifications, occurrence IDs, cues and
decision journal entries.

The refusal invents nothing. It does not observe the evidence, commit the
decision, treat a failed
requirement as met, substitute a value for a failed expression, or make room in
the scheduled table. The program can observe the evidence in a later event,
supply other input, or wait for a scheduled qualification to apply.

These sites were fatal `unclassified` in rc.10. Their rollback, successful
effects and diagnostic text are unchanged, except that an unobserved
`reopen ACTION because EVIDENCE` now reports
`cannot reopen ACTION because unobserved evidence EVIDENCE` in place of the
`qualified` diagnostic it borrowed.

`id_text` of a number that is no identifier's handle joined
`evaluation/expression` in rc.13, with its diagnostic text
(`id_text: N is not the handle of an identifier`) unchanged. It was the last
surveyed native-function failure that stayed fatal. The refusal does not
create an identifier for the number or read it as the empty text; `id_text(0)`
is still the empty text. An `id` parameter sent as anything but 1 to 1,024
bytes of text is still `input/payload_invalid`.

## Versioning

The schema versions the outcome contract, independently of save schemas and
package versions. Code meanings and origins in this catalog are stable for
`caveat-dispatch/0.1`. Adding a documented code for a previously unclassified
site is compatible, with its regression tests and release note. Reassigning an
existing code's meaning/origin, removing a code, or changing the wire structure
incompatibly requires a new outcome schema version.

Consumers must handle future codes without treating them as policy rejections
or assuming success: preserve the code/origin in diagnostics, and fail any
assertion whose required classification is not met. An unknown origin/schema
is a protocol failure. No promise of cross-runtime save compatibility follows
from an outcome schema match; the save contract governs restoration.

## Changes

- `evaluation/unobserved_evidence`, `evaluation/not_committed`,
  `evaluation/expression`, `evaluation/requirement_failed` and
  `limit/scheduled_limit` classify acting on or citing unobserved evidence, reopening a
  decision that was never committed, a failed arithmetic or history read
  (including a negative square root, `latest` of an empty history and a
  decision read with no numeric value), a
  false requirement and a full scheduled-qualification table. Before them, each
  such event was fatal `unclassified`, although the runtime had already
  rolled it back ([recovery](#recovering-from-unobserved-evidence-expression-failures-and-the-scheduling-limit)).
- `evaluation/expression` also classifies `id_text` of a number other than 0
  that is not the handle of an identifier the session holds, in a guard, an
  effect, a procedure argument or a binding (rc.13; Version Lab F309-F312,
  F338-F339). Before it, such an event was fatal `unclassified`, although the
  runtime had already rolled it back. The rollback and the diagnostic text are
  unchanged, and no accepted event changes.
- `limit/history_limit` classifies a `sample` or `commit` on a full reading
  stream or decision series. Before it, such an event was fatal
  `unclassified`: a host lost the session, although the runtime had already
  rolled the event back. The rollback and the diagnostic text are unchanged.
  Both fresh authors in the v3 authoring trial guarded their programs
  against it by hand.
- `limit/identifier_limit` classifies an event that would hold more
  identifiers than the program declares, or more than 1 MiB of their text.
  It arrived with [Identifiers 0.1](caveat-identifiers-0.1.md), so no earlier
  behavior changes. An identifier payload that is not 1 to 1,024 bytes of text
  is `input/payload_invalid`, like any other malformed payload.
- `policy/not_permitted` classifies a commit whose `permitted by` clause
  finds no usable grant. It arrived with
  [Permission 0.1](caveat-permission-0.1.md), so no earlier behavior changes.
  Its origin is `policy` because the program's own clause states the
  requirement, so a bare expected rejection matches it. Its message is
  runtime text, not a literal from the source.
- `input/payload_invalid` also classifies a fraction sent for a typed
  parameter (`in` or `kind`) inside its range, which names no member. Such a
  payload used to be accepted. An isolated value outside the range is still
  `input/bound_exceeded`. See
  [Typed parameters 0.1](caveat-typed-parameters-0.1.md#changes).
- `limit/renewal_limit` classifies a `renew` of a renewable evidence that
  already has its declared number of occurrences. Before it, such an event was
  fatal `unclassified`: a host lost the session, and with it every other
  subject the program tracked, although the runtime had already rolled the
  event back. The agent ledger keeps one renewable push per pull request, so
  one pull request's 65th push ended the session for all of them. The
  rollback and the diagnostic text are unchanged. It has its own code, not
  `history_limit`, because the exhausted resource is a renewable evidence's
  occurrences, not a reading stream or decision series.
- `evaluation/decision_in_force` classifies a `commit` to a decision series
  whose current revision is still in force, which Reactive 0.5 requires to be
  explicitly reopened first. Before it, such an event was fatal
  `unclassified`: a host lost the session, although the runtime had already
  rolled the event back. Run A2 of authoring trial v1 left this refusal to
  the runtime, so under this contract its second `decide` ended the session.
  The rollback and the diagnostic text are unchanged. Its origin is
  `evaluation`, as for a state's range: the commit ran, and the series' state
  refused it. It is not `policy`, because the program wrote no clause for it,
  so a bare expected rejection does not match it. It is not a `limit`,
  because the series may have room; a full series is still
  `limit/history_limit`. The check comes before a `permitted by` clause's, so
  it is never reported as `not_permitted`.
- `evaluation/ungrounded_citation` classifies an event on which a `because`
  citation, on a binding or on a `set`, cites evidence or a caveat that what
  it explains never read. Explanations 0.1 already rejected such an event
  atomically, but as fatal `unclassified`: a host lost the session, although
  the runtime had already rolled the event back. Whether a citation holds
  depends on the session, not only on the program's text, so a program cannot
  be refused for it when it loads without refusing programs whose citations
  hold on every path they take. A late caveat can make a declaration win for
  the first time on a clock tick: the program the save tests play lost its
  session that way 40 seconds in, with no save involved. It now keeps the
  session, and each tick is refused and advances no time until a renewal
  replaces the bite the caveat is due on. A renewal changes which occurrence
  `qualified(1, bite)` names, a cited state can change apart from the value
  derived from it, and `or` does not read its second operand when the first
  is true. The rollback and the diagnostic text are unchanged.
  Its origin is `evaluation`, as for a state's range: the explanation was
  evaluated and did not hold. It is not `policy`, because no clause of the
  program refuses the event, so a bare expected rejection does not match it.
- 2026-09-29: `dispatch_view_outcome`, natively
  `dispatch_view_outcome_json` and in the developer kit
  `session.dispatchView`, is the outcome contract for a host that redraws
  from the view ([The view path](#the-view-path)). Before it, such a host
  had two ways, each with a cost: the kit's `dispatch()` builds, serializes
  and parses the full snapshot and the host then asks for the view, and the
  legacy `dispatch_view` throws on a classified refusal instead of returning
  it. [Performance Baseline 0.1](../experiments/performance-0.1/RESULTS.md)
  found that snapshot the largest measured cost a host wanting the view does
  not use. An accepted outcome now carries the view, and the snapshot is not
  built; refusals and fatal errors are `dispatch_outcome`'s. No existing entry
  point, output or code changes.
- 2026-10-03: `evaluation/bound_exceeded` also classifies a clock event whose
  `dt` would make the session clock nonfinite, which
  [Elapsed 0.1](caveat-elapsed-0.1.md) promised to reject but the runtime did
  not check. Declared bounds and restore keep the clock finite, so no
  supported path reaches it, and no existing outcome changes.
