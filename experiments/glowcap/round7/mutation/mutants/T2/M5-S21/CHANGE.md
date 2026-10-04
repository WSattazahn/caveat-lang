# T2 M5-S21

- File: `glowcap.ts`
- Line: 254
- Site: commit: history 'committed' because [...basis]: add one evidence id the value never reads

Before:

```
  pushBounded(s.decision.history, { change: 'committed' as const, because: [...basis] }, MAX_HISTORY);
```

After:

```
  pushBounded(s.decision.history, { change: 'committed' as const, because: [...basis, 'taste_pit'] }, MAX_HISTORY);
```
