# T2 M5-S15

- File: `glowcap.ts`
- Line: 198
- Site: mushroomViewBase probably_unsafe: because [...s.contradictedBy]: add one evidence id the value never reads

Before:

```
    : bs === 'probably_unsafe' ? [...s.contradictedBy]
```

After:

```
    : bs === 'probably_unsafe' ? [...s.contradictedBy, 'taste_pit']
```
