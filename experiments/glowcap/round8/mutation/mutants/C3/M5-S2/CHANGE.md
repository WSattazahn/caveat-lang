# C3 M5 S2

Site: bind $m.label (statement at line 389): add to `because support` evidence the value never reads

File: `glowcap.cav`
Line: 389 (line numbers are those of the original file)

Line 389, before:

```
    bind $m.label = "Probably a glowcap" when belief == 1 because support;
```

Line 389, after:

```
    bind $m.label = "Probably a glowcap" when belief == 1 because support, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
