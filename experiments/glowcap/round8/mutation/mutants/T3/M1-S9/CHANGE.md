# T3 M1 S9

Site: tick fade: drop 'taste_faded' from the caveat map entry

File: `glowcap.ts`
Line: 389 (line numbers are those of the original file)

Line 389, before:

```
          s.caveats[ev] = [...(s.caveats[ev] ?? []), 'taste_faded'];
```

Line 389, after:

```
          s.caveats[ev] = [...(s.caveats[ev] ?? [])];
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
