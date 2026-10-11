# C3 M5 S94

Site: bind diary.c4 (statement at line 580): add to `because e2, e3` evidence the value never reads

File: `glowcap.cav`
Line: 580 (line numbers are those of the original file)

Line 580, before:

```
bind diary.c4 = 1 when rtype(r4) == 3 and rs4 == 3 and e2 + e3 > 0 because e2, e3;
```

Line 580, after:

```
bind diary.c4 = 1 when rtype(r4) == 3 and rs4 == 3 and e2 + e3 > 0 because e2, e3, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
