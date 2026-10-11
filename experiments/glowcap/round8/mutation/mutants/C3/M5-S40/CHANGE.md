# C3 M5 S40

Site: bind diary.c1 (statement at line 505): add to `because e3` evidence the value never reads

File: `glowcap.cav`
Line: 505 (line numbers are those of the original file)

Line 505, before:

```
bind diary.c1 = 1 when single(r1) and rs1 == 3 and e3 > 0 because e3;
```

Line 505, after:

```
bind diary.c1 = 1 when single(r1) and rs1 == 3 and e3 > 0 because e3, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
