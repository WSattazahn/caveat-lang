# C3 M5 S54

Site: bind diary.c2 (statement at line 526): add to `because e1` evidence the value never reads

File: `glowcap.cav`
Line: 526 (line numbers are those of the original file)

Line 526, before:

```
bind diary.c2 = 1 when single(r2) and rs2 == 1 and e1 > 0 because e1;
```

Line 526, after:

```
bind diary.c2 = 1 when single(r2) and rs2 == 1 and e1 > 0 because e1, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
