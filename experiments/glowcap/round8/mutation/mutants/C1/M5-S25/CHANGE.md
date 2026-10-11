# C1 M5 S25

Site: bind journal.cites1 (statement at line 464): add to `because jcite1` evidence the value never reads

File: `glowcap.cav`
Line: 464 (line numbers are those of the original file)

Line 464, before:

```
bind journal.cites1 = jcite1 when j1 > 0 because jcite1;
```

Line 464, after:

```
bind journal.cites1 = jcite1 when j1 > 0 because jcite1, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
