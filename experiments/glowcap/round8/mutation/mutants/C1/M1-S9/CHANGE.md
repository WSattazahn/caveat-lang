# C1 M1 S9

Site: remove `secondhand qualifies witness_$m`

File: `glowcap.cav`
Line: 258 (line numbers are those of the original file)

Line 258, before:

```
    secondhand qualifies witness_$m;
```

Line 258, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
