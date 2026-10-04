# T2 M5-S22

- File: `glowcap.ts`
- Line: 255
- Site: commit: journal 'C' entry because [...basis]: add one evidence id the value never reads

Before:

```
  pushBounded(s.journal, { what: 'C' as const, because: [...basis] }, MAX_JOURNAL);
```

After:

```
  pushBounded(s.journal, { what: 'C' as const, because: [...basis, 'taste_pit'] }, MAX_JOURNAL);
```
