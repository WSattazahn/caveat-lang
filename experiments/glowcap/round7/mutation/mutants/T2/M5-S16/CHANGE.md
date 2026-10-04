# T2 M5-S16

- File: `glowcap.ts`
- Line: 199
- Site: mushroomViewBase uncertain: because [...s.contradictedBy]: add one evidence id the value never reads

Before:

```
    : bs === 'uncertain' ? [...s.contradictedBy] // CR4: cite the counterexample only
```

After:

```
    : bs === 'uncertain' ? [...s.contradictedBy, 'taste_pit'] // CR4: cite the counterexample only
```
