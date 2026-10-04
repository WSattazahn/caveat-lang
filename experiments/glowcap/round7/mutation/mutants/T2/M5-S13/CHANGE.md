# T2 M5-S13

- File: `glowcap.ts`
- Line: 188
- Site: mushroomViewBase too risky: because [...s.contradictedBy]: add one evidence id the value never reads

Before:

```
      because: [...s.contradictedBy],
```

After:

```
      because: [...s.contradictedBy, 'taste_pit'],
```
