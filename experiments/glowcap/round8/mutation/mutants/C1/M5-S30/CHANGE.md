# C1 M5 S30

Site: bind journal.cites6 (statement at line 469): add to `because jcite6` evidence the value never reads

File: `glowcap.cav`
Line: 469 (line numbers are those of the original file)

Line 469, before:

```
bind journal.cites6 = jcite6 when j6 > 0 because jcite6;
```

Line 469, after:

```
bind journal.cites6 = jcite6 when j6 > 0 because jcite6, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
