# Allocation attribution methods

This is a description of the native diagnostic implementation, not a measurement
result or optimization claim. The original `collector_profile.rs` remains
unchanged. `event_phase_profile.rs` runs the same source augmentation, event
sequence, rollback assertions, archive drain/hash, save measurements and successful
collector-work reporting. It additionally retains raw event samples in fixed,
preallocated buffers and serializes them only after measurement.

## Build and timing separation

The example takes `SOURCE CYCLES MODE DRAIN_EVERY PROBES timing|origins`.
The last argument must agree with its compile-time allocator:

- `event-phase-profile`: timing mode, with the original `System` counting
  allocator path and requested-byte `LIVE`/`PEAK` atomics. No allocation headers
  or origin recorder compile into this mode.
- `event-phase-origins`: origin mode, with a header-backed `System` allocator.
  Runtime phase clocks are disabled; phase entries, transitions and the current
  phase index remain available. The outer `apply` clock still runs.

A runtime switch after argument parsing would leave pre-main allocations without
headers, so the two allocator choices require separate builds. Both are diagnostic
builds; fresh ordinary controls are needed to expose their respective combined
timing/working-set effects. Origin-mode time is not ordinary latency, and no fixed
correction is subtracted from it.

The outer clock covers `ReactiveSession::apply`. Runtime phase-transition TLS
operations are inside this interval. In origins mode, per-allocation phase TLS
queries, header access, locked fixed counters and fixed-array copies at new peaks
are also inside it. Reset/begin, final recorder extraction and returning the fixed
snapshot happen outside the clock. Later save serialization, archive draining,
NDJSON hashing and output construction are outside apply, as in the original
harness. Origin mode has `clock_enabled: false`, zero recorded phase durations and
`remainder_ns: null`; those zeros do not mean the phases took no time.

## Requested bytes and header layout

The primary measure is the exact size requested by the Rust caller in its
`Layout`. This includes requested vector/map/container capacity even if unused.
It excludes allocator metadata/free lists, stack storage, OS reservations and RSS.

Origins mode prepends an allocation header containing an event epoch (`u64`) and
phase index (`usize`). On the current x64 build this is 16 bytes; the executable
reports `origin_layout.header_bytes` rather than assuming that size. The helper
uses `Layout::extend` and `pad_to_align` to obtain a valid `System` layout and
payload offset, preserving the caller's alignment. Padding includes both the
gap before the payload and final alignment rounding.

The full layout size requested from `System` is recorded separately. This is
still a requested size, not the physical size of an allocation returned by the
system allocator. Header and alignment overhead do not enter the primary requested
heap/retained measurements. Separate header-inclusive peaks expose their cost.

There is no pointer map or dynamically allocated recorder state. One fixed state
is protected by a spin lock; all `System` calls and the allocation-free TLS phase
getter run outside that lock. Critical sections use scalar counters, pointer
header reads/writes and fixed arrays. This serializes bookkeeping, not an entire
underlying allocation. Measurements use the native single-threaded event harness;
if another thread allocated during a window, its default phase would be `outside`.
The recorder does not establish global host isolation.

## Event epochs and allocation ownership

There are twelve origin buckets: the eleven public phase indices and one
`prior_event` bucket at `origin_layout.prior_event_index` (11). Phase 0 is
`outside`, an unclassified current-event allocation, not the same as prior-event
storage. The other phase names are emitted in `phase_names`.

At `begin_event`, the allocator advances its epoch, folds all currently live
requested bytes and allocation counts into `prior_event`, clears current-event
buckets and event-local flow counters, and records both requested-byte baselines.
No heap walk or mutation of existing headers is needed. Later deallocation reads
the stored header: an older/zero epoch debits prior-event storage; this event's
epoch debits its recorded phase. Allocations outside an active event receive epoch
zero. At the next event they are part of its prior-event baseline.

`prior_event` therefore includes pre-session source/argument buffers, preallocated
sample buffers, session state retained from earlier events, and any harness
objects already live at the start. In rejected-release samples it also includes
the saved rollback-reference string held by the original harness. It is not a
claim that all those bytes belong to the session or a known historical phase.

Successful `realloc` is a logical replacement: debit the entire old requested
size from its stored epoch/phase; credit the entire new requested size to the
currently executing event/phase, even if the pointer did not move. This changes
origin on growth, shrink and same-size reallocation. The old and new alignment
are equal, so the payload offset is unchanged. Failed reallocation preserves the
old header, bytes and ownership. Failed allocation adds no live bytes. Layout
overflow while adding diagnostic metadata returns failure rather than issuing an
invalid `System` request.

This convention records logical requested-allocation ownership, not which bytes
were physically copied, rewritten or moved. Transient internal memory used by
`System::realloc` is opaque. A successful reallocation updates old/new counters
atomically without recording a fictitious simultaneous old-plus-new live peak.

## One global peak and its live origins

For each apply window the allocator retains the **first strict maximum** of total
live caller-requested bytes. At that instant it copies the entire live-origin
byte/count partition and records the phase where the maximum occurred. This is
one global peak; it is not a sum of independent phase maxima. A peak in archive
construction can include live objects allocated earlier in transaction preparation.

If the event-start baseline remains the maximum, `requested_peak_phase` is null
and the saved partition is the all-prior-event starting state. Equal later maxima
do not replace the first one. A separate first-strict maximum records the full
header-inclusive `System` request total; it can occur at a different point.

`Sample.allocations` is null in timing mode and an object in origins mode:

| Fields | Meaning |
|---|---|
| `epoch` | This allocator event window. |
| `event_start_requested_live_bytes`, `event_start_system_requested_live_bytes` | Live baselines before apply, including harness storage. |
| `requested_live_at_peak_bytes`, `requested_peak_phase` | Absolute requested-byte maximum and location; null location means the starting maximum remained. |
| `requested_additional_peak_bytes` | Requested maximum minus event-start requested baseline; equals the sample's ordinary `peak_additional_bytes`. |
| `requested_live_by_origin_at_peak[12]`, `live_allocations_by_origin_at_peak[12]` | Live requested bytes/allocation counts by origin at that same maximum. |
| `metadata_bytes_at_requested_peak`, `alignment_padding_bytes_at_requested_peak`, `system_requested_bytes_at_requested_peak` | Header/padding costs and full System request total at the requested-byte maximum. |
| `system_requested_peak_bytes`, `system_requested_peak_phase`, `system_requested_additional_peak_bytes` | Separate full-System-request maximum, location and increment over its own baseline. |
| `requested_live_by_origin_at_system_peak[12]`, `metadata_bytes_at_system_peak`, `alignment_padding_bytes_at_system_peak` | Requested origins and instrumentation costs at the separate System-request maximum. |
| `requested_live_at_end_bytes`, `requested_live_by_origin_at_end[12]`, `system_requested_live_at_end_bytes` | End-of-apply observation before returning the sample; the returned outcome can still be live. |
| `flow_by_operation_phase[11]` | Calls and cumulative logical requested-byte traffic by executing phase, distinct from allocation origin. |

Each flow entry contains `alloc_calls`, `alloc_zeroed_calls`, `realloc_calls`,
`dealloc_calls`, `allocation_failures`, `realloc_failures`,
`requested_allocated_bytes` and `requested_freed_bytes`. Failed calls count as
calls/failures but add no allocated/freed bytes. Successful realloc adds the full
new request and full old free to that phase's traffic. A free's executing phase
may differ from the allocation's original phase. These cumulative flows are not
live heap and must not be substituted for its peak partition.

At the requested peak, origin bytes sum to `requested_live_at_peak_bytes`; adding
metadata and padding produces `system_requested_bytes_at_requested_peak`. At the
System peak, its origin bytes plus metadata/padding sum to
`system_requested_peak_bytes`. End origin bytes sum to requested live-at-end.
The metadata count equals the live allocation count times the header size.

## Harness baseline and output limits

The growth, unchanged, unrelated, rejected and successful sample vectors all
reserve their fixed maximum capacities before `base_heap`; no vector grows during
the registered event sequence. `sample_buffer_requested_bytes` is the sum of
their capacities times `size_of::<Sample>()`. It is a subset of
`harness_baseline_requested_bytes`, recorded before constructing the session.
Initialization is an optional stack sample. No event output is streamed during
the measured sequence.

Sample buffers differ in size between timing and origins builds. Their requested
bytes are excluded from retained-session measurements by the pre-session baseline,
but they remain real live bytes in absolute origin snapshots and may perturb the
working set. Their headers/padding are also present in origins-mode absolute
System-request snapshots. Do not misread a larger absolute prior-event bucket as
larger session retention. Use the emitted buffer/baseline context and per-event
additional peak; do not invent a session-only partition by subtracting arbitrary
bytes from individual phase buckets.

The pre-release report allocation is corrected out of final retained heap as in
the original harness. This example allocates the one successful-release sample
buffer before the session baseline, instead of allocating and separately
correcting it later. Raw `event_samples` contains optional initialization plus all
growth, unchanged, unrelated, failed-release and release samples. Phase summaries
retain the original median/peak/work schema. Growth-tail samples overlap growth
and must not be added as a separate population.

Timing samples include `phases.phase_ns`, `phase_entries`, `transitions` and
`clock_enabled`, plus `phase_total_ns` and `remainder_ns`. The latter is outer
apply time minus the sum of the recorder's nonoverlapping phase durations; it is
unmeasured/instrumentation residual, not measured `outside` work. A negative
residual fails the harness. Origins mode emits no phase-time attribution.

## Preliminary checks

Five focused allocator tests passed: aligned/zeroed allocation and deallocation;
growth/shrink reallocation and payload preservation; earlier surviving origins at
a later-phase global peak; prior-event reallocation; and failed allocation/realloc
retaining the original live block. They include origin/metadata/padding partition
equalities. Strict example Clippy passed in both modes. These were preliminary
source checks, not dispatch measurements or a final-head gate receipt. Exact
commands/output are in `allocator-focused.*` and `example-*-clippy.*` beside this
document. Final source validation and frozen capture are coordinated separately.
