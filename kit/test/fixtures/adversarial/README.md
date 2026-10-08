# Retained adversarial regressions

The seven original synthetic sources and the authentic rc.15 save come from the
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


## Sequence fixtures and arbitrary-event sweep derivatives

`marker-journal.cav.txt` and `post-collection.cav.txt` preserve the original
source bytes exactly. The direct M/A/P regressions load those files, and the
authentic rc.15 save remains paired with `post-collection.cav.txt`. Their original
hashes are unchanged. The `.txt` suffix explicitly marks a sequence-bound source
fixture whose test driver supplies its valid protocol, rather than a standalone
program intended to accept every random sequence of its declared events.

The corresponding `.cav` programs remain in every tracked-program consumer.
Their sole edit is `on decide when not committed(trust) commit ...`, guarding the
plain, single-use commitment. Without that authored guard, `record/start`,
`decide`, `decide` ends with the unclassified `commitment trust already exists`
error in both rc.15 and rc.16. The failed integration sweep correctly exposed
that fixture protocol mismatch. A decision *series* has a different revision
contract; this repair does not change either runtime behavior or error catalog.

`provenance.json` ties each derivative to its original bytes and exact string
replacement. `adversarial-sweep-fixtures.test.mjs` checks those relationships,
the original fatal control, and accepted repeated decisions on the derivatives
before and after restore. The arbitrary-event consumers continue to load the
`.cav` versions through their unchanged file discovery:

- `experiments/departure-gate/sweep.mjs`: full 32-seed, 400-event windowed outcome
  sweep against rc.15; `fatal=false` remains mandatory for every row.
- `scripts/test-interface.mjs`: native/full/reactive interface comparison.
- `kit/test/dispatch-view-differential.test.mjs`: seeded dispatch/view comparison.

There is no new skip, path exclusion or weaker gate. The retained original
historical ZIP and the first integration revision remain unchanged. Run the
focused fixture checks with `node --test kit/test/adversarial-sweep-fixtures.test.mjs`
after the normal build; the full kit suite discovers them automatically.
