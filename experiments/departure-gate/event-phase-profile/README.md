# Event phases and allocation origins — profiling only

This experiment starts from accepted runtime PR #177
`055e2af0b411d8385941dd8522201e5afc73cb34`, preserving the prior renewal-removal
diagnostics and Windows scanner repair at `677ad1365398fe37bb24ff8fb6cfbae82d3ef5b5`.
It does not implement an optimization or change the language contract.

Build three native release artifacts with locked dependencies and the same
toolchain. Preserve each executable before the next build replaces the example:

```
cargo build --release --locked --manifest-path runtime/Cargo.toml --example collector_profile --no-default-features --features collector-metrics
cargo build --release --locked --manifest-path runtime/Cargo.toml --example event_phase_profile --no-default-features --features event-phase-profile
cargo build --release --locked --manifest-path runtime/Cargo.toml --example event_phase_profile --no-default-features --features event-phase-origins
```

The new example takes the original source/cycles/mode/drain/probes arguments and
an additional literal `timing` or `origins`, matching its compiled feature.
Neither new build enables the older per-record renewal or withdrawal clocks.

The native feature adds a fixed set of guards around actual scopes. Timing
slices are exclusive; resumed scopes accumulate into their own phase. Successful
departures have 13 transitions; paths that return earlier have fewer. No phase
clock runs per record. Preparation includes validation and staged-session cloning;
evaluation includes ordinary rule execution. Collection, provenance compaction,
combined archive construction/removal, binding evaluation, final provenance
settlement, commit/rollback cleanup and the apply wrapper are separate phases.
The combined archive phase preserves the interleaved source algorithm. Implicit
destruction stays in its original order and remains inside the corresponding
guard. Host drain, output serialization and save generation are outside apply.

Allocation-origin mode disables phase clocks but uses the same scope tags. Its
allocator adds per-block metadata, reported separately from requested payload
bytes. It observes the phase of the global event peak and the origins of blocks
still alive there, including a separate prior-event/harness category. Realloc
replaces a logical allocation: on success its entire new requested block receives
the current origin; on failure the old block/origin remains. These are allocation
requests, not physical copy traffic. Independent phase peaks must not be summed.
Requested heap is not RSS, resident memory, allocator freelists, or stack use.

`profile.py` freezes a finite 12-cell population: four mutual3000 primary triples,
mutual60/300/1000 controls, self3000, high-degree1000, and reachable-chain300/1000/3000.
Each cell runs a fresh accepted-source reference for finite semantic comparison,
then fresh plain/timing/origins processes in predetermined rotating order. All
attempts, raw streams, failures and source/build identities remain in the review
bundle. The four primary triples are balanced only to within one ordering slot.
Supplemental single triples are descriptive, not stable tail estimates.

The companion local review bundle contains the exact source revision, build
receipts, executable hashes, frozen protocol, all raw event samples, correctness
receipts, previous 677 diff/raw bundle, analysis scripts and reproduction steps.
No claim is made here that profiling establishes a speedup, universal memory
bound, supported workload budget, release readiness, or authenticated history.
Renewal batching remains a scaling concern. Required chains and archive storage
remain active engineering problems. Any next optimization requires a separate
user decision after reviewing this investigation.
