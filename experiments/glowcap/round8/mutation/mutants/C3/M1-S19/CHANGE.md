# C3 M1 S19

Site: whyAbsorb "Already eaten" by a doubted witness: drop the examined(doubted) caveat-bearing argument from the because (and the guard)

File: `glowcap.cav`
Line: 426 (line numbers are those of the original file)

Line 426, before:

```
        and (examined(doubted) or true) because observed(witness_$m), examined(doubted);
```

Line 426, after:

```
        because observed(witness_$m);
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
