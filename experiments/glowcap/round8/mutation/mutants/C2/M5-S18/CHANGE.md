# C2 M5 S18

Site: bind $m.label (statement at line 332): add to `because nothing` evidence the value never reads

File: `glowcap.cav`
Line: 332 (line numbers are those of the original file)

Line 332, before:

```
    bind $m.label = "" when $m_consumed because nothing;
```

Line 332, after:

```
    bind $m.label = "" when $m_consumed because observed(a_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
