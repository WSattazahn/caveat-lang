# CAVEAT Withdrawal 0.1

Evidence can turn out to be wrong: a test log misread, a witness who was
elsewhere. Before this profile a program could only add a caveat to evidence
or record a contrary observation. In the
[agent ledger](../experiments/agent-ledger/README.md), a check result recorded
as passed and then found to be misread could only be followed by a failed
one. A decision made on the misread result still cited it, with nothing to
mark it as wrong.

```caveat
claim misreading;
evidence recheck from "the agent re-read the CI log";
event misread;
on misread reveal recheck supports misreading;
on misread withdraw latest(checks) because recheck;
on misread when committed(merge) and not reopened(merge) and rests_on_withdrawn(merge)
    reopen merge because recheck;
```

A withdrawal records that an observation is no longer stood behind, and why.
It removes nothing, and it does not make the opposite claim true: withdrawing
"the checks passed" is not "the checks failed".

## Syntax

```text
withdraw EVIDENCE because EVIDENCE;           -- an effect, in a rule or procedure
withdraw latest(STREAM) because EVIDENCE;
withdrawn(EVIDENCE)                           -- predicates
withdrawn(latest(STREAM))
rests_on_withdrawn(DECISION)
```

## What `withdraw` does

When the effect runs it resolves both names to concrete occurrences:
- the target: named evidence resolves to its current occurrence, and
  `latest(STREAM)` to the stream's current reading;
- the reason: named evidence, resolved to its current occurrence.

Both must be observed, or the event fails, as `qualify` does for unobserved
evidence. A later renewal does not move a withdrawal to a different
occurrence.

Then:

1. **A record.** The session's `withdrawals` gains `{evidence, because,
   sequence, event}`. This record is authoritative: `withdrawn(E)` asks it.
   Withdrawing evidence that is already withdrawn keeps the first record and
   does nothing else.
2. **Current values show it.** The target gains the built-in caveat
   `withdrawn`, exactly as [late qualification](caveat-late-qualification-0.1.md)
   applies a caveat. Every current value whose lineage or grounds include the
   target gains it, and so does every later `qualified(…, E)` and
   `observed(E)`.
3. **Nothing recorded changes.** The observation and its relation to its
   claim stay. So do reading archives, every decision's frozen basis and
   grounds, and the decision journal. A decision keeps what it was made on.

The effect is part of its event's transaction, so a refused or failed event
withdraws nothing. It is reported in `effects` as
`{"kind": "withdraw", "evidence", "because"}`.

## Reading withdrawn history

History keeps what it held: membership, order, values and recorded metadata do
not change. A new computation that reads a withdrawn reading through `latest`,
`history_at` or `fold_history` gets that reading's value with the `withdrawn`
caveat added. Copying `latest(checks)` into a state after the withdrawal gives a
state that carries `withdrawn`. So reading a withdrawn value again never makes
it look clean, and a withdrawn reading is never skipped silently.

## Predicates

- `withdrawn(E)` is true when E's current occurrence is withdrawn. A fresh
  occurrence of renewable evidence is not withdrawn because an earlier one was.
- `withdrawn(latest(S))` is true when the stream's current reading is
  withdrawn.
- `rests_on_withdrawn(D)` is true when the decision's current revision was
  grounded on withdrawn evidence. It compares the evidence in the revision's
  frozen [grounds](caveat-explanations-0.2.md) against the withdrawal records:
  - not its frozen caveats, which record what was known then;
  - not its lineage, since evidence that only a guard read is not what a
    decision rests on.

Each predicate carries the withdrawal's reason as its evidence. Nothing
reopens by itself: a program states what a withdrawal means for its decisions.

## The built-in caveat

A program with a `withdraw` effect has a caveat named `withdrawn`, with
consequence `material`. The predicates alone do not create it: without a
`withdraw`, they are always false. It exists only as the effect of a
withdrawal. Such a program cannot:
- declare anything else named `withdrawn`;
- apply it with `qualify`, `qualified(…)` or `commit … retaining`;
- `examine` it: examining does not reverse a withdrawal.

It may read it, as in `has_caveat(estimate, withdrawn)`.

## Snapshot and save

The snapshot lists `withdrawals`, in the order they happened. A save holds the
same list, and leaves it out when it is empty, so a save made before this
profile restores with none. Restoring refuses a list that:
- names unknown evidence;
- withdraws the same evidence twice;
- is dated after the save's own sequence, or to an undeclared event;
- disagrees with the graph: every record needs its `withdrawn qualifies E`
  relation, and every such relation needs its record;
- appears in a program that does not use withdrawal.

## Not in this profile

This profile does not include:
- undoing a withdrawal;
- judging whether a reason is good enough;
- withdrawing a reading by its value, such as "the reading about commit X";
- permission.
