# C3 M1 S7

Site: remove the tick-time `qualify taste_$m with taste_faded` (lines 358-359)

File: `glowcap.cav`
Line: 359 (line numbers are those of the original file)

Line 358, before:

```
    on tick when $m_tasted_at >= 0 and elapsed() - $m_tasted_at >= 60 and not $m_retasted
```

Line 358, after: (line removed)

Line 359, before:

```
        qualify taste_$m with taste_faded;
```

Line 359, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
