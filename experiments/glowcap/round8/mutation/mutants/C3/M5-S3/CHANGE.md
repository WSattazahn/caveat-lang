# C3 M5 S3

Site: bind $m.label (statement at line 390): add to `because contradiction` evidence the value never reads

File: `glowcap.cav`
Line: 390 (line numbers are those of the original file)

Line 390, before:

```
    bind $m.label = "Probably a duskcap" when belief == 2 because contradiction;
```

Line 390, after:

```
    bind $m.label = "Probably a duskcap" when belief == 2 because contradiction, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
