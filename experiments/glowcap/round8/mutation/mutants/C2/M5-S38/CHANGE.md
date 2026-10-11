# C2 M5 S38

Site: bind journal.$e_cites (statement at line 535): add to `because $e_g` evidence the value never reads

File: `glowcap.cav`
Line: 535 (line numbers are those of the original file)

Line 535, before:

```
    bind journal.$e_cites = $e_g when $e_shown and not $e_obs because $e_g;
```

Line 535, after:

```
    bind journal.$e_cites = $e_g when $e_shown and not $e_obs because $e_g, observed(a_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
