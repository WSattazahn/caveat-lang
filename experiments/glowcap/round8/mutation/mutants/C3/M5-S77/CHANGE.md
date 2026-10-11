# C3 M5 S77

Site: bind diary.c3 (statement at line 556): add to `because e1, e2` evidence the value never reads

File: `glowcap.cav`
Line: 556 (line numbers are those of the original file)

Line 556, before:

```
bind diary.c3 = 1 when rtype(r3) == 3 and rs3 == 2 and e1 + e2 > 0 because e1, e2;
```

Line 556, after:

```
bind diary.c3 = 1 when rtype(r3) == 3 and rs3 == 2 and e1 + e2 > 0 because e1, e2, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
