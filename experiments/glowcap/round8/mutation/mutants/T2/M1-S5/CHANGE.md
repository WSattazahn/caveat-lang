# T2 M1 S5

Site: taste: caveat list [TASTED_IN_DARK] -> []

File: `glowcap.ts`
Line: 436 (line numbers are those of the original file)

Line 436, before:

```
      const caveats = s.glow > 0 ? [] : [TASTED_IN_DARK];
```

Line 436, after:

```
      const caveats = s.glow > 0 ? [] : [];
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
