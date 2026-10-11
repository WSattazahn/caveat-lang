# T2 M1 S7

Site: witness: caveats [SECONDHAND] -> []

File: `glowcap.ts`
Line: 464 (line numbers are those of the original file)

Line 464, before:

```
      observe(s, { id: evId, kind, caveats: [SECONDHAND] });
```

Line 464, after:

```
      observe(s, { id: evId, kind, caveats: [] });
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
