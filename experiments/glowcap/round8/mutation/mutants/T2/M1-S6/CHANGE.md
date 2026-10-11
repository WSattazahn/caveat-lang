# T2 M1 S6

Site: retaste: caveat list [TASTED_IN_DARK] -> []

File: `glowcap.ts`
Line: 447 (line numbers are those of the original file)

Line 447, before:

```
      const caveats = s.glow > 0 ? [] : [TASTED_IN_DARK];
```

Line 447, after:

```
      const caveats = s.glow > 0 ? [] : [];
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
