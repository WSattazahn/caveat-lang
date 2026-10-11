# C2 M1 S20

Site: journal.$e_cites for a doubted doubt/undoubt record: drop `d, doubted` from the binding (value and because)

File: `glowcap.cav`
Line: 537 (line numbers are those of the original file)

Line 536, before:

```
    bind journal.$e_cites = qualified($e_g, d, doubted) when $e_shown and not $e_obs and $e_flag1
```

Line 536, after:

```
    bind journal.$e_cites = $e_g when $e_shown and not $e_obs and $e_flag1
```

Line 537, before:

```
        because qualified($e_g, d, doubted);
```

Line 537, after:

```
        because $e_g;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
