# Departure continuation gates

This directory holds the registered C3 gate, compatibility sweeps, authentic
published rc.15 restore fixture, pin census, and the separate withdrawal
reachability proposal. The rc.16 continuation is unpublished.

## Reproduce

After the repository's normal dependency setup and `npm run build`, run:

```sh
node experiments/departure-gate/run.mjs
```

The runtime CI workflow runs this command. It builds the locked native rc.15
baseline at commit `3a88ba0f80d563b4493840dc7bd7e195329b8302` from a Git
archive in a temporary directory, builds the current native sweep examples,
then runs both against the same current source population. It fetches that
exact commit if a shallow checkout lacks it. It does not create a worktree,
switch a branch, merge, or publish. Rust, Cargo, Git, tar and Node are required.
The baseline directory is retained for inspection; its path is printed.

`sweep.mjs` records every delivered non-windowed `.cav` program, including
skips for sequential or invalid examples, and every windowed fixture. The
no-window comparison covers each dispatch outcome, saved text, and restored
saved text over 8 seeds of 150 events. Windowed comparisons cover dispatch
outcome classifications over 32 seeds of 400 events; departure effects and
archive metadata intentionally differ from retirement-only rc.15.

`c3.mjs` runs the round-7 C3 adapter with both windows for 60 cycles / 44,460
events. It measures compressed adapter saves, raw saves, normalized live-save
structure, dispatch percentiles, pin attribution, undrained items and the
serialized archive. `cross-restore.mjs` restores the original published rc.15
save text, checks exact restoration at checkpoints, and follows accepted source
write paths until all 61 states have received a set. That coverage is derived
from source paths checked against before/after guards, not an instrumented
runtime statement trace. `pinned.mjs` runs the five registered windowed corpus
programs, including the forever-skipped commit fixture, for 8 seeds of 4,000
events with 100-event pin/restore checkpoints.

The runner writes receipts under `test-results/`; `check-results.mjs` rejects
population omissions, incompatible digests, C3 outcome drift, different 30/60
adapter save sizes or normalized structures, late C3 median dispatch >=1ms,
incomplete state-write coverage, missing pin checkpoints, or measurements from
different WASM artifacts. `--skip-sweeps` reuses already observed native sweep
files; it still reruns all current WASM measurements. It is for local receipt
collection, not the CI default.

## Published fixture provenance

`fixtures/rc15-c3-save.json` is the exact 226,589-byte save produced through
the actual verified published rc.15 WASM, after the 60-cycle C3 run. It has
not been reconstructed or parsed and reserialized. Its source and save hashes,
published package digest, WASM digest and release revision are in
`fixtures/rc15-c3-provenance.json`. The original measurement is retained beside
it. This identifies the artifact used; it does not authenticate arbitrary saves
or supplied evidence. The runner verifies fixture/source hashes before use.

## Recorded continuation evidence

The machine-readable Windows continuation receipts are in `results/`.
They identify an unpublished working build based on Claude's preserved WIP
`7f551e5774d61ec9f88e0d603e5b01196597b00c`, with runtime source fingerprint
`8d20715134565c8df04a321385145f170de4ab7457b5bcb0cc1ec245b0b805dc` and reactive
WASM SHA-256 `daab2c71023777a3a9e5dc72f7882c653147c4acb445470368cecd76ed30bc59`.
The build explicitly records `clean:false`; it is not a clean release artifact.

The registered C3 adapter save is 2,646 bytes at both 30 and 60 cycles. Raw
save sizes are 25,682 and 25,683 bytes (growing ordinal digits, identical
normalized structure). The published rc.15 baseline grows from 11,870 to
22,182 adapter bytes. Both runs accept the same 44,460 events with the same
outcome digest. Dispatch timings in the reports are single-host observations,
not a controlled cross-machine performance claim.

The current C3 archive holds 1,578 items: 1,011 departed records and 567
provenance nodes, totaling 557,347 serialized JSON bytes. Draining leaves zero
pending items and the saved session unchanged. This historical data grows
outside the save. Process-memory readings include the entire Node/WASM
workload and must not be described as isolated archive heap measurements.

The published save restores at 226,589 raw / 22,182 adapter bytes. After the
first event it uses 92,588 / 6,530 bytes; after all 61 states have been written
(742 accepted events), 26,513 / 2,698 bytes. Preserved older own grounds can
retain extra records until replaced; restore does not silently discard them.

## Exact archive membership and its boundary

[ARCHIVE-DESIGN.md](ARCHIVE-DESIGN.md) records the owner-approved transfer
metadata amendment. Native and actual-WASM kit tests distinguish the original
ambiguous memberships after copies, overlaps, merges, replacements and
restore, and cover failed events and missing/corrupt proof. CLI replay, inline
authoring, browser and persistent line-protocol consumers are exercised.
Complete matching archive reconstruction is exact membership, not authenticated
history. Without complete proof, explanation text is explicitly conservative.

## Broader boundedness remains unresolved

The registered C3 result is not a universal save-bound proof. Current pin
semantics retain self/mutual withdrawal cycles and reachable reason chains.
[WITHDRAWAL-REACHABILITY-DESIGN.md](WITHDRAWAL-REACHABILITY-DESIGN.md) proposes
independent roots and required dependency edges for review. Its eight fixtures
and 1,000-cycle measurements observe the current runtime; no collector has
been implemented. The owner explicitly requires review before any such
semantic change. Exact own grounds, true required withdrawal reasons,
permissions, source-observable behavior and complete archive reconstruction
remain requirements. Reachable chains can still grow after isolated cycles
are collected.
