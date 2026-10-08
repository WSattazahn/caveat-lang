# Withdrawal batch extraction

Status: separate implementation and testing authorized on 2026-10-08 UTC.
Experimental baseline: `c1d8fea1164e9d89edc8ff33af5a3bb608bc37d3` (PR #176).
PRs #174, #175 and #176 remain unchanged. No merge, publication, release,
rc.16 readiness or unrestricted interactive-workload approval is authorized.

## Problem and scope

Bulk departure currently searches the authoritative withdrawal vector and removes
one element separately for each departing record. The searches and stable vector
shifts repeat while the event transaction still holds its original session.
Departure also runs before final bindings, so a late rejected event pays these
costs before the prospective session is discarded.

First measure the actual block, including its first copy-on-write detachment,
using a separate diagnostic build. Timing instrumentation is not part of the
executable used to judge the performance target. Report diagnostic overhead and
observed attribution; do not assume this block accounts for every release cost.

The authorized change extracts departures as a batch, preserving exact records
and order. It does not change candidate selection, required pins, reason counts,
root reachability, lineage compaction, archive membership, schema, admission
checks or transaction boundaries. It does not introduce deferred collection,
capacity reclamation, a general undo system, lossy summaries or new syntax.

## Required invariants and consumers

- Registration retains the first withdrawal per subject, including its original
  reason, event and sequence. Repeated withdrawals remain idempotent.
- Retained withdrawals remain in their original occurrence order in snapshots
  and saves. Only legitimately departing subjects lose their withdrawal records.
- Each removed withdrawal is attached to its matching archive entry with its
  complete original payload. Global archive/effect order remains history name
  followed by numeric occurrence, independently of extraction order.
- Subject/reason pin updates happen through the existing authoritative index
  paths. Extraction must not count another removal or weaken retention.
- Full, partial and empty extraction preserve valid restore continuation,
  predicates, qualifications, reasons, permissions and decisions.
- Rejected events restore exact state and retain the earlier undrained archive;
  no record or provenance item from the rejected attempt is published.
- Draining remains independent of live execution. Programs without windows keep
  the same behavior. Existing snapshot/history/capacity choices from #175–176
  remain the experimental contract, with their original limits.

## Measurements and review gate

The experiment protocol freezes repetitions, workload, aggregation, baseline and
build/harness identities before candidate measurement. At 3,000 mutual cycles,
require at least 20% lower successful and rejected release latency under matched
conditions, without increasing either additional requested-heap peak. Include
increasing-size, small-event, high-degree and unrelated-event checks; expose any
regression rather than hiding it in a mostly-growth aggregate.

Use the unchanged native `collector_profile` target harness. Keep diagnostic
cost attribution, target measurements, functional equivalence and integration
gates as separate evidence. Requested Rust heap excludes allocator metadata,
stack and RSS; whole-dispatch additional peak is not isolated extractor memory.

Preserve failures and raw attempts. Return the exact review revision, before/after
absolute measurements, the literal result for each target and the smallest next
recommendation if it misses. Do not redefine success or expand the scope to make
the gate pass. Finite native samples do not establish a browser or tail guarantee.

## Validation

Add focused regressions for interleaved retained/departing withdrawals, numeric
versus lexical record ordering, records without withdrawals, exact reasons,
partial/final release, existing undrained archive, failed release and restore.
Retain existing collector, compaction, archive, real-capacity and compatibility
checks. Reuse the external PR176 review's independently authored WASM matrix with
explicit new baseline/candidate identities and unchanged substantive assertions;
this is a rerun of that matrix, not new independent coverage.

Run affected native profiles and strict lint, optimized depth/capacity checks,
WASM/kit consumers, C3, compatibility/older-save continuation, existing integration
and installed-package checks, followed by final CI and a fresh current-file
CAVEAT evidence audit. No memory or latency target substitutes for correctness.

Required-chain growth, conservative retention, index/transaction copies, retained
allocation capacity and archive storage remain active engineering problems. This
follow-up is one specific optimization. Authoring, diagnostics, integration and
discovery remain product concerns for evidence-aware applications and agents;
no new wrapper or restriction to release checking follows from this work.
