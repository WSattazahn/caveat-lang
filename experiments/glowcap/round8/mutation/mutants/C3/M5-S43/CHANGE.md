# C3 M5 S43

Site: bind diary.c1 (statement at line 508): add to `because e6` evidence the value never reads

File: `glowcap.cav`
Line: 508 (line numbers are those of the original file)

Line 508, before:

```
bind diary.c1 = 1 when single(r1) and rs1 == 6 and e6 > 0 because e6;
```

Line 508, after:

```
bind diary.c1 = 1 when single(r1) and rs1 == 6 and e6 > 0 because e6, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
