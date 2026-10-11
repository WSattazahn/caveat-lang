# C3 M5 S33

Site: bind $m.whyTaste (statement at line 432): add to `because observed(witness_$m)` evidence the value never reads

File: `glowcap.cav`
Line: 432 (line numbers are those of the original file)

Line 432, before:

```
    bind $m.whyTaste = "Already eaten" when observed(witness_$m) because observed(witness_$m);
```

Line 432, after:

```
    bind $m.whyTaste = "Already eaten" when observed(witness_$m) because observed(witness_$m), observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
