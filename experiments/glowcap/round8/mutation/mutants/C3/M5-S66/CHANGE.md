# C3 M5 S66

Site: bind diary.c2 (statement at line 538): add to `because rd2` evidence the value never reads

File: `glowcap.cav`
Line: 538 (line numbers are those of the original file)

Line 538, before:

```
bind diary.c2 = 1 when rtype(r2) >= 5 and rd2 > 0 because rd2;
```

Line 538, after:

```
bind diary.c2 = 1 when rtype(r2) >= 5 and rd2 > 0 because rd2, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
