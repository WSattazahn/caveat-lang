# C3 M1 S11

Site: label "Probably a glowcap (tasted in the dark)" (taste): carries -> false

File: `glowcap.cav`
Line: 396 (line numbers are those of the original file)

Line 396, before:

```
        when $m_sweet and carries(taste_$m, tasted_in_dark) because $m_tasted;
```

Line 396, after:

```
        when $m_sweet and false because $m_tasted;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
