# C1 M5 S27

Site: bind journal.cites3 (statement at line 466): add to `because jcite3` evidence the value never reads

File: `glowcap.cav`
Line: 466 (line numbers are those of the original file)

Line 466, before:

```
bind journal.cites3 = jcite3 when j3 > 0 because jcite3;
```

Line 466, after:

```
bind journal.cites3 = jcite3 when j3 > 0 because jcite3, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
