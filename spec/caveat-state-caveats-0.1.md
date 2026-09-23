# CAVEAT State Caveats 0.1

An evidence name can be renewed while a plan still rests on its previous
occurrence. `carries(sighting, stale)` asks about today's occurrence. A live
state that retained the plan's basis needs to ask about the evidence it
actually used, and name those exact occurrences when reconsidering the plan.

This profile exposes information the runtime already keeps. It adds no timer,
history, or saved-state table.

## Inspecting live grounds

```caveat
has_caveat(STATE, CAVEAT)
```

True exactly when the named numeric state's **grounds** include the named
caveat. Both names are checked when the program loads. `STATE` is a declared
state, not an arbitrary expression, event parameter, or evidence name.

- Late and scheduled qualifications already update the state's grounds for
  each concrete occurrence it uses. Renewing an evidence alias does not change
  those grounds. The predicate therefore sees an older occurrence becoming
  stale even while a newer occurrence stays fresh.
- A caveat carried only by the state's control lineage does not make the
  answer true. A guard that happened to read stale evidence is not content.
- The answer's lineage is the state's full lineage, whether true or false.
  When read for grounds, it supplies the state's grounds. Grounded explanations
  can cite the answer without discarding its audit dependencies.
- Replacing a state's grounds reevaluates bindings that read the predicate,
  even when its number and the graph are unchanged.
- It can occur in expressions, bindings, `define`, procedure guards and state
  initializers reading an earlier state. Pure functions still cannot capture
  state. A procedure may pass the caveat through a `CAVEAT caveat` parameter;
  the state remains a declared name.

## Selecting reopening witnesses

```caveat
reopen ACTION because caveated(STATE, CAVEAT)
```

Select every observed evidence identity in the state's grounds that **currently
carries** the caveat, directly, through another caveat, or through a claim it
bears on. These are concrete identities: the runtime never turns an old
`sighting` back into the newer `sighting@2` by resolving its renewable alias.
The selection is ordered by first observation, like the decision journal.

An empty selection rejects the entire event without changing anything. A
state may have an authored extra caveat from `qualified(value, evidence,
extra)` without that caveat qualifying the evidence itself. `has_caveat` can
then be true while `caveated` finds no witness; the selector does not invent
one.

All newly added reopening witnesses from one selector execution produce one
`reopened` journal entry. It records their union of caveats **now**. Individual
reopening graph edges and effect reports retain their existing shape. Already
recorded witnesses are omitted; selecting only previously recorded witnesses
is idempotent and appends no entry. The state's full lineage and the selected
evidence's current qualifications qualify the reopening, together with the
usual guard and decision-selection dependencies.

The selector does not automatically guard against an already reopened plan.
That is a policy choice, expressed as `not reopened(ACTION)`. Named evidence
and `latest(STREAM)` reopening selectors keep their existing behavior.

## Example

```caveat
state plan_basis = 0;
decisions plan limit 8;

on choose set plan_basis = latest_assessment;
on choose commit plan because enough using plan_basis;

on advance when committed(plan) and not reopened(plan)
    and has_caveat(plan_basis, stale)
    reopen plan because caveated(plan_basis, stale);
```

Here `latest_assessment` is a program-owned state. When its evidence ages,
the separate live `plan_basis` receives the caveat even if the assessment has
since changed. The commitment's original grounds and all earlier journal
entries stay frozen. A host reads the runtime's journal for the ordered
reopening causes rather than reconstructing them from unordered grounds.

Both operations work through module name linking and typed procedure
specialization. Existing save/restore already preserves the states, exact
evidence identities, qualifications and journal they read. The browser bridge
uses the same evaluator and view; no host gameplay rule is required.
