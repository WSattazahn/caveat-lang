# T3 M1 S13

Site: save restore: remove the tasted_in_dark restore

File: `glowcap.ts`
Line: 515 (line numbers are those of the original file)

Line 515, before:

```
    if (t.dark) cav.push('tasted_in_dark');
```

Line 515, after: (line removed)

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
