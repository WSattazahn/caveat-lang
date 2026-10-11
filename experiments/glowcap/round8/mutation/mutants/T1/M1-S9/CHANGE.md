# T1 M1 S9

Site: retaste: caveat map entry [DARK] -> []

File: `glowcap.ts`
Line: 453 (line numbers are those of the original file)

Line 453, before:

```
      s.evidenceCaveats[ev] = s.glow > 0 ? [] : [DARK];
```

Line 453, after:

```
      s.evidenceCaveats[ev] = s.glow > 0 ? [] : [];
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
