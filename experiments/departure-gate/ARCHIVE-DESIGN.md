# Minimal archive provenance amendment (implemented for review, 2026-10-07)

Owner decision: retain both a bounded live save and exact reconstruction from a
complete matching host-drained archive. No language syntax or event replay engine.

## Reproduction

`runtime/tests/departure.rs` has a native fixture with `left` reading records
{s@1,s@2,s@4} and `right` reading {s@1,s@3,s@4}. Their original lower-bound markers are identical.
Without transfer roots, copying either into `chosen` after departure produced
the same save and archive; the regression now requires distinct roots and
correct, different reconstructed memberships.
A `reused` state also illustrates false exactness: its old holder list has three
records, so comparing holder count with marker.read can return the wrong subset
following replacement with the other value.

## Data

Each marker gains optional `archive_ref`, a 64-character lowercase SHA-256 digest.
Missing refs are legacy/unknown membership and cannot establish exactness.
A provenance copy keeps this one root. Replacement replaces it. A merge joins two
roots, deduplicating equal roots, while retaining the existing lower-bound marker.
Mixing a known root with an unknown marker yields an unknown root.

The existing departed-record archive items keep their shape. The archive additionally
hands over these items (the mixed stream is `ArchiveItem` in Rust):

- `{kind:"provenance", id, history, operation:"record", source_id, record, departed_at}`
- `{kind:"provenance", id, history, operation:"union", parents:[sorted unique IDs]}`

Canonical hash inputs are UTF-8 JSON arrays with no extra whitespace:

- `["caveat-archive-provenance/0.1","record",source_id,history,record,departed_at]`
- `["caveat-archive-provenance/0.1","union",history,parents]`

A leaf references a departed record by source-scoped identity, not authenticated
payload. The host verifies record/history/ordinal/departed_at against the matching
record archive item. Every recursive leaf must have the requested source_id.
Unions have at least two distinct sorted parents; singleton joins reuse the root.
A complete validated reachable DAG yields a deduplicated exact set of record names.
Missing nodes or records, conflicts, invalid hashes, cycles, or inconsistent scope
remain conservative. Hash checking does not authenticate the supplied evidence.

## Runtime lifecycle

Pure expression operations build temporary immutable DAG nodes. At the accepted
event boundary, after bindings have evaluated successfully, changed saved holders,
cue qualifications, projected binding qualifications/explanations and new archive
records export their reachable nodes once per event. All live roots then drop their
DAG children and retain only the fixed-size digest. Historical transfer metadata lives in the growing undrained queue until the
host drains it into external storage. Live provenance holders retain only roots;
the queue is excluded from saves but still consumes memory before draining.

Transaction clones stage nodes with ordinary values. Refusal or binding failure
publishes no nodes or departures. Copying an existing opaque root emits nothing.
Restore retains saved roots without reading an archive. Binding recomputation may
create pure deterministic union nodes, already emitted by the original session.
Restore settles them without publishing; its archive stays empty.
Repeated snapshot/view/save queries only read the settled roots and never allocate
IDs or append history. Source loading without departure emits no archive fields.

## Verification

Tests cover the ambiguous source fixture, post-departure copies, replacements,
merges and overlap; complete/missing/corrupt archive; save/restore continuation;
failed events and binding failures; and bounds on live root representation after
long play and drain. Existing outcome/digest gates remain necessary. C3 dispatch
and adapter/raw save bytes are measured again after implementation; earlier results
are a baseline, not evidence for the new metadata path.

## Separate retention limit

This amendment bounds transfer metadata per live marker. It does not bound
the number of genuinely pinned records. Withdrawal cycles and chains can keep
retired records alive under the current pin rules. The owner authorized a
[separate reachability proposal](WITHDRAWAL-REACHABILITY-DESIGN.md) and regression
fixtures for review, not a runtime semantics change. C3 is one registered
workload gate, not a universal save-size proof.
