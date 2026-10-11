# T2 M1 S3

Site: mushroomRow qualifier: cav.includes(TASTE_FADED) -> false

File: `glowcap.ts`
Line: 246 (line numbers are those of the original file)

Line 246, before:

```
    const qualifier = cav.includes(TASTE_FADED) ? ' (taste has faded)' : cav.includes(TASTED_IN_DARK) ? ' (tasted in the dark)' : null;
```

Line 246, after:

```
    const qualifier = false ? ' (taste has faded)' : cav.includes(TASTED_IN_DARK) ? ' (tasted in the dark)' : null;
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
