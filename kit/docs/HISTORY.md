# History, departure and the host archive

Introduced in rc.16, departure lets eligible retired records leave a windowed
session. The runtime keeps exact own grounds and required withdrawal reasons.
Inherited lineage can compact to a marker, while the host keeps the historical
records and membership proof separately. This helps long-running integrations
without treating a caveat as something to discard when it becomes inconvenient.

## Keep the archive when you need history

The session API exposes `session.undrained`, the number of pending archive
items, and `session.drainArchive()`, which returns those items and clears that
buffer. Items include both departed records and provenance nodes. Later chunks
can refer to earlier chunks. Keep the full matching archive for exact
historical reconstruction; saving only the latest chunk is insufficient.

For a short-lived example, with an already opened `session`:

```js
import { explain, dependents } from 'caveat-lang/explain';

// Demonstration only: a long-running host should persist chunks outside this array.
const archive = [];
function retainPendingHistory() {
  archive.push(...session.drainArchive());
}

retainPendingHistory();
const report = explain(session.snapshot(), [], { archive });
const affected = dependents(session.snapshot(), 'sky', { archive });
```

`sky` must be a name in your program. `explain`'s second argument is the event
list you choose to report; `[]` supplies no event log. Neither helper executes
the program or changes a decision. For a long-running service, persist chunks
outside the runtime rather than accumulating the example array indefinitely.
Draining removes the runtime buffer, so retain the returned items until your
storage write succeeds. The runtime does not supply a durable acknowledgment
protocol or a host database transaction.

`caveat serve` has matching `undrained` and `drainArchive` operations. Its
`explain` and `dependents` requests accept an `archive` array supplied by the
host. The server does not keep an extra accumulated archive for you. See
[Serve 0.1](reference/spec/caveat-serve-0.1.md) for exact request fields.

## Interpret conservative and historical results correctly

A marker summarizes a departed history range. It does not assert that every
record within that range was read. The matching complete provenance graph and
record entries permit exact membership reconstruction through copying and
merging. Without enough consistent archive data, the report keeps
`archive_status: "unavailable"` and a conservative explanation; it must not be
presented as an exact list of missing reasons.

The separate historical `archive` section reports each supplied record as
`status: "complete"` or `"unavailable"`, with unresolved dependencies visible.
Its explicit scope is `"provided records and their referenced closure"` and
`authenticated` is always `false`. Complete means that the supplied record and
its referenced dependencies are consistent. It does not establish a complete
session inventory, every incoming relationship, trusted payloads, or a unique
branch when multiple executions use the same program source and occurrence
names. Preserve this scope when rendering reports to people or passing them to
agents.

Archived withdrawal records retain their original concrete reasons and
relationships, including relationships between records that depart together.
Those historical results do not grant current permission or change current
decision status. Application policy must decide what action follows from the
current assessment.

## Saves, snapshots and compatibility

A snapshot contains the records currently retained by the session. An eligible
departed record may stop appearing in later snapshots; do not use snapshot
inventory as an exhaustive historical log. Use the archive for historical data.

A save contains live state and compact markers, not the drained archive or the
undrained buffer. Store the exact program source with the save, and keep the
host archive separately. Restore starts with an empty pending archive and does
not depart anything on load. Valid older saves can continue; after restore,
the next accepted event may collect eligible records. A rejected event must
roll back its changes, including proposed departures and archive output.

The runtime preserves the distinction between exact own grounds, which retain
records, and inherited provenance that ordinary departure is permitted to
compact. The withdrawal collector additionally retains uncertain dependency
paths conservatively. It does not rewrite genuine reasons to meet a size
target. Collection occurs within the event transaction. Earlier collection can
free storage for a later event, but does not run early to rescue an event that
the existing capacity admission check has refused.

Programs without windows have no departure or archive output. Windowed programs
can have different retained snapshots from rc.15 as eligible records depart;
that difference is part of this API, rather than lost current evidence.

## Budget storage and synchronous work separately

Windows and unreachable-cycle collection do not provide a universal save bound.
A required withdrawal-reason chain, exact own grounds or conservative retention
can keep older records live. Live heap, temporary transaction/collection peaks,
serialized saves and host archive storage are separate costs. Draining does
not eliminate the storage needed to retain history, and frequent draining does
not change execution or collection eligibility.

A large final release can perform substantial synchronous work. Test the
actual application event mix against explicit pause and memory budgets before
placing it on a frame or request deadline. The version's
[release notes](https://github.com/WSattazahn/caveat-lang/blob/main/docs/releases/v0.1.0-rc.16.md)
record measured workloads and their limitations.

The packaged [Windows](reference/spec/caveat-windows-0.1.md),
[lineage-compaction](reference/spec/caveat-lineage-compaction-0.1.md),
[departure](reference/spec/caveat-departure-0.1.md) and
[collector contract](reference/docs/design/withdrawal-collector-draft.md)
describe the retention rules. The
[root inventory](reference/experiments/departure-gate/WITHDRAWAL-ROOT-INVENTORY.md)
records the source consumers behind conservative retention.
