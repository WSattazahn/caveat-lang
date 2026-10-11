# C2 M1 S6

Site: remove `on tick when $m_fades and not $m_retasted qualify t_$m with faded`

File: `glowcap.cav`
Line: 280 (line numbers are those of the original file)

Line 280, before:

```
    on tick when $m_fades and not $m_retasted qualify t_$m with faded;
```

Line 280, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
