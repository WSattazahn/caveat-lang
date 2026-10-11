# C3 M5 S1

Site: bind $m.label (statement at line 388): add to `because nothing` evidence the value never reads

File: `glowcap.cav`
Line: 388 (line numbers are those of the original file)

Line 388, before:

```
    bind $m.label = "Glowing mushroom" because nothing;
```

Line 388, after:

```
    bind $m.label = "Glowing mushroom" because observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
