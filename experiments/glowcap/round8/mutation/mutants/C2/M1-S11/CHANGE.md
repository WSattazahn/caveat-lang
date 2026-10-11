# C2 M1 S11

Site: label "Probably a duskcap (tasted in the dark)" (taste): carries(t_$m, dark) -> false

File: `glowcap.cav`
Line: 316 (line numbers are those of the original file)

Line 316, before:

```
        when $m_tasted and $m_taste == 2 and carries(t_$m, dark) because qualified(1, t_$m);
```

Line 316, after:

```
        when $m_tasted and $m_taste == 2 and false because qualified(1, t_$m);
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
