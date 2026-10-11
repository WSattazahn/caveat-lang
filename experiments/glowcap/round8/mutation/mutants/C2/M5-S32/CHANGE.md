# C2 M5 S32

Site: bind $m.why_taste (statement at line 354): add to `because qualified(1, a_$m)` evidence the value never reads

File: `glowcap.cav`
Line: 354 (line numbers are those of the original file)

Line 354, before:

```
    bind $m.why_taste = "Already eaten" when $m_eaten_by_absorb because qualified(1, a_$m);
```

Line 354, after:

```
    bind $m.why_taste = "Already eaten" when $m_eaten_by_absorb because qualified(1, a_$m), observed(a_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
