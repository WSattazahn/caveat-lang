# C3 M5 S96

Site: bind diary.c4 (statement at line 582): add to `because e4, e5` evidence the value never reads

File: `glowcap.cav`
Line: 582 (line numbers are those of the original file)

Line 582, before:

```
bind diary.c4 = 1 when rtype(r4) == 3 and rs4 == 5 and e4 + e5 > 0 because e4, e5;
```

Line 582, after:

```
bind diary.c4 = 1 when rtype(r4) == 3 and rs4 == 5 and e4 + e5 > 0 because e4, e5, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
