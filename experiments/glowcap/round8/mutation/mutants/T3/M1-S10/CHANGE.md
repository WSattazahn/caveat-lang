# T3 M1 S10

Site: save fullToken: includes('tasted_in_dark') flag test -> false

File: `glowcap.ts`
Line: 452 (line numbers are those of the original file)

Line 452, before:

```
  return `${refToken(ev)}${k}${cav.includes('tasted_in_dark') ? '*' : ''}${cav.includes('doubted') ? '!' : ''}`;
```

Line 452, after:

```
  return `${refToken(ev)}${k}${false ? '*' : ''}${cav.includes('doubted') ? '!' : ''}`;
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
