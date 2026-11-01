# CAVEAT Incremental Evaluation 0.1

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release.

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
  grounds differ after the event. A value differs when its bits do, so a zero
  that changes its sign has changed, as the clock's does;
- **the runtime clock**, through [`elapsed()`](caveat-elapsed-0.1.md). It counts
  as changed when the session's elapsed value changes, even if no state or
  graph record changed. Reads through a `define` or a pure function argument
  retain this dependency;
- **the graph**, through `observed`, `examined`, `committed`, `reopened`,
  `qualified`, `latest`, `has_sample` or a history read. It counts as changed
  when the event revealed, qualified, examined, committed, reopened or sampled
  anything, or changed a record those read.

A declaration whose every graph read names a reading stream or decision
series is scoped to those histories. After a graph change it is evaluated again
only if:

- a record of one of its histories changed: the history itself, one of its
  readings or revisions (`NAME@N`), a commitment basis or ground, a reopening's
  qualification, or a graph node or edge that touches one of them; or
- the event changed something a read of any history can reach. That covers a
  `qualifies` edge, a `supports` or `opposes` relation from evidence already
  in the graph, a qualification that is not a new reading's, a take-back,
  a withdrawal, a renewal, a retirement or a permission. It also covers a
  record no history owns but that a scoped read consults, or any removal.

A relation from evidence already in the graph counts because it brings the
caveats of the claim it names to that evidence, and a scoped read can list
evidence that belongs to no history: the reason a reading was withdrawn for,
the cause a decision was reopened by, or what an outcome was recorded because
of. Any other change to a claim or to evidence outside every history reaches no
scoped declaration. A sample of `api_tests` or a commitment in `api_gate`
leaves the declarations that read only `web_tests` as they were.

For an author this means `bind hud.elapsed = elapsed();` is evaluated when the
clock changes, and a binding that reads only `support` is evaluated when
`support` changes. The runtime clock dependency is distinct from any state
named `elapsed`; an ordinary event does not invalidate it merely by dispatching.

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
value, lineage or explanation, or in whether the event succeeds. Values are
compared as JSON text, which tells `-0` from `0` where `==` does not. The whole
test suite is therefore also a test of the equivalence. Breaking the change
detection on purpose makes it fail at once, which is how the check itself was
tested.

`runtime/examples/profile_dispatch.rs` reports where an event's time goes for
any program:

```sh
cargo run --release --no-default-features --example profile_dispatch -- PROGRAM.cav
```

## Changes

- 2026-09-28: a state that changes only the sign of a zero, from `0` to `-0`
  or back, counts as changed. It compared unchanged with `==`, so a binding
  that reads it kept showing the old sign, `atan2(x, -1)` pi where it should
  now be -pi, until a restore evaluated every binding in full
  ([Save 0.1](caveat-save-0.1.md#changes)). The debug check failed on that
  angle, but it compared values with `==` too, so a binding showing the zero
  itself, `bind hud.x = x`, passed with the wrong sign.
- 2026-11-01: a `supports` or `opposes` relation revealed from evidence
  already in the graph makes every scoped declaration evaluate again (D036).
  It was placed in no history, so a binding that read that evidence as a
  withdrawal's reason, a reopening's cause or an outcome's evidence kept its
  old lineage, without the caveats the relation brought, until something else
  made it evaluate again; a restored session showed them. The debug check
  failed on it.
