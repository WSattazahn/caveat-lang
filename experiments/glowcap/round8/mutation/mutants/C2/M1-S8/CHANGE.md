# C2 M1 S8

Site: remove the regrowth-time `qualify t_$m with faded after $m_fade - dt` (lines 288-289)

File: `glowcap.cav`
Line: 289 (line numbers are those of the original file)

Line 288, before:

```
    on tick when $m_regrows and $m_fade > dt and not $m_retasted
```

Line 288, after: (line removed)

Line 289, before:

```
        qualify t_$m with faded after $m_fade - dt;
```

Line 289, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
