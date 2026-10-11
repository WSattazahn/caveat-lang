# C3 M5 S109

Site: bind diary.c5 (statement at line 602): add to `because e1, e2` evidence the value never reads

File: `glowcap.cav`
Line: 602 (line numbers are those of the original file)

Line 602, before:

```
bind diary.c5 = 1 when rtype(r5) == 3 and rs5 == 2 and e1 + e2 > 0 because e1, e2;
```

Line 602, after:

```
bind diary.c5 = 1 when rtype(r5) == 3 and rs5 == 2 and e1 + e2 > 0 because e1, e2, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
