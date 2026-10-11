# T3 M1 S5

Site: witness: caveats ['secondhand'] -> []

File: `glowcap.ts`
Line: 323 (line numbers are those of the original file)

Line 323, before:

```
      observe(s, ev, kind, ['secondhand']);
```

Line 323, after:

```
      observe(s, ev, kind, []);
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
