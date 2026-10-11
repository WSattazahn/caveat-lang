# C1 M1 S6

Site: jcite5: drop the doubt_mark operand (journal entry 5 loses `doubted`)

File: `glowcap.cav`
Line: 175 (line numbers are those of the original file)

Line 175, before:

```
define jcite5 = j5 + if(y5 == 1, doubt_mark, 0);
```

Line 175, after:

```
define jcite5 = j5;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
