# C2 M1 S22

Site: journal.$e_cites on page 2, doubted: drop `d, doubted`

File: `glowcap.cav`
Line: 556 (line numbers are those of the original file)

Line 554, before:

```
    bind journal.$e_cites = qualified(j2_s + j2_c + j2_x, d, doubted)
```

Line 554, after:

```
    bind journal.$e_cites = j2_s + j2_c + j2_x
```

Line 556, before:

```
        because qualified(j2_s + j2_c + j2_x, d, doubted);
```

Line 556, after:

```
        because j2_s + j2_c + j2_x;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
