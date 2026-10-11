# C2 M1 S3

Site: $m_can_retaste: the carries(t_$m, faded) test behaves as if the caveat were absent (false)

File: `glowcap.cav`
Line: 212 (line numbers are those of the original file)

Line 212, before:

```
    define $m_can_retaste = not $m_consumed and $m_taste > 0 and carries(t_$m, faded)
```

Line 212, after:

```
    define $m_can_retaste = not $m_consumed and $m_taste > 0 and false
```

Choice: A `qualify`/`qualifies` statement is removed; a `carries`/`has_caveat` test is replaced by `false` (as if the caveat were absent); a caveat-bearing argument or operand is removed and the rest of the expression kept.
