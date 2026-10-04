# T1 M5-S6

- File: `glowcap.ts`
- Line: 167
- Site: mushroomRow too risky: because [...belief.contradictedBy]: add one evidence id the value never reads

Before:

```
    return { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: [...belief.contradictedBy] };
```

After:

```
    return { present: true, label: 'Too risky — taste first', canAbsorb: false, canTaste: true, because: [...belief.contradictedBy, 'taste_pit'] };
```
