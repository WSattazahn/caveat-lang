# C2 M1 S15

Site: label "Probably a duskcap (tasted in the dark)" (retaste): carries(r_$m, dark) -> false

File: `glowcap.cav`
Line: 327 (line numbers are those of the original file)

Line 327, before:

```
        when $m_retasted and $m_taste == 2 and carries(r_$m, dark) because qualified(1, r_$m);
```

Line 327, after:

```
        when $m_retasted and $m_taste == 2 and false because qualified(1, r_$m);
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
