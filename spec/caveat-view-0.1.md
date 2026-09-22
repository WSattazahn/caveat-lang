# CAVEAT View 0.1

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
| `commitments`, `commitment_grounds`, `decision_series` | decisions, their grounds and revisions |
| `relations` | live graph relations, in insertion order |

## API

```text
ReactiveSession::dispatch_view_json(event, payload) -> ReactiveView
ReactiveSession::view() -> ReactiveView
WebReactiveSession.dispatch_view(event, payload) -> compact JSON
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
