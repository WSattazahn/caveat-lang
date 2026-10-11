# C1 M5 S14

Site: bind $m.why_absorb (statement at line 405): add to `because contradiction` evidence the value never reads

File: `glowcap.cav`
Line: 405 (line numbers are those of the original file)

Line 405, before:

```
    bind $m.why_absorb = "Too risky untasted" when $m_too_risky because contradiction;
```

Line 405, after:

```
    bind $m.why_absorb = "Too risky untasted" when $m_too_risky because contradiction, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
