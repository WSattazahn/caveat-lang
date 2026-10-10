# CAVEAT View 0.2

Status: specified for rc.17 (Path to 1.0, A4). The implementation and its
delta gate are A5; until A5 merges, no runtime or kit function returns this
schema. [View 0.1](caveat-view-0.1.md) and its bytes do not change.

This specification adopts the [View 0.2 design note](../docs/design/view-0.2.md)
with the owner's decisions recorded in [Path to 1.0](../docs/releases/path-to-1.0.md):
caveats stay sorted and only evidence moves to first-observed order
(2026-10-10), and the delta view is gated on the whole corpus
(2026-10-05).

View 0.2 changes three things for a host that asks for it:

1. a decision's grounds list their evidence in the order it was first
   observed, frozen when the decision was made;
2. a commitment says `reopened`, not `open`;
3. a **delta** carries only what an accepted event changed, so a host can keep
   its view current without receiving or comparing the whole view.

A host that does not ask for 0.2 keeps getting 0.1.

## The full view

Schema `caveat-reactive-view/0.2`. Its fields are View 0.1's, with the same
meaning, except as this table says:

| Field | In 0.2 |
| --- | --- |
| `schema` | `"caveat-reactive-view/0.2"` |
| `sequence`, `last_event`, `bindings`, `binding_explanations`, `cues`, `effects`, `decision_series`, `decision_journal`, `relations` | as in 0.1 |
| `commitments` | as in 0.1, sorted by `action`, except that each has `reopened` in place of `open`, with the same value |
| `commitment_grounds` | as in 0.1, except the order of each grounds' `evidence` (below) |

### Ordered evidence in grounds

In `commitment_grounds[name].evidence`, evidence is listed in the order it was
first observed: the order the commitment's `committed` journal entry gives in
`because` ([Decision Journal 0.1](caveat-decision-journal-0.1.md)). The order is
fixed when the commitment is made and never changes afterwards. A later
observation, withdrawal, departure of other records or collection does not
reorder it.

- The membership is the same as 0.1's. Only the order differs.
- `caveats` stay sorted by name, as in 0.1 and in journal entries. A caveat
  has no observation of its own to order it by.
- When a record named in the grounds departs, its name leaves `evidence` and
  a `departed` marker stands for it ([Lineage Compaction 0.1](caveat-lineage-compaction-0.1.md)).
  The remaining names keep their order. `departed` and `inherited` are as in
  0.1.
- `binding_explanations` are unchanged from 0.1: their evidence stays
  sorted. Their order was not part of this decision.

The order outlives the journal entry it came from: when a `journal window`
retires and departs that entry, the grounds keep the order. A save therefore
records the order wherever it differs from name order, as an additive field
of `caveat-reactive-save/0.1` that A5 defines. A save made before that field
restores with the order its journal entry gives, when the entry is retained,
and otherwise with name order; it never invents an observation order.

### `reopened`

`reopened: true` means the commitment was reopened and awaits a new
decision, which is what 0.1's `open: true` meant. It is the word the source
uses (`reopened(name)`). `open` does not appear in 0.2.

## The delta

Schema `caveat-reactive-view-delta/0.2`. A delta describes how the full 0.2
view changed across one accepted event:

```json
{
  "schema": "caveat-reactive-view-delta/0.2",
  "since": 41,
  "sequence": 42,
  "last_event": "read",
  "bindings": {"set": {"hud": {"text": "17 C"}}, "removed": []},
  "binding_explanations": {"set": {"hud": {"text": {"evidence": ["temperature@3"], "caveats": []}}}, "removed": []},
  "cues": [],
  "effects": [{"kind": "sample", "stream": "temperature", "id": "temperature@3", "value": 17, "relation": "opposes", "target": "warm_enough"}],
  "commitments": {"set": [], "removed": []},
  "commitment_grounds": {"set": {}, "removed": []},
  "decision_series": {"set": {}, "removed": []},
  "decision_journal": {"removed": [], "appended": []},
  "relations": {"removed": [], "appended": [{"from": "temperature@3", "relation": "opposes", "to": "warm_enough", "origin": "live"}]}
}
```

`since` is the sequence of the view the delta applies to, and `sequence` and
`last_event` are the new view's. Every member is always present, empty when
nothing of its kind changed.

Applying a delta `D` to a full view `V` whose `sequence` equals `D.since`
gives the next full view `V'`:

| Member | How `V'` is formed |
| --- | --- |
| `schema` | stays `caveat-reactive-view/0.2` |
| `sequence`, `last_event` | taken from `D` |
| `cues`, `effects` | taken from `D` whole: they belong to the last event only, and are empty when it emitted or did nothing |
| `bindings`, `binding_explanations` | for each `target` and `property` in `set`, the value is replaced or added; each `[target, property]` in `removed` is removed, and a target left with no properties is removed |
| `commitments` | each commitment in `removed` (by `action`) is removed; each in `set` replaces the commitment with the same `action` or is added; the list is then sorted by `action` |
| `commitment_grounds`, `decision_series` | each name in `set` is replaced or added with its whole value; each name in `removed` is removed |
| `decision_journal`, `relations` | the entries at the 0-based positions listed in `removed` (ascending, positions in `V`) are dropped, then `appended` is added at the end, in order |

Sorting by `action` compares the names' UTF-8 bytes, as the full view's order
does. The two lists may only drop entries and append new ones, which is how
the runtime changes them: the journal appends entries and departs its oldest
retired ones; the graph appends relations, and departure or collection drops
some, leaving the rest in order. A relation can occur more than once, so
positions, not values, say which entries go.

The delta for an event is exact: `apply(V, D)` equals the full view `view()`
returns after the event, as JSON values (object members in any order, arrays
in order, numbers equal as numbers, `-0` distinct from `0` as the save keeps
it). A member's `set` names only what changed, but naming an unchanged value
is not an error; a delta that names nothing beyond `since`, `sequence`,
`last_event`, `cues` and `effects` is valid and expected for an event that
changed nothing else.

A changed explanation counts as a change even when the shown value did not
change: an explanation is what a Caveat host shows.

### What produces a delta

Only an accepted event. The API mirrors the view path of
[Dispatch 0.1](caveat-dispatch-0.1.md#the-view-path):

```text
ReactiveSession::view_v2() -> ReactiveView 0.2
ReactiveSession::dispatch_view_delta_outcome_json(event, payload) -> DispatchViewDeltaOutcome
WebReactiveSession.view_v2() -> compact JSON
WebReactiveSession.dispatch_view_delta_outcome(event, payload) -> compact JSON
kit: session.view({ schema: '0.2' }), session.dispatchViewDelta(event, payload)
```

An accepted event returns
`{"schema":"caveat-dispatch/0.1","outcome":"accepted","delta":{…}}`. A refused
event returns the same refusal as `dispatch_outcome`, with no delta, and
changes nothing, so the host's view stays current. A fatal error is reported
as on every other path. The transaction, validation and atomicity are those of
`dispatch`; producing the delta does not change what the event does.

### Keeping a host's view current

A host holds one full 0.2 view and its `sequence`.

- **Start, and after any restore:** take `view({ schema: '0.2' })`. A restore
  starts a different session object, and no delta crosses it.
- **On each accepted delta:** if `D.since` equals the held sequence, apply
  it. Otherwise the host missed a change (for example an event dispatched
  through another path) and takes a full view instead of applying.
- **On a refusal:** nothing changed; keep the held view.
- **Draining the archive** does not change the view
  ([Departure 0.1](caveat-departure-0.1.md)), so it needs no resync.

A delta never authenticates anything; it carries the same data as the full
view, with the same [host trust boundary](caveat-save-0.1.md#host-trust-boundary).

## The delta gate

The delta is accepted only with the check the owner set on 2026-10-05, run
in the runtime workflow (A5):

- For every program the save sweep discovers, minus exclusions that are named
  and counted, over seeded event runs, the full view at `N` with the delta
  applied equals the full view at `N+1`, after every accepted event. The gate
  fails if the population is empty or smaller than the recorded count.
- The applier the gate uses is written independently of the runtime's delta
  builder, from this specification.
- Negative controls corrupt deltas (a dropped `set` entry, a wrong position in
  `removed`, a stale `since`) and the gate must detect each.
- Named cases cover an accepted event, an accepted event that changes nothing
  else, a refusal, a restore, a sequence mismatch, a change to an explanation
  only, a late qualification, a departure, a journal window and a collection.
- Dispatch, view building, delta building, applying, and bytes are measured
  separately.

## What stays the same

`view()`, `dispatch_view` and `dispatch_view_outcome` return
`caveat-reactive-view/0.1`, byte for byte. The snapshot, the save (apart from
the additive order field above) and every refusal code are unchanged.

## Changes

- 2026-10-10: specified (Path to 1.0, A4).
