# C2 M5 S31

Site: bind $m.why_taste (statement at line 353): add to `because nothing` evidence the value never reads

File: `glowcap.cav`
Line: 353 (line numbers are those of the original file)

Line 353, before:

```
    bind $m.why_taste = "Out of reach" when $m_out_of_reach because nothing;
```

Line 353, after:

```
    bind $m.why_taste = "Out of reach" when $m_out_of_reach because observed(a_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
