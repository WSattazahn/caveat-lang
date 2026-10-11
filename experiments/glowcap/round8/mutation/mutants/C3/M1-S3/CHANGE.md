# C3 M1 S3

Site: $m_can_retaste: carries(taste_$m, taste_faded) behaves as absent (false)

File: `glowcap.cav`
Line: 277 (line numbers are those of the original file)

Line 277, before:

```
        and carries(taste_$m, taste_faded) and not $m_retasted;
```

Line 277, after:

```
        and false and not $m_retasted;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
