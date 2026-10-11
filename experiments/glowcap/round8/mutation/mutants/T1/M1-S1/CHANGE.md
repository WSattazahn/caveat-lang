# T1 M1 S1

Site: isDoubted: the includes(DOUBTED) caveat test behaves as if the caveat were absent (false)

File: `glowcap.ts`
Line: 130 (line numbers are those of the original file)

Line 130, before:

```
  return (s.evidenceCaveats[ev] ?? []).includes(DOUBTED);
```

Line 130, after:

```
  return false;
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
