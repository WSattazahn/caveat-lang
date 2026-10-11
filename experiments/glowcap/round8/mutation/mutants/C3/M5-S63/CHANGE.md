# C3 M5 S63

Site: bind diary.c2 (statement at line 535): add to `because e3, e4` evidence the value never reads

File: `glowcap.cav`
Line: 535 (line numbers are those of the original file)

Line 535, before:

```
bind diary.c2 = 1 when rtype(r2) == 3 and rs2 == 4 and e3 + e4 > 0 because e3, e4;
```

Line 535, after:

```
bind diary.c2 = 1 when rtype(r2) == 3 and rs2 == 4 and e3 + e4 > 0 because e3, e4, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
