# T3 M4 S5

Site: pushHistory: journal trust entry because reversed

File: `glowcap.ts`
Line: 244 (line numbers are those of the original file)

Line 244, before:

```
  pushJournal(s, { entry: entry.change, because: [...entry.because] });
```

Line 244, after:

```
  pushJournal(s, { entry: entry.change, because: [...entry.because].reverse() });
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
