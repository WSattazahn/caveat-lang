# T3 M1 S1

Site: isDoubted: includes('doubted') -> false

File: `glowcap.ts`
Line: 65 (line numbers are those of the original file)

Line 65, before:

```
const isDoubted = (s: State, ev: string): boolean => (s.caveats[ev] ?? []).includes('doubted');
```

Line 65, after:

```
const isDoubted = (s: State, ev: string): boolean => false;
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
