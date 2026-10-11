# C2 M1 S24

Site: journal.$e_cites on page 4, doubted: drop `d, doubted`

File: `glowcap.cav`
Line: 562 (line numbers are those of the original file)

Line 560, before:

```
    bind journal.$e_cites = qualified(j4_s + j4_c + j4_x, d, doubted)
```

Line 560, after:

```
    bind journal.$e_cites = j4_s + j4_c + j4_x
```

Line 562, before:

```
        because qualified(j4_s + j4_c + j4_x, d, doubted);
```

Line 562, after:

```
        because j4_s + j4_c + j4_x;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
