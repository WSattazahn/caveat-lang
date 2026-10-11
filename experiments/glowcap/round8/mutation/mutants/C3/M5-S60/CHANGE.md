# C3 M5 S60

Site: bind diary.c2 (statement at line 532): add to `because e0, e1` evidence the value never reads

File: `glowcap.cav`
Line: 532 (line numbers are those of the original file)

Line 532, before:

```
bind diary.c2 = 1 when rtype(r2) == 3 and rs2 == 1 and e0 + e1 > 0 because e0, e1;
```

Line 532, after:

```
bind diary.c2 = 1 when rtype(r2) == 3 and rs2 == 1 and e0 + e1 > 0 because e0, e1, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
