# CAVEAT Reading Notes 0.1 (draft) — keeping the observed text with a reading

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.1: implemented on `main`, not yet in a release. It becomes stable when a 1.x release ships it. It adds to
[Sampling](caveat-reactive-0.1.md), [Neutral samples 0.1](caveat-neutral-sample-0.1.md)
and [Identifiers 0.1](caveat-identifiers-0.1.md); it changes no program that
loads today.

A reading's value is a number, but what was observed is often text: a version
such as `22.22.0`, or a range such as `^22.22.2 || ^24.15.0 || >=26.0.0`. A
program can compare a number, so an author encodes the text (22.22.0 as
222200) and the recorded reading, `node_min@1 = 222202`, is no longer what the
source said. A range cannot be encoded at all, so only part of it is kept.
Before this profile there was no place for the text:

```text
does not load: line 6, column 1: sample requires supports CLAIM or opposes CLAIM
```

(`caveat-lang@1.0.2`, `caveat check`, on the program below.) This profile
lets a sample keep the text of an [identifier](caveat-identifiers-0.1.md)
with the reading it takes:

```caveat
claim compatible;
evidence registry from "the npm registry";
readings node_min from registry limit 8;
identifiers limit 64;
event read minimum min 0 max 999999, range id;
on read sample node_min = minimum supports compatible noting range;
```

After `read {"minimum": 222202, "range": "^22.22.2 || ^24.15.0 || >=26.0.0"}`,
`node_min@1` has the value 222202 and the note
`^22.22.2 || ^24.15.0 || >=26.0.0`. Decisions still compare the number. The
reading now also records what the registry said.

## Syntax

```text
sample STREAM = EXPRESSION [supports|opposes CLAIM] noting EXPRESSION;
```

The note's expression comes last. It is read as `id_text(EXPRESSION)`: an
identifier's handle, such as an `id` parameter or a state that holds one.
Anything that is not a number does not load:

```text
sample STREAM noting requires an identifier handle
```

The form stands wherever `sample` does: in a rule or as a procedure step. A
procedure passes a handle as an ordinary numeric argument.

`noting` is a word of the form only where what follows it parses as an
expression and something precedes it. A 1.0 program may name a state
`noting`, and `sample s = noting supports c` keeps its 1.0 meaning.

## Meaning

When the sample runs, the note's expression is evaluated and its text is kept
with the reading. A value that is not the handle of an identifier the session
holds refuses the event as `evaluation/expression`, as `id_text` does, and
nothing the event did is kept.

The note is text only. It joins neither the reading's lineage nor its grounds,
adds no evidence and no caveat, and changes nothing a decision rests on. A
reading with a note and one without, taken from the same value under the same
guard, have the same provenance. Reading the value back through `latest`,
`history_at` or `fold_history` gives the number, as before.

A reading keeps its note for as long as the stream holds it. A reading that
retires (its stream's `limit`, or a [window](caveat-windows-0.1.md)) takes
its note with it.

## Snapshot, effects and save

A noted reading's entry in `reading_streams` gains `"note"`, and the `sample`
effect reports it:

```json
{"id": "node_min@1", "value": 222202, "note": "^22.22.2 || ^24.15.0 || >=26.0.0", "...": "..."}
{"kind": "sample", "stream": "node_min", "id": "node_min@1", "value": 222202, "relation": "supports", "target": "compatible", "note": "^22.22.2 || ^24.15.0 || >=26.0.0"}
```

Both leave `note` out when there is none, so no snapshot, view or save of a
program that does not use `noting` changes by a byte. No refusal code is
added.

`caveat-reactive-save/0.1` holds the note in the reading's entry. Restore
refuses a note that is not the text of an identifier the save holds, or one on
a stream that no `sample … noting` of the program takes:

```text
reading node_min@1 has a note no sample of this program could take
```

A runtime before 1.1 does not restore a save that holds a note.

## Not in this profile

- Text values in expressions. A note is kept and shown; a program cannot
  compare or compute with it. Decisions compare numbers.
- A text literal as the note. A note is what was observed, so it comes from
  the host as an `id` parameter.
- Notes on revealed evidence, renewals or decisions.

## Corpus sweep

No tracked program writes `noting`, and none names anything `noting`; before
this profile the form did not load. The save sweep
(`runtime/examples/save_sweep.rs`, 8 runs of 150 seeded events) over all
tracked programs gives the same digest for every program as `main` before this
change: no program that loads today changes an outcome, a save or a restored
save.

## Implementation

`sample` carries an optional note expression, parsed as `id_text(…)`. The
reading and the `sample` effect report keep its text. Restore accepts a note
only where a `noting` sample of the stream could have taken it, from the
save's identifiers. Tests: `runtime/tests/reading_notes.rs`.

## Changes

- 2026-10-19: drafted and implemented for 1.1 (issue #220).
