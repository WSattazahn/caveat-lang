# C1 M1 S11

Site: $m_in_dark: the has_caveat(…, tasted_in_dark) test (state form of carries) behaves as if the caveat were absent (false)

File: `glowcap.cav`
Line: 286 (line numbers are those of the original file)

Line 286, before:

```
    define $m_in_dark = has_caveat($m_taste, tasted_in_dark);
```

Line 286, after:

```
    define $m_in_dark = false;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
