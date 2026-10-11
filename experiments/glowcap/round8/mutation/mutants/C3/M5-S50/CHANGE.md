# C3 M5 S50

Site: bind diary.c1 (statement at line 515): add to `because rd1` evidence the value never reads

File: `glowcap.cav`
Line: 515 (line numbers are those of the original file)

Line 515, before:

```
bind diary.c1 = 1 when rtype(r1) >= 5 and rd1 > 0 because rd1;
```

Line 515, after:

```
bind diary.c1 = 1 when rtype(r1) >= 5 and rd1 > 0 because rd1, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
