# C2 M1 S5

Site: remove `on retaste when glow == 0 qualify r_$m with dark`

File: `glowcap.cav`
Line: 261 (line numbers are those of the original file)

Line 261, before:

```
    on retaste when glow == 0 qualify r_$m with dark;
```

Line 261, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
