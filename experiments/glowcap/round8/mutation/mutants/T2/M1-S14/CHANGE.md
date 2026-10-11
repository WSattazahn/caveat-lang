# T2 M1 S14

Site: save decode: remove the TASTED_IN_DARK restore

File: `glowcap.ts`
Line: 659 (line numbers are those of the original file)

Line 659, before:

```
    if (match[4].includes('d')) caveats.push(TASTED_IN_DARK);
```

Line 659, after: (line removed)

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
