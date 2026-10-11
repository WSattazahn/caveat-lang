# C2 M1 S25

Site: journal.$e_cites on page 5, doubted: drop `d, doubted`

File: `glowcap.cav`
Line: 565 (line numbers are those of the original file)

Line 563, before:

```
    bind journal.$e_cites = qualified(j5_s + j5_c + j5_x, d, doubted)
```

Line 563, after:

```
    bind journal.$e_cites = j5_s + j5_c + j5_x
```

Line 565, before:

```
        because qualified(j5_s + j5_c + j5_x, d, doubted);
```

Line 565, after:

```
        because j5_s + j5_c + j5_x;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
