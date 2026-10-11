# T1 M1 S4

Site: mushroomRule faded: tc.includes(FADED) -> false

File: `glowcap.ts`
Line: 253 (line numbers are those of the original file)

Line 253, before:

```
  const faded = tc.includes(FADED);
```

Line 253, after:

```
  const faded = false;
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
