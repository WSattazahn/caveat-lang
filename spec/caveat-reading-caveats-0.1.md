# CAVEAT Reading Caveats 0.1 — a caveat on the one reading a sample takes

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.5: it becomes stable when a 1.x release ships it. It adds to
[Sampling](caveat-reactive-0.1.md), [Neutral samples 0.1](caveat-neutral-sample-0.1.md),
[Reading Notes 0.1](caveat-reading-notes-0.1.md) and
[Late Qualification 0.1](caveat-late-qualification-0.1.md); it changes no
program that loads today.

A reading's value is often a probability or a score. Below some level, or
inside some band, the author knows the reading is weak, and wants every
decision that uses it to carry that doubt. Before this profile a program has
two ways to say so, and neither says it about the reading alone:

- a declared `uncertain qualifies estimator;` puts the caveat on the
  evidence, so every reading copies it, the sure ones too;
- a rule `on belief when p < 0.9 qualify latest(beliefs) with uncertain;`
  qualifies the reading after it is taken. That is a
  [late caveat](caveat-late-qualification-0.1.md): it needs a second rule
  with the sample's condition written again, and a rule between the two
  reads the reading without it.

This profile lets the sample say it:

```caveat
claim door_open;
evidence estimator from "the robot's state estimator: probability the door ahead is open";
caveat uncertain consequence material;
readings beliefs from estimator limit 32;
decisions plan limit 1 irreversible;
event belief p min 0 max 1;
event go;
on belief sample beliefs = p supports door_open with uncertain when p > 0.3 and p < 0.9;
on go commit plan because enough using latest(beliefs);
```

After `belief {"p": 0.95}`, `beliefs@1` carries nothing and `go` commits.
After `belief {"p": 0.5}`, `beliefs@1` carries `uncertain`, and `go` is
refused as `caveated_irreversible`: the plan would rest on `uncertain`
(material), which its commit does not retain. `using latest(beliefs)
retaining uncertain` commits it on purpose.

## Syntax

```text
sample STREAM = EXPRESSION [supports|opposes CLAIM] [noting EXPRESSION] with CAVEAT[, CAVEAT...] [when CONDITION];
```

The clause comes last, after the stance and the note. Each `CAVEAT` is a
declared caveat, named once. `CONDITION` is a boolean expression; it reads
what the sample's expression may read, including the event's parameters and,
in a procedure, its arguments. The form stands wherever `sample` does: in a
rule, under the rule's own `when`, and as a procedure step. A procedure's
`caveat` parameter may be named.

`with` is a word of the form only where what follows it is a list of names
and, optionally, `when` and an expression that parses. A program may name a
state `with`, and `sample s = with;` keeps its meaning. Before this profile
no sample with `with` after its value loaded, so no program that loads today
reads differently.

These do not load:

| Source | Message |
| --- | --- |
| a caveat that is not declared, or a name that is not a caveat | the message `qualify` gives for the same name |
| `withdrawn`, in a program that withdraws | `sample with cannot apply withdrawn: only withdraw applies it` |
| a caveat that `unqualify` takes back | `sample with cannot name C: it is taken back by unqualify, so only qualify applies it` |
| a condition that is not boolean | `sample STREAM with ... when requires a boolean condition` |
| a caveat named twice, or `when` with nothing after it | `sample expects STREAM = EXPRESSION [supports\|opposes CLAIM] [noting EXPRESSION] [with CAVEAT[, CAVEAT...] [when CONDITION]]` |

## Meaning

When the sample runs, its value is taken as before. Then the condition, if
there is one, is evaluated. When it is true, or when there is none, each
named caveat qualifies the new reading, `STREAM@N`, as the caveats the
reading copies from its evidence do. When it is false, the reading is taken
without them. A condition that fails to evaluate refuses the event as an
expression in the sample would.

The caveats are on that reading alone. The evidence, its other readings and
the claim the reading bears on are not qualified. A value computed from the
reading carries them, a decision whose grounds hold the reading carries them,
and `carries(latest(STREAM), C)`, `carries(reading(STREAM, N), C)` and
`carries_any(…)` see them, all as for any caveat on a reading.

The condition's lineage joins the reading's observation whichever way it
falls, as a sampling guard's does
([Reactive 0.5](caveat-reactive-0.5.md), "A sample is a new observation"): a
reading whose condition read a state rests on that state's evidence and
caveats too. A reading for which the condition was false is therefore not
the same as one taken with no clause; it rests on what was checked.

The caveats are not late caveats. They are on the reading from the moment it
exists, in the same event, so no read ever sees the reading without them.
They are never taken back: the reading keeps them until it retires or is
withdrawn, and a withdrawal works as it does today.

## Snapshot, effects and save

Nothing new. A reading's caveats are already listed in its provenance in
`reading_streams` and in the `qualifies` relations, and the `sample` effect
is unchanged. `caveat-reactive/0.1`, `caveat-reactive-save/0.1`,
`caveat-explain/0.1` and the views gain no field. No refusal code is added.

A save holds the qualification as it holds any other. Restore accepts a
caveat on a reading only where a mechanism of the source could have put it
there ([A3](../docs/RESTORE_TRUST_BOUNDARY.md)); `sample STREAM … with C` is
one, for every reading of `STREAM`. Conditions are not evaluated, as guards
are not.

## Not in this profile

- Taking a reading's own caveat back. A take-back caveat cannot be named in
  `with`; a program that needs to take a doubt back writes the late
  `qualify latest(STREAM) with C` it can `unqualify`.
- Caveats on revealed evidence, renewals or decisions by the same clause.
- A condition per caveat. A clause has one condition; a reading that needs a
  second caveat on another condition is qualified late for it.
- A graded or numeric caveat. The grade is the reading's value; the caveat
  says what is known to be wrong with it.

## Invariants kept

From [Caveat's invariants](../docs/CAVEAT_ESSENCE.md):

- Every value carries what it rests on: the clause only adds caveats.
- A caveat is never discharged. It reaches every value read from the reading.
- A refused event publishes nothing; the clause adds no refusal of its own.

## Proofs

`proofs/lean/Caveat` models evidence, grounds and frozen bases. A caveat on a
reading is a qualification of evidence the model already covers, so no
theorem changes. The implementation adds runtime tests.

## Corpus sweep

`node experiments/reading-caveats-sweep/sweep.mjs` reads every tracked Caveat
program and writes
[`RESULTS.json`](../experiments/reading-caveats-sweep/RESULTS.json). It reads
source text only and runs no program. At this profile's revision:

- 212 programs. No sample has the word `with` after its `=`, and no program
  declares anything named `with`, so no program reaches the new form and none
  changes meaning;
- four rules in two programs put a caveat on one reading today with
  `qualify latest(STREAM) with C`. They keep their meaning; this profile does
  not change late caveats.

The save sweep (`runtime/examples/save_sweep.rs`, 8 runs of 150 seeded
events) and the replay save sweep over every tracked program give the same
digest for every program before this change and after it: no program that
loads today changes an outcome, a save or a restored save.

## Implementation

- `sample` carries an optional `with` clause: the caveats and the condition.
  The parser reads it from the right, before `noting`, only where the words
  after `with` have its shape. Load checks each caveat as `qualify` does and
  refuses `withdrawn` and take-back caveats.
- When the sample runs, the condition is evaluated before the reading is
  added, its provenance joins the observation, and when it holds each caveat
  qualifies the new reading beside the caveats it copies from its evidence.
- Restore counts `(C, STREAM)` among the pairs a mechanism can produce.
- `runtime/tests/reading_caveats.rs` covers the band, the caveat on one
  reading only, an irreversible commit refused and retained, the condition's
  lineage either way, a clause without a condition, a procedure step and
  procedure parameters, a save round trip and restore, the clause after a
  stance and a note, a state named `with`, and the load errors.
- The language reference, the authoring guide and the VS Code grammar name
  the form.

## Changes

- 2026-10-26: drafted and implemented for 1.5, from the graded-belief design
  note's P1 ([graded belief](../docs/design/graded-belief.md)).
