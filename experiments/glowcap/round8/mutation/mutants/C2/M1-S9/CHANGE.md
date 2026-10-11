# C2 M1 S9

Site: remove the regrowth-time `qualify r_$m with faded after $m_fade - dt` (lines 290-291)

File: `glowcap.cav`
Line: 291 (line numbers are those of the original file)

Line 290, before:

```
    on tick when $m_regrows and $m_fade > dt and $m_retasted
```

Line 290, after: (line removed)

Line 291, before:

```
        qualify r_$m with faded after $m_fade - dt;
```

Line 291, after: (line removed)

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
