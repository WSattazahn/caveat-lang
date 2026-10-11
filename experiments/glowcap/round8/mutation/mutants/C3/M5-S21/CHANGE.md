# C3 M5 S21

Site: bind $m.whyAbsorb (statement at line 419): add to `because nothing` evidence the value never reads

File: `glowcap.cav`
Line: 419 (line numbers are those of the original file)

Line 419, before:

```
    bind $m.whyAbsorb = "Marked to avoid" when $m_marked == 1 because nothing;
```

Line 419, after:

```
    bind $m.whyAbsorb = "Marked to avoid" when $m_marked == 1 because observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
