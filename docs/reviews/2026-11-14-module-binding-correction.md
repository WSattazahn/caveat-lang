# Correct module bindings and preserve historical save choices

This supersedes the decision in the earlier
[compiler compatibility review](2026-11-14-compiler-compatibility.md) to keep
successful 1.4 root capture. The owner explicitly requested that the bug be
fixed, rather than preserved as the default. PR #479's draft `unset` repair
was merged separately at `1c6b81241f97b2109675049faa4f868fbf28c5f3`.
This correction is unreleased. The reconstructed root-capture/save-ambiguity
finding is recorded as D091, without asserting that it is identical to any
unrecovered D084–D088 entry. The ledger records its observation in published
release builds; it does not record approval to merge this follow-up.

## The defect and the unavoidable ambiguity

A module-local `hold=7` used in `add( hold , because )`, with `because=3`,
should produce 10. The old linker mistook the reference for an effect
keyword and left it bare. A root `hold=11` then made the same program
produce 14. The same error affects `release`, authored functions and
procedure calls. The corrected linker qualifies references and leaves only
actual effect keywords bare. Fresh reactive loads and public `link::link`
use it directly. They do not fall back to a successful wrong interpretation.
A source that loaded only because it captured a wrong root declaration may
now correctly fail validation.

The collision fixture's published 1.0.0 and 1.4.1 initial saves are identical
bytes, but their next event produces 10 and 14 respectively. Both have
`source_id=fnv1a64:8cb496ca5fa15ec4:195` and source SHA-256
`b8f3d86d72c1db1d3ca8ed50967eac9934a1fec07d855c87af7c02cd1890b138`.
There is no reliable version or state heuristic that can choose both
histories correctly.

## Restore contract

- Affected state, named replay and positional replay saves record
  `module_bindings`: `scoped` or `legacy-hold-capture`.
- Unmarked affected saves require an explicit choice when both distinct
  linked programs fully load. If only one loads, restore uses that one.
  Saved state validation is never used to pick a convenient interpretation.
- A saved marker selects the interpretation. An explicit conflicting option
  is refused. The selected program supplies the initial state, bounds and
  capabilities before saved state is applied or events replayed. Cache keys
  include the interpretation.
- Source identity still covers the original raw bytes. When both linkers
  produce identical source, saves omit the marker and preserve their bytes.
- Selection continues historical state; it does not rewrite frozen decisions
  or migrate past state to corrected semantics. It authenticates nothing.

The [save specification](../../spec/caveat-save-0.1.md#module-binding-interpretation)
defines the native, WASM and kit APIs. Serve accepts an optional per-request
choice and a startup flag for an existing state file. Successful explicit
startup restore checkpoints the marked save atomically before ready, without
waiting for another event. Refused restore leaves the prior file/session.

`savedEvents` and `carrySave` read event definitions from the restored
session's snapshot: freshly loading corrected source can legitimately fail
for a legacy-only program. This adds the cost of one full snapshot to these
helpers; it avoids introducing another public interface API. Sessions close
on success and errors. Raw save text is not parsed and reserialized by the
production restore adapter, preserving signed zero.

## Evidence

The ten raw saves in
[`runtime/tests/fixtures/module-bindings`](../../runtime/tests/fixtures/module-bindings/)
were generated through each published package's own public Node loader and
shipped WASM after checking tarball size and SHA-256 against its release
record. `provenance.json` records package and save hashes. The fixtures cover
identical initial saves, state after an event, both replay encodings, and a
legacy-only source whose corrected function call has the wrong arity.

Native regressions cover explicit selection, all three writers, conflicting
and malformed markers/options, cache interleaving, raw source identity,
unknown-state validation and subsequent events. The existing genuine rc.11
grounds fixture remains unchanged. An older D083 unit fixture depended on
root capture; it now asserts corrected fresh refusal and explicitly checks
the historical restore classification under the legacy interpretation.

Public kit tests cover both WASM builds, legacy-only save helpers, strict
options, signed-zero forwarding, session closure, serve state checkpointing
  and retention of the actual restore error when a fresh source load refuses.
Final command results and remote checks are recorded on the pull request.

## Scope limits

The original eleven D084–D088 reproducer programs and unpushed Claude patch
remain unavailable. These reconstructed fixes cannot establish that every
item in that original inventory is closed. No release, publication, broad
performance improvement or new full historical release sweep is claimed.
The bug is corrected for fresh sessions; retaining legacy semantics is an
explicit historical continuation, not the default behavior.
