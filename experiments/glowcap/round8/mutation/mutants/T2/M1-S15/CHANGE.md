# T2 M1 S15

Site: save decode: remove the TASTE_FADED restore

File: `glowcap.ts`
Line: 660 (line numbers are those of the original file)

Line 660, before:

```
    if (match[4].includes('f')) caveats.push(TASTE_FADED);
```

Line 660, after: (line removed)

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
