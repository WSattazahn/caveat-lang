# C2 M1 S4

Site: remove `on taste when glow == 0 qualify t_$m with dark`

File: `glowcap.cav`
Line: 251 (line numbers are those of the original file)

Line 251, before:

```
    on taste when glow == 0 qualify t_$m with dark;
```

Line 251, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
