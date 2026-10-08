# Retained adversarial regressions

These unchanged synthetic sources and the authentic rc.15 save come from the
bounded adversarial pass on `590fae59aa4295ac5209f930a1e58ac152bd4603`.
`provenance.json` records their hashes, original driver hashes, runtime identities
and the four events that produced the older save. The save text is copied without
JSON normalization. The original source bundle and execution receipts remain
separate evidence; these tests do not replace those receipts.

`adversarial-archive.test.mjs` retains candidate-side M00-M15 and A00-A20 across
immediate/delayed drain and live/restored snapshots. `adversarial-withdrawal.test.mjs`
retains candidate-side W01-W20. `adversarial-post-collection.test.mjs` retains
P00/P01, P10-P19 and rc.15-to-candidate P02. P00/P01 share a positive-control test.
The original same-version rc.15 execution and two-version W comparison remain
historical receipts; this suite uses the current built runtime and the frozen
older save, without downloading a historical package during tests.

Run from the repository root after `npm run build`:

```
node --test kit/test/adversarial-archive.test.mjs kit/test/adversarial-withdrawal.test.mjs kit/test/adversarial-post-collection.test.mjs
```

`npm run test:kit` discovers them automatically. They assert malformed-input
refusal, valid controls, transactional rollback, drain independence, exact reasons
and conservative archive closure. They deliberately preserve accepted controls
for source-possible save edits and coherent supplied archive edits: consistency
does not authenticate history. The archive fixture has no live decisions, so
its unchanged-decision assertion does not establish a nonempty policy case.

Muse F256/F358/F362/F363/F364 original probes and expectations were unavailable.
These independently derived IDs do not claim to reproduce those findings. Their
absence alone is not a failed release requirement; reconsider a concrete supplied
failure or material uncovered case. This suite is not a prompt-injection, host
permission, performance or security-certification test.
