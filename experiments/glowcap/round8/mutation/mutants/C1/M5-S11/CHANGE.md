# C1 M5 S11

Site: bind $m.label (statement at line 400): add to `because $m_taste` evidence the value never reads

File: `glowcap.cav`
Line: 400 (line numbers are those of the original file)

Line 400, before:

```
    bind $m.label = "Probably a duskcap (taste has faded)" when not $m_consumed and $m_taste == 2 and $m_faded because $m_taste;
```

Line 400, after:

```
    bind $m.label = "Probably a duskcap (taste has faded)" when not $m_consumed and $m_taste == 2 and $m_faded because $m_taste, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
