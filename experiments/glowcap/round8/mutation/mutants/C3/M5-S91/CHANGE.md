# C3 M5 S91

Site: bind diary.c4 (statement at line 577): add to `because e6` evidence the value never reads

File: `glowcap.cav`
Line: 577 (line numbers are those of the original file)

Line 577, before:

```
bind diary.c4 = 1 when single(r4) and rs4 == 6 and e6 > 0 because e6;
```

Line 577, after:

```
bind diary.c4 = 1 when single(r4) and rs4 == 6 and e6 > 0 because e6, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
