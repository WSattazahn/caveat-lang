# C3 M1 S21

Site: diary.d1: drop the examined(doubted) caveat-bearing argument (because examined(doubted) -> because nothing), so journal record 1 never shows `doubted`

File: `glowcap.cav`
Line: 517 (line numbers are those of the original file)

Line 517, before:

```
bind diary.d1 = 1 when rdoubted(r1, rs1, doubts) and (examined(doubted) or true) because examined(doubted);
```

Line 517, after:

```
bind diary.d1 = 1 when rdoubted(r1, rs1, doubts) and (examined(doubted) or true) because nothing;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
