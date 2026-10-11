# C2 M1 S18

Site: why_absorb "Already eaten" by a doubted witness: drop the `doubted` caveat-bearing argument from the citation (because qualified(1, w_$m, doubted) -> qualified(1, w_$m), with the guard's qualified($m_doubted, w_$m, doubted) likewise)

File: `glowcap.cav`
Line: 349 (line numbers are those of the original file)

Line 349, before:

```
        and qualified($m_doubted, w_$m) == 1 because qualified(1, w_$m, doubted);
```

Line 349, after:

```
        and qualified($m_doubted, w_$m) == 1 because qualified(1, w_$m);
```

Line 349, before:

```
        and qualified($m_doubted, w_$m, doubted) == 1 because qualified(1, w_$m, doubted);
```

Line 349, after:

```
        and qualified($m_doubted, w_$m) == 1 because qualified(1, w_$m, doubted);
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
