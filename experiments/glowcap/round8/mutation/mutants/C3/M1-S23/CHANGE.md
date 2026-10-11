# C3 M1 S23

Site: diary.d3: drop the examined(doubted) caveat-bearing argument (because examined(doubted) -> because nothing), so journal record 3 never shows `doubted`

File: `glowcap.cav`
Line: 563 (line numbers are those of the original file)

Line 563, before:

```
bind diary.d3 = 1 when rdoubted(r3, rs3, doubts) and (examined(doubted) or true) because examined(doubted);
```

Line 563, after:

```
bind diary.d3 = 1 when rdoubted(r3, rs3, doubts) and (examined(doubted) or true) because nothing;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
