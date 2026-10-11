# C3 M5 S90

Site: bind diary.c4 (statement at line 576): add to `because e5` evidence the value never reads

File: `glowcap.cav`
Line: 576 (line numbers are those of the original file)

Line 576, before:

```
bind diary.c4 = 1 when single(r4) and rs4 == 5 and e5 > 0 because e5;
```

Line 576, after:

```
bind diary.c4 = 1 when single(r4) and rs4 == 5 and e5 > 0 because e5, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
