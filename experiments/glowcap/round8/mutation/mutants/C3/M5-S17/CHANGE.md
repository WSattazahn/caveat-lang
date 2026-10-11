# C3 M5 S17

Site: bind $m.label (statement at line 412): add to `because $m_retasted` evidence the value never reads

File: `glowcap.cav`
Line: 413 (line numbers are those of the original file)

Line 413, before:

```
        when $m_bitter and $m_retasted and carries(retaste_$m, taste_faded) because $m_retasted;
```

Line 413, after:

```
        when $m_bitter and $m_retasted and carries(retaste_$m, taste_faded) because $m_retasted, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
