# T2 M1 S1

Site: isDoubted: includes(DOUBTED) -> false

File: `glowcap.ts`
Line: 123 (line numbers are those of the original file)

Line 123, before:

```
  return e.caveats.includes(DOUBTED);
```

Line 123, after:

```
  return false;
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
