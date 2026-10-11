# C2 M1 S21

Site: journal.$e_cites on page 1, doubted: drop `d, doubted` (value and because)

File: `glowcap.cav`
Line: 553 (line numbers are those of the original file)

Line 551, before:

```
    bind journal.$e_cites = qualified(j1_s + j1_c + j1_x, d, doubted)
```

Line 551, after:

```
    bind journal.$e_cites = j1_s + j1_c + j1_x
```

Line 553, before:

```
        because qualified(j1_s + j1_c + j1_x, d, doubted);
```

Line 553, after:

```
        because j1_s + j1_c + j1_x;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
