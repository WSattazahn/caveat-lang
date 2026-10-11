# C2 M5 S1

Site: bind $m.label (statement at line 306): add to `because nothing` evidence the value never reads

File: `glowcap.cav`
Line: 306 (line numbers are those of the original file)

Line 306, before:

```
    bind $m.label = "Glowing mushroom" when belief_none because nothing;
```

Line 306, after:

```
    bind $m.label = "Glowing mushroom" when belief_none because observed(a_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
