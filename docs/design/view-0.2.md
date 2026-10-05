# Design note: View 0.2, ordered grounds and a delta view

Status: a design note for the owner's decision (rc.15 plan, PR 7). Nothing
here is implemented or promised, and no specification changes with it. The
owner's card asks whether to implement it in rc.16 or to defer it. Receipts
are `path: Lx–Ly` at `main` `8a822cc`.

## What a host sees today

[View 0.1](../../spec/caveat-view-0.1.md) is the part of the snapshot that an
event can change. A host gets it after every accepted event. Three things
about it cost host authors effort.

1. **Grounds come sorted by name.** A commitment's grounds are a
   `Provenance`, whose `evidence` and `caveats` are `BTreeSet`s
   (`runtime/src/reactive_expr.rs: L104–107`), and the view lends them as
   they are (`runtime/src/reactive.rs: L1152`). The journal, by contrast,
   lists `because` in the order the evidence was first observed
   (`spec/caveat-decision-journal-0.1.md: L31–33`). A host that wants to say
   "decided on A, then B" has to read the journal instead of the grounds,
   or rebuild the order itself.
2. **The whole view is sent every time.** A view is about 2.9 KB on the
   Glowcap beat (`spec/caveat-view-0.1.md: L42–43`). Most events change one
   or two bindings, and a host that re-renders compares the whole view with
   the last one to find what changed. The runtime already works out what an
   event changed, so it can re-evaluate only the expressions that read it
   (`runtime/src/reactive.rs: L951–998`). That work isn't exposed to the
   host.
3. **`open: true` reads as "active".** On a commitment it means the
   commitment was reopened and is waiting for a new decision
   (`runtime/src/map.rs: L133–136`). The View 0.1 table now says so
   (`spec/caveat-view-0.1.md: L17`), but the field name still misleads a
   host engineer who reads the JSON before the table. The owner raised this
   at 03:49 UTC.

## Proposal: `caveat-reactive-view/0.2`

There is one new schema, and the hosts that want it opt in. The 0.1 view and
its bytes stay as they are.

- **Ordered grounds.** In 0.2, `commitment_grounds[name].evidence` lists
  evidence in first-observed order, as the journal's `because` already does.
  `caveats` stay sorted, because a caveat has no observation of its own.
  The membership is the same as 0.1's set; only the order changes.
- **`reopened` instead of `open`.** Each 0.2 commitment has
  `reopened: true|false`, the same word as the source predicate
  `reopened(name)`. The field `open` does not appear in 0.2.
- **A delta view.** `dispatch_view_delta(event, payload)` returns
  `{"schema": "caveat-reactive-view-delta/0.2", "since": N, "sequence": N+1, …}`.
  It holds only what changed since sequence `N`: changed or removed
  bindings and their explanations, commitments whose state changed, the
  journal entries added, relations added or removed, and the event's
  `cues` and `effects` (which belong to that one event anyway). A host
  applies a delta to the last full 0.2 view to get the next one. The
  equality the runtime would test is: last full view plus delta equals
  `view()` at `N+1`.
- **Restore and refusal.** A delta is relative to the sequence it names, so
  after a restore, or when `since` doesn't match the host's last sequence,
  the host asks for a full view. A refused event changes nothing and
  returns a refusal without a delta, as `dispatch_view_outcome` does today
  ([Dispatch 0.1](../../spec/caveat-dispatch-0.1.md#the-view-path)).

## What stays the same

`view()`, `dispatch_view` and `dispatch_view_outcome` keep returning
`caveat-reactive-view/0.1`, byte for byte, until a host asks for 0.2. The
snapshot, the save and every refusal code are unchanged. The kit adds
`session.view({ schema: '0.2' })` and `session.dispatchViewDelta`, and its
types describe both schemas.

## Migration for a host

1. Replace `commitment.open` with `commitment.reopened`. The value is the
   same.
2. If the host shows grounds as a list, drop any sorting it added to match
   the journal.
3. To re-render less, call `dispatchViewDelta` and apply each delta to the
   last full view. If `since` doesn't match, ask for a full view.

A host that does nothing keeps 0.1.

## Costs and open questions

- First-observed order for grounds needs each evidence name's first
  observation sequence. The journal has that order when it records an
  entry. Grounds would need it kept alongside the set, or worked out when
  the view is built.
- The delta must be checked against the full view for every corpus program,
  in the save sweep's style: the full view at `N` plus the delta equals the
  full view at `N+1`, across seeded event runs.
- Whether a delta should also carry `binding_explanations` that changed
  while the value didn't. The proposal says yes, because an explanation is
  what a Caveat host shows.

## Decision asked

Implement View 0.2 in rc.16, or defer it. This note recommends rc.16. Round
8's hosts would be written against it, and the `open` rename is cheapest
before more hosts read 0.1.
