# C3 M1 S9

Site: remove the regrowth-time `qualify taste_$m with taste_faded after …` (lines 364-365)

File: `glowcap.cav`
Line: 365 (line numbers are those of the original file)

Line 364, before:

```
    on tick when $m_regrows and $m_tasted_at >= 0 and not $m_retasted
```

Line 364, after: (line removed)

Line 365, before:

```
        qualify taste_$m with taste_faded after 60 - (elapsed() - $m_tasted_at);
```

Line 365, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
