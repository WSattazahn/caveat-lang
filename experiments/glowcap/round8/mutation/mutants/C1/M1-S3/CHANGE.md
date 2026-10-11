# C1 M1 S3

Site: jcite2: drop the doubt_mark operand (journal entry 2 loses `doubted`)

File: `glowcap.cav`
Line: 172 (line numbers are those of the original file)

Line 172, before:

```
define jcite2 = j2 + if(y2 == 1, doubt_mark, 0);
```

Line 172, after:

```
define jcite2 = j2;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
