# C3 M5 S52

Site: bind diary.d1 (statement at line 517): add to `because examined(doubted)` evidence the value never reads

File: `glowcap.cav`
Line: 517 (line numbers are those of the original file)

Line 517, before:

```
bind diary.d1 = 1 when rdoubted(r1, rs1, doubts) and (examined(doubted) or true) because examined(doubted);
```

Line 517, after:

```
bind diary.d1 = 1 when rdoubted(r1, rs1, doubts) and (examined(doubted) or true) because examined(doubted), observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
