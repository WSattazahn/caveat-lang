# CAVEAT Reopening Triggers 0.1

A decision often has to be reconsidered whenever a certain kind of observation
arrives: a new push after a merge decision, a failed check after it. Before
this profile, every rule that took such a reading needed its own reopening
rule after it:

```caveat
on pushed sample pushes = commit supports changed;
on pushed when committed(merge) and not reopened(merge) reopen merge because latest(pushes);
```

The [agent ledger](../experiments/agent-ledger/README.md) wrote such rules by
hand, and forgetting one leaves a stale decision in force with nothing to show
it. A decision series can now declare what reopens it:

```caveat
decisions merge limit 8 reopened by pushes, checks opposing ready;
```

The program still states when the decision is reconsidered, once, where the
decision is declared. The runtime carries out that statement. It does not
judge whether a reading is concerning enough, and it does not treat a newer
reading as replacing an older one, or as being about the same subject.

## Syntax

```text
decisions NAME limit N reopened by TRIGGER, TRIGGER, …;
TRIGGER := STREAM | STREAM supporting CLAIM | STREAM opposing CLAIM
```

Each `STREAM` must be a declared reading stream, and each `CLAIM` a declared
claim. Only decision series declare triggers; a one-off commitment has no
declaration to put them on.

## Meaning

Whenever a `sample` records a reading into `STREAM`, and the reading's relation
and claim match the trigger's filter if it has one, each series with a matching
trigger runs a reopening step:

```caveat
when committed(D) and not reopened(D) reopen D because latest(STREAM);
```

The details:
- **When.** The step runs immediately after the reading is recorded, before
  the next effect, in rule order.
- **Guard.** It inherits the sample's guard, as a procedure step inherits its
  call's guard.
- **Semantics.** It is an ordinary reopening. Its provenance, reopening
  relation, journal entry, skipped-effect dependencies, execution-budget
  charge and rollback are exactly those of the same authored rule in that
  position. The decision's journal names the triggering reading.
- **Order.** Series are processed in name order. A series with several
  matching triggers runs once.
- **New observation, not new value.** A reading whose value repeats the last
  one still triggers.

So:

| Sequence | Result |
| --- | --- |
| a matching reading, then the first commitment | the reading does not reopen the later commitment |
| a commitment, then a matching reading | the commitment reopens at once |
| two matching readings while one revision is in force | the first reopens it; the second finds it already open and adds nothing |
| a reading, a new commitment, another reading | each reading reopens the revision in force when it was taken |

The third row differs from an unconditional authored `reopen`, which would
record the second reading as another cause. A trigger reopens only a revision
still in force.

## What stays authored

- Any other condition, such as "and the pull request has not been merged
  elsewhere", or a value test. The ledger keeps its own rules for exactly this
  reason: both of its reopening rules also require `$p_merged == 0`.
- Marking current values as stale.
- Deciding that readings are about the same subject.

## Save

Triggers come from the source, so saves hold nothing new. A reopening they
cause is recorded like any other. Restoring accepts it because the event that
made it samples, directly or through a procedure, into a stream the decision
watches, with a matching relation and claim. An event that could not have
triggered the reopening is still refused.
