# C3 M5 S125

Site: bind diary.c6 (statement at line 625): add to `because e1, e2` evidence the value never reads

File: `glowcap.cav`
Line: 625 (line numbers are those of the original file)

Line 625, before:

```
bind diary.c6 = 1 when rtype(r6) == 3 and rs6 == 2 and e1 + e2 > 0 because e1, e2;
```

Line 625, after:

```
bind diary.c6 = 1 when rtype(r6) == 3 and rs6 == 2 and e1 + e2 > 0 because e1, e2, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
