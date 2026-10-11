# C3 M5 S31

Site: bind $m.whyTaste (statement at line 430): add to `because nothing` evidence the value never reads

File: `glowcap.cav`
Line: 430 (line numbers are those of the original file)

Line 430, before:

```
    bind $m.whyTaste = "Out of reach" when $m_out_of_reach because nothing;
```

Line 430, after:

```
    bind $m.whyTaste = "Out of reach" when $m_out_of_reach because observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
