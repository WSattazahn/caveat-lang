# T2 M1 S12

Site: save encode: includes(DOUBTED) flag test -> false

File: `glowcap.ts`
Line: 619 (line numbers are those of the original file)

Line 619, before:

```
    if (ev.caveats.includes(DOUBTED)) flags += 'x';
```

Line 619, after:

```
    if (false) flags += 'x';
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
