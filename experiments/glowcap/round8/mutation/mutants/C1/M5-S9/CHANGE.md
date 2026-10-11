# C1 M5 S9

Site: bind $m.label (statement at line 398): add to `because $m_taste` evidence the value never reads

File: `glowcap.cav`
Line: 398 (line numbers are those of the original file)

Line 398, before:

```
    bind $m.label = "Probably a duskcap (tasted in the dark)" when not $m_consumed and $m_taste == 2 and $m_in_dark because $m_taste;
```

Line 398, after:

```
    bind $m.label = "Probably a duskcap (tasted in the dark)" when not $m_consumed and $m_taste == 2 and $m_in_dark because $m_taste, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
