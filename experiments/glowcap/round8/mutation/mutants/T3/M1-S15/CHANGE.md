# T3 M1 S15

Site: save restore: remove the doubted restore

File: `glowcap.ts`
Line: 521 (line numbers are those of the original file)

Line 521, before:

```
    if (t.doubted) cav.push('doubted');
```

Line 521, after: (line removed)

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
