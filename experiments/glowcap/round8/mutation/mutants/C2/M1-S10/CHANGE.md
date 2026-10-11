# C2 M1 S10

Site: label "Probably a glowcap (tasted in the dark)" (taste): carries(t_$m, dark) behaves as absent (false)

File: `glowcap.cav`
Line: 313 (line numbers are those of the original file)

Line 313, before:

```
        when $m_tasted and $m_taste == 1 and carries(t_$m, dark) because qualified(1, t_$m);
```

Line 313, after:

```
        when $m_tasted and $m_taste == 1 and false because qualified(1, t_$m);
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
