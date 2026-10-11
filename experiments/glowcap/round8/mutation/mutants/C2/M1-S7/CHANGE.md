# C2 M1 S7

Site: remove `on tick when $m_fades and $m_retasted qualify r_$m with faded`

File: `glowcap.cav`
Line: 281 (line numbers are those of the original file)

Line 281, before:

```
    on tick when $m_fades and $m_retasted qualify r_$m with faded;
```

Line 281, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
