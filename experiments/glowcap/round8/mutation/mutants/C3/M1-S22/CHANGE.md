# C3 M1 S22

Site: diary.d2: drop the examined(doubted) caveat-bearing argument (because examined(doubted) -> because nothing), so journal record 2 never shows `doubted`

File: `glowcap.cav`
Line: 540 (line numbers are those of the original file)

Line 540, before:

```
bind diary.d2 = 1 when rdoubted(r2, rs2, doubts) and (examined(doubted) or true) because examined(doubted);
```

Line 540, after:

```
bind diary.d2 = 1 when rdoubted(r2, rs2, doubts) and (examined(doubted) or true) because nothing;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
