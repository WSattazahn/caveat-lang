# CAVEAT Procedure Symbols 0.1

A procedure's parameters were numbers. Round 6 of the
[glowcap benchmark](../experiments/glowcap/RESULTS.md) added a second way to
observe a mushroom: watching another creature eat one. Everything the slime
learns from it is what it learns from an absorption, but the rules differ in
which evidence they name, and a procedure could not take evidence. So the twelve
absorb rules were written out again for `witness`.

A parameter can now name a graph symbol:

```caveat
proc learn(e evidence, sort) {
    when sort == sort.glowcap reveal e supports glowing_is_safe;
    when sort == sort.glowcap set support = support + qualified(1, e);
    when sort == sort.duskcap reveal e opposes glowing_is_safe;
    when sort == sort.duskcap and committed(trust) reopen trust because e;
};

for mushroom as $m {
    evidence absorb_$m from "the slime absorbed it";
    evidence witness_$m from "the slime saw it eaten";
    secondhand qualifies witness_$m;
    on absorb when target == $index call learn(absorb_$m, sort);
    on witness when target == $index call learn(witness_$m, sort);
};
```

The caveat that makes a witness secondhand is declared once, on the evidence,
and reaches everything the procedure builds on it.

## Syntax

```text
proc NAME(PARAMETER, ...) { STEPS };
PARAMETER := NAME | NAME evidence | NAME claim | NAME caveat
           | NAME readings | NAME decisions
```

In the body a symbol parameter stands wherever a name of its kind can: in
`reveal`, `qualify`, `reopen … because`, `examine`, `commit … retaining`,
`sample … supports`, in `qualified(…)` and the graph predicates, and as an
argument that passes it on to another procedure's symbol parameter. Numeric
parameters keep their meaning: frozen numbers, evaluated once per call.

## Histories

A `readings` parameter names a declared reading stream, and a `decisions`
parameter a declared decision series. Each stands wherever its history's name
can: `sample S = …`, `latest`, `history_count`, `history_at`, `fold_history`
and `has_sample` for a stream; `commit D`, `reopen D`, `latest`,
`history_count`, `history_at`, `fold_history`, `committed` and `reopened` for
a series; and `reopen … because latest(S)`. The second instrument of
[authoring trial v3](../experiments/agent-authoring/v3/RESULTS.md) needed one:

```caveat
proc take(s readings, d decisions, cm) {
    when history_count(s) >= 8 reject "That instrument is done.";
    when cm >= 10 sample s = cm supports ice_safe;
    when cm < 10 sample s = cm opposes ice_safe;
    when cm < 10 and committed(d) and not reopened(d) and latest(d) >= 10
        reopen d because latest(s);
};
on measure call take(thickness, rink, cm);
on scan call take(sonar_thickness, rink, cm);
```

Each stream keeps its own evidence, caveats and limit, which its declaration
fixes. A parameter names a history; it does not create one.

## Meaning: one procedure per set of names

Symbols are not values. A call passes names, fixed in the source, so when the
program loads each call is replaced by a call of a **specialization**: the
procedure with the names substituted, created once for each distinct set of
names and called `NAME[absorb_cave]`, the name that errors report. From then on
it is an ordinary procedure: validated, costed and run exactly as if it had been
written out by hand. The runtime, provenance and transactions are unchanged.

- An argument for a symbol parameter must be the bare name of a declared symbol
  or history of that kind, or, inside another procedure, that procedure's own
  symbol parameter. Anything else is an error when the program loads.
- A symbol parameter's name cannot also be a declared symbol, history, state or
  constant, or another parameter of the same procedure: a body that says `e`
  must mean one thing.
- Specializations draw on the declared symbols, so creating them always ends.
  A cycle through procedures is rejected as before, and specialization adds at
  most 1,024 procedures.
- A procedure with symbol parameters that is never called is checked for syntax
  only: its body names nothing until a call supplies the names.

In a bundle, a module can offer such a procedure (`call senses::learn(chart,
safe)`), and its symbol parameters are guarded against being fused with the
module's declarations like any other parameter.

## Changes

- `readings` and `decisions` parameters, after authoring trial v3. Both
  maintainers there wrote a second instrument's rules out again, and one noted
  that a procedure could not take a reading stream. Execution is unchanged: a
  specialization renames histories as it renames symbols.
