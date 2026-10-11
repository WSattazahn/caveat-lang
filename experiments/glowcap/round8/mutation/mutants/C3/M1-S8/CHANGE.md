# C3 M1 S8

Site: remove the tick-time `qualify retaste_$m with taste_faded` (lines 360-361)

File: `glowcap.cav`
Line: 361 (line numbers are those of the original file)

Line 360, before:

```
    on tick when $m_tasted_at >= 0 and elapsed() - $m_tasted_at >= 60 and $m_retasted
```

Line 360, after: (line removed)

Line 361, before:

```
        qualify retaste_$m with taste_faded;
```

Line 361, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
