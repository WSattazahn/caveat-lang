# T3 M1 S7

Site: retaste: caveats ['tasted_in_dark'] -> []

File: `glowcap.ts`
Line: 343 (line numbers are those of the original file)

Line 343, before:

```
      observe(s, ev, kind, s.glow > 0 ? [] : ['tasted_in_dark']);
```

Line 343, after:

```
      observe(s, ev, kind, s.glow > 0 ? [] : []);
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
