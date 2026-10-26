# D026 corpus sweep

Evidence for the D026 fix ([Withdrawal 0.1](../../spec/caveat-withdrawal-0.1.md),
"Changes", 2026-10-26): a value read through `committed(D)`, `reopened(D)` or
`in_basis(D, E)` now carries `withdrawn` when the decision's basis lists
withdrawn evidence.

`sweep.mjs` runs every tracked Caveat program that loads on its own on two
runtimes with the same seeded events (20 runs of 30 events each), and compares
the outcome and snapshot after every event. It fails on any difference other
than `withdrawn` appearing in a caveat list, in a commitment's retained
caveats, or as a `retains withdrawn` relation.

```sh
npm install caveat-lang@1.4.0 --prefix /tmp/caveat-1.4.0
npm run build
node experiments/d026-sweep/sweep.mjs /tmp/caveat-1.4.0/node_modules/caveat-lang/runtime dist/pkg-reactive --write
```

[`RESULTS.json`](RESULTS.json), run 2026-10-26 against the published 1.4.0
runtime: 181 programs run (31 are modules or need their imports linked),
168 identical, 13 differ only by `withdrawn`, none otherwise, and no event
accepted by one runtime and refused by the other.

Where the 13 differ:

- a display or state read through one of the three predicates after the
  withdrawal, as in `kit/examples/intel-betrayal/intel.cav`;
- a decision revised after a withdrawal reopened it. The new revision's
  lineage holds its predecessor's basis, so it carries `withdrawn` there
  (`explain` prints "also carrying, through its lineage: withdrawn"), as it
  already carried any declared or late caveat of that basis. Its grounds, and
  so `rests_on_withdrawn`, are unchanged. `caveat-lang demo agent` is one:
  its invariant in `kit/lib/demo.mjs` now expects it.
