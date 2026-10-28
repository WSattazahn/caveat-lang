# CAVEAT Thresholds 0.1 — a named number the record lists

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.5: it becomes stable when a 1.x release ships it. It adds to
[Define 0.1](caveat-define-0.1.md) and [Reactive 0.5](caveat-reactive-0.5.md)
("Decision series"); it changes no program that loads today.

A decision often turns a number into a choice by comparing it with a bar:
go when the belief is at least 0.9, look again when it is at least 0.3. The
record says which number the decision read and what it chose. It does not
say which bars it was compared with. A reviewer reconstructs them from the
source, which the record binds by its SHA-256 but does not quote. A
[define](caveat-define-0.1.md) names the bar in the source, but it is
inlined and leaves no trace.

This profile names the bar so that the record lists it:

```caveat
claim door_open;
evidence estimator from "the robot's state estimator: probability the door ahead is open";
readings beliefs from estimator limit 32;
decisions plan limit 32 reopened by beliefs;
threshold go_bar = 0.9;
threshold look_bar = 0.3;
event belief p min 0 max 1;
on belief sample beliefs = p supports door_open;
on belief when not committed(plan) or reopened(plan)
    commit plan because enough
    using if(latest(beliefs) >= go_bar, 2, if(latest(beliefs) >= look_bar, 1, 0));
```

After `belief {"p": 0.55}`, `plan@1 = 1`, and its record lists
`go_bar = 0.9` and `look_bar = 0.3`. `caveat explain` prints:

```text
    plan@1 = 1  in force
      based on beliefs@1
      thresholds: go_bar = 0.9, look_bar = 0.3
```

## Syntax

```text
threshold NAME = NUMBER;
```

`NUMBER` is a number literal, signed or not. `NAME` is read wherever a number
is: in a guard, a value, a procedure step, a binding, a citation, a define or
a state's initializer. In a module a threshold's name is the module's own, as
a define's is, and a module may still name a state `threshold`.

These do not load:

| Source | Message |
| --- | --- |
| anything but a name, `=` and one number literal | `threshold expects NAME = NUMBER` |
| a name a state, history or event parameter has | `threshold NAME collides with a state` (`a history`, `an event parameter`) |
| a name a function has | `threshold NAME collides with a function` |
| a name another threshold or a define has | `duplicate threshold NAME` |

A threshold that nothing reads loads.

## Meaning

A threshold reads as its number. It is not state: no event changes it, it
has no evidence, no caveat and no lineage, and a value computed from it is
exactly the value computed from the literal written in its place. Changing
it is a source change, which the record already tells apart by the source's
SHA-256.

When a commit runs, its record lists every threshold the commit named, with
its value:

- the thresholds in its `using`, `citing` and `retaining … because`
  expressions;
- the thresholds in its own guard, and in the guard of every rule or step
  that called the procedure it runs in;
- a threshold named through a define, as if written there.

A threshold is listed whichever branch of an `if` the evaluation took: the
record says which bars the decision was compared with, not which comparison
decided it. A threshold read into a state by an earlier event is not listed;
the commit read the state, whose value its record already holds.

Only commitments list thresholds. A state, a binding or a reading that reads
one records the number, as before.

## Snapshot, save and explain

A commitment's entry in `commitment_bases` gains `thresholds`, a map from
name to value, left out when the commit named none:

```json
"plan@1": {"value": 1, "provenance": {"...": "..."}, "thresholds": {"go_bar": 0.9, "look_bar": 0.3}}
```

`caveat-explain/0.1` gives each revision the same optional `thresholds`, and
the text prints a `thresholds:` line under `based on`. No snapshot, save,
view or explanation of a program that declares no threshold changes by a
byte, and none of a program whose commits name none. No refusal code is
added.

`caveat-reactive-save/0.1` holds the map in the commitment's basis. Restore
accepts a listed threshold only when the source declares it with that value
([A3](../docs/RESTORE_TRUST_BOUNDARY.md)):

```text
commitment plan@1 lists threshold go_bar = 0.8, which the source does not declare
```

A runtime before this profile does not restore a save that lists a
threshold.

## Not in this profile

- Thresholds in the records of states, bindings or readings.
- Which comparison decided a value. The record lists the bars; the value and
  its grounds say what was read.
- A threshold computed from other values. A threshold is a number in the
  source.
- Replaying with a changed threshold to list the decisions it would flip. The
  graded-belief note keeps that as later tooling over replay and this record.

## Invariants kept

From [Caveat's invariants](../docs/CAVEAT_ESSENCE.md):

- A value carries what it rests on: a threshold adds nothing to rest on.
- A record is frozen when it is made; its thresholds are part of it.
- A refused event publishes nothing; this profile adds no refusal.

## Proofs

`proofs/lean/Caveat` models evidence, grounds and frozen bases. A threshold
is a number literal there, and the listed names are a record field that no
theorem reads, so no theorem changes. The implementation adds runtime tests.

## Corpus sweep

`node experiments/thresholds-sweep/sweep.mjs` reads every tracked Caveat
program and writes
[`RESULTS.json`](../experiments/thresholds-sweep/RESULTS.json). It reads
source text only and runs no program. At this profile's revision:

- 212 programs. No statement starts with `threshold` and no program uses
  the word otherwise, so no program reaches the new form and none changes
  meaning. A program that names a state `threshold` keeps its meaning: the
  word starts the new statement only before a name, `=` and one number;
- 11 defines in 4 programs name a bare number, the bars and constants a
  program could declare as thresholds. They keep their meaning; a define is
  unchanged.

The save sweep (`runtime/examples/save_sweep.rs`, 8 runs of 150 seeded
events) and the replay save sweep over every tracked program give the same
digest for every program before this change and after it.

## Implementation

- `threshold NAME = NUMBER` is parsed as a statement and registered as a
  define whose expression is a threshold node holding the name and the
  number. It evaluates and validates as the number; load checks it as it
  checks a define.
- A guarded effect runs with the thresholds of each enclosing guard on a
  stack, kept only when the source declares a threshold. A commit lists
  those and the ones its own expressions name in its basis.
- Restore refuses a listed threshold the source does not declare at that
  value.
- `runtime/tests/thresholds.rs` covers the robot's two bars, a threshold
  beside the literal it names, a guard, a define, a calling rule, a
  threshold read into state earlier, a negative and an unused threshold, a
  save round trip and two tampered saves, and the load errors.
  `runtime/tests/modules.rs` covers a module that declares a threshold and a
  state named `threshold`; `kit/test/explain.test.mjs` the explain record and
  text.
- The language reference, the authoring guide and the VS Code grammar name
  the form.

## Changes

- 2026-10-26: drafted and implemented for 1.5, from the graded-belief design
  note's P2 ([graded belief](../docs/design/graded-belief.md)).
