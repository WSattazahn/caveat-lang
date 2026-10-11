# C3 M5 S57

Site: bind diary.c2 (statement at line 529): add to `because e4` evidence the value never reads

File: `glowcap.cav`
Line: 529 (line numbers are those of the original file)

Line 529, before:

```
bind diary.c2 = 1 when single(r2) and rs2 == 4 and e4 > 0 because e4;
```

Line 529, after:

```
bind diary.c2 = 1 when single(r2) and rs2 == 4 and e4 > 0 because e4, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
