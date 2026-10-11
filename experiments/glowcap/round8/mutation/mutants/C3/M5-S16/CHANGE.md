# C3 M5 S16

Site: bind $m.label (statement at line 410): add to `because $m_retasted` evidence the value never reads

File: `glowcap.cav`
Line: 411 (line numbers are those of the original file)

Line 411, before:

```
        when $m_sweet and $m_retasted and carries(retaste_$m, taste_faded) because $m_retasted;
```

Line 411, after:

```
        when $m_sweet and $m_retasted and carries(retaste_$m, taste_faded) because $m_retasted, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
