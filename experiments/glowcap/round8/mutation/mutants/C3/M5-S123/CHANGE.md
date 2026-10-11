# C3 M5 S123

Site: bind diary.c6 (statement at line 623): add to `because e6` evidence the value never reads

File: `glowcap.cav`
Line: 623 (line numbers are those of the original file)

Line 623, before:

```
bind diary.c6 = 1 when single(r6) and rs6 == 6 and e6 > 0 because e6;
```

Line 623, after:

```
bind diary.c6 = 1 when single(r6) and rs6 == 6 and e6 > 0 because e6, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
