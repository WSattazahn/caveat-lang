# CAVEAT Incremental Evaluation 0.1

An event should cost what it touches, not the size of the program. Round 6 of
the [glowcap benchmark](../experiments/glowcap/RESULTS.md) showed the opposite:
declaring eight times the entities made every event about ten times slower, to
1.1 ms. A profile of that program found where the time went:

| Per event (native, 32 entities) | Before | After |
| --- | --- | --- |
| Evaluating every binding | 510 µs | only the ones whose inputs changed |
| Copying the session for the transaction | 55 µs | pointer copies |
| Building the view | 26 µs | borrowed, not copied |
| **Whole event, native** | **611 µs** | **74 µs** |
| **Whole event, WebAssembly, host JSON parse included** | **1,090 µs** | **160 µs** |

Rules were not the cost: about 1,600 guards took some 30 µs. This profile
changes nothing a program means. It changes what an event pays for.

## What a binding costs

Each `TARGET.PROPERTY` is shown by the last of its `bind` declarations that
matches, and its lineage includes every declaration's guard. So the property is
the unit of work: after an event, a property is evaluated again only if one of
its declarations can read something the event changed. Otherwise it keeps what
it showed, which is exactly what evaluating it again would give, because an
expression's result depends only on what it reads.

What a declaration can read is worked out when the program loads, from its
condition, its value and its `because` citations, over every branch whether it
is taken or not:

- a **state** it names. It counts as changed when its value, its lineage or its
  grounds differ after the event;
- **the graph**, through `observed`, `examined`, `committed`, `reopened`,
  `qualified`, `latest`, `has_sample` or a history read. It counts as changed
  when the event revealed, qualified, examined, committed, reopened or sampled
  anything, or changed a record those read.

For an author this means a binding that reads `now` is evaluated on every tick,
and one that reads only `support` is evaluated when `support` changes.

## What a transaction copies

An event still runs as a transaction: it either publishes whole or leaves the
session exactly as it was. The copy it works on shares everything with the
session until an effect writes to it:

- each state lives in its own shared cell, and a `set` copies only that cell;
- the graph, its symbols, and the observation, predicate, commitment, reading
  and decision records are copied only when an effect changes them;
- load-time data (declarations, rules, bindings, the world) is never copied;
- the shown bindings are handed to the transaction and handed back if it fails,
  because rules never read them and evaluation writes them only after it has
  succeeded.

## How it is checked

Debug builds, which is what `cargo test` runs, evaluate every binding in full
after every event as well, and fail if the incremental result differs in any
value, lineage or explanation, or in whether the event succeeds. The whole test
suite is therefore also a test of the equivalence. Breaking the change detection
on purpose makes it fail at once, which is how the check itself was tested.

`runtime/examples/profile_dispatch.rs` reports where an event's time goes for
any program:

```sh
cargo run --release --no-default-features --example profile_dispatch -- PROGRAM.cav
```
