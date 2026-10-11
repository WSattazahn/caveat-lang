# C3 M5 S68

Site: bind diary.d2 (statement at line 540): add to `because examined(doubted)` evidence the value never reads

File: `glowcap.cav`
Line: 540 (line numbers are those of the original file)

Line 540, before:

```
bind diary.d2 = 1 when rdoubted(r2, rs2, doubts) and (examined(doubted) or true) because examined(doubted);
```

Line 540, after:

```
bind diary.d2 = 1 when rdoubted(r2, rs2, doubts) and (examined(doubted) or true) because examined(doubted), observed(absorb_cave);
```

Choice: Choice (same in C1-C3): the added citation is `observed(absorb_cave)` (C2 spells the evidence `a_cave`): the cave mushroom's first-life absorption, which this binding is not about. A true `observed(...)` carries that evidence; `because nothing` becomes `because observed(...)`, otherwise it is appended after a comma.
