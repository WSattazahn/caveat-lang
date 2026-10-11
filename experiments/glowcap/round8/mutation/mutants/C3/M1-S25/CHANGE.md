# C3 M1 S25

Site: diary.d5: drop the examined(doubted) caveat-bearing argument (because examined(doubted) -> because nothing), so journal record 5 never shows `doubted`

File: `glowcap.cav`
Line: 609 (line numbers are those of the original file)

Line 609, before:

```
bind diary.d5 = 1 when rdoubted(r5, rs5, doubts) and (examined(doubted) or true) because examined(doubted);
```

Line 609, after:

```
bind diary.d5 = 1 when rdoubted(r5, rs5, doubts) and (examined(doubted) or true) because nothing;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
