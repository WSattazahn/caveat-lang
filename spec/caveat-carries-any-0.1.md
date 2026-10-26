# CAVEAT Carries Any 0.1 — asking whether evidence carries any caveat

**Status:** stable from 1.4.0 under the [compatibility commitment](../docs/COMPATIBILITY.md); drafted for 1.4. It adds to
[Renewal 0.1](caveat-renewal-0.1.md) ("Asking whether evidence carries a
caveat"), [Latest 0.1](caveat-latest-0.1.md),
[Earlier Occurrence 0.1](caveat-earlier-occurrence-0.1.md) and
[Reading Index 0.1](caveat-reading-index-0.1.md); it changes no program that
loads today.

A rule that keeps a decision off doubted evidence has to name every doubt:

```caveat
caveat flaky consequence material;
caveat wrong_config consequence high;
on ship when withdrawn(latest(checks)) or carries(latest(checks), flaky)
    or carries(latest(checks), wrong_config) reject "the latest check is in doubt";
```

The rule is right until someone declares a third caveat. A check qualified
with `stale` then passes the guard, and nothing says so: the list was the
rule, and the list is out of date. An author who means "any caveat" can only
write the caveats that exist when the rule is written.

This profile lets the rule say what it means:

```caveat
on ship when carries_any(latest(checks)) reject "the latest check is in doubt";
```

or, to let routine doubts through and stop only the ones that matter:

```caveat
on ship when carries_any(latest(checks), material) reject "the latest check is in doubt";
```

## Syntax

```text
carries_any(TARGET)
carries_any(TARGET, CONSEQUENCE)
TARGET      := EVIDENCE | latest(NAME) | occurrence(EVIDENCE, INDEX) | reading(STREAM, INDEX)
CONSEQUENCE := negligible | low | material | high | catastrophic
```

`carries_any(…)` is a boolean expression and stands wherever
`carries(…)` does: in a guard, a value, a `bind` and a procedure step. Each
`TARGET` is what the same position of `carries(TARGET, CAVEAT)` accepts, with
the same load rules: `EVIDENCE` is declared evidence, `latest(NAME)` a reading
stream or declared evidence, `occurrence(EVIDENCE, INDEX)` renewable evidence
and `reading(STREAM, INDEX)` a reading stream. A procedure's `evidence` or
`readings` parameter may be the target. A pure source function may not ask
it, as it may not ask `carries`.

These do not load:

| Source | Message |
| --- | --- |
| no argument, more than two, or a target that is not one of the four forms | `carries_any expects carries_any(EVIDENCE) or carries_any(EVIDENCE, CONSEQUENCE), where EVIDENCE may be latest(NAME), occurrence(E, I) or reading(STREAM, I)` |
| a second argument that is not a consequence word | `carries_any's second argument is a consequence: one of negligible, low, material, high, catastrophic` |
| `occurrence` or `reading` without an index | `occurrence(…) in carries_any takes evidence and an index` |
| a target the same `carries` form refuses | that form's message |

The consequence words are already reserved, so no caveat, claim or state can
be named like one.

**A program that defines `fn carries_any`.** Before this profile, a source
function could be named `carries_any`. A program that defines one keeps it:
every `carries_any(…)` in that program calls the function, as it did, and
the predicate is not available to it. A program that defines no such function
gets the predicate.

## Meaning

`carries_any(TARGET)` is true exactly when `carries(TARGET, C)` is true for
some caveat `C`: a caveat the program declares, or `withdrawn` in a program
that withdraws. `carries_any(TARGET, LEVEL)` is true exactly when that holds
for some `C` of consequence `LEVEL` or above, in the order `negligible`,
`low`, `material`, `high`, `catastrophic`. `withdrawn` is `material`.

So it is the list written out over every caveat, computed when the
expression is evaluated, never when the rule was written:

- the target is resolved as `carries` resolves it: the current occurrence of
  evidence, the current reading of a stream for `latest(…)`, the occurrence
  or reading an index names. A target that is not observed, a stream with no
  reading, and an index that names nothing held make it false, and nothing
  is refused;
- a caveat counts when it qualifies the target directly, through another
  caveat or through the claim it bears on, and has not been taken back
  ([Take-back 0.1](caveat-take-back-0.1.md)): whatever `carries` counts. A
  caveat added later by `qualify`, scheduled with `after`, or applied by a
  withdrawal counts from then on, and stops counting when it is taken back;
- a caveat that is examined or deferred still counts. Attention says what
  has been looked at, not what is no longer doubted.

Its lineage is the lineage of `carries(TARGET, C)`: the target's evidence
and the caveats on it, whether the answer is true or false. That is also the
lineage of the list written as `carries` terms joined by `or`. A list that
starts with `withdrawn(…)` differs in one way: `withdrawn(…)` brings the
withdrawal's reason into the lineage when it is true, and `carries_any` does
not, as `carries(TARGET, withdrawn)` does not.

The predicate reads the graph, as `carries` does, so a binding or state that
asks it is current after any event that changes what the target carries.

## Snapshot, save, explain and views

Nothing new. The predicate adds no field, no schema member and no refusal
code. It reaches the host's predicate interface as kind `carries:*` (with the
level, `carries:*material`), `latest:carries:*`, or the occurrence form of
either; no declared caveat can be spelled with `*`. `caveat-reactive/0.1`,
`caveat-reactive-save/0.1`, `caveat-replay-save/0.1` and `/0.2`,
`caveat-explain/0.1`, `caveat-dependents/0.1` and the views are unchanged,
and a save restores as it did.

`caveat check` is unchanged.

## Not in this profile

- An "any caveat" form of `has_caveat(STATE, CAVEAT)` for states.
- Naming a set of caveats (`carries_any(E, flaky, stale)`). A list of named
  caveats is written as today.
- A consequence range or an exact consequence.

## Invariants kept

From [Caveat's invariants](../docs/CAVEAT_ESSENCE.md):

- A predicate reads the graph and changes nothing in it.
- It discharges no caveat: a value it guards carries what it read.
- A refused event publishes nothing; this profile adds no refusal.

## Proofs

`proofs/lean/Caveat` models evidence, grounds and frozen bases. The predicate
is a disjunction of predicates the model already covers and records nothing,
so no theorem changes. The implementation adds runtime tests.

## Corpus sweep

`node experiments/carries-any-sweep/sweep.mjs` reads every tracked Caveat
program and writes
[`RESULTS.json`](../experiments/carries-any-sweep/RESULTS.json). It reads
source text only and runs no program. At this profile's revision:

- 203 programs. None writes the word `carries_any` or defines a function of
  that name, so no program reaches the new form and none changes meaning;
- no tracked program writes "any caveat" as a hand-written list (two or more
  `carries(T, C)` or `withdrawn(T)` joined by `or` on one target). The sweep
  reports each such list with the declared caveats it leaves out, when it is
  pointed at other programs.

The save sweep (`runtime/examples/save_sweep.rs`, 8 runs of 150 seeded
events) and the replay save sweep over every tracked program give the same
digest for every program before this change and after it: no program that
loads today changes an outcome, a save or a restored save.

## Implementation

- The expression parser reads `carries_any(…)` as a call, and the expander
  turns it into a `carries` predicate whose caveat is `*` followed by the
  level, unless the program defines `fn carries_any`. The session answers it
  over the target's caveats, with the consequence each caveat was declared
  with, and validates it with the target rules of `carries`.
- `runtime/tests/carries_any.rs` covers the four targets, the level, a take-back,
  a withdrawal, a renewal, a guard refusing a decision and a save round trip,
  a binding that follows the caveats, an examined caveat that still counts,
  the lineage beside the hand-written list at every step, a program with no
  caveat, a procedure parameter, a program defining `fn carries_any`, the
  load errors, and a source function.
- The language reference, the authoring guide and the VS Code grammar name
  the form.

## Changes

- 2026-10-25: drafted and implemented for 1.4.
