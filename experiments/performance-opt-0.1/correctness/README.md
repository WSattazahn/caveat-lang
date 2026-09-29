# The view path changes no behaviour

`dispatch_view_outcome` (natively `dispatch_view_outcome_json`) and the kit's
`session.dispatchView` are additional entry points
([Dispatch 0.1](../../../spec/caveat-dispatch-0.1.md#the-view-path)). These
checks show that they return what the existing paths return, and that the
existing paths are unchanged. They are correctness evidence, not
measurements.

| Check | Where | What it requires |
| --- | --- | --- |
| Every refusal, on both paths | the `rejected` helpers of `runtime/tests/dispatch_outcomes.rs` and `scripts/test-dispatch-outcomes.mjs`; `kit/test/dispatch-view.test.mjs` | each refusal those tests make goes through the view path first: the same text, byte for byte, and the save, snapshot and view unchanged. Together they reach every code of Dispatch 0.1 but `limit/depth_limit`, which no loadable program reaches; the runtime's unit test checks that both paths classify every code, that one included, alike |
| Fatal outcomes | the same files | the same thrown report, and in the kit the same `CaveatError`, after which the session is unusable; traps mark the runtime trapped |
| Differential | `kit/test/dispatch-view-differential.test.mjs`, part of `npm run test:kit` | twins, one through `dispatch()` and `view()` and one through `dispatchView()`, over every tracked program that opens (80) with 120 seeded random events each, and the Glowcap replay (10,000 events), Glowcap with 16 mushrooms (2,000), agent ledger (20) and Trail Rescue (140) streams: the same outcome, view, save and snapshot after every event |
| Existing exports unchanged | `legacy-identity.mjs`, `legacy-identity.json` | a build of the base, main at 768275b, against a build of this branch: `dispatch_outcome`, the legacy `dispatch_view` and the legacy `dispatch` return or throw the same text, and `snapshot()`, `view()` and `save()` after every event are the same text, in both builds (`pkg` and `pkg-reactive`), over the random streams and the Glowcap, ledger and Trail Rescue streams |

`legacy-identity.json` names each build by the sha256 of its WebAssembly. To
repeat it, build the base and this branch with `npm run build` and run:

```sh
node experiments/performance-opt-0.1/correctness/legacy-identity.mjs \
  --base=PATH/TO/base/dist --out=legacy-identity.json
```
