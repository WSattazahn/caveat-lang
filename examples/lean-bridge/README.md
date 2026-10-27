# Claims that rest on a Lean proof

A small example for the [Lean bridge design note](../../docs/design/lean-bridge.md).
Three results rest on lemmas: `checked_result` on a theorem Lean has fully
checked, `assumed_result` on a theorem whose proof assumes a result for now
(as large formalizations do with established results not yet formalized), and
`combined_result` on the first theorem and a model's unformalized proof
sketch.

| File | What it is |
| --- | --- |
| `LeanBridgeDemo.lean` | `lt_two_pow` (every natural number is smaller than two to its power), the assumed `Assumed.two_pow_add`, and `add_lt_two_pow_mul`, which uses both. |
| `lean-grounds.json` | Which evidence names which theorem, the pinned SHA-256 of each statement and each assumption, and the events to send. |
| `grounds.cav` | The program: the two Lean lemmas, the model's lemma with an `unchecked` caveat, `assumed_for_now` for a lemma that still assumes a result, three results, withdrawal, take-back and reopening. |
| `grounds.scenarios.json` | What must happen when a check passes, fails, reports an assumption or reports it proven, when an assumption or the model's lemma is refuted, and when a fixed proof is checked again. |

From the repository root:

```
npm run verify:lean-grounds   # needs Lean 4.34.1 on PATH or CAVEAT_LEAN_BIN
npm run test:lean-grounds
npm run test:lean-bridge      # needs the built runtime (npm run build)
```

`verify:lean-grounds` writes `test-results/lean-grounds/events.jsonl`, the
events a host would send the program, and `report.json`, and runs seven
controls that change the source and must each get their intended result. The `Lean scaffold`
workflow runs it on every pull request; `Runtime` runs the scenarios.

What the results say after both checks pass (the second with one assumption),
a model claims its lemma and the results are derived:

```
assumed_result@1 = null  in force
  based on product_lemma (caveats: assumed_for_now)
checked_result@1 = null  in force
  based on lean_lemma
combined_result@1 = null  in force
  based on lean_lemma, model_lemma (caveats: unchecked)
```

A failed check of `lt_two_pow` (`lean_failed`) reopens `checked_result` and
`combined_result`. A disproved assumption (`refute_assumption`) reopens only
`assumed_result`, and a counterexample to the model's lemma (`refute_model`)
only `combined_result`. A check that reports the assumption proven
(`product_checked` with `assumed` 0) takes the caveat back: later results rest
on the lemma without it, and `assumed_result@1` keeps what it rested on.
