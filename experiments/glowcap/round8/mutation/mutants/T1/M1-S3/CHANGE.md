# T1 M1 S3

Site: mushroomRule dark: tc.includes(DARK) -> false

File: `glowcap.ts`
Line: 252 (line numbers are those of the original file)

Line 252, before:

```
  const dark = tc.includes(DARK);
```

Line 252, after:

```
  const dark = false;
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
