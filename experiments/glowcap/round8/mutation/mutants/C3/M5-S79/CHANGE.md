# C3 M5 S79

Site: bind diary.c3 (statement at line 558): add to `because e3, e4` evidence the value never reads

File: `glowcap.cav`
Line: 558 (line numbers are those of the original file)

Line 558, before:

```
bind diary.c3 = 1 when rtype(r3) == 3 and rs3 == 4 and e3 + e4 > 0 because e3, e4;
```

Line 558, after:

```
bind diary.c3 = 1 when rtype(r3) == 3 and rs3 == 4 and e3 + e4 > 0 because e3, e4, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
