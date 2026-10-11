# C3 M5 S18

Site: bind $m.label (statement at line 414): add to `because nothing` evidence the value never reads

File: `glowcap.cav`
Line: 414 (line numbers are those of the original file)

Line 414, before:

```
    bind $m.label = "" when $m_eaten because nothing;
```

Line 414, after:

```
    bind $m.label = "" when $m_eaten because observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
