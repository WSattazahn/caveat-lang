# C2 M1 S12

Site: label "Probably a glowcap (taste has faded)" (taste): carries(t_$m, faded) -> false

File: `glowcap.cav`
Line: 318 (line numbers are those of the original file)

Line 318, before:

```
        when $m_tasted and $m_taste == 1 and carries(t_$m, faded) because qualified(1, t_$m);
```

Line 318, after:

```
        when $m_tasted and $m_taste == 1 and false because qualified(1, t_$m);
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
