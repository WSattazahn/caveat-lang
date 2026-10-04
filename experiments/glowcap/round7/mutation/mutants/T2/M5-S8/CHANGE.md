# T2 M5-S8

- File: `glowcap.ts`
- Line: 162
- Site: whyNot Too risky untasted: [...s.contradictedBy]: add one evidence id the value never reads

Before:

```
    : m.tasted === null && s.contradictedBy.length >= 2 ? mk('Too risky untasted', [...s.contradictedBy])
```

After:

```
    : m.tasted === null && s.contradictedBy.length >= 2 ? mk('Too risky untasted', [...s.contradictedBy, 'taste_pit'])
```
