# C2 M1 S1

Site: recovery commit on a doubted page: drop the `d, doubted` caveat-bearing arguments from the using expression (qualified(j5_x + j6_s, d, doubted) -> j5_x + j6_s)

File: `glowcap.cav`
Line: 153 (line numbers are those of the original file)

Line 153, before:

```
    when trusting == 1 and j5_n >= 100000000 commit trust because enough using qualified(j5_x + j6_s, d, doubted);
```

Line 153, after:

```
    when trusting == 1 and j5_n >= 100000000 commit trust because enough using j5_x + j6_s;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
