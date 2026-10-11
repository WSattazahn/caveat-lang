# C2 M1 S13

Site: label "Probably a duskcap (taste has faded)" (taste): carries(t_$m, faded) -> false

File: `glowcap.cav`
Line: 320 (line numbers are those of the original file)

Line 320, before:

```
        when $m_tasted and $m_taste == 2 and carries(t_$m, faded) because qualified(1, t_$m);
```

Line 320, after:

```
        when $m_tasted and $m_taste == 2 and false because qualified(1, t_$m);
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
