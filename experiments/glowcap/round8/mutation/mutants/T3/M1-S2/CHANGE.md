# T3 M1 S2

Site: canRetaste: includes('taste_faded') -> false

File: `glowcap.ts`
Line: 91 (line numbers are those of the original file)

Line 91, before:

```
    && (s.caveats[evId('taste', id, m.life)] ?? []).includes('taste_faded');
```

Line 91, after:

```
    && false;
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
