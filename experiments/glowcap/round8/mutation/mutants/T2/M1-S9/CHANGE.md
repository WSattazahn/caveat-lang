# T2 M1 S9

Site: tick fade: remove e.caveats.push(TASTE_FADED)

File: `glowcap.ts`
Line: 514 (line numbers are those of the original file)

Line 514, before:

```
          e.caveats.push(TASTE_FADED);
```

Line 514, after: (line removed)

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
