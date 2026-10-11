# C3 M5 S26

Site: bind $m.whyAbsorb (statement at line 424): add to `because observed(witness_$m)` evidence the value never reads

File: `glowcap.cav`
Line: 424 (line numbers are those of the original file)

Line 424, before:

```
    bind $m.whyAbsorb = "Already eaten" when observed(witness_$m) because observed(witness_$m);
```

Line 424, after:

```
    bind $m.whyAbsorb = "Already eaten" when observed(witness_$m) because observed(witness_$m), observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
