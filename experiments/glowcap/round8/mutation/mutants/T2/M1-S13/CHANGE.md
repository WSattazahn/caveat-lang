# T2 M1 S13

Site: save decode: remove the SECONDHAND restore

File: `glowcap.ts`
Line: 658 (line numbers are those of the original file)

Line 658, before:

```
    if (match[1] === 'w') caveats.push(SECONDHAND);
```

Line 658, after: (line removed)

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
