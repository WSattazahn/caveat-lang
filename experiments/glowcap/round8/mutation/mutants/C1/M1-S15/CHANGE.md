# C1 M1 S15

Site: on retaste: the caveat-bearing `dark` argument to observe becomes 0, so a retaste never carries tasted_in_dark

File: `glowcap.cav`
Line: 318 (line numbers are those of the original file)

Line 318, before:

```
    on retaste call observe(retaste_$m, sort, if(glow > 0, 0, 1), 4, $m_life * 10 + $index);
```

Line 318, after:

```
    on retaste call observe(retaste_$m, sort, 0, 4, $m_life * 10 + $index);
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
