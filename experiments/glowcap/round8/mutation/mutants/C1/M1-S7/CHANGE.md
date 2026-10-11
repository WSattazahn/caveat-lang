# C1 M1 S7

Site: jcite6: drop the doubt_mark operand (journal entry 6 loses `doubted`)

File: `glowcap.cav`
Line: 176 (line numbers are those of the original file)

Line 176, before:

```
define jcite6 = j6 + if(y6 == 1, doubt_mark, 0);
```

Line 176, after:

```
define jcite6 = j6;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
