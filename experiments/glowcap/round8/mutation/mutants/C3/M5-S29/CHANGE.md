# C3 M5 S29

Site: bind $m.whyTaste (statement at line 428): add to `because $m_tasted` evidence the value never reads

File: `glowcap.cav`
Line: 428 (line numbers are those of the original file)

Line 428, before:

```
    bind $m.whyTaste = "Already tasted" when $m_tasted because $m_tasted;
```

Line 428, after:

```
    bind $m.whyTaste = "Already tasted" when $m_tasted because $m_tasted, observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
