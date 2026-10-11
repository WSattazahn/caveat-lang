# C1 M1 S10

Site: $m_eaten_cite: drop the doubt_mark operand, so "Already eaten" no longer cites `doubted` for a doubted consuming witness

File: `glowcap.cav`
Line: 283 (line numbers are those of the original file)

Line 283, before:

```
    define $m_eaten_cite = $m_regrow + if($m_doubted_consumer == 1, doubt_mark, 0);
```

Line 283, after:

```
    define $m_eaten_cite = $m_regrow;
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
