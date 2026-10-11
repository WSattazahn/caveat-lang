# C1 M5 S1

Site: bind $m.label (statement at line 390): add to `because nothing` evidence the value never reads

File: `glowcap.cav`
Line: 390 (line numbers are those of the original file)

Line 390, before:

```
    bind $m.label = "Glowing mushroom" when $m_unknown and belief_none because nothing;
```

Line 390, after:

```
    bind $m.label = "Glowing mushroom" when $m_unknown and belief_none because observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
