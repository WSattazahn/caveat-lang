# T2 M1 S16

Site: save decode: remove the DOUBTED restore

File: `glowcap.ts`
Line: 661 (line numbers are those of the original file)

Line 661, before:

```
    if (match[4].includes('x')) caveats.push(DOUBTED);
```

Line 661, after: (line removed)

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
