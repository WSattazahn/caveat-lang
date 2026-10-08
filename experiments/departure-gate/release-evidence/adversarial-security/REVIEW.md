# Bounded adversarial pass: no defect found in the executed cases

**Keep the frozen experimental candidate. This pass found no violation of its tested restore, withdrawal-retention or archive-reporting requirements, and supplies no reason to undo the collector or early-boxing improvement.** It does not authorize merge or release. The exact Muse findings remain an explicit coverage gap: their original probes and expectations were not available, so this report does not claim to replay F256, F358, F362, F363 or F364.

The candidate is **unpublished** `590fae59aa4295ac5209f930a1e58ac152bd4603`. The comparison cell is the published `caveat-lang@0.1.0-rc.15` artifact, source revision `3a88ba0f80d563b4493840dc7bd7e195329b8302`. Both were executed with their own version-matched JavaScript wrappers and WASM. No runtime source, retention rule, serialization schema or existing performance result was changed.

## Pass/fail coverage

| Executed family | Result | What the result establishes |
|---|---:|---|
| New marker/archive probes (M/A) | **103/103 passed** | Genuine marker/journal saves restore; malformed markers refuse; missing or inconsistent archive dependencies stay unavailable; valid unrelated components remain usable. |
| New withdrawal probes (W01–W20) | **40/40 cell cases passed** | Shared guarantees hold in both versions, with explicitly different departure expectations. The 20 comparisons of shared result fields also match. |
| New mixed post-collection probes (P) | **14/14 passed** | A collected component can coexist with an independently required withdrawal; malformed edits refuse; authentic and older saves restore and continue transactionally. |
| Four existing repository contract files | **30/30 tests passed** | Current archive membership, historical reporting, conservative markers and documented restore trust boundaries. |
| Exact Muse F256/F358/F362/F363/F364 | **Not run** | Original source, mutation inventory and expected outputs are unavailable. New IDs are not substitutes for historical findings. |

The 157 new result rows include repeated drain/restore and version combinations; they are not 157 independent attacks or a statistical security estimate. The additional 20 W comparisons reuse those same observations. The [source coverage map](coverage-source.md) identifies existing tests and the missing intersections that motivated the new cases. It distinguishes inspected test source from tests executed here.

## What the adversarial cases exercised

**Markers and archive closure.** Thirteen candidate malformed-marker saves were refused with a restore error, including invalid history, endpoints, counts, time, duplicate histories, grounds outside basis and malformed reference syntax. Two accepted controls demonstrate a different boundary: a syntactically valid but unresolved reference, or an omitted optional reference, can restore while explanations remain explicitly conservative. An archive reference is not a credential or authenticated history.

The archive tests start from real simultaneous departures: `a@2` retains its original self-withdrawal; `r@1` depends on it; `z@1` is an unrelated valid component. Twenty-one archive variants run across immediate/delayed draining and live/restored snapshots. All 72 negative evaluations report the affected closure unavailable and omit its raw historical payload; all retain `z@1` as complete with value 7. Missing records/nodes, conflicting duplicates in both orders, invalid withdrawals, sequences, relations and nested provenance were covered. Twelve positive evaluations accept original archives, identical reversed duplicates and deliberately coherent payload edits. Every report retains `authenticated: false` and the scope “provided records and their referenced closure.” The archive fixture has no live decisions, so its unchanged-decision assertion is not a nonempty decision-policy test.

**Required reasons and write-once withdrawal.** The W suite refuses twelve malformed withdrawal-save variants in each version. Repeating a withdrawal retains its first concrete reason, event and sequence. Policy and final-binding refusals preserve the exact save and snapshot. Tests restore while a historical reading still needs an old reason, then actually read it and check its withdrawn/stale qualifications and own grounds before releasing the holder. A four-cycle reachable chain retains its distinct reasons through restore and continuation. Required chains were not shortened to make collection succeed.

**Mixed collection and older saves.** The P fixture actually collects isolated `a@2` while state own grounds retain `b@2` and its original withdrawal; a genuine reading departure marker survives in a later commitment. Ten independent malformed edits of this save refuse, including withdrawn endpoints replaced by departed records and one-sided ledger/graph omissions. Valid-save controls verify exact same-version restoration and failed final-binding rollback, including the draft's existing undrained archive. An authentic rc.15 save restores on the candidate without collecting on load; a rejected event does not migrate it; the next accepted event collects the eligible component while preserving the required one.

## What is a contract difference, and what remains a trust limit

Rc.15 retains retired records and has no departure/archive API. The candidate may legitimately remove eligible records from current snapshots and place their original withdrawal relationships in the archive. The tests classify that approved lifecycle difference separately from a lost reason or reversed withdrawal. They do not demand blanket snapshot equality between versions.

Source-possible edited withdrawal metadata and coordinated removal of a withdrawal record plus its graph membership are accepted controls under the documented restore trust boundary. Likewise, a consistent supplied archive payload edit can remain “complete” within the visible closure. These are not newly discovered authentication bypasses: neither API promises authenticated or exhaustive history. No result here establishes resistance to prompt injection, credential disclosure or unauthorized agent tool use; those were outside this runtime-only investigation.

## Evidence, preparation failures and limits

The final successful commands, exact argv, raw streams and before/after artifact checks are under `commands/`. The archive/marker output is `archive-marker-results-001/`; W conclusions use `withdrawal-results-003.json` and its artifact directory; P conclusions use `results/post-collection-002/`. Original save strings, edited saves, source, archive inputs, reports and runtime identities are retained for concrete reproduction.

Preparation failures are preserved, not counted as security refusals. W attempt 001 had 40 fixture-load failures from unsupported comment syntax. Attempt 002 passed six cases but had 34 load failures because the “unobserved” fixture declared evidence that no source rule could ever observe. Attempt 003 corrected those fixtures, kept all expectations, and passed all 40. P's first command failed before runtime loading because its output parent directory was absent; its next command passed all 14 cases. A pre-execution P wrapper correction explicitly gated rc.15's unsupported archive API. No runtime was repaired to obtain these results.

Cell provenance checks cover the candidate kit files from the exact Git tree, preserved ordinary WASM build identity, and rc.15 tarball SHA-256/SHA-512 against the recorded official publication. This is local verification against that publication record, not a new registry or attestation lookup. Both artifacts remained unchanged. These are WASM correctness checks, not fresh Rust builds or native timing measurements. No full native/browser suite or new CI run is claimed.

The recommendation is bounded: retain the candidate and its existing performance caveats; no semantic repair is indicated by this corpus. Obtain Muse's original five probes before claiming that specific historical inventory is covered. Broader memory costs and the measured successful-release latency tradeoff remain open engineering concerns. PRs #174–#177 and the completed performance evidence were left unchanged; nothing was pushed, merged, published or released.
