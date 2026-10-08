# NEW withdrawal probes, frozen candidate versus published rc.15

Final attempt `withdrawal-003` passed **40/40 cell expectations and 20/20 shared differential projections**. This finite pass found no withdrawal/restore regression in its tested cases. No runtime source was changed. It does not reproduce F256/F358/F362/F363/F364: those exact sources and expectations were unavailable when these NEW W01–W20 probes were frozen.

Candidate: `590fae59aa4295ac5209f930a1e58ac152bd4603`, loaded reactive WASM SHA-256 `52e17a54eb1be7f17aad6fc51be3a6a2db3308fad6970a817b59547c51c99cbe`. Published rc.15: `3a88ba0f80d563b4493840dc7bd7e195329b8302`, WASM `29edda09e93f61381049b5bb614a77d74bf22873ca37fef40da014dbe2fed577`. Each cell used its version-matched kit wrapper and explicit runtime directory. The outer command receipt verified every cell file before and after execution; `runtimeInputsStable=true`. Candidate and rc.15 build metadata identify different compiler hosts (Windows and Linux); this is a behavioral WASM comparison, not a timing or native-build comparison.

| NEW probes | Expected class and observed result in both cells |
|---|---|
| W01 | Unmodified withdrawal save restored exactly; continuation matched. |
| W02–W05 | Unknown or unobserved subject/reason rejected with `kind=restore`; observed diagnostics identified unknown evidence or unobserved evidence. |
| W06–W10 | Duplicate target, zero/future sequence, undeclared event, and declared event with no matching reachable withdrawal rejected with `kind=restore`. |
| W11–W13 | Missing withdrawn edge, orphan withdrawn edge, and unknown withdrawal-record field rejected with `kind=restore`. |
| W14 | Accepted source-possible reason/event substitution even though the source guard was false; subsequent predicate used the edited `second` reason. This is the documented unauthenticated-history limit. |
| W15 | Accepted coordinated omission of both withdrawal record and withdrawn graph edge. This is the documented unauthenticated-history limit. |
| W16 | Repeated withdrawal before and after restore kept the first record exactly (`subject`, `reason`, sequence 2, event `pull`), emitted no repeated withdraw effect, and retained the original reason's `stale` qualification. |
| W17 | Both failed transactions preserved snapshot and exact save; candidate archive stayed empty. Raw outcomes were `policy/reject` and `evaluation/expression`. The assertions enforce rejection and exact rollback; the two origin/code labels are recorded observations and shared-projection equality, not separate expected-code assertions. A subsequent valid withdrawal succeeded. |
| W18 | Two own-ground holders retained self-reason `a@2`; restore retained the last holder and exact record. Refused/failed last-root releases rolled back. Final accepted release produced the expected version-specific lifecycle below. |
| W19 | Restored **while the unread historical holder still pinned `a@2`**. Later `latest(kept)` produced value 1 with lineage and own grounds `[a@2, kept@1]`, both carrying `stale` and `withdrawn`. Clearing the state and replacing the historical holder produced the expected lifecycle below. |
| W20 | Four cycles retained all eight distinct withdrawal records in the required alternating reason chain, including `b@5 because a@5`. Restore and continuation matched; `withdrawn(b)` carried the actual reason `a@5`, then `a@6` after another cycle, with `stale` and `withdrawn`. Candidate emitted no archive for this required chain. |

Every malformed-save case first restored its unmodified control and compared its snapshot. An arbitrary load error or runtime failure could not satisfy a rejection expectation.

The approved collection differences were explicit:

| Probe | Candidate after final root/holder removal | Published rc.15 |
|---|---|---|
| W18 | Archived exactly `[a@2]`, with original withdrawal `{evidence:a@2, because:a@2, sequence:1, event:cycle}`. `a@2` left active retired history. | Kept `a@2` and its original withdrawal in retired history. No archive API. |
| W19 | Archived exactly `[a@2, kept@1]`, preserving the original withdrawal and reading provenance referencing `a@2`. | Kept the old reason/history. No archive API. |

Within candidate, uninterrupted versus restored continuation produced equal ordered archive arrays. Shared differential projections compare live values, actual reasons, own grounds and caveats; they deliberately do not demand that rc.15 have candidate-only departure markers or an archive API. Root-owned archive completeness probes are separate.

## Preserved failed preparation attempts

These are harness failures, not runtime security defects or successful invalid-save rejection evidence. Their outputs were never overwritten or relabeled:

- `withdrawal-001`: all 40 cell attempts failed at source load because four new fixture headers incorrectly used unsupported `--` comments. Original driver, fixtures and plan are in `withdrawal-attempt-001-inputs/`; all outcomes and source copies remain under `withdrawal-results-001.json.artifacts/`.
- `withdrawal-002`: removed those headers; W18–W20 passed in both cells, while W01–W17 failed source validation because `unseen` had no source rule that could observe it. Inputs/plan are in `withdrawal-attempt-002-inputs/`; outcomes remain under `withdrawal-results-002.json.artifacts/`.
- `withdrawal-003`: added an **unexecuted** `observe_unseen` event/rule. Thus the evidence remained unobserved in every saved history while becoming source-observable, as the intended malformed-save probe requires. All W expectations and mutation rules remained unchanged. Plan 003 was frozen before these 40 attempts; all passed.

Across preparation and final capture, 120 cell attempts are preserved: 74 fixture-load failures and 46 passes. Final-runtime conclusions use the successful 003 population; 001/002 do not establish the malformed-save guarantees they never reached.

## Replay and evidence

From this directory, use a **fresh** output path and recorder name:

```text
python -B record-run.py withdrawal-replay -- node tests/withdrawal-probes.mjs --root . --plan withdrawal-plan-003.json --out withdrawal-results-replay.json
```

The driver checks the frozen inventory, input hashes and complete cells manifest before loading. Every probe gets a fresh runtime instance. Per-probe source, restore input, save, snapshot, dispatch outcome, error and available archive drain are retained; an append-only attempt ledger and raw command stdout/stderr accompany the summary. Existing output paths are refused.

| Artifact | SHA-256 |
|---|---|
| `withdrawal-plan-003.json` | `4fc7011e7914b3874ff5f9f61337c2dec20834aa4b75e57b9c31dfcf1a622816` |
| `tests/withdrawal-probes.mjs` | `be48960858e5eafcd28792a766faa62c63d982c735901b111d254b072cfa2e82` |
| `withdrawal-results-003.json` | `57a5bb81107e9e1a04c0ed29362f44828240d00195517a9df0e408ced1d7b8a7` |
| `withdrawal-results-003.json.artifacts/attempts.jsonl` | `5a3337069567bb45ef184156ef2794895bf79a26a72ff0825f1fe45fee68e06c` |
| `commands/withdrawal-003.json` | `c1e7ec63b5f9ac899b77202d909c80248ad7dcb17086794b229845e5908f005e` |

The expectation sources and relevant existing tests are listed in the frozen plan. This is bounded synthetic runtime evidence, not an authenticated-history guarantee, exhaustive adversarial coverage, performance result, published candidate claim, or authorization to merge/release.
