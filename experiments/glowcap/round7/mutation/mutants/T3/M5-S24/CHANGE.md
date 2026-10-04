# T3 M5-S24

- File: `glowcap.ts`
- Line: 330
- Site: observe: journal 'reopened' entry evidence [clone(ev)]: add one evidence id the value never reads

Before:

```
    pushCapped(s.journal, { type: 'reopened', evidence: [clone(ev)] }, MAX_JOURNAL);
```

After:

```
    pushCapped(s.journal, { type: 'reopened', evidence: [clone(ev), { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }] }, MAX_JOURNAL);
```
