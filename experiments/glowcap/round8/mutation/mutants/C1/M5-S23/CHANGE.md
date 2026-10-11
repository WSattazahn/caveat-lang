# C1 M5 S23

Site: bind belief.supported_by (statement at line 450): add to `because support` evidence the value never reads

File: `glowcap.cav`
Line: 450 (line numbers are those of the original file)

Line 450, before:

```
bind belief.supported_by = support because support;
```

Line 450, after:

```
bind belief.supported_by = support because support, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
