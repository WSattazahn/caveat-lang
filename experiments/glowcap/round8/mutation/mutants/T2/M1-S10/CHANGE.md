# T2 M1 S10

Site: save encode: the includes(TASTED_IN_DARK) flag test -> false (dark not saved)

File: `glowcap.ts`
Line: 617 (line numbers are those of the original file)

Line 617, before:

```
    if (ev.caveats.includes(TASTED_IN_DARK)) flags += 'd';
```

Line 617, after:

```
    if (false) flags += 'd';
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
