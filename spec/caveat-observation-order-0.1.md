# CAVEAT Observation Order 0.1

A value can rest only on evidence that has been observed. Rules run in source
order, so a rule that qualifies a value with evidence a *later* rule of the same
event reveals fails when it runs. Round 6 of the
[glowcap benchmark](../experiments/glowcap/RESULTS.md) hit exactly this:

```caveat
on absorb when $m_here set $m_consumed = qualified(1, absorb_$m);   -- rule 10
on absorb when $m_here and sort == sort.glowcap reveal absorb_$m supports glowing_is_safe;
```

The error came on the first absorb, as a runtime failure. It was a certain
failure, knowable from the source alone, so it now comes when the program loads:

```text
event absorb, rule 10: qualified(…, absorb_cave) runs before rule 12 reveals absorb_cave in
the same event, so it can only fail; reveal absorb_cave first, or guard the rule with
observed(absorb_cave)
```

## What is rejected

Only uses that are **certain** to fail. A use is a place that needs evidence
`E` observed when it runs:

- `qualified(…, E)` where evaluating the rule's guard or effect is sure to reach
  it: not inside an `if` branch, the right side of `and` or `or`, a `require`
  value, or a fold body. A `because` citation counts;
- `qualify E with CAVEAT`;
- `reopen ACTION because E`.

Procedure bodies are checked where they run, with the calling rule's guard
carried into each step. The program is rejected when a use of `E` in event `X`
meets all of these:

1. `E` is not observed when the program loads (no declared relation reveals it);
2. no rule of any other event can reveal `E`;
3. no earlier step of `X` can reveal `E`;
4. the rule's guard does not mention `observed(E)`;
5. either nothing in `X` reveals `E` either, so nothing ever observes it; or every
   later step of `X` that reveals `E` has, among its `and`-separated
   conditions, every condition the use was guarded by, and each of those reads
   only the event's parameters and constants.

In the second case, any dispatch that reaches a reveal of `E` passed the use
first with `E` unobserved, because the conditions it shares cannot change during
the event. So `E` can never become observed, and the use can never succeed.

## What is left alone

Anything that could succeed on some dispatch loads, and the runtime reports
it if it fails:

- a guard that reads state or the graph, such as `when count > 0`, which may be
  false the first time and let a later rule reveal `E`;
- a reveal under a different guard, such as another target's;
- a use in a branch that may not be taken;
- evidence revealed by another event or observed when the program loads;
- a rule guarded by `observed(E)`, which says the author knows.

Bindings are not checked here: they run after every rule, so order within an
event does not apply to them.
