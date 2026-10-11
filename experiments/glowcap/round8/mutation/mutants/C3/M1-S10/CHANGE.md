# C3 M1 S10

Site: remove the regrowth-time `qualify retaste_$m with taste_faded after …` (lines 366-367)

File: `glowcap.cav`
Line: 367 (line numbers are those of the original file)

Line 366, before:

```
    on tick when $m_regrows and $m_tasted_at >= 0 and $m_retasted
```

Line 366, after: (line removed)

Line 367, before:

```
        qualify retaste_$m with taste_faded after 60 - (elapsed() - $m_tasted_at);
```

Line 367, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
