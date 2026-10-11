# C3 M1 S18

Site: label "Probably a duskcap (taste has faded)" (retaste): carries -> false

File: `glowcap.cav`
Line: 413 (line numbers are those of the original file)

Line 413, before:

```
        when $m_bitter and $m_retasted and carries(retaste_$m, taste_faded) because $m_retasted;
```

Line 413, after:

```
        when $m_bitter and $m_retasted and false because $m_retasted;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
