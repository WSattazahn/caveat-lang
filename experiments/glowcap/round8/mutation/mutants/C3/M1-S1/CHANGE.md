# C3 M1 S1

Site: journal caveats: drop the `['diary', d<n>]` member of the caveat union, the member that carries `doubted`

File: `adapter.mjs`
Line: 63 (line numbers are those of the original file)

Line 63, before:

```
        caveats: caveated(v, ['diary', `c${n}`], ['diary', `d${n}`]),
```

Line 63, after:

```
        caveats: caveated(v, ['diary', `c${n}`]),
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
