# C1 M5 S16

Site: bind $m.why_absorb (statement at line 407): add to `because $m_taste` evidence the value never reads

File: `glowcap.cav`
Line: 407 (line numbers are those of the original file)

Line 407, before:

```
    bind $m.why_absorb = "Known duskcap" when not $m_consumed and $m_taste == 2 because $m_taste;
```

Line 407, after:

```
    bind $m.why_absorb = "Known duskcap" when not $m_consumed and $m_taste == 2 because $m_taste, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
