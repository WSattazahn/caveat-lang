# C3 M5 S6

Site: bind $m.label (statement at line 393): add to `because $m_tasted` evidence the value never reads

File: `glowcap.cav`
Line: 393 (line numbers are those of the original file)

Line 393, before:

```
    bind $m.label = "Glowcap" when $m_sweet because $m_tasted;
```

Line 393, after:

```
    bind $m.label = "Glowcap" when $m_sweet because $m_tasted, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
