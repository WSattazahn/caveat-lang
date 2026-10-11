# T1 M1 S10

Site: doubt: drop DOUBTED from the caveat map entry

File: `glowcap.ts`
Line: 470 (line numbers are those of the original file)

Line 470, before:

```
        s.evidenceCaveats[ev] = [...(s.evidenceCaveats[ev] ?? []), DOUBTED];
```

Line 470, after:

```
        s.evidenceCaveats[ev] = [...(s.evidenceCaveats[ev] ?? [])];
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
