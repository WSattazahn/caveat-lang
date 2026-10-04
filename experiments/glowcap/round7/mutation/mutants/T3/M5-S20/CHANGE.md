# T3 M5-S20

- File: `glowcap.ts`
- Line: 310
- Site: observe: observation journal entry evidence [clone(ev)]: add one evidence id the value never reads

Before:

```
  pushCapped(s.journal, { type: 'observed', evidence: [clone(ev)] }, MAX_JOURNAL); // CR16
```

After:

```
  pushCapped(s.journal, { type: 'observed', evidence: [clone(ev), { id: 'taste_pit', kind: 'glowcap' as Kind, caveats: [] }] }, MAX_JOURNAL); // CR16
```
