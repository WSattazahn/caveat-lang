# T1 M5-S19

- File: `glowcap.ts`
- Line: 204
- Site: whyNot Too risky untasted: [...belief.contradictedBy]: add one evidence id the value never reads

Before:

```
      ? make('Too risky untasted', [...belief.contradictedBy])
```

After:

```
      ? make('Too risky untasted', [...belief.contradictedBy, 'taste_pit'])
```
