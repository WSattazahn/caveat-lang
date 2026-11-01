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

## Worst cases

How expensive one event can get, and which cap stops it. Every cap below is
per session or per event and is checked inside the event's transaction, so
reaching one never leaves half an event applied. Times were measured
2026-10-25 on the WebAssembly build from main in the same container, and are
orders of magnitude only.

**One withdrawal's reach.** A withdrawal adds the `withdrawn` caveat to every
current value whose lineage or grounds hold the evidence
([Withdrawal 0.1](../spec/caveat-withdrawal-0.1.md)), so it can touch every
value the session holds. That is bounded by the program: its states and
bindings are fixed when it loads, reading streams and decision series hold at
most 1,024 records between them ([Reactive 0.5](../spec/caveat-reactive-0.5.md)),
and windowed histories hold at most 65,536 ([compatibility](COMPATIBILITY.md#limits)).
A withdrawal reopens nothing by itself. Each reopening is an authored step,
from a rule or a declared trigger, and is charged to the event's budget of
4,096 steps, past which the event is refused as `limit/work_limit`. A
reopening does not set off further reopenings: triggers run only when a
reading is sampled ([Reopening Triggers 0.1](../spec/caveat-reopening-triggers-0.1.md)).
So one withdrawal reopens at most as many decisions as the program declares,
and at most 1,024. With 500 decision series all resting on one piece of
evidence, the event that withdrew it and reopened all 500 took about 68 ms;
with 100 it took about 5 ms. The cost grows faster than the number of
decisions.

**Lineage growth.** Lineage only grows by union, so a value can rest on at
most every name the session holds, and the runtime caps it at 1,024
identifiers and 65,536 bytes of names
([Reactive 0.4](../spec/caveat-reactive-0.4.md)). Departed windowed
records count as one marker per history
([Lineage Compaction 0.1](../spec/caveat-lineage-compaction-0.1.md)). A
state that adds two new renewable occurrences to its own lineage every event
cost about 250 µs per event at 200 names and about 590 µs near 1,000. An
event that would take a value past the cap is refused with
`limit/provenance_limit`, and the session goes on (from 1.4.0; before it,
such an event ended the session as fatal `unclassified`, and a `carries`
or `carries_any` read past the cap still did until 1.5.0, finding D037).

**Expression work.** One expression evaluates at most 65,536 nodes, and a
history fold reads at most 256 records
([Reactive 0.6](../spec/caveat-reactive-0.6.md)). A fold whose reducer is large can
reach the node cap once its stream holds enough readings. Such an event is
refused with `limit/evaluation_limit` (from 1.4.0; before it, fatal
`unclassified`).

**Expansion.** A program's expression expands to at most 4,096 nodes, with
its function calls and `define`s inlined, or the program does not load. Each
call's arguments are expanded once and its body reads them by position, so a
function that reads a parameter twice, as `min` and `max` do, does not copy
the argument (from 1.5.0, finding D034). Before it, each read copied the
argument, so `min` nested to the right over eight values passed the limit
although the same `min` written balanced loaded.

**Text.** A text value, bound or computed, holds at most 65,536 bytes. An
event whose expression would make a longer one is refused with
`limit/text_limit`, and the session goes on (from 1.4.1; before it, such an
event ended the session as fatal `unclassified`, finding D025). A text
literal past the limit does not load.

**Other per-event caps.** Procedure calls nest at most 64 deep
(`limit/depth_limit`); at most 4,096 delayed qualifications wait at once
(`limit/scheduled_limit`); a session holds at most 65,536 identifiers
(`limit/identifier_limit`). These refuse the event
([Dispatch 0.1](../spec/caveat-dispatch-0.1.md)).

**What is not bounded per event.** Event time has no fixed bound, and saves,
archives and memory grow with retained history
([compatibility](COMPATIBILITY.md#limits)).

**Smallest cause of a refusal.** Caveat reports what a refused event's
guards read, which can be more than what caused the refusal. Finding the
smallest set of evidence that explains a failure is a minimal-diagnosis
problem (Reiter, "A theory of diagnosis from first principles", *Artificial
Intelligence* 32, 1987). In general it is as hard as minimum hitting set,
one of Karp's NP-complete problems (Karp, "Reducibility among combinatorial
problems", 1972). Caveat does not try to compute it.
