# C2 M5 S8

Site: bind $m.label (statement at line 314): add to `because qualified(1, t_$m)` evidence the value never reads

File: `glowcap.cav`
Line: 314 (line numbers are those of the original file)

Line 314, before:

```
    bind $m.label = "Duskcap — avoid" when $m_tasted and $m_taste == 2 because qualified(1, t_$m);
```

Line 314, after:

```
    bind $m.label = "Duskcap — avoid" when $m_tasted and $m_taste == 2 because qualified(1, t_$m), observed(a_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
