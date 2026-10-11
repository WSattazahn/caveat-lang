# T3 M4 S7

Site: commit: history entry because reversed

File: `glowcap.ts`
Line: 274 (line numbers are those of the original file)

Line 274, before:

```
  pushHistory(s, { change: 'committed', because: [...basis] });
```

Line 274, after:

```
  pushHistory(s, { change: 'committed', because: [...basis].reverse() });
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
