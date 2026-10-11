# C3 M1 S15

Site: label "Probably a glowcap (tasted in the dark)" (retaste): carries -> false

File: `glowcap.cav`
Line: 407 (line numbers are those of the original file)

Line 407, before:

```
        when $m_sweet and $m_retasted and carries(retaste_$m, tasted_in_dark) because $m_retasted;
```

Line 407, after:

```
        when $m_sweet and $m_retasted and false because $m_retasted;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
