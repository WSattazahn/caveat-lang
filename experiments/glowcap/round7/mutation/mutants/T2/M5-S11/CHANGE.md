# T2 M5-S11

- File: `glowcap.ts`
- Line: 176
- Site: mushroomViewBase tasted glowcap: because [tasteId]: add one evidence id the value never reads

Before:

```
    return { present: true, label: qualifier ? `Probably a glowcap${qualifier}` : 'Glowcap', canAbsorb: true, canTaste: false, because: [tasteId] };
```

After:

```
    return { present: true, label: qualifier ? `Probably a glowcap${qualifier}` : 'Glowcap', canAbsorb: true, canTaste: false, because: [tasteId, 'taste_pit'] };
```
