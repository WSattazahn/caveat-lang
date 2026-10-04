# T2 M5-S18

- File: `glowcap.ts`
- Line: 222
- Site: observe: observation journal entry because [evId]: add one evidence id the value never reads

Before:

```
  pushBounded(s.journal, { what: kind === 'glowcap' ? ('+' as const) : ('-' as const), because: [evId] }, MAX_JOURNAL);
```

After:

```
  pushBounded(s.journal, { what: kind === 'glowcap' ? ('+' as const) : ('-' as const), because: [evId, 'taste_pit'] }, MAX_JOURNAL);
```
