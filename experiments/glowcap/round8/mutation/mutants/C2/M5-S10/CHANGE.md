# C2 M5 S10

Site: bind $m.label (statement at line 317): add to `because qualified(1, t_$m)` evidence the value never reads

File: `glowcap.cav`
Line: 318 (line numbers are those of the original file)

Line 318, before:

```
        when $m_tasted and $m_taste == 1 and carries(t_$m, faded) because qualified(1, t_$m);
```

Line 318, after:

```
        when $m_tasted and $m_taste == 1 and carries(t_$m, faded) because qualified(1, t_$m), observed(a_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
