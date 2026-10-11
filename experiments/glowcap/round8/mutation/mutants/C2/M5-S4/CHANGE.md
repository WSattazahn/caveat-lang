# C2 M5 S4

Site: bind $m.label (statement at line 309): add to `because contradiction` evidence the value never reads

File: `glowcap.cav`
Line: 309 (line numbers are those of the original file)

Line 309, before:

```
    bind $m.label = "Could be a duskcap — taste first" when belief_uncertain because contradiction;
```

Line 309, after:

```
    bind $m.label = "Could be a duskcap — taste first" when belief_uncertain because contradiction, observed(a_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
