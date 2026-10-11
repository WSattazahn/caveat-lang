# C1 M5 S18

Site: bind $m.why_absorb (statement at line 409): add to `because $m_eaten_cite` evidence the value never reads

File: `glowcap.cav`
Line: 409 (line numbers are those of the original file)

Line 409, before:

```
    bind $m.why_absorb = "Already eaten" when $m_eaten_cite > 0 because $m_eaten_cite;
```

Line 409, after:

```
    bind $m.why_absorb = "Already eaten" when $m_eaten_cite > 0 because $m_eaten_cite, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
