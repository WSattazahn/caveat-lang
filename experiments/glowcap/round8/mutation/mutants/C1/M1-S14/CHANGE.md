# C1 M1 S14

Site: remove `on taste qualify taste_$m with taste_faded after fade_seconds`

File: `glowcap.cav`
Line: 312 (line numbers are those of the original file)

Line 312, before:

```
    on taste qualify taste_$m with taste_faded after fade_seconds;
```

Line 312, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
