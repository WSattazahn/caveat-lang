# T1 M4 S4

Site: journal trust entry because reversed

File: `glowcap.ts`
Line: 324 (line numbers are those of the original file)

Line 324, before:

```
    pushBounded(s.journal, { text, because: [...change.because] }, JOURNAL_MEMORY);
```

Line 324, after:

```
    pushBounded(s.journal, { text, because: [...change.because].reverse() }, JOURNAL_MEMORY);
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
