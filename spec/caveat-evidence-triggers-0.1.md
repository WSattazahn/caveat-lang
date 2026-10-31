# CAVEAT Evidence Triggers 0.1 — reopen a decision when new evidence on its claim arrives

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Proposed for 1.5 and not implemented: it becomes stable when a 1.x release ships it. It adds to
[Reopening Triggers 0.1](caveat-reopening-triggers-0.1.md) and
[Changed Triggers 0.1](caveat-changed-triggers-0.1.md); it changes no
program that loads today.

[Reopening Triggers 0.1](caveat-reopening-triggers-0.1.md) lets a decision
series declare what reopens it, but only a reading in a stream can trigger
it. Much evidence is not a stream reading. A paper, a report or a review is
declared evidence, revealed once with its stance on a claim. When such
evidence arrives after a decision about that claim, the program needs one
rule for each pair of evidence and decision:

```caveat
claim brenodil_stroke;
evidence p03 from "P03 trial: stroke 1 vs 1";
evidence p40 from "P40 cohort: stroke HR 1.6";
decisions q17 limit 8;
decisions q27 limit 8;
event read;
event publish_p40;
on read reveal p03 opposes brenodil_stroke;
on publish_p40 reveal p40 supports brenodil_stroke;
on publish_p40 when committed(q17) and not reopened(q17) reopen q17 because p40;
on publish_p40 when committed(q27) and not reopened(q27) reopen q27 because p40;
```

Every new paper needs its own copy of those rules, naming each decision it
bears on. A program that forgets one leaves a decision in force on old
evidence, with nothing to show it. That is the problem the reading-stream
triggers solved for streams. This profile lets the declaration say it for
declared evidence too:

```caveat
decisions q17 limit 8 reopened by any evidence on brenodil_stroke;
decisions q27 limit 8 reopened by any evidence on brenodil_stroke;
on read reveal p03 opposes brenodil_stroke;
on publish_p40 reveal p40 supports brenodil_stroke;
```

After `read, decide, publish_p40`, `q17` and `q27` are reopened because
`p40`. A later paper on the same claim needs only its `reveal`.

## Syntax

```text
decisions NAME limit N reopened by TRIGGER, TRIGGER, …;
TRIGGER := [changed] STREAM [supporting CLAIM | opposing CLAIM]   -- as before
         | any evidence supporting CLAIM
         | any evidence opposing CLAIM
         | any evidence on CLAIM
```

`any evidence` is read only as the first two words of a trigger, so it
reserves nothing. Today `reopened by any evidence …` refuses to load, because
a stream named `any` may be followed only by `supporting` or `opposing`. So no
program that loads changes. A stream named `any` keeps both of its forms, and
so does a stream named `evidence`. `CLAIM` must be a declared claim.
`changed` does not combine with `any evidence`: declared evidence has no
previous value to compare.

## Meaning

Whenever a `reveal EVIDENCE supports CLAIM` or `reveal EVIDENCE opposes CLAIM`
makes evidence observed, each series with a matching trigger runs a
reopening step:

```caveat
when committed(D) and not reopened(D) reopen D because EVIDENCE;
```

- **Matching.** `supporting CLAIM` matches a reveal that supports `CLAIM`,
  `opposing CLAIM` one that opposes it, and `on CLAIM` either.
- **Only new observations.** The step runs when the reveal observes the
  evidence's current occurrence for the first time. Revealing evidence that
  is already observed records nothing new, so it triggers nothing. A renewed
  occurrence is new, and it triggers when it is revealed. A neutral reveal
  ([Neutral Reveal 0.1](caveat-neutral-reveal-0.1.md)) takes no stance, so it
  matches no trigger.
- **Everything else as Reopening Triggers 0.1.** The step runs straight
  after the reveal, in rule order, inheriting the reveal's guard. It is an
  ordinary reopening, with the same provenance, journal entry, budget charge
  and rollback as the same authored rule. Series run in name order, and a
  series with several matching triggers runs once. A trigger reopens only a
  revision still in force.

A withdrawal is not a reveal, and it triggers nothing. A program still states
what a withdrawal means for its decisions
([Withdrawal 0.1](caveat-withdrawal-0.1.md), "Nothing reopens by itself").

## What stays authored

As in Reopening Triggers 0.1: any other condition, value tests, marking values
stale, and deciding that two claims are about the same subject. A decision
that rests on several claims declares a trigger for each.

## Check

C002 `no-reopening-path` counts an `any evidence` trigger as a reopening path
when some rule reveals evidence with a matching stance.

## Save

Triggers come from the source, so saves hold nothing new. A reopening they
cause is recorded like any other. Restoring accepts it because the event that
made it reveals, directly or through a procedure, evidence with a matching
stance on the claim, observed for the first time. An event that could not
have triggered the reopening is still refused.

## Not in this profile

- **Triggers on withdrawal or reinstatement.** These are author decisions
  today, and stay so.
- **Triggers on qualification.** A late `qualify` changes what a decision
  carries, not what it rests on. [Late Qualification 0.1](caveat-late-qualification-0.1.md)
  already shows it on current values.

## Implementation (when scheduled)

- The runtime trigger table gains claim-keyed entries for reveals, beside
  the stream-keyed ones, and the restore check gains the reveal case.
- A corpus sweep confirms that no tracked program changes. A save sweep over
  all tracked programs gives the same digest before and after.
- Scenario cases named after this profile: supporting, opposing and `on`;
  a repeated reveal; a renewed occurrence; a neutral reveal; a trigger inside
  a procedure; restore of a triggered reopening.
- The kit reference, the authoring guide and the VS Code grammar name the
  form.
