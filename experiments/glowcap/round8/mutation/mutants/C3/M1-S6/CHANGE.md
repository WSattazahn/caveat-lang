# C3 M1 S6

Site: remove `on retaste when glow == 0 qualify retaste_$m with tasted_in_dark`

File: `glowcap.cav`
Line: 313 (line numbers are those of the original file)

Line 313, before:

```
    on retaste when glow == 0 qualify retaste_$m with tasted_in_dark;
```

Line 313, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
