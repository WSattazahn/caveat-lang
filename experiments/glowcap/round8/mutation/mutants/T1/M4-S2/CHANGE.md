# T1 M4 S2

Site: commit: history entry because reversed

File: `glowcap.ts`
Line: 279 (line numbers are those of the original file)

Line 279, before:

```
  const entry: HistoryEntry = { change: 'committed', because: [...basis] };
```

Line 279, after:

```
  const entry: HistoryEntry = { change: 'committed', because: [...basis].reverse() };
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
