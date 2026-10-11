# C3 M5 S56

Site: bind diary.c2 (statement at line 528): add to `because e3` evidence the value never reads

File: `glowcap.cav`
Line: 528 (line numbers are those of the original file)

Line 528, before:

```
bind diary.c2 = 1 when single(r2) and rs2 == 3 and e3 > 0 because e3;
```

Line 528, after:

```
bind diary.c2 = 1 when single(r2) and rs2 == 3 and e3 > 0 because e3, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
