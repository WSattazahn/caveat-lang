# C3 M5 S32

Site: bind $m.whyTaste (statement at line 431): add to `because observed(absorb_$m)` evidence the value never reads

File: `glowcap.cav`
Line: 431 (line numbers are those of the original file)

Line 431, before:

```
    bind $m.whyTaste = "Already eaten" when observed(absorb_$m) because observed(absorb_$m);
```

Line 431, after:

```
    bind $m.whyTaste = "Already eaten" when observed(absorb_$m) because observed(absorb_$m), observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
