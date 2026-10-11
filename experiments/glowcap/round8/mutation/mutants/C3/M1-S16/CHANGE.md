# C3 M1 S16

Site: label "Probably a duskcap (tasted in the dark)" (retaste): carries -> false

File: `glowcap.cav`
Line: 409 (line numbers are those of the original file)

Line 409, before:

```
        when $m_bitter and $m_retasted and carries(retaste_$m, tasted_in_dark) because $m_retasted;
```

Line 409, after:

```
        when $m_bitter and $m_retasted and false because $m_retasted;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
