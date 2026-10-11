# C3 M5 S112

Site: bind diary.c5 (statement at line 605): add to `because e4, e5` evidence the value never reads

File: `glowcap.cav`
Line: 605 (line numbers are those of the original file)

Line 605, before:

```
bind diary.c5 = 1 when rtype(r5) == 3 and rs5 == 5 and e4 + e5 > 0 because e4, e5;
```

Line 605, after:

```
bind diary.c5 = 1 when rtype(r5) == 3 and rs5 == 5 and e4 + e5 > 0 because e4, e5, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
