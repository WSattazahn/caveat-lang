# C3 M5 S97

Site: bind diary.c4 (statement at line 583): add to `because e5, e6` evidence the value never reads

File: `glowcap.cav`
Line: 583 (line numbers are those of the original file)

Line 583, before:

```
bind diary.c4 = 1 when rtype(r4) == 3 and rs4 == 6 and e5 + e6 > 0 because e5, e6;
```

Line 583, after:

```
bind diary.c4 = 1 when rtype(r4) == 3 and rs4 == 6 and e5 + e6 > 0 because e5, e6, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
