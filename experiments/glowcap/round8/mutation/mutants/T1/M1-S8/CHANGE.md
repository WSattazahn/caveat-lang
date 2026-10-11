# T1 M1 S8

Site: witness: caveat map entry [SECONDHAND] -> []

File: `glowcap.ts`
Line: 422 (line numbers are those of the original file)

Line 422, before:

```
      s.evidenceCaveats[ev] = [SECONDHAND];
```

Line 422, after:

```
      s.evidenceCaveats[ev] = [];
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
