# C3 M5 S19

Site: bind $m.whyAbsorb (statement at line 417): add to `because nothing` evidence the value never reads

File: `glowcap.cav`
Line: 417 (line numbers are those of the original file)

Line 417, before:

```
    bind $m.whyAbsorb = "" because nothing;
```

Line 417, after:

```
    bind $m.whyAbsorb = "" because observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
