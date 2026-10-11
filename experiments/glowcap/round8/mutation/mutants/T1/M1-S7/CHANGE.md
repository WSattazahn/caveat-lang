# T1 M1 S7

Site: taste: caveat map entry [DARK] -> [] (taste never carries tasted_in_dark)

File: `glowcap.ts`
Line: 392 (line numbers are those of the original file)

Line 392, before:

```
      s.evidenceCaveats[ev] = s.glow > 0 ? [] : [DARK];
```

Line 392, after:

```
      s.evidenceCaveats[ev] = s.glow > 0 ? [] : [];
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
