# C2 M1 S26

Site: journal.$e_cites on page 6, doubted: drop `d, doubted`

File: `glowcap.cav`
Line: 568 (line numbers are those of the original file)

Line 566, before:

```
    bind journal.$e_cites = qualified(j6_s + j6_c + j6_x, d, doubted)
```

Line 566, after:

```
    bind journal.$e_cites = j6_s + j6_c + j6_x
```

Line 568, before:

```
        because qualified(j6_s + j6_c + j6_x, d, doubted);
```

Line 568, after:

```
        because j6_s + j6_c + j6_x;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
