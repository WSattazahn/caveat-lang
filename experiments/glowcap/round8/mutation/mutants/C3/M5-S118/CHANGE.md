# C3 M5 S118

Site: bind diary.c6 (statement at line 618): add to `because e1` evidence the value never reads

File: `glowcap.cav`
Line: 618 (line numbers are those of the original file)

Line 618, before:

```
bind diary.c6 = 1 when single(r6) and rs6 == 1 and e1 > 0 because e1;
```

Line 618, after:

```
bind diary.c6 = 1 when single(r6) and rs6 == 1 and e1 > 0 because e1, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
