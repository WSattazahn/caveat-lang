# T1 M5-S10

- File: `glowcap.ts`
- Line: 181
- Site: mushroomRow uncertain: because [...belief.contradictedBy]: add one evidence id the value never reads

Before:

```
        because: [...belief.contradictedBy], // CR4: cite the counterexample only
```

After:

```
        because: [...belief.contradictedBy, 'taste_pit'], // CR4: cite the counterexample only
```
