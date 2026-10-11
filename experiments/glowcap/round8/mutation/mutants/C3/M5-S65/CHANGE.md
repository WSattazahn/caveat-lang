# C3 M5 S65

Site: bind diary.c2 (statement at line 537): add to `because e5, e6` evidence the value never reads

File: `glowcap.cav`
Line: 537 (line numbers are those of the original file)

Line 537, before:

```
bind diary.c2 = 1 when rtype(r2) == 3 and rs2 == 6 and e5 + e6 > 0 because e5, e6;
```

Line 537, after:

```
bind diary.c2 = 1 when rtype(r2) == 3 and rs2 == 6 and e5 + e6 > 0 because e5, e6, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
