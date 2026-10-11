# C3 M5 S62

Site: bind diary.c2 (statement at line 534): add to `because e2, e3` evidence the value never reads

File: `glowcap.cav`
Line: 534 (line numbers are those of the original file)

Line 534, before:

```
bind diary.c2 = 1 when rtype(r2) == 3 and rs2 == 3 and e2 + e3 > 0 because e2, e3;
```

Line 534, after:

```
bind diary.c2 = 1 when rtype(r2) == 3 and rs2 == 3 and e2 + e3 > 0 because e2, e3, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
