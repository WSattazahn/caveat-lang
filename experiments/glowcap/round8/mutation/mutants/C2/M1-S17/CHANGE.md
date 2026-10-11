# C2 M1 S17

Site: label "Probably a duskcap (taste has faded)" (retaste): carries(r_$m, faded) -> false

File: `glowcap.cav`
Line: 331 (line numbers are those of the original file)

Line 331, before:

```
        when $m_retasted and $m_taste == 2 and carries(r_$m, faded) because qualified(1, r_$m);
```

Line 331, after:

```
        when $m_retasted and $m_taste == 2 and false because qualified(1, r_$m);
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
