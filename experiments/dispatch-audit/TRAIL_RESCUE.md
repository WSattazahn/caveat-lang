# Trail Rescue supplemental structured-outcome audit

This is a **new supplemental run on the new structured runtime**, not a rescore or reconstruction of the frozen historical results. Status: **PASS** (24/24 scenarios).

Run `node experiments/dispatch-audit/trail-rescue.mjs` after building the current WASM. It runs the exact 24 histories in `experiments/trail-rescue/scenarios.json`, through the unchanged `web/trail-rescue-policy.js` adapter and unchanged `game/trail_rescue.cav`. Their contents are checked against commit `3980c3d` before replay. Actual working-tree and baseline SHA-256 hashes are retained: the existing scenarios checkout differs only by CRLF versus Git LF, which is explicitly verified without editing the file. The other historical input files match baseline bytes. The original harness and adapter files remain unchanged. The original five additional checks, including its 200 × 80 seeded fuzz run, are outside this supplemental audit.

The delegate translates a returned core rejection into a private tagged exception solely to satisfy the unchanged adapter interface. Exceptions before any core call are `adapter_exception`, not core policy or structured host refusals. Every escaped core exception, protocol/JSON failure, or output failure stops the run as fatal; a failed core session is discarded without further save/view/snapshot probes. The legacy scenario reject booleans still check admission only. Actual structured origins below are separate evidence, not a retroactive policy interpretation.

| Scope | Dispatches | Accepted | Policy rejected | Input rejected | Evaluation rejected | Limit rejected | Adapter exceptions | Fatal |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Registered dispatch steps | 151 | 118 | 20 | 2 | 0 | 0 | 11 | 0 |
| All attempts including restored shadows | 178 | 135 | 30 | 2 | 0 | 0 | 11 | 0 |

Checked every declared projection and admission expectation, every refusal's save/view/snapshot and public projection atomicity, 11 explicit restores with shadow continuation, and 24 final restores. Accepted reports are compared to the complete current snapshot. The JSON captures the complete core reports, adapter exception messages, exact events, projections, restore/atomicity hashes, and scenario traces.

## Refusals in registered histories

| Scenario | Step | Observed category | Code | Message |
|---|---:|---|---|---|
| TR02 | 2 | core_policy_rejection | reject | A plan needs fresh, undisputed clear evidence. |
| TR03 | 1 | core_policy_rejection | reject | Choose a supported plan first. |
| TR03 | 2 | core_policy_rejection | reject | A plan needs fresh, undisputed clear evidence. |
| TR03 | 3 | core_policy_rejection | reject | A plan needs fresh, undisputed clear evidence. |
| TR04 | 3 | core_policy_rejection | reject | A plan needs fresh, undisputed clear evidence. |
| TR05 | 6 | core_policy_rejection | reject | Choose a supported plan first. |
| TR07 | 6 | core_policy_rejection | reject | Choose a supported plan first. |
| TR12 | 1 | adapter_exception |  | Error: Unknown event type. |
| TR12 | 2 | adapter_exception |  | Error: Unknown tunnel. |
| TR12 | 3 | adapter_exception |  | Error: Expected clear or blocked. |
| TR12 | 4 | adapter_exception |  | Error: Unknown observation method. |
| TR12 | 5 | adapter_exception |  | Error: Expected clear or blocked. |
| TR12 | 6 | adapter_exception |  | Error: Expected clear or blocked. |
| TR12 | 7 | adapter_exception |  | Error: A scout reads the tunnel itself. |
| TR12 | 8 | core_input_rejection | bound_exceeded | dt must be finite and in 0..30 |
| TR12 | 9 | core_input_rejection | bound_exceeded | dt must be finite and in 0..30 |
| TR12 | 10 | adapter_exception |  | Error: Expected a finite time step. |
| TR12 | 11 | adapter_exception |  | Error: Expected a finite time step. |
| TR12 | 12 | adapter_exception |  | Error: Unknown tunnel. |
| TR12 | 13 | adapter_exception |  | Error: Unknown tunnel. |
| TR13 | 4 | core_policy_rejection | reject | No scout tokens remain. |
| TR13 | 7 | core_policy_rejection | reject | No scout tokens remain. |
| TR14 | 2 | core_policy_rejection | reject | That visitor has already reported. |
| TR14 | 5 | core_policy_rejection | reject | That visitor has already reported. |
| TR15 | 4 | core_policy_rejection | reject | Reconsider the existing plan first. |
| TR15 | 5 | core_policy_rejection | reject | Reconsider the existing plan first. |
| TR18 | 5 | core_policy_rejection | reject | The rescue has ended. |
| TR18 | 6 | core_policy_rejection | reject | The rescue has ended. |
| TR18 | 7 | core_policy_rejection | reject | The rescue has ended. |
| TR18 | 8 | core_policy_rejection | reject | The rescue has ended. |
| TR18 | 9 | core_policy_rejection | reject | The rescue has ended. |
| TR18 | 10 | core_policy_rejection | reject | The rescue has ended. |
| TR18 | 11 | core_policy_rejection | reject | The rescue has ended. |

## Reproduction evidence

The complete JSON report is stored in [result-trail-rescue.json.gz](result-trail-rescue.json.gz). It contains 9,188,715 bytes of JSON in 456,068 compressed bytes; decompression was verified byte-for-byte after writing. Archive SHA-256: `dcb6c904eb353649b6a159ff782a958339cd812201173ecdbcc5909a2ced9b43`. All core reports, snapshots, events, and provenance remain in the archive. Read it from the repository root with:

```sh
node -e "process.stdout.write(require('node:zlib').gunzipSync(require('node:fs').readFileSync('experiments/dispatch-audit/result-trail-rescue.json.gz')))"
```

Trace SHA-256: `b00a5bff9253aeec15bf27d507c7e5ef138e68aba577c802d358f489b7b83b4d`. The JSON records the runtime JS/WASM hashes, build metadata, runtime source/spec hashes, script hash, source/adapter/scenario/harness hashes, and start/finish times. Restored shadow attempts are separated from registered steps to avoid inflating the scenario rejection count.
