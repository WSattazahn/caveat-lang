# T3 M1 S14

Site: save restore: remove the taste_faded restore

File: `glowcap.ts`
Line: 519 (line numbers are those of the original file)

Line 519, before:

```
      else cav.push('taste_faded');
```

Line 519, after: (line removed)

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
