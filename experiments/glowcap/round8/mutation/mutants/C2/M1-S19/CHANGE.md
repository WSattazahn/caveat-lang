# C2 M1 S19

Site: why_taste "Already eaten" by a doubted witness: drop the `doubted` argument from the citation (and its guard)

File: `glowcap.cav`
Line: 357 (line numbers are those of the original file)

Line 357, before:

```
        and qualified($m_doubted, w_$m) == 1 because qualified(1, w_$m, doubted);
```

Line 357, after:

```
        and qualified($m_doubted, w_$m) == 1 because qualified(1, w_$m);
```

Line 357, before:

```
        and qualified($m_doubted, w_$m, doubted) == 1 because qualified(1, w_$m, doubted);
```

Line 357, after:

```
        and qualified($m_doubted, w_$m) == 1 because qualified(1, w_$m, doubted);
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
