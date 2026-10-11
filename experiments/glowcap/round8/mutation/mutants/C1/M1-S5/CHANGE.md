# C1 M1 S5

Site: jcite4: drop the doubt_mark operand (journal entry 4 loses `doubted`)

File: `glowcap.cav`
Line: 174 (line numbers are those of the original file)

Line 174, before:

```
define jcite4 = j4 + if(y4 == 1, doubt_mark, 0);
```

Line 174, after:

```
define jcite4 = j4;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
