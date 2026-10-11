# C3 M5 S132

Site: bind diary.d6 (statement at line 632): add to `because examined(doubted)` evidence the value never reads

File: `glowcap.cav`
Line: 632 (line numbers are those of the original file)

Line 632, before:

```
bind diary.d6 = 1 when rdoubted(r6, rs6, doubts) and (examined(doubted) or true) because examined(doubted);
```

Line 632, after:

```
bind diary.d6 = 1 when rdoubted(r6, rs6, doubts) and (examined(doubted) or true) because examined(doubted), observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
