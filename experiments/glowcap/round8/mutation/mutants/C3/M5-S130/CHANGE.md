# C3 M5 S130

Site: bind diary.c6 (statement at line 630): add to `because rd6` evidence the value never reads

File: `glowcap.cav`
Line: 630 (line numbers are those of the original file)

Line 630, before:

```
bind diary.c6 = 1 when rtype(r6) >= 5 and rd6 > 0 because rd6;
```

Line 630, after:

```
bind diary.c6 = 1 when rtype(r6) >= 5 and rd6 > 0 because rd6, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
