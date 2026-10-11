# C3 M5 S20

Site: bind $m.whyAbsorb (statement at line 418): add to `because contradiction` evidence the value never reads

File: `glowcap.cav`
Line: 418 (line numbers are those of the original file)

Line 418, before:

```
    bind $m.whyAbsorb = "Too risky untasted" when $m_too_risky because contradiction;
```

Line 418, after:

```
    bind $m.whyAbsorb = "Too risky untasted" when $m_too_risky because contradiction, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
