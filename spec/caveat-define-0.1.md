# CAVEAT Define 0.1

Conditions repeat. In the glowcap beat, `support > 0 and contradiction == 0`
appeared once for each binding that depended on the belief, and every label
guard for a mushroom began with the same `consumed == 0 and known == 0`. Pure
functions cannot help, because by design they cannot read state or the graph.

```caveat
define probably_safe = support > 0 and contradiction == 0;
define uncertain = support > 0 and contradiction > 0;

for mushroom as $m {
    define $m_unknown = $m_consumed == 0 and $m_known == 0;
    bind $m.label = "Probably a glowcap" when $m_unknown and probably_safe because support;
};
```

## Syntax and meaning

```text
define NAME = EXPRESSION;
```

A define names an expression over state, event parameters, graph predicates,
histories, functions and other defines. Wherever NAME is read, in a rule guard,
a value, a procedure step, a binding, a citation or another define, the
expression is inlined before validation. Values, lineage and grounds are
exactly those of writing the expression there by hand. A define holds no
value, has no event of its own, and appears nowhere in the snapshot.

Inside a `for` block, `define $m_x = …` expands per member like any other
statement.

## Checks

The following are rejected before execution:
- a define that refers to itself, directly or through other defines;
- a name that collides with a state, function, history or event parameter;
- a duplicate name.

A define is validated even when unused, like a function. Pure functions still
cannot capture state, so they cannot read a define. `define` is reserved, and
in a module a define's name is the module's own, like a state's.
