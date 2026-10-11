# T3 M1 S3

Site: mushroomRow qualifier: tc.includes('taste_faded') -> false

File: `glowcap.ts`
Line: 156 (line numbers are those of the original file)

Line 156, before:

```
  const qual = tc.includes('taste_faded') ? ' (taste has faded)' : tc.includes('tasted_in_dark') ? ' (tasted in the dark)' : null;
```

Line 156, after:

```
  const qual = false ? ' (taste has faded)' : tc.includes('tasted_in_dark') ? ' (tasted in the dark)' : null;
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
