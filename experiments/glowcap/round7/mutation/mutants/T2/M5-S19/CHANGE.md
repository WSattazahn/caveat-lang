# T2 M5-S19

- File: `glowcap.ts`
- Line: 243
- Site: observe: history 'reopened' because [evId]: add one evidence id the value never reads

Before:

```
      pushBounded(s.decision.history, { change: 'reopened' as const, because: [evId] }, MAX_HISTORY);
```

After:

```
      pushBounded(s.decision.history, { change: 'reopened' as const, because: [evId, 'taste_pit'] }, MAX_HISTORY);
```
