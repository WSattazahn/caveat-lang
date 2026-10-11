# T2 M4 S5

Site: commit: history entry because reversed

File: `glowcap.ts`
Line: 340 (line numbers are those of the original file)

Line 340, before:

```
  pushBounded(d.history, { change: 'committed', because: [...basis] }, DECISION_KEEP);
```

Line 340, after:

```
  pushBounded(d.history, { change: 'committed', because: [...basis].reverse() }, DECISION_KEEP);
```

Choice: The list is reversed with `.reverse()` (on a copy where the original would otherwise be mutated).
