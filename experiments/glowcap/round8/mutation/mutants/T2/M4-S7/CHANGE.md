# T2 M4 S7

Site: recovery commit argument reversed

File: `glowcap.ts`
Line: 385 (line numbers are those of the original file)

Line 385, before:

```
    return commit(s, s.recovery);
```

Line 385, after:

```
    return commit(s, [...s.recovery].reverse());
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
