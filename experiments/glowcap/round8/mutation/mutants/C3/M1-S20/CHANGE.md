# C3 M1 S20

Site: whyTaste "Already eaten" by a doubted witness: drop examined(doubted) from the because (and the guard)

File: `glowcap.cav`
Line: 434 (line numbers are those of the original file)

Line 434, before:

```
        and (examined(doubted) or true) because observed(witness_$m), examined(doubted);
```

Line 434, after:

```
        because observed(witness_$m);
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
