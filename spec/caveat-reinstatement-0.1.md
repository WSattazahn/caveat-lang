# CAVEAT Reinstatement 0.1 — standing behind withdrawn evidence again

**Status:** not part of the [compatibility commitment](../docs/COMPATIBILITY.md#what-is-not-stable); it may change in any release. Proposed for 1.5 and not implemented: it becomes stable when a 1.x release ships it. It adds to
[Withdrawal 0.1](caveat-withdrawal-0.1.md),
[Renewal 0.1](caveat-renewal-0.1.md) and
[In Basis 0.1](caveat-in-basis-0.1.md); it changes no program that loads
today.

A withdrawal can itself turn out to be wrong. A data source is found
unreliable, and every study built on it is withdrawn. A later audit clears
part of the source, and two of those studies are sound after all. A test
result is withdrawn as run against the wrong configuration, and the
configuration turns out to have been right.
[Withdrawal 0.1](caveat-withdrawal-0.1.md) lists "undoing a withdrawal" under
"Not in this profile", and nothing else in the language does it. A withdrawal
record is permanent, and `withdrawn` is the one caveat that
[`unqualify`](caveat-take-back-0.1.md) may not take back.

Authors work around it today in three ways, and each one costs something:

1. **A second piece of evidence for the same observation.** The author
   declares `p14_verified` next to `p14` and changes every formula to
   `if(withdrawn(p14), if(observed(p14_verified), qualified(1, p14_verified), 0), qualified(1, p14))`.
   The observation now has two names. Its claim relation (`supports harm`)
   and any caveats added to the first name have to be copied by hand.
   Every rule that reads the evidence must know about both names.
2. **Renewal.** The author declares `renewable p14 limit 2` and sends
   `renew p14` followed by `reveal p14 supports harm`. The new occurrence is
   clean, but renewal means a *new* source, so caveats added by `qualify`
   (an expression of concern, for example) are dropped. Nothing records why
   the evidence came back. The program has to be edited to declare
   `renewable` before the first withdrawal, and a save binds its exact
   source, so a session already running cannot gain it.
3. **A new decision that ignores the withdrawal.** The author changes the
   formula so it reads the evidence whether or not it is withdrawn. The
   answer is right, but every value read from the evidence keeps
   `withdrawn`, and `explain` lists a withdrawn report as a reason for an
   answer that is meant to rest on it soundly.

All three need hand-written rules to reopen the decisions that were made
while the evidence was out. None of them records the reason in a place that
`explain`, a save or `rests_on_withdrawn` can see. This profile adds one effect:

```caveat
claim harm;
caveat concern consequence material;
evidence p14 from "P14 cohort: liver injury HR 1.9";
evidence corvane_finding from "the journal: Corvane data not to be relied on";
evidence corvane_audit from "audit: the 2019-2021 Corvane extract is sound";
decisions liver_signal limit 8;
event read;
event finding;
event worry;
event audit;
event decide;
on read reveal p14 supports harm;
on finding reveal corvane_finding;
on finding withdraw p14 because corvane_finding;
on worry qualify p14 with concern;
on audit reveal corvane_audit;
on audit reinstate p14 because corvane_audit;
on audit when committed(liver_signal) and not reopened(liver_signal)
    and in_basis(liver_signal, occurrence(p14, 1))
    reopen liver_signal because corvane_audit;
on decide commit liver_signal because enough
    using if(withdrawn(p14), 0, qualified(1, p14));
```

After `read, worry, finding, decide, audit, decide`, `liver_signal@1` is 0
and rests on the withdrawn `p14@1`. `liver_signal@2` is 1 and rests on
`p14@2` and `corvane_audit`. It carries `concern`, because the expression
of concern was about the study, and the study is what came back. `explain`
prints:

```text
  p14@1 supports harm  caveats: concern, withdrawn  withdrawn at #3 because corvane_finding
  p14@2 supports harm  caveats: concern  reinstated at #5 because corvane_audit
```

## Syntax

```text
reinstate EVIDENCE because EVIDENCE;        -- an effect, in a rule or procedure step
reinstated(EVIDENCE)                         -- a predicate
```

The target is declared evidence, renewable or not. It may not be a reading
stream's template, and `reinstate latest(STREAM)` does not load (see "Not in
this profile"). The reason is declared evidence.

`reinstate` is read only at the start of an effect. Today a rule whose effect
starts with it does not load ("rule requires set, reveal, examine, commit,
reopen, emit, sample, call, or reject effect"), so no program that loads
changes. An event, state or evidence named `reinstate` keeps loading, and so
does `on reinstate reinstate p14 because audit;`. The predicate
`reinstated(…)` is an unknown source function today, and a program that
defines `fn reinstated` keeps its function, as with `grounds_of`
([Grounds Of 0.1](caveat-grounds-of-0.1.md)).

## What `reinstate` does

When the effect runs, it resolves the target to its current occurrence and
the reason to its current occurrence. The reason must be observed, or the
event is refused as `evaluation/unobserved_evidence`, as `withdraw` is.

- **Not withdrawn: nothing happens.** If the target's current occurrence is
  not withdrawn, the effect does nothing and reports nothing, as
  withdrawing evidence that is already withdrawn does nothing. So
  reinstating twice is harmless, and an author needs no guard against it.
- **A withdrawn reason is refused.** If the reason is itself withdrawn, the
  event is refused as `evaluation/withdrawn_reason`, a new code. A
  reinstatement resting on a reason no longer stood behind would bring the
  evidence back already in doubt, and no later state would show why.

Otherwise, as part of the event's transaction:

1. **A new occurrence.** The target gains a new occurrence `E@k+1`, numbered
   after its latest one. It is the current occurrence from then on, as after
   a `renew`, and every rule and binding that names the evidence means it.
2. **The same observation, observed now.** The new occurrence is observed at
   once, at this event. It takes the withdrawn occurrence's claim relation
   (`supports` or `opposes` and its claim) and every caveat on it except
   `withdrawn`: declared caveats and those a `qualify` added. This is where
   reinstatement differs from renewal. A renewal is a new source and
   inherits only declared caveats. A reinstatement is the same observation
   stood behind again, so what was learned about it still holds.
3. **Grounded on its reason.** A value read from the new occurrence carries
   the reason in its grounds. `qualified(1, p14)` after the reinstatement
   rests on `p14@2` and `corvane_audit`. If the reason is later withdrawn,
   every value read from `p14@2` carries `withdrawn`, and
   `rests_on_withdrawn(D)` is true for a decision made on it. The evidence
   is not withdrawn by that: `withdrawn(p14)` stays false until a program
   withdraws `p14` again. A program that wants both writes
   `withdraw p14 because …` in the same rule.
4. **A record.** The session's `reinstatements` gains
   `{evidence, occurrence, because, sequence, event}`.
5. **Nothing recorded changes.** The withdrawn occurrence keeps its
   withdrawal, its relations and its caveats. Every decision made on it, or
   made while it was out, keeps its frozen basis, grounds and caveats. The
   journal is unchanged.

It is reported in `effects` as
`{"kind": "reinstate", "evidence", "occurrence", "because"}`.

### Nothing reopens by itself

As with withdrawal, a program states what a reinstatement means for its
decisions. The decisions made while the evidence was out are usually the
ones to reopen. They read the withdrawn occurrence, and
`in_basis(D, occurrence(E, N))` ([In Basis 0.1](caveat-in-basis-0.1.md))
finds them by that occurrence. The example above uses it.

### Limit

A target that is declared `renewable … limit N` shares that limit: a
reinstatement is an occurrence, and one past the limit refuses the event as
`limit/renewal_limit`, as `renew` does. Evidence that is not renewable may be
reinstated up to 16 times. Each reinstatement needs a withdrawal before it,
so the limit counts real round trips, not repeats.

## Predicates

- `reinstated(E)` is true when E's current occurrence was made by a
  reinstatement. It carries the reinstatement's reason as its evidence, as
  `withdrawn(E)` carries the withdrawal's reason.
- `withdrawn(E)` reads the current occurrence, so it is false after a
  reinstatement. `withdrawn(occurrence(E, 1))` is still true.
- `rests_on_withdrawn(D)` is unchanged. It is true for a decision made on the
  withdrawn occurrence and false for one made only on the reinstated one,
  unless the reinstatement's reason is later withdrawn (step 3).

## Check

`caveat check` gains no rule. C002 `no-reopening-path` still reports a
series nothing reopens. A program that reinstates evidence and has no rule
that reopens on its reason gets no warning, because whether a decision made
without the evidence must be decided again is the author's call.

## Snapshot and save

The snapshot lists `reinstatements`, in the order they happened. A save holds
the same list and leaves it out when it is empty, so a save made before this
profile restores with none, byte for byte. Restoring refuses a list that:

- names unknown evidence, or an occurrence the save does not hold;
- reinstates an occurrence whose predecessor has no withdrawal record;
- gives a reason that nothing observes, or one withdrawn before the
  reinstatement's sequence;
- is dated after the save's own sequence, or to an undeclared event;
- disagrees with the graph: every reinstated occurrence needs the relations
  and caveats step 2 gives it, and no others;
- appears in a program that has no `reinstate` effect.

## Not in this profile

- **Withdrawn readings.** `reinstate latest(STREAM)` does not load. A
  reading is one sample, and the way to stand behind a value again is a new
  sample, which [Withdrawal 0.1](caveat-withdrawal-0.1.md) already keeps
  free of `withdrawn`.
- **Judging the reason.** Whether an audit is good enough, or whether a
  retracted paper may ever come back, is the program's policy, written as a
  guard: `on audit when observed(p14_retraction) reject "A retracted paper stays out.";`.
- **Withdrawn decisions.** Undoing a decision's revision is the separate
  "undo across revisions" item. Reversible foreclosure
  ([Foreclosure 0.2](caveat-foreclosure-0.2.md)) is about a decision ruling
  options out, not about evidence. Neither covers this profile, and this
  profile covers neither.
- **A whole source.** `withdraw every` ([Untrusted Sources 0.1](caveat-untrusted-sources-0.1.md))
  withdraws everything a source said. Reinstating names one piece of evidence
  at a time, because a partial clearance is the usual case: an audit clears
  one extract of a source and not the rest.

## Invariants kept

- A withdrawal record is never removed or changed (Withdrawal 0.1, step 3).
- `withdrawn` is never taken off an occurrence. A reinstatement makes a new
  occurrence instead.
- A decision keeps the basis, grounds and caveats it was made on.
- Provenance stays true: a value read from the reinstated evidence names the
  occurrence it read and the reason it came back.

## Implementation (when scheduled)

- The runtime, the snapshot and save, View 0.2 and the kit types, as for
  [Take-back 0.1](caveat-take-back-0.1.md).
- A Lean step beside the withdrawal and take-back steps, with the invariants
  above as theorems.
- A corpus sweep that confirms no tracked program changes, and a save sweep
  over all tracked programs that gives the same digest before and after.
- Scenario cases named after this profile: reinstate and decide; reinstate
  something not withdrawn; a withdrawn reason; withdraw the reason after a
  reinstatement; withdraw, reinstate and withdraw again; the limit.
- The kit reference, the authoring guide and the VS Code grammar name the
  effect and the predicate.

## Questions for the owner

These are settled above as the recommended choice, and are listed because
each one changes what programs mean:

1. **Inherit added caveats (step 2).** Recommended: yes, because the
   observation is the same one. The alternative is renewal's rule (declared
   caveats only), which would drop an expression of concern.
2. **A new refusal code for a withdrawn reason.** Recommended:
   `evaluation/withdrawn_reason`. The alternative is to allow it and let the
   new occurrence carry `withdrawn` from step 3 at once. That loads more
   programs but hides the mistake.
3. **The limit for evidence that is not renewable.** Recommended: 16.
