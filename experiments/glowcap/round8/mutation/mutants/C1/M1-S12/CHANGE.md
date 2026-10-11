# C1 M1 S12

Site: $m_faded: the has_caveat(…, taste_faded) test behaves as if the caveat were absent (false)

File: `glowcap.cav`
Line: 287 (line numbers are those of the original file)

Line 287, before:

```
    define $m_faded = has_caveat($m_taste, taste_faded);
```

Line 287, after:

```
    define $m_faded = false;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
