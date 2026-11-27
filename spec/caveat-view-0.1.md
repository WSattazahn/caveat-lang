# CAVEAT View 0.1

**Status:** stable from 1.0.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); until then a prerelease may still change it.

rc.16 amendment: provenance in View 0.1 may carry `departed` markers and the
`inherited` split specified by [Lineage Compaction 0.1](caveat-lineage-compaction-0.1.md).
Current retained snapshots can omit eligible departed records under
[Departure 0.1](caveat-departure-0.1.md) and the
[withdrawal collection](caveat-withdrawal-collection-0.1.md); archive-backed
history is separate. These changes do not implement View 0.2 or a delta view;
[View 0.2](caveat-view-0.2.md) specifies them, and leaves View 0.1 unchanged.
The [rc.16 scope amendment](../docs/releases/rc16-development.md#scope-amendment-for-release-preparation-2026-10-08)
explicitly defers that API.

`WebReactiveSession.dispatch` returns the full snapshot as pretty-printed JSON
after every event. That includes the static world, every symbol, every
event signature and the complete lineage of every value. On the glowcap beat
the snapshot is 24 KB, and building and parsing it was about half the cost of
an event.

A host redraws only what an event can change. The **view** is that part:

| Field | Meaning |
| --- | --- |
| `schema` | `"caveat-reactive-view/0.1"` |
| `sequence`, `last_event` | as in the snapshot |
| `bindings`, `binding_explanations` | what to show, and what each shown value cites |
| `cues`, `effects` | what the last accepted event emitted and did |
| `commitments`, `commitment_grounds`, `decision_series` | decisions, their grounds and revisions. A commitment's `open: true` means it was reopened and awaits a new decision, not that it is active |
| `decision_journal` | ordered, append-only commitment and reopening entries, with the reasons recorded at each change ([journal](caveat-decision-journal-0.1.md)) |
| `relations` | live graph relations, in insertion order |

As in the snapshot, `origin: "live"` means current graph membership, including
restored edges; see the [restore trust boundary](caveat-save-0.1.md#host-trust-boundary).

## API

```text
ReactiveSession::dispatch_view_json(event, payload) -> ReactiveView
ReactiveSession::dispatch_view_outcome_json(event, payload) -> DispatchViewOutcome
ReactiveSession::dispatch_view_fields_outcome_json(event, payload, fields) -> DispatchViewOutcome
ReactiveSession::view() -> ReactiveView
WebReactiveSession.dispatch_view(event, payload) -> compact JSON
WebReactiveSession.dispatch_view_outcome(event, payload) -> compact JSON
WebReactiveSession.dispatch_view_fields_outcome(event, payload, fields) -> compact JSON
WebReactiveSession.view() -> compact JSON
```

`dispatch_view` runs the same transaction as `dispatch`, with the same
validation, atomicity and failure behaviour, and returns the view instead of
the snapshot. The view's fields are equal to the same fields of the snapshot
after the same events. `dispatch`, `snapshot` and the snapshot schema are
unchanged. A host reads static declarations, such as the world's entities,
once from `snapshot()`.

On the glowcap beat a view is 2.9 KB, and dispatch plus parse takes about half
as long as with the snapshot. The remaining cost is the transaction itself:
the session is copied for rollback, and every binding is re-evaluated after
each event.

`dispatch_view_outcome`, `session.dispatchView` in the developer kit, returns
the view inside a dispatch outcome: `{"outcome":"accepted","view":…}` for an
accepted event, where the view is exactly what `view()` then returns, and a
classified refusal as the returned value `dispatch_outcome` gives, where
`dispatch_view` throws. See
[Dispatch 0.1](caveat-dispatch-0.1.md#the-view-path).

### Selected fields

`dispatch_view_fields_outcome`, `session.dispatchView(event, payload, { fields })`
in the developer kit, is `dispatch_view_outcome` with only the named fields.
`fields` lists names from `bindings`, `binding_explanations`, `cues`,
`effects`, `commitments`, `commitment_grounds`, `decision_series`,
`decision_journal` and `relations`. The view keeps `schema`, `sequence` and
`last_event`, then each named field exactly as the full view has it, in the
full view's order. The runtime does not build the fields left out. An
unknown or repeated name is an error before the event is applied, and the
session is unchanged. The transaction, the refusals and the session after the
event are those of `dispatch_view_outcome`.

`decisionHistory` reads `decision_series`, `decision_journal` and
`commitments`, so a view meant for it names all three.

### Reused view

`session.dispatchView(event, payload, { reuse: true })` in the developer kit
returns the same view, read-only, built from a patch of what the event
changed, with every part it left alone the same object as before. See
[View Patch 0.1](caveat-view-patch-0.1.md).

## Changes

- 2026-10-31: the reused view, built from the [view patch](caveat-view-patch-0.1.md).
  The full view and every other call are unchanged.

- 2026-10-19: `dispatch_view_fields_outcome` returns a view with only the
  fields a host names. A host that reads a few fields no longer pays to build
  and parse the rest. The full view and every other call are unchanged.

- 2026-09-29: `dispatch_view_outcome` returns the view as a dispatch outcome
  ([Dispatch 0.1](caveat-dispatch-0.1.md#the-view-path)). A host that wants
  the view and the refusal as a value no longer has to take the snapshot as
  well. `dispatch_view`, `view` and the view's fields are unchanged.
