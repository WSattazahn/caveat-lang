# C3 M5 S78

Site: bind diary.c3 (statement at line 557): add to `because e2, e3` evidence the value never reads

File: `glowcap.cav`
Line: 557 (line numbers are those of the original file)

Line 557, before:

```
bind diary.c3 = 1 when rtype(r3) == 3 and rs3 == 3 and e2 + e3 > 0 because e2, e3;
```

Line 557, after:

```
bind diary.c3 = 1 when rtype(r3) == 3 and rs3 == 3 and e2 + e3 > 0 because e2, e3, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
