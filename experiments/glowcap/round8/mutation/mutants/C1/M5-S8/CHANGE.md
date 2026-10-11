# C1 M5 S8

Site: bind $m.label (statement at line 397): add to `because $m_taste` evidence the value never reads

File: `glowcap.cav`
Line: 397 (line numbers are those of the original file)

Line 397, before:

```
    bind $m.label = "Probably a glowcap (tasted in the dark)" when not $m_consumed and $m_taste == 1 and $m_in_dark because $m_taste;
```

Line 397, after:

```
    bind $m.label = "Probably a glowcap (tasted in the dark)" when not $m_consumed and $m_taste == 1 and $m_in_dark because $m_taste, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
