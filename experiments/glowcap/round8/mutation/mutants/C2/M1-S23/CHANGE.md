# C2 M1 S23

Site: journal.$e_cites on page 3, doubted: drop `d, doubted`

File: `glowcap.cav`
Line: 559 (line numbers are those of the original file)

Line 557, before:

```
    bind journal.$e_cites = qualified(j3_s + j3_c + j3_x, d, doubted)
```

Line 557, after:

```
    bind journal.$e_cites = j3_s + j3_c + j3_x
```

Line 559, before:

```
        because qualified(j3_s + j3_c + j3_x, d, doubted);
```

Line 559, after:

```
        because j3_s + j3_c + j3_x;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
