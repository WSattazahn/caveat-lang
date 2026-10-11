# C3 M1 S4

Site: remove `on taste when glow == 0 qualify taste_$m with tasted_in_dark`

File: `glowcap.cav`
Line: 300 (line numbers are those of the original file)

Line 300, before:

```
    on taste when glow == 0 qualify taste_$m with tasted_in_dark;
```

Line 300, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
