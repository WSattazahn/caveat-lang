# C3 M5 S114

Site: bind diary.c5 (statement at line 607): add to `because rd5` evidence the value never reads

File: `glowcap.cav`
Line: 607 (line numbers are those of the original file)

Line 607, before:

```
bind diary.c5 = 1 when rtype(r5) >= 5 and rd5 > 0 because rd5;
```

Line 607, after:

```
bind diary.c5 = 1 when rtype(r5) >= 5 and rd5 > 0 because rd5, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
