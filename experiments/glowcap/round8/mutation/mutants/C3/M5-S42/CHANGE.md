# C3 M5 S42

Site: bind diary.c1 (statement at line 507): add to `because e5` evidence the value never reads

File: `glowcap.cav`
Line: 507 (line numbers are those of the original file)

Line 507, before:

```
bind diary.c1 = 1 when single(r1) and rs1 == 5 and e5 > 0 because e5;
```

Line 507, after:

```
bind diary.c1 = 1 when single(r1) and rs1 == 5 and e5 > 0 because e5, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
