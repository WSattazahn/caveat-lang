# Measured interpretation and next-task recommendation

The strongest next candidate is to box archive entries when they enter the temporary map, then sort and publish those same boxes. This targets temporary representation in the phase that dominates the measured bulk release. It is a proposal only: no runtime optimization was made or benchmarked. The evidence does not establish a 20% whole-dispatch peak improvement.

The frozen capture is `11c2a6a741609d3b640f27224ab6c9943b07f72c`, runtime tree `89f3a5bde145a3071ce6e41652a866e6b722f3b1`. The arithmetic in `measured-findings.json` and `measured-findings.md` comes from the preserved stdout of all 48 processes across 12 cells. Twelve accepted-reference processes establish finite semantic equality only; their timings are excluded. Four process triples cover the primary mutual3000 case; smaller and other workloads have one triple each. The coordinator held agent-controlled builds/tests during capture; this does not establish an isolated host.

## Bulk release: time and the simultaneous peak

| Measure | Successful release | Rejected release |
|---|---:|---:|
| Fresh plain apply median | 63.16950 ms | 56.52440 ms |
| Timing-mode apply median | 62.75105 ms | 58.59990 ms |
| Archive construction/removal median time | 27.33100 ms | 28.89195 ms |
| Median per-event archive phase share | 43.555% | 48.938% |
| Collection median time | 22.49850 ms | 23.17235 ms |
| Median per-event collection share | 36.359% | 39.683% |
| Requested additional global peak | 27,403,251 B | 27,403,464 B |
| Absolute requested global peak in origins mode | 41,530,214 B | 43,627,579 B |
| Prior-event requested bytes alive at that peak | 14,126,963 B | 16,224,115 B |
| Current archive-phase origin bytes alive at that peak | 24,229,401 B | 24,229,401 B |
| Current collection-origin bytes alive at that peak | 3,171,130 B | 3,171,130 B |

Each rejected process first contributes its median of three failed events; the table then reduces the four process values. Shares are computed per event before medians. Independent phase medians must not be added to reconstruct a median total. Every primary origins event peaks during archive construction/removal, with the same requested-byte partition for its outcome. The remaining current-origin bytes are preparation 445 B and evaluation 2,275 B on success / 2,488 B on rejection. These figures describe one simultaneous peak, not a sum of phase peaks.

The archive phase requests 39,840,627 B through 189,809 allocation and 33 reallocation calls per primary event, and frees 25,049,937 B through deallocation/reallocation accounting. Collection requests 7,481,331 B through 184,687 allocation calls. These are cumulative requested logical bytes; reallocations charge the whole new request. They are not physical copy counts and do not identify individual maps or vectors.

Transaction preparation itself is small (median 0.00395/0.00415 ms), but this does not make transaction copying free: shared containers detach in later evaluation, collection and archive phases. Successful final settlement is 6.06245 ms median; commit cleanup 4.19950 ms. Rejected rollback cleanup is 4.11345 ms. Required information, undrained archive, retained container capacity and temporary copies remain different costs.

## Perturbation and memory boundaries

Paired timing/plain apply ratios have median 0.99403 on success and 1.04334 on rejection, with ranges 0.97189–1.04642 and 0.98449–1.08983. Allocation-origin/plain ratios are 1.09411 and 1.18348, with ranges 1.05502–1.14901 and 1.09693–1.19637. Origin mode has no phase clocks; its elapsed times describe instrumentation cost, not a substitute estimate of ordinary latency. The reported phase rankings come from timing mode. Four process triples bound these observations; they do not establish population confidence intervals.

Every reported additional requested peak and session-retained value matches the fresh plain control in these cells. This equality excludes diagnostic headers/padding and corrects for preallocated report buffers; it is not equal process memory. At the successful primary requested peak, origin headers occupy 4,878,848 B and padding 820,290 B; System-requested storage is 47,229,352 B. On rejection these are 4,878,880 B, 820,293 B and 49,326,752 B. Origin report buffers request 4,870,080 B versus timing's 948,384 B in the primary case and are present before the event baseline. Prior-event bytes therefore include harness storage; they must not be labeled wholly as retained session state.

Requested heap includes the requested capacity of live containers, but excludes allocator free lists, allocator-internal bookkeeping and OS reservation/RSS. Metadata/header totals describe this diagnostic's added representation. The successful origins event ends 9,811,276 B above its start before drain; the rejected one ends 49 B above start while the returned error remains live. This is not evidence of a rollback leak.

After successful release and drain, session-retained requested heap falls from 9,252,154 to 2,948,056 B; save size from 1,290,931 to 298,583 B. Retired records fall from 5,998 to zero and withdrawals from 6,000 to two. The complete primary process emits 3,228,566 archive bytes, including 99 unrelated-probe departures; growth itself emits no archive. Required archive output must remain exact even if staging changes.

## Required-chain growth is a distinct cost

| Chain cycles | Last-10% plain apply median | Timing median | Origins median | Timing collection share | Growth additional peak | Retained heap | Save bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| 300 | 1.5100 ms | 1.4466 ms | 1.6417 ms | 57.727% | 858,728 B | 791,426 B | 118,670 B |
| 1,000 | 4.8587 ms | 4.7148 ms | 5.5799 ms | 61.335% | 2,966,209 B | 2,720,830 B | 398,709 B |
| 3,000 | 14.1688 ms | 14.5153 ms | 17.8791 ms | 64.301% | 8,574,441 B | 7,811,422 B | 1,238,709 B |

These are descriptive one-triple cases, not stable tail-latency estimates. At the largest observed chain3000 growth peak, 6,527,491 B of current-event live requests originate in evaluation and 2,046,090 B in collection, although the peak occurs in collection. Thus peak location and allocation origin lead to different conclusions. Collection dominates elapsed tail growth, while evaluation-created storage supplies most additional live bytes at the peak. The still-required chain survives; its retained heap/save growth is not an archive leak. Earlier archive boxing is not expected to address this workload, which has no departing archive during growth.

## Why earlier boxing is a concrete, bounded candidate

Current source inserts inline `ArchiveEntry` values into a temporary map at `runtime/src/reactive_departure.rs:952`, attaches departing journal entries through that map at line985, collects inline entries into an ordering vector at line995, sorts by history and numeric occurrence, then boxes each entry at line1005. The ordering vector's allocation remains alive as its consuming iterator publishes boxes. Moving an entry between these representations does not itself allocate its owned string payloads; allocating the containers and final boxes does request storage.

The separate layout-only probe links the already-built frozen runtime rlib with the matching Rust 1.98.1 compiler. It constructs no session and runs no event. On this x86_64 build:

| Type / logical storage | Bytes |
|---|---:|
| `ArchiveEntry` (alignment 8) | 648 |
| `(bool, ArchiveEntry)` map value | 656 |
| `(bool, Box<ArchiveEntry>)` map value | 16 |
| `Box<ArchiveEntry>` | 8 |
| `ArchiveItem` | 120 |
| Inline ordering payload for 5,998 entries | 3,886,704 |
| Box-pointer ordering payload for 5,998 entries | 47,984 |
| Logical ordering payload difference | 3,838,720 |

That difference is about 14.0% of the measured whole-dispatch additional peak. It is a representation-size calculation, **not measured savings at that peak**. Earlier boxing still allocates all required 648-byte entries, with earlier lifetimes. Map node occupancy/capacity, sort scratch and their temporal overlap were not separately attributed. The 24,229,401 archive-origin bytes also include other required payloads and detached containers. The measured data cannot honestly label all of them duplicate staging or predict a 20% improvement.

The scoped implementation proposal is therefore: store boxed entries in the existing map, mutate the same boxes for journal attachment, collect/sort those boxes with the exact comparator, and move each existing box into `ArchiveItem::Record`. Keep eligibility, holdings/reasons, archive fields/order, graph changes, withdrawal extraction, journal bounds, effects and rollback unchanged. Avoid combining this task with collector, transaction or archive format redesign.

If the owner elects to implement it, a **proposed, unmeasured** finite criterion is at least 10% lower total requested additional peak for both mutual3000 success and rejection, using fresh matched ordinary counting-allocator builds: at most 24,662,925 B and 24,663,117 B relative to these observed baselines. Freeze a fresh baseline and aggregation before running; assess all registered attempts. Require exact snapshot/save/ordered-archive equality, rollback equivalence and unchanged retained heap after drain. Report matched plain latency and all smaller/chain/high-degree phase peaks, even if unchanged or worse; do not claim a language-wide memory improvement. A 20% bar would instead be an owner's aspirational requirement (21,922,600 B / 21,922,771 B), not a gain established by this profile. No implementation or future acceptance threshold was authorized by this profiling task itself.

Layout provenance is preserved under `layout-probe/attempt-003/receipt.json` with source, compiler, rlib, executable and raw-stream hashes. The first attempt failed before compilation on sandbox Git access; the second used the scratch directory's different default compiler and failed compatibility checking. Both failures remain preserved. The successful retry explicitly pins `rustc +1.98.1`; no benchmark or runtime source was rebuilt by the probe.
