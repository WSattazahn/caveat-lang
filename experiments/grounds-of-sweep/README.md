# Grounds Of sweep

Evidence for [Grounds Of 0.1](../../spec/caveat-grounds-of-0.1.md), the 1.5
draft that adds `grounds_of(SERIES)`.

`node experiments/grounds-of-sweep/sweep.mjs [--write]` reads every tracked
`.cav` file as text and writes [`RESULTS.json`](RESULTS.json). It fails if any
tracked program mentions `grounds_of`. Such a program is the only kind whose
meaning the profile could touch: before it, `grounds_of(...)` loaded only as a
call to a program's own `fn grounds_of`, which keeps its meaning. None does.
The results also list the programs that read a decision series with
`latest(SERIES)`, where the new form would stand; each keeps its meaning,
since `latest` is unchanged.

The save sweep (`runtime/examples/save_sweep.rs`) over every tracked program,
run on `main` before the change and on the branch, gives the same digest for
every program.
