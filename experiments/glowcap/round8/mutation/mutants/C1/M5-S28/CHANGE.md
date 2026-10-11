# C1 M5 S28

Site: bind journal.cites4 (statement at line 467): add to `because jcite4` evidence the value never reads

File: `glowcap.cav`
Line: 467 (line numbers are those of the original file)

Line 467, before:

```
bind journal.cites4 = jcite4 when j4 > 0 because jcite4;
```

Line 467, after:

```
bind journal.cites4 = jcite4 when j4 > 0 because jcite4, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
