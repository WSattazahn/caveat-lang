# C2 M5 S52

Site: bind journal.$e_trust_cites (statement at line 573): add to `because $e_h` evidence the value never reads

File: `glowcap.cav`
Line: 573 (line numbers are those of the original file)

Line 573, before:

```
    bind journal.$e_trust_cites = $e_h when $e_trust_shown because $e_h;
```

Line 573, after:

```
    bind journal.$e_trust_cites = $e_h when $e_trust_shown because $e_h, observed(a_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
