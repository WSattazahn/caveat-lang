# Limits

What Caveat does not do, and what it costs. The kit README summarizes this
page; the numbers here are the record behind that summary.

## What it does not do

- **It does not check that evidence is true.** A caveat is what the program's
  author declares about a source. Caveat carries it through every computation
  and decision that uses the source, but it cannot tell whether the source is
  reliable, and it does not measure a model's confidence. The result is only
  as good as the sources and caveats the program declares.
- **It does not authenticate saves.** A save records the SHA-256 of the source
  it was made under, so restoring it against a different program is refused.
  It carries no signature: an edited save can drop a caveat, change a reading
  or remove a decision and still restore. Restore refuses only what no source
  mechanism could have produced; it does not repair a save or prove its
  history ([restore trust boundary](RESTORE_TRUST_BOUNDARY.md)). Keep saves
  where only your application writes them, or sign them yourself. The same
  holds for the event list a `caveat serve --state` file keeps.
- **It does not know who sent an event.** Your application decides who may
  send which event; `caveat-lang/handoff` helps keep an agent to its own
  events.
- **What is proved is a model, not the runtime.** Lean theorems cover a small
  mathematical model of the language. The Rust runtime that ships is checked
  by tests and by sampled agreement with that model, not by proof
  ([what is checked, and how](VERIFICATION.md)).

## What it costs

Measured 2026-10-25 on `caveat-lang@1.3.0` from npm, Node 22.22.0, Linux, a
4-core cloud container, with other work running. Treat these as orders of
magnitude, not benchmarks.

| Measure | Result |
| --- | --- |
| Package | No npm dependencies; the runtime is a prebuilt 3.6 MB WebAssembly file |
| CLI start (`caveat-lang --version`) | 0.35 to 0.64 s over three runs |
| `dispatch`, program below | 2,200 to 2,700 events per second over three runs of 2,000 events |
| `dispatchView`, program below | 4,900 to 6,200 events per second over the same runs |
| `validate`, N one-line `state` declarations | 2,500: 0.8 s; 5,000: 1.3 s; 10,000: 4.3 s; 20,000: 16.0 s |

Load time grows faster than linearly with the number of declarations, so a
very large generated program can take seconds to load and can exceed the MCP
authoring bridge's 10-second operation limit before its 1 MiB source limit.

Each event costs well under a millisecond on this program, which is far more
than the same rule written as plain application code. Caveat is meant for the
decisions an application records, not for inner loops.

The glowcap benchmark's round-6 replay program, a larger game policy, took
51.6 µs median per event plus view, measured after its fuzz runs, and its
policy, adapter and runtime came to 482,716 bytes gzipped (about 483 KB)
([results](../experiments/glowcap/RESULTS.md)).

Memory and save size grow with retained history. Window sizes alone do not
bound live memory or saves; see the
[history integration guide](../kit/docs/HISTORY.md). A host that opens many
sessions without closing them can exhaust the runtime's WebAssembly memory.

The throughput program, run with `node perf.mjs` beside an installed
`caveat-lang@1.3.0`:

```js
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
const src = `evidence e from "x";
claim c;
readings r from e window 64;
state total = 0;
event go v min 0 max 100;
on go sample r = v supports c;
on go set total = latest(r);
bind out.text = number_text(total);
`;
const rt = await loadRuntimeFromDirectory();
const s = rt.open(src);
for (const [label, fn] of [['dispatch', i => s.dispatch('go', { v: i % 100 })],
                           ['dispatchView', i => s.dispatchView('go', { v: i % 100 })]]) {
  const N = 2000; const t = performance.now();
  for (let i = 0; i < N; i++) fn(i);
  const dt = performance.now() - t;
  console.log(`${label}: ${N} events in ${dt.toFixed(0)} ms = ${(N / dt * 1000).toFixed(0)}/s`);
}
s.close();
```

The `validate` programs are `state s0 = 0;` through `state s(N-1) = 0;`
followed by `event go;`, one declaration per line, timed with
`npx --no-install caveat-lang validate`.
