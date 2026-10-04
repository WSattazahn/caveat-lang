# T3 M5-S22

- File: `glowcap.ts`
- Line: 318
- Site: commit: journal 'committed' entry evidence clone(basis): add one evidence id the value never reads

Before:

```
    pushCapped(s.journal, { type: 'committed', evidence: clone(basis) }, MAX_JOURNAL);
```

After:

```
    pushCapped(s.journal, { type: 'committed', evidence: [...clone(basis), { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }] }, MAX_JOURNAL);
```
