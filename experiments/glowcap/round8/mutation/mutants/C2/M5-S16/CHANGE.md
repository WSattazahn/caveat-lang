# C2 M5 S16

Site: bind $m.label (statement at line 328): add to `because qualified(1, r_$m)` evidence the value never reads

File: `glowcap.cav`
Line: 329 (line numbers are those of the original file)

Line 329, before:

```
        when $m_retasted and $m_taste == 1 and carries(r_$m, faded) because qualified(1, r_$m);
```

Line 329, after:

```
        when $m_retasted and $m_taste == 1 and carries(r_$m, faded) because qualified(1, r_$m), observed(a_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
