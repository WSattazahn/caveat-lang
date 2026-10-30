# Findings from the 2026-10-28 PR recovery

These findings came from refreshing and testing open pull requests. They are
recorded as `find` events, without recording an owner approval, a merge, a
release, or a green GitHub Actions run. Local corrective revisions below name
observed source in the recovery checkout; subsequent published review heads
may have different Git identities. Passing focused checks do not establish a
complete release or merge gate.

## D030 — retired graph nodes retain storage in unmerged PR #382

**Scope:** the proposed `IdTable` in [PR #382](https://github.com/WSattazahn/caveat-lang/pull/382),
refreshed locally on main as `cbefc5ded5dea7082b1cd006b1f6b33f0968ea5f`.
This regression is in an unmerged proposal, not published 1.4.1.

Graph node IDs increase monotonically. The proposed table indexed a chunked
list directly by ID; removal replaced an entry with a tombstone but retained
its slot. A graph with few live nodes therefore kept storage proportional to
all node IDs it had ever issued, and copying or iterating the table visited
that retained history.

The new regression in `runtime/tests/graph_storage.rs` warms up 1,000 IDs,
then issues and retires 10,000 more while retaining four live nodes. It counts
live allocations rather than elapsed time. Against the original proposed
table, this retained **912,680 additional bytes** and failed a 64 KiB slack
bound. The same test passes after local corrective commit
`27058be7e66678282c90a27b6b992923e2f7637c`, which restores main's existing
`HashMap` graph storage and retains the independent chunked journal change.
The test also checks that IDs remain monotonic and the last four nodes exist.

The observed command was:

```sh
cargo test --manifest-path runtime/Cargo.toml --test graph_storage -- --nocapture
```

The earlier combined graph/journal speedup measurement does not describe this
corrected implementation. No replacement performance improvement is claimed.
This regression bounds the tested small-live-graph workload; it is not a
universal bound on provenance, journals, archives, or all runtime memory.

## D031 — the ledger replay gate can accept a failed process

**Scope:** repository verification, specifically `ledger/replay-check.mjs` on
main `66d13a0269b3a3f04e208f1906ea0453d341ab06`. This is not a language
semantic or a published-runtime finding.

The local execution sandbox returned a `spawnSync` result with `status: 0`, an
`EPERM` process error, and empty output. The gate checked the status but not
the error, filtered an empty result into zero events, and incorrectly printed
`0 recorded events replay.` as success. That observation was discarded;
subsequent historical replay checks ran with functioning subprocess execution
and reported the actual event counts.

Local corrective commit `e32021cdf0d0294adb06ce63c772519bf13859fb` requires
successful process execution, an initial record, the expected number of accepted event records, and exit zero. Four regression tests cover the observed
status-zero process error, signals, failed exits, empty/partial/extra/malformed
output, unknown records, refusals, and fatal records. They pass with actual
subprocess execution:

```sh
node --test ledger/replay-check.test.mjs
```

The guard detects missing or unsuccessful local execution. It does not
authenticate historical CI claims or establish that every other repository
check correctly interprets its own subprocess results.

## D032 — the project ledger displays 1.4.1 as 1.4.0

**Scope:** the display binding in `ledger/project.cav`, not a missing release
or an incorrect `ship_minor` event. Main already records version 141 for
1.4.1. Its binding rounded `(version - 100) / 10` and appended a literal
`.0`, so the displayed release was `1.4.0 stands`.

The added L09 scenario fails before correction with expected `1.4.1 stands`
and actual `1.4.0 stands`. Local corrective commit
`5d930a7f4858cc5f5ce93b9e2ecbedc510eb26f2` renders the integer minor and patch
digits separately. L09 checks 1.4.0, 1.4.1, and 1.4.5 and verifies that the
three commitment grounds remain attached to their original main-run
evidence. **1.4.5 is a synthetic scenario value, not a recorded release.**

After correction, `npm run test:ledger` passed four replay-guard tests and
nine scenarios and replayed all 193 main events, displaying `1.4.1 stands`.
The correction did not rewrite `events.jsonl`. L04's documentation now says
that it preserves the historical prefix through 1.3.0; the replay checker
checks the complete growing record.

## D033 — native touch delivery can lag the rescue test's fake clock

**Scope:** the browser test driver in `scripts/test-rescue.mjs`, inherited by
[PR #389](https://github.com/WSattazahn/caveat-lang/pull/389). This finding is
not a demonstrated game-physics defect or a reproduction of the historical
CI failure that finished with two hull points instead of three.

A Chromium CDP touch-dispatch promise can resolve before the page receives
its native pointer event. The old driver could then advance the fake game
clock through up to 30 simulation ticks while the source still held the old
aim. Local corrective commit `0c4f8b5989950496c98b1ca6d119508d15d1315d`
waits for the actual pointer delivery while the fake clock stays paused,
then asserts that the source aim matches before advancing it. The corrective
commit changes the test driver only.

For the negative control, the original PR source
`6c71679578685db5a2d1fd7743791e6eea78bdc3` received only the new pre-clock aim
assertion and selected the native-mobile route. Running the derived
`rescue-negative-control.mjs` exited 1 with:

```text
The pointer aim must reach the source before advancing the game clock
```

A short positive probe with pointer acknowledgement exited zero. The corrected
canonical Chromium run subsequently passed the native-mobile harbor route
with all original expectations: 32 rescued, three hull points, three
readings, four revisions, and retry. The command for the complete Chromium
suite is:

```sh
BROWSERS=chromium node scripts/test-rescue.mjs
```

Those observations use the task's Playwright Chromium installation on Linux.
The historical CI hull discrepancy has **not** been reproduced locally, and
the confirmed input-delivery race alone does not prove its cause. Full-suite
receipts and WebKit results must be assessed separately from these focused
controls; neither is a claim about physical Safari hardware.

## Follow-up — 2026-10-29

The observations above describe the recovery work on 2026-10-28. This follow-up
records subsequent review and delivery without replacing those observations
or their failures. The four `find` events in this report's ledger change do
not assert a new release, approval, merge event, or CI event.

### D031 and D032: corrected in merged #393

[PR #393](https://github.com/WSattazahn/caveat-lang/pull/393) merged as
`bb4755c81d0d8ac894be283ac3c3511bf8773915` from reviewed head
`91d78adcb2851587d11c74fd52b6e1103f1e05d6`. Further review strengthened the
replay guard to match physical input lines, event names, payloads, ordering,
sequences, and corresponding ledger snapshots. The same-count duplicate and
reordered-output controls reproduce a defensive validation gap; they do not
show that the production CLI emitted such output.

At that head, all seven replay-guard tests and nine ledger scenarios passed;
the complete 194-event record replayed and displayed `1.4.1 stands`. The
original 193-event prefix remained unchanged. All 26 applicable CI checks
passed before merge; Legacy Door 3D QA was conditionally skipped. The
[runtime workflow](https://github.com/WSattazahn/caveat-lang/actions/runs/40947695421)
and [Carmack workflow](https://github.com/WSattazahn/caveat-lang/actions/runs/40947695164)
retain those CI observations. The four-test and 193-event results above remain
the earlier local observations, not the final merge results.

### D033: test-driver correction included in merged #389

[PR #389](https://github.com/WSattazahn/caveat-lang/pull/389) merged as
`2b8096f6d9adb7696949a9193ce63ee7d808615b` from reviewed head
`3ac0eef0a97a5898ce5437afe3b602e56f819cf9`, after refreshing against the merged
#393. It retains the native-touch delivery correction and separately fixes
break-page recovery and stale asynchronous completions.

The final review passed all 32 controlled break-page recovery cases locally
on Windows (Chromium 156.0.8078.4 and WebKit 27.2) and in CI. The complete
Chromium and WebKit rescue suites also passed locally and in CI. The final
[Chromium rescue job](https://github.com/WSattazahn/caveat-lang/actions/runs/40955494837/job/123737450469)
and [WebKit rescue job](https://github.com/WSattazahn/caveat-lang/actions/runs/40955494837/job/123737450461)
each ran the suite once without a test retry. The
[Carmack recovery job](https://github.com/WSattazahn/caveat-lang/actions/runs/40955494851/job/123735946319)
ran all 32 controlled cases. All 26 applicable CI checks passed before merge;
Legacy Door 3D QA was conditionally skipped.

An earlier local Chromium attempt at source tree
`68602ebf1855daeb017612fd9aaaa69e2273406b` timed out in the unchanged desktop
screenshot capture after 20 seconds, before reaching mobile. That failed
local assessment is retained; the later passing runs are separate evidence.
The original two-hull CI failure still has no conclusively demonstrated
cause. The confirmed input-delivery race is not proof of its cause, and
neither the controlled recovery tests nor the rescue suites establish
physical Safari/device behavior or natural WebAssembly trap frequency.

### D030: the corrected proposal remains unmerged

At this review, #382 remains open at
`9f96f76db2a47db4bd16affd954ddb3c8c312246`, retaining the journal change and
restoring `HashMap` graph storage. Its original combined graph/journal
performance claim remains withdrawn. No new speedup measurement or merge
of that proposal is asserted here.
