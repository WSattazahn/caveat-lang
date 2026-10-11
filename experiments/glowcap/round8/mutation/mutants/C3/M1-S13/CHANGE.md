# C3 M1 S13

Site: label "Probably a glowcap (taste has faded)" (taste): carries -> false

File: `glowcap.cav`
Line: 400 (line numbers are those of the original file)

Line 400, before:

```
        when $m_sweet and carries(taste_$m, taste_faded) because $m_tasted;
```

Line 400, after:

```
        when $m_sweet and false because $m_tasted;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
