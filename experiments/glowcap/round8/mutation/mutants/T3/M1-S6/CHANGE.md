# T3 M1 S6

Site: taste: caveats ['tasted_in_dark'] -> []

File: `glowcap.ts`
Line: 333 (line numbers are those of the original file)

Line 333, before:

```
      observe(s, ev, kind, s.glow > 0 ? [] : ['tasted_in_dark']);
```

Line 333, after:

```
      observe(s, ev, kind, s.glow > 0 ? [] : []);
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
