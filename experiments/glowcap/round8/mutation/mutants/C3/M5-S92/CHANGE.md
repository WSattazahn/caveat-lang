# C3 M5 S92

Site: bind diary.c4 (statement at line 578): add to `because e0, e1` evidence the value never reads

File: `glowcap.cav`
Line: 578 (line numbers are those of the original file)

Line 578, before:

```
bind diary.c4 = 1 when rtype(r4) == 3 and rs4 == 1 and e0 + e1 > 0 because e0, e1;
```

Line 578, after:

```
bind diary.c4 = 1 when rtype(r4) == 3 and rs4 == 1 and e0 + e1 > 0 because e0, e1, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
