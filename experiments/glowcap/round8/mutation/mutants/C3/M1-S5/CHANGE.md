# C3 M1 S5

Site: retaste refusal "The taste has not faded yet.": carries(taste_$m, taste_faded) behaves as absent (false)

File: `glowcap.cav`
Line: 309 (line numbers are those of the original file)

Line 309, before:

```
    on retaste when not carries(taste_$m, taste_faded) reject "The taste has not faded yet.";
```

Line 309, after:

```
    on retaste when not false reject "The taste has not faded yet.";
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
