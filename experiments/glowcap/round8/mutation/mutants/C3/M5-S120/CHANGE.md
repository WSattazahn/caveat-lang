# C3 M5 S120

Site: bind diary.c6 (statement at line 620): add to `because e3` evidence the value never reads

File: `glowcap.cav`
Line: 620 (line numbers are those of the original file)

Line 620, before:

```
bind diary.c6 = 1 when single(r6) and rs6 == 3 and e3 > 0 because e3;
```

Line 620, after:

```
bind diary.c6 = 1 when single(r6) and rs6 == 3 and e3 > 0 because e3, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
