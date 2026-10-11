# C3 M5 S124

Site: bind diary.c6 (statement at line 624): add to `because e0, e1` evidence the value never reads

File: `glowcap.cav`
Line: 624 (line numbers are those of the original file)

Line 624, before:

```
bind diary.c6 = 1 when rtype(r6) == 3 and rs6 == 1 and e0 + e1 > 0 because e0, e1;
```

Line 624, after:

```
bind diary.c6 = 1 when rtype(r6) == 3 and rs6 == 1 and e0 + e1 > 0 because e0, e1, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
