# T2 M1 S11

Site: save encode: includes(TASTE_FADED) flag test -> false

File: `glowcap.ts`
Line: 618 (line numbers are those of the original file)

Line 618, before:

```
    if (ev.caveats.includes(TASTE_FADED)) flags += 'f';
```

Line 618, after:

```
    if (false) flags += 'f';
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
