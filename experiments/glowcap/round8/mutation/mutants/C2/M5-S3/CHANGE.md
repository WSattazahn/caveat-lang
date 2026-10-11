# C2 M5 S3

Site: bind $m.label (statement at line 308): add to `because contradiction` evidence the value never reads

File: `glowcap.cav`
Line: 308 (line numbers are those of the original file)

Line 308, before:

```
    bind $m.label = "Probably a duskcap" when belief_unsafe because contradiction;
```

Line 308, after:

```
    bind $m.label = "Probably a duskcap" when belief_unsafe because contradiction, observed(a_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
