# T2 M1 S2

Site: canRetaste: includes(TASTE_FADED) -> false

File: `glowcap.ts`
Line: 198 (line numbers are those of the original file)

Line 198, before:

```
  return taste !== undefined && taste.caveats.includes(TASTE_FADED);
```

Line 198, after:

```
  return taste !== undefined && false;
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
