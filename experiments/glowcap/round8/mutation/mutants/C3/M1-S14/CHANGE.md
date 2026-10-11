# C3 M1 S14

Site: label "Probably a duskcap (taste has faded)" (taste): carries -> false

File: `glowcap.cav`
Line: 402 (line numbers are those of the original file)

Line 402, before:

```
        when $m_bitter and carries(taste_$m, taste_faded) because $m_tasted;
```

Line 402, after:

```
        when $m_bitter and false because $m_tasted;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
