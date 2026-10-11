# C1 M5 S15

Site: bind $m.why_absorb (statement at line 406): add to `because nothing` evidence the value never reads

File: `glowcap.cav`
Line: 406 (line numbers are those of the original file)

Line 406, before:

```
    bind $m.why_absorb = "Marked to avoid" when not $m_consumed and $m_marked == 1 because nothing;
```

Line 406, after:

```
    bind $m.why_absorb = "Marked to avoid" when not $m_consumed and $m_marked == 1 because observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
