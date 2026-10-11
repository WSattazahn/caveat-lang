# C1 M1 S16

Site: remove `on retaste qualify retaste_$m with taste_faded after fade_seconds`

File: `glowcap.cav`
Line: 321 (line numbers are those of the original file)

Line 321, before:

```
    on retaste qualify retaste_$m with taste_faded after fade_seconds;
```

Line 321, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
