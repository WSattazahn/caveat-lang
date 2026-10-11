# C2 M1 S27

Site: journal.$e_trust_cites when doubted: drop `d, doubted` (value and because)

File: `glowcap.cav`
Line: 576 (line numbers are those of the original file)

Line 574, before:

```
    bind journal.$e_trust_cites = qualified($e_h, d, doubted)
```

Line 574, after:

```
    bind journal.$e_trust_cites = $e_h
```

Line 576, before:

```
        because qualified($e_h, d, doubted);
```

Line 576, after:

```
        because $e_h;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
