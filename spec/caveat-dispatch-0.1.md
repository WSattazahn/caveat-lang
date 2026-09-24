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

Both use the same payload resolver, rules and atomic transaction as the legacy
API. They do not replay events, pre-execute a second interpreter, or classify
failure messages by matching text. Normal input classification happens in the
core before rules run; evaluation classification happens at the failing effect.

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
| `input` | `unknown_event` | A well-formed payload reaches admission for an undeclared event. |
| `input` | `payload_invalid` | Invalid payload JSON/type, duplicate fields, wrong parameter set, or invalid named parameter member/type. |
| `input` | `bound_exceeded` | A supplied numeric event parameter fails its declared finite range. |
| `evaluation` | `bound_exceeded` | An executed state assignment fails the state's finite range. |
| `limit` | `work_limit` | The event's execution-step budget is exhausted, including skipped procedure effects. |
| `limit` | `depth_limit` | The defensive procedure execution depth guard is reached. |
| `limit` | `history_limit` | A `sample` or `commit` would add a record to a reading stream or decision series that already holds its declared limit. |

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
`rejected: `. For example, an expression `require(false, ...)`, arithmetic
failure or unavailable history item is not yet a classified rejection. New
callers must not downgrade these errors to pass an expected-rejection
assertion. Their legacy behavior remains unchanged.

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

- `limit/history_limit` classifies a `sample` or `commit` on a full reading
  stream or decision series. Before it, such an event was fatal
  `unclassified`: a host lost the session, although the runtime had already
  rolled the event back. The rollback and the diagnostic text are unchanged.
  Both fresh authors in the v3 authoring trial guarded their programs
  against it by hand.
