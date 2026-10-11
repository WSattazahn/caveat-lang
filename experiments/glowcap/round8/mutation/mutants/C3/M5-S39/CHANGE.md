# C3 M5 S39

Site: bind diary.c1 (statement at line 504): add to `because e2` evidence the value never reads

File: `glowcap.cav`
Line: 504 (line numbers are those of the original file)

Line 504, before:

```
bind diary.c1 = 1 when single(r1) and rs1 == 2 and e2 > 0 because e2;
```

Line 504, after:

```
bind diary.c1 = 1 when single(r1) and rs1 == 2 and e2 > 0 because e2, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
