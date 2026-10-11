# C1 M5 S29

Site: bind journal.cites5 (statement at line 468): add to `because jcite5` evidence the value never reads

File: `glowcap.cav`
Line: 468 (line numbers are those of the original file)

Line 468, before:

```
bind journal.cites5 = jcite5 when j5 > 0 because jcite5;
```

Line 468, after:

```
bind journal.cites5 = jcite5 when j5 > 0 because jcite5, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
