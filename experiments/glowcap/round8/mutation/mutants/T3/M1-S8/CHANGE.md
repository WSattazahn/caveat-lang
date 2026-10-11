# T3 M1 S8

Site: doubt: drop 'doubted' from the caveat map entry

File: `glowcap.ts`
Line: 368 (line numbers are those of the original file)

Line 368, before:

```
      s.caveats[obs.ev] = [...(s.caveats[obs.ev] ?? []), 'doubted'];
```

Line 368, after:

```
      s.caveats[obs.ev] = [...(s.caveats[obs.ev] ?? [])];
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
