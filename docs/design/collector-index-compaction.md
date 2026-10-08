# Collector index compaction

Status: implementation and testing authorized for a separate reviewable follow-up,
2026-10-08 UTC. Base: `3d77aa27bcffa16f39818b2684e9afb3b102ed17` (PR #175).
The owner accepts that repaired collector as the experimental development baseline
and its explicit snapshot, historical-completeness and capacity-only choices.
PR #174 stays unchanged at `f5ec8294efe2be24705f234ef75e5f5459aa5e89`.
No merge, publication, release or rc.16 readiness is authorized.

## Scope and invariants

Reduce the representation and copying cost of derived withdrawal-collector
indexes. Authoritative withdrawals and their exact reasons remain intact.
Retain the current transaction boundary, retention eligibility, qualified
computation, event outcomes, exact saved strings and globally ordered archive
items. Preserve the four independent-review repairs in `3d77aa`.

Do not weaken retention, defer collection, move capacity admission checks,
change the save schema, add lossy summaries or import host archives into
execution to achieve the target. This work does not authorize a general
transaction undo system, bulk-departure rewrite or capacity reclamation.

The first implementation investigates compact deterministic adjacency storage,
exact reverse-reason multiplicities and avoiding unchanged adjacency clones.
Single and small groups should avoid an allocated tree node; larger groups
still need efficient ordered operations. Inspect actual consumers and degree
distributions and retain high-degree regressions. Representation transitions
must preserve uniqueness, ordering and exact overlap counts.

## Frozen success criterion

For each quantity at 3,000 cycles of the existing reachable-chain workload:

* `F`: original retaining-baseline cost at `f5ec829`;
* `R`: repaired collector cost at `3d77aa`;
* `C`: compaction candidate cost.

Both gates must satisfy `(C - F) <= 0.5 * (R - F)`. Report absolute bytes,
the added costs, the denominator and the resulting percentage. Do not replace
this with a percentage of total runtime memory or select another fixture if
the gate fails. The two quantities are retained requested Rust heap and the
maximum additional requested heap during growth dispatch, measured by the
existing `collector_profile` harness.

The frozen observations motivating this task were:

| Quantity | Retaining baseline F | Repaired collector R | Candidate ceiling from those observations |
| --- | ---: | ---: | ---: |
| Retained requested heap | 6,004,653 B | 13,413,126 B | 9,708,889 B |
| Growth-dispatch additional heap peak | 6,139,810 B | 14,176,145 B | 10,157,977 B |

These are reference observations, not results for this implementation. Repeat
the baseline and repaired comparison with matching workload, harness, compiler,
optimization settings and definitions, and expose any changed reference values.
Use integer arithmetic for the gate where practical. Counter/allocator overhead
is included; requested bytes exclude allocator metadata, stacks and RSS.
The dispatch peak includes the transaction and archive work; a difference of
whole-dispatch peaks is not isolated collector scratch allocation.

## Verification

Run the same existing chain/self/mutual/final-root-release workloads at increasing
history sizes, plus unrelated mutations and the focused high-degree fixture.
Preserve before/after executable, source, fixture and harness hashes. Keep
measurement runs free of concurrent local builds/tests; report repetition counts
and residual host-load limits. Never infer a tail-latency guarantee from one
release sample or from a mostly-tick aggregate.

Compare repaired-baseline and candidate exact saves, snapshots, current values,
qualifications, reasons, decisions, outcomes and ordered archive data wherever
the harness exposes them. Original retaining baseline comparison still allows
the previously approved collection differences; do not demand that it depart
records it deliberately retains. Check real mutation and unchanged paths,
shared references, multiplicity, high-degree transitions, bulk release,
rejected-event rollback, valid older saves and restore continuation.

Run the affected native profiles, strict lint/format, collector/capacity gates,
WASM/kit consumers, installed-package checks, no-window and windowed compatibility
sweeps, registered C3, and final CI. The evidence plan and report must identify
which checks cover correctness, compatibility, measurement or integration.

## Decision after measurement

Return an exact review commit, reproducible before/after records, regression
findings and a literal result for each target. If either misses, report the miss
and the smallest proposed next step; do not redefine success or automatically
expand this task. Passing both targets does not establish universal boundedness
or suitability for unrestricted interactive use. A supported-workload claim
requires a named workload and resource budget.

Required-chain representation, conservative retention, excess allocation capacity,
bulk-departure costs and archive growth remain engineering work after this task.
Exact external storage still costs space; moving it does not eliminate it.
Ease of authoring, integration, discovery and use by people and agents remains
on the product agenda. Improve the existing qualified-computation, explanation,
authored-action and reconsideration paths without narrowing Caveat to release
checking or adding a wrapper without an observed need.
