# C1 M1 S13

Site: on taste: the caveat-bearing `dark` argument to observe becomes 0, so a taste never carries tasted_in_dark

File: `glowcap.cav`
Line: 308 (line numbers are those of the original file)

Line 308, before:

```
    on taste call observe(taste_$m, sort, if(glow > 0, 0, 1), 2, $m_life * 10 + $index);
```

Line 308, after:

```
    on taste call observe(taste_$m, sort, 0, 2, $m_life * 10 + $index);
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
