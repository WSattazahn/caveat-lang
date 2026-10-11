# C1 M1 S8

Site: observe: remove `when dark == 1 qualify e with tasted_in_dark` (tastes and retastes lose tasted_in_dark)

File: `glowcap.cav`
Line: 202 (line numbers are those of the original file)

Line 202, before:

```
    when dark == 1 qualify e with tasted_in_dark;
```

Line 202, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
