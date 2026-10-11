# C3 M5 S82

Site: bind diary.c3 (statement at line 561): add to `because rd3` evidence the value never reads

File: `glowcap.cav`
Line: 561 (line numbers are those of the original file)

Line 561, before:

```
bind diary.c3 = 1 when rtype(r3) >= 5 and rd3 > 0 because rd3;
```

Line 561, after:

```
bind diary.c3 = 1 when rtype(r3) >= 5 and rd3 > 0 because rd3, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
