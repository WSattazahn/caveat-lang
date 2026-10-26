# CAVEAT Foreclosure 0.2 — decisions that rule options out while in force

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Draft for 1.4: it becomes stable when a 1.x release ships it. It adds to
[Foreclosure 0.1](caveat-foreclosure-0.1.md),
[Holds 0.1](caveat-holds-0.1.md) and
[Dispatch 0.1](caveat-dispatch-0.1.md); it changes no program that loads
today.

[Foreclosure 0.1](caveat-foreclosure-0.1.md) lets an irreversible decision
close off other decisions for good. It left one question open (Questions
settled, 5): may a decision that can be undone foreclose? Many exclusions
are of that kind. "We ruled out a rewrite" is a decision with reasons, and
the reasons can turn out wrong. While it stands, nobody should start a
rewrite. Once it is reopened, a rewrite is an option again.

Today a program writes that as a guard with a message:

```caveat
decisions no_rewrite limit 4;
decisions rewrite limit 4;
on rule_out commit no_rewrite because enough citing qualified(1, review);
on rewrite_it when committed(no_rewrite) and not reopened(no_rewrite)
    reject "Ruled out: no_rewrite.";
on rewrite_it commit rewrite because enough using 1;
```

It works, but the exclusion is a string. `explain` shows no link between
`no_rewrite` and `rewrite`. Every rule that commits `rewrite`, including one
inside a procedure or a new rule written later, needs its own copy of the
guard. And the refusal is `policy/reject`, the same as any authored refusal.

This profile lets the declaration say it:

```caveat
decisions no_rewrite limit 4 forecloses rewrite;
decisions rewrite limit 4;
on rule_out commit no_rewrite because enough citing qualified(1, review);
on rewrite_it commit rewrite because enough using 1;
```

While `no_rewrite`'s current revision is in force, a `rewrite_it` event is
refused as `evaluation/foreclosed`, with the message
`rewrite is foreclosed by no_rewrite@1`. When `no_rewrite` is reopened,
because its review was withdrawn, a trigger fired or a rule reopened it,
`rewrite` is open again. `explain` prints:

```text
  no_rewrite: 1 of at most 4, forecloses rewrite
  rewrite: 0 of at most 4, foreclosed by no_rewrite@1
```

## Syntax

```text
decisions NAME limit N [irreversible [strict]] [forecloses ACTION, ACTION, …] [reopened by TRIGGER, TRIGGER, …];
```

`forecloses` may now follow `limit N` directly. With `irreversible` before
it, the declaration means what Foreclosure 0.1 says, unchanged. Without it,
the series is an ordinary series of any limit that forecloses each `ACTION`
while it is in force. Each `ACTION` is as in Foreclosure 0.1: a declared
decision series or a plain commitment that a `commit` names, after
procedures and `for` blocks are expanded.

The word is still read only in this position, so it reserves nothing. A
series, stream or claim named `forecloses` keeps loading, and
`decisions forecloses limit 2 forecloses other;` declares a series named
`forecloses`.

The load rules of Foreclosure 0.1 apply to both forms, each with its
message: an `ACTION` that is not a decision, a series that names itself, and
an `ACTION` named twice. A `forecloses` with no action, or words after the
actions other than `reopened by`, does not load:
`decisions expects NAME limit CAPACITY [irreversible] [forecloses ACTION, …] [reopened by STREAM, …], not …`.
The 0.1 message `forecloses follows irreversible: …` is gone, because that
form now loads. A load error's message is not part of the promise; its code
and status are. A declaration without either word keeps today's message,
`decisions expects NAME limit CAPACITY [reopened by STREAM, …]`, byte for byte.

## Meaning

### In force

A reversible series `D` is **in force** when it has a current revision and
that revision is not reopened, that is, exactly when
`committed(D) and not reopened(D)` is true. While `D` is in force, an executed
`commit` of an `ACTION` it forecloses is refused as `evaluation/foreclosed`,
with the message `ACTION is foreclosed by D@K`, where `D@K` is the current
revision.

- **Reopening lifts it.** Any reopening of `D` lifts the exclusion: a
  `reopen`, a declared trigger, or a withdrawal reopening a revision on hold.
  The reopening is recorded as before, and nothing else changes. A new
  revision `D@K+1`, once committed, forecloses again.
- **Only what it rests on lifts it.** A foreclosure is not evidence. While
  `D` is in force, the exclusion stands even if `D` rests on withdrawn
  evidence and no rule has reopened it yet. A program reopens `D` when its
  reasons fail, as it does today, and the option opens then. The author
  decides when a decision is in question; the runtime does not decide it
  for them.
- **Holds.** A hold keeps `D` in force, so it keeps the exclusion while the
  reopenings it defers wait. Its release applies them and lifts the
  exclusion. A hold never keeps a revision in force on withdrawn evidence
  ([Holds 0.1](caveat-holds-0.1.md)): a withdrawal that reaches a held
  revision reopens it, and the exclusion lifts with it.
- **Only later commits.** As in Foreclosure 0.1, a revision of `ACTION`
  already committed keeps its status, basis and caveats. Whether `D` may be
  committed while `ACTION` is in force is the author's policy, written as a
  guard.
- **Within an event.** Commits run in rule order. A commit of `ACTION` before
  the commit of `D` in the same event is accepted; one after it is refused,
  and the event publishes nothing.
- **Order of checks.** `foreclosed` is checked first, before every other
  check of the commit, as in Foreclosure 0.1.
- **Several foreclosures.** When more than one revision forecloses the same
  action at once (an irreversible revision, or a reversible series in
  force), the message names the one committed first, by event sequence and
  then by series name. When that one lifts, the next is named.
- **Mutual exclusion.** Two series may each foreclose the other. Whichever is
  committed first keeps the other out until it is reopened.

A reversible foreclosure has no rule about caveats. `caveated_irreversible`
and `strict` ([Strict Irreversible 0.1](caveat-strict-irreversible-0.1.md))
stay rules of irreversible series only.

### Codes

No new code. A commit refused by a reversible foreclosure is
`evaluation/foreclosed`, the 1.2.0 code for an executed commit of an action a
decision forecloses. A program that declares no `forecloses` never produces
it.

## Check

`caveat check` is unchanged. C002 `no-reopening-path` still reports a
reversible series without a reopening path: reopening is how its exclusion
lifts, so the warning matters more for it, not less.

## Snapshot, save, explain and views

- The `caveat-reactive/0.1` snapshot gains `forecloses`: by reversible series
  that forecloses any action, those actions in source order. It is absent
  when no reversible series forecloses, so every program that loads today
  keeps its snapshot byte for byte. Irreversible series stay under
  `irreversible`.
- A save holds nothing new. Restoring refuses a save whose journal commits a
  foreclosed action while a reversible revision that forecloses it was in
  force: after its commit and before its reopening, placed by sequence and,
  within an event, by journal order. A reopening that left a windowed
  journal can't be placed, and that revision is not checked. As with every
  restore check, restore never authenticates history.
- `caveat-explain/0.1` gives a reversible series `forecloses`, and a
  foreclosed series or plain commitment `foreclosed_by`: the revisions that
  foreclose it now, by event sequence and then name. A reversible revision
  is listed only while it is in force. The text is as in Foreclosure 0.1:
  `, forecloses A, B` and `, foreclosed by D@K`.
- `dependents` and the views are unchanged.

## Not in this profile

- Lifting the exclusion when `D` merely rests on withdrawn evidence, before
  anything reopens it (see "Only what it rests on lifts it").
- Foreclosing an option value, or foreclosing for a period.
- A `foreclosed(ACTION)` predicate. `committed(D) and not reopened(D)` asks it.

## Questions settled

1. **A new code?** No. The refusal is the same kind of refusal, with the same
   origin, and a host already handles `foreclosed`.
2. **When does it lift?** On reopening, the moment the program records that
   the decision is in question. It does not lift on withdrawn evidence alone:
   a program that wants that reopens on withdrawal, as the example above
   could with `rests_on_withdrawn(no_rewrite)`.
3. **May the series be `limit 1`?** Yes. A reversible series of one revision
   forecloses until it is reopened, and then for good it forecloses nothing.

## Invariants kept

From [Caveat's invariants](../docs/CAVEAT_ESSENCE.md):

- A foreclosure deletes no evidence, adds no relation and rewrites no basis.
- Reopening a decision records its cause, and keeps its grounds and caveats.
- A refused event publishes nothing, so a refused commit leaves every option
  as it was.

## Proofs

`proofs/lean/Caveat` models evidence, grounds and frozen bases. The rule
refuses a commit before it records anything, so the frozen-basis and
rollback theorems cover it without change. The implementation adds runtime
tests.

## Corpus sweep

`node experiments/reversible-foreclosure-sweep/sweep.mjs` reads every tracked
Caveat program and writes
[`RESULTS.json`](../experiments/reversible-foreclosure-sweep/RESULTS.json). It
reads source text only and runs no program. At this profile's revision:

- 203 programs declare 168 decision series, in 138 programs. None writes
  `forecloses` after `limit N` without `irreversible`, and no declared name is
  spelled `forecloses`. No program reaches the new form;
- one rule writes the convention this profile replaces:
  `kit/examples/handoff/handoff.cav`'s `sub_rewrite` rejects a commit of
  `rewrite` while `no_rewrite` is in force. It is left as written.

The save sweep (`runtime/examples/save_sweep.rs`, 8 runs of 150 seeded
events) and the replay save sweep over every tracked program give the same
digest for every program before this change and after it.

## Implementation

- The runtime reads `forecloses` after `limit N` with or without
  `irreversible`, keeps the reversible ones apart from the irreversible
  ones, and asks both when a commit runs. A reversible series forecloses
  while its current revision is not open. Restore checks the journal as
  above. The snapshot carries `forecloses`.
- `runtime/tests/reversible_foreclosure.rs` covers the exclusion and its
  lifting by a reopening, a trigger and a withdrawal; a new revision; a
  commit already made; holds, deferral and release; rule order within an
  event; mutual exclusion; an irreversible and a reversible foreclosure of
  the same action; a procedure; the snapshot field; a save round trip and
  restore's check both ways; the load rules; C002. It also runs 300 random
  histories of 40 steps, with saves and restores in place, beside a twin
  program that writes the same exclusion as guards. The twin must accept and
  refuse the same events, and a foreclosed refusal must match the twin's
  guard.
- The kit's `explain` and types describe the field. The language reference
  and the authoring guide describe the form.

## Changes

- 2026-10-25: drafted and implemented for 1.4. Its random test found D021
  ([Holds 0.1](caveat-holds-0.1.md), Changes).
