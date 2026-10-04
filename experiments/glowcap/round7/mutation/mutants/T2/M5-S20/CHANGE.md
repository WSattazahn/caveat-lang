# T2 M5-S20

- File: `glowcap.ts`
- Line: 244
- Site: observe: journal 'R' entry because [evId]: add one evidence id the value never reads

Before:

```
      pushBounded(s.journal, { what: 'R' as const, because: [evId] }, MAX_JOURNAL);
```

After:

```
      pushBounded(s.journal, { what: 'R' as const, because: [evId, 'taste_pit'] }, MAX_JOURNAL);
```
