# C3 M5 S81

Site: bind diary.c3 (statement at line 560): add to `because e5, e6` evidence the value never reads

File: `glowcap.cav`
Line: 560 (line numbers are those of the original file)

Line 560, before:

```
bind diary.c3 = 1 when rtype(r3) == 3 and rs3 == 6 and e5 + e6 > 0 because e5, e6;
```

Line 560, after:

```
bind diary.c3 = 1 when rtype(r3) == 3 and rs3 == 6 and e5 + e6 > 0 because e5, e6, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
