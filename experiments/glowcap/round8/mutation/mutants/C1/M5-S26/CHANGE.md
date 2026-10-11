# C1 M5 S26

Site: bind journal.cites2 (statement at line 465): add to `because jcite2` evidence the value never reads

File: `glowcap.cav`
Line: 465 (line numbers are those of the original file)

Line 465, before:

```
bind journal.cites2 = jcite2 when j2 > 0 because jcite2;
```

Line 465, after:

```
bind journal.cites2 = jcite2 when j2 > 0 because jcite2, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
