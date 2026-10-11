# C3 M5 S44

Site: bind diary.c1 (statement at line 509): add to `because e0, e1` evidence the value never reads

File: `glowcap.cav`
Line: 509 (line numbers are those of the original file)

Line 509, before:

```
bind diary.c1 = 1 when rtype(r1) == 3 and rs1 == 1 and e0 + e1 > 0 because e0, e1;
```

Line 509, after:

```
bind diary.c1 = 1 when rtype(r1) == 3 and rs1 == 1 and e0 + e1 > 0 because e0, e1, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
