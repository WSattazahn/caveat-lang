# C1 M1 S4

Site: jcite3: drop the doubt_mark operand (journal entry 3 loses `doubted`)

File: `glowcap.cav`
Line: 173 (line numbers are those of the original file)

Line 173, before:

```
define jcite3 = j3 + if(y3 == 1, doubt_mark, 0);
```

Line 173, after:

```
define jcite3 = j3;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
