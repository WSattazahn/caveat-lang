# Claims that rest on a Lean proof

A small example for the [Lean bridge design note](../../docs/design/lean-bridge.md).
Two results rest on lemmas: `checked_result` on a theorem Lean has checked,
`combined_result` on that theorem and a model's unformalized proof sketch.

| File | What it is |
| --- | --- |
| `LeanBridgeDemo.lean` | The theorem: every natural number is smaller than two to its power. |
| `lean-grounds.json` | Which evidence names which theorem, the statement's pinned SHA-256, and the events to send. |
| `grounds.cav` | The program: Lean lemma, model lemma with an `unchecked` caveat, two results, withdrawal and reopening. |
| `grounds.scenarios.json` | What must happen when the check passes, when it fails, when the model's lemma is refuted and when a fixed proof is checked again. |

From the repository root:

```
npm run verify:lean-grounds   # needs Lean 4.34.1 on PATH or CAVEAT_LEAN_BIN
npm run test:lean-grounds
npm run test:lean-bridge      # needs the built runtime (npm run build)
```

`verify:lean-grounds` writes `test-results/lean-grounds/events.jsonl`, the
events a host would send the program, and `report.json`, and runs four
controls that break the theorem and must each be refused. The `Lean scaffold`
workflow runs it on every pull request; `Runtime` runs the scenarios.

What the results say after the check passes, a model claims its lemma and
both results are derived:

```
checked_result@1 = null  in force
  based on lean_lemma
combined_result@1 = null  in force
  based on lean_lemma, model_lemma (caveats: unchecked)
```

A failed check (`lean_failed`) reopens both. A counterexample to the model's
lemma (`refute_model`) reopens only `combined_result`.
