# C1 M1 S2

Site: jcite1: drop the `if(y1 == 1, doubt_mark, 0)` operand, so journal entry 1 no longer cites `doubted`

File: `glowcap.cav`
Line: 171 (line numbers are those of the original file)

Line 171, before:

```
define jcite1 = j1 + if(y1 == 1, doubt_mark, 0);
```

Line 171, after:

```
define jcite1 = j1;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
