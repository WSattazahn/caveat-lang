# C3 M5 S58

Site: bind diary.c2 (statement at line 530): add to `because e5` evidence the value never reads

File: `glowcap.cav`
Line: 530 (line numbers are those of the original file)

Line 530, before:

```
bind diary.c2 = 1 when single(r2) and rs2 == 5 and e5 > 0 because e5;
```

Line 530, after:

```
bind diary.c2 = 1 when single(r2) and rs2 == 5 and e5 > 0 because e5, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
