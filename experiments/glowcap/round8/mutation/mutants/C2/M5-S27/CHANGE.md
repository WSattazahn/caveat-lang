# C2 M5 S27

Site: bind $m.why_absorb (statement at line 348): add to `because qualified(1, w_$m, doubted)` evidence the value never reads

File: `glowcap.cav`
Line: 349 (line numbers are those of the original file)

Line 349, before:

```
        and qualified($m_doubted, w_$m, doubted) == 1 because qualified(1, w_$m, doubted);
```

Line 349, after:

```
        and qualified($m_doubted, w_$m, doubted) == 1 because qualified(1, w_$m, doubted), observed(a_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
