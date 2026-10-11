# C3 M5 S23

Site: bind $m.whyAbsorb (statement at line 421): add to `because $m_retasted` evidence the value never reads

File: `glowcap.cav`
Line: 421 (line numbers are those of the original file)

Line 421, before:

```
    bind $m.whyAbsorb = "Known duskcap" when $m_bitter and $m_retasted because $m_retasted;
```

Line 421, after:

```
    bind $m.whyAbsorb = "Known duskcap" when $m_bitter and $m_retasted because $m_retasted, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
