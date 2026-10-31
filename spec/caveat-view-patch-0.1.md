# CAVEAT View Patch 0.1

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.5, implemented 2026-10-31. It adds an option and a runtime call; [View 0.1](caveat-view-0.1.md) and every other call are unchanged.

A host that redraws from the whole [View 0.1](caveat-view-0.1.md) after every
event pays to build, serialize and parse all of it, although most of it did
not change. The view patch carries only what changed, and the developer kit
applies it to the view the host already holds. Every part the event left
alone is the same object as before, so the host gets the whole view at the
cost of the change. The patch is the transport; what the host reads is the
reused view.

## The reused view

`session.dispatchView(event, payload, { reuse: true })` in the developer kit
returns the same outcome as `session.dispatchView(event, payload)`: the same
JSON text, byte for byte, for every accepted event and every refusal. Three
things differ.

- The view is read-only. Every object and array in it is frozen, so writing
  to it throws in strict code and does nothing otherwise. A host that edits
  the view it is given uses `dispatchView` without `reuse`.
- Each accepted event gives a new view object. A member, a map entry or a
  list entry the event left alone is the same object as in the view the last
  `reuse` call returned; one that changed is a new object. A host can compare
  parts by identity to find what to redraw.
- The view is built from the patch in the kit, not parsed whole.

`reuse` cannot be combined with `fields`; either is checked before the event
is applied, and the session is then unchanged. `agent` works as with
`dispatchView`. Any other call that can change the session, such as
`dispatch`, `dispatchView` without `reuse`, `dispatchViewDelta` or
`drainArchive`, ends the reuse even when it is refused or changes nothing:
the next `reuse` call builds the whole view again, with every part a new
object, and from then on reuses parts as before.

## API

```text
ReactiveSession::dispatch_view_patch_outcome_json_from(base, held, event, payload, agent)
    -> DispatchViewPatchOutcome
WebReactiveSession.dispatch_view_patch_outcome(event, payload, held) -> compact JSON
WebReactiveSession.dispatch_view_patch_outcome_as(event, payload, held, agent) -> compact JSON
```

The transaction, its validation, its refusals and its fatal errors are those
of `dispatch_view_outcome`. A refusal is the same returned value. An accepted
event returns `{"schema":"caveat-dispatch/0.1","outcome":"accepted","patch":…}`.

`held` is the `sequence` of the view the host holds from the last patch call
on this session, or none. The runtime keeps the view as of its last patch
call, and forgets it when anything else changes the session. When it still
keeps the view as of `held`, the patch applies to that view; otherwise it
applies to nothing and gives every member.

## The patch

```json
{
  "schema": "caveat-reactive-view-patch/0.1",
  "since": 47,
  "view_schema": "caveat-reactive-view/0.1",
  "sequence": 48,
  "last_event": "withdraw",
  "members": { "bindings": { "set": { "api": { "set": { "audit_pass": 1.0 } } } } }
}
```

`since` is the sequence of the view the patch applies to, or null when it
applies to nothing. `schema` of the resulting view is `view_schema`, and its
`sequence` and `last_event` are the patch's.

`members` names only the members that changed. A member it leaves out is the
held view's member unchanged; a patch with null `since` names every member.
Each member changes in one of three ways.

| Member | Change |
| --- | --- |
| `cues`, `effects` | `{"value": …}`: the new member, whole |
| `bindings`, `binding_explanations` | a map change whose entries are map changes |
| `commitments`, `commitment_grounds`, `decision_series` | a map change |
| `decision_journal`, `relations` | a list change |

A map change is `{"keys": […], "set": {…}}`. `keys`, when present, is every
key of the new map in the view's order; when it is absent the keys and their
order are the held map's. `set` gives, by key, each entry that is new or
changed: the new value, or for `bindings` and `binding_explanations` a map
change for that target, which gives `keys` whenever the held view has no
entry for the target. Every other key keeps the held map's entry.
`commitments` is a list in the view, ordered by `action`; its map change is
keyed by `action`, and the new list is the entries in `keys` order.

A list change is `{"items": […]}`: the new list, in order, where an array
`[start, length]` is that run of the held list's entries and an object is a
new entry.

Applying the patch to the view as of `since` gives exactly the view
`dispatch_view_outcome` returns for the same event, so the same JSON text.
`kit/test/dispatch-view-reuse.test.mjs` checks this after every event of a
seeded random stream on every tracked program, and on the recorded Glowcap,
ledger and Trail Rescue streams, with other calls between.

## Cost

On the release-gate workload in WebAssembly (`bench/carmack/profile.mjs`, one
local run on 2026-10-31), the median event took 261 µs with the whole view
built fresh, 168 µs with it reused, and 110 µs with no view: the view's share
fell from about 150 µs to about 58 µs. On the ledger workload it fell from
431 µs to 237 µs, against 179 µs with no view. The patch averages 1.4 KB per
event where the whole view averages 13 KB, and the kit parses and applies it
in about 13 µs. Most of what is left is the runtime comparing each member
with the view it holds; it falls as the rules leave unchanged values
untouched.

## Changes

- 2026-10-31: first version.
